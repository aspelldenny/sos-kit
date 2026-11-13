//! `[stage]` — where an app is in the A→K process, and the gate that keeps the record honest
//! (plan `docs/plans/QUY-TRINH-LAM-APP-NHAP-2026-11-12.md`, «Bản đầy đủ A→K», 13/11/2026).
//!
//! Observed failure: the pieces (research, product page, kit, design tool) each ran, but what
//! joined them was the owner's memory. The board listed Ninety-Five at «SOUL» while it was being
//! coded; visual design was called on some apps and forgotten on others; the demand test was
//! dropped without a word. Now each repo says its own stage in `.sos.toml` `[stage]`, and every
//! passed stage leaves "date | evidence", or an explicit "bỏ qua | reason" (skip).
//!
//! The gate checks presence, not quality: no stage is recorded, and `current` does not move past a
//! stage, while an earlier one has nothing. Whether the evidence is good is the work of the stage's
//! own check (`sos gate blueprint` for E, `sos gate skeleton` for G, Người soát, Chủ nhà).

use crate::config::StageConfig;
use crate::Report;
use regex::Regex;

/// One stage of the process.
pub struct Stage {
    pub key: char,
    pub name: &'static str,
    /// The stage's name in the plan, as the board writes it.
    pub vi: &'static str,
    pub who: &'static str,
    pub makes: &'static str,
    /// What has to be true before the stage is recorded and the next one starts.
    pub passes_when: &'static str,
    pub reads: &'static [&'static str],
    /// `Escaped-from:` values (the commit-msg ledger) for a defect this stage let through.
    pub escaped_from: &'static [&'static str],
}

pub const STAGES: [Stage; 11] = [
    Stage { key: 'A', name: "Research", vi: "Spy", who: "research session + Hub",
            makes: "SPY-*.md research notes (kept by Hub until the repo exists)",
            passes_when: "Hub gives a verdict: build or drop",
            reads: &[], escaped_from: &[] },
    Stage { key: 'B', name: "Product page", vi: "SOUL", who: "Hub",
            makes: "SOUL (vibe, why, philosophy, connections) and a one-page PRD with the aha moment in one sentence",
            passes_when: "Chủ nhà approves the SOUL",
            reads: &[], escaped_from: &[] },
    Stage { key: 'C', name: "Demand test", vi: "Thử nhu cầu", who: "Hub designs it, Chủ nhà posts it",
            makes: "a measurement against a threshold set beforehand",
            passes_when: "the threshold is met; skipping is a valid result when the reason is written (\"bỏ qua | reason\")",
            reads: &[], escaped_from: &[] },
    Stage { key: 'D', name: "Repo", vi: "Tạo repo", who: "Quản đốc",
            makes: "the repo with SOS Kit installed; SOUL, PRD and Hub's verdict in it; [stage] backfilled for A–C",
            passes_when: "`sos check` is green",
            reads: &["harness-lite/README.md"], escaped_from: &[] },
    Stage { key: 'E', name: "Blueprint", vi: "UX / BLUEPRINT", who: "Kiến trúc sư, then Người soát",
            makes: "journeys with concrete examples, the data model and the irreversible actions; the first journey leads to B's aha moment",
            passes_when: "`sos gate blueprint` passes and Người soát has reviewed the blueprint",
            reads: &["harness-lite/roles/architect.md", "harness-lite/templates/BLUEPRINT.md"], escaped_from: &["blueprint"] },
    Stage { key: 'F', name: "Visual design", vi: "Thiết kế nhìn", who: "design tool, brief written by Hub from the blueprint",
            makes: "mockups, colour, type and components",
            passes_when: "Chủ nhà approves the mockups before H's first slice; or \"mặc định | reason\" to keep the default UI. Runs alongside G",
            reads: &["the blueprint ([blueprint] file)"], escaped_from: &[] },
    Stage { key: 'G', name: "Skeleton", vi: "Bộ xương", who: "Thợ",
            makes: "one journey running end to end on real data, with a journey test; the default UI is fine",
            passes_when: "`sos gate skeleton` passes",
            reads: &["harness-lite/roles/worker.md", "the blueprint's skeleton journey"], escaped_from: &["skeleton"] },
    Stage { key: 'H', name: "Growth and review", vi: "Lớn dần + soát", who: "Thợ, Người soát",
            makes: "every remaining slice, each held to the screen craft floor",
            passes_when: "Chủ nhà has used it on a real device",
            reads: &["harness-lite/CRAFT-UI.md", "harness-lite/roles/reviewer.md"], escaped_from: &["growth", "review"] },
    Stage { key: 'I', name: "Launch prep", vi: "Chuẩn bị ra mắt", who: "Quản đốc + Hub",
            makes: "store page, privacy, measurement of the numbers promised at B/C, a channel plan built from C",
            passes_when: "every item on the launch checklist is ticked",
            reads: &[], escaped_from: &[] },
    Stage { key: 'J', name: "Launch and measure", vi: "Ra mắt + đo", who: "Chủ nhà + the repo",
            makes: "the numbers over the agreed window (60 days unless B/C said otherwise)",
            passes_when: "the window has passed",
            reads: &[], escaped_from: &[] },
    Stage { key: 'K', name: "Decision", vi: "Quyết định", who: "Hub + Chủ nhà",
            makes: "grow, keep or stop; stopping lists the shutdown steps (store, users, archive the repo)",
            passes_when: "the decision is recorded here",
            reads: &[], escaped_from: &[] },
];

