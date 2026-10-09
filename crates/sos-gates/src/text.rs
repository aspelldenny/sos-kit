//! Wording gate (was `quality-gate`): banned phrases, internal leaks, thinking artifacts.
//!
//! `filter` strips violations (for LLM output at runtime); `check` only reports them
//! (for files at commit time). Fixes over quality-gate 0.1.0: banned phrases are matched
//! with a case-insensitive regex, so case folding that changes byte length (e.g. `İ`)
//! cannot shift or panic the cut; reported snippets are truncated on a char boundary.

use crate::config::TextConfig;
use crate::{git, glob, Report};
use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

/// Built-in leak patterns (quality-gate `rules/base.toml`): AI vendor and model names that
/// must not reach users. Disable with `[text] base = false`.
pub const BASE_LEAKS: &[&str] = &[
    r"Claude\s*(Opus|Sonnet|Haiku)\s*[\d.]*",
    r"Gemini\s*(Flash|Pro)\s*[\d.]*",
    r"GPT-[34][\w.-]*",
    "OpenRouter",
    "OpenAI",
    "Anthropic",
    "text-embedding-3-small",
];

static THINKING: LazyLock<[Regex; 2]> = LazyLock::new(|| {
    [
        Regex::new(r"(?si)<thinking>.*?</thinking>").unwrap(),
        Regex::new(r"(?si)<reflection>.*?</reflection>").unwrap(),
    ]
});
static BLANKS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    ThinkingArtifact,
    InternalLeak,
    BannedPhrase,
}

impl Kind {
    fn tag(self) -> &'static str {
        match self {
            Kind::ThinkingArtifact => "thinking_artifact",
            Kind::InternalLeak => "internal_leak",
            Kind::BannedPhrase => "banned_phrase",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub kind: Kind,
    pub found: String,
    pub line: usize,
}

/// Compiled rules: thinking flag + (kind, regex) list in pipeline order.
pub struct Rules {
    strip_thinking: bool,
    patterns: Vec<(Kind, Regex)>,
}

impl Rules {
    pub fn from_config(c: &TextConfig) -> Result<Self> {
        let mut patterns = Vec::new();
        let base: &[&str] = if c.base { BASE_LEAKS } else { &[] };
        for p in base.iter().copied().chain(c.leaks.iter().map(String::as_str)) {
            let re = Regex::new(&format!("(?i){p}")).with_context(|| format!("invalid leak pattern {p:?}"))?;
            // A pattern that can match "" (e.g. `^|x`) makes the engine report empty matches
            // where the real one starts, hiding violations: reject it at load time.
            if re.find_iter("x\nOpenAI x").any(|m| m.as_str().is_empty()) || re.is_match("") {
                anyhow::bail!("leak pattern {p:?} can match empty text; make every alternative match at least one character");
            }
            patterns.push((Kind::InternalLeak, re));
        }
        for b in c.banned.iter().filter(|b| !b.is_empty()) {
            patterns.push((Kind::BannedPhrase, Regex::new(&format!("(?i){}", regex::escape(b)))?));
        }
        Ok(Self { strip_thinking: c.strip_thinking, patterns })
    }
}

fn snippet(s: &str) -> String {
    if s.chars().count() > 50 {
        format!("{}...", s.chars().take(50).collect::<String>())
    } else {
        s.to_string()
    }
}

fn strip(text: &mut String, re: &Regex, kind: Kind, out: &mut Vec<Violation>) {
    // Skip empty matches (a pattern like `^|x` matches "" first) instead of stopping at them.
    while let Some(m) = re.find_iter(text).find(|m| !m.as_str().is_empty()) {
        out.push(Violation { kind, found: snippet(m.as_str()), line: text[..m.start()].matches('\n').count() + 1 });
        let r = m.range();
        text.replace_range(r, "");
    }
}

/// Strip every violation; returns cleaned text and what was removed.
/// Order (as quality-gate): thinking → leaks → banned → collapse 3+ newlines to 2.
pub fn filter(input: &str, rules: &Rules) -> (String, Vec<Violation>) {
    let mut text = input.to_string();
    let mut v = Vec::new();
    if rules.strip_thinking {
        for re in THINKING.iter() {
            strip(&mut text, re, Kind::ThinkingArtifact, &mut v);
        }
    }
    for kind in [Kind::InternalLeak, Kind::BannedPhrase] {
        for (k, re) in rules.patterns.iter().filter(|(k, _)| *k == kind) {
            strip(&mut text, re, *k, &mut v);
        }
    }
    (BLANKS.replace_all(&text, "\n\n").into_owned(), v)
}

/// Report violations without changing the text. Line numbers refer to the ORIGINAL text.
pub fn check(input: &str, rules: &Rules) -> Vec<Violation> {
    let mut v = Vec::new();
    let line_of = |pos: usize| input[..pos].matches('\n').count() + 1;
    if rules.strip_thinking {
        for re in THINKING.iter() {
            for m in re.find_iter(input) {
                v.push(Violation { kind: Kind::ThinkingArtifact, found: snippet(m.as_str()), line: line_of(m.start()) });
            }
        }
    }
    for (k, re) in &rules.patterns {
        for m in re.find_iter(input).filter(|m| !m.as_str().is_empty()) {
            v.push(Violation { kind: *k, found: snippet(m.as_str()), line: line_of(m.start()) });
        }
    }
    v.sort_by_key(|x| x.line);
    v
}

/// Gate: check named files (working tree) or, with `staged`, the staged blob of every staged
/// file matching `[text].files`.
pub fn gate(root: &Path, cfg: &TextConfig, files: &[String], staged: bool) -> Result<Report> {
    let mut r = Report::new("text");
    r.fix = "reword the flagged text (rules: .sos.toml [text] or .quality-gate.toml); a rule that \
             is wrong for this project belongs in that file, not in a bypass"
        .into();
    let rules = Rules::from_config(cfg)?;
    let mut targets: Vec<(String, String)> = Vec::new();
    if staged {
        if cfg.files.is_empty() {
            r.skipped = Some("no [text].files patterns in .sos.toml".into());
            return Ok(r);
        }
        let globs: Vec<Regex> = cfg.files.iter().map(|g| glob::to_regex(g)).collect();
        for f in git::staged_files(root)? {
            if glob::matches_any(&globs, &f) {
                if let Some(body) = git::staged_blob(root, &f)? {
                    targets.push((f, body));
                }
            }
        }
    } else {
        for f in files {
            let body = std::fs::read_to_string(root.join(f)).with_context(|| format!("reading {f}"))?;
            targets.push((f.clone(), body));
        }
    }
    for (name, body) in targets {
        for v in check(&body, &rules) {
            r.errors.push(format!("{name}:{}: [{}] \"{}\"", v.line, v.kind.tag(), v.found));
        }
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(banned: &[&str], leaks: &[&str], base: bool) -> Rules {
        Rules::from_config(&TextConfig {
            files: vec![],
            banned: banned.iter().map(|s| s.to_string()).collect(),
            leaks: leaks.iter().map(|s| s.to_string()).collect(),
            base,
            strip_thinking: true,
        })
        .unwrap()
    }

    // Ported from quality-gate tests/filter_test.rs (same inputs and expectations).
    #[test]
    fn thinking_strip() {
        let (out, v) = filter("Hello <thinking>secret</thinking> world", &rules(&[], &[], false));
        assert_eq!(out, "Hello  world");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, Kind::ThinkingArtifact);
    }

    #[test]
    fn multiline_thinking_and_reflection() {
        let (out, _) = filter("Before\n<thinking>\na\nb\n</thinking>\n<reflection>x</reflection>After", &rules(&[], &[], false));
        assert!(!out.contains("<thinking>") && !out.contains("<reflection>"));
        assert!(out.contains("Before") && out.contains("After"));
    }

    #[test]
    fn internal_leak_vietnamese() {
        let (out, v) = filter("Claude Opus 4 phân tích dữ liệu", &rules(&[], &[], true));
        assert_eq!(out.trim(), "phân tích dữ liệu");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, Kind::InternalLeak);
    }

