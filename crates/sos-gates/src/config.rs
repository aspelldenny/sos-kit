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
//! files = ["docs/STATE.md", "docs/BACKLOG.md"]
//! soft = 1000                  # warn at or above (lines)
//! hard = 1500                  # block at or above
//!
//! [features]                   # FEATURES.json identity rules (was features-guard)
//! file = "docs/FEATURES.json"
//! allow_regression = true      # a passed slice may go back to false (honest status)
//!
//! [git]
//! protect_default_branch = false   # true: non-Markdown commits must happen on a branch
//! default_branch = ""              # empty: origin/HEAD, else main, else master
//!
//! [advisor]                        # read by harness-lite/scripts/advise
//! backend = "claude"               # claude | codex | auto (other family than the agent) | off
//! model = ""                       # empty: opus for claude, Codex config default for codex
//! after = 2                        # consecutive failing test runs before advice
//! max = 2                          # advice calls per failing streak, then "stop and report"
//! effort = ""                      # empty: the CLI default
//! service_tier = ""                # codex only, e.g. "default" (empty: Codex config)
//! tests = ["node scripts/check.mjs"]  # the project's own test commands, besides the built-in runners
//!
//! [bite]                         # read by harness-lite/scripts/bite (do tests catch a wrong change?)
//! test = "python3 -m unittest discover -s tests -t . -v"   # per-test verdicts need verbose output
//! cwd = "server"                   # where `test` runs, relative to the repo root
//! src = '^server/reel/.*\.py$'    # regex: source files whose changed lines get mutants
//! tests = '^server/tests/'         # regex: test files
//! mutants = 8                      # edits per run (each one runs `test`)
//!
//! [state]
//! file = "docs/BACKLOG.md"         # current-work doc: session-start status and advisor intent
//!
//! [roles.worker]                   # model per role, read by `sos role` and `sos check`
//! backend = "claude"               # claude | codex
//! model = "sonnet"
//! effort = "medium"
//! [roles.reviewer.claude]          # keyed by the family that built the work:
//! backend = "codex"                # Claude-built work is reviewed by Codex, and the reverse
//! model = "gpt-…"
//! effort = "high"
//! sandbox = "read-only"            # codex only: read-only | workspace-write
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
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub advisor: AdvisorConfig,
    #[serde(default)]
    pub state: StateConfig,
    #[serde(default)]
    pub bite: BiteConfig,
    #[serde(default)]
    pub roles: std::collections::BTreeMap<String, RoleEntry>,
}

/// Read by `harness-lite/scripts/bite` (Python); validated here so a typo fails `sos check`.
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct BiteConfig {
    #[serde(default)]
    pub test: String,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub tests: String,
    #[serde(default)]
    pub mutants: Option<u32>,
}

/// The doc that says what is being worked on now (status at session start, advisor intent).
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StateConfig {
    #[serde(default = "state_default")]
    pub file: String,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self { file: state_default() }
    }
}

fn state_default() -> String {
    "docs/BACKLOG.md".into()
}

pub const ROLES: [&str; 3] = ["worker", "reviewer", "architect"];

