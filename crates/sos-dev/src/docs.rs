//! `sos-dev docs-check` — docs stay true to the repo (port of `tests/docs-check.py`).
//!
//! Every repo path and every `sos …` command named in the live Markdown must exist.
//! Scope: tracked *.md outside archive/ and the historical logs (CHANGELOG.md, docs/research/,
//! docs/plans/). Paths resolve against the file's directory, the repo root, and — for files that
//! are installed into apps under harness-lite/ — the kit layout (scripts/, adapters/, templates/app/).
//!
//! The port keeps the Python rules on purpose (same regexes, same order, same line numbers from
//! Python's `splitlines`), so both versions print the same problem list on the same tree;
//! `tests/sos-dev.sh` diffs them.

use crate::git;
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SKIP: &[&str] = &["archive/", "docs/research/", "docs/plans/", "CHANGELOG.md"];
const PREFIXES: &[&str] = &[
    "harness-lite/", "docs/", "scripts/", "adapters/", "templates/", "crates/", "tests/", "recipes/", "skills/",
];
/// Where a kit file lands in an app: harness-lite/<x> comes from these kit paths.
const APP_TO_KIT: &[(&str, &str)] = &[
    ("harness-lite/scripts/", "scripts/"),
    ("harness-lite/adapters/", "adapters/"),
    ("harness-lite/hooks/", "templates/app/hooks/"),
    ("harness-lite/templates/", "templates/app/"),
];
/// Paths that exist only inside a project after `sos install`, or are examples, not kit files.
const APP_ONLY: &str = r"^(docs/(STATE|FEATURES|BACKLOG|DESIGN)|\.sos\.toml$|AGENTS\.md$|CLAUDE\.md$|harness-lite/(UPSTREAM\.json|hooks/|templates/)|\.claude/(agents|settings)|\.codex/(hooks|config)|\.git/|\.env|evidence/|\.advise-state|hooks/(pre-commit|pre-push|commit-msg)\.local|~/|\$)";

/// Python's `str.isspace()` set differs from Rust's `char::is_whitespace` only in U+001C–U+001F.
fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python's `read_text()` (universal newlines) followed by `str.splitlines()`.
pub fn py_lines(text: &str) -> Vec<String> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let sep = |c: char| matches!(c, '\n' | '\x0b' | '\x0c' | '\x1c' | '\x1d' | '\x1e' | '\u{85}' | '\u{2028}' | '\u{2029}');
    let mut lines: Vec<String> = text.split(sep).map(str::to_string).collect();
    if text.is_empty() || text.ends_with(sep) {
        lines.pop();
    }
    lines
}

struct Checker {
    root: PathBuf,
    top: BTreeSet<String>,
    cmds: BTreeSet<String>,
    gates: BTreeSet<String>,
    app_only: Regex,
    link: Regex,
    code: Regex,
    sos: Regex,
    skip_link: Regex,
    ext: Regex,
}

fn help_names(sos: &Path, args: &[&str]) -> Result<BTreeSet<String>> {
    let out = Command::new(sos)
        .args(args)
        .output()
        .map_err(|_| anyhow!("docs-check: build sos first ({} not found)", sos.display()))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let after = text
        .split_once("Commands:")
        .ok_or_else(|| anyhow!("docs-check: no `Commands:` in `{} {}`", sos.display(), args.join(" ")))?
        .1;
    let section = after.split("Options:").next().unwrap_or("");
    let re = Regex::new(r"(?m)^\s{2}([a-z][a-z-]+)\s").expect("static regex");
    Ok(re.captures_iter(section).map(|c| c[1].to_string()).collect())
}

