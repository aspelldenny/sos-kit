//! The table, `--json`, `--explain` and the suggestions. Counts go in rows; anything estimated
//! (money, result tokens, heuristics) is labelled as such and kept apart.

use super::prices::{cost, Prices, Tokens};
use super::{Args, Filter, Scan};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

#[derive(Default, Clone)]
pub struct Row {
    pub calls: u64,
    pub tokens: Tokens,
    /// Sum over the calls that report it; `thinking_calls` says how many did.
    pub thinking: Option<u64>,
    pub thinking_calls: u64,
    pub usd: Option<f64>,
    priced: bool,
}

fn total(t: &Tokens) -> u64 {
    t.input + t.cache_write + t.cache_read + t.output
}

fn add(a: &mut Tokens, b: &Tokens) {
    a.input += b.input;
    a.cache_write += b.cache_write;
    a.cache_write_1h += b.cache_write_1h;
    a.cache_read += b.cache_read;
    a.output += b.output;
}

/// Rows keyed by (group, model); `--by model` has one group per model.
pub fn rows(scan: &Scan, by: &str, prices: &Prices) -> BTreeMap<(String, String), Row> {
    let mut out: BTreeMap<(String, String), Row> = BTreeMap::new();
    for c in &scan.calls {
        let key = match by {
            "model" => c.model.clone(),
            "project" => c.project.clone(),
            "day" => c.day.clone(),
            "session" => c.session.clone(),
            _ => c.role.clone(),
        };
        let model = if by == "model" { String::new() } else { c.model.clone() };
        let r = out.entry((key, model)).or_default();
        if r.calls == 0 {
            r.priced = true;
        }
        r.calls += 1;
        add(&mut r.tokens, &c.tokens);
        if let Some(t) = c.thinking {
            r.thinking = Some(r.thinking.unwrap_or(0) + t);
            r.thinking_calls += 1;
        }
        match prices.get(&c.model) {
            Some(p) if r.priced => r.usd = Some(r.usd.unwrap_or(0.0) + cost(p, &c.tokens)),
            _ => {
                r.priced = false;
                r.usd = None;
            }
        }
    }
    out
}

pub fn n(v: u64) -> String {
    let s = v.to_string();
    let mut o = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            o.push(',');
        }
        o.push(ch);
    }
    o
}

/// "—" when no call reports thinking, the sum when all do, "N+" when only some do.
fn thinking_cell(r: &Row) -> String {
    match r.thinking {
        None => "—".into(),
        Some(t) if r.thinking_calls < r.calls => format!("{}+", n(t)),
        Some(t) => n(t),
    }
}

fn usd(v: Option<f64>) -> String {
    match v {
        Some(x) => {
            let s = format!("{x:.1}");
            let (int, frac) = s.split_once('.').unwrap_or((&s, "0"));
            format!("{}.{frac}", n(int.parse().unwrap_or(0)))
        }
        None => "—".into(),
    }
}

pub fn grid(lines: &[Vec<String>], left: usize) -> String {
    let w: Vec<usize> = (0..lines[0].len()).map(|i| lines.iter().map(|l| l[i].chars().count()).max().unwrap_or(0)).collect();
    let mut s = String::new();
    for l in lines {
        let cells: Vec<String> = l.iter().enumerate().map(|(i, c)| {
            let pad = w[i] - c.chars().count();
            if i < left { format!("{c}{}", " ".repeat(pad)) } else { format!("{}{c}", " ".repeat(pad)) }
        }).collect();
        let _ = writeln!(s, "{}", cells.join("  ").trim_end());
    }
    s
}

