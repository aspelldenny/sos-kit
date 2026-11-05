//! Codex rollouts: `<sessions>/YYYY/MM/DD/rollout-*.jsonl`.
//!
//! - Usage: `token_usage_record.payload.usage`, one per response, deduped by `response_id`;
//!   a compaction response is such a record and counts once; its copy inside `compacted` is
//!   ignored. Adding up `token_count.last_token_usage` instead over-counts (repeated totals) and
//!   misses compaction (review 05/11/2026, usage pass, point 1).
//! - A rollout with no `token_usage_record` at all falls back to the differences of
//!   `token_count.info.total_token_usage`; a total that goes down is a counter reset.
//! - `input_tokens` includes `cached_input_tokens` (checked on every record on 05/11/2026), so the
//!   uncached part is input − cached − cache_write; `reasoning_output_tokens` is part of output.
//! - Model: `turn_context.payload.model` of the response's turn. Role: `main`, or for a subagent
//!   `session_meta.payload.source.subagent.thread_spawn.agent_role` (`unknown` when null).
//!   `originator` is only the app that ran it, not a role.

use super::{close, entries, lines_of, prices::Tokens, utc_day, Call, Filter, Scan};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn tokens(u: &Value) -> (Tokens, Option<u64>) {
    let g = |k: &str| u.get(k).and_then(Value::as_u64).unwrap_or(0);
    let (input, cached, write) = (g("input_tokens"), g("cached_input_tokens"), g("cache_write_input_tokens"));
    (Tokens { input: input.saturating_sub(cached + write), cache_write: write, cache_write_1h: 0, cache_read: cached, output: g("output_tokens") },
     u.get("reasoning_output_tokens").and_then(Value::as_u64))
}

struct Rec {
    day: String,
    turn: String,
    model_at: String,
    tokens: Tokens,
    thinking: Option<u64>,
}

