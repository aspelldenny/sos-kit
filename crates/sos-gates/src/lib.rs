//! SOS Kit v3 gates — agent-neutral checks run by git hooks, CI, or a person.
//!
//! Merged from the former sister tools (decision 2026-10-09, docs/plans/V3_TRIAGE_2026-10-08.md §G):
//! `quality-gate check` → [`text`], `doc-rotate cap-check` → [`docs`],
//! `claude-hooks features-guard` → [`features`] (now a git-level check, any agent),
//! `doctor runtime-scan` → [`local_secrets`] (now delegates patterns to gitleaks).
//!
//! Every gate returns a [`Report`]; a failed report always carries a "how to fix" line.

pub mod config;
pub mod docs;
pub mod features;
pub mod git;
pub mod glob;
pub mod local_secrets;
pub mod text;

/// Outcome of one gate.
#[derive(Debug, Default, Clone)]
pub struct Report {
    pub gate: &'static str,
    /// Problems that block (exit 1).
    pub errors: Vec<String>,
    /// Problems that only warn.
    pub warnings: Vec<String>,
    /// How to fix the errors; printed only when there are errors.
    pub fix: String,
    /// Set when the gate had nothing to check (e.g. not configured).
    pub skipped: Option<String>,
}

impl Report {
    pub fn new(gate: &'static str) -> Self {
        Self { gate, ..Default::default() }
    }
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
    /// Human-readable lines for the terminal.
    pub fn render(&self) -> String {
        let mut out = String::new();
        if let Some(why) = &self.skipped {
            out.push_str(&format!("[{}] skipped: {why}\n", self.gate));
            return out;
        }
        for w in &self.warnings {
            out.push_str(&format!("[{}] warning: {w}\n", self.gate));
        }
        for e in &self.errors {
            out.push_str(&format!("[{}] {e}\n", self.gate));
        }
        if !self.errors.is_empty() && !self.fix.is_empty() {
            out.push_str(&format!("  → How to fix: {}\n", self.fix));
        }
        if self.errors.is_empty() && self.warnings.is_empty() {
            out.push_str(&format!("[{}] ok\n", self.gate));
        }
        out
    }
}
