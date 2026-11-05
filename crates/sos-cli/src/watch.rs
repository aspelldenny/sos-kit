//! The advisor trigger: fed every shell command an agent ran plus its output, it returns advisor
//! guidance when the same agent's tests have failed `after` times in a row, and nothing otherwise.
//! `sos hook <agent> post-bash` calls it in-process. Port of scripts/test-watch.py (kept for apps
//! whose hooks still call it).
//!
//! Rules (docs/plans/V3_TRIAGE_2026-10-08.md §F):
//!   - only commands matching TEST_RE count (tests and builds), one streak per runner (`xcodebuild test`,
//!     `make check`, `cargo test`…, flags, filters and log paths ignored, so `cargo test -p a` and `-p b`
//!     share one streak); its passing run resets it. A script or heredoc that we cannot classify still
//!     counts when its output carries a known runner's verdict (reading a log with `tail` does not);
//!   - failure = non-zero exit OR failure text in the output (agents pipe through `| tail`, which
//!     hides the exit code);
//!   - after `after` (default 2) consecutive failures, ask the advisor (`sos advise`);
//!   - at most `max` (default 2) advisor calls per failing streak; then tell the agent to stop and
//!     report to Quản đốc instead of calling again;
//!   - an advisor that cannot run (advise exit 3: not logged in, CLI unusable) does not use up a call;
//!     Chủ nhà gets a desktop notification (SOS_NOTIFY_CMD overrides, SOS_NOTIFY=off disables), the
//!     agent is told to report it now, then it stays quiet for ADVISE_UNAVAILABLE_QUIET seconds (default 900);
//!   - settings come from the advisor's (.sos.toml [advisor], env); backend "off" or SOS_ADVISOR=off
//!     disables it. An internal error is reported in one line, never raised.
//!
//! SOS_ADVISE_CMD (a command line) replaces the in-process advisor for `--config` and the call, as
//! SOS_NOTIFY_CMD replaces the notifier: the adapter tests drive the streak logic with a stub.

use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};

const FAIL_RE: &str = r"FAILED \(|^FAILED |^FAIL:|^ERROR:|\*\* TEST FAILED \*\*|Test Suite .* failed|test result: FAILED|Tests? failed|BUILD FAILED|failures?=[1-9]|errors?=[1-9]|\b[1-9]\d* failed";
/// Matched against the start of each command segment (split on ; && || |), after env assignments and
/// wrappers, so `cat pytest.ini` or `grep cargo test` is not a test run.
const TEST_RE: &str = r"(python3? -m )?(pytest|unittest)\b|make (check|test|ready|build)\b|xcodebuild\b|swift (test|build)\b|cargo (test|nextest|build|check)\b|go (test|build)\b|(npm|pnpm|yarn|bun) (run )?(test|build)\b|(npx )?(vitest|jest)\b";
/// A command whose kind we cannot tell from its words: a script or heredoc that may run tests.
const SCRIPT_RE: &str = r"(python3?|bash|sh|zsh|ruby|node|swift)\s+(-\s|-c\b|\S+\.(py|sh|rb|js|swift)\b)|.*<<";
/// Verdict lines that name the runner when such a script ran tests (not when a log is merely read).
const OUTPUT_SUITES: [(&str, &str, &str); 3] = [
    (r"\*\* TEST FAILED \*\*|Testing failed:", r"\*\* TEST SUCCEEDED \*\*", "xcodebuild test"),
    (r"\*\* BUILD FAILED \*\*", r"\*\* BUILD SUCCEEDED \*\*", "xcodebuild build"),
    (r"test result: FAILED", r"test result: ok", "cargo test"),
];
const WRAPPER: &str = r"^(?:\w+=\S*\s+|(?:timeout|gtimeout)\s+\S+\s+|time\s+|(?:uv|poetry|pipenv) run\s+|bundle exec\s+)*";

/// One shell command an agent ran.
pub struct Call<'a> {
    pub key: &'a str,
    pub cmd: &'a str,
    pub exit: Option<i64>,
    pub agent: &'a str,
}

