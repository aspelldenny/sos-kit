//! Claude Code transcripts: `<projects>/<encoded cwd>/<session>.jsonl` and
//! `<session>/subagents/agent-<id>.jsonl` + `.meta.json`.
//!
//! - Usage: `assistant.message.usage`; one API message streams as several lines with the same
//!   `message.id`: the line with the most output tokens counts (as agent-usage.py does).
//! - Dedup is per session, not per file: history repeated on resume or copied to another file of
//!   the same session counts once.
//! - Role: `api-request.querySource` joined by `assistant.requestRef == api-request.id`; else the
//!   subagent meta's `agentType` in the same namespace (`agent:custom:…`, `agent:builtin:…`);
//!   the main file without querySource is `sdk`; nothing known → `unknown`.
//! - A sidechain line in a main file belongs to the subagent file when it exists (dropped there);
//!   without one it is kept, with the meta role or `unknown` (agent-usage.py drops all of them).
//! - `<synthetic>` or no usage: skipped and counted.

use super::{close, entries, lines_of, prices::Tokens, utc_day, utc_key, Call, Filter, LargeContext, Scan};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

const BUILTIN: &[&str] = &["general-purpose", "Explore", "Plan", "statusline-setup", "claude-code-guide", "output-style-setup"];

/// `agentType` in querySource's namespace.
pub fn role_of(agent_type: &str, builtin: Option<bool>) -> String {
    if agent_type.starts_with("agent:") || agent_type == "sdk" {
        return agent_type.to_string();
    }
    if builtin.unwrap_or_else(|| BUILTIN.contains(&agent_type)) {
        format!("agent:builtin:{agent_type}")
    } else {
        format!("agent:custom:{agent_type}")
    }
}

struct Meta {
    role: Option<String>,
    tool_use: Option<String>,
}

fn read_meta(p: &Path) -> Meta {
    let v: Option<Value> = std::fs::read_to_string(p).ok().and_then(|t| serde_json::from_str(&t).ok());
    let s = |k: &str| v.as_ref().and_then(|v| v.get(k)).and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_string);
    Meta {
        role: s("agentType").map(|t| role_of(&t, v.as_ref().and_then(|v| v.get("isBuiltIn")).and_then(Value::as_bool))),
        tool_use: s("toolUseId"),
    }
}

/// Dedup across the files of one run: (session, key) → index in `calls`, its output count, and
/// whether its role came from a source (querySource or meta) rather than a fallback.
#[derive(Default)]
pub struct Dedup {
    seen: HashMap<(String, String), (usize, u64, bool)>,
}

impl Dedup {
    /// A copy with more output replaces the kept one, but a role found from a source is never
    /// replaced by a fallback (`sdk`/`unknown`): a resumed copy may lack the api-request line.
    pub fn add(&mut self, scan: &mut Scan, key: Option<String>, mut call: Call, known: bool) {
        let Some(key) = key else {
            scan.skipped.no_id += 1;
            scan.calls.push(call);
            return;
        };
        let out = call.tokens.output;
        match self.seen.get_mut(&(call.session.clone(), key.clone())) {
            Some((idx, best, was_known)) => {
                scan.dedup_dropped += 1;
                let kept = &mut scan.calls[*idx];
                if out >= *best {
                    if *was_known && !known {
                        call.role = std::mem::take(&mut kept.role);
                    }
                    *best = out;
                    *was_known |= known;
                    *kept = call;
                } else if known && !*was_known {
                    kept.role = call.role;
                    *was_known = true;
                }
            }
            None => {
                self.seen.insert((call.session.clone(), key), (scan.calls.len(), out, known));
                scan.calls.push(call);
            }
        }
    }
}

struct Pending {
    key: Option<String>,
    session: String,
    ts: String,
    cwd: String,
    model: String,
    tokens: Tokens,
    thinking: Option<u64>,
    request_ref: Option<String>,
    sidechain: bool,
    agent_id: Option<String>,
}

