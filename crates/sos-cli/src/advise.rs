//! `sos advise`: ask a stronger model for direction on the current work. Agent-neutral.
//!
//!   sos advise [--backend claude|codex|auto] [--model M] [--agent claude|codex] [--error-file F] "question"
//!   sos advise --config    print the resolved advisor settings as JSON
//!   sos advise --check     one tiny call to prove the configured advisor answers (after fixing a login)
//!
//! Settings, strongest first: command-line flags, env (ADVISE_BACKEND, ADVISE_MODEL, ADVISE_AFTER,
//! ADVISE_MAX, ADVISE_EFFORT, ADVISE_SERVICE_TIER, SOS_STATE_FILE, SOS_ADVISOR=off), `.sos.toml`
//! [advisor] and [state] file, defaults (backend claude, after 2, max 2). Read leniently: a broken
//! `.sos.toml` gives the defaults here (`sos check` and the gates report it), never a crash in a hook.
//! `auto` asks the other model family than the agent at work (a different eye), falling back to the
//! same family when the other CLI is missing; a missing chosen CLI also falls back, and says so.
//!
//! The advisor gets a snapshot, not the transcript: the question, the intent (ADVISE_BRIEF, else the
//! first section of the state doc and the open features in docs/FEATURES.json), the latest failure,
//! git status, recent commits, the uncommitted diff, and the files the failure and the change mention
//! (or the last commit's files when the tree is clean). Output: advice on stdout; a copy in
//! evidence/advice/; one JSON line per call in .advise-state/calls.jsonl (on failure the error and,
//! when known, the fix). Exit 0 advice, 1 failed, 2 off or misconfigured, 3 unavailable (the CLI is
//! not logged in or not usable); the test watcher does not count an unavailable call against `max`.
//! Port of scripts/advise (kept for apps whose hooks still call it).

use regex::Regex;
use serde_json::{json, Value};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const SYSTEM: &str = "You are a senior engineer advising a coding agent. You see a snapshot of its work; \
you do not edit files. Follow the requested answer format exactly.";
const FORMAT: &str = "You are the advisor to a worker agent in this repository. You do not edit files.
Give direction, not code dumps. Be concrete and short (under 300 words). Answer in exactly these parts:
VERDICT: continue | change approach | stop and escalate
WHY: the root cause or key insight, citing file:line or test names
NEXT: 1-3 concrete steps for the worker
RISKS: what the worker is likely missing (edge cases, other callers, data)
CHECK: the single check that would prove the next step worked";
const CODE_EXT: &str = "(?:py|swift|rs|ts|tsx|js|jsx|go|kt|java|rb|c|cc|cpp|h|m|sql|toml|json|ya?ml|md)";
const TIMEOUT: u64 = 240;
/// Each git call that builds the snapshot (a hung textconv or external diff must not hold the hook).
const GIT_LIMIT: Duration = Duration::from_secs(20);

/// When the whole call must be over (the hook sets it; a manual `sos advise` has none).
static DEADLINE: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Bound everything this process still runs for the advisor by `at` (first call wins).
pub fn set_deadline(at: Instant) {
    let _ = DEADLINE.set(at);
}

/// `cap`, shortened to what is left before the deadline.
pub fn limit(cap: Duration) -> Duration {
    DEADLINE.get().map_or(cap, |d| cap.min(d.saturating_duration_since(Instant::now())))
}

/// The resolved advisor settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub backend: String,
    pub model: String,
    pub after: i64,
    pub max: i64,
    pub effort: String,
    pub service_tier: String,
    pub state: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { backend: "claude".into(), model: String::new(), after: 2, max: 2, effort: String::new(),
                   service_tier: String::new(), state: "docs/BACKLOG.md".into() }
    }
}

impl Settings {
    /// The JSON `--config` prints (Python json.dumps layout, which older readers expect).
    pub fn to_json(&self) -> String {
        let q = |v: &str| serde_json::to_string(v).unwrap_or_default();
        format!("{{\"backend\": {}, \"model\": {}, \"after\": {}, \"max\": {}, \"effort\": {}, \"service_tier\": {}, \"state\": {}}}",
                q(&self.backend), q(&self.model), self.after, self.max, q(&self.effort), q(&self.service_tier), q(&self.state))
    }
}

