//! `sos predict` on a small repo: which lines count, how each is dated, and the two output shapes
//! (wave 3.1). Parity with `skills/depth-check/scripts/predictions.py` is checked separately.
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git").args(args).current_dir(dir)
        .env("GIT_AUTHOR_DATE", "2026-01-10T12:00:00Z").env("GIT_COMMITTER_DATE", "2026-01-10T12:00:00Z")
        .env("GIT_CONFIG_NOSYSTEM", "1").env("HOME", dir)
        .status().unwrap().success();
    assert!(ok, "git {args:?}");
}

fn write(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, body).unwrap();
}

fn sos(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_sos")).arg("predict").args(args).current_dir(dir).env("TZ", "UTC").output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned())
}

const PLAN: &str = "# Giả thuyết ra mắt

Dự đoán: retention ≥ 40% sau 7 ngày.
short
Một dòng thường không có số.

## Kết quả
Thực tế: 31%, trượt.
";

fn repo() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    git(r, &["init", "-q"]);
    git(r, &["config", "user.email", "t@example.com"]);
    git(r, &["config", "user.name", "t"]);
    write(r, "docs/plan.md", PLAN);
    // Skipped: history files restate predictions, and archive/ is old work.
    write(r, "CHANGELOG.md", "Dự đoán: retention ≥ 40% sau 7 ngày.\n");
    write(r, "archive/old.md", "Dự đoán: retention ≥ 40% sau 7 ngày.\n");
    write(r, "src/notes.rs", "// Dự đoán: retention ≥ 40% sau 7 ngày.\n");
    git(r, &["add", "-A"]);
    git(r, &["commit", "-qm", "plan"]);
    // Uncommitted: dated by the week in the path, or by mtime.
    write(r, "content/2026-10-W3/post.md", "Kill criteria: dưới 500 views @ 48h thì bỏ.\n");
    write(r, "notes/today.txt", "Pass if conversion > 3% in week 2.\n");
    t
}

#[test]
fn finds_scores_and_dates_candidates() {
    let t = repo();
    let (rc, out) = sos(t.path(), &["--json"]);
    assert_eq!(rc, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(keys, ["repo", "today", "total", "rows"]);
    let rows = v["rows"].as_array().unwrap();
    let files: Vec<&str> = rows.iter().map(|r| r["file"].as_str().unwrap()).collect();
    assert_eq!(rows.len(), 3, "{files:?}");
    assert_eq!(v["total"], 3);

    let plan = rows.iter().find(|r| r["file"] == "docs/plan.md").unwrap();
    let keys: Vec<&str> = plan.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(keys, ["file", "line", "heading", "text", "score", "result_nearby", "stated_dates", "date_source", "written", "age_days"]);
    assert_eq!(plan["line"], 3);
    assert_eq!(plan["heading"], "Giả thuyết ra mắt");
    // keyword 3 + heading keyword 2 + outcome 2 + window 1.
    assert_eq!(plan["score"], 8);
    assert_eq!(plan["result_nearby"], true);
    assert_eq!((&plan["date_source"], &plan["written"]), (&"git".into(), &"2026-01-10".into()));

    let post = rows.iter().find(|r| r["file"] == "content/2026-10-W3/post.md").unwrap();
    assert_eq!((&post["date_source"], &post["written"]), (&"path".into(), &"2026-10-15".into()));
    let note = rows.iter().find(|r| r["file"] == "notes/today.txt").unwrap();
    assert_eq!(note["date_source"], "mtime");
    assert_eq!(note["written"], v["today"]);
    assert_eq!(note["result_nearby"], false);

    // Strongest first.
    let scores: Vec<i64> = rows.iter().map(|r| r["score"].as_i64().unwrap()).collect();
    assert!(scores.windows(2).all(|w| w[0] >= w[1]), "{scores:?}");
}

#[test]
fn filters_and_markdown_table() {
    let t = repo();
    let (_, out) = sos(t.path(), &["--days", "100000"]);
    assert!(out.contains("0 candidate lines at least 100000 days old; showing 0"), "{out}");

    let (_, out) = sos(t.path(), &["--limit", "1"]);
    assert!(out.contains("3 candidate lines at least 0 days old; showing 1,"), "{out}");
    assert!(out.contains("| docs/plan.md:3 | 8 | 2026-01-10 | "), "{out}");
    // A negative limit drops that many from the end, like a Python slice.
    let (_, out) = sos(t.path(), &["--limit", "-1"]);
    assert!(out.contains("showing 2,"), "{out}");

    let (_, out) = sos(t.path(), &[]);
    assert!(out.starts_with("# Candidate predictions and thresholds — "), "{out}");
    assert!(out.contains("| File:line | Score | Written | Age (d) | Result words nearby | Heading | Text |\n|---|---|---|---|---|---|---|\n"), "{out}");
    assert!(out.contains("| content/2026-10-W3/post.md:1 | ") && out.contains("2026-10-15 (path)"), "{out}");
    assert!(out.contains(" (mtime) | "), "{out}");
}

#[test]
fn outside_git_walks_the_folder() {
    let t = tempfile::tempdir().unwrap();
    write(t.path(), "a/plan.md", "Dự đoán: retention ≥ 40% sau 7 ngày.\n");
    write(t.path(), ".hidden/plan.md", "Dự đoán: retention ≥ 40% sau 7 ngày.\n");
    write(t.path(), "node_modules/x/plan.md", "Dự đoán: retention ≥ 40% sau 7 ngày.\n");
    let (rc, out) = sos(t.path(), &["--json"]);
    assert_eq!(rc, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let files: Vec<&str> = v["rows"].as_array().unwrap().iter().map(|r| r["file"].as_str().unwrap()).collect();
    assert_eq!(files, ["a/plan.md"]);
}
