//! `sos hook` — the agent hooks in Rust. This module only translates payloads to calls
//! (claude.rs, codex.rs); the checks live in `guards/`, the advisor trigger in `watch.rs`.
//! Origin (port of adapters/{claude,codex}/hook.py, scripts/env-guard.sh,
//! scripts/sim-guard and scripts/status.sh; plan: docs/plans/RUST_ALL_TOOLS_2026-11-04.md, wave 1a).
//!
//!   sos hook claude|codex session-start   status lines (Claude: stdout; Codex: additionalContext)
//!   sos hook claude|codex pre-edit        refuse edits of real .env files
//!   sos hook claude|codex pre-bash        refuse xcodebuild test runs that clone the simulator
//!   sos hook claude|codex post-bash       advisor trigger (watch.rs, in-process; asks `sos advise`)
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
    if event == Event::PostBash {
        // The advisor never blocks: past its budget the hook just ends, silently, before the
        // agent's own 320 s limit. The advisor's deadline counts from the same start (reading the
        // payload included), so every child it runs is stopped before this exit (review 05/11/2026).
        crate::advise::set_deadline(Instant::now() + crate::watch::BUDGET);
        std::thread::spawn(|| {
            std::thread::sleep(crate::watch::BUDGET + Duration::from_secs(20));
            std::process::exit(0);
        });
        return;
    }
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

/// The git top level of `dir`. Bounded (10 s, git stopped with its children): a hung git must not
/// outlive the hook (review 05/11/2026).
fn git_top(dir: &Path) -> Option<PathBuf> {
    let mut c = Command::new("git");
    c.current_dir(dir).args(["rev-parse", "--show-toplevel"]);
    let o = crate::role::capture(&mut c, String::new(), Duration::from_secs(10)).ok()?;
    o.status.is_some_and(|s| s.success()).then(|| PathBuf::from(o.stdout.trim_end_matches(['\n', '\r'])))
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