/// `.sos.toml` [advisor] / [state] file, then env. A key of the wrong type keeps its default.
pub fn settings(root: &Path) -> Settings {
    let mut s = Settings::default();
    let doc = std::fs::read_to_string(root.join(".sos.toml")).ok().and_then(|t| t.parse::<toml::Table>().ok());
    if let Some(doc) = doc {
        if let Some(a) = doc.get("advisor").and_then(|a| a.as_table()) {
            let text = |k: &str, into: &mut String| {
                if let Some(v) = a.get(k).and_then(|v| v.as_str()) {
                    *into = v.to_string();
                }
            };
            text("backend", &mut s.backend);
            text("model", &mut s.model);
            text("effort", &mut s.effort);
            text("service_tier", &mut s.service_tier);
            if let Some(n) = a.get("after").and_then(|v| v.as_integer()) {
                s.after = n;
            }
            if let Some(n) = a.get("max").and_then(|v| v.as_integer()) {
                s.max = n;
            }
        }
        if let Some(f) = doc.get("state").and_then(|t| t.get("file")).and_then(|v| v.as_str()) {
            s.state = f.to_string();
        }
    }
    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    for (var, into) in [("ADVISE_BACKEND", &mut s.backend), ("ADVISE_MODEL", &mut s.model), ("ADVISE_EFFORT", &mut s.effort),
                        ("ADVISE_SERVICE_TIER", &mut s.service_tier), ("SOS_STATE_FILE", &mut s.state)] {
        if let Some(v) = env(var) {
            *into = v;
        }
    }
    if let Some(n) = env("ADVISE_AFTER").and_then(|v| v.trim().parse().ok()) {
        s.after = n;
    }
    if let Some(n) = env("ADVISE_MAX").and_then(|v| v.trim().parse().ok()) {
        s.max = n;
    }
    if std::env::var("SOS_ADVISOR").as_deref() == Ok("off") {
        s.backend = "off".into();
    }
    s
}

/// One advisor request.
#[derive(Default)]
pub struct Ask {
    pub backend: Option<String>,
    pub model: Option<String>,
    pub agent: Option<String>,
    pub error_file: String,
    pub question: Vec<String>,
    pub check: bool,
}

/// What the call printed and its exit code, so the test watcher can use it without a process.
pub struct Outcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

fn fail(code: i32, msg: String) -> Outcome {
    Outcome { code, stdout: String::new(), stderr: msg + "\n" }
}

/// The git top level of `dir`, else `dir`.
pub fn repo_root(dir: &Path) -> PathBuf {
    let top = git(dir, &["rev-parse", "--show-toplevel"]);
    let top = top.trim_end_matches(['\n', '\r']);
    if top.is_empty() { dir.to_path_buf() } else { dir.join(top) }
}

/// git's stdout, or "" when it fails or runs past GIT_LIMIT (it is stopped).
fn git(dir: &Path, args: &[&str]) -> String {
    let mut c = Command::new("git");
    c.args(args).current_dir(dir);
    crate::role::capture(&mut c, String::new(), limit(GIT_LIMIT)).ok()
        .filter(|o| o.status.is_some_and(|s| s.success()))
        .map(|o| o.stdout).unwrap_or_default()
}

/// (backend, note); the note explains a fallback. None when neither CLI is on PATH.
fn pick_backend(wanted: &str, agent: &str) -> Result<(String, String), String> {
    let other = |b: &str| match b { "claude" => "codex", "codex" => "claude", _ => "" }.to_string();
    let order = if wanted == "auto" {
        let first = match other(agent).as_str() { "" => "codex".to_string(), o => o.to_string() };
        let second = other(&first);
        [first, second]
    } else {
        let second = match other(wanted).as_str() { "" => wanted.to_string(), o => o.to_string() };
        [wanted.to_string(), second]
    };
    for b in &order {
        if (b == "claude" || b == "codex") && crate::kit::find_program(b).is_some() {
            let note = if *b == order[0] { String::new() } else { format!("({} CLI not found; used {b})", order[0]) };
            return Ok((b.clone(), note));
        }
    }
    Err("neither claude nor codex CLI is on PATH".into())
}

