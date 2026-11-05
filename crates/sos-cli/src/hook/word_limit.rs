//! `sos hook word-limit --max N` — Claude Code Stop hook: how long was the closing message?
//!
//! Wave 4.6 of docs/plans/RUST_ALL_TOOLS_2026-11-04.md. Chủ nhà reads the last message of a turn;
//! a wall of text there costs attention. The limit is easy to make annoying, so the default is to
//! measure, not to block: a note on stderr (exit 0) and, with `--log FILE`, one TSV line per turn
//! (unix seconds, session id, words, max, over 0/1) so a few days of trial can be counted before
//! anyone turns `--block` on.
//!
//! The message is the payload's `last_assistant_message` when present, else the assistant text
//! after the last tool call or user entry in `transcript_path` (sidechain and meta lines skipped).
//! A word is a whitespace-separated token with a letter or digit in it (Markdown marks, bullets
//! and table pipes are not words).
//!
//! `--block` exits 2 when the message is over (Claude Code then continues the turn with the reason)
//! unless `stop_hook_active` says this stop already comes from a block. Errors (no payload, no
//! transcript) never block: a Stop hook that blocks on its own failure would keep the turn going.

use serde_json::{Map, Value};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// How much of the transcript's end to read: the closing message is near the end.
const TAIL_BYTES: u64 = 2 * 1024 * 1024;

pub fn words(text: &str) -> usize {
    text.split_whitespace().filter(|w| w.chars().any(char::is_alphanumeric)).count()
}

/// The closing assistant text in a transcript tail (JSON lines, oldest first). A line that is not
/// a JSON object is an error: it may be the user entry that ends the turn, so what comes before
/// it can no longer be trusted to be the closing message.
pub fn closing_text(lines: &str) -> Result<String, String> {
    let mut parts: Vec<String> = Vec::new();
    for (i, line) in lines.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(Value::Object(d)) = serde_json::from_str::<Value>(line) else {
            return Err(format!("transcript line {} (of the tail read) is not a JSON object", i + 1));
        };
        if d.get("isSidechain").and_then(Value::as_bool) == Some(true) || d.get("isMeta").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        match d.get("type").and_then(Value::as_str) {
            Some("user") => parts.clear(),
            Some("assistant") => {
                let content = d.get("message").and_then(|m| m.get("content"));
                match content {
                    Some(Value::String(t)) => parts.push(t.clone()),
                    Some(Value::Array(blocks)) => {
                        for b in blocks {
                            match b.get("type").and_then(Value::as_str) {
                                Some("text") => parts.push(b.get("text").and_then(Value::as_str).unwrap_or("").to_string()),
                                Some("tool_use") => parts.clear(),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(parts.join("\n"))
}

fn tail(path: &Path) -> std::io::Result<String> {
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    let start = len.saturating_sub(TAIL_BYTES);
    f.seek(SeekFrom::Start(start))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf).into_owned();
    // Cut mid-file: the first line is partial.
    Ok(if start > 0 { text.split_once('\n').map(|(_, r)| r.to_string()).unwrap_or_default() } else { text })
}

fn message(p: &Map<String, Value>) -> Result<String, String> {
    if let Some(m) = p.get("last_assistant_message").and_then(Value::as_str) {
        return Ok(m.to_string());
    }
    let t = super::s(p, "transcript_path");
    if t.is_empty() {
        return Err("the payload has neither last_assistant_message nor transcript_path".into());
    }
    let lines = tail(Path::new(t)).map_err(|e| format!("{t}: {e}"))?;
    closing_text(&lines).map_err(|e| format!("{t}: {e}"))
}

pub fn run(max: usize, block: bool, log: Option<&str>) -> i32 {
    let Some(p) = super::payload() else {
        eprintln!("sos hook word-limit: the payload on stdin is not a JSON object; not measured.");
        return 0;
    };
    let text = match message(&p) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("sos hook word-limit: {e}; not measured.");
            return 0;
        }
    };
    let n = words(&text);
    let over = n > max;
    if let Some(log) = log {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let line = format!("{secs}\t{}\t{n}\t{max}\t{}\n", super::s(&p, "session_id"), over as u8);
        let wrote = std::fs::OpenOptions::new().create(true).append(true).open(log).and_then(|mut f| f.write_all(line.as_bytes()));
        if let Err(e) = wrote {
            eprintln!("sos hook word-limit: cannot append to {log}: {e}");
        }
    }
    if !over {
        return 0;
    }
    let active = p.get("stop_hook_active").and_then(Value::as_bool).unwrap_or(false);
    if block && !active {
        eprintln!("The closing message has {n} words (limit {max}). Rewrite it in at most {max} words: \
                   the result first, then only what the reader must decide or do.");
        return 2;
    }
    eprintln!("sos hook word-limit: closing message {n} words (limit {max}).");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_words_not_marks() {
        assert_eq!(words("**Xong.** PR #49 — 3 lỗi\n- a\n| x | y |\n---"), 8);
        assert_eq!(words(""), 0);
    }

    #[test]
    fn closing_text_after_last_tool_call() {
        let l = [
            r#"{"type":"user","message":{"content":"làm đi"}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"early words here"}]}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash"}]}}"#,
            r#"{"type":"user","message":{"content":[{"type":"tool_result"}]}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"long hidden"}]}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"final one"}]}}"#,
            r#"{"type":"assistant","isSidechain":true,"message":{"content":[{"type":"text","text":"subagent noise"}]}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"two"}]}}"#,
            r#"{"type":"system","content":"x"}"#,
            "",
        ]
        .join("\n");
        assert_eq!(closing_text(&l).unwrap(), "final one\ntwo");
        assert_eq!(closing_text("").unwrap(), "");
        // A cut user line after an old message: not measured, never the old text.
        let cut = [r#"{"type":"assistant","message":{"content":[{"type":"text","text":"one two three"}]}}"#,
                   r#"{"type":"user","message":"#].join("\n");
        assert!(closing_text(&cut).is_err());
    }

    #[test]
    fn tail_drops_partial_first_line() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("t.jsonl");
        let filler = "x".repeat(TAIL_BYTES as usize);
        std::fs::write(&f, format!("{filler}\n{}\n", r#"{"type":"assistant","message":{"content":"hi there"}}"#)).unwrap();
        assert_eq!(closing_text(&tail(&f).unwrap()).unwrap(), "hi there");
    }
}
