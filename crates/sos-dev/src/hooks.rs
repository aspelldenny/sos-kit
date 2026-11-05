//! `sos-dev install-hooks` — arm the kit repo's own git hooks (port of `scripts/install-hooks.sh`).
//!
//! Points `core.hooksPath` at the tracked `hooks/` folder, so the tracked hook IS the running
//! hook: an edit to hooks/pre-commit is live at once, no stale copy in .git/hooks drifting from
//! source (Két dogfood 2026-06-03). core.hooksPath is local git state, so a fresh clone runs this
//! again. Idempotent.
//!
//! Guard (2026-06 dogfood): never silently take over a repo whose core.hooksPath already points
//! elsewhere (its own security gate) — ask on a terminal, refuse otherwise.
//!
//! Change from the script: the old hook copies are looked up in `git rev-parse --git-common-dir`
//! instead of a literal `.git/hooks`, which is a file, not a folder, inside a linked worktree.

use crate::git;
use anyhow::{bail, Result};
use std::fs;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

const HOOKS: &[&str] = &["pre-commit", "pre-push"];

#[cfg(unix)]
fn make_executable(p: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(m) = fs::metadata(p) {
        let mut perm = m.permissions();
        perm.set_mode(perm.mode() | 0o111);
        let _ = fs::set_permissions(p, perm);
    }
}
#[cfg(not(unix))]
fn make_executable(_: &Path) {}

fn git_hooks_dir(root: &Path) -> Result<PathBuf> {
    let out = git::run(root, &["rev-parse", "--git-common-dir"])?;
    let common = PathBuf::from(String::from_utf8_lossy(&out).trim());
    Ok(root.join(common).join("hooks"))
}

/// Returns the process exit code.
pub fn run(root: &Path) -> Result<i32> {
    if !root.join("hooks/pre-commit").is_file() {
        bail!("hooks/pre-commit not found — this command arms the sos-kit repo's own hooks; in a project use `sos install`");
    }
    for h in HOOKS {
        let p = root.join("hooks").join(h);
        if p.is_file() {
            make_executable(&p);
        }
    }

    let existing = git::run(root, &["config", "--local", "core.hooksPath"])
        .map(|o| String::from_utf8_lossy(&o).trim().to_string())
        .unwrap_or_default();
    if !existing.is_empty() && existing != "hooks" {
        eprintln!("⚠️  core.hooksPath is already set to '{existing}' (this repo's own hook chain).");
        eprintln!("    Pointing it at sos-kit's hooks/ would REDIRECT every hook → silently disabling");
        eprintln!("    your existing hooks (e.g. a security gate). sos-kit's hooks/ may not contain them.");
        eprintln!("    → Merge sos-kit's checks INTO your hook instead (additive), don't override.");
        if std::io::stdin().is_terminal() {
            print!("    Override core.hooksPath → hooks/ anyway? [y/N] ");
            let _ = std::io::stdout().flush();
            let mut ans = String::new();
            let _ = std::io::stdin().lock().read_line(&mut ans);
            if !matches!(ans.trim_end_matches(['\r', '\n']), "y" | "Y") {
                println!("    Aborted.");
                return Ok(1);
            }
        } else {
            eprintln!("    ABORTED (non-interactive — won't silently clobber). Re-run in a terminal to override.");
            return Ok(1);
        }
    }

    let git_hooks = git_hooks_dir(root)?;
    let is_sample = |p: &Path| {
        fs::read(p)
            .map(|b| {
                let first = b.split(|c| *c == b'\n').next().unwrap_or(&[]);
                String::from_utf8_lossy(first).to_lowercase().contains("sample")
            })
            .unwrap_or(false)
    };
    for h in HOOKS {
        let p = git_hooks.join(h);
        if p.is_file() && !is_sample(&p) {
            eprintln!("⚠️  Existing .git/hooks/{h} → moved to .git/hooks/{h}.pre-hookspath.bak (core.hooksPath overrides it; .bak = escape hatch).");
        }
    }

    git::run(root, &["config", "core.hooksPath", "hooks"])?;
    let mut names: Vec<String> = fs::read_dir(root.join("hooks"))
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| !n.starts_with('.'))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    let live: String = names.iter().map(|n| format!("{n} ")).collect();
    println!("✓ core.hooksPath → hooks/ (tracked hooks now live: {live})");

    // Retire stale copies the old copy method left behind: core.hooksPath overrides them, but a
    // leftover reads as "installed" and confuses. Rename, never delete — keep an escape hatch.
    for h in HOOKS {
        let p = git_hooks.join(h);
        if p.is_file() {
            let _ = fs::rename(&p, git_hooks.join(format!("{h}.pre-hookspath.bak")));
        }
    }
    Ok(0)
}
