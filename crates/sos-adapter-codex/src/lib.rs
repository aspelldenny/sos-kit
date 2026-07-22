//! P078b1 — Codex CLI adapter foundation (detect/plan/render/verify/uninstall).
//!
//! `CodexAdapter` implements the core-defined `Adapter` trait
//! (`sos_core::adapter::Adapter`). Scope of this crate at b1:
//!
//! - `detect()` — STRUCTURAL: static facts about Codex CLI 0.145.0
//!   (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:32`), enriched by
//!   a best-effort, FAIL-SAFE `codex --version` probe. Absence of the
//!   `codex` binary must never panic or error — the structural oracle for
//!   this ticket runs on a machine WITHOUT Codex installed (Decision 5,
//!   `docs/ticket/P078b1-codex-adapter-foundation.md`).
//! - `verify()` — the machine surface of the PARTIAL-declaration
//!   mechanism: reports the 5 known Codex capability gaps
//!   (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:15-22`) with an
//!   explicit `FindingStatus` (`core/POLICY.md` oracle vocab). It never
//!   asserts `Sound` for a gap — `core/ROLES.md` separation-invariant #5:
//!   capability absence must be explicit, not simulated.
//! - `plan()` / `render()` / `uninstall()` — minimal-honest stubs. Real
//!   artifact rendering (`AGENTS.md`, `.codex/**`) is P078b2/b3 — this
//!   crate does not fake render output.

use sos_core::adapter::{
    Adapter, Artifact, Asset, Capabilities, Finding, FindingStatus, Findings, Plan, RemovalPlan,
};
use sos_core::manifest::ManagedManifest;
use std::process::Command;

pub struct CodexAdapter;

impl CodexAdapter {
    /// Static, host-neutral facts known about Codex CLI 0.145.0 from the
    /// discovery report — always present regardless of whether `codex` is
    /// installed on this machine.
    fn static_features() -> Vec<String> {
        vec![
            "hooks".to_string(),
            "multi_agent".to_string(),
            "sandbox_mode_read_only".to_string(),
            "apply_patch".to_string(),
        ]
    }

    /// Best-effort runtime probe. FAIL-SAFE: if `codex` is absent, not on
    /// PATH, or the shell-out otherwise errors, this returns `None` and the
    /// caller falls back to static-only `Capabilities` — never panics,
    /// never returns an `Err` up through `detect()` (the trait's `detect()`
    /// signature has no `Result`, so a probe failure MUST be absorbed
    /// here, not propagated).
    fn probe_codex_version() -> Option<String> {
        Command::new("codex")
            .arg("--version")
            .output()
            .ok()
            .filter(|out| out.status.success())
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string())
    }
}

impl Adapter for CodexAdapter {
    fn detect(&self) -> Capabilities {
        let mut features = Self::static_features();
        if let Some(version) = Self::probe_codex_version() {
            features.push(format!("codex_cli_present:{version}"));
        }
        // Absent codex => features stays static-only. No panic, no error.
        Capabilities { features }
    }

    fn plan(&self, _capabilities: &Capabilities) -> Plan {
        // Stub: real managed-operation planning for AGENTS.md/.codex/**
        // artifacts is P078b2/b3. Empty plan lets `install --runtime codex
        // --dry-run` run structurally (zero mutation either way).
        Plan::default()
    }

    fn render(&self, asset: &Asset, _capabilities: &Capabilities) -> Artifact {
        // Stub: real Codex-native rendering (AGENTS.md/.codex/agents/*.toml/
        // .agents/skills/*/SKILL.md/.codex/config.toml) is P078b2. This
        // passthrough mirrors ClaudeAdapter's d1-stub shape — no Codex
        // format transform happens here.
        Artifact {
            target_path: asset.identity.clone(),
            content: asset.content.clone(),
        }
    }

