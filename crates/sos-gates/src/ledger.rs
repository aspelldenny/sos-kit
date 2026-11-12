//! The Caught-by ledger (plan `docs/plans/VAI-CHUYEN-GIA-2026-11-11.md`, N1).
//!
//! Every fix commit carries `Caught-by: <layer>` naming what found the defect, and optionally
//! `Should-catch: <layer>` naming the earliest layer that ought to have. The commit-msg gate
//! A fix commit also carries `Escaped-from: <stage>`: the stage of the work that let the defect
//! through (plan `docs/plans/QUY-TRINH-LAM-APP-NHAP-2026-11-12.md`, change 5), so the stage that
//! leaks most is visible from real use instead of from reading the process. The commit-msg gate
//! enforces it; `sos score` counts it. Without the record nobody can tell whether a kit change
//! catches more defects earlier or lets more reach a person.

use crate::config::LedgerConfig;
use crate::Report;
use anyhow::{Context, Result};
use regex::Regex;

/// Layers, cheapest first. `owner`, `user` and `prod` mean the defect reached a person: an escape.
pub const LAYERS: [&str; 8] = ["gate", "test", "bite", "reviewer", "advisor", "owner", "user", "prod"];
pub const ESCAPES: [&str; 3] = ["owner", "user", "prod"];

/// Stages of the work, in order. `blueprint`: the design was wrong or silent about it. `skeleton`:
/// the first end-to-end journey. `growth`: building after the skeleton (also the answer for work
/// that had no blueprint). `review`: a reviewer saw the area and passed it.
pub const STAGES: [&str; 4] = ["blueprint", "skeleton", "growth", "review"];

/// Conventional `fix:` / `fix(scope):` / `hotfix:`, plain "Fix the …", and Vietnamese "Sửa …"
/// (how the maintainer's apps write it: 6 of home-control's subjects in Oct–Nov 2026).
const FIX_DEFAULT: &str = r"(?i)^((hot|bug)?fix(es|ed)?|sửa)\b";

pub struct Ledger {
    fix: Regex,
}

/// What one commit message says.
#[derive(Debug, PartialEq, Default)]
pub struct Entry {
    pub fix: bool,
    pub caught_by: Option<String>,
    pub should_catch: Option<String>,
    pub escaped_from: Option<String>,
    /// Trailer values outside [`LAYERS`] (or [`STAGES`] for `Escaped-from:`), as written.
    pub bad: Vec<String>,
}

impl Ledger {
    pub fn new(cfg: &LedgerConfig) -> Result<Self> {
        let pat = if cfg.fix.is_empty() { FIX_DEFAULT } else { &cfg.fix };
        Ok(Self { fix: Regex::new(pat).with_context(|| format!("[ledger] fix is not a valid regex: {pat:?}"))? })
    }

    /// Reads a commit message as git leaves it for commit-msg: comment lines and everything from
    /// the scissors line on are dropped, and only the last paragraph is read for trailers (git's
    /// own trailer block), so a `Caught-by:` in the middle of the body does not count.
    pub fn read(&self, msg: &str) -> Entry {
        let lines: Vec<&str> = msg.lines()
            .take_while(|l| !(l.starts_with('#') && l.contains(">8")))
            .filter(|l| !l.starts_with('#'))
            .collect();
        let subject_at = lines.iter().position(|l| !l.trim().is_empty());
        let subject = subject_at.map(|i| lines[i].trim()).unwrap_or("");
        let mut e = Entry { fix: !Self::skipped(subject) && self.fix.is_match(subject), ..Default::default() };
        if Self::skipped(subject) {
            return e;
        }
        // The trailer paragraphs: those at the end of the message made only of `Key: value` lines
        // (a value may continue on an indented line, as git allows). Agents often put
        // `Co-Authored-By:` in a paragraph of its own after `Caught-by:`, so more than git's
        // single last paragraph is read; a paragraph with prose stops the walk.
        let body = &lines[subject_at.map_or(lines.len(), |i| i + 1)..];
        let is_trailer = |l: &str| l.split_once(':').is_some_and(|(k, _)| !k.is_empty() && !k.contains(char::is_whitespace));
        let continued = |l: &str| l.starts_with([' ', '\t']);
        let paras: Vec<&[&str]> = body.split(|l| l.trim().is_empty()).filter(|p| !p.is_empty()).collect();
        let block: Vec<&str> = paras.iter().rev()
            .take_while(|p| is_trailer(p[0]) && p.iter().all(|l| is_trailer(l) || continued(l)))
            .flat_map(|p| p.iter().copied())
            .collect();
        for l in block.iter().filter(|l| !continued(l)) {
            let Some((k, v)) = l.split_once(':') else { continue };
            let (slot, allowed): (_, &[&str]) = match k.trim().to_ascii_lowercase().as_str() {
                "caught-by" => (&mut e.caught_by, &LAYERS),
                "should-catch" => (&mut e.should_catch, &LAYERS),
                "escaped-from" => (&mut e.escaped_from, &STAGES),
                _ => continue,
            };
            let v = v.trim().to_ascii_lowercase();
            if !allowed.contains(&v.as_str()) {
                e.bad.push(l.trim().to_string());
            }
            *slot = Some(v);
        }
        e
    }

