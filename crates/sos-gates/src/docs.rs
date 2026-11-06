//! State-doc size cap (was `doc-rotate cap-check`, which hardcoded DISCOVERIES/CHANGELOG).
//! Files come from `[docs].files`; at `soft` lines it warns, at `hard` it blocks.
//!
//! `[docs].sized` also measures length: a line cap alone passed a 51-line BACKLOG of 18 KB with
//! one 4,256-character item, of which the session banner shows 220.

use crate::config::DocsConfig;
use crate::{git, Report};
use std::path::Path;

pub fn gate(root: &Path, cfg: &DocsConfig) -> Report {
    let mut r = Report::new("docs");
    r.fix = "move old, settled entries into an archive file (e.g. docs/archive/) and keep the live \
             file to current state; raise [docs] soft/hard in .sos.toml only if the file truly needs it"
        .into();
    if cfg.files.is_empty() && cfg.sized.is_empty() {
        r.skipped = Some("no [docs].files in .sos.toml".into());
        return r;
    }
    for f in &cfg.files {
        // The version that will be committed (index), not the working copy.
        let body = match git::committed_view(root, f) {
            Ok(Some(b)) => b,
            Ok(None) => continue, // a listed doc that does not exist yet is fine
            Err(e) => {
                r.errors.push(format!("{f}: cannot read ({e:#})"));
                continue;
            }
        };
        let lines = body.lines().count();
        if lines >= cfg.hard {
            r.errors.push(format!("{f} has {lines} lines (hard cap {})", cfg.hard));
        } else if lines >= cfg.soft {
            r.warnings.push(format!("{f} has {lines} lines (soft cap {}, hard {})", cfg.soft, cfg.hard));
        }
    }
    for f in &cfg.sized {
        let body = match git::committed_view(root, f) {
            Ok(Some(b)) => b,
            Ok(None) => continue,
            Err(e) => {
                r.errors.push(format!("{f}: cannot read ({e:#})"));
                continue;
            }
        };
        if body.len() > cfg.max_bytes {
            r.errors.push(format!("{f} is {} bytes (max_bytes {})", body.len(), cfg.max_bytes));
            r.fix = SIZED_FIX.into();
        }
        for (i, line) in body.lines().enumerate() {
            let chars = line.chars().count();
            if chars > cfg.max_item_chars {
                r.errors.push(format!("{f}:{} is {chars} characters (max_item_chars {})", i + 1, cfg.max_item_chars));
                r.fix = SIZED_FIX.into();
            }
        }
    }
    r
}

const SIZED_FIX: &str = "agents load this file: keep each item to what to do next and link the detail in docs/plans/; \
                         move settled entries to an archive or the changelog; raise [docs] max_bytes or max_item_chars \
                         in .sos.toml only if the file truly needs it";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("a.md"), "x\n".repeat(5)).unwrap();
        std::fs::write(d.path().join("b.md"), "x\n".repeat(10)).unwrap();
        let cfg = DocsConfig { files: vec!["a.md".into(), "b.md".into(), "missing.md".into()], soft: 5, hard: 10, ..Default::default() };
        let r = gate(d.path(), &cfg);
        assert_eq!(r.warnings.len(), 1, "{:?}", r.warnings);
        assert_eq!(r.errors.len(), 1, "{:?}", r.errors);
        assert!(r.errors[0].starts_with("b.md"));
        assert!(gate(d.path(), &DocsConfig::default()).skipped.is_some());
    }

    #[test]
    fn sized_files_are_measured_in_bytes_and_line_length() {
        let d = tempfile::tempdir().unwrap();
        // Few lines, so the line cap passes all of these: that is the gap this closes.
        std::fs::write(d.path().join("ok.md"), format!("{}\n", "é".repeat(10))).unwrap(); // 10 chars, 20 bytes
        std::fs::write(d.path().join("big.md"), "x\n".repeat(30)).unwrap();
        std::fs::write(d.path().join("long.md"), format!("short\n{}\n", "é".repeat(11))).unwrap();
        let cfg = DocsConfig { sized: vec!["ok.md".into(), "big.md".into(), "long.md".into(), "missing.md".into()],
                               max_bytes: 40, max_item_chars: 10, ..Default::default() };
        let r = gate(d.path(), &cfg);
        assert_eq!(r.errors, ["big.md is 60 bytes (max_bytes 40)", "long.md:2 is 11 characters (max_item_chars 10)"]);
        assert!(r.fix.contains("docs/plans/"), "{}", r.fix);
        // Listed in `files` only: a line cap, no length rule (a changelog stays as it is).
        let r = gate(d.path(), &DocsConfig { files: vec!["big.md".into()], max_bytes: 25, ..Default::default() });
        assert!(r.ok());
        assert!(!r.fix.contains("docs/plans/"));
    }
}