/// One Agent/Task tool call and what came back to it.
#[derive(Default, Clone)]
pub struct AgentCall {
    role: String,
    cwd: String,
    /// Async spawn acknowledgments inside --since/--until (by the ack's own day).
    acks: u64,
    /// Last non-acknowledgment result: (UTC time key of the result line, text chars, is_error).
    /// The time key picks the last one across resumed copies whatever order files are read in.
    last: Option<(String, u64, bool)>,
}

/// The subagents of one session: agentId → (has its own .jsonl, meta role).
pub type Subagents = HashMap<String, (bool, Option<String>)>;

/// Where a file sits: the main transcript, or a subagent with its meta.
pub enum Kind<'a> {
    Main { subagents: &'a Subagents },
    Sub { agent_id: String, meta_role: Option<String>, tool_use: Option<String> },
}

fn text_chars(content: &Value) -> u64 {
    match content {
        Value::String(s) => s.chars().count() as u64,
        Value::Array(a) => a.iter()
            .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|b| b.get("text").and_then(Value::as_str))
            .map(|t| t.chars().count() as u64).sum(),
        _ => 0,
    }
}

fn u(v: &Value, path: &[&str]) -> Option<u64> {
    path.iter().try_fold(v, |v, k| v.get(k)).and_then(Value::as_u64)
}