impl Checker {
    fn new(root: &Path, sos: &Path) -> Result<Self> {
        let mut top: BTreeSet<String> = fs::read_dir(root)
            .with_context(|| format!("cannot list {}", root.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        top.extend([".claude", ".codex", ".github"].map(String::from));
        let re = |s: &str| Regex::new(s).expect("static regex");
        Ok(Checker {
            root: root.to_path_buf(),
            top,
            cmds: help_names(sos, &["--help"])?,
            gates: help_names(sos, &["gate", "--help"])?,
            app_only: re(APP_ONLY),
            link: re(r"\]\(([^)\s\x1c-\x1f]+)\)"),
            code: re(r"`([^`\n]+)`"),
            sos: re(r"^sos[\s\x1c-\x1f]+([a-z-]+)(?:[\s\x1c-\x1f]+([a-z-]+))?"),
            skip_link: re(r"^(https?:|mailto:|#)"),
            ext: re(r"\.(md|toml|json|sh|py|rs|yml)$"),
        })
    }

    /// `rel_md` is the Markdown file's path from the root, with forward slashes.
    fn exists(&self, rel: &str, rel_md: &str) -> bool {
        let rel = rel.split('#').next().unwrap_or("");
        let rel = rel.trim_end_matches('/').trim_end_matches(['.', ',', ';', ':']);
        if rel.is_empty() {
            return true;
        }
        let md_dir = self.root.join(Path::new(rel_md).parent().unwrap_or(Path::new("")));
        let mut cands = vec![md_dir.join(rel), self.root.join(rel)];
        for (app, kit) in APP_TO_KIT {
            if let Some(rest) = rel.strip_prefix(app) {
                cands.push(self.root.join(format!("{kit}{rest}")));
            }
        }
        let parent_name = Path::new(rel_md).parent().and_then(|p| p.file_name()).and_then(|n| n.to_str());
        if parent_name == Some("harness-lite") || rel_md.contains("harness-lite/") {
            for (app, kit) in APP_TO_KIT {
                let short = app.strip_prefix("harness-lite/").unwrap_or(app);
                if let Some(rest) = rel.strip_prefix(short) {
                    cands.push(self.root.join(format!("{kit}{rest}")));
                }
            }
        }
        cands.iter().any(|c| c.exists())
    }

    fn check_line(&self, rel_md: &str, n: usize, line: &str, problems: &mut Vec<String>) {
        for c in self.link.captures_iter(line) {
            let link = &c[1];
            if self.skip_link.is_match(link) {
                continue;
            }
            if !self.exists(link, rel_md) {
                problems.push(format!("{rel_md}:{n}: broken link {link}"));
            }
        }
        for c in self.code.captures_iter(line) {
            let t = c[1].trim_matches(py_space);
            if let Some(m) = self.sos.captures(t) {
                let cmd = &m[1];
                if !self.cmds.contains(cmd) {
                    problems.push(format!("{rel_md}:{n}: unknown command `sos {cmd}`"));
                } else if cmd == "gate" {
                    if let Some(g) = m.get(2) {
                        if !self.gates.contains(g.as_str()) {
                            problems.push(format!("{rel_md}:{n}: unknown gate `sos gate {}`", g.as_str()));
                        }
                    }
                }
                continue;
            }
            if t.contains(' ')
                || t.chars().any(|ch| "<>*{}|$=()".contains(ch))
                || (!t.contains('/') && !self.ext.is_match(t))
            {
                continue;
            }
            let first = t.split('/').next().unwrap_or("");
            if !self.top.contains(first) && !PREFIXES.iter().any(|p| t.starts_with(p)) {
                continue;
            }
            if self.app_only.is_match(t) && !self.exists(t, rel_md) {
                continue;
            }
            if !self.exists(t, rel_md) {
                problems.push(format!("{rel_md}:{n}: path not found `{t}`"));
            }
        }
    }
}

/// Runs the check and prints the report. Returns true when there is no problem.
pub fn run(root: &Path, sos: &Path) -> Result<bool> {
    let checker = Checker::new(root, sos)?;
    let files: Vec<String> = git::ls_files(root, &["--", "*.md"])?
        .into_iter()
        .filter(|f| !SKIP.iter().any(|s| f.starts_with(s)))
        .collect();
    let mut problems = Vec::new();
    for rel_md in &files {
        let bytes = match fs::read(root.join(rel_md)) {
            Ok(b) => b,
            Err(e) => {
                problems.push(format!("{rel_md}: cannot read ({e})"));
                continue;
            }
        };
        let Ok(text) = String::from_utf8(bytes) else {
            problems.push(format!("{rel_md}: not valid UTF-8"));
            continue;
        };
        for (i, line) in py_lines(&text).iter().enumerate() {
            checker.check_line(rel_md, i + 1, line, &mut problems);
        }
    }
    for p in &problems {
        println!("{p}");
    }
    println!("docs-check: {} files, {} problem(s)", files.len(), problems.len());
    Ok(problems.is_empty())
}

/// Default sos binary: `<root>/target/debug/sos`.
pub fn default_sos(root: &Path) -> PathBuf {
    root.join("target/debug/sos")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_follow_python_splitlines() {
        assert_eq!(py_lines("a\r\nb\rc\n"), vec!["a", "b", "c"]);
        assert_eq!(py_lines("a\u{2028}b\x0cc"), vec!["a", "b", "c"]);
        assert_eq!(py_lines("a\n\nb"), vec!["a", "", "b"]);
        assert!(py_lines("").is_empty());
        assert_eq!(py_lines("\n"), vec![""]);
        assert_eq!(py_lines("a\u{85}"), vec!["a"]);
    }
}