/// POSIX shell words the way Python's shlex reads them (`punct`: `;&|` runs are their own tokens;
/// `comments`: `#` outside quotes ends the line). The tokens read, and false when a quote or an
/// escape is left open (the tokens before it stand).
fn lex(s: &str, punct: bool, comments: bool) -> (Vec<String>, bool) {
    let cs: Vec<char> = s.chars().collect();
    let n = cs.len();
    let ws = |c: char| matches!(c, ' ' | '\t' | '\r' | '\n');
    let is_p = |c: char| punct && matches!(c, ';' | '&' | '|');
    let skip_line = |i: &mut usize| {
        while *i < n && cs[*i] != '\n' {
            *i += 1;
        }
        *i = (*i + 1).min(n);
    };
    let mut out = Vec::new();
    let mut i = 0;
    loop {
        while i < n && ws(cs[i]) {
            i += 1;
        }
        if i >= n {
            return (out, true);
        }
        if comments && cs[i] == '#' {
            skip_line(&mut i);
            continue;
        }
        if is_p(cs[i]) {
            let start = i;
            while i < n && is_p(cs[i]) {
                i += 1;
            }
            out.push(cs[start..i].iter().collect());
            continue;
        }
        let mut tok = String::new();
        let mut quoted = false;
        while i < n {
            let c = cs[i];
            if ws(c) || is_p(c) {
                break;
            }
            if comments && c == '#' {
                skip_line(&mut i);
                break;
            }
            if c == '\'' || c == '"' {
                quoted = true;
                i += 1;
                loop {
                    if i >= n {
                        return (out, false);
                    }
                    let d = cs[i];
                    if d == c {
                        i += 1;
                        break;
                    }
                    if c == '"' && d == '\\' {
                        if i + 1 >= n {
                            return (out, false);
                        }
                        let e = cs[i + 1];
                        if e != '\\' && e != '"' {
                            tok.push('\\');
                        }
                        tok.push(e);
                        i += 2;
                        continue;
                    }
                    tok.push(d);
                    i += 1;
                }
                continue;
            }
            if c == '\\' {
                if i + 1 >= n {
                    return (out, false);
                }
                tok.push(cs[i + 1]);
                i += 2;
                continue;
            }
            tok.push(c);
            i += 1;
        }
        if !tok.is_empty() || quoted {
            out.push(tok);
        }
    }
}

/// PowerShell paths (`node .\scripts\check.mjs`): a backslash between path characters is a separator,
/// which POSIX word rules would otherwise eat.
fn windows_paths(cmd: &str) -> String {
    let cs: Vec<char> = cmd.chars().collect();
    let word = |c: char| c.is_alphanumeric() || c == '_';
    (0..cs.len()).map(|i| {
        let before = i > 0 && (word(cs[i - 1]) || cs[i - 1] == '.' || cs[i - 1] == ':');
        let after = cs.get(i + 1).is_some_and(|&c| word(c) || c == '.');
        if cs[i] == '\\' && before && after { '/' } else { cs[i] }
    }).collect()
}

/// Command segments as word lists, split on ; && || | outside quotes (shell grammar, not a regex).
/// An open quote ends the list: what follows it cannot be classified.
fn token_segments(cmd: &str, windows: bool) -> Vec<Vec<String>> {
    let cmd = if windows { windows_paths(cmd) } else { cmd.to_string() };
    let (toks, closed) = lex(&cmd, true, true);
    let mut segs = vec![Vec::new()];
    for t in toks {
        if !t.is_empty() && t.chars().all(|c| matches!(c, ';' | '&' | '|')) {
            segs.push(Vec::new());
        } else {
            segs.last_mut().expect("never empty").push(t);
        }
    }
    if !closed {
        segs.pop();
    }
    segs
}

