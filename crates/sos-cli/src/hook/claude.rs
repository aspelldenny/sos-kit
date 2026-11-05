//! Claude Code payloads: tool_input.file_path/notebook_path, tool_response{stdout,stderr}, agent_id.

use super::{context_json, payload, project_root, s, tool_input, Event};
use crate::watch;
use crate::guards::{env::env_guard, sim as sim_guard};
use crate::status;
use serde_json::Value;

pub(crate) fn claude(event: Event) -> i32 {
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
        // Routed to rules.rs by mod.rs (it needs --file); never reaches here.
        Event::Rules => 2,
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
            let key = format!("claude-{}-{agent}", s(&d, "session_id"));
            let input = tool_input(&d);
            let call = watch::Call { key: &key, cmd: s(&input, "command"), exit: failure.then_some(1), agent: "claude" };
            let text = watch::run(&root, &call, &out);
            if !text.trim().is_empty() {
                println!("{}", context_json(&event, text.trim()));
            }
            0
        }
    }
}