pub fn get(key: char) -> Option<&'static Stage> {
    STAGES.iter().find(|s| s.key == key)
}

fn index(key: char) -> usize {
    STAGES.iter().position(|s| s.key == key).expect("known stage")
}

/// Stages that must be recorded before `key` is recorded or becomes current: every earlier one,
/// except that F (visual design) runs alongside G and only has to be in before H.
pub fn requires(key: char) -> Vec<char> {
    STAGES[..index(key)].iter().map(|s| s.key).filter(|&k| !(key == 'G' && k == 'F')).collect()
}

/// What a recorded stage says.
#[derive(Debug, PartialEq)]
pub enum Mark {
    Done { date: String, evidence: String },
    Skipped(String),
    Default(String),
}

const SKIP: [&str; 3] = ["bỏ qua", "skip", "skipped"];
const DEFAULT: [&str; 2] = ["mặc định", "default"];

pub fn parse_mark(key: char, value: &str) -> Result<Mark, String> {
    let want = if key == 'F' { "\"YYYY-MM-DD | evidence\", \"bỏ qua | reason\" or \"mặc định | reason\"" } else { "\"YYYY-MM-DD | evidence\" or \"bỏ qua | reason\"" };
    let Some((head, tail)) = value.split_once('|') else {
        return Err(format!("{key} = {value:?}: write {want}"));
    };
    let (head, tail) = (head.trim(), tail.trim());
    if tail.is_empty() {
        return Err(format!("{key} = {value:?}: nothing after `|` (write the evidence or the reason)"));
    }
    let lower = head.to_lowercase();
    if SKIP.contains(&lower.as_str()) {
        return Ok(Mark::Skipped(tail.into()));
    }
    if DEFAULT.contains(&lower.as_str()) {
        return if key == 'F' { Ok(Mark::Default(tail.into())) } else { Err(format!("{key} = {value:?}: \"mặc định\" is only for F (default UI); write {want}")) };
    }
    let date = Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("static regex");
    if date.is_match(head) {
        if !real_date(head) {
            return Err(format!("{key} = {value:?}: {head} is not a calendar date (YYYY-MM-DD)"));
        }
        return Ok(Mark::Done { date: head.into(), evidence: tail.into() });
    }
    Err(format!("{key} = {value:?}: before `|` comes a date (YYYY-MM-DD) or \"bỏ qua\"; write {want}"))
}