/// The words after env assignments and wrappers (`CI=1 timeout 60 x` -> `x`), as WRAPPER does.
fn unwrap(mut words: &[String]) -> &[String] {
    let assignment = |w: &str| w.find('=').is_some_and(|i| i > 0 && w[..i].chars().all(|c| c.is_alphanumeric() || c == '_'));
    while let Some(w) = words.first() {
        let two = words.get(1).map(|s| s.as_str());
        if assignment(w) || w == "time" {
            words = &words[1..];
        } else if w == "timeout" || w == "gtimeout"
            || matches!((w.as_str(), two), ("uv" | "poetry" | "pipenv", Some("run")) | ("bundle", Some("exec"))) {
            words = &words[words.len().min(2)..];
        } else {
            break;
        }
    }
    words
}

/// The streak key of a test segment: its leading words up to the first flag or path
/// (`xcodebuild -scheme X -only-testing:Y test > log` -> `xcodebuild test`), so reruns with other
/// flags, filters or log files continue the same streak.
fn runner(seg: &str) -> Option<String> {
    let words: Vec<&str> = seg.split_whitespace().collect();
    let first = *words.first()?;
    if first == "xcodebuild" && words.iter().any(|w| ["-list", "-version", "-showsdks", "-showBuildSettings", "-showdestinations"].contains(w)) {
        return None;
    }
    let mut head: Vec<&str> = Vec::new();
    for w in &words {
        let limit = if head.len() > 1 && head[1] == "run" { 3 } else { 2 }; // `npm run test`, else `cargo test`
        if w.starts_with('-') || w.chars().any(|c| "/=><.:".contains(c)) || head.len() == limit {
            break;
        }
        head.push(w);
    }
    if words.len() > 2 && words[1] == "-m" {
        return Some(words[..3].join(" ")); // python3 -m pytest
    }
    if head.first() == Some(&"xcodebuild") {
        let test = words.contains(&"test") || words.contains(&"test-without-building");
        return Some(if test { "xcodebuild test" } else { "xcodebuild build" }.into());
    }
    Some(if head.is_empty() { first.to_string() } else { head.join(" ") })
}

/// Python shlex.quote: a word as the shell would need it.
fn quote(w: &str) -> String {
    if w.is_empty() {
        return "''".into();
    }
    if w.chars().all(|c| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c)) {
        return w.into();
    }
    format!("'{}'", w.replace('\'', "'\"'\"'"))
}

/// `.sos.toml` [advisor] tests as word lists: the project's own test commands (POD-agent's
/// `node scripts/check.mjs` never reached the advisor, 23/10/2026). The nearest `.sos.toml` from
/// `dir` up; a broken file or a wrong type is ignored here (`sos check` reports it).
fn project_tests(dir: &Path) -> Vec<Vec<String>> {
    let Some(f) = dir.ancestors().map(|d| d.join(".sos.toml")).find(|f| f.is_file()) else { return Vec::new() };
    let text = String::from_utf8_lossy(&std::fs::read(f).unwrap_or_default()).into_owned();
    let Ok(doc) = text.parse::<toml::Table>() else { return Vec::new() };
    let Some(tests) = doc.get("advisor").and_then(|a| a.get("tests")).and_then(|t| t.as_array()) else { return Vec::new() };
    tests.iter().filter_map(|t| t.as_str()).filter_map(|t| match lex(t, false, false) {
        (words, true) if !words.is_empty() => Some(words),
        _ => None,
    }).collect()
}

/// The streak key of the first test-running segment of the command, or None. `own` are the
/// project's test commands (word lists): a segment whose words start with one is a run of it.
fn test_segment(cmd: &str, own: &[Vec<String>], re: &Res, windows: bool) -> Option<String> {
    for words in token_segments(cmd, windows) {
        let joined = words.join(" ");
        let s = re.wrapper.replace(joined.trim(), "");
        if re.test.is_match(&s) {
            return runner(&s.split_whitespace().collect::<Vec<_>>().join(" "));
        }
        let words = unwrap(&words);
        if let Some(t) = own.iter().find(|t| starts_with(words, t)) {
            return Some(t.iter().map(|w| quote(w)).collect::<Vec<_>>().join(" "));
        }
    }
    None
}

