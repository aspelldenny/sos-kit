//! `sos usage` on hand-written transcripts (tests/usage/fixtures). The expected numbers below were
//! worked out by hand from the fixture lines, not by running the code.
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const SECRET: &str = "SOS-SECRET-7f3a9c";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/usage/fixtures")
}

fn usage(args: &[&str]) -> (i32, String, String) {
    let f = fixtures();
    let mut c = Command::new(env!("CARGO_BIN_EXE_sos"));
    c.arg("usage");
    for (flag, path) in [("--dir", f.join("claude")), ("--codex-dir", f.join("codex")), ("--prices", f.join("../../../crates/sos-cli/src/usage/prices.toml"))] {
        if !args.contains(&flag) {
            c.arg(flag).arg(path);
        }
    }
    let o = c
        .args(args)
        .env("HOME", f.join("no-home"))
        .output()
        .unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
}

fn json(args: &[&str]) -> Value {
    let mut all = vec!["--json", "--since", "2026-09-01", "--until", "2026-11-05"];
    all.extend_from_slice(args);
    let (rc, out, err) = usage(&all);
    assert_eq!(rc, 0, "stderr: {err}");
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"))
}

/// (calls, input, cache_write, cache_read, output) of the row whose `key` field is `val`.
fn row(doc: &Value, agent: &str, key: &str, val: &str) -> (u64, u64, u64, u64, u64) {
    let rows: Vec<&Value> = doc["rows"].as_array().unwrap().iter().filter(|r| r["agent"] == agent && r[key] == val).collect();
    assert_eq!(rows.len(), 1, "one row for {key}={val}: {rows:?}");
    let r = rows[0];
    let g = |f: &str| r[f].as_u64().unwrap();
    (g("calls"), g("input"), g("cache_write"), g("cache_read"), g("output"))
}

#[test]
fn claude_by_model_counts_each_call_once() {
    let d = json(&["--by", "model"]);
    // opus: m1 (largest of three streamed copies), m2, the call with no id (counted per line, like
    // agent-usage.py), o1 (S0), x1, c1 (once across the resumed copy), c2.
    assert_eq!(row(&d, "claude", "model", "claude-opus-5-5"), (7, 24, 1000, 2100, 40383));
    // sonnet: a1's two calls (mside1 once: the main file's sidechain copy is owned by agent-a1), b1,
    // and mside9, a sidechain whose subagent file is missing, so it is kept.
    assert_eq!(row(&d, "claude", "model", "claude-sonnet-5-5"), (4, 12, 511, 101, 59));
    assert_eq!(row(&d, "claude", "model", "claude-haiku-5-5"), (2, 60100, 0, 60000, 30));
    assert_eq!(d["rows"].as_array().unwrap().len(), 3, "no <synthetic> row: {}", d["rows"]);
    let opus = d["rows"].as_array().unwrap().iter().find(|r| r["model"] == "claude-opus-5-5").unwrap();
    assert_eq!((opus["thinking"].as_u64(), opus["thinking_calls"].as_u64()), (Some(30040), Some(2)), "thinking is known for 2 of 7 calls");
    let m = &d["meta"]["agents"]["claude"];
    assert_eq!(m["skipped"]["synthetic"], 1);
    assert_eq!(m["skipped"]["no_id"], 1, "counted, but reported: it cannot be deduplicated");
    assert_eq!(m["skipped"]["bad_json"], 2, "the broken line and the cut-off last line");
    assert_eq!(m["skipped"]["sidechain_owned"], 1);
}

#[test]
fn claude_roles_results_and_large_context() {
    let d = json(&["--by", "role"]);
    // main-file calls without a querySource count as sdk: m1, m2, the no-id call, o1, x1, c1, c2.
    assert_eq!(row(&d, "claude", "role", "sdk").0, 7);
    assert_eq!(row(&d, "claude", "role", "unknown"), (1, 7, 0, 0, 3), "mside9: no subagent file, no meta");
    assert_eq!(row(&d, "claude", "role", "agent:custom:worker"), (3, 5, 511, 101, 56));
    assert_eq!(row(&d, "claude", "role", "agent:builtin:Explore").0, 2);
    let e = &d["estimates"]["by_agent"]["claude"];
    let w = &e["subagent_results"]["agent:custom:worker"];
    assert_eq!((w["results"].as_u64(), w["subagent_result_text_chars"].as_u64(), w["spawn_acks"].as_u64()), (Some(1), Some(1500), Some(1)));
    let x = &e["subagent_results"]["agent:builtin:Explore"];
    assert_eq!((x["subagent_result_text_chars"].as_u64(), x["errors"].as_u64()), (Some(7), Some(1)));
    let large = e["large_initial_context"].as_array().unwrap();
    assert_eq!(large.len(), 1, "{large:?}");
    assert_eq!(large[0]["agent_id"], "a2");
    assert_eq!(large[0]["parent_tool_use"], "t3");
}

#[test]
fn filters_by_project_and_day() {
    let d = json(&["--by", "project", "--project", "/w/app"]);
    let rows = d["rows"].as_array().unwrap();
    assert!(rows.iter().all(|r| r["project"] == "/w/app"), "{rows:?}");
    let d = json(&["--by", "day", "--project", "app"]);
    let days: Vec<&str> = d["rows"].as_array().unwrap().iter().map(|r| r["day"].as_str().unwrap()).collect();
    assert!(days.contains(&"2026-09-20") && days.contains(&"2026-11-02") && !days.contains(&"2026-08-01") && !days.contains(&"2026-11-04"), "{days:?}");
    let (rc, _, err) = usage(&["--since", "2026-9-1"]);
    assert_eq!(rc, 2, "a bad date is an error: {err}");
}