pub fn read(lines: impl Iterator<Item = Option<String>>, file_name: &str, filter: &Filter, scan: &mut Scan) {
    let mut session = String::new();
    let mut cwd = String::new();
    let mut role = "main".to_string();
    let mut parent: Option<String> = None;
    let mut model_of_turn: HashMap<String, String> = HashMap::new();
    let mut model = String::from("?");
    let mut records: Vec<Rec> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut dup = 0u64;
    let mut totals: Vec<(String, String, Value)> = Vec::new(); // (day, model, total_token_usage)
    for line in lines {
        let Some(line) = line else {
            scan.skipped.oversize_lines += 1;
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(e) = serde_json::from_str::<Value>(&line) else {
            scan.skipped.bad_json += 1;
            continue;
        };
        let day = utc_day(e.get("timestamp").and_then(Value::as_str).unwrap_or(""));
        let p = e.get("payload").unwrap_or(&Value::Null);
        match e.get("type").and_then(Value::as_str) {
            Some("session_meta") => {
                session = p.get("id").or_else(|| p.get("session_id")).and_then(Value::as_str).unwrap_or("").to_string();
                cwd = p.get("cwd").and_then(Value::as_str).unwrap_or("").to_string();
                if let Some(ts) = p.pointer("/source/subagent/thread_spawn") {
                    role = ts.get("agent_role").and_then(Value::as_str).filter(|r| !r.is_empty()).unwrap_or("unknown").to_string();
                    parent = ts.get("parent_thread_id").and_then(Value::as_str).map(str::to_string);
                }
            }
            Some("turn_context") => {
                if let Some(m) = p.get("model").and_then(Value::as_str) {
                    model = m.to_string();
                    if let Some(t) = p.get("turn_id").and_then(Value::as_str) {
                        model_of_turn.insert(t.to_string(), m.to_string());
                    }
                }
            }
            Some("token_usage_record") => {
                let Some(u) = p.get("usage") else { continue };
                let (t, th) = tokens(u);
                let rec = Rec { day, turn: p.get("turn_id").and_then(Value::as_str).unwrap_or("").to_string(), model_at: model.clone(), tokens: t, thinking: th };
                match p.get("response_id").and_then(Value::as_str) {
                    Some(id) if seen.contains_key(id) => dup += 1,
                    Some(id) => {
                        seen.insert(id.to_string(), records.len());
                        records.push(rec);
                    }
                    None => {
                        scan.skipped.no_id += 1;
                        records.push(rec);
                    }
                }
            }
            Some("event_msg") if p.get("type").and_then(Value::as_str) == Some("token_count") => {
                if let Some(t) = p.pointer("/info/total_token_usage") {
                    totals.push((day, model.clone(), t.clone()));
                }
            }
            _ => {}
        }
    }
    let session = if session.is_empty() { file_name.to_string() } else { session };
    scan.dedup_dropped += dup;
    let mut calls: Vec<(String, String, Tokens, Option<u64>)> = Vec::new();
    let source;
    if !records.is_empty() {
        source = format!("token_usage_record: {} response(s), {dup} duplicate(s) dropped", records.len());
        for r in records {
            let m = model_of_turn.get(&r.turn).cloned().unwrap_or(r.model_at);
            calls.push((r.day, m, r.tokens, r.thinking));
        }
    } else if !totals.is_empty() {
        let mut prev: Option<Value> = None;
        let mut resets = 0;
        for (day, m, t) in totals {
            let keys = ["input_tokens", "cached_input_tokens", "cache_write_input_tokens", "output_tokens", "reasoning_output_tokens"];
            let delta = match &prev {
                None => t.clone(),
                Some(pv) => {
                    let g = |v: &Value, k: &str| v.get(k).and_then(Value::as_u64).unwrap_or(0);
                    // A field missing on either side is unknown, not 0: it neither signals a reset
                    // nor becomes a known delta.
                    let both = |k: &str| t.get(k).is_some() && pv.get(k).is_some();
                    if keys.iter().any(|k| both(k) && g(&t, k) < g(pv, k)) {
                        resets += 1;
                        t.clone()
                    } else {
                        // A field only counts as known when both totals carry it (thinking stays
                        // unknown rather than becoming a known 0).
                        Value::Object(keys.iter().filter(|k| **k != "reasoning_output_tokens" || both(k))
                            .map(|k| (k.to_string(), Value::from(g(&t, k) - g(pv, k)))).collect())
                    }
                }
            };
            prev = Some(t);
            let (tk, th) = tokens(&delta);
            if tk.input + tk.cache_read + tk.cache_write + tk.output > 0 {
                calls.push((day, m, tk, th));
            }
        }
        source = format!("token_count totals (no token_usage_record), {resets} counter reset(s)");
        if resets > 0 {
            scan.coverage.push(format!("codex {session}: {resets} token counter reset(s) in a rollout without per-response records"));
        }
    } else {
        source = "no usage".into();
    }
    scan.explain.push(format!("codex {session} ({role}{}): {source}", parent.map(|p| format!(", parent {p}")).unwrap_or_default()));
    if !filter.project_ok(&cwd) {
        return;
    }
    for (day, m, t, th) in calls {
        if filter.day_ok(&day) {
            scan.calls.push(Call { session: session.clone(), role: role.clone(), model: m, project: cwd.clone(), day, tokens: t, thinking: th });
        }
    }
}

/// Rollout files under `dir`; symlinks are not followed (see `entries`).
fn walk(dir: &Path, out: &mut Vec<PathBuf>, scan: &mut Scan) {
    for p in entries(dir, scan) {
        if p.is_dir() {
            walk(&p, out, scan);
        } else if p.file_name().is_some_and(|n| { let n = n.to_string_lossy(); n.starts_with("rollout-") && n.ends_with(".jsonl") }) {
            out.push(p);
        }
    }
}

pub fn scan_dir(dir: &Path, filter: &Filter, max: usize) -> anyhow::Result<Scan> {
    let mut scan = Scan::default();
    if !dir.is_dir() {
        scan.coverage.push(format!("no Codex sessions folder at {}", dir.display()));
        return Ok(scan);
    }
    let mut files = Vec::new();
    walk(dir, &mut files, &mut scan);
    for f in files {
        let modified = std::fs::metadata(&f).ok().and_then(|m| m.modified().ok());
        let age = modified.and_then(|m| std::time::SystemTime::now().duration_since(m).ok()).map(|d| d.as_secs() / 86_400);
        if age.is_some_and(|a| super::days_ago(a) < filter.since) {
            continue;
        }
        scan.files += 1;
        let name = f.file_stem().unwrap_or_default().to_string_lossy().into_owned();
        if let Some(mut lines) = lines_of(&f, max, &mut scan) {
            read(&mut lines, &name, filter, &mut scan);
            close(lines, &f, &mut scan);
        }
    }
    Ok(scan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f() -> Filter {
        Filter { since: "2026-01-01".into(), until: None, project: None }
    }
    fn rec(resp: &str, turn: &str, input: u64, cached: u64, out: u64) -> String {
        format!(r#"{{"timestamp":"2026-10-27T10:00:00Z","type":"token_usage_record","payload":{{"turn_id":"{turn}","response_id":"{resp}","usage":{{"input_tokens":{input},"cached_input_tokens":{cached},"cache_write_input_tokens":0,"output_tokens":{out},"reasoning_output_tokens":2}}}}}}"#)
    }
    fn count(i: u64, c: u64, o: u64) -> String {
        format!(r#"{{"timestamp":"2026-10-27T10:00:00Z","type":"event_msg","payload":{{"type":"token_count","info":{{"total_token_usage":{{"input_tokens":{i},"cached_input_tokens":{c},"cache_write_input_tokens":0,"output_tokens":{o},"reasoning_output_tokens":0}},"last_token_usage":{{"input_tokens":{i},"cached_input_tokens":0,"output_tokens":{o}}}}}}}}}"#)
    }
    const META_CHILD: &str = r#"{"timestamp":"2026-10-27T09:00:00Z","type":"session_meta","payload":{"id":"T2","cwd":"/w/app","originator":"Codex Desktop","source":{"subagent":{"thread_spawn":{"parent_thread_id":"T1","depth":1,"agent_role":null}}}}}"#;

    fn run(lines: &[String]) -> Scan {
        let mut s = Scan::default();
        read(lines.iter().cloned().map(Some), "rollout-x", &f(), &mut s);
        s
    }

    #[test]
    fn records_dedup_by_response_and_count_compaction_once_not_token_count() {
        let lines = vec![
            META_CHILD.to_string(),
            r#"{"timestamp":"2026-10-27T09:00:01Z","type":"turn_context","payload":{"turn_id":"u1","model":"gpt-6-astra"}}"#.to_string(),
            rec("r1", "u1", 1000, 900, 50),
            count(1000, 900, 50), count(1000, 900, 50), // repeated total: not a source here
            rec("r1", "u1", 1000, 900, 50),            // same response again
            r#"{"timestamp":"2026-10-27T09:30:00Z","type":"turn_context","payload":{"turn_id":"u2","model":"gpt-6"}}"#.to_string(),
            rec("rc", "u2", 244293, 0, 1911),           // compaction response
            r#"{"timestamp":"2026-10-27T10:00:00Z","type":"compacted","payload":{"latest_token_usage_record":{"response_id":"rc","usage":{"input_tokens":244293,"output_tokens":1911}}}}"#.to_string(),
        ];
        let s = run(&lines);
        assert_eq!(s.calls.len(), 2);
        assert_eq!(s.dedup_dropped, 1);
        assert_eq!((s.calls[0].role.as_str(), s.calls[0].model.as_str()), ("unknown", "gpt-6-astra"));
        assert_eq!((s.calls[0].tokens.input, s.calls[0].tokens.cache_read, s.calls[0].thinking), (100, 900, Some(2)));
        assert_eq!((s.calls[1].model.as_str(), s.calls[1].tokens.input + s.calls[1].tokens.output), ("gpt-6", 246204));
        assert!(s.explain[0].contains("parent T1"), "{:?}", s.explain);
    }

    #[test]
    fn fallback_keeps_missing_thinking_unknown() {
        let c = |i: u64| format!(r#"{{"timestamp":"2026-10-27T10:00:00Z","type":"event_msg","payload":{{"type":"token_count","info":{{"total_token_usage":{{"input_tokens":{i},"cached_input_tokens":0,"output_tokens":1}}}}}}}}"#);
        let mut s = Scan::default();
        read([c(10), c(25)].into_iter().map(Some), "rollout-x", &f(), &mut s);
        assert_eq!(s.calls.len(), 2);
        assert!(s.calls.iter().all(|c| c.thinking.is_none()), "no reasoning field: unknown, not 0");
        // Present on one side only: no false reset, the other counters still a difference.
        let r = |i: u64, o: u64, th: &str| format!(r#"{{"timestamp":"2026-10-27T10:00:00Z","type":"event_msg","payload":{{"type":"token_count","info":{{"total_token_usage":{{"input_tokens":{i},"output_tokens":{o}{th}}}}}}}}}"#);
        for (a, b) in [(r(10, 5, r#","reasoning_output_tokens":2"#), r(25, 6, "")), (r(10, 5, ""), r(25, 6, r#","reasoning_output_tokens":2"#))] {
            let mut s = Scan::default();
            read([a, b].into_iter().map(Some), "rollout-y", &f(), &mut s);
            let tot: (u64, u64) = s.calls.iter().fold((0, 0), |x, c| (x.0 + c.tokens.input, x.1 + c.tokens.output));
            assert_eq!(tot, (25, 6), "a missing field is not a reset");
            assert!(s.calls[1].thinking.is_none());
            assert!(s.coverage.is_empty(), "{:?}", s.coverage);
        }
    }

    #[test]
    fn fallback_uses_total_differences_and_survives_a_reset() {
        let lines = vec![
            r#"{"timestamp":"2026-10-27T09:00:00Z","type":"session_meta","payload":{"id":"T1","cwd":"/w/app","source":"exec"}}"#.to_string(),
            count(100, 0, 10), count(100, 0, 10), count(250, 50, 30), count(40, 0, 5),
        ];
        let s = run(&lines);
        let total: u64 = s.calls.iter().map(|c| c.tokens.input + c.tokens.cache_read + c.tokens.output).sum();
        // 110 + (150 + 20) + reset 45 = 325
        assert_eq!(total, 325);
        assert_eq!(s.calls[0].role, "main");
        assert_eq!(s.coverage.len(), 1);
    }
}
