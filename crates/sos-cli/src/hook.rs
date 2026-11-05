//! `sos hook` — the agent hooks in Rust (port of adapters/{claude,codex}/hook.py, scripts/env-guard.sh,
//! scripts/sim-guard and scripts/status.sh; plan: docs/plans/RUST_ALL_TOOLS_2026-11-04.md, wave 1a).
//!
//!   sos hook claude|codex session-start   status lines (Claude: stdout; Codex: additionalContext)
//!   sos hook claude|codex pre-edit        refuse edits of real .env files
//!   sos hook claude|codex pre-bash        refuse xcodebuild test runs that clone the simulator
//!   sos hook claude|codex post-bash       advisor trigger: still scripts/test-watch.py (wave 2)
//!   sos hook env-guard <path>…  |  sim-guard --cmd <command>  |  status   agent-neutral forms
//!
//! Blocking: exit 2 with the reason on stderr. Codex on Windows may run hooks under
//! `powershell -Command`, which turns any nonzero exit into 1 (does not block), so there it answers
//! with a PreToolUse "deny" JSON and exit 0. An unreadable payload on a guard blocks. No policy here
//! beyond what the Python adapters had; the git pre-commit stays the backstop for every agent.

use clap::{Subcommand, ValueEnum};
use serde_json::Value;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::{sim_guard, status};

#[derive(Clone, Copy, PartialEq, ValueEnum)]
pub enum Event {
    SessionStart,
    PreEdit,
    PreBash,
    PostBash,
}

#[derive(Subcommand)]
pub enum HookCmd {
    /// Claude Code hook (payload on stdin): session-start, pre-edit, pre-bash, post-bash.
    Claude {
        event: Event,
        /// Hook protocol the caller speaks (event names, output shape). This binary runs only the
        /// versions it knows; an older one rejects the flag, which the launcher turns into a block.
        #[arg(long, default_value_t = PROTOCOL)]
        protocol: u32,
    },
    /// Codex hook (payload on stdin): session-start, pre-edit, pre-bash, post-bash.
    Codex {
        event: Event,
        #[arg(long, default_value_t = PROTOCOL)]
        protocol: u32,
    },
    /// Exit 2 when any path is a real .env file (.env.example is allowed).
    EnvGuard { paths: Vec<String> },
    /// Exit 2 when the shell command would start an xcodebuild test run that clones the simulator.
    SimGuard {
        #[arg(long, allow_hyphen_values = true)]
        cmd: String,
    },
    /// Print the session-start status of the repo in the current directory.
    Status,
}

/// The `sos hook <agent> <event>` contract version. Append-only: names, events and this flag are
/// only ever added; a change to the payload or output shape is a new number (docs/plans/RUST_ARCHITECTURE_2026-11-05.md §3).
pub const PROTOCOL: u32 = 1;

fn protocol_ok(p: u32) -> bool {
    if p == PROTOCOL {
        return true;
    }
    eprintln!("BLOCKED: this hook speaks protocol {p}, but sos {} knows only protocol {PROTOCOL}.\n\
               How to fix: install the sos version that matches the wiring (https://github.com/aspelldenny/sos-kit#quick-start), \
               or run `sos update --wiring` to rewrite the hook commands for this sos.", env!("CARGO_PKG_VERSION"));
    false
}

/// A guard that has not decided within this time blocks. The agent's own hook timeout counts as
/// "continue" (Claude: a timed-out command hook does not block), so the decision must come from
/// here, well inside the `timeout` the templates give the guard entries (60 s; review 05/11/2026 1b, point 1).
pub const GUARD_DEADLINE: Duration = Duration::from_secs(30);

fn guard_deadline(event: Event, codex: bool) {
    if !matches!(event, Event::PreEdit | Event::PreBash) {
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(GUARD_DEADLINE);
        let msg = format!("BLOCKED: SOS Kit hook did not decide within {} s, so the action is blocked. Retry; if it repeats, \
                           run the same `sos hook` by hand to see where it hangs.", GUARD_DEADLINE.as_secs());
        let code = if codex { codex_block(&msg) } else {
            eprintln!("{msg}");
            2
        };
        std::process::exit(code);
    });
}

pub fn run(cmd: HookCmd) -> i32 {
    match cmd {
        HookCmd::Claude { protocol, .. } | HookCmd::Codex { protocol, .. } if !protocol_ok(protocol) => 2,
        HookCmd::Claude { event, .. } => {
            guard_deadline(event, false);
            claude(event)
        }
        HookCmd::Codex { event, .. } => {
            guard_deadline(event, true);
            codex(event)
        }
        HookCmd::EnvGuard { paths } => match env_guard(&paths) {
            Some(msg) => {
                eprintln!("{msg}");
                2
            }
            None => 0,
        },
        HookCmd::SimGuard { cmd } => match sim_guard::check(&cmd) {
            Some(msg) => {
                eprintln!("{msg}");
                2
            }
            None => 0,
        },
        HookCmd::Status => {
            print!("{}", status::render(Path::new(".")));
            0
        }
    }
}