    fn verify(&self) -> Findings {
        // Machine surface of the PARTIAL-declaration mechanism
        // (`docs/ticket/P078b1-codex-adapter-foundation.md` Task 4). Each
        // Finding mirrors a gap in
        // `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:15-22` and
        // `adapters/codex/CAPABILITY.md`. Exactly 5 Findings — never
        // Sound.
        Findings {
            items: vec![
                Finding {
                    target_path: ".codex/agents/<role>.toml".to_string(),
                    message: "No Codex equivalent of Claude's per-role built-in tool allowlist \
                              (`tools: Read,Write,Glob`). Architect envelope enforced via \
                              PreToolUse-hook + prose + sandbox_mode=read-only, weaker than \
                              Claude's structural tool removal."
                        .to_string(),
                    status: FindingStatus::Partial,
                },
                Finding {
                    target_path: ".claude/commands/*.md (no Codex equivalent)".to_string(),
                    message: "Repo-distributed named slash commands do not exist in Codex \
                              (custom prompts are deprecated + personal-only). Replacement = \
                              repo skill invocation (`$name`)."
                        .to_string(),
                    status: FindingStatus::Missing,
                },
                Finding {
                    target_path: ".agents/skills/<name>/SKILL.md".to_string(),
                    message: "Skill-level `allowed-tools` is not mechanically enforced by \
                              Codex — declared but not gated at the tool-call layer."
                        .to_string(),
                    status: FindingStatus::Partial,
                },
                Finding {
                    target_path: "ticket-version approval gate".to_string(),
                    message: "No native semantic ticket-version approval in Codex \
                              (`approval_policy=on-request` approves operations, not a \
                              ticket version). Must be built via a persisted approved-version \
                              marker + PreToolUse guard (P078b3)."
                        .to_string(),
                    status: FindingStatus::Missing,
                },
                Finding {
                    target_path: "architect Read/Glob path interception".to_string(),
                    message: "Codex has no Read/Glob-equivalent tool to intercept — reads \
                              happen via shell (rg/sed/cat). Architect read-restriction \
                              requires inspecting shell command text (P078b3). Enforcement \
                              is additionally not unbypassable: hook config is ignored for \
                              untrusted repos and can be user-disabled — Git/CI backstops \
                              must be retained."
                        .to_string(),
                    status: FindingStatus::Partial,
                },
            ],
        }
    }

    fn uninstall(&self, _manifest: &ManagedManifest) -> RemovalPlan {
        // Stub: real safe-removal planning for rendered Codex artifacts is
        // P078b2/b3 (nothing has been rendered yet at b1).
        RemovalPlan::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_adapter_satisfies_adapter_trait_bound() {
        fn assert_adapter(_a: impl Adapter) {}
        assert_adapter(CodexAdapter);
    }

    #[test]
    fn detect_is_fail_safe_without_codex_installed() {
        // Structural oracle (Decision 5): must not panic regardless of
        // whether `codex` binary exists on this machine.
        let capabilities = CodexAdapter.detect();
        assert!(!capabilities.features.is_empty());
        assert!(capabilities.features.contains(&"hooks".to_string()));
    }

    #[test]
    fn verify_reports_exactly_five_gaps_none_sound() {
        let findings = CodexAdapter.verify();
        assert_eq!(
            findings.items.len(),
            5,
            "verify() must declare exactly 5 known Codex capability gaps"
        );
        for finding in &findings.items {
            assert_ne!(
                finding.status,
                FindingStatus::Sound,
                "gap-declaring Finding must never assert Sound: {}",
                finding.target_path
            );
        }
    }

    #[test]
    fn plan_render_uninstall_are_honest_stubs() {
        let adapter = CodexAdapter;
        let capabilities = adapter.detect();
        let plan = adapter.plan(&capabilities);
        assert!(plan.operations.is_empty(), "plan() stub must be empty, not fake-rendered");

        let asset = Asset {
            identity: "test-asset".to_string(),
            content: "test-content".to_string(),
        };
        let artifact = adapter.render(&asset, &capabilities);
        assert_eq!(artifact.target_path, "test-asset");
        assert_eq!(artifact.content, "test-content");

        let manifest = ManagedManifest {
            owner: "sos-adapter-codex".to_string(),
            source_version: "0.1.0".to_string(),
            source_identity: "test".to_string(),
            target_path: "test".to_string(),
            content_hash: "test".to_string(),
            rollback_ref: None,
        };
        let removal = adapter.uninstall(&manifest);
        assert!(removal.steps.is_empty(), "uninstall() stub must be empty");
    }
}
