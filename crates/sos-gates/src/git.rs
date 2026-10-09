//! Thin git helpers: gates read the STAGED blob (what will be committed), not the working tree.
//! Anything that cannot be read exactly (non-UTF-8 path or content) is an error, never a skip.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git").current_dir(root).args(args).output().context("running git")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// First letter of git's status: A, C, M, R, D, T…
    pub status: char,
    pub path: String,
}

/// Staged changes (including deletions). Rename/copy report the new path.
pub fn staged_changes(root: &Path) -> Result<Vec<Change>> {
    let out = git(root, &["diff", "--cached", "--name-status", "-z", "--no-renames"])?;
    if !out.status.success() {
        bail!("git diff --cached failed: {}", String::from_utf8_lossy(&out.stderr));
    }
    let mut fields = out.stdout.split(|b| *b == 0).filter(|s| !s.is_empty());
    let mut changes = Vec::new();
    while let (Some(st), Some(p)) = (fields.next(), fields.next()) {
        let path = String::from_utf8(p.to_vec())
            .map_err(|_| anyhow::anyhow!("staged path is not UTF-8: {:?} — rename it", String::from_utf8_lossy(p)))?;
        changes.push(Change { status: st[0] as char, path });
    }
    Ok(changes)
}

/// Staged paths that will exist after the commit (everything but deletions).
pub fn staged_files(root: &Path) -> Result<Vec<String>> {
    Ok(staged_changes(root)?.into_iter().filter(|c| c.status != 'D').map(|c| c.path).collect())
}

/// Content of `path` in the index, or None if the path is not in the index.
pub fn index_blob(root: &Path, path: &str) -> Result<Option<String>> {
    blob(root, &format!(":{path}"))
}

/// Alias kept for readability at call sites that only look at staged files.
pub fn staged_blob(root: &Path, path: &str) -> Result<Option<String>> {
    index_blob(root, path)
}

/// Content of `path` at HEAD, or None (no HEAD yet, or path absent at HEAD).
pub fn head_blob(root: &Path, path: &str) -> Result<Option<String>> {
    blob(root, &format!("HEAD:{path}"))
}

fn blob(root: &Path, spec: &str) -> Result<Option<String>> {
    let out = git(root, &["cat-file", "blob", spec])?;
    if !out.status.success() {
        return Ok(None);
    }
    String::from_utf8(out.stdout).map(Some).map_err(|_| anyhow::anyhow!("{spec} is not UTF-8 text"))
}

/// The committed-to-be version of a file: index if tracked, else working tree, else None.
/// An existing but unreadable working file is an error.
pub fn committed_view(root: &Path, path: &str) -> Result<Option<String>> {
    if let Some(b) = index_blob(root, path)? {
        return Ok(Some(b));
    }
    if head_blob(root, path)?.is_some() {
        return Ok(None); // in HEAD but not in the index: deletion staged, gone after the commit
    }
    let p = root.join(path);
    // try_exists: a permission error is an error, not "absent".
    if !p.try_exists().with_context(|| format!("checking {path}"))? {
        return Ok(None);
    }
    std::fs::read_to_string(&p).map(Some).with_context(|| format!("reading {path}"))
}

/// True when git ignores `path` (relative to root).
pub fn is_ignored(root: &Path, path: &str) -> Result<bool> {
    Ok(git(root, &["check-ignore", "-q", "--", path])?.status.success())
}

/// All tracked paths in the index.
pub fn tracked_files(root: &Path) -> Result<Vec<String>> {
    let out = git(root, &["ls-files", "-z"])?;
    if !out.status.success() {
        bail!("git ls-files failed");
    }
    Ok(out.stdout.split(|b| *b == 0).filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned()).collect())
}

/// Paths of the repository's config files: `config` and, if present, the per-worktree
/// `config.worktree`. Works for linked worktrees and submodules (where `.git` is a file).
pub fn config_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for name in ["config", "config.worktree"] {
        let o = git(root, &["rev-parse", "--git-path", name])?;
        if !o.status.success() {
            continue;
        }
        let mut bytes = o.stdout;
        while bytes.last().is_some_and(|b| *b == b'\n' || *b == b'\r') {
            bytes.pop();
        }
        let p = path_from_bytes(bytes)?;
        out.push(if p.is_absolute() { p } else { root.join(p) });
    }
    Ok(out)
}

#[cfg(unix)]
fn path_from_bytes(b: Vec<u8>) -> Result<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    Ok(PathBuf::from(std::ffi::OsString::from_vec(b))) // lossless, even for non-UTF-8 names
}

#[cfg(not(unix))]
fn path_from_bytes(b: Vec<u8>) -> Result<PathBuf> {
    String::from_utf8(b).map(PathBuf::from).map_err(|_| anyhow::anyhow!("git path is not UTF-8"))
}