/// `words` run the configured command `t`. `./scripts/x` = `scripts/x`. A bare program name also
/// matches that program by path (`node` = `C:/Program Files/nodejs/node.exe`, from PowerShell's
/// `& "…\node.exe" …`); one given with a directory must match it (`./tools/check.sh` is not
/// `./scripts/check.sh`, review 23/10/2026).
fn starts_with(words: &[String], t: &[String]) -> bool {
    fn norm(w: &str) -> &str {
        w.strip_prefix("./").unwrap_or(w)
    }
    fn name(w: &str) -> String {
        let base = w.rsplit('/').next().unwrap_or(w);
        match base.len().checked_sub(4) {
            Some(i) if base.is_char_boundary(i) && base[i..].eq_ignore_ascii_case(".exe") => base[..i].to_string(),
            _ => base.to_string(),
        }
    }
    if t.is_empty() || words.len() < t.len() {
        return false;
    }
    // Decided before norm(): `./check.sh` names a directory too.
    let same = if t[0].contains('/') { norm(&words[0]) == norm(&t[0]) } else { name(&words[0]) == name(&t[0]) };
    same && words[1..t.len()].iter().zip(&t[1..]).all(|(w, x)| norm(w) == norm(x))
}

/// A script or heredoc whose own output is the only clue that it ran tests.
fn is_script(cmd: &str, re: &Res) -> bool {
    re.script.is_match(&re.wrapper.replace(cmd.trim(), ""))
}

/// (runner, failed) from a known runner's verdict line in the output, or None.
fn suite_from_output(output: &str, re: &Res) -> Option<(String, bool)> {
    for (fail, ok, name) in &re.suites {
        if fail.is_match(output) {
            return Some((name.to_string(), true));
        }
        if ok.is_match(output) {
            return Some((name.to_string(), false));
        }
    }
    None
}

