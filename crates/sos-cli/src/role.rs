//! `sos role <worker|reviewer|architect> --from <claude|codex> --brief <file>`: run one harness
//! role on the model `.sos.toml` [roles] names for it, from any agent.
//!
//! Why it exists (TurnSigil, 09–14/10/2026): a background `codex exec` with stdin open waited
//! 40 minutes for input; every app wrote its own model table and runner. Here the prompt goes
//! in on stdin and stdin is closed, the run has a wall-clock limit, the final message lands in
//! one file, and the model comes from one table that `sos check` compares with the wrappers.

use anyhow::{bail, Context, Result};
use sos_gates::config::{self, RoleSpec};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct Args {
    pub role: String,
    pub from: String,
    pub brief: PathBuf,
    pub out: Option<PathBuf>,
    pub timeout: Option<u64>,
    pub dry_run: bool,
}

const DEFAULT_TIMEOUT: u64 = 1800;

/// The spec for `role` when work was built by `from`, and where it came from.
pub fn resolve(cfg: &config::Config, role: &str, from: &str) -> Result<RoleSpec> {
    match cfg.roles.get(role).and_then(|e| e.for_family(from)) {
        Some(s) => Ok(s.clone()),
        None => bail!(
            "no model for `{role}` when {from} built the work: add [roles.{role}] (or [roles.{role}.{from}]) \
             with backend, model and effort to .sos.toml (see harness-lite/templates/sos.toml)"
        ),
    }
}

fn state_excerpt(root: &Path, file: &str) -> String {
    let Ok(text) = std::fs::read_to_string(root.join(file)) else { return String::new() };
    let mut out = Vec::new();
    let mut seen = 0;
    for line in text.lines() {
        if line.starts_with("## ") {
            seen += 1;
            if seen > 1 {
                break;
            }
        }
        if seen == 1 {
            out.push(line);
        }
    }
    let s = out.join("\n");
    if s.is_empty() { String::new() } else { format!("Current state ({file}, first section):\n{}\n\n", head(&s, 2500)) }
}