    #[test]
    fn banned_phrase_case_insensitive_vietnamese() {
        let (out, v) = filter("TUYỆT VỜI! Kết quả tốt", &rules(&["Tuyệt vời"], &[], false));
        assert_eq!(out, "! Kết quả tốt");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, Kind::BannedPhrase);
    }

    #[test]
    fn clean_and_empty() {
        let r = rules(&["x y"], &["pipeline"], true);
        assert!(filter("Bình thường.", &r).1.is_empty());
        assert_eq!(filter("", &r).0, "");
        assert!(!filter("a\n\n\n\nb", &r).0.contains("\n\n\n"));
    }

    // Regressions fixed in the port.
    #[test]
    fn long_vietnamese_match_does_not_panic_on_truncation() {
        let long = format!("<thinking>{}</thinking>", "đường dài ".repeat(20));
        let v = check(&long, &rules(&[], &[], false));
        assert_eq!(v.len(), 1);
        assert!(v[0].found.ends_with("..."));
    }

    #[test]
    fn case_folding_that_changes_length_is_safe() {
        // "İ".to_lowercase() is 3 bytes vs 2: quality-gate 0.1.0 cut the wrong range.
        let (out, v) = filter("İstanbul act now ok", &rules(&["act now"], &[], false));
        assert_eq!(out, "İstanbul  ok");
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn check_reports_original_line_numbers() {
        let v = check("line1\n<thinking>\nx\n</thinking>\nact now", &rules(&["act now"], &[], false));
        assert_eq!(v.iter().map(|x| x.line).collect::<Vec<_>>(), vec![2, 5]);
    }

    #[test]
    fn base_can_be_disabled() {
        assert!(check("OpenAI", &rules(&[], &[], false)).is_empty());
        assert_eq!(check("OpenAI", &rules(&[], &[], true)).len(), 1);
    }

    #[test]
    fn invalid_regex_fails_fast() {
        let c = TextConfig { leaks: vec!["(".into()], ..TextConfig::default() };
        assert!(Rules::from_config(&c).is_err());
    }
}
