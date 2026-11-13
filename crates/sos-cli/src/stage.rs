//! `sos stage` — where the app is in the A→K process, what is missing, and what comes next (who
//! does it, what they read). Read-only; reads `[stage]` in the working `.sos.toml` (the gate reads the staged one). `--json` is the same for the
//! board that lists the apps, so it never has to keep its own copy of the A→K table.

use sos_gates::stage::{self, Mark, Reading, STAGES};
use std::path::{Path, PathBuf};

#[derive(clap::Args)]
pub struct Args {
    /// Repository (default: current directory).
    #[arg(long, default_value = ".")]
    pub dir: PathBuf,
    /// Print the reading as JSON.
    #[arg(long)]
    pub json: bool,
}

fn root(dir: &Path) -> PathBuf {
    std::process::Command::new("git").current_dir(dir).args(["rev-parse", "--show-toplevel"]).output().ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        .unwrap_or_else(|| dir.to_path_buf())
}

const RECORD: &str = "\"YYYY-MM-DD | evidence\" (or \"bỏ qua | reason\")";

/// The stage the next step is about: the first hole, the current stage while it is open, or the
/// stage to move on to. None while the table has errors, or after K.
pub fn target(r: &Reading) -> Option<char> {
    if let Some(&k) = r.missing().first() {
        return Some(k);
    }
    if !r.errors.is_empty() {
        return None;
    }
    let Some(c) = r.current else { return Some('D') };
    if !r.has(c) {
        return Some(c);
    }
    let mut n = STAGES.iter().skip_while(|x| x.key != c).nth(1)?.key;
    // F runs alongside G: after E both may start, and H waits on F.
    if n == 'F' && r.has('E') {
        n = 'G';
    }
    if n == 'G' && r.has('G') {
        n = 'H';
    }
    if n == 'H' && !r.has('F') {
        n = 'F';
    }
    Some(n)
}

/// The one next step, in a sentence.
pub fn next(r: &Reading) -> String {
    // The commonest hole first, in the words that fix it.
    let missing = r.missing();
    if !missing.is_empty() {
        return format!("backfill {} as {RECORD}", stage::list(&missing));
    }
    if !r.errors.is_empty() {
        return format!("fix [stage] in .sos.toml: {}", r.errors[0]);
    }
    let Some(c) = r.current else {
        return format!("set [stage] current: \"D\" right after `sos install`, with A, B, C backfilled as {RECORD} in the same commit");
    };
    if !r.has(c) {
        let s = stage::get(c).expect("known stage");
        let also = if c == 'G' && !r.has('F') { " F (visual design) runs alongside and is needed before H." } else { "" };
        return format!("{}: {}. When {}, record {c} = {RECORD} and move current on.{also}", s.who, s.makes, s.passes_when);
    }
    let Some(t) = target(r) else { return "done: K is recorded".into() };
    if t == 'F' && r.has('G') {
        return "record F before H's first slice: Chủ nhà approves the mockups, or \"mặc định | reason\" for the default UI".into();
    }
    let n = stage::get(t).expect("known stage");
    let also = if t == 'G' && !r.has('F') { "; F (visual design) runs alongside, needed before H" } else { "" };
    format!("set current = \"{}\" ({}: {}){also}", n.key, n.name, n.who)
}

fn describe(m: &Result<Mark, String>) -> String {
    match m {
        Ok(Mark::Done { date, evidence }) => format!("{date} | {evidence}"),
        Ok(Mark::Skipped(why)) => format!("skipped | {why}"),
        Ok(Mark::Default(why)) => format!("default | {why}"),
        Err(e) => format!("unreadable: {e}"),
    }
}

/// The one-line form for the session-start status (None when [stage] is not set).
pub fn status_line(cfg: &sos_gates::config::StageConfig) -> Option<String> {
    let r = stage::read(cfg);
    let c = r.current?;
    let s = stage::get(c)?;
    Some(format!("Stage: {c} {} — next: {} (`sos stage`)", s.name, next(&r)))
}

/// The config as it is being edited: the working `.sos.toml` when there is one, else what the gates
/// read. `sos stage`, `sos check` and the session-start status all read this, so they agree before
/// the file is staged (tradejournal 13/11/2026: check said "current is empty" while stage said H);
/// only `sos gate stage` reads the staged file, because it judges what will be committed.
pub fn working(root: &Path) -> anyhow::Result<sos_gates::config::Config> {
    let file = root.join(sos_gates::config::FILE);
    if file.exists() {
        Ok(sos_gates::config::parse(&std::fs::read_to_string(&file)?)?)
    } else {
        sos_gates::config::load(root)
    }
}