/// A YYYY-MM-DD string that names a real day (month 1–12, day within that month, leap years).
fn real_date(d: &str) -> bool {
    let n = |r: std::ops::Range<usize>| d[r].parse::<u32>().unwrap_or(0);
    let (y, m, day) = (n(0..4), n(5..7), n(8..10));
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let last = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=last).contains(&day)
}

/// The table read: the current stage, what is recorded, and every problem.
#[derive(Debug, Default)]
pub struct Reading {
    pub current: Option<char>,
    /// Recorded stages in A→K order, with what they say (Err: the value is malformed).
    pub recorded: Vec<(char, Result<Mark, String>)>,
    pub errors: Vec<String>,
}

impl Reading {
    pub fn has(&self, key: char) -> bool {
        self.recorded.iter().any(|(k, m)| *k == key && m.is_ok())
    }
    /// Stages that `current` still waits on.
    pub fn missing(&self) -> Vec<char> {
        self.current.map(|c| requires(c).into_iter().filter(|&k| !self.has(k)).collect()).unwrap_or_default()
    }
}

fn letter(s: &str) -> Option<char> {
    let mut it = s.trim().chars();
    match (it.next(), it.next()) {
        (Some(c), None) => get(c.to_ascii_uppercase()).map(|s| s.key),
        _ => None,
    }
}

pub fn read(cfg: &StageConfig) -> Reading {
    let mut r = Reading::default();
    let cur = cfg.current.trim();
    if !cur.is_empty() {
        match letter(cur) {
            Some(c) => r.current = Some(c),
            None => r.errors.push(format!("current = {cur:?}: use one stage letter, A to K")),
        }
    }
    for (k, v) in &cfg.gates {
        match letter(k) {
            None => r.errors.push(format!("[stage.gates] {k}: not a stage (use A to K)")),
            Some(c) => {
                let m = parse_mark(c, v);
                if let Err(e) = &m {
                    r.errors.push(e.clone());
                }
                r.recorded.push((c, m));
            }
        }
    }
    for k in cfg.reads.keys() {
        if letter(k).is_none() {
            r.errors.push(format!("[stage.reads] {k}: not a stage (use A to K)"));
        }
    }
    r.recorded.sort_by_key(|(k, _)| index(*k));
    if r.recorded.windows(2).any(|w| w[0].0 == w[1].0) {
        r.errors.push("[stage.gates] names a stage twice (keys are case-insensitive)".into());
    }
    let holes = |key: char| -> Vec<char> { requires(key).into_iter().filter(|&k| !r.has(k)).collect() };
    let mut order = Vec::new();
    for (k, m) in &r.recorded {
        if m.is_ok() {
            let h = holes(*k);
            if !h.is_empty() {
                order.push(format!("{k} is recorded but {} before it {} nothing", list(&h), if h.len() == 1 { "has" } else { "have" }));
            }
        }
    }
    if let Some(c) = r.current {
        let h = holes(c);
        if !h.is_empty() {
            order.push(format!("current = \"{c}\" but {} before it {} nothing", list(&h), if h.len() == 1 { "has" } else { "have" }));
        }
        // F and G run alongside: G may finish while current is still F.
        let ahead: Vec<char> = r.recorded.iter().map(|(k, _)| *k).filter(|&k| index(k) > index(c) && !(c == 'F' && k == 'G')).collect();
        if !ahead.is_empty() {
            order.push(format!("{} recorded but current is still \"{c}\": move current on", list(&ahead)));
        }
    } else if !r.recorded.is_empty() {
        order.push("stages are recorded but current is empty: set current".into());
    }
    r.errors.extend(order);
    r
}