/// Read one transcript from any line source. Pure apart from `scan`/`dedup`, so tests feed lines.
pub fn read(lines: impl Iterator<Item = Option<String>>, file_session: &str, kind: Kind, filter: &Filter,
            large_context: u64, scan: &mut Scan, dedup: &mut Dedup) {
    let mut api: HashMap<String, String> = HashMap::new();
    let mut pending: Vec<Pending> = Vec::new();
    // tool_use_id → (session, call day, call)
    let mut agents: BTreeMap<String, (String, String, AgentCall)> = BTreeMap::new();
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
        let s = |k: &str| e.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let session = Some(s("sessionId")).filter(|x| !x.is_empty()).unwrap_or_else(|| file_session.to_string());
        match e.get("type").and_then(Value::as_str) {
            Some("api-request") => {
                if let (Some(id), Some(q)) = (e.get("id").and_then(Value::as_str), e.get("querySource").and_then(Value::as_str)) {
                    api.insert(id.to_string(), q.to_string());
                }
            }
            Some("assistant") => {
                let null = Value::Null;
                let msg = e.get("message").unwrap_or(&null);
                for c in msg.get("content").and_then(Value::as_array).into_iter().flatten() {
                    let is_agent = matches!(c.get("name").and_then(Value::as_str), Some("Agent" | "Task"));
                    if c.get("type").and_then(Value::as_str) == Some("tool_use") && is_agent {
                        if let Some(id) = c.get("id").and_then(Value::as_str) {
                            let t = c.pointer("/input/subagent_type").and_then(Value::as_str).unwrap_or("general-purpose");
                            agents.insert(id.to_string(), (session.clone(), utc_day(&s("timestamp")),
                                                           AgentCall { role: role_of(t, None), cwd: s("cwd"), ..Default::default() }));
                        }
                    }
                }
                let model = msg.get("model").and_then(Value::as_str).unwrap_or("?").to_string();
                let Some(usage) = msg.get("usage").filter(|u| u.is_object()) else {
                    scan.skipped.synthetic += 1;
                    continue;
                };
                if model == "<synthetic>" {
                    scan.skipped.synthetic += 1;
                    continue;
                }
                let tokens = Tokens {
                    input: u(usage, &["input_tokens"]).unwrap_or(0),
                    cache_write: u(usage, &["cache_creation_input_tokens"]).unwrap_or(0),
                    cache_write_1h: u(usage, &["cache_creation", "ephemeral_1h_input_tokens"]).unwrap_or(0),
                    cache_read: u(usage, &["cache_read_input_tokens"]).unwrap_or(0),
                    output: u(usage, &["output_tokens"]).unwrap_or(0),
                };
                let key = msg.get("id").and_then(Value::as_str).or_else(|| e.get("requestRef").and_then(Value::as_str))
                    .or_else(|| e.get("requestId").and_then(Value::as_str)).map(str::to_string);
                pending.push(Pending {
                    key, session, ts: s("timestamp"), cwd: s("cwd"), model, tokens,
                    thinking: u(usage, &["output_tokens_details", "thinking_tokens"]),
                    request_ref: e.get("requestRef").and_then(Value::as_str).map(str::to_string),
                    sidechain: e.get("isSidechain").and_then(Value::as_bool).unwrap_or(false),
                    agent_id: e.get("agentId").and_then(Value::as_str).map(str::to_string),
                });
            }
            Some("user") => {
                let is_async = e.pointer("/toolUseResult/isAsync").and_then(Value::as_bool).unwrap_or(false);
                let line_key = utc_key(&s("timestamp"));
                for c in e.pointer("/message/content").and_then(Value::as_array).into_iter().flatten() {
                    if c.get("type").and_then(Value::as_str) != Some("tool_result") {
                        continue;
                    }
                    let Some((_, call_day, a)) = c.get("tool_use_id").and_then(Value::as_str).and_then(|id| agents.get_mut(id)) else { continue };
                    // The result's own day; a line without a timestamp falls back to the call's.
                    let key = if line_key.is_empty() { call_day.clone() } else { line_key.clone() };
                    if is_async {
                        if filter.day_ok(&key[..key.len().min(10)]) {
                            a.acks += 1;
                        }
                    } else {
                        let err = c.get("is_error").and_then(Value::as_bool).unwrap_or(false);
                        // Within a file, lines are in order: a later line is the later result.
                        a.last = Some((key, text_chars(c.get("content").unwrap_or(&Value::Null)), err));
                    }
                }
            }
            _ => {}
        }
    }
    // The subagent's real first call, before any filter: the heuristic is about how it started.
    let first = pending.iter().map(|p| (utc_key(&p.ts), p)).filter(|(k, _)| !k.is_empty()).min_by(|a, b| a.0.cmp(&b.0)).map(|(_, p)| p)
        .map(|p| (utc_day(&p.ts), p.cwd.clone(), p.tokens.input + p.tokens.cache_write + p.tokens.cache_read, p.request_ref.clone()));
    let mut mismatch = 0;
    for p in pending {
        let day = utc_day(&p.ts);
        if !filter.day_ok(&day) || !filter.project_ok(&p.cwd) {
            continue;
        }
        let qs = p.request_ref.as_ref().and_then(|r| api.get(r)).cloned();
        let (role, known) = match &kind {
            Kind::Main { subagents } if p.sidechain => {
                let sub = p.agent_id.as_ref().and_then(|a| subagents.get(a));
                if sub.is_some_and(|(has_file, _)| *has_file) {
                    scan.skipped.sidechain_owned += 1;
                    continue;
                }
                let meta = sub.and_then(|(_, m)| m.clone());
                match qs.or(meta) {
                    Some(r) => (r, true),
                    None => ("unknown".into(), false),
                }
            }
            Kind::Main { .. } => match qs {
                Some(r) => (r, true),
                None => ("sdk".into(), false),
            },
            Kind::Sub { meta_role, .. } => {
                if let (Some(q), Some(m)) = (&qs, meta_role) {
                    if q != m {
                        mismatch += 1;
                    }
                }
                match qs.or_else(|| meta_role.clone()) {
                    Some(r) => (r, true),
                    None => ("unknown".into(), false),
                }
            }
        };
        let call = Call { session: p.session, role, model: p.model, project: p.cwd, day, tokens: p.tokens, thinking: p.thinking };
        dedup.add(scan, p.key, call, known);
    }
    if let Kind::Sub { agent_id, tool_use, meta_role } = &kind {
        if mismatch > 0 {
            scan.explain.push(format!("subagent {agent_id}: querySource and meta agentType ({}) differ on {mismatch} call(s); querySource used",
                                      meta_role.as_deref().unwrap_or("-")));
        }
        if let Some((day, cwd, read, rref)) = first {
            if read > large_context && filter.day_ok(&day) && filter.project_ok(&cwd) {
                let role = rref.and_then(|r| api.get(&r).cloned()).or_else(|| meta_role.clone()).unwrap_or_else(|| "unknown".into());
                scan.large.push(LargeContext { session: file_session.to_string(), agent_id: agent_id.clone(), role,
                                               first_call_input_tokens: read, parent_tool_use: tool_use.clone().unwrap_or_else(|| "unknown".into()) });
            }
        }
    }
    for (id, (session, _, a)) in agents {
        match scan.agents.get_mut(&(session.clone(), id.clone())) {
            // A copy of the same history (resume): one call, its last result once.
            Some(old) => {
                old.acks = old.acks.max(a.acks);
                // The later result by time; on a tie (or no time) the copy read first stays, so
                // the outcome does not depend on file order.
                if let Some(new) = a.last {
                    if old.last.as_ref().is_none_or(|o| new.0 > o.0) {
                        old.last = Some(new);
                    }
                }
            }
            None => {
                scan.agents.insert((session, id), a);
            }
        }
    }
}