    /// Merges, reverts and autosquash markers are not commits of their own: no ledger record.
    fn skipped(subject: &str) -> bool {
        ["Merge ", "Revert ", "fixup!", "squash!", "amend!"].iter().any(|p| subject.starts_with(p))
    }
}

/// The commit-msg gate.
pub fn gate(cfg: &LedgerConfig, msg: &str) -> Result<Report> {
    let mut r = Report::new("ledger");
    if !cfg.require {
        r.skipped = Some("[ledger] require = false".into());
        return Ok(r);
    }
    let e = Ledger::new(cfg)?.read(msg);
    for b in &e.bad {
        r.errors.push(format!("unknown value in `{b}`"));
    }
    if e.fix && e.caught_by.is_none() {
        r.errors.push("this is a fix commit but has no `Caught-by:` trailer".into());
    }
    if e.fix && cfg.stage && e.escaped_from.is_none() {
        r.errors.push("this is a fix commit but has no `Escaped-from:` trailer".into());
    }
    if !r.errors.is_empty() {
        r.fix = format!(
            "end the message with a line `Caught-by: <layer>` naming what found the defect, one of {} \
             (owner = Chủ nhà found it, user = a user, prod = production logs){}. Optional: `Should-catch: <layer>`, \
             the earliest layer that ought to have caught it. Not a fix? Start the subject with another word.",
            LAYERS.join(", "),
            if cfg.stage {
                format!(
                    ", and a line `Escaped-from: <stage>` naming the stage of the work that let it through, one of {} \
                     (blueprint = the design was wrong or silent about it, skeleton = the first end-to-end journey, \
                     growth = building after that, or work that had no blueprint, review = a reviewer saw it and passed it)",
                    STAGES.join(", ")
                )
            } else {
                String::new()
            }
        );
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(msg: &str) -> Entry {
        Ledger::new(&LedgerConfig::default()).unwrap().read(msg)
    }

    #[test]
    fn recognises_fix_subjects() {
        for s in ["fix: x", "fix(ui): x", "Fix the crash", "hotfix: x", "bugfix: x", "fixes #3", "FIX: x", "Sửa lỗi Sếp gặp", "sửa: x"] {
            assert!(read(s).fix, "{s}");
        }
        for s in ["feat: x", "fixture: add data", "prefix: x", "Merge branch 'fix'", "Revert \"fix: x\"", "fixup! fix: x", "docs: fix typo", "Sửasai"] {
            assert!(!read(s).fix, "{s}");
        }
        // Comment lines git leaves in the file are not the subject.
        assert!(read("# Please enter\nfix: y\n").fix);
    }

    #[test]
    fn reads_trailers() {
        let e = read("fix: y\n\nbody\n\nCaught-by: Reviewer\nShould-catch: test\n");
        assert_eq!(e.caught_by.as_deref(), Some("reviewer"));
        assert_eq!(e.should_catch.as_deref(), Some("test"));
        assert!(e.bad.is_empty());
        assert_eq!(read("fix: y\n\nCaught-by: luck").bad, vec!["Caught-by: luck"]);
        // Only the final trailer block counts: not prose after it, not a line mid-body, not past the scissors.
        assert_eq!(read("fix: y\n\nCaught-by: owner\nthis is prose").caught_by, None);
        assert_eq!(read("fix: y\n\nCaught-by: owner\n\nmore body").caught_by, None);
        assert_eq!(read("fix: y\n\nbody\n# ------------------------ >8 ------------------------\nCaught-by: test\n").caught_by, None);
        assert!(read("docs: y\n\nThe old Caught-by: luck rule is gone.\n").bad.is_empty());
        assert_eq!(read("fix: y\n\nCaught-by: test\nCo-Authored-By: A <a@b>\n").caught_by.as_deref(), Some("test"));
        // A trailer paragraph followed by another trailer paragraph (Co-Authored-By on its own).
        assert_eq!(read("fix: y\n\nbody\n\nCaught-by: reviewer\nShould-catch: test\n\nCo-Authored-By: A <a@b>\n").caught_by.as_deref(), Some("reviewer"));
        // A folded value (git interpret-trailers reads both trailers here).
        assert_eq!(read("fix: crash\n\nCaught-by: test\nReviewed-by: A <a@b>,\n B <b@c>\n").caught_by.as_deref(), Some("test"));
    }

    #[test]
    fn skipped_commits_are_never_blocked() {
        let c = LedgerConfig::default();
        for m in ["squash! fix: legacy\n\nCaught-by: old-tool", "Revert \"fix: legacy\"\n\nCaught-by: old-tool",
                  "Merge branch 'fix'\n\nCaught-by: x", "fixup! fix: a", "amend! fix: a\n\nCaught-by: nope"] {
            assert!(gate(&c, m).unwrap().ok(), "{m}");
        }
    }

    #[test]
    fn gate_blocks_unrecorded_fix_and_unknown_layer() {
        let c = LedgerConfig::default();
        assert!(gate(&c, "feat: x").unwrap().ok());
        assert!(gate(&c, "fix: x\n\nCaught-by: owner\nEscaped-from: growth").unwrap().ok());
        let r = gate(&c, "fix: x").unwrap();
        assert!(!r.ok() && r.fix.contains("Caught-by: <layer>") && r.fix.contains("Escaped-from: <stage>"));
        assert!(!gate(&c, "feat: x\n\nCaught-by: luck").unwrap().ok());
        assert!(gate(&LedgerConfig { require: false, ..Default::default() }, "fix: x").unwrap().ok());
        assert!(gate(&LedgerConfig { fix: "^repair".into(), ..Default::default() }, "fix: x").unwrap().ok());
        assert!(!gate(&LedgerConfig { fix: "^repair".into(), ..Default::default() }, "repair: x").unwrap().ok());
    }

    #[test]
    fn gate_asks_which_stage_let_the_defect_through() {
        let c = LedgerConfig::default();
        // The stage is required beside the layer, and must be a known one.
        let r = gate(&c, "fix: x\n\nCaught-by: owner").unwrap();
        assert!(!r.ok() && r.errors.iter().any(|e| e.contains("Escaped-from")), "{:?}", r.errors);
        assert!(!gate(&c, "fix: x\n\nCaught-by: owner\nEscaped-from: design").unwrap().ok());
        assert!(!gate(&c, "fix: x\n\nEscaped-from: blueprint").unwrap().ok());
        assert_eq!(read("fix: x\n\nCaught-by: owner\nEscaped-from: Blueprint\n\nCo-Authored-By: A <a@b>\n").escaped_from.as_deref(), Some("blueprint"));
        // A layer name is not a stage, and the reverse.
        assert!(!gate(&c, "fix: x\n\nCaught-by: review\nEscaped-from: growth").unwrap().ok());
        assert!(!gate(&c, "fix: x\n\nCaught-by: test\nEscaped-from: test").unwrap().ok());
        // [ledger] stage = false: the layer alone is enough, the hint does not mention the stage,
        // and a stage that is written must still be a known one.
        let off = LedgerConfig { stage: false, ..Default::default() };
        assert!(gate(&off, "fix: x\n\nCaught-by: owner").unwrap().ok());
        assert!(!gate(&off, "fix: x").unwrap().fix.contains("Escaped-from"));
        assert!(!gate(&off, "fix: x\n\nCaught-by: owner\nEscaped-from: nowhere").unwrap().ok());
    }
}
