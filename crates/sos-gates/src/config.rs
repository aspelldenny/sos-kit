//! `.sos.toml` — one config for every gate. A missing file or section means defaults;
//! an unknown key is an error (a typo must not silently disable a gate).
//!
//! ```toml
//! [text]                       # wording gate (was quality-gate)
//! files = ["App/**/*.swift", "marketing/**/*.txt"]   # staged files to scan
//! banned = ["act now"]         # case-insensitive substrings
//! leaks = ["sk-[A-Za-z0-9]+"]  # case-insensitive regexes
//! base = true                  # also apply the built-in AI-vendor leak list
//! strip_thinking = true        # <thinking>/<reflection> blocks count as violations
//!
//! [docs]                       # size cap for state docs (was doc-rotate cap-check)
//! files = ["docs/STATE.md", "docs/DISCOVERIES.md"]
//! soft = 1000                  # warn at or above (lines)
//! hard = 1500                  # block at or above
//!
//! [features]                   # FEATURES.json identity rules (was features-guard)
//! file = "docs/FEATURES.json"
//! allow_regression = true      # a passed slice may go back to false (honest status)
//! ```
//!
//! A legacy `.quality-gate.toml` (`[banned_phrases].items`, `[internal_leaks].patterns`,
//! `[thinking].strip`) is still read and merged into `[text]`.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::Path;

pub const FILE: &str = ".sos.toml";
pub const LEGACY_TEXT_FILE: &str = ".quality-gate.toml";

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub text: TextConfig,
    #[serde(default)]
    pub docs: DocsConfig,
    #[serde(default)]
    pub features: FeaturesConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TextConfig {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub banned: Vec<String>,
    #[serde(default)]
    pub leaks: Vec<String>,
    #[serde(default = "yes")]
    pub base: bool,
    #[serde(default = "yes")]
    pub strip_thinking: bool,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self { files: vec![], banned: vec![], leaks: vec![], base: true, strip_thinking: true }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DocsConfig {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default = "soft_default")]
    pub soft: usize,
    #[serde(default = "hard_default")]
    pub hard: usize,
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self { files: vec![], soft: soft_default(), hard: hard_default() }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FeaturesConfig {
    #[serde(default = "features_default")]
    pub file: String,
    #[serde(default = "yes")]
    pub allow_regression: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self { file: features_default(), allow_regression: true }
    }
}

fn yes() -> bool {
    true
}
fn soft_default() -> usize {
    1000
}
fn hard_default() -> usize {
    1500
}
fn features_default() -> String {
    "docs/FEATURES.json".into()
}

#[derive(Debug, Deserialize, Default)]
struct LegacyText {
    banned_phrases: Option<LegacyItems>,
    internal_leaks: Option<LegacyPatterns>,
    thinking: Option<LegacyThinking>,
}
#[derive(Debug, Deserialize, Default)]
struct LegacyItems {
    #[serde(default)]
    items: Vec<String>,
}
#[derive(Debug, Deserialize, Default)]
struct LegacyPatterns {
    #[serde(default)]
    patterns: Vec<String>,
}
#[derive(Debug, Deserialize, Default)]
struct LegacyThinking {
    strip: bool,
}

pub fn parse(src: &str) -> Result<Config> {
    let cfg: Config = toml::from_str(src).context("parsing .sos.toml")?;
    if cfg.docs.soft == 0 || cfg.docs.soft >= cfg.docs.hard {
        bail!("[docs] needs 0 < soft < hard (got soft={}, hard={})", cfg.docs.soft, cfg.docs.hard);
    }
    Ok(cfg)
}

/// Load `.sos.toml` (optional) and merge a legacy `.quality-gate.toml` (optional) from `root`.
///
/// Reads the version in the git index when the file is tracked (the config that will be
/// committed), else the working tree, so an unstaged edit cannot switch a gate off.
pub fn load(root: &Path) -> Result<Config> {
    let read = |name: &str| -> Result<Option<String>> {
        match crate::git::committed_view(root, name) {
            Ok(v) => Ok(v),
            Err(_) if !root.join(".git").exists() => {
                // Not a git checkout (e.g. `sos filter` on a server): plain file.
                let p = root.join(name);
                if p.exists() { Ok(Some(std::fs::read_to_string(p)?)) } else { Ok(None) }
            }
            Err(e) => Err(e),
        }
    };
    let mut cfg = match read(FILE)? {
        Some(src) => parse(&src).with_context(|| format!("in {FILE}"))?,
        None => Config::default(),
    };
    if let Some(src) = read(LEGACY_TEXT_FILE)? {
        let l: LegacyText = toml::from_str(&src).with_context(|| format!("parsing {LEGACY_TEXT_FILE}"))?;
        merge_unique(&mut cfg.text.banned, l.banned_phrases.map(|b| b.items).unwrap_or_default());
        merge_unique(&mut cfg.text.leaks, l.internal_leaks.map(|p| p.patterns).unwrap_or_default());
        if let Some(t) = l.thinking {
            cfg.text.strip_thinking = t.strip;
        }
    }
    Ok(cfg)
}

fn merge_unique(into: &mut Vec<String>, more: Vec<String>) {
    for m in more {
        if !into.contains(&m) {
            into.push(m);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_defaults() {
        let c = parse("").unwrap();
        assert!(c.text.base && c.text.strip_thinking && c.features.allow_regression);
        assert_eq!((c.docs.soft, c.docs.hard), (1000, 1500));
        assert_eq!(c.features.file, "docs/FEATURES.json");
    }

    #[test]
    fn unknown_key_is_an_error() {
        assert!(parse("[text]\nbaned = [\"x\"]\n").is_err());
        assert!(parse("[txt]\n").is_err());
    }

    #[test]
    fn bad_caps_rejected() {
        assert!(parse("[docs]\nsoft = 10\nhard = 5\n").is_err());
    }

    #[test]
    fn legacy_quality_gate_file_merges() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join(".sos.toml"), "[text]\nbanned = [\"a\"]\n").unwrap();
        std::fs::write(
            d.path().join(".quality-gate.toml"),
            "[banned_phrases]\nitems = [\"a\", \"b\"]\n[internal_leaks]\npatterns = [\"x+\"]\n[thinking]\nstrip = false\n",
        )
        .unwrap();
        let c = load(d.path()).unwrap();
        assert_eq!(c.text.banned, vec!["a", "b"]);
        assert_eq!(c.text.leaks, vec!["x+"]);
        assert!(!c.text.strip_thinking);
    }
}