#[test]
fn codex_records_compaction_and_fallback() {
    let d = json(&["--agent", "codex", "--by", "model"]);
    // gpt-6-astra: r1 once (1000 in, 900 cached) + child k1 (300, 100 cached). gpt-6: rc once (compacted copy
    // ignored) + old file from totals: 100, +150, then a reset to 40 → 290 in, 50 cached, 35 out.
    assert_eq!(row(&d, "codex", "model", "gpt-6-astra"), (2, 300, 0, 1000, 80));
    assert_eq!(row(&d, "codex", "model", "gpt-6"), (4, 244293 + 240, 0, 50, 1911 + 35));
}

#[test]
fn nothing_from_the_transcripts_text_reaches_the_output() {
    for args in [&["--agent", "all", "--explain"][..], &["--agent", "all", "--explain", "--json"][..], &["--agent", "all", "--by", "session"][..]] {
        let mut all = vec!["--since", "2026-09-01"];
        all.extend_from_slice(args);
        let (rc, out, err) = usage(&all);
        assert_eq!(rc, 0, "{err}");
        assert!(!out.contains(SECRET) && !err.contains(SECRET), "transcript text leaked with {args:?}");
        assert!(!out.contains("reply ") && !out.contains("do it"), "{out}");
    }
}

#[test]
fn table_has_totals_and_footnotes() {
    let (rc, out, err) = usage(&["--since", "2026-09-01", "--until", "2026-11-05", "--by", "model"]);
    assert_eq!(rc, 0, "{err}");
    assert!(out.contains("claude-opus-5-5") && out.contains("total"), "{out}");
    assert!(out.contains("40,383"), "thousands separators: {out}");
}

#[test]
fn table_marks_partial_thinking() {
    let (_, out, _) = usage(&["--since", "2026-09-01", "--until", "2026-11-05", "--by", "model"]);
    assert!(out.contains("30,040+") && out.contains("thinking: N+"), "{out}");
}

#[test]
fn a_broken_prices_file_is_named_not_quoted() {
    let dir = std::env::temp_dir().join(format!("sos-usage-prices-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("p.toml");
    std::fs::write(&f, format!("version = \"x\"\n[prices.\"m\"]\ninput = {SECRET}\n")).unwrap();
    let (rc, out, err) = usage(&["--prices", f.to_str().unwrap(), "--json", "--explain"]);
    assert_eq!(rc, 2);
    assert!(!out.contains(SECRET) && !err.contains(SECRET), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn no_home_and_no_dirs_is_an_error_not_the_current_folder() {
    let o = Command::new(env!("CARGO_BIN_EXE_sos")).arg("usage").env_remove("HOME").env_remove("USERPROFILE")
        .current_dir(fixtures().join("claude")).output().unwrap();
    assert_eq!(o.status.code(), Some(2), "{}", String::from_utf8_lossy(&o.stdout));
    assert!(String::from_utf8_lossy(&o.stderr).contains("--dir"));
}

#[cfg(unix)]
#[test]
fn codex_does_not_follow_symlinks() {
    let t = std::env::temp_dir().join(format!("sos-usage-links-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&t);
    let src = fixtures().join("codex/2026/10/27");
    let day = t.join("2026/10/27");
    std::fs::create_dir_all(&day).unwrap();
    for e in std::fs::read_dir(&src).unwrap() {
        let e = e.unwrap();
        std::fs::copy(e.path(), day.join(e.file_name())).unwrap();
    }
    std::os::unix::fs::symlink(t.join("2026"), t.join("mirror")).unwrap();
    std::os::unix::fs::symlink(&t, day.join("loop")).unwrap();
    let run = |dir: &Path| {
        let (rc, out, err) = usage(&["--agent", "codex", "--codex-dir", dir.to_str().unwrap(), "--json", "--since", "2026-09-01", "--by", "model"]);
        assert_eq!(rc, 0, "{err}");
        serde_json::from_str::<Value>(&out).unwrap()
    };
    let (a, b) = (run(&fixtures().join("codex")), run(&t));
    assert_eq!(a["rows"], b["rows"], "an alias folder must not count twice");
    assert!(b["meta"]["agents"]["codex"]["coverage_warnings"].to_string().contains("symlink not followed"));
    let _ = std::fs::remove_dir_all(&t);
}

#[cfg(unix)]
#[test]
fn claude_does_not_follow_a_linked_session_folder() {
    let t = std::env::temp_dir().join(format!("sos-usage-clinks-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&t);
    let p = t.join("-w-app");
    std::fs::create_dir_all(&p).unwrap();
    let src = fixtures().join("claude/-w-app");
    std::fs::copy(src.join("S1.jsonl"), p.join("S1.jsonl")).unwrap();
    std::os::unix::fs::symlink(src.join("S1"), p.join("S1")).unwrap();
    let (rc, out, err) = usage(&["--dir", t.to_str().unwrap(), "--json", "--since", "2026-09-01", "--by", "model"]);
    assert_eq!(rc, 0, "{err}");
    let d: Value = serde_json::from_str(&out).unwrap();
    assert!(d["rows"].as_array().unwrap().iter().all(|r| r["model"] != "claude-haiku-5-5"), "haiku is only in the linked subagent files");
    assert!(d["meta"]["agents"]["claude"]["coverage_warnings"].to_string().contains("symlink not followed"));
    let _ = std::fs::remove_dir_all(&t);
}