/// Totals of what came back to the Agent calls, once every file is read.
pub fn finish(scan: &mut Scan, filter: &Filter) {
    for a in std::mem::take(&mut scan.agents).into_values() {
        if !filter.project_ok(&a.cwd) {
            continue;
        }
        let r = scan.results.entry(a.role).or_default();
        r.spawn_acks += a.acks;
        match a.last {
            Some((key, _, true)) if filter.day_ok(&key[..key.len().min(10)]) => r.errors += 1,
            Some((key, chars, false)) if filter.day_ok(&key[..key.len().min(10)]) => {
                r.results += 1;
                r.subagent_result_text_chars += chars;
            }
            _ => {}
        }
    }
}

fn mtime_day(p: &Path) -> Option<String> {
    let m = std::fs::metadata(p).ok()?.modified().ok()?;
    let age = std::time::SystemTime::now().duration_since(m).ok()?.as_secs() / 86_400;
    Some(super::days_ago(age))
}

#[allow(clippy::too_many_arguments)] // one call site; a struct would only rename the list
fn read_file(path: &Path, session: &str, kind: Kind, filter: &Filter, max: usize, large_context: u64, scan: &mut Scan, dedup: &mut Dedup) {
    scan.files += 1;
    if let Some(mut lines) = lines_of(path, max, scan) {
        read(&mut lines, session, kind, filter, large_context, scan, dedup);
        close(lines, path, scan);
    }
}