/// What the people at a stage read: the kit's files, then the app's own `[stage.reads]`.
pub fn reads(cfg: &StageConfig, key: char) -> Vec<String> {
    let own = cfg.reads.iter().filter(|(k, _)| letter(k) == Some(key)).flat_map(|(_, v)| v.iter().cloned());
    get(key).map(|s| s.reads.iter().map(|x| x.to_string()).collect::<Vec<_>>()).unwrap_or_default().into_iter().chain(own).collect()
}

pub fn list(keys: &[char]) -> String {
    keys.iter().map(char::to_string).collect::<Vec<_>>().join(", ")
}

pub const FIX: &str = "in `.sos.toml` [stage.gates] give every earlier stage a line: `X = \"YYYY-MM-DD | evidence\"` (what proves it passed, a path or a command), \
     or `X = \"bỏ qua | reason\"` when it was skipped (F may say `\"mặc định | reason\"` for the default UI); then set `current`. \
     F runs alongside G and is only needed before H. `sos stage` prints the table and what is next.";

pub fn gate(cfg: &StageConfig) -> Report {
    let mut r = Report::new("stage");
    if cfg.current.trim().is_empty() && cfg.gates.is_empty() {
        r.skipped = Some("[stage] current not set".into());
        return r;
    }
    r.errors = read(cfg).errors;
    if !r.errors.is_empty() {
        r.fix = FIX.into();
    }
    r
}

