//! `sos sessions idle|count` through the binary (wave 4.3, 4.4).
//! Session files are written with `lastActivityAt` relative to now (the plan's proof: fake ages).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

fn session(dir: &Path, id: &str, cli: &str, idle_secs: u128, archived: bool) {
    let org = dir.join("acct").join("org");
    std::fs::create_dir_all(&org).unwrap();
    let body = format!(
        r#"{{"sessionId":"{id}","cliSessionId":"{cli}","cwd":"/w/{id}","title":"T {id}","lastActivityAt":{},"createdAt":1,"isArchived":{archived},"other":[1,2]}}"#,
        now_ms() - idle_secs * 1000
    );
    std::fs::write(org.join(format!("{id}.json")), body).unwrap();
}

fn sos(args: &[&str], dir: &Path, stdin: &str) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_sos"))
        .args(args)
        .env("SOS_SESSIONS_DIR", dir)
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
fn count_two_allows_three_blocks() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    session(p, "local_me", "cli-me", 1, false);
    session(p, "local_a", "cli-a", 60, false);
    session(p, "local_b", "cli-b", 3600, false);
    session(p, "local_old", "cli-old", 3 * 3600, false);
    session(p, "local_done", "cli-done", 10, true);
    std::fs::write(p.join("acct/org/scheduled-tasks.json"), "{}").unwrap();
    let hook = r#"{"session_id":"cli-me","tool_name":"mcp__ccd_session__start_session"}"#;

    let o = sos(&["sessions", "count", "--hook", "--max", "3"], p, hook);
    assert_eq!(o.status.code(), Some(0), "2 others active → allowed: {}", text(&o));
    let o = sos(&["sessions", "count"], p, "");
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "3", "the asker counts without --hook");

    session(p, "local_c", "cli-c", 30, false);
    let o = sos(&["sessions", "count", "--hook", "--max", "3"], p, hook);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("xếp hàng chờ") && text(&o).contains("3 session"), "{}", text(&o));
    // A longer window takes in the old one.
    let o = sos(&["sessions", "count", "--within", "4h", "--exclude", "local_me"], p, "");
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "4");
}

#[test]
fn count_fails_closed() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    session(p, "local_a", "cli-a", 1, false);
    for (stdin, why) in [("not json", "not a JSON object"), ("{}", "no session_id")] {
        let o = sos(&["sessions", "count", "--hook", "--max", "3"], p, stdin);
        assert_eq!(o.status.code(), Some(2), "{why}: {}", text(&o));
        assert!(text(&o).contains(why), "{}", text(&o));
    }
    std::fs::write(p.join("acct/org/local_broken.json"), "{\"sessionId\":").unwrap();
    let o = sos(&["sessions", "count", "--max", "3"], p, "");
    assert_eq!(o.status.code(), Some(2));
    assert!(text(&o).contains("local_broken.json") && text(&o).contains("How to fix"), "{}", text(&o));
    let o = sos(&["sessions", "count", "--max", "3", "--dir", &p.join("missing").to_string_lossy()], p, "");
    assert_eq!(o.status.code(), Some(2));
    std::fs::remove_file(p.join("acct/org/local_broken.json")).unwrap();
    std::fs::write(p.join("acct/org/local_blind.json"), r#"{"sessionId":"local_blind","createdAt":1,"isArchived":false}"#).unwrap();
    let o = sos(&["sessions", "count", "--max", "3"], p, "");
    assert_eq!(o.status.code(), Some(2), "no lastActivityAt is not an old session: {}", text(&o));
    assert!(text(&o).contains("local_blind") && text(&o).contains("How to fix"), "{}", text(&o));
}

#[test]
fn idle_reports_quiet_archived_and_missing() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    session(p, "local_quiet", "c1", 30 * 3600, false);
    session(p, "local_busy", "c2", 600, false);
    session(p, "local_done", "c3", 99 * 3600, true);
    let o = sos(&["sessions", "idle", "--over", "24h"], p, "");
    assert_eq!(o.status.code(), Some(0));
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    assert!(out.starts_with("local_quiet\t1d 6h\tT local_quiet\t/w/local_quiet"), "{out}");
    assert_eq!(out.lines().count(), 1, "{out}");

    let o = sos(&["sessions", "idle", "--over", "24h", "--json", "local_quiet", "local_busy", "local_done", "local_gone"], p, "");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let ids: Vec<_> = v.as_array().unwrap().iter().map(|r| (r["id"].as_str().unwrap(), r["found"].as_bool().unwrap())).collect();
    assert_eq!(ids, [("local_quiet", true), ("local_done", true), ("local_gone", false)]);
    assert_eq!(v[1]["archived"], true);

    let o = sos(&["sessions", "idle", "not-an-id"], p, "");
    assert_eq!(o.status.code(), Some(2));
}