pub fn scan_dir(dir: &Path, filter: &Filter, max: usize, large_context: u64) -> anyhow::Result<Scan> {
    let mut scan = Scan::default();
    let mut dedup = Dedup::default();
    if !dir.is_dir() {
        scan.coverage.push(format!("no Claude transcripts folder at {}", dir.display()));
        return Ok(scan);
    }
    // A file last written before --since holds nothing newer.
    let old = |p: &Path| mtime_day(p).is_some_and(|d| d < filter.since);
    let is = |p: &Path, ext: &str| p.extension().is_some_and(|e| e == ext);
    for project in entries(dir, &mut scan).into_iter().filter(|p| p.is_dir()) {
        for main in entries(&project, &mut scan).into_iter().filter(|p| is(p, "jsonl")) {
            let stem = main.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            let session_dir = project.join(&stem);
            let subdir = session_dir.join("subagents");
            let mut subagents = Subagents::new();
            let mut metas: HashMap<String, Meta> = HashMap::new();
            let mut files = Vec::new();
            // Built paths are checked too: `entries` only sees links inside a folder it lists.
            let linked = [&session_dir, &subdir].into_iter().find(|p| std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink()));
            if let Some(l) = linked {
                scan.coverage.push(format!("symlink not followed: {}", l.display()));
            }
            for f in if linked.is_some() { Vec::new() } else { entries(&subdir, &mut scan) } {
                let name = f.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let Some(rest) = name.strip_prefix("agent-") else { continue };
                if let Some(id) = rest.strip_suffix(".meta.json") {
                    let meta = read_meta(&f);
                    subagents.entry(id.to_string()).or_insert((false, None)).1 = meta.role.clone();
                    metas.insert(id.to_string(), meta);
                } else if let Some(id) = rest.strip_suffix(".jsonl") {
                    subagents.entry(id.to_string()).or_insert((false, None)).0 = true;
                    files.push((f.clone(), id.to_string()));
                }
            }
            if !old(&main) {
                read_file(&main, &stem, Kind::Main { subagents: &subagents }, filter, max, large_context, &mut scan, &mut dedup);
            }
            for (path, id) in files {
                if old(&path) {
                    continue;
                }
                // Only a meta file `entries` listed (never one behind a link).
                let meta = metas.remove(&id).unwrap_or(Meta { role: None, tool_use: None });
                read_file(&path, &stem, Kind::Sub { agent_id: id, meta_role: meta.role, tool_use: meta.tool_use },
                          filter, max, large_context, &mut scan, &mut dedup);
            }
        }
    }
    finish(&mut scan, filter);
    Ok(scan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f() -> Filter {
        Filter { since: "2026-01-01".into(), until: None, project: None }
    }

    fn asst(id: &str, out: u64, extra: &str) -> String {
        format!(r#"{{"type":"assistant","sessionId":"S","timestamp":"2026-11-05T10:00:00Z","cwd":"/w/app",{extra}"message":{{"id":"{id}","model":"claude-opus-5-5","usage":{{"input_tokens":3,"cache_creation_input_tokens":100,"cache_creation":{{"ephemeral_5m_input_tokens":60,"ephemeral_1h_input_tokens":40}},"cache_read_input_tokens":1000,"output_tokens":{out}}}}}}}"#)
    }

    fn run(lines: &[String], kind: Kind) -> Scan {
        run_with(&[lines], kind, &f())
    }

    /// Several files of one session, read in order, then finished.
    fn run_with(files: &[&[String]], kind: Kind, filter: &Filter) -> Scan {
        let mut scan = Scan::default();
        let mut d = Dedup::default();
        for lines in files {
            let k = match &kind {
                Kind::Main { subagents } => Kind::Main { subagents },
                Kind::Sub { agent_id, meta_role, tool_use } => Kind::Sub { agent_id: agent_id.clone(), meta_role: meta_role.clone(), tool_use: tool_use.clone() },
            };
            read(lines.iter().cloned().map(Some), "S", k, filter, 50_000, &mut scan, &mut d);
        }
        finish(&mut scan, filter);
        scan
    }

    #[test]
    fn streamed_message_counts_once_with_the_largest_output_and_query_source_role() {
        let owners = Subagents::new();
        let lines = vec![
            r#"{"type":"api-request","id":"R1","querySource":"agent:custom:tho"}"#.to_string(),
            asst("m1", 8, r#""requestRef":"R1","#),
            asst("m1", 40261, r#""requestRef":"R1","#),
            asst("m1", 12, r#""requestRef":"R1","#),
            asst("m2", 5, ""),
            r#"{"type":"assistant","sessionId":"S","timestamp":"2026-11-05T10:00:00Z","message":{"id":"x","model":"<synthetic>","usage":{"output_tokens":1}}}"#.to_string(),
            "{not json".to_string(),
        ];
        let s = run(&lines, Kind::Main { subagents: &owners });
        assert_eq!(s.calls.len(), 2);
        assert_eq!((s.calls[0].role.as_str(), s.calls[0].tokens.output), ("agent:custom:tho", 40261));
        assert_eq!(s.calls[0].tokens.cache_write_1h, 40);
        assert_eq!(s.calls[1].role, "sdk");
        assert_eq!((s.dedup_dropped, s.skipped.synthetic, s.skipped.bad_json), (2, 1, 1));
    }

    #[test]
    fn sidechain_belongs_to_the_subagent_file_when_there_is_one() {
        let mut owners = Subagents::new();
        owners.insert("a1".to_string(), (true, Some("agent:custom:worker".to_string())));
        owners.insert("a2".to_string(), (false, Some("agent:custom:reviewer".to_string())));
        let lines = vec![asst("m1", 1, r#""isSidechain":true,"agentId":"a1","#), asst("m2", 1, r#""isSidechain":true,"agentId":"zz","#),
                         asst("m3", 1, r#""isSidechain":true,"agentId":"a2","#)];
        let s = run(&lines, Kind::Main { subagents: &owners });
        assert_eq!(s.skipped.sidechain_owned, 1);
        assert_eq!(s.calls.len(), 2);
        assert_eq!(s.calls[0].role, "unknown", "no file, no meta: kept as unknown");
        assert_eq!(s.calls[1].role, "agent:custom:reviewer", "transcript gone, meta still there: kept with the meta role");
    }

    #[test]
    fn subagent_role_falls_back_to_meta_and_flags_large_first_context() {
        let lines = [asst("m1", 1, "")];
        let mut s = Scan::default();
        let mut d = Dedup::default();
        read(lines.iter().cloned().map(Some), "S", Kind::Sub { agent_id: "a1".into(), meta_role: Some(role_of("worker", Some(false))), tool_use: Some("toolu_1".into()) },
             &f(), 1000, &mut s, &mut d);
        // The first call is the real first one, not the first inside --since.
        let early = asst("m0", 1, "").replace("2026-11-05T10", "2026-10-31T10").replace(r#""cache_read_input_tokens":1000"#, r#""cache_read_input_tokens":0"#)
            .replace(r#""cache_creation_input_tokens":100"#, r#""cache_creation_input_tokens":0"#);
        let late = asst("m1", 1, "").replace(r#""input_tokens":3"#, r#""input_tokens":90000"#);
        let since = Filter { since: "2026-11-01".into(), until: None, project: None };
        let sub = Kind::Sub { agent_id: "a9".into(), meta_role: None, tool_use: None };
        assert!(run_with(&[&[early.clone(), late.clone()][..]], sub, &since).large.is_empty(), "a mid-run call is not an initial context");
        // Earliest by instant, not by string: 00:30+07:00 on the 3rd is before 23:00Z on the 2nd.
        let early = early.replace("2026-10-31T10:00:00Z", "2026-11-03T00:30:00+07:00");
        let late = late.replace("2026-11-05T10:00:00Z", "2026-11-02T23:00:00Z");
        let sub = Kind::Sub { agent_id: "a9".into(), meta_role: None, tool_use: None };
        assert!(run_with(&[&[late, early][..]], sub, &f()).large.is_empty(), "offset timestamps are compared as instants");
        assert_eq!(s.calls[0].role, "agent:custom:worker");
        assert_eq!(s.large.len(), 1);
        assert_eq!((s.large[0].first_call_input_tokens, s.large[0].parent_tool_use.as_str()), (1103, "toolu_1"));
        assert_eq!(role_of("Explore", None), "agent:builtin:Explore");
    }

    #[test]
    fn agent_results_split_into_final_text_acks_and_errors() {
        let owners = Subagents::new();
        let call = |id: &str, t: &str| format!(r#"{{"type":"assistant","sessionId":"S","timestamp":"2026-11-05T10:00:00Z","cwd":"/w","message":{{"id":"c{id}","model":"m","content":[{{"type":"tool_use","id":"{id}","name":"Agent","input":{{"subagent_type":"{t}","prompt":"SOS-SECRET"}}}}],"usage":{{"output_tokens":1}}}}}}"#);
        let res = |id: &str, content: &str, extra: &str| format!(r#"{{"type":"user",{extra}"message":{{"content":[{{"type":"tool_result","tool_use_id":"{id}","content":{content}}}]}}}}"#);
        let lines = vec![
            call("t1", "worker"), call("t2", "worker"), call("t3", "Explore"),
            res("t1", r#""fifteen chars!!""#, ""),
            res("t2", r#""Async agent launched""#, r#""toolUseResult":{"isAsync":true},"#),
            res("t3", r#"[{"type":"text","text":"abc"},{"type":"image"},{"type":"text","text":"de"}]"#, ""),
            res("t3", r#""boom""#, "").replace(r#""content":"boom""#, r#""content":"boom","is_error":true"#),
        ];
        let s = run(&lines, Kind::Main { subagents: &owners });
        let w = &s.results["agent:custom:worker"];
        assert_eq!((w.results, w.subagent_result_text_chars, w.spawn_acks, w.errors), (1, 15, 1, 0));
        let e = &s.results["agent:builtin:Explore"];
        assert_eq!((e.results, e.errors), (0, 1), "the last result decides: an error after text");
        // The same history copied into a resumed file of the session counts once.
        let s = run_with(&[&lines[..], &lines[..]], Kind::Main { subagents: &owners }, &f());
        let w = &s.results["agent:custom:worker"];
        assert_eq!((w.results, w.subagent_result_text_chars, w.spawn_acks), (1, 15, 1));
        // An older copy that stopped at an error, and the full history: the later result wins in
        // either file order.
        let at = |ts: &str, l: &str| l.replacen(r#"{"type":"user","#, &format!(r#"{{"type":"user","timestamp":"{ts}","#), 1);
        let full = [lines[0].clone(), at("2026-11-04T08:00:00Z", &res("t1", r#""boom""#, "").replace(r#""content":"boom""#, r#""content":"boom","is_error":true"#)),
                    at("2026-11-06T08:00:00+07:00", &res("t1", r#""fifteen chars!!""#, ""))];
        let old = [full[0].clone(), full[1].clone()];
        for files in [[&full[..], &old[..]], [&old[..], &full[..]]] {
            let s = run_with(&files, Kind::Main { subagents: &owners }, &f());
            let w = &s.results["agent:custom:worker"];
            assert_eq!((w.results, w.errors, w.subagent_result_text_chars), (1, 0, 15));
        }
    }

    #[test]
    fn a_result_counts_on_its_own_day() {
        let owners = Subagents::new();
        let call = r#"{"type":"assistant","sessionId":"S","timestamp":"2026-11-03T10:00:00Z","cwd":"/w","message":{"id":"c","model":"m","content":[{"type":"tool_use","id":"t1","name":"Agent","input":{"subagent_type":"worker"}}],"usage":{"output_tokens":1}}}"#.to_string();
        let res = r#"{"type":"user","timestamp":"2026-11-06T01:00:00Z","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"done"}]}}"#.to_string();
        let lines = [call, res];
        let until5 = Filter { since: "2026-11-01".into(), until: Some("2026-11-05".into()), project: None };
        assert!(run_with(&[&lines[..]], Kind::Main { subagents: &owners }, &until5).results.get("agent:custom:worker").is_none_or(|r| r.results == 0));
        let from6 = Filter { since: "2026-11-06".into(), until: None, project: None };
        assert_eq!(run_with(&[&lines[..]], Kind::Main { subagents: &owners }, &from6).results["agent:custom:worker"].results, 1);
    }

    #[test]
    fn a_resumed_copy_without_the_request_line_keeps_the_known_role() {
        let owners = Subagents::new();
        let a = [r#"{"type":"api-request","id":"R1","querySource":"agent:custom:worker"}"#.to_string(), asst("m1", 5, r#""requestRef":"R1","#)];
        let b = [asst("m1", 5, r#""requestRef":"R1","#)];
        for files in [[&a[..], &b[..]], [&b[..], &a[..]]] {
            let s = run_with(&files, Kind::Main { subagents: &owners }, &f());
            assert_eq!(s.calls.len(), 1);
            assert_eq!(s.calls[0].role, "agent:custom:worker");
        }
    }
}