/// The pre-commit form: checks the `[stage]` table of the staged `.sos.toml`, and only when that
/// file is staged, so a hole blocks the commit that writes it and nothing else.
pub fn gate_staged(root: &std::path::Path) -> anyhow::Result<Report> {
    let file = crate::config::FILE;
    if !crate::git::staged_files(root)?.iter().any(|p| p == file) {
        let mut r = Report::new("stage");
        r.skipped = Some(format!("{file} not staged"));
        return Ok(r);
    }
    let Some(src) = crate::git::staged_blob(root, file)? else {
        return Ok(Report::new("stage"));
    };
    let cfg = crate::config::parse(&src)?;
    Ok(gate(&cfg.stage))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(current: &str, gates: &[(&str, &str)]) -> StageConfig {
        StageConfig { current: current.into(), gates: gates.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), ..Default::default() }
    }

    #[test]
    fn g_may_finish_while_current_is_f() {
        let ae = [("A", "bỏ qua | x"), ("B", "2026-11-12 | b"), ("C", "bỏ qua | x"), ("D", "2026-11-12 | d"), ("E", "2026-11-13 | e")];
        let mut g = ae.to_vec();
        g.push(("G", "2026-11-20 | sos gate skeleton"));
        assert!(gate(&cfg("F", &g)).ok(), "{:?}", read(&cfg("F", &g)).errors);
        assert!(!gate(&cfg("H", &g)).ok(), "H still waits on F");
    }

    #[test]
    fn marks_parse_and_reject() {
        assert_eq!(parse_mark('B', "2026-11-12 | SOUL.md"), Ok(Mark::Done { date: "2026-11-12".into(), evidence: "SOUL.md".into() }));
        assert_eq!(parse_mark('C', "Bỏ qua | build now"), Ok(Mark::Skipped("build now".into())));
        assert_eq!(parse_mark('C', "skip | x"), Ok(Mark::Skipped("x".into())));
        assert!(parse_mark('B', "2028-02-29 | leap day").is_ok());
        assert_eq!(parse_mark('F', "mặc định | chosen 13/11"), Ok(Mark::Default("chosen 13/11".into())));
        for (k, v) in [('E', "mặc định | x"), ('B', "2026-11-12"), ('B', "2026-11-12 |  "), ('B', "12/11 | SOUL.md"), ('B', "done | SOUL.md"), ('B', ""), ('B', "2026-13-45 | x"), ('B', "2026-02-29 | x"), ('B', "2026-04-31 | x"), ('B', "2026-00-10 | x")] {
            assert!(parse_mark(k, v).is_err(), "{k} = {v:?}");
        }
    }

    #[test]
    fn a_fresh_table_is_skipped_and_a_full_one_passes() {
        assert!(gate(&cfg("", &[])).skipped.is_some());
        let ok = cfg("G", &[("A", "2026-11-10 | spy"), ("B", "2026-11-12 | SOUL.md"), ("C", "bỏ qua | build now"),
                            ("D", "2026-11-12 | sos check"), ("E", "2026-11-12 | docs/BLUEPRINT.md"), ("F", "mặc định | default UI")]);
        let r = gate(&ok);
        assert!(r.ok() && r.skipped.is_none(), "{:?}", r.errors);
    }

    #[test]
    fn f_runs_alongside_g_but_blocks_h() {
        let base = [("A", "bỏ qua | x"), ("B", "bỏ qua | x"), ("C", "bỏ qua | x"), ("D", "2026-11-12 | sos check"), ("E", "2026-11-12 | bp")];
        assert!(gate(&cfg("G", &base)).ok(), "G without F");
        let mut g = base.to_vec();
        g.push(("G", "2026-11-14 | sos gate skeleton"));
        assert!(gate(&cfg("G", &g)).ok(), "G recorded without F");
        let r = gate(&cfg("H", &g));
        assert!(!r.ok() && r.errors.iter().any(|e| e.contains("current = \"H\" but F")), "{:?}", r.errors);
        assert_eq!(read(&cfg("H", &g)).missing(), vec!['F']);
    }

    #[test]
    fn holes_and_stale_current_block() {
        // A later stage written over an earlier one with nothing: the board drift this exists for.
        let r = gate(&cfg("G", &[("A", "2026-11-10 | spy"), ("E", "2026-11-12 | bp")]));
        assert!(r.errors.iter().any(|e| e.starts_with("E is recorded but B, C, D")), "{:?}", r.errors);
        assert!(r.errors.iter().any(|e| e.starts_with("current = \"G\" but B, C, D")), "{:?}", r.errors);
        assert!(!r.fix.is_empty());
        let r = gate(&cfg("B", &[("A", "2026-11-10 | spy"), ("B", "2026-11-12 | SOUL.md"), ("C", "bỏ qua | x")]));
        assert!(r.errors.iter().any(|e| e.contains("C recorded but current is still \"B\"")), "{:?}", r.errors);
        let r = gate(&cfg("", &[("A", "2026-11-10 | spy")]));
        assert!(r.errors.iter().any(|e| e.contains("current is empty")));
        assert!(!gate(&cfg("Z", &[])).ok());
        assert!(!gate(&cfg("A", &[("Q", "2026-11-10 | x")])).ok());
        assert!(!gate(&cfg("B", &[("a", "2026-11-10 | x"), ("A", "2026-11-10 | y")])).ok(), "duplicate key");
        // A malformed earlier line counts as missing.
        assert!(!gate(&cfg("B", &[("A", "soon")])).ok());
        assert!(gate(&cfg("b", &[("a", "2026-11-10 | x")])).ok(), "letters are case-insensitive");
    }

    #[test]
    fn stages_map_onto_the_ledger_and_the_stage_gates() {
        let ledger: Vec<&str> = STAGES.iter().flat_map(|s| s.escaped_from.iter().copied()).collect();
        let mut sorted = ledger.clone();
        sorted.sort();
        let mut want = crate::ledger::STAGES.to_vec();
        want.sort();
        assert_eq!(sorted, want, "every Escaped-from value belongs to exactly one stage");
        assert_eq!(get('E').unwrap().escaped_from, ["blueprint"]);
        assert!(get('E').unwrap().passes_when.contains("sos gate blueprint"));
        assert_eq!(get('G').unwrap().escaped_from, ["skeleton"]);
        assert!(get('G').unwrap().passes_when.contains("sos gate skeleton"));
        assert_eq!(get('H').unwrap().escaped_from, ["growth", "review"]);
    }
}
