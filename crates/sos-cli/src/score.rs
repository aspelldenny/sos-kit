//! `sos score` — one scorecard for an app (or the kit itself), so two runs can be compared:
//! with the kit and without, before an upgrade and after (plan `docs/plans/VAI-CHUYEN-GIA-2026-11-11.md`, N2).
//!
//! Read-only on the repo: git history (fix commits by their `Caught-by:` layer and `Escaped-from:` stage), `.sos.toml`,
//! `evidence/ready/runs.log`, `evidence/bite/*.json`, `.advise-state/calls.jsonl`; with `--tokens`,
//! `sos usage` for this folder. `--save` writes the JSON to `evidence/score/`.
//!
//! The headline number is the escape rate: of the fixes whose layer is recorded, the share a person
//! found (owner, user, prod). A kit change that lowers it catches defects earlier; one that raises it
//! is a step back, whatever else it adds.

use anyhow::{bail, Context, Result};
use clap::Args as ClapArgs;
use serde_json::{json, Map, Value};
use sos_gates::ledger::{Ledger, ESCAPES, LAYERS, STAGES};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(ClapArgs)]
pub struct Args {
    /// Repository (default: current directory).
    #[arg(long, default_value = ".")]
    pub dir: PathBuf,
    /// First day counted (YYYY-MM-DD); default: the whole history.
    #[arg(long)]
    pub since: Option<String>,
    /// Print JSON instead of the table.
    #[arg(long)]
    pub json: bool,
    /// Also write the JSON to evidence/score/<date>-<head>.json.
    #[arg(long)]
    pub save: bool,
    /// Also count Claude and Codex tokens for this folder (runs `sos usage`; slower).
    #[arg(long)]
    pub tokens: bool,
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let o = Command::new("git").current_dir(root).args(args).output().context("running git")?;
    if !o.status.success() {
        bail!("git {}: {}", args.join(" "), String::from_utf8_lossy(&o.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

/// Commits as (date, message), oldest first; merges left out (their fixes are counted in the branch).
fn commits(root: &Path, since: Option<&str>) -> Result<Vec<(String, String)>> {
    if git(root, &["rev-parse", "--verify", "-q", "HEAD"]).is_err() {
        return Ok(Vec::new());
    }
    // Committer date: when the commit landed here, which is what --since asks about.
    Ok(git(root, &["log", "--no-merges", "--reverse", "--date=short", "--format=%cd%x1f%B%x1e"])?
        .split('\x1e')
        .filter_map(|c| c.trim_start_matches('\n').split_once('\x1f'))
        .filter(|(d, _)| after(d, since))
        .map(|(d, m)| (d.to_string(), m.to_string()))
        .collect())
}

fn ratio(n: usize, d: usize) -> Value {
    if d == 0 { Value::Null } else { json!(((n as f64 / d as f64) * 1000.0).round() / 1000.0) }
}

fn after(day: &str, since: Option<&str>) -> bool {
    since.is_none_or(|s| day.get(..10).unwrap_or(day) >= s)
}

fn fixes(root: &Path, cfg: &sos_gates::config::Config, since: Option<&str>) -> Result<Value> {
    let ledger = Ledger::new(&cfg.ledger)?;
    let cs = commits(root, since)?;
    let mut by: BTreeMap<&str, usize> = LAYERS.iter().map(|l| (*l, 0)).collect();
    let mut should: BTreeMap<String, usize> = BTreeMap::new();
    let mut from: BTreeMap<&str, usize> = STAGES.iter().map(|s| (*s, 0)).collect();
    let (mut fixes, mut unclassified, mut first_recorded) = (0, 0, None);
    let mut days: Vec<&str> = cs.iter().map(|(d, _)| d.as_str()).collect();
    days.dedup();
    for (day, msg) in &cs {
        let e = ledger.read(msg);
        if !e.fix {
            continue;
        }
        fixes += 1;
        match e.caught_by.as_deref().and_then(|c| LAYERS.iter().find(|l| **l == c)) {
            Some(l) => {
                *by.get_mut(l).unwrap() += 1;
                first_recorded.get_or_insert_with(|| day.clone());
            }
            None => unclassified += 1,
        }
        if let Some(s) = e.should_catch {
            *should.entry(s).or_default() += 1;
        }
        if let Some(n) = e.escaped_from.as_deref().and_then(|s| from.get_mut(s)) {
            *n += 1;
        }
    }
    let classified: usize = by.values().sum();
    let escapes: usize = ESCAPES.iter().map(|l| by[l]).sum();
    Ok(json!({
        "commits": cs.len(),
        "first_day": cs.first().map(|c| c.0.clone()),
        "last_day": cs.last().map(|c| c.0.clone()),
        "active_days": days.len(),
        "fixes": fixes,
        "fix_ratio": ratio(fixes, cs.len()),
        "caught_by": by,
        "classified": classified,
        "unclassified": unclassified,
        "first_recorded_day": first_recorded,
        "escapes": escapes,
        "escape_rate": ratio(escapes, classified),
        "should_catch": should,
        "escaped_from": from,
    }))
}

fn ready(root: &Path, cfg: &sos_gates::config::Config, since: Option<&str>) -> Value {
    let log = std::fs::read_to_string(root.join(crate::ready::LOG)).unwrap_or_default();
    // Line: "PASS <rev> <YYYY-MM-DDTHH:MM:SS> <n>s exit=<c> — <cmd>"
    let runs: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty() && after(l.split(' ').nth(2).unwrap_or(""), since)).collect();
    let pass = runs.iter().filter(|l| l.starts_with("PASS")).count();
    json!({
        "cmd_set": !cfg.ready.cmd.trim().is_empty(),
        "runs": runs.len(),
        "pass": pass,
        "fail": runs.len() - pass,
        "last": runs.last(),
    })
}

fn bite(root: &Path, since: Option<&str>) -> Value {
    let (mut runs, mut killed, mut counted, mut errors) = (0, 0, 0, 0);
    let mut names: Vec<_> = std::fs::read_dir(root.join("evidence/bite")).into_iter().flatten().flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".json")).collect();
    names.sort();
    for n in names {
        // Name: YYYYMMDD-HHMMSS-<head>.json
        let day = n.get(..8).map(|d| format!("{}-{}-{}", &d[..4], &d[4..6], &d[6..8])).unwrap_or_default();
        if !after(&day, since) {
            continue;
        }
        let Ok(v) = std::fs::read_to_string(root.join("evidence/bite").join(&n)).map(|s| serde_json::from_str::<Value>(&s).unwrap_or(Value::Null)) else { continue };
        runs += 1;
        if v.get("error").is_some() {
            errors += 1;
            continue;
        }
        for m in v["mutants"].as_array().into_iter().flatten() {
            if m["no_tests_ran"].as_bool() == Some(true) {
                continue;
            }
            counted += 1;
            if m["killed"].as_bool() == Some(true) {
                killed += 1;
            }
        }
    }
    json!({"runs": runs, "not_measured": errors, "mutants": counted, "killed": killed, "kill_rate": ratio(killed, counted)})
}

fn advisor(root: &Path, cfg: &sos_gates::config::Config, since: Option<&str>) -> Value {
    let log = std::fs::read_to_string(root.join(".advise-state/calls.jsonl")).unwrap_or_default();
    let (mut calls, mut ok, mut roles) = (0, 0, 0);
    for l in log.lines() {
        let Ok(v) = serde_json::from_str::<Value>(l) else { continue };
        // `sos role` writes to the same log with a compact stamp (20261111-160759); normalise it.
        let ts = v["ts"].as_str().unwrap_or("");
        let day = match ts.get(..8) {
            Some(d) if d.bytes().all(|b| b.is_ascii_digit()) => format!("{}-{}-{}", &d[..4], &d[4..6], &d[6..]),
            _ => ts.get(..10).unwrap_or(ts).to_string(),
        };
        if !after(&day, since) {
            continue;
        }
        if v["kind"].as_str() == Some("role") {
            roles += 1;
            continue;
        }
        calls += 1;
        if v["ok"].as_bool() == Some(true) {
            ok += 1;
        }
    }
    json!({"backend": cfg.advisor.backend, "calls": calls, "ok": ok, "failed": calls - ok, "role_runs": roles})
}

fn tokens(root: &Path, since: &str) -> Value {
    let exe = std::env::current_exe().unwrap_or_else(|_| "sos".into());
    let out = Command::new(exe)
        .args(["usage", "--json", "--agent", "all", "--since", since, "--project"])
        .arg(root)
        .output();
    let Some(o) = out.ok().filter(|o| o.status.success()) else { return json!({"error": "sos usage failed"}) };
    let Ok(v) = serde_json::from_slice::<Value>(&o.stdout) else { return json!({"error": "sos usage gave no JSON"}) };
    let mut t: BTreeMap<&str, u64> = BTreeMap::new();
    for r in v["rows"].as_array().into_iter().flatten() {
        for k in ["calls", "input", "cache_write", "cache_read", "output"] {
            *t.entry(k).or_default() += r[k].as_u64().unwrap_or(0);
        }
    }
    // A model with no price is unknown, not free: list it and mark the total partial.
    let (mut usd, mut unpriced) = (0.0, std::collections::BTreeSet::new());
    for e in v["estimates"]["by_agent"].as_object().into_iter().flat_map(|m| m.values())
        .flat_map(|a| a["est_usd"].as_array().cloned().unwrap_or_default())
    {
        match e["est_usd"].as_f64() {
            Some(x) => usd += x,
            None => {
                unpriced.insert(e["model"].as_str().unwrap_or("?").to_string());
            }
        }
    }
    let mut m: Map<String, Value> = t.into_iter().map(|(k, n)| (k.to_string(), json!(n))).collect();
    m.insert("est_usd".into(), json!((usd * 100.0).round() / 100.0));
    m.insert("est_usd_partial".into(), json!(!unpriced.is_empty()));
    m.insert("unpriced_models".into(), json!(unpriced));
    m.insert("since".into(), json!(since));
    Value::Object(m)
}

pub fn score(root: &Path, since: Option<&str>, with_tokens: bool) -> Result<Value> {
    let cfg = sos_gates::config::load(root)?;
    let f = fixes(root, &cfg, since)?;
    let head = git(root, &["rev-parse", "--short", "HEAD"]).map(|s| s.trim().to_string()).unwrap_or_default();
    let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let kit = root.join("harness-lite/CONTRACT.md").exists() || root.join("crates/sos-cli").exists();
    let tok = if with_tokens {
        let s = since.map(str::to_string).or_else(|| f["first_day"].as_str().map(str::to_string)).unwrap_or_else(|| crate::role::local_time("%Y-%m-%d"));
        tokens(root, &s)
    } else {
        Value::Null
    };
    Ok(json!({
        "repo": name,
        "head": head,
        "generated": crate::role::local_time("%Y-%m-%dT%H:%M:%S"),
        "since": since,
        "kit": kit,
        "sos": env!("CARGO_PKG_VERSION"),
        "history": f,
        "ready": ready(root, &cfg, since),
        "bite": bite(root, since),
        "advisor": advisor(root, &cfg, since),
        "tokens": tok,
    }))
}

fn pct(v: &Value) -> String {
    v.as_f64().map_or("—".into(), |r| format!("{:.0}%", r * 100.0))
}

fn table(s: &Value) -> String {
    let h = &s["history"];
    let mut o = format!(
        "sos score — {} @ {} ({}{})\n",
        s["repo"].as_str().unwrap_or(""),
        s["head"].as_str().unwrap_or(""),
        if s["kit"].as_bool() == Some(true) { "with SOS Kit" } else { "no SOS Kit" },
        s["since"].as_str().map(|d| format!(", since {d}")).unwrap_or_default()
    );
    o += &format!(
        "  commits      {} over {} active days ({} → {})\n",
        h["commits"], h["active_days"], h["first_day"].as_str().unwrap_or("—"), h["last_day"].as_str().unwrap_or("—")
    );
    o += &format!("  fixes        {} ({} of commits); layer recorded on {}, not on {}\n", h["fixes"], pct(&h["fix_ratio"]), h["classified"], h["unclassified"]);
    let by: Vec<String> = LAYERS.iter().filter_map(|l| h["caught_by"][l].as_u64().filter(|n| *n > 0).map(|n| format!("{l} {n}"))).collect();
    if !by.is_empty() {
        o += &format!("  caught by    {}\n", by.join(" · "));
    }
    o += &format!("  ESCAPE RATE  {} ({} of {} recorded fixes were found by a person: owner, user, prod)\n", pct(&h["escape_rate"]), h["escapes"], h["classified"]);
    if let Some(m) = h["should_catch"].as_object().filter(|m| !m.is_empty()) {
        o += &format!("  should catch {}\n", m.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(" · "));
    }
    // Stages in the order of the work, so the table reads from design to review.
    let from: Vec<String> = STAGES.iter().filter_map(|st| h["escaped_from"][st].as_u64().filter(|n| *n > 0).map(|n| format!("{st} {n}"))).collect();
    if !from.is_empty() {
        o += &format!("  escaped from {} (the stage that let each defect through; repair the largest)\n", from.join(" · "));
    }
    let r = &s["ready"];
    o += &format!(
        "  ready        {}; {} runs, {} pass, {} fail{}\n",
        if r["cmd_set"].as_bool() == Some(true) { "cmd set" } else { "cmd EMPTY" },
        r["runs"], r["pass"], r["fail"],
        r["last"].as_str().map(|l| format!("; last: {l}")).unwrap_or_default()
    );
    let b = &s["bite"];
    o += &format!("  bite         {} runs; {}/{} mutants killed ({})\n", b["runs"], b["killed"], b["mutants"], pct(&b["kill_rate"]));
    let a = &s["advisor"];
    o += &format!("  advisor      {}: {} calls, {} failed ({} `sos role` runs not counted)\n", a["backend"].as_str().unwrap_or(""), a["calls"], a["failed"], a["role_runs"]);
    let t = &s["tokens"];
    if let Some(e) = t["error"].as_str() {
        o += &format!("  tokens       not counted: {e}\n");
    } else if !t.is_null() {
        o += &format!(
            "  tokens       {} calls, output {}, cache read {}, est ${}{} (since {})\n",
            t["calls"], t["output"], t["cache_read"], t["est_usd"],
            if t["est_usd_partial"] == true { format!(" + unpriced {}", t["unpriced_models"]) } else { String::new() },
            t["since"].as_str().unwrap_or("")
        );
    }
    if h["unclassified"].as_u64().unwrap_or(0) > 0 {
        o += "  note: fixes without `Caught-by:` are not in the escape rate; history before the ledger stays unrecorded.\n";
    }
    o
}

pub fn run(a: Args) -> Result<i32> {
    if let Some(s) = &a.since {
        if s.len() != 10 || s.as_bytes()[4] != b'-' || s.as_bytes()[7] != b'-' || !s.bytes().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()) {
            bail!("--since takes a day written YYYY-MM-DD: {s}");
        }
    }
    let root = PathBuf::from(git(&a.dir, &["rev-parse", "--show-toplevel"]).context("sos score runs inside a git repository")?.trim());
    let s = score(&root, a.since.as_deref(), a.tokens)?;
    let text = serde_json::to_string_pretty(&s)?;
    if a.save {
        let dir = root.join("evidence/score");
        std::fs::create_dir_all(&dir)?;
        let p = dir.join(format!("{}-{}.json", crate::role::local_time("%Y%m%d-%H%M%S"), s["head"].as_str().unwrap_or("")));
        std::fs::write(&p, &text).with_context(|| format!("writing {}", p.display()))?;
        eprintln!("sos score: saved {}", p.strip_prefix(&root).unwrap_or(&p).display());
    }
    if a.json {
        println!("{text}");
    } else {
        print!("{}", table(&s));
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(d: &Path, msg: &str) {
        let ok = Command::new("git").current_dir(d)
            .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "core.hooksPath=/dev/null", "commit", "-q", "--allow-empty", "-m", msg])
            .status().unwrap().success();
        assert!(ok);
    }

    #[test]
    fn counts_fixes_by_layer_and_escape_rate() {
        let d = tempfile::tempdir().unwrap();
        assert!(Command::new("git").current_dir(d.path()).args(["init", "-q"]).status().unwrap().success());
        let empty = score(d.path(), None, false).unwrap();
        assert_eq!(empty["history"]["commits"], 0);
        assert!(empty["history"]["escape_rate"].is_null());
        for m in ["feat: a", "fix: old bug", "fix: b\n\nCaught-by: test", "fix: c\n\nCaught-by: owner\nShould-catch: test\nEscaped-from: blueprint",
                  "Fix d\n\nCaught-by: reviewer\nEscaped-from: growth", "fixup! fix: b", "fix: e\n\nCaught-by: user\nEscaped-from: blueprint"] {
            commit(d.path(), m);
        }
        let s = score(d.path(), None, false).unwrap();
        let h = &s["history"];
        assert_eq!(h["commits"], 7);
        assert_eq!(h["fixes"], 5);
        assert_eq!(h["unclassified"], 1);
        assert_eq!(h["classified"], 4);
        assert_eq!(h["caught_by"]["test"], 1);
        assert_eq!(h["caught_by"]["reviewer"], 1);
        assert_eq!(h["escapes"], 2);
        assert_eq!(h["escape_rate"], 0.5);
        assert_eq!(h["should_catch"]["test"], 1);
        assert_eq!(s["ready"]["cmd_set"], false);
        assert_eq!(s["kit"], false);
        let t = table(&s);
        assert!(t.contains("ESCAPE RATE  50% (2 of 4"), "{t}");
        // Stage that let each defect through; a fix without the trailer is in none of them.
        assert_eq!(h["escaped_from"]["blueprint"], 2);
        assert_eq!(h["escaped_from"]["growth"], 1);
        assert_eq!(h["escaped_from"]["skeleton"], 0);
        assert!(t.contains("escaped from blueprint 2 · growth 1 ("), "{t}");
        // A day after every commit leaves nothing counted.
        let later = score(d.path(), Some("2999-01-01"), false).unwrap();
        assert_eq!(later["history"]["commits"], 0);
    }

    #[test]
    fn reads_ready_bite_and_advisor_records() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path();
        assert!(Command::new("git").current_dir(p).args(["init", "-q"]).status().unwrap().success());
        std::fs::create_dir_all(p.join("evidence/ready")).unwrap();
        std::fs::write(p.join("evidence/ready/runs.log"), "FAIL abc 2026-11-01T10:00:00 3s exit=1 — make ready\nPASS abd 2026-11-02T10:00:00 3s exit=0 — make ready\n").unwrap();
        std::fs::create_dir_all(p.join("evidence/bite")).unwrap();
        std::fs::write(p.join("evidence/bite/20261102-100000-abd.json"),
            r#"{"mutants":[{"killed":true},{"killed":false},{"no_tests_ran":true}]}"#).unwrap();
        std::fs::create_dir_all(p.join(".advise-state")).unwrap();
        std::fs::write(p.join(".advise-state/calls.jsonl"), "{\"ts\":\"2026-11-01T09:00:00\",\"ok\":true}\n{\"ts\":\"2026-11-02T09:00:00\",\"ok\":false}\n{\"ts\":\"20261101-090000\",\"kind\":\"role\",\"ok\":false}\n{\"ts\":\"20261103-090000\",\"kind\":\"role\",\"ok\":true}\n").unwrap();
        std::fs::write(p.join(".sos.toml"), "[ready]\ncmd = \"make ready\"\n").unwrap();
        let s = score(p, None, false).unwrap();
        assert_eq!(s["ready"]["cmd_set"], true);
        assert_eq!((s["ready"]["pass"].as_u64(), s["ready"]["fail"].as_u64()), (Some(1), Some(1)));
        assert_eq!((s["bite"]["killed"].as_u64(), s["bite"]["mutants"].as_u64()), (Some(1), Some(2)));
        assert_eq!((s["advisor"]["calls"].as_u64(), s["advisor"]["failed"].as_u64()), (Some(2), Some(1)));
        assert_eq!(s["advisor"]["role_runs"], 2);
        let s = score(p, Some("2026-11-02"), false).unwrap();
        assert_eq!(s["ready"]["runs"], 1);
        assert_eq!(s["advisor"]["calls"], 1);
        assert_eq!(s["advisor"]["role_runs"], 1);
        assert_eq!(s["bite"]["runs"], 1);
    }
}