struct Res {
    /// Said with any advice: an override sos could not use (the default stood in for it).
    note: Option<String>,
    fail: Regex,
    test: Regex,
    script: Regex,
    wrapper: Regex,
    suites: Vec<(Regex, Regex, &'static str)>,
}

impl Res {
    /// ADVISE_FAIL_RE / ADVISE_TEST_RE override the defaults (same meaning as in test-watch.py). One
    /// that this regex engine cannot compile (Python lookaround or backreferences) gives the default,
    /// so failures still count, and the advice says so (review 05/11/2026).
    fn load() -> Result<Res> {
        let mut notes = Vec::new();
        let mut pick = |var: &str, default: &str, wrap: &dyn Fn(&str) -> String| -> Result<Regex> {
            if let Ok(v) = std::env::var(var) {
                match Regex::new(&wrap(&v)) {
                    Ok(re) => return Ok(re),
                    Err(_) => notes.push(format!("{var} uses syntax sos cannot read (lookaround or backreferences?); the default pattern was used")),
                }
            }
            Regex::new(&wrap(default)).context(var.to_string())
        };
        let fail = pick("ADVISE_FAIL_RE", FAIL_RE, &|r| format!("(?m){r}"))?;
        let test = pick("ADVISE_TEST_RE", TEST_RE, &|r| format!("^(?:{r})"))?;
        Ok(Res {
            note: (!notes.is_empty()).then(|| format!("(advisor: {})", notes.join("; "))),
            fail,
            test,
            script: Regex::new(&format!("(?s)^(?:{SCRIPT_RE})"))?,
            wrapper: Regex::new(WRAPPER)?,
            suites: OUTPUT_SUITES.iter().map(|(f, o, n)| Ok((Regex::new(f)?, Regex::new(o)?, *n))).collect::<Result<_>>()?,
        })
    }
}

const WIN_TOAST: &str = concat!(
    "$n=[Windows.UI.Notifications.ToastNotificationManager,Windows.UI.Notifications,ContentType=WindowsRuntime];",
    "$x=$n::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02);",
    "$t=$x.GetElementsByTagName('text');",
    "[void]$t.Item(0).AppendChild($x.CreateTextNode($env:SOS_TITLE));",
    "[void]$t.Item(1).AppendChild($x.CreateTextNode($env:SOS_TEXT));",
    "$n::CreateToastNotifier('{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe')",
    ".Show([Windows.UI.Notifications.ToastNotification]::new($x))"
);

/// A command line as argv. Windows: quotes group words and are dropped, backslashes stay
/// (C:\tools\x.exe); elsewhere POSIX words.
fn split_cmd(s: &str) -> Vec<String> {
    if !cfg!(windows) {
        return lex(s, false, false).0;
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;
    for c in s.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => cur.push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                started = true;
            }
            None if c.is_whitespace() => {
                if started || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                started = false;
            }
            None => cur.push(c),
        }
    }
    if started || !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Tell the human directly: an agent may not pass the message on (Chủ nhà, 20/10/2026: "có lỗi phải
/// báo, không được im lặng"). Never fails: the message to the worker comes after this.
fn notify(text: &str) {
    if std::env::var("SOS_NOTIFY").as_deref() == Ok("off") {
        return;
    }
    let title = "SOS Kit: advisor cannot run";
    let have = |p: &str| crate::kit::find_program(p);
    let mut cmd = if let Some(custom) = std::env::var("SOS_NOTIFY_CMD").ok().filter(|c| !c.is_empty()) {
        let argv = split_cmd(&custom);
        let Some((prog, args)) = argv.split_first() else { return };
        let mut c = Command::new(crate::kit::find_program(prog).unwrap_or_else(|| prog.into()));
        c.args(args).arg(format!("{title}: {text}"));
        c
    } else if cfg!(target_os = "macos") && have("osascript").is_some() {
        let q = |v: &str| serde_json::to_string(v).unwrap_or_default();
        let mut c = Command::new("osascript");
        c.args(["-e", &format!("display notification {} with title {} sound name \"Basso\"", q(text), q(title))]);
        c
    } else if let Some(p) = have("notify-send") {
        let mut c = Command::new(p);
        c.args(["-u", "critical", title, text]);
        c
    } else if cfg!(windows) && have("powershell").is_some() {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-NonInteractive", "-Command", WIN_TOAST]);
        c
    } else {
        return;
    };
    cmd.env("SOS_TITLE", title).env("SOS_TEXT", text);
    let _ = crate::hook::run_limited(&mut cmd, b"", Duration::from_secs(10));
}

fn head(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn tail(s: &str, n: usize) -> String {
    let count = s.chars().count();
    s.chars().skip(count.saturating_sub(n)).collect()
}

/// The advisor's settings: SOS_ADVISE_CMD `--config`, else in-process.
fn advisor_settings(root: &Path) -> (String, i64, i64) {
    let Some(cmd) = std::env::var("SOS_ADVISE_CMD").ok().filter(|c| !c.is_empty()) else {
        let s = crate::advise::settings(root);
        return (s.backend, s.after, s.max);
    };
    let o = external(&cmd, &["--config".to_string()], root, crate::advise::limit(Duration::from_secs(30)));
    let v: serde_json::Value = serde_json::from_str(o.stdout.trim()).unwrap_or_default();
    let backend = v.get("backend").and_then(|b| b.as_str()).unwrap_or("claude").to_string();
    let int = |k: &str| v.get(k).and_then(|n| n.as_i64()).unwrap_or(2);
    (backend, int("after"), int("max"))
}

fn external(cmd: &str, args: &[String], root: &Path, limit: Duration) -> crate::advise::Outcome {
    let argv = split_cmd(cmd);
    let Some((prog, rest)) = argv.split_first() else {
        return crate::advise::Outcome { code: 1, stdout: String::new(), stderr: "SOS_ADVISE_CMD is empty".into() };
    };
    let mut c = Command::new(crate::kit::find_program(prog).unwrap_or_else(|| prog.into()));
    c.args(rest).args(args).current_dir(root);
    match crate::role::capture(&mut c, String::new(), limit) {
        Ok(r) => crate::advise::Outcome { code: r.status.and_then(|s| s.code()).unwrap_or(1), stdout: r.stdout, stderr: r.stderr },
        Err(e) => crate::advise::Outcome { code: 1, stdout: String::new(), stderr: format!("could not start SOS_ADVISE_CMD: {e}") },
    }
}

fn read_count(p: &Path) -> i64 {
    std::fs::read_to_string(p).ok().and_then(|t| t.trim().parse().ok()).unwrap_or(0)
}

/// Advisor guidance for this command, or "" (the advisor never blocks; errors come back as one line).
pub fn run(root: &Path, call: &Call, output: &str) -> String {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| watch(root, call, output))) {
        Ok(Ok(text)) => text,
        Ok(Err(e)) => format!("(advisor hook error, continuing without advice: {e:#})\n"),
        Err(_) => "(advisor hook error, continuing without advice: internal panic)\n".into(),
    }
}