pub fn render(scans: &[(&'static str, Scan)], a: &Args, f: &Filter, prices: &Prices) -> String {
    if a.json {
        return render_json(scans, a, f, prices);
    }
    let mut s = String::new();
    for (agent, scan) in scans {
        let rs = rows(scan, &a.by, prices);
        let sessions: BTreeSet<&str> = scan.calls.iter().map(|c| c.session.as_str()).collect();
        let _ = writeln!(s, "{agent}: {} file(s), {} session(s), {} call(s) since {}{}{}", n(scan.files), n(sessions.len() as u64),
                         n(scan.calls.len() as u64), f.since, f.until.as_deref().map(|u| format!(" until {u}")).unwrap_or_default(),
                         f.project.as_deref().map(|p| format!(", project {p}")).unwrap_or_default());
        if rs.is_empty() {
            let _ = writeln!(s, "  nothing counted\n");
        } else {
            let grand: u64 = rs.values().map(|r| total(&r.tokens)).sum();
            let mut lines = vec![[a.by.as_str(), "model", "calls", "input", "cache write", "cache read", "output", "thinking", "result_chars", "est $", "share"]
                .iter().map(|x| x.to_string()).collect::<Vec<_>>()];
            let mut shown_results = BTreeSet::new();
            let mut sum = Row { priced: true, usd: Some(0.0), ..Default::default() };
            for ((key, model), r) in &rs {
                let chars = if a.by == "role" && shown_results.insert(key.clone()) {
                    scan.results.get(key).filter(|x| x.results > 0).map(|x| n(x.subagent_result_text_chars)).unwrap_or_default()
                } else {
                    String::new()
                };
                lines.push(vec![key.clone(), if model.is_empty() { "—".into() } else { model.clone() }, n(r.calls), n(r.tokens.input),
                                n(r.tokens.cache_write), n(r.tokens.cache_read), n(r.tokens.output),
                                thinking_cell(r), chars, usd(r.usd),
                                format!("{:.0}%", 100.0 * total(&r.tokens) as f64 / grand.max(1) as f64)]);
                sum.calls += r.calls;
                add(&mut sum.tokens, &r.tokens);
                sum.usd = match (sum.usd, r.usd) { (Some(x), Some(y)) => Some(x + y), _ => None };
                if let Some(t) = r.thinking {
                    sum.thinking = Some(sum.thinking.unwrap_or(0) + t);
                }
                sum.thinking_calls += r.thinking_calls;
            }
            lines.push(vec!["total".into(), String::new(), n(sum.calls), n(sum.tokens.input), n(sum.tokens.cache_write), n(sum.tokens.cache_read),
                            n(sum.tokens.output), thinking_cell(&sum), String::new(),
                            if sum.usd.is_some() { usd(sum.usd) } else { "— (some models have no price)".into() }, "100%".into()]);
            s.push_str(&grid(&lines, 2));
            if sum.thinking.is_some() && sum.thinking_calls < sum.calls {
                let _ = writeln!(s, "thinking: N+ = only {} of {} calls report it (older transcripts lack the field); the sum covers those.",
                                 n(sum.thinking_calls), n(sum.calls));
            }
            let _ = writeln!(s, "est $: comparison unit at list prices of {} (thinking is inside output; not a bill: plans are subscriptions).",
                             prices.version);
            if *agent == "codex" {
                let _ = writeln!(s, "codex: input is the uncached part; cache read = cached_input_tokens; thinking = reasoning_output_tokens (inside output).");
            }
            for sug in suggestions(agent, scan, &rs, prices, &a.by) {
                let _ = writeln!(s, "suggest: {sug}");
            }
        }
        for w in coverage(scan) {
            let _ = writeln!(s, "coverage: {w}");
        }
        if a.explain {
            for e in &scan.explain {
                let _ = writeln!(s, "explain: {e}");
            }
            for l in &scan.large {
                let _ = writeln!(s, "explain: heuristic large_initial_context (a lead to look at, not a fork cost): session {} subagent {} ({}) first call read {} tokens; spawned by {}",
                                 l.session, l.agent_id, l.role, n(l.first_call_input_tokens), l.parent_tool_use);
            }
            for (role, r) in &scan.results {
                let _ = writeln!(s, "explain: {role}: {} result(s), {} chars of returned text (≈{} tokens, estimate), {} async spawn ack(s), {} error(s)",
                                 r.results, n(r.subagent_result_text_chars), n(r.subagent_result_text_chars / 4), r.spawn_acks, r.errors);
            }
        }
        s.push('\n');
    }
    s
}

pub fn coverage(scan: &Scan) -> Vec<String> {
    let k = &scan.skipped;
    let mut v = scan.coverage.clone();
    if k.oversize_lines > 0 {
        v.push(format!("{} line(s) over --max-line-bytes skipped: their usage is not counted", k.oversize_lines));
    }
    if k.bad_json > 0 {
        v.push(format!("{} line(s) not valid JSON skipped (often a line still being written)", k.bad_json));
    }
    if k.invalid_utf8 > 0 {
        v.push(format!("{} line(s) not valid UTF-8 skipped: their usage is not counted", k.invalid_utf8));
    }
    if k.no_id > 0 {
        v.push(format!("{} call(s) without a message/response id counted per line (cannot dedup)", k.no_id));
    }
    v
}

/// Same tokens, another model's prices: a price comparison, not a forecast. Plus long results.
fn suggestions(agent: &str, scan: &Scan, rs: &BTreeMap<(String, String), Row>, prices: &Prices, by: &str) -> Vec<String> {
    let mut out = Vec::new();
    if by != "role" {
        return out;
    }
    let grand: u64 = rs.values().map(|r| total(&r.tokens)).sum::<u64>().max(1);
    let cheaper = if agent == "claude" { "claude-sonnet-5-5" } else { "" };
    if let Some(alt) = prices.get(cheaper) {
        for ((role, model), r) in rs {
            let share = total(&r.tokens) as f64 / grand as f64;
            if share < 0.05 || !(model.contains("opus") || model.contains("fable")) {
                continue;
            }
            let Some(now) = r.usd else { continue };
            let then = cost(alt, &r.tokens);
            out.push(format!("{role} on {model}: {:.0}% of tokens, est ${}; the same tokens at {cheaper} prices ≈ ${} ({:+.0}%) — \
                              a price comparison holding the tokens fixed, not a forecast of cost or quality after switching; prices of {}",
                             share * 100.0, usd(Some(now)), usd(Some(then)), 100.0 * (then - now) / now.max(1e-9), prices.version));
        }
    }
    for (role, r) in &scan.results {
        if r.results > 0 && r.subagent_result_text_chars / r.results > 6000 {
            out.push(format!("{role} hands back {} chars per result on average (≈{} tokens, estimate): a short reply with the details in a file returns less",
                             n(r.subagent_result_text_chars / r.results), n(r.subagent_result_text_chars / r.results / 4)));
        }
    }
    out
}

fn render_json(scans: &[(&'static str, Scan)], a: &Args, f: &Filter, prices: &Prices) -> String {
    let mut meta = serde_json::Map::new();
    let mut rows_out = Vec::new();
    let mut est = serde_json::Map::new();
    for (agent, scan) in scans {
        let sessions: BTreeSet<&str> = scan.calls.iter().map(|c| c.session.as_str()).collect();
        meta.insert(agent.to_string(), json!({
            "files": scan.files, "sessions": sessions.len(), "calls": scan.calls.len(), "dedup_dropped": scan.dedup_dropped,
            "skipped": scan.skipped, "coverage_warnings": coverage(scan),
        }));
        for ((key, model), r) in rows(scan, &a.by, prices) {
            let mut row = json!({
                "agent": agent, "model": if model.is_empty() { Value::Null } else { Value::from(model) },
                "calls": r.calls, "input": r.tokens.input, "cache_write": r.tokens.cache_write, "cache_write_1h": r.tokens.cache_write_1h,
                "cache_read": r.tokens.cache_read, "output": r.tokens.output, "thinking": r.thinking, "thinking_calls": r.thinking_calls,
            });
            row[a.by.as_str()] = Value::from(key);
            rows_out.push(row);
        }
        let money: Vec<Value> = rows(scan, &a.by, prices).into_iter()
            .map(|((key, model), r)| {
                let mut m = json!({ "model": model, "est_usd": r.usd });
                m[a.by.as_str()] = Value::from(key);
                m
            })
            .collect();
        let results: serde_json::Map<String, Value> = scan.results.iter().map(|(k, r)| (k.clone(), json!({
            "results": r.results, "subagent_result_text_chars": r.subagent_result_text_chars,
            "subagent_result_tokens_est": r.subagent_result_text_chars / 4, "spawn_acks": r.spawn_acks, "errors": r.errors,
        }))).collect();
        est.insert(agent.to_string(), json!({
            "est_usd": money, "subagent_results": results, "large_initial_context": scan.large,
            "suggestions": suggestions(agent, scan, &rows(scan, &a.by, prices), prices, &a.by),
        }));
        if a.explain {
            est.get_mut(*agent).unwrap()["explain"] = json!(scan.explain);
        }
    }
    let doc = json!({
        "meta": { "since": f.since, "until": f.until, "project": f.project, "by": a.by, "agents": meta },
        "rows": rows_out,
        "estimates": { "prices_version": prices.version, "by_agent": est },
    });
    serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_and_money() {
        assert_eq!(n(0), "0");
        assert_eq!(n(1234567), "1,234,567");
        assert_eq!(n(999), "999");
        assert_eq!(usd(Some(1234.56)), "1,234.6");
        assert_eq!(usd(Some(0.96)), "1.0");
        assert_eq!(usd(None), "—");
    }
}
