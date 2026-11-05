//! The few git calls sos-dev needs. Paths come from `-z` output, so names with spaces,
//! quotes or non-ASCII letters arrive as they are (git quotes them otherwise).

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(dir: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .with_context(|| format!("cannot run git {}", args.join(" ")))?;
    if !out.status.success() {
        bail!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(out.stdout)
}

/// The top of the working tree that contains `dir`.
pub fn toplevel(dir: &Path) -> Result<PathBuf> {
    let out = run(dir, &["rev-parse", "--show-toplevel"]).context("not inside a git working tree")?;
    // Only git's record terminator is dropped: on Unix a folder name may end in a space or a CR.
    let s = String::from_utf8(out).context("git top level is not UTF-8")?;
    let s = s.strip_suffix('\n').unwrap_or(&s);
    // Windows git may end the line with CRLF, and a Windows path cannot end in CR.
    #[cfg(windows)]
    let s = s.strip_suffix('\r').unwrap_or(s);
    let s = s.to_string();
    if s.is_empty() {
        bail!("not inside a git working tree");
    }
    Ok(PathBuf::from(s))
}

/// NUL-separated path list from `git ls-files -z …`. A name that is not UTF-8 is an error, not a
/// lossy guess: a guessed name would point at another file, or none, and the gates would skip it.
pub fn ls_files(root: &Path, args: &[&str]) -> Result<Vec<String>> {
    let mut full = vec!["ls-files", "-z"];
    full.extend_from_slice(args);
    let out = run(root, &full)?;
    out.split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| {
            String::from_utf8(s.to_vec()).map_err(|_| {
                anyhow::anyhow!(
                    "file name is not UTF-8: {} (rename it; sos-dev cannot check it safely)",
                    String::from_utf8_lossy(s)
                )
            })
        })
        .collect()
}
