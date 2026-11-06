//! `sos usage --by skill`: how often each extension was actually used, from what is already on disk.
//! Skills come from `Skill` tool calls in Claude transcripts, subagents from the Agent results the
//! scan already counts, the advisor and `sos role` from the repo's `.advise-state/calls.jsonl`.
//! Read-only, and nothing has to be logged by hand. Only names, days and outcomes are kept.

use super::{Args, Filter, Scan};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::{Path, PathBuf};

#[derive(Default, Debug, PartialEq)]
pub struct Count {
    pub runs: u64,
    pub failed: u64,
    /// Transcripts only; `None` for what the repo log counts.
    pub sessions: Option<u64>,
}

type Rows = BTreeMap<(&'static str, String), Count>;

/// The log's two time formats (advisor `2026-10-20T21:12:01`, role `20261020-211201`) as a local day.
fn log_day(ts: &str) -> String {
    match (ts.get(..4), ts.get(4..6), ts.get(6..8)) {
        (Some(y), Some(m), Some(d)) if ts.as_bytes().get(8) == Some(&b'-') => format!("{y}-{m}-{d}"),
        _ => ts.get(..10).unwrap_or("").to_string(),
    }
}

/// Fold one `calls.jsonl` into `rows`; lines that do not parse are not counted.
pub fn add_log(text: &str, f: &Filter, rows: &mut Rows) {
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        if !f.day_ok(&log_day(v.get("ts").and_then(Value::as_str).unwrap_or(""))) {
            continue;
        }
        let key = match v.get("kind").and_then(Value::as_str) {
            Some("role") => ("role", v.get("role").and_then(Value::as_str).unwrap_or("?").to_string()),
            None => ("advisor", "advise".to_string()),
            Some(_) => continue,
        };
        let c = rows.entry(key).or_default();
        c.runs += 1;
        c.failed += u64::from(v.get("ok").and_then(Value::as_bool) == Some(false));
    }
}

/// Fold what a scan saw into `rows`.
pub fn add_scan(scan: &Scan, rows: &mut Rows) {
    let mut sessions: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for u in scan.skills.values() {
        let c = rows.entry(("skill", u.name.clone())).or_default();
        c.runs += 1;
        c.failed += u64::from(u.failed);
        sessions.entry(&u.name).or_default().insert(&u.session);
    }
    for (name, s) in sessions {
        rows.entry(("skill", name.to_string())).or_default().sessions = Some(s.len() as u64);
    }
    for (role, r) in &scan.results {
        if r.results + r.errors > 0 {
            let c = rows.entry(("subagent", role.clone())).or_default();
            c.runs += r.results + r.errors;
            c.failed += r.errors;
        }
    }
}

/// The repo whose log to read: `--project` when it is a folder here, else the one around the cwd.
/// A `--project` that is only a folder name cannot be placed, so no log is read for it.
pub fn log_path(project: Option<&str>) -> Option<PathBuf> {
    let start = match project {
        Some(p) if Path::new(p).is_dir() => PathBuf::from(p),
        Some(_) => return None,
        None => std::env::current_dir().ok()?,
    };
    start.ancestors().map(|d| d.join(".advise-state/calls.jsonl")).find(|p| p.is_file())
}

pub fn render(scans: &[(&'static str, Scan)], a: &Args, f: &Filter) -> String {
    let mut rows = Rows::new();
    for (_, scan) in scans {
        add_scan(scan, &mut rows);
    }
    // A count of zero means "not used" only if the sources were read: say when they were not.
    let mut coverage: Vec<String> = scans.iter().flat_map(|(_, s)| super::table::coverage(s)).collect();
    if scans.iter().all(|(_, s)| s.files == 0) {
        coverage.push("no transcript was read, so an empty row here does not mean unused".into());
    }
    let log = log_path(f.project.as_deref());
    if let Some(p) = &log {
        match std::fs::read(p) {
            Ok(b) => add_log(&String::from_utf8_lossy(&b), f, &mut rows),
            Err(e) => coverage.push(format!("could not read {}: {e}", p.display())),
        }
    }
    let notes = [
        "skills are Skill tool calls in Claude transcripts: a skill the user typed as a slash command is not one, and Codex has no such tool",
        "days are UTC in transcripts, local in .advise-state/calls.jsonl",
    ];
    let log_text = log.as_ref().map(|p| p.display().to_string());
    if a.json {
        let list: Vec<Value> = rows.iter().map(|((kind, name), c)| {
            json!({"kind": kind, "name": name, "runs": c.runs, "failed": c.failed, "sessions": c.sessions})
        }).collect();
        let doc = json!({"meta": {"by": "skill", "since": f.since, "until": f.until, "project": f.project,
                                   "log": log_text, "notes": notes, "coverage_warnings": coverage}, "rows": list});
        return serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n";
    }
    let mut s = String::new();
    let _ = writeln!(s, "use since {}{}{}", f.since, f.until.as_deref().map(|u| format!(" until {u}")).unwrap_or_default(),
                     f.project.as_deref().map(|p| format!(", project {p}")).unwrap_or_default());
    if rows.is_empty() {
        let _ = writeln!(s, "  nothing counted");
    } else {
        let mut lines = vec![["kind", "name", "runs", "failed", "sessions"].map(String::from).to_vec()];
        for ((kind, name), c) in &rows {
            lines.push(vec![kind.to_string(), name.clone(), c.runs.to_string(), c.failed.to_string(),
                            c.sessions.map(|x| x.to_string()).unwrap_or_default()]);
        }
        s.push_str(&super::table::grid(&lines, 2));
    }
    let _ = writeln!(s, "advisor and role log: {}", log_text.as_deref().unwrap_or("none found (run inside the repo, or pass its path as --project)"));
    for n in notes {
        let _ = writeln!(s, "note: {n}");
    }
    for c in &coverage {
        let _ = writeln!(s, "coverage: {c}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_the_log_by_kind_and_day() {
        let f = Filter { since: "2026-10-20".into(), until: Some("2026-10-21".into()), project: None };
        let text = [
            r#"{"ts": "2026-10-19T23:00:00", "ok": true}"#,
            r#"{"ts": "2026-10-20T21:12:01", "ok": true, "question": "x"}"#,
            r#"{"ts": "2026-10-21T01:00:00", "ok": false}"#,
            r#"{"ts": "20261021-090000", "kind": "role", "role": "reviewer", "ok": true}"#,
            r#"{"ts": "20261022-090000", "kind": "role", "role": "reviewer", "ok": true}"#,
            r#"{"ts": "2026-10-21T01:00:00", "kind": "other", "ok": true}"#,
            "not json",
        ].join("\n");
        let mut rows = Rows::new();
        add_log(&text, &f, &mut rows);
        assert_eq!(rows[&("advisor", "advise".into())], Count { runs: 2, failed: 1, sessions: None });
        assert_eq!(rows[&("role", "reviewer".into())], Count { runs: 1, failed: 0, sessions: None });
        assert_eq!(rows.len(), 2);
    }
}
