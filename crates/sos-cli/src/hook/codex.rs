//! Codex payloads: apply_patch V4A, and the Windows deny-JSON block.

use super::{context_json, payload, project_root, s, tool_input, Event};
use crate::watch;
use crate::guards::{env::env_guard, sim as sim_guard};
use crate::status;
use serde_json::Value;
use std::io::Write;

/// Refuse a Codex tool call (see the module note on Windows).
pub(crate) fn codex_block(msg: &str) -> i32 {
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

pub(crate) fn codex(event: Event) -> i32 {
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
            let paths = sos_rules::patch_paths(&patch);
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
        // Routed to rules.rs by mod.rs (it needs --file); never reaches here.
        Event::Rules => 2,
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
            let key = format!("codex-{}", s(&d, "session_id"));
            let exit = out.strip_prefix("Exit code: ")
                .and_then(|r| r.chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok());
            let input = tool_input(&d);
            let call = watch::Call { key: &key, cmd: s(&input, "command"), exit, agent: "codex" };
            let text = watch::run(&root, &call, &out);
            if !text.trim().is_empty() {
                println!("{}", context_json("PostToolUse", text.trim()));
            }
            0
        }
    }
}
