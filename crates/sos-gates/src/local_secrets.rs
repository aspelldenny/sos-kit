//! Local-state secrets (was `doctor runtime-scan`): tokens in files that never pass through a
//! commit diff but leak in other ways — `.git/config` remote URLs, agent config files.
//!
//! Changes from doctor 0.1.3:
//! - detection is delegated to gitleaks (hundreds of maintained rules instead of 9 hand-written
//!   ones that missed `sk-proj-…`); gitleaks is already required by the pre-commit;
//! - `.env*` files are no longer scanned for tokens (that is where secrets belong); instead each
//!   real `.env*` must be ignored by git and untracked;
//! - Codex config files are scanned too; the git config is located with `git rev-parse
//!   --git-path config`, so linked worktrees and submodules are covered.
//! - a real `.env*` tracked anywhere in the index is an error, not only at the root.

use crate::{git, Report};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Agent/tool config files at the repo root; the git config is found via `git rev-parse`.
pub const TARGETS: &[&str] = &[
    ".mcp.json",
    ".claude/settings.json",
    ".claude/settings.local.json",
    ".codex/config.toml",
    ".codex/hooks.json",
];

pub fn gate(root: &Path) -> Result<Report> {
    let mut r = Report::new("local-secrets");
    r.fix = "move the token out of the file (credential helper or SSH for git remotes; env vars \
             for MCP/agent config) and rotate it; add real .env files to .gitignore and \
             `git rm --cached` them"
        .into();

    let is_real_env = |name: &str| {
        let n = name.to_lowercase();
        n.starts_with(".env") && n != ".env.example"
    };
    // 1a. No real .env file tracked anywhere in the repo (index, so a deleted working copy
    //     or a nested services/.env is still caught).
    for path in git::tracked_files(root)? {
        let base = path.rsplit('/').next().unwrap_or(&path);
        if is_real_env(base) {
            r.errors.push(format!("{path} is tracked by git"));
        }
    }
    // 1b. Untracked real .env files at the root must be ignored.
    for entry in std::fs::read_dir(root).context("reading repo root")? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if is_real_env(&name) && !r.errors.iter().any(|e| e.starts_with(&format!("{name} "))) && !git::is_ignored(root, &name)? {
            r.errors.push(format!("{name} is not in .gitignore"));
        }
    }

    // 2. Tokens in local config files, via gitleaks.
    let mut present: Vec<(String, std::path::PathBuf)> = Vec::new();
    for cfg in git::config_paths(root)? {
        if cfg.try_exists().with_context(|| format!("checking {}", cfg.display()))? {
            let name = if cfg.ends_with("config.worktree") { ".git/config.worktree" } else { ".git/config" };
            present.push((name.into(), cfg));
        }
    }
    for t in TARGETS {
        if root.join(t).is_file() {
            present.push((t.to_string(), root.join(t)));
        }
    }
    if present.is_empty() {
        return Ok(r);
    }
    let tmp = std::env::temp_dir().join(format!("sos-local-secrets-{}", std::process::id()));
    std::fs::create_dir_all(&tmp)?;
    for (name, src) in &present {
        // Copy into a scratch dir so gitleaks scans exactly these files and nothing else.
        std::fs::copy(src, tmp.join(name.replace('/', "__")))?;
    }
    let report_path = tmp.join("report.json");
    let out = Command::new("gitleaks")
        .args(["dir", "--no-banner", "--redact", "--log-level", "error", "--exit-code", "3", "--report-format", "json", "--report-path"])
        .arg(&report_path)
        .arg(&tmp)
        .output();
    let result = match out {
        Err(_) => {
            r.errors.push("gitleaks is not installed (brew install gitleaks)".into());
            Ok(r)
        }
        Ok(o) if o.status.code() == Some(3) => {
            let findings: serde_json::Value = std::fs::read_to_string(&report_path).ok()
                .and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
            for f in findings.as_array().into_iter().flatten() {
                let file = f["File"].as_str().unwrap_or("?");
                let file = Path::new(file).file_name().map(|n| n.to_string_lossy().replace("__", "/")).unwrap_or_default();
                r.errors.push(format!("{file}:{}: {}", f["StartLine"], f["RuleID"].as_str().unwrap_or("secret")));
            }
            if r.errors.is_empty() {
                r.errors.push("gitleaks reported a secret (report unreadable)".into());
            }
            Ok(r)
        }
        Ok(o) if o.status.success() => Ok(r),
        Ok(o) => {
            r.errors.push(format!("gitleaks failed: {}", String::from_utf8_lossy(&o.stderr).trim()));
            Ok(r)
        }
    };
    let _ = std::fs::remove_dir_all(&tmp);
    result
}