/// Thirty and TurnSigil, 15–17/10/2026: 7 of 11 automatic calls failed in 2 s with "OAuth session
/// expired" and nobody noticed. Name the fix so the session-start status can show it. Matched only
/// against the last lines, the CLI's own error: `codex exec` echoes the whole prompt on stderr first.
fn unavailable_fix(backend: &str, text: &str) -> String {
    let auth = Regex::new(r"(?i)failed to authenticate|oauth (session|token)|not logged in|log ?in again|please (run )?\S*\s?login|unauthori[sz]ed|\b401\b|invalid api key")
        .expect("valid regex");
    let lines: Vec<&str> = text.trim().lines().collect();
    let tail = lines[lines.len().saturating_sub(6)..].join("\n");
    if !auth.is_match(&tail) {
        return String::new();
    }
    match backend {
        "claude" => "run `claude` in a terminal and /login (the hook uses the CLI's own login)",
        "codex" => "run `codex login`",
        _ => "",
    }.to_string()
}

fn head(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn tail(s: &str, n: usize) -> String {
    let count = s.chars().count();
    s.chars().skip(count.saturating_sub(n)).collect()
}

/// A failing test file brings its subject: test_pay.py -> pay.py, PayTests.swift -> Pay.swift.
fn test_subject(base: &str) -> String {
    let b = base.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    let followed_by_dot = |j: usize| b.get(j) == Some(&b'.');
    while i < b.len() {
        if i == 0 && b.starts_with(b"test_") {
            i += 5;
            continue;
        }
        let rest = &b[i..];
        let cut = [&b"_test"[..], b"_spec", b".test", b".spec", b"Tests", b"Test"].iter()
            .find(|p| rest.starts_with(p) && followed_by_dot(i + p.len())).map(|p| p.len());
        match cut {
            Some(n) => i += n,
            None => {
                out.push(b[i]);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| base.to_string())
}

fn relevant_files(root: &Path, failure: &str, diff: &str) -> Vec<PathBuf> {
    let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let named = Regex::new(&format!(r"([\w./-]+\.{CODE_EXT})\b")).expect("valid regex");
    names.extend(named.captures_iter(failure).map(|c| c[1].to_string()));
    let changed = Regex::new(r"(?m)^\+\+\+ b/(\S+)").expect("valid regex");
    names.extend(changed.captures_iter(diff).map(|c| c[1].to_string()));
    if diff.is_empty() {
        // Clean tree: the last commit is the work in progress.
        names.extend(git(root, &["show", "--name-only", "--format=", "HEAD"]).split_whitespace().map(str::to_string));
    }
    let tracked = git(root, &["ls-files"]);
    let tracked: Vec<&str> = tracked.split_whitespace().collect();
    let base = |n: &str| n.rsplit(['/', '\\']).next().unwrap_or(n).to_string();
    for n in names.clone() {
        let b = base(&n);
        let subject = test_subject(&b);
        if subject != b {
            names.extend(tracked.iter().filter(|t| base(t) == subject).map(|t| t.to_string()));
        }
    }
    let Ok(top) = root.canonicalize() else { return Vec::new() };
    let mut out = Vec::new();
    for n in &names {
        let mut rel = n.as_str();
        while let Some(r) = rel.strip_prefix("./").or_else(|| rel.strip_prefix('/')) {
            rel = r;
        }
        let Ok(p) = root.join(rel).canonicalize() else { continue };
        let Ok(inside) = p.strip_prefix(&top) else { continue };
        let skip = inside.components().any(|c| matches!(c, Component::Normal(s) if s == "evidence" || s == ".advise-state"));
        if !skip && p.is_file() && p.metadata().is_ok_and(|m| m.len() < 200_000) {
            out.push(inside.to_path_buf());
        }
    }
    out.truncate(8);
    out
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

/// The first `## ` section of a markdown document, heading included.
fn first_section(text: &str) -> Option<String> {
    let mut lines = text.split_inclusive('\n');
    let mut out = String::new();
    for l in lines.by_ref() {
        if l.starts_with("## ") {
            out.push_str(l);
            break;
        }
    }
    if out.is_empty() {
        return None;
    }
    for l in lines {
        if l.starts_with("## ") {
            break;
        }
        out.push_str(l);
    }
    Some(out)
}

/// What the work is for. Without it the advisor can only say why a test is red, not what to build.
/// A relative ADVISE_BRIEF is read from `base`: the caller's directory (the hook's is the project root).
fn intent(root: &Path, base: &Path, state: &str) -> String {
    if let Some(brief) = std::env::var_os("ADVISE_BRIEF").filter(|b| !b.is_empty()).map(|b| base.join(b)).filter(|b| b.is_file()) {
        let text = String::from_utf8_lossy(&std::fs::read(brief).unwrap_or_default()).into_owned();
        return format!("\n## Brief\n{}", head(&text, 6000));
    }
    let mut found = Vec::new();
    let doc = root.join(state);
    if doc.is_file() {
        let text = String::from_utf8_lossy(&std::fs::read(&doc).unwrap_or_default()).into_owned();
        if let Some(sec) = first_section(&text) {
            found.push(format!("From {state} (first section, the project's state doc):\n{}", head(sec.trim(), 3000)));
        }
    }
    let features = root.join("docs").join("FEATURES.json");
    if features.is_file() {
        let v: Value = std::fs::read_to_string(&features).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null);
        let list = v.get("features").and_then(|f| f.as_array()).cloned().unwrap_or_default();
        let open: Vec<Value> = if list.iter().all(|f| f.is_object()) {
            list.into_iter().filter(|f| !truthy(f.get("passes"))).collect()
        } else {
            Vec::new()
        };
        if !open.is_empty() {
            let mut buf = Vec::new();
            let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
            if serde::Serialize::serialize(&open, &mut ser).is_ok() {
                found.push(format!("Open features from docs/FEATURES.json:\n{}", head(&String::from_utf8_lossy(&buf), 3000)));
            }
        }
    }
    if !found.is_empty() {
        return format!("\n## Intent\n{}", found.join("\n\n"));
    }
    "\n## Intent\n(none found: no ADVISE_BRIEF, state doc ([state] file) or open docs/FEATURES.json entry. \
     Do not guess what the product must do; say which intent your advice depends on.)".into()
}

fn shown(p: &Path) -> String {
    p.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/")
}

fn build_prompt(root: &Path, base: &Path, question: &str, errfile: &str, state: &str) -> String {
    let mut parts = vec![FORMAT.to_string(), format!("\n## Worker's question\n{question}"), intent(root, base, state)];
    let failure = if !errfile.is_empty() && base.join(errfile).is_file() {
        tail(&String::from_utf8_lossy(&std::fs::read(base.join(errfile)).unwrap_or_default()), 8000)
    } else {
        String::new()
    };
    if !failure.is_empty() {
        parts.push(format!("\n## Latest failure\n{failure}"));
    }
    let excl = [":(exclude)evidence", ":(exclude).advise-state"];
    let status = git(root, &["status", "--short", "--", ".", excl[0], excl[1]]);
    parts.push(format!("\n## git status\n{}", status.lines().take(50).collect::<Vec<_>>().join("\n")));
    parts.push(format!("\n## Recent commits\n{}", git(root, &["log", "--oneline", "-5"])));
    let diff = git(root, &["diff", "HEAD", "--", ".", excl[0], excl[1]]);
    let shown_diff = if diff.is_empty() { "(none: the tree is clean)".to_string() } else { head(&diff, 30_000) };
    parts.push(format!("\n## Uncommitted diff (truncated)\n{shown_diff}"));
    let mut budget: i64 = 40_000;
    for rel in relevant_files(root, &failure, &diff) {
        let body = head(&String::from_utf8_lossy(&std::fs::read(root.join(&rel)).unwrap_or_default()), 12_000);
        let len = body.chars().count() as i64;
        if budget - len < 0 {
            break;
        }
        budget -= len;
        parts.push(format!("\n## File {}\n```\n{body}\n```", shown(&rel)));
    }
    parts.join("\n")
}

struct Reply {
    ok: bool,
    text: String,
    cost: Option<f64>,
    model: String,
}

fn call(root: &Path, backend: &str, model: &str, prompt: String, effort: &str, tier: &str) -> Reply {
    // Windows: npm installs `claude.cmd`/`codex.cmd`, which Command::new("codex") never finds.
    let program = crate::kit::find_program(backend).unwrap_or_else(|| PathBuf::from(backend));
    let mut c = Command::new(&program);
    c.current_dir(root);
    if backend == "claude" {
        let model = if model.is_empty() { "opus" } else { model };
        c.args(["-p", "--model", model]);
        if !effort.is_empty() {
            c.args(["--effort", effort]);
        }
        c.args(["--tools", "", "--output-format", "json", "--strict-mcp-config", "--disable-slash-commands",
                "--setting-sources", "", "--system-prompt", SYSTEM]);
    } else {
        c.arg("exec");
        if !model.is_empty() {
            c.args(["-m", model]);
        }
        if !effort.is_empty() {
            c.args(["-c", &format!("model_reasoning_effort=\"{effort}\"")]);
        }
        if !tier.is_empty() {
            c.args(["-c", &format!("service_tier=\"{tier}\"")]);
        }
        c.args(["--sandbox", "read-only", "--skip-git-repo-check", "-"]);
    }
    let default_model = if model.is_empty() { if backend == "claude" { "opus" } else { "codex default" } } else { model };
    let budget = limit(Duration::from_secs(TIMEOUT));
    let r = match crate::role::capture(&mut c, prompt, budget) {
        Ok(r) => r,
        Err(e) => return Reply { ok: false, text: format!("could not start the {backend} CLI: {e}"), cost: None, model: model.to_string() },
    };
    let Some(status) = r.status else {
        return Reply { ok: false, text: format!("timed out after {} s", budget.as_secs()), cost: None, model: model.to_string() };
    };
    if backend == "claude" {
        let last = r.stdout.trim().lines().last().unwrap_or("");
        match serde_json::from_str::<Value>(last) {
            Ok(Value::Object(d)) => {
                let used: Vec<String> = d.get("modelUsage").and_then(|m| m.as_object()).map(|m| m.keys().cloned().collect()).unwrap_or_default();
                let text = match d.get("result") { Some(Value::String(s)) => s.clone(), None | Some(Value::Null) => String::new(), Some(v) => v.to_string() };
                Reply {
                    ok: status.success() && !truthy(d.get("is_error")),
                    text,
                    cost: d.get("total_cost_usd").and_then(|c| c.as_f64()),
                    model: if used.is_empty() { default_model.to_string() } else { used.join(",") },
                }
            }
            _ => {
                let raw = if r.stderr.is_empty() { &r.stdout } else { &r.stderr };
                Reply { ok: false, text: tail(raw.trim(), 500), cost: None, model: default_model.to_string() }
            }
        }
    } else {
        let ok = status.success() && !r.stdout.trim().is_empty();
        let text = if status.success() { r.stdout.trim() } else { r.stderr.trim() }.to_string();
        Reply { ok, text, cost: None, model: default_model.to_string() }
    }
}

/// Ask the advisor from directory `start` (relative --error-file and ADVISE_BRIEF are read from
/// there). Never panics on bad input; every failure is an exit code and a message.
pub fn ask(start: &Path, a: &Ask) -> Outcome {
    let root = repo_root(start);
    let s = settings(&root);
    let wanted = a.backend.clone().unwrap_or_else(|| s.backend.clone());
    if wanted == "off" {
        return fail(2, "advise: the advisor is off (.sos.toml [advisor] backend or SOS_ADVISOR)".into());
    }
    if !["claude", "codex", "auto"].contains(&wanted.as_str()) {
        return fail(2, format!("advise: unknown backend '{wanted}' (claude | codex | auto | off)"));
    }
    let (backend, note) = match pick_backend(&wanted, a.agent.as_deref().unwrap_or("")) {
        Ok(b) => b,
        Err(note) => return fail(1, format!("advise: {note}")),
    };
    let model = a.model.clone().unwrap_or_else(|| s.model.clone());
    let question = if a.check {
        "--check".to_string()
    } else if a.question.is_empty() {
        "What is the most likely root cause, and what should change next?".to_string()
    } else {
        a.question.join(" ")
    };
    let prompt = if a.check { "Reply with the single word OK.".to_string() } else { build_prompt(&root, start, &question, &a.error_file, &s.state) };
    let started = Instant::now();
    let r = call(&root, &backend, &model, prompt, if a.check { "" } else { &s.effort }, &s.service_tier);
    let fix = if r.ok { String::new() } else { unavailable_fix(&backend, &r.text) };
    let secs = (started.elapsed().as_secs_f64() * 10.0).round() / 10.0;
    // A record that could not be kept is said, never claimed (review 05/11/2026).
    let mut lost: Vec<String> = Vec::new();
    let state = root.join(".advise-state");
    let mut line = json!({"ts": crate::role::local_time("%Y-%m-%dT%H:%M:%S"), "backend": backend, "model": r.model,
        "agent": a.agent, "seconds": secs, "ok": r.ok, "cost_usd": r.cost, "question": head(&question, 160)});
    if !r.ok {
        line["error"] = json!(head(r.text.trim(), 200));
        line["fix"] = json!(fix);
    }
    let logged = std::fs::create_dir_all(&state).and_then(|_| {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(state.join("calls.jsonl"))?;
        writeln!(f, "{line}")
    });
    if let Err(e) = logged {
        lost.push(format!("advise: could not log the call in .advise-state/calls.jsonl: {e}"));
    }
    let fix_line = if fix.is_empty() { String::new() } else { format!("\nFix: {fix}") };
    if a.check {
        if r.ok {
            // The login is fixed: let the hook call again at once.
            match std::fs::remove_file(state.join("unavailable")) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound =>
                    lost.push(format!("advise: could not remove .advise-state/unavailable ({e}); the hook stays quiet until it expires")),
                _ => {}
            }
            let notes = lost.iter().map(|l| format!("{l}\n")).collect::<String>();
            return Outcome { code: 0, stdout: format!("advise --check: {backend} {} answers{fix_line}\n{notes}", r.model), stderr: String::new() };
        }
        let notes = lost.iter().map(|l| format!("\n{l}")).collect::<String>();
        let msg = format!("advise --check: {backend} {} failed — {}{fix_line}{notes}", r.model, head(r.text.trim(), 300));
        return fail(if fix.is_empty() { 1 } else { 3 }, msg);
    }
    let dir = root.join("evidence").join("advice");
    let name = format!("{}-{backend}.md", crate::role::local_time("%Y%m%d-%H%M%S"));
    let body = format!("# advice ({backend} {}, {secs}s, ok={})\n\n**Q:** {question}\n\n{}\n", r.model, if r.ok { "True" } else { "False" }, r.text);
    let rec = match std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(dir.join(&name), body)) {
        Ok(()) => format!("evidence/advice/{name}"),
        Err(e) => {
            lost.push(format!("advise: could not keep the advice in evidence/advice/{name}: {e}"));
            "not kept".to_string()
        }
    };
    let notes = lost.iter().map(|l| format!("{l}\n")).collect::<String>();
    if !r.ok {
        let then = if fix.is_empty() { String::new() } else { format!(". Fix: {fix}, then `sos advise --check`") };
        let see = if rec == "not kept" { String::new() } else { format!(" (see {rec})") };
        return fail(if fix.is_empty() { 1 } else { 3 }, format!("advise: {backend} failed — {}{see}{then}\n{notes}", head(r.text.trim(), 300)).trim_end().to_string());
    }
    let cost = match r.cost { Some(c) if c != 0.0 => format!(", ${c:.3}"), _ => String::new() };
    let note = if note.is_empty() { note } else { note + " " };
    let kept = if rec == "not kept" { "advice not kept".to_string() } else { format!("logged {rec}") };
    Outcome { code: 0, stdout: format!("{}\n\n{note}(advisor: {backend} {}, {secs}s{cost}; {kept})\n{notes}", r.text, r.model), stderr: String::new() }
}

#[derive(clap::Args)]
#[command(trailing_var_arg = true)]
pub struct Args {
    /// claude | codex | auto (the other family than --agent); default from .sos.toml [advisor].
    #[arg(long)]
    backend: Option<String>,
    /// Model for the backend (default from .sos.toml [advisor], else the CLI's own).
    #[arg(long)]
    model: Option<String>,
    /// The agent asking (claude | codex), for `auto`.
    #[arg(long)]
    agent: Option<String>,
    /// A file with the latest failure output (its last 8000 characters go in the prompt).
    #[arg(long, default_value = "")]
    error_file: String,
    /// Print the resolved advisor settings as JSON and exit.
    #[arg(long)]
    config: bool,
    /// One tiny call to prove the configured advisor answers (after fixing a login).
    #[arg(long)]
    check: bool,
    /// The question (words are joined with spaces).
    #[arg(allow_hyphen_values = true)]
    question: Vec<String>,
}

pub fn run(a: Args) -> anyhow::Result<i32> {
    let cwd = std::env::current_dir()?;
    if a.config {
        println!("{}", settings(&repo_root(&cwd)).to_json());
        return Ok(0);
    }
    let o = ask(&cwd, &Ask { backend: a.backend, model: a.model, agent: a.agent, error_file: a.error_file, question: a.question, check: a.check });
    print!("{}", o.stdout);
    eprint!("{}", o.stderr);
    Ok(o.code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_files_bring_their_subject() {
        for (base, want) in [("test_pay.py", "pay.py"), ("pay_test.go", "pay.go"), ("pay_spec.rb", "pay.rb"), ("pay.test.ts", "pay.ts"),
                             ("pay.spec.js", "pay.js"), ("PayTests.swift", "Pay.swift"), ("PayTest.kt", "Pay.kt"), ("pay.py", "pay.py"),
                             ("contest.py", "contest.py"), ("Testing.swift", "Testing.swift")] {
            assert_eq!(test_subject(base), want, "{base}");
        }
    }

    #[test]
    fn auth_failures_name_the_fix_only_from_the_last_lines() {
        assert!(unavailable_fix("claude", "Error: OAuth session expired").contains("/login"));
        assert_eq!(unavailable_fix("codex", "HTTP 401 Unauthorized"), "run `codex login`");
        let echoed = format!("prompt mentions not logged in\n{}", "line\n".repeat(6));
        assert_eq!(unavailable_fix("codex", &echoed), "");
        assert_eq!(unavailable_fix("claude", "assertion failed"), "");
    }

    #[test]
    fn first_section_stops_at_the_next_heading() {
        assert_eq!(first_section("# T\nintro\n## A\none\n### sub\n## B\ntwo\n").as_deref(), Some("## A\none\n### sub\n"));
        assert_eq!(first_section("# only a title\n"), None);
    }

    #[test]
    fn settings_read_sos_toml_leniently() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join(".sos.toml"), "[advisor]\nbackend = \"codex\"\nafter = 3\nmax = \"x\"\n[state]\nfile = \"docs/STATE.md\"\n").unwrap();
        let s = settings(d.path());
        assert_eq!((s.backend.as_str(), s.after, s.max, s.state.as_str()), ("codex", 3, 2, "docs/STATE.md"));
        std::fs::write(d.path().join(".sos.toml"), "[advisor\nbroken").unwrap();
        assert_eq!(settings(d.path()).after, 2);
    }
}
