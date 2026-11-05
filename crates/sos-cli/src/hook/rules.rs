//! `sos hook claude|codex rules --file F…` and `sos hook rules --file F…`: the declarative rules
//! engine (crate sos-rules) as a hook. Payload on stdin; a block is the rules' message on stderr
//! with exit 2 (Codex on Windows: the deny/block JSON, exit 0). A rules file that cannot be loaded
//! blocks every call it would have checked, with how to lint it.

use super::codex_block;
use serde_json::Value;
use sos_rules::check::ProcessRunner;

/// All checks of one hook call end this long after the hook's 30-second timer started
/// (`GUARD_START`, hook/mod.rs), leaving 5 s to kill their process trees and answer.
const CHECK_BUDGET: std::time::Duration = std::time::Duration::from_secs(25);
use sos_rules::config::Places;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Agent {
    Claude,
    Codex,
    /// `sos hook rules`: agent-neutral, Claude's exit-code convention.
    Neutral,
}

fn files_from(base: &Path, files: &[String], home: Option<&Path>) -> Vec<PathBuf> {
    files.iter().map(|f| sos_rules::paths::expand(f, home, base).unwrap_or_else(|_| PathBuf::from(f))).collect()
}

fn answer(agent: Agent, post: bool, msg: &str) -> i32 {
    if agent == Agent::Codex && cfg!(windows) {
        if !post {
            return codex_block(msg);
        }
        // PostToolUse cannot deny (the tool already ran); Codex reads `decision: block` as
        // feedback for the model, the same role exit 2 + stderr plays elsewhere.
        let q = |v: &str| serde_json::to_string(v).unwrap_or_default();
        println!("{{\"decision\": \"block\", \"reason\": {}}}", q(msg.trim()));
        let _ = std::io::stdout().flush();
        let _ = writeln!(std::io::stderr(), "{}", msg.trim());
        return 0;
    }
    eprintln!("{}", msg.trim());
    2
}

/// Hook mode. `base` resolves relative `--file` paths and is the working directory of the rules.
pub(crate) fn hook(agent: Agent, base: PathBuf, files: &[String], explain: bool) -> i32 {
    let start = super::GUARD_START.get().copied().unwrap_or_else(std::time::Instant::now);
    let mut places = Places::from_env();
    places.cwd = base;
    let paths = files_from(&places.cwd, files, places.home.as_deref());
    let mut buf = Vec::new();
    let payload = match std::io::stdin().read_to_end(&mut buf) {
        Ok(_) => serde_json::from_slice::<Value>(&buf).unwrap_or(Value::Null),
        Err(_) => Value::Null,
    };
    let post = matches!(payload.get("hook_event_name").and_then(Value::as_str), Some("PostToolUse" | "PostToolUseFailure"));
    if paths.is_empty() {
        return answer(agent, post, "BLOCKED: `sos hook … rules` needs at least one --file <rules.toml>.");
    }
    let set = match sos_rules::load(&paths, &places) {
        Ok(s) => s,
        Err(e) => return answer(agent, post, &sos_rules::load_error_message(&e, &paths)),
    };
    let out = sos_rules::decide(&set, &payload, &places, &ProcessRunner::until(start + CHECK_BUDGET));
    if explain {
        for t in &out.trace {
            eprintln!("[rules] {t}");
        }
    }
    if out.blocks.is_empty() {
        return 0;
    }
    answer(agent, out.post || post, &out.blocks.join("\n\n"))
}

/// `--lint`: load, print warnings; exit 1 on an error.
pub(crate) fn lint(files: &[String]) -> i32 {
    let places = Places::from_env();
    let paths = files_from(&places.cwd, files, places.home.as_deref());
    if paths.is_empty() {
        eprintln!("cần ít nhất một --file <rules.toml>");
        return 1;
    }
    match sos_rules::load(&paths, &places) {
        Ok(set) => {
            for w in &set.warnings {
                println!("cảnh báo: {w}");
            }
            println!("ok: {} luật", set.rules.len());
            0
        }
        Err(e) => {
            eprintln!("lỗi: {e}");
            1
        }
    }
}

/// `--cases C.jsonl`: run recorded cases; exit 1 on any failure.
pub(crate) fn cases(files: &[String], cases: &str) -> i32 {
    let places = Places::from_env();
    let paths = files_from(&places.cwd, files, places.home.as_deref());
    i32::from(sos_rules::cases::run(&paths, &places.cwd.join(cases)) > 0)
}
