//! `sos hook` — the agent hooks in Rust. This module only translates payloads to calls
//! (claude.rs, codex.rs); the checks live in `guards/`, the advisor trigger in `watch.rs`.
//! Origin (port of adapters/{claude,codex}/hook.py, scripts/env-guard.sh,
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

use crate::guards::{env::env_guard, sim as sim_guard};
use crate::status;

mod claude;
mod codex;

use claude::claude;
use codex::{codex, codex_block};

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

pub(crate) fn payload() -> Option<serde_json::Map<String, Value>> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf).ok()?;
    match serde_json::from_slice(&buf).ok()? {
        Value::Object(m) => Some(m),
        _ => None,
    }
}

pub(crate) fn s<'a>(d: &'a serde_json::Map<String, Value>, k: &str) -> &'a str {
    d.get(k).and_then(Value::as_str).unwrap_or("")
}

pub(crate) fn tool_input(d: &serde_json::Map<String, Value>) -> serde_json::Map<String, Value> {
    d.get("tool_input").and_then(Value::as_object).cloned().unwrap_or_default()
}

/// Python json.dumps layout (`, ` and `: `), which the adapter tests and any log reader expect.
pub(crate) fn context_json(event: &str, text: &str) -> String {
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
pub(crate) fn project_root(claude: bool) -> PathBuf {
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
pub(crate) fn test_watch(root: &Path, args: &[String], out: &str) -> String {
    let Some(scripts) = scripts_dir(root) else { return String::new() };
    let py = python();
    let mut cmd = Command::new(&py);
    cmd.arg(scripts.join("test-watch.py")).args(args).current_dir(root)
        .env("PYTHONUTF8", "1").env("SOS_PYTHON", &py);
    run_limited(&mut cmd, out.as_bytes(), Duration::from_secs(300))
        .map(|(_, o)| String::from_utf8_lossy(&o).into_owned()).unwrap_or_default()
}