/// Everything the watcher runs ends within this (the hook's own limit is 320 s).
pub const BUDGET: Duration = Duration::from_secs(290);

fn watch(start: &Path, call: &Call, output: &str) -> Result<String> {
    if std::env::var("SOS_ADVISOR").as_deref() == Ok("off") {
        return Ok(String::new());
    }
    // No-op under `sos hook`, which set it when the hook started (the first deadline stands).
    crate::advise::set_deadline(std::time::Instant::now() + BUDGET);
    let re = Res::load()?;
    let text = watch_with(start, call, output, &re)?;
    Ok(match &re.note {
        Some(n) if !text.is_empty() => format!("{n}\n{text}"),
        _ => text,
    })
}

fn watch_with(start: &Path, call: &Call, output: &str, re: &Res) -> Result<String> {
    let (seg, failed) = match test_segment(call.cmd, &project_tests(start), re, cfg!(windows)) {
        Some(seg) => (seg, call.exit.is_some_and(|c| c != 0) || re.fail.is_match(output)),
        None => match is_script(call.cmd, re).then(|| suite_from_output(output, re)).flatten() {
            Some(found) => found,
            None => return Ok(String::new()),
        },
    };
    let root = crate::advise::repo_root(start);
    let (backend, after, max) = advisor_settings(&root);
    if backend == "off" {
        return Ok(String::new());
    }
    let state = root.join(".advise-state");
    std::fs::create_dir_all(&state).with_context(|| format!("creating {}", state.display()))?;
    // One streak per runner, so another suite passing does not hide this failure.
    let safe: String = call.key.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).take(100).collect();
    let digest = Sha256::digest(seg.as_bytes());
    let key = format!("{safe}-{}", digest.iter().take(4).map(|b| format!("{b:02x}")).collect::<String>());
    let (count_f, calls_f) = (state.join(format!("{key}.fails")), state.join(format!("{key}.calls")));
    if !failed {
        let _ = std::fs::remove_file(&count_f);
        let _ = std::fs::remove_file(&calls_f);
        return Ok(String::new());
    }
    let fails = read_count(&count_f) + 1;
    std::fs::write(&count_f, fails.to_string())?;
    if fails < after {
        return Ok(String::new());
    }
    let quiet: i64 = std::env::var("ADVISE_UNAVAILABLE_QUIET").ok().and_then(|q| q.trim().parse().ok()).unwrap_or(900);
    let down = state.join("unavailable"); // one marker for the repo: a lost login breaks every session
    if let Ok(m) = std::fs::metadata(&down).and_then(|m| m.modified()) {
        let age = SystemTime::now().duration_since(m).map(|d| d.as_secs_f64()).unwrap_or(0.0);
        if age < quiet as f64 {
            return Ok(String::new());
        }
    }
    let calls = read_count(&calls_f);
    if calls >= max {
        if calls == max {
            // Say it once, then stay quiet for the rest of the streak.
            std::fs::write(&calls_f, (calls + 1).to_string())?;
            return Ok(format!("ADVISOR LIMIT: the advisor was already consulted {max} times while these tests kept failing. \
                               Stop changing code; report to Quản đốc what you tried, the failing evidence and your best hypothesis.\n"));
        }
        return Ok(String::new());
    }
    std::fs::write(&calls_f, (calls + 1).to_string())?;
    std::fs::write(&count_f, "0")?; // next advice needs another `after` failures
    let err = state.join(format!("{key}.last-error.txt"));
    std::fs::write(&err, tail(output, 8000))?;
    let q = format!("The same tests have now failed {fails} times in a row (command: {}). \
                     What is the root cause, and should I change approach?", call.cmd);
    let err_s = err.to_string_lossy().into_owned();
    let r = match std::env::var("SOS_ADVISE_CMD").ok().filter(|c| !c.is_empty()) {
        Some(cmd) => external(&cmd, &["--agent".into(), call.agent.into(), "--error-file".into(), err_s, q], &root,
                              crate::advise::limit(Duration::from_secs(270))),
        None => crate::advise::ask(&root, &crate::advise::Ask {
            agent: Some(call.agent.to_string()), error_file: err_s, question: vec![q], ..Default::default()
        }),
    };
    let stderr = r.stderr.trim();
    if r.code == 3 {
        std::fs::write(&calls_f, calls.to_string())?; // not a consultation: give the call back
        std::fs::write(&down, head(stderr, 500))?;
        let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        notify(&format!("{name}: {}", tail(stderr, 200)));
        return Ok(format!("ADVISOR UNAVAILABLE (no advisor call used): {}\n\
                           Tell Chủ nhà now, in your next message, that the advisor needs this fix; \
                           do not leave it for the end of your report. Then continue on your own evidence.\n", head(stderr, 400)));
    }
    let _ = std::fs::remove_file(&down);
    if r.code == 0 {
        return Ok(format!("ADVISOR (automatic, after repeated test failures). Weigh it against your own evidence; \
                           if you disagree, say why in your report.\n\n{}\n", r.stdout.trim()));
    }
    Ok(format!("(advisor unavailable: {} — continue on your own evidence)\n", head(stderr, 300)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn seg(cmd: &str, own: &[Vec<String>], windows: bool) -> Option<String> {
        test_segment(cmd, own, &Res::load().unwrap(), windows)
    }

    #[test]
    fn lexer_reads_like_python_shlex() {
        assert_eq!(lex("a 'b c' \"d\\\"e\\x\" f\\ g", false, false), (words(&["a", "b c", "d\"e\\x", "f g"]), true));
        assert_eq!(lex("a&&b|c;d 2>&1 # x; y\nz", true, true), (words(&["a", "&&", "b", "|", "c", ";", "d", "2>", "&", "1", "z"]), true));
        assert_eq!(lex("a#b c", true, true).0, words(&["a"]));
        assert_eq!(lex("ok; 'open", true, true), (words(&["ok", ";"]), false));
        assert_eq!(lex("'' x", false, false).0, words(&["", "x"]));
    }

    #[test]
    fn runners_share_a_streak_across_flags() {
        let s = |c: &str| seg(c, &[], false);
        assert_eq!(s("cargo test -p a").as_deref(), Some("cargo test"));
        assert_eq!(s("CI=1 timeout 60 cargo test -p b 2>&1 | tail -20").as_deref(), Some("cargo test"));
        assert_eq!(s("cd x && npm run test -- --watch=false").as_deref(), Some("npm run test"));
        assert_eq!(s("python3 -m pytest -k x").as_deref(), Some("python3 -m pytest"));
        assert_eq!(s("xcodebuild -scheme X -only-testing:Y test > log").as_deref(), Some("xcodebuild test"));
        assert_eq!(s("xcodebuild -scheme X build").as_deref(), Some("xcodebuild build"));
        assert_eq!(s("xcodebuild -list"), None);
        assert_eq!(s("cat pytest.ini"), None);
        assert_eq!(s("grep cargo test"), None);
        assert_eq!(s("echo 'cargo test"), None);
    }

    #[test]
    fn scripts_count_only_by_a_runner_verdict() {
        let re = Res::load().unwrap();
        assert!(is_script("bash run.sh", &re) && is_script("python3 - <<EOF\nx\nEOF", &re) && !is_script("tail -50 log.txt", &re));
        assert_eq!(suite_from_output("test result: FAILED. 1 passed", &re), Some(("cargo test".into(), true)));
        assert_eq!(suite_from_output("** TEST SUCCEEDED **", &re), Some(("xcodebuild test".into(), false)));
        assert_eq!(suite_from_output("all good", &re), None);
    }

    #[test]
    fn project_test_commands_match_as_test_watch_did() {
        // tests/adapters/test_watch_units.py, case for case.
        let own = vec![words(&["node", "scripts/check.mjs"]), words(&["node", "scripts/my check.mjs"]), words(&["./scripts/check.sh"])];
        let mut cases: Vec<(&str, Option<&str>, bool)> = vec![
            ("node scripts/check.mjs", Some("node scripts/check.mjs"), false),
            ("CI=1 timeout 60 node scripts/check.mjs --full", Some("node scripts/check.mjs"), false),
            ("cd x && node scripts/check.mjs", Some("node scripts/check.mjs"), false),
            ("node \"scripts/check.mjs --other\"", None, false),
            ("node \"scripts/my check.mjs\"", Some("node 'scripts/my check.mjs'"), false),
            ("node scripts/check.mjsx", None, false),
            ("node ./scripts/check.mjs", Some("node scripts/check.mjs"), false),
            ("\"/opt/node/bin/node\" scripts/check.mjs", Some("node scripts/check.mjs"), false),
            ("cat scripts/check.mjs", None, false),
            ("scripts/check.sh --fast", Some("./scripts/check.sh"), false),
            ("./scripts/check.sh", Some("./scripts/check.sh"), false),
            ("./tools/check.sh", None, false), // same file name, another script (review 23/10/2026)
            ("check.sh", None, false),
        ];
        // PowerShell spellings (Codex on Windows, review 23/10/2026): backslashes read as paths.
        cases.extend([
            (r"node .\scripts\check.mjs", Some("node scripts/check.mjs"), true),
            (r#"& "C:\Program Files\nodejs\node.exe" scripts\check.mjs"#, Some("node scripts/check.mjs"), true),
            (r"cd E:\Extension; node .\scripts\check.mjs --full", Some("node scripts/check.mjs"), true),
            (r"node scripts\check.mjsx", None, true),
        ]);
        for (cmd, want, windows) in cases {
            assert_eq!(seg(cmd, &own, windows).as_deref(), want, "{cmd}");
        }
        let own2 = vec![words(&["./check.sh"]), words(&["node.exe", "tools/run.mjs"])];
        for (cmd, want) in [
            ("./check.sh", Some("./check.sh")),
            ("./tools/check.sh", None), // `./check.sh` names its directory (review 23/10/2026)
            ("node.exe tools/run.mjs", Some("node.exe tools/run.mjs")),
            ("node tools/run.mjs", Some("node.exe tools/run.mjs")),
            ("\"/opt/node/bin/node.exe\" tools/run.mjs", Some("node.exe tools/run.mjs")),
        ] {
            assert_eq!(seg(cmd, &own2, false).as_deref(), want, "{cmd}");
        }
    }

    #[test]
    fn project_tests_come_from_the_nearest_sos_toml() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join("sub")).unwrap();
        std::fs::write(d.path().join(".sos.toml"), "[advisor] # note\ntests = [ # \"not this\"\n  \"x ] y\", 'lit # z', 3, \"unclosed '\" ]\n").unwrap();
        assert_eq!(project_tests(&d.path().join("sub")), vec![words(&["x", "]", "y"]), words(&["lit", "#", "z"])]);
        std::fs::write(d.path().join(".sos.toml"), "[advisor]\ntests = \"not a list\"\n").unwrap();
        assert!(project_tests(d.path()).is_empty());
    }

    #[test]
    fn notify_command_lines_split_per_platform() {
        if cfg!(windows) {
            assert_eq!(split_cmd(r#""C:\tools\my n.exe" -x"#), words(&[r"C:\tools\my n.exe", "-x"]));
        } else {
            assert_eq!(split_cmd("/bin/echo 'a b' c"), words(&["/bin/echo", "a b", "c"]));
        }
    }

    #[test]
    fn windows_paths_turn_only_separators() {
        assert_eq!(windows_paths(r"node .\scripts\check.mjs"), "node ./scripts/check.mjs");
        assert_eq!(windows_paths(r#"echo "a\"b" x\\y"#), r#"echo "a\"b" x\\y"#);
    }
}