/// The block message when any path is a real .env file. Case-insensitive (macOS and Windows
/// filesystems are). The name is checked both as POSIX reads it (a backslash is part of a file name,
/// so `.env.local\notes.txt` is one name) and as Windows does (a backslash separates), so neither
/// form can hide it. A path with a
/// NUL byte is refused: no tool can write it, and the Python guard crashed on it, which blocked.
pub fn env_guard(paths: &[String]) -> Option<String> {
    let is_env = |base: &str| {
        let base = base.to_ascii_lowercase();
        base != ".env.example" && (base == ".env" || base.starts_with(".env."))
    };
    if let Some(p) = paths.iter().find(|p| p.contains('\0')) {
        return Some(format!("BLOCKED: a path to edit contains a NUL byte: {:?}\n\
                             How to fix: re-issue the edit with a plain file path.", p));
    }
    let blocked: Vec<&str> = paths.iter().map(String::as_str).filter(|p| {
        let p = p.trim_end_matches('/');
        is_env(p.rsplit('/').next().unwrap_or("")) || is_env(p.rsplit(['/', '\\']).next().unwrap_or(""))
    }).collect();
    if blocked.is_empty() {
        return None;
    }
    Some(format!(
        "BLOCKED: agents must not edit real .env files: {}\n\
         Why: .env holds live secrets; an agent edit puts them in its context and logs.\n\
         How to fix: edit .env.example (placeholders only) and ask Chủ nhà to put the real value in .env by hand.",
        blocked.join(" ")
    ))
}

/// Every path named on any '*** <header>: <path>' line of a Codex apply_patch. Deliberately broader
/// than the V4A grammar: Codex trims any whitespace (incl. Unicode) around headers, so a narrow match
/// can be bypassed (review 2026-10-09). Split on \n only: Codex keeps U+2028 inside a path.
fn patch_paths(patch: &str) -> Vec<String> {
    let ws = |c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c);
    patch.split('\n').filter_map(|line| {
        let s = line.trim_matches(ws);
        if !s.starts_with("***") {
            return None;
        }
        let path = s.split_once(':')?.1.trim_matches(ws);
        (!path.is_empty()).then(|| path.to_string())
    }).collect()
}

fn payload() -> Option<serde_json::Map<String, Value>> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf).ok()?;
    match serde_json::from_slice(&buf).ok()? {
        Value::Object(m) => Some(m),
        _ => None,
    }
}

fn s<'a>(d: &'a serde_json::Map<String, Value>, k: &str) -> &'a str {
    d.get(k).and_then(Value::as_str).unwrap_or("")
}

fn tool_input(d: &serde_json::Map<String, Value>) -> serde_json::Map<String, Value> {
    d.get("tool_input").and_then(Value::as_object).cloned().unwrap_or_default()
}

/// Python json.dumps layout (`, ` and `: `), which the adapter tests and any log reader expect.
fn context_json(event: &str, text: &str) -> String {
    let q = |v: &str| serde_json::to_string(v).unwrap_or_default();
    format!("{{\"hookSpecificOutput\": {{\"hookEventName\": {}, \"additionalContext\": {}}}}}", q(event), q(text))
}

fn git_top(dir: &Path) -> Option<PathBuf> {
    let o = Command::new("git").current_dir(dir).args(["rev-parse", "--show-toplevel"]).output().ok()?;
    o.status.success().then(|| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim_end_matches(['\n', '\r'])))
}

/// Claude: $CLAUDE_PROJECT_DIR; Codex (and fallback): the git top level, else the current directory.
/// Always absolute: the advisor script runs with this as its working directory, so a relative
/// script path found from here would point elsewhere (review 05/11/2026).
fn project_root(claude: bool) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if claude {
        if let Some(d) = std::env::var_os("CLAUDE_PROJECT_DIR").filter(|d| !d.is_empty()) {
            return cwd.join(d);
        }
    }
    git_top(&cwd).map(|t| cwd.join(t)).unwrap_or(cwd)
}

