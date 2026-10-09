//! Repository-shape gates (were bash scripts in sos-kit/scripts/):
//! staged secrets via gitleaks, `.env` commits, case collisions, code on the default branch.

use crate::config::GitConfig;
use crate::{git, Report};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// `.env` or `.env.<suffix>`, case-insensitive, except `.env.example` — the same rule as
/// harness-lite/scripts/env-guard.sh (so the edit guard and the commit gate agree).
pub fn is_real_env(name: &str) -> bool {
    let n = name.to_lowercase();
    (n == ".env" || n.starts_with(".env.")) && n != ".env.example"
}

fn base(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Staged diff scanned by gitleaks (was pre-commit gate "secrets").
pub fn secrets(root: &Path) -> Result<Report> {
    let mut r = Report::new("secrets");
    r.fix = "remove the secret from the staged file (env var / .env, keep .env.example placeholders); \
             if gitleaks is missing: `brew install gitleaks`; a false positive gets a `gitleaks:allow` \
             comment on that line with the reason"
        .into();
    let out = Command::new("gitleaks")
        .current_dir(root)
        .args(["git", "--pre-commit", "--staged", "--redact", "--no-banner", "--log-level", "warn", "."])
        .output();
    match out {
        Err(_) => r.errors.push("gitleaks is not installed".into()),
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
            let finding: Vec<&str> = text.lines().filter(|l| l.contains("Finding") || l.contains("RuleID") || l.contains("File:") || l.contains("Line:")).collect();
            if finding.is_empty() {
                r.errors.push(format!("gitleaks failed: {}", text.trim()));
            } else {
                r.errors.extend(finding.iter().map(|l| l.trim().to_string()));
            }
        }
    }
    Ok(r)
}

/// No real `.env*` file may be committed (was scripts/block-env-commit.sh).
pub fn env_commit(root: &Path) -> Result<Report> {
    let mut r = Report::new("env-commit");
    r.fix = "`git restore --staged <file>`; keep secrets local and commit only .env.example".into();
    for c in git::staged_changes(root)? {
        if c.status != 'D' && is_real_env(base(&c.path)) {
            r.errors.push(format!("{} is staged", c.path));
        }
    }
    Ok(r)
}

/// Paths that differ only by letter case break checkouts on macOS/Windows
/// (was scripts/check-case-collision.sh). Checks the index, including ancestor dirs.
pub fn case_collision(root: &Path) -> Result<Report> {
    let mut r = Report::new("case");
    r.fix = "rename one of the colliding paths so they differ by more than letter case".into();
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut reported = std::collections::HashSet::new();
    for path in git::tracked_files(root)? {
        let mut parts: Vec<&str> = Vec::new();
        for seg in path.split('/') {
            parts.push(seg);
            let p = parts.join("/");
            let key = p.to_lowercase();
            match seen.get(&key) {
                Some(prev) if prev != &p => {
                    if reported.insert(key.clone()) {
                        r.errors.push(format!("{prev} vs {p}"));
                    }
                }
                Some(_) => {}
                None => {
                    seen.insert(key, p);
                }
            }
        }
    }
    Ok(r)
}

fn current_branch(root: &Path) -> Option<String> {
    let o = Command::new("git").current_dir(root).args(["branch", "--show-current"]).output().ok()?;
    let b = String::from_utf8_lossy(&o.stdout).trim().to_string();
    (!b.is_empty()).then_some(b)
}

fn default_branch(root: &Path, cfg: &GitConfig) -> Option<String> {
    if !cfg.default_branch.is_empty() {
        return Some(cfg.default_branch.clone());
    }
    let o = Command::new("git").current_dir(root).args(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"]).output().ok()?;
    if o.status.success() {
        return Some(String::from_utf8_lossy(&o.stdout).trim().trim_start_matches("origin/").to_string());
    }
    let unborn = !Command::new("git").current_dir(root).args(["rev-parse", "--verify", "-q", "HEAD"])
        .output().map(|o| o.status.success()).unwrap_or(false);
    if unborn {
        return current_branch(root); // first commit: the branch being born is the default
    }
    for b in ["main", "master"] {
        let ok = Command::new("git").current_dir(root).args(["show-ref", "--verify", "--quiet", &format!("refs/heads/{b}")])
            .status().map(|s| s.success()).unwrap_or(false);
        if ok {
            return Some(b.into());
        }
    }
    None
}

/// With `[git] protect_default_branch = true`: no non-Markdown change committed directly on
/// the default branch (was scripts/no-code-on-default.sh). Merges are allowed.
pub fn branch(root: &Path, cfg: &GitConfig) -> Result<Report> {
    let mut r = Report::new("branch");
    if !cfg.protect_default_branch {
        r.skipped = Some("[git] protect_default_branch = false".into());
        return Ok(r);
    }
    let Some(cur) = current_branch(root) else {
        r.warnings.push("detached HEAD: branch rule not applied".into());
        return Ok(r);
    };
    let Some(def) = default_branch(root, cfg) else {
        r.warnings.push("cannot resolve the default branch; set [git] default_branch".into());
        return Ok(r);
    };
    let git_dir = Command::new("git").current_dir(root).args(["rev-parse", "--git-dir"]).output()?;
    let merge_head = root.join(String::from_utf8_lossy(&git_dir.stdout).trim()).join("MERGE_HEAD");
    if cur != def || merge_head.exists() {
        return Ok(r);
    }
    r.fix = format!("create a branch (`git switch -c <name>`) and commit there; Markdown-only commits on {def} are allowed");
    for c in git::staged_changes(root)? {
        if !c.path.to_lowercase().ends_with(".md") {
            r.errors.push(format!("{} staged on {def}", c.path));
        }
    }
    Ok(r)
}
