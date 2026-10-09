//! State-doc size cap (was `doc-rotate cap-check`, which hardcoded DISCOVERIES/CHANGELOG).
//! Files come from `[docs].files`; at `soft` lines it warns, at `hard` it blocks.

use crate::config::DocsConfig;
use crate::{git, Report};
use std::path::Path;

pub fn gate(root: &Path, cfg: &DocsConfig) -> Report {
    let mut r = Report::new("docs");
    r.fix = "move old, settled entries into an archive file (e.g. docs/archive/) and keep the live \
             file to current state; raise [docs] soft/hard in .sos.toml only if the file truly needs it"
        .into();
    if cfg.files.is_empty() {
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
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("a.md"), "x\n".repeat(5)).unwrap();
        std::fs::write(d.path().join("b.md"), "x\n".repeat(10)).unwrap();
        let cfg = DocsConfig { files: vec!["a.md".into(), "b.md".into(), "missing.md".into()], soft: 5, hard: 10 };
        let r = gate(d.path(), &cfg);
        assert_eq!(r.warnings.len(), 1, "{:?}", r.warnings);
        assert_eq!(r.errors.len(), 1, "{:?}", r.errors);
        assert!(r.errors[0].starts_with("b.md"));
        assert!(gate(d.path(), &DocsConfig::default()).skipped.is_some());
    }
}