/// The kit's scripts/ for this project: $SOS_KIT_DIR, an app's harness-lite/, or the kit repo itself.
fn scripts_dir(root: &Path) -> Option<PathBuf> {
    let mut cands: Vec<PathBuf> = std::env::var_os("SOS_KIT_DIR").map(|d| vec![PathBuf::from(d)]).unwrap_or_default();
    cands.push(root.join("harness-lite"));
    cands.push(root.to_path_buf());
    let cwd = std::env::current_dir().unwrap_or_default();
    cands.into_iter().map(|k| cwd.join(k).join("scripts")).find(|s| s.join("test-watch.py").is_file())
}

/// $SOS_PYTHON, else the first of python3/python that runs (Windows: `python3` is often the Store
/// alias that only fails; the Python adapters used their own interpreter, which always ran).
fn python() -> String {
    if let Some(p) = std::env::var("SOS_PYTHON").ok().filter(|p| !p.is_empty()) {
        return p;
    }
    let order: &[&str] = if cfg!(windows) { &["python", "python3"] } else { &["python3", "python"] };
    order.iter().find(|p| run_limited(Command::new(p).args(["-c", ""]), b"", Duration::from_secs(10))
        .is_some_and(|(ok, _)| ok)).unwrap_or(&order[0]).to_string()
}

/// Run `cmd` with `input` on stdin, within `limit` for the process AND for reading its output (a
/// descendant holding stdout open must not hang the hook). (success, stdout), or None when it could
/// not start or ran out of time (it is killed).
pub fn run_limited(cmd: &mut Command, input: &[u8], limit: Duration) -> Option<(bool, Vec<u8>)> {
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let input = input.to_vec();
        std::thread::spawn(move || {
            let _ = stdin.write_all(&input);
        });
    }
    let (tx, rx) = std::sync::mpsc::channel();
    if let Some(mut out) = child.stdout.take() {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = out.read_to_end(&mut buf);
            let _ = tx.send(buf);
        });
    }
    let deadline = Instant::now() + limit;
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    let out = rx.recv_timeout(deadline.saturating_duration_since(Instant::now())).ok()?;
    Some((status.success(), out))
}

/// scripts/test-watch.py with `out` on stdin; its stdout, or nothing (the advisor never blocks).
fn test_watch(root: &Path, args: &[String], out: &str) -> String {
    let Some(scripts) = scripts_dir(root) else { return String::new() };
    let py = python();
    let mut cmd = Command::new(&py);
    cmd.arg(scripts.join("test-watch.py")).args(args).current_dir(root)
        .env("PYTHONUTF8", "1").env("SOS_PYTHON", &py);
    run_limited(&mut cmd, out.as_bytes(), Duration::from_secs(300))
        .map(|(_, o)| String::from_utf8_lossy(&o).into_owned()).unwrap_or_default()
}

fn claude(event: Event) -> i32 {
    let root = project_root(true);
    let unreadable = || {
        eprintln!("BLOCKED: sos hook claude could not read the hook payload.");
        2
    };
    match event {
        Event::SessionStart => {
            print!("{}", status::render(&root));
            0
        }
        Event::PreEdit => {
            let Some(d) = payload() else { return unreadable() };
            let ti = tool_input(&d);
            let paths: Vec<String> = ["file_path", "notebook_path"].iter()
                .map(|k| s(&ti, k).to_string()).filter(|p| !p.is_empty()).collect();
            match env_guard(&paths) {
                Some(msg) => {
                    eprintln!("{msg}");
                    2
                }
                None => 0,
            }
        }
        Event::PreBash => {
            let Some(d) = payload() else { return unreadable() };
            if s(&d, "tool_name") != "Bash" {
                return 0;
            }
            match sim_guard::check(s(&tool_input(&d), "command")) {
                Some(msg) => {
                    eprintln!("{msg}");
                    2
                }
                None => 0,
            }
        }
        Event::PostBash => {
            let Some(d) = payload() else { return 0 };
            if s(&d, "tool_name") != "Bash" {
                return 0;
            }
            let mut out = match d.get("tool_response") {
                Some(Value::Object(r)) => format!("{}\n{}", s(r, "stdout"), s(r, "stderr")),
                None | Some(Value::Null) => "\n".into(),
                Some(Value::String(t)) => t.clone(),
                Some(other) => other.to_string(),
            };
            let event = match s(&d, "hook_event_name") { "" => "PostToolUse", e => e }.to_string();
            let failure = event == "PostToolUseFailure";
            if failure {
                out = format!("{}\n{out}", s(&d, "error"));
            }
            let agent = match s(&d, "agent_id") { "" => "main", a => a };
            let mut args = vec![
                "--key".into(), format!("claude-{}-{agent}", s(&d, "session_id")),
                "--cmd".into(), s(&tool_input(&d), "command").to_string(),
                "--agent".into(), "claude".into(),
            ];
            if failure {
                args.extend(["--exit".into(), "1".into()]);
            }
            let text = test_watch(&root, &args, &out);
            if !text.trim().is_empty() {
                println!("{}", context_json(&event, text.trim()));
            }
            0
        }
    }
}