pub fn run(a: Args) -> anyhow::Result<i32> {
    let root = root(&a.dir);
    let cfg = working(&root)?;
    let r = stage::read(&cfg.stage);
    let cur = r.current.and_then(stage::get);
    let nt = target(&r).and_then(stage::get);
    if a.json {
        let v = serde_json::json!({
            "current": r.current.map(String::from),
            "name": cur.map(|s| s.name),
            "vi": cur.map(|s| s.vi),
            "who": cur.map(|s| s.who),
            "makes": cur.map(|s| s.makes),
            "passes_when": cur.map(|s| s.passes_when),
            "reads": cur.map(|s| s.reads),
            "recorded": r.recorded.iter().map(|(k, m)| (k.to_string(), serde_json::Value::String(describe(m)))).collect::<serde_json::Map<_, _>>(),
            "missing": r.missing().iter().map(char::to_string).collect::<Vec<_>>(),
            "errors": r.errors,
            "next": next(&r),
            "next_stage": nt.map(|s| s.key.to_string()),
            "next_who": nt.map(|s| s.who),
            "next_reads": nt.map(|s| stage::reads(&cfg.stage, s.key)),
        });
        crate::kit::say(&serde_json::to_string_pretty(&v)?);
        return Ok(if r.errors.is_empty() { 0 } else { 1 });
    }
    let mut out = Vec::new();
    match cur {
        None => out.push("Stage: not set".to_string()),
        Some(s) => {
            out.push(format!("Stage: {} {} ({})", s.key, s.name, s.vi));
            out.push(format!("  who:         {}", s.who));
            out.push(format!("  makes:       {}", s.makes));
            out.push(format!("  passes when: {}", s.passes_when));
            let reads = stage::reads(&cfg.stage, s.key);
            if !reads.is_empty() {
                out.push(format!("  reads:       {}", reads.join(", ")));
            }
            if !s.escaped_from.is_empty() {
                out.push(format!("  a fix for a defect this stage let through: Escaped-from: {}", s.escaped_from.join(" or ")));
            }
        }
    }
    out.push(String::new());
    for s in &STAGES {
        let mark = r.recorded.iter().find(|(k, _)| *k == s.key).map(|(_, m)| describe(m));
        let here = if r.current == Some(s.key) { "▶" } else { " " };
        out.push(format!("{here} {} {:<19} {}", s.key, s.name, mark.unwrap_or_else(|| "—".into())));
    }
    out.push(String::new());
    for e in &r.errors {
        out.push(format!("⛔ {e}"));
    }
    if !r.errors.is_empty() {
        out.push(format!("  → How to fix: {}", stage::FIX));
    }
    out.push(format!("Next: {}", next(&r)));
    if let Some(n) = nt.filter(|n| Some(n.key) != r.current) {
        let reads = stage::reads(&cfg.stage, n.key);
        let reads = if reads.is_empty() { String::new() } else { format!("; reads {}", reads.join(", ")) };
        out.push(format!("  {} {}: {}{reads}", n.key, n.name, n.who));
    }
    crate::kit::say(&out.join("\n"));
    Ok(if r.errors.is_empty() { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sos_gates::config::StageConfig;

    fn reading(current: &str, gates: &[(&str, &str)]) -> Reading {
        stage::read(&StageConfig { current: current.into(), gates: gates.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), ..Default::default() })
    }
    const AE: [(&str, &str); 5] = [("A", "bỏ qua | x"), ("B", "2026-11-12 | SOUL.md"), ("C", "bỏ qua | x"), ("D", "2026-11-12 | sos check"), ("E", "2026-11-13 | bp")];

    #[test]
    fn next_step_follows_the_table() {
        assert!(next(&reading("", &[])).starts_with("set [stage] current: \"D\""));
        assert_eq!(next(&reading("D", &[])), format!("backfill A, B, C as {RECORD}"));
        assert!(next(&reading("E", &AE[..4])).starts_with("Kiến trúc sư, then Người soát:"));
        let n = next(&reading("E", &AE));
        assert!(n.starts_with("set current = \"G\"") && n.contains("F (visual design) runs alongside"), "{n}");
        let n = next(&reading("G", &AE));
        assert!(n.starts_with("Thợ:") && n.contains("sos gate skeleton") && n.contains("F (visual design)"), "{n}");
        let mut g = AE.to_vec();
        g.push(("G", "2026-11-20 | sos gate skeleton"));
        assert!(next(&reading("G", &g)).starts_with("record F before H"));
        g.push(("F", "mặc định | x"));
        assert!(next(&reading("G", &g)).starts_with("set current = \"H\""));
        assert_eq!(next(&reading("G", &[("E", "2026-11-13 | bp")])), format!("backfill A, B, C, D as {RECORD}"));
        assert!(next(&reading("A", &[("Q", "2026-11-13 | x")])).starts_with("fix [stage]"));
        // E done: the next step is G, and target says so (its reads come with it).
        assert_eq!(target(&reading("E", &AE)), Some('G'));
        // G finished while current is F: F stays the work; F done too -> H.
        let mut fg = AE.to_vec();
        fg.push(("G", "2026-11-20 | sos gate skeleton"));
        assert_eq!(target(&reading("F", &fg)), Some('F'));
        fg.push(("F", "2026-11-21 | mockups approved"));
        assert_eq!(target(&reading("F", &fg)), Some('H'));
        assert!(next(&reading("F", &fg)).starts_with("set current = \"H\""));
        assert_eq!(target(&reading("", &[])), Some('D'));
    }
}
