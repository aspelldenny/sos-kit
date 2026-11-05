//! `sos hook word-limit` through the binary (wave 4.6).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn sos(args: &[&str], dir: &Path, stdin: &str) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_sos"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = c.stdin.take().unwrap().write_all(stdin.as_bytes());
    c.wait_with_output().unwrap()
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn word_limit_measures_and_blocks_only_when_asked() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let t = p.join("t.jsonl");
    std::fs::write(&t, [
        r#"{"type":"user","message":{"content":"go"}}"#,
        r#"{"type":"assistant","message":{"content":[{"type":"text","text":"one two three four five six"}]}}"#,
        r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash"}]}}"#,
        r#"{"type":"user","message":{"content":[{"type":"tool_result","content":"ok"}]}}"#,
        r#"{"type":"assistant","message":{"content":[{"type":"text","text":"**Done.** PR #49 green —"}]}}"#,
    ].join("\n")).unwrap();
    let log = p.join("wl.tsv");
    let payload = format!(r#"{{"session_id":"s1","transcript_path":{}}}"#, serde_json::json!(t.to_string_lossy()));

    let o = sos(&["hook", "word-limit", "--max", "3", "--log", &log.to_string_lossy()], p, &payload);
    assert_eq!(o.status.code(), Some(0), "warn only by default: {}", text(&o));
    assert!(text(&o).contains("4 words (limit 3)"), "{}", text(&o));
    let o = sos(&["hook", "word-limit", "--max", "4", "--block", "--log", &log.to_string_lossy()], p, &payload);
    assert_eq!(o.status.code(), Some(0), "at the limit is fine: {}", text(&o));
    let logged = std::fs::read_to_string(&log).unwrap();
    let rows: Vec<Vec<&str>> = logged.lines().map(|l| l.split('\t').collect()).collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(&rows[0][1..], ["s1", "4", "3", "1"]);
    assert_eq!(&rows[1][1..], ["s1", "4", "4", "0"]);

    let o = sos(&["hook", "word-limit", "--max", "3", "--block"], p, &payload);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    let again = payload.replace('}', r#","stop_hook_active":true}"#);
    assert_eq!(sos(&["hook", "word-limit", "--max", "3", "--block"], p, &again).status.code(), Some(0), "never twice in a row");
    let o = sos(&["hook", "word-limit", "--max", "3", "--block"], p, r#"{"last_assistant_message":"a b"}"#);
    assert_eq!(o.status.code(), Some(0));
    let cut = p.join("cut.jsonl");
    std::fs::write(&cut, [r#"{"type":"assistant","message":{"content":[{"type":"text","text":"one two three"}]}}"#,
                          r#"{"type":"user","message":"#].join("\n")).unwrap();
    let cut_payload = format!(r#"{{"transcript_path":{}}}"#, serde_json::json!(cut.to_string_lossy()));
    for bad in ["nope", r#"{"transcript_path":"/no/such/file"}"#, "{}", cut_payload.as_str()] {
        let o = sos(&["hook", "word-limit", "--max", "1", "--block"], p, bad);
        assert_eq!(o.status.code(), Some(0), "errors never block ({bad}): {}", text(&o));
        assert!(text(&o).contains("not measured"), "{}", text(&o));
    }
}