/// One role's model: either the same for every builder, or per family that built the work.
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum RoleEntry {
    One(RoleSpec),
    ByFamily(ByFamily),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ByFamily {
    pub claude: Option<RoleSpec>,
    pub codex: Option<RoleSpec>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoleSpec {
    pub backend: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub effort: String,
    #[serde(default)]
    pub sandbox: String,
    #[serde(default)]
    pub service_tier: String,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

impl RoleEntry {
    /// The spec for work built by `from` (claude | codex), if configured.
    pub fn for_family(&self, from: &str) -> Option<&RoleSpec> {
        match self {
            RoleEntry::One(s) => Some(s),
            RoleEntry::ByFamily(f) => if from == "codex" { f.codex.as_ref() } else { f.claude.as_ref() },
        }
    }
}

/// Read by `harness-lite/scripts/advise` (Python); validated here so a typo fails `sos check`.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AdvisorConfig {
    #[serde(default = "advisor_backend")]
    pub backend: String,
    #[serde(default)]
    pub model: String,
    #[serde(default = "two")]
    pub after: u32,
    #[serde(default = "two")]
    pub max: u32,
    #[serde(default)]
    pub effort: String,
    #[serde(default)]
    pub service_tier: String,
    /// The project's own test commands (prefixes, e.g. "node scripts/check.mjs"), counted like
    /// the built-in runners. POD-agent's check never reached the advisor (23/10/2026).
    #[serde(default)]
    pub tests: Vec<String>,
}

impl Default for AdvisorConfig {
    fn default() -> Self {
        Self { backend: advisor_backend(), model: String::new(), after: 2, max: 2, effort: String::new(), service_tier: String::new(), tests: Vec::new() }
    }
}

fn advisor_backend() -> String {
    "claude".into()
}
fn two() -> u32 {
    2
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct GitConfig {
    #[serde(default)]
    pub protect_default_branch: bool,
    #[serde(default)]
    pub default_branch: String,
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
    let cfg: Config = toml::from_str(src).map_err(|e| {
        // An untagged [roles] entry hides the real key error; say what a role entry may hold.
        let msg = e.to_string();
        if msg.contains("did not match any variant of untagged enum RoleEntry") {
            anyhow::anyhow!("{msg}\n[roles.<role>] takes backend, model, effort, sandbox, service_tier, timeout_secs; \
                             or only [roles.<role>.claude] / [roles.<role>.codex] tables with those keys (not both)")
        } else {
            anyhow::Error::from(e)
        }
    }).context("parsing .sos.toml")?;
    if !["claude", "codex", "auto", "off"].contains(&cfg.advisor.backend.as_str()) {
        bail!("[advisor] backend must be claude, codex, auto or off (got {:?})", cfg.advisor.backend);
    }
    if cfg.advisor.after == 0 {
        bail!("[advisor] after must be at least 1");
    }
    if cfg.advisor.tests.iter().any(|t| t.trim().is_empty()) {
        bail!("[advisor] tests: an entry is empty (each is the start of a test command, e.g. \"node scripts/check.mjs\")");
    }
    for (role, entry) in &cfg.roles {
        if !ROLES.contains(&role.as_str()) {
            bail!("[roles.{role}]: unknown role (use {})", ROLES.join(", "));
        }
        let specs: Vec<(&str, &RoleSpec)> = match entry {
            RoleEntry::One(s) => vec![("", s)],
            RoleEntry::ByFamily(ByFamily { claude, codex }) => {
                if claude.is_none() && codex.is_none() {
                    bail!("[roles.{role}] needs backend/model/effort, or [roles.{role}.claude] / [roles.{role}.codex]");
                }
                [("claude", claude), ("codex", codex)].into_iter().filter_map(|(k, v)| v.as_ref().map(|s| (k, s))).collect()
            }
        };
        for (fam, s) in specs {
            let at = if fam.is_empty() { format!("[roles.{role}]") } else { format!("[roles.{role}.{fam}]") };
            if !["claude", "codex"].contains(&s.backend.as_str()) {
                bail!("{at} backend must be claude or codex (got {:?})", s.backend);
            }
            if s.model.trim().is_empty() || s.effort.trim().is_empty() {
                bail!("{at} needs model and effort: a role without them silently runs the caller's model");
            }
            if !s.sandbox.is_empty() && !["read-only", "workspace-write"].contains(&s.sandbox.as_str()) {
                bail!("{at} sandbox must be read-only or workspace-write (got {:?})", s.sandbox);
            }
            if s.backend == "claude" && (!s.sandbox.is_empty() || !s.service_tier.is_empty()) {
                bail!("{at}: sandbox and service_tier apply to the codex backend only");
            }
        }
    }
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
    fn advisor_section() {
        let c = parse("[advisor]\nbackend = \"auto\"\nafter = 3\n").unwrap();
        assert_eq!((c.advisor.backend.as_str(), c.advisor.after, c.advisor.max), ("auto", 3, 2));
        assert!(parse("[advisor]\nbackend = \"gpt\"\n").is_err());
        assert!(parse("[advisor]\nafter = 0\n").is_err());
        assert_eq!(parse("[advisor]\ntests = [\"node scripts/check.mjs\"]\n").unwrap().advisor.tests, ["node scripts/check.mjs"]);
        assert!(parse("[advisor]\ntests = [\" \"]\n").is_err());
    }

    #[test]
    fn state_and_roles() {
        let c = parse("").unwrap();
        assert_eq!(c.state.file, "docs/BACKLOG.md");
        let c = parse("[state]\nfile = \"docs/PROGRESS.md\"\n[roles.worker]\nbackend = \"claude\"\nmodel = \"sonnet\"\neffort = \"medium\"\n\
                       [roles.reviewer.claude]\nbackend = \"codex\"\nmodel = \"m\"\neffort = \"high\"\nsandbox = \"read-only\"\n").unwrap();
        assert_eq!(c.state.file, "docs/PROGRESS.md");
        assert_eq!(c.roles["worker"].for_family("codex").unwrap().model, "sonnet");
        assert_eq!(c.roles["reviewer"].for_family("claude").unwrap().backend, "codex");
        assert!(c.roles["reviewer"].for_family("codex").is_none());
        assert!(parse("[roles.tester]\nbackend = \"claude\"\nmodel = \"m\"\neffort = \"high\"\n").is_err());
        assert!(parse("[roles.worker]\nbackend = \"claude\"\nmodel = \"sonnet\"\n").is_err(), "effort required");
        assert!(parse("[roles.worker]\nbackend = \"gpt\"\nmodel = \"m\"\neffort = \"high\"\n").is_err());
        assert!(parse("[roles.worker]\nbackend = \"claude\"\nmodel = \"m\"\neffort = \"high\"\nsandbox = \"read-only\"\n").is_err());
        assert!(parse("[roles.worker]\nbackend = \"codex\"\nmodel = \"m\"\neffort = \"high\"\nsandbx = \"x\"\n").is_err());
        let mixed = "[roles.reviewer]\nbackend = \"claude\"\nmodel = \"opus\"\neffort = \"high\"\n[roles.reviewer.codex]\nbackend = \"claude\"\nmodel = \"m\"\neffort = \"high\"\n";
        assert!(parse(mixed).is_err(), "a spec and a family table together must not drop the spec silently");
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