/// The first `n` bytes of `s`, cut on a character boundary (state docs are often Vietnamese).
fn head(s: &str, n: usize) -> &str {
    let mut end = s.len().min(n);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// The last `n` bytes of `s`, cut on a character boundary.
fn tail(s: &str, n: usize) -> &str {
    let mut start = s.len().saturating_sub(n);
    while !s.is_char_boundary(start) {
        start += 1;
    }
    &s[start..]
}

fn prompt(root: &Path, role: &str, spec: &RoleSpec, state: &str, brief: &str) -> String {
    let rules = match role {
        "reviewer" => "You are Người soát. Read only: do not edit files. If you cannot run the product here, \
                       say so and label your review static. Rank each finding blocking or non-blocking.",
        "architect" => "You are Kiến trúc sư. Change only the design document the brief names; no product code.",
        _ => "You are Thợ. Work only in this worktree; run your checks in the foreground before you hand back. \
              If the sandbox blocks `git commit`, leave the changes uncommitted and say so.",
    };
    format!(
        "Read AGENTS.md, harness-lite/CONTRACT.md and harness-lite/roles/{role}.md, then do the brief.\n{rules}\n\
         Repository: {}\nModel: {} / effort {}\n\n{state}Brief:\n{brief}\n",
        root.display(), spec.model, spec.effort
    )
}

fn command(root: &Path, role: &str, spec: &RoleSpec, out: &Path) -> Vec<String> {
    let s = |v: &str| v.to_string();
    if spec.backend == "codex" {
        let sandbox = if !spec.sandbox.is_empty() { spec.sandbox.as_str() }
                      else if role == "reviewer" { "read-only" } else { "workspace-write" };
        let mut c = vec![s("codex"), s("exec"), s("-m"), spec.model.clone(),
                         s("-c"), format!("model_reasoning_effort=\"{}\"", spec.effort)];
        if !spec.service_tier.is_empty() {
            c.extend([s("-c"), format!("service_tier=\"{}\"", spec.service_tier)]);
        }
        c.extend([s("-s"), s(sandbox), s("-C"), root.display().to_string(),
                  s("-o"), out.display().to_string(), s("-")]);
        c
    } else {
        let tools = match role {
            "reviewer" => "Read,Glob,Grep,Bash(git diff:*),Bash(git log:*),Bash(git show:*)",
            "architect" => "Read,Glob,Grep,Edit,Write",
            _ => "Read,Glob,Grep,Edit,Write,Bash",
        };
        let mut c = vec![s("claude"), s("-p"), s("--model"), spec.model.clone(), s("--effort"), spec.effort.clone(),
                         s("--output-format"), s("json"), s("--no-session-persistence"), s("--strict-mcp-config")];
        let base = if role == "reviewer" { "Read,Glob,Grep,Bash" } else { tools };
        c.extend([s("--tools"), s(base), s("--allowedTools"), s(tools), s("--permission-mode"), s("dontAsk")]);
        c
    }
}

fn in_worktree(root: &Path) -> bool {
    let q = |a: &str| Command::new("git").current_dir(root).args(["rev-parse", "--path-format=absolute", a]).output()
        .ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let (dir, common) = (q("--git-dir"), q("--git-common-dir"));
    !dir.is_empty() && dir != common
}

pub fn run(a: &Args) -> Result<i32> {
    if !config::ROLES.contains(&a.role.as_str()) {
        bail!("role must be one of {}", config::ROLES.join(", "));
    }
    if !["claude", "codex"].contains(&a.from.as_str()) {
        bail!("--from is the family that built the work: claude or codex");
    }
    let root = crate::kit::repo_root(Path::new("."))?;
    let cfg = config::load(&root)?;
    let spec = resolve(&cfg, &a.role, &a.from)?;
    let brief = std::fs::read_to_string(&a.brief).with_context(|| format!("reading brief {}", a.brief.display()))?;
    let stamp = chrono_stamp();
    let out = a.out.clone().unwrap_or_else(|| root.join("evidence/roles").join(format!("{stamp}-{}-{}.md", a.role, spec.backend)));
    let text = prompt(&root, &a.role, &spec, &state_excerpt(&root, &cfg.state.file), &brief);
    let cmd = command(&root, &a.role, &spec, &out);
    let timeout = a.timeout.or(spec.timeout_secs).unwrap_or(DEFAULT_TIMEOUT);
    eprintln!("sos role: {} for {}-built work → {} {} (effort {}), timeout {timeout}s, output {}",
              a.role, a.from, spec.backend, spec.model, spec.effort, out.display());
    if a.dry_run {
        println!("{}", serde_json::to_string(&cmd)?);
        return Ok(0);
    }
    if a.role == "worker" && !in_worktree(&root) {
        bail!("a worker run edits files: start it inside its own git worktree (git worktree add …), not the main checkout");
    }
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let started = Instant::now();
    let mut c = Command::new(&cmd[0]);
    c.args(&cmd[1..]).current_dir(&root).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(unix)]
    {
        // Own process group, so a timeout also stops the CLI's children (they hold the pipes).
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    let mut child = c.spawn().with_context(|| format!("{} CLI not found on PATH (no fallback to another model family)", cmd[0]))?;
    // Write the prompt and close stdin (dropped at the end of the thread), so the CLI never waits
    // for more input; a thread, so a CLI that does not read stdin cannot block us before the timer.
    let mut stdin = child.stdin.take().expect("piped stdin");
    let prompt_t = std::thread::spawn(move || { let _ = stdin.write_all(text.as_bytes()); });
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let (out_tx, out_rx) = std::sync::mpsc::channel();
    let (err_tx, err_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || { let mut s = String::new(); let _ = stdout.read_to_string(&mut s); let _ = out_tx.send(s); });
    std::thread::spawn(move || { let mut s = String::new(); let _ = stderr.read_to_string(&mut s); let _ = err_tx.send(s); });
    let status = loop {
        if let Some(st) = child.try_wait()? {
            break Some(st);
        }
        if started.elapsed() > Duration::from_secs(timeout) {
            kill_group(child.id());
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(200));
    };
    // Anything the CLI left running in its group (a server it started) would hold our pipes open:
    // stop the group, then wait for the readers only briefly.
    kill_group(child.id());
    drop(prompt_t);
    let so = out_rx.recv_timeout(Duration::from_secs(5)).unwrap_or_default();
    let se = err_rx.recv_timeout(Duration::from_secs(5)).unwrap_or_default();
    let secs = started.elapsed().as_secs();
    let mut ok = status.is_some_and(|s| s.success());
    // Claude prints JSON on stdout; Codex writes its last message to `out` itself.
    let mut report = if spec.backend == "claude" {
        let v = serde_json::from_str::<serde_json::Value>(so.lines().last().unwrap_or("")).unwrap_or_default();
        if v.get("is_error").and_then(|e| e.as_bool()) == Some(true) {
            ok = false;
        }
        v.get("result").and_then(|r| r.as_str()).unwrap_or_default().to_string()
    } else {
        std::fs::read_to_string(&out).unwrap_or_default()
    };
    if spec.backend == "claude" && ok && !report.is_empty() {
        std::fs::write(&out, &report)?;
    }
    log(&root, &a.role, &a.from, &spec, secs, ok);
    match status {
        None => {
            eprintln!("sos role: stopped after {timeout}s with no result (raise timeout_secs in [roles.{}] or narrow the brief)", a.role);
            Ok(124)
        }
        Some(_) if !ok || report.trim().is_empty() => {
            if report.trim().is_empty() {
                report = format!("{}\n{}", se.trim(), so.trim());
            }
            eprintln!("sos role: {} failed after {secs}s:\n{}", spec.backend, tail(&report, 2000));
            Ok(1)
        }
        Some(_) => {
            println!("{report}");
            eprintln!("sos role: done in {secs}s; report in {}", out.display());
            Ok(0)
        }
    }
}

/// SIGKILL the process group led by `pid` (the CLI and whatever it started).
fn kill_group(pid: u32) {
    #[cfg(unix)]
    // SAFETY: kill(2) with a negative pid signals that process group; no memory is touched.
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
    }
    #[cfg(not(unix))]
    let _ = pid;
}

fn log(root: &Path, role: &str, from: &str, s: &RoleSpec, secs: u64, ok: bool) {
    let dir = root.join(".advise-state");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let line = serde_json::json!({"ts": chrono_stamp(), "kind": "role", "role": role, "from": from,
        "backend": s.backend, "model": s.model, "effort": s.effort, "seconds": secs, "ok": ok});
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("calls.jsonl")) {
        let _ = writeln!(f, "{line}");
    }
}

/// Local time as YYYYmmdd-HHMMSS without a date crate: ask `date`.
fn chrono_stamp() -> String {
    Command::new("date").arg("+%Y%m%d-%H%M%S").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty()).unwrap_or_else(|| "now".into())
}