/// Refuse a Codex tool call (see the module note on Windows).
fn codex_block(msg: &str) -> i32 {
    let msg = msg.trim();
    if cfg!(windows) {
        let q = |v: &str| serde_json::to_string(v).unwrap_or_default();
        println!("{{\"hookSpecificOutput\": {{\"hookEventName\": \"PreToolUse\", \"permissionDecision\": \"deny\", \
                  \"permissionDecisionReason\": {}}}}}", q(msg));
        let _ = std::io::stdout().flush();
        let _ = writeln!(std::io::stderr(), "{msg}");
        return 0;
    }
    eprintln!("{msg}");
    2
}

fn codex(event: Event) -> i32 {
    let root = project_root(false);
    let unreadable = || codex_block("BLOCKED: sos hook codex could not read the hook payload.");
    match event {
        Event::SessionStart => {
            let text = status::render(&root);
            if !text.trim().is_empty() {
                println!("{}", context_json("SessionStart", text.trim()));
            }
            0
        }
        Event::PreEdit => {
            let Some(d) = payload() else { return unreadable() };
            if s(&d, "tool_name") != "apply_patch" {
                return 0;
            }
            let Some(patch) = tool_input(&d).get("command").and_then(Value::as_str).map(str::to_string) else {
                return codex_block("BLOCKED: this apply_patch has no patch text; re-issue it as a standard '*** Begin Patch' patch.");
            };
            let paths = patch_paths(&patch);
            if paths.is_empty() {  // a patch we cannot parse might touch anything: block rather than guess
                return codex_block("BLOCKED: could not find file paths in this apply_patch; re-issue it as a standard \
                                    '*** Begin Patch' patch.");
            }
            env_guard(&paths).map_or(0, |msg| codex_block(&msg))
        }
        Event::PreBash => {
            let Some(d) = payload() else { return unreadable() };
            if s(&d, "tool_name") != "Bash" {
                return 0;
            }
            // The Python adapter crashed on a non-string command, which blocked where a simulator exists.
            let Some(command) = tool_input(&d).get("command").and_then(Value::as_str).map(str::to_string) else {
                return if cfg!(target_os = "macos") {
                    codex_block("BLOCKED: this shell call has no command text.")
                } else {
                    0
                };
            };
            sim_guard::check(&command).map_or(0, |msg| codex_block(&msg))
        }
        Event::PostBash => {
            let Some(d) = payload() else { return 0 };
            if s(&d, "tool_name") != "Bash" {
                return 0;
            }
            let out = match d.get("tool_response") {
                Some(Value::String(t)) => t.clone(),
                Some(v) => v.to_string(),
                None => "null".into(),
            };
            let mut args = vec![
                "--key".into(), format!("codex-{}", s(&d, "session_id")),
                "--cmd".into(), s(&tool_input(&d), "command").to_string(),
                "--agent".into(), "codex".into(),
            ];
            if let Some(code) = out.strip_prefix("Exit code: ").map(|r| r.chars().take_while(char::is_ascii_digit).collect::<String>()) {
                if !code.is_empty() {
                    args.extend(["--exit".into(), code]);
                }
            }
            let text = test_watch(&root, &args, &out);
            if !text.trim().is_empty() {
                println!("{}", context_json("PostToolUse", text.trim()));
            }
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_guard_blocks_real_env_files_only() {
        let v = |p: &[&str]| env_guard(&p.iter().map(|s| s.to_string()).collect::<Vec<_>>()).is_some();
        assert!(v(&["/r/.env"]));
        assert!(v(&["app/.Env.Local"]));
        assert!(v(&["C:\\work\\app\\.env"]));
        assert!(v(&["/REPO/.env.local\\notes.txt"]));  // POSIX: one file name
        assert!(v(&["notes\0.txt"]));
        assert!(!v(&["/r/.env.example", "notes.txt", ".envrc"]));
    }

    #[test]
    fn patch_paths_reads_every_header() {
        let p = "*** Begin Patch\n*** Add File: a.txt\n+hi\n\u{a0}*** Update File: dir\u{2028}/.env\n*** Move to: b\n*** End Patch";
        assert_eq!(patch_paths(p), vec!["a.txt", "dir\u{2028}/.env", "b"]);
        assert!(patch_paths("garbage").is_empty());
    }
}
