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
//! - `plan()` / `render()` — **P078b2 LIVE**: declarative render of the 10
//!   Codex-native artifacts (`AGENTS.md`, 4× `.codex/agents/*.toml`, 4×
//!   `.agents/skills/*/SKILL.md`, `.codex/config.toml`) —
//!   `docs/ticket/P078b2-codex-render.md`. `render()` is per-`Asset`
//!   (trait shape, `sos_core::adapter::Adapter::render`); `plan()`
//!   enumerates the fixed 10-`Asset` set (`templates::all_assets()`) and
//!   calls `render()` on each, mapping `Artifact{target_path, content}` →
//!   `ManagedOperation{description, target_path, content}` for the
//!   install engine (`sos-install::engine`) to apply — engine code is
//!   untouched (it already consumes `ManagedOperation` generically).
//!   Content is a crate-embedded template/format-string
//!   (`templates.rs`) — `render()` never reads `core/**` off the
//!   filesystem; every artifact carries a pointer to a stable core ID,
//!   not a copy of role/skill semantics (Decision 1/2,
//!   `core/ASSETS.md:51`). `uninstall()` remains an honest stub (real
//!   safe-removal for rendered artifacts is P078b3).

mod templates;

use sos_core::adapter::{
    Adapter, Artifact, Asset, Capabilities, Finding, FindingStatus, Findings, ManagedOperation,
    Plan, RemovalPlan,
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

    fn plan(&self, capabilities: &Capabilities) -> Plan {
        // P078b2 LIVE: enumerate the fixed 10-Asset set, render() each,
        // map Artifact -> ManagedOperation. No fs mutation here (Plan
        // construction only) — engine (sos-install::engine) resolves +
        // applies against the real filesystem separately.
        let operations = templates::all_assets()
            .iter()
            .map(|asset| {
                let artifact = self.render(asset, capabilities);
                ManagedOperation {
                    description: format!("render Codex artifact `{}`", asset.identity),
                    target_path: artifact.target_path,
                    content: artifact.content,
                }
            })
            .collect();
        Plan { operations }
    }

    fn render(&self, asset: &Asset, _capabilities: &Capabilities) -> Artifact {
        // P078b2 LIVE: identity selects a crate-embedded template
        // (templates.rs) — no `core/**` filesystem read at render time
        // (Decision 1). Unknown identity panics (programmer error — the
        // only caller is plan()'s fixed 10-Asset set).
        Artifact {
            target_path: templates::target_path_for(&asset.identity).to_string(),
            content: templates::content_for(&asset.identity),
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
    fn uninstall_remains_an_honest_stub() {
        // Real safe-removal for rendered artifacts is P078b3 — b2 only
        // renders (declarative), never touches removal.
        let manifest = ManagedManifest {
            owner: "sos-adapter-codex".to_string(),
            source_version: "0.1.0".to_string(),
            source_identity: "test".to_string(),
            target_path: "test".to_string(),
            content_hash: "test".to_string(),
            rollback_ref: None,
        };
        let removal = CodexAdapter.uninstall(&manifest);
        assert!(removal.steps.is_empty(), "uninstall() stub must be empty");
    }

    // -- P078b2 structural oracle (Decision 5) --------------------------
    // Runs on ANY machine, no `codex` binary required. Fresh in-memory
    // render only — never writes into the sos-kit repo itself
    // (Decision 6, checked separately by the Regression gate: `git
    // status` must show no `.codex/`/`AGENTS.md`/`.agents/` in this repo).

    #[test]
    fn plan_renders_exactly_ten_artifacts() {
        let adapter = CodexAdapter;
        let capabilities = adapter.detect();
        let plan = adapter.plan(&capabilities);
        assert_eq!(
            plan.operations.len(),
            10,
            "plan() must render exactly 10 declarative artifacts (Decision 4: 4 skills, no init)"
        );
    }

    #[test]
    fn render_is_per_asset_and_pure() {
        // Trait shape check (Worker CHALLENGE anchor #1/#3): render() takes
        // ONE Asset, returns ONE Artifact — no fs side-effect.
        let adapter = CodexAdapter;
        let capabilities = adapter.detect();
        let asset = Asset {
            identity: templates::AGENT_ARCHITECT.to_string(),
            content: templates::AGENT_ARCHITECT.to_string(),
        };
        let artifact = adapter.render(&asset, &capabilities);
        assert_eq!(artifact.target_path, ".codex/agents/architect.toml");
        assert!(!artifact.content.is_empty());
    }

    fn rendered_map() -> std::collections::HashMap<String, String> {
        let adapter = CodexAdapter;
        let capabilities = adapter.detect();
        adapter
            .plan(&capabilities)
            .operations
            .into_iter()
            .map(|op| (op.target_path, op.content))
            .collect()
    }

    #[test]
    fn toml_artifacts_parse_ok() {
        let rendered = rendered_map();
        for path in [
            ".codex/agents/architect.toml",
            ".codex/agents/worker.toml",
            ".codex/agents/advisory-watch.toml",
            ".codex/agents/boundary-check.toml",
            ".codex/config.toml",
        ] {
            let content = rendered.get(path).unwrap_or_else(|| panic!("missing rendered artifact {path}"));
            content
                .parse::<toml::Value>()
                .unwrap_or_else(|e| panic!("{path} failed to parse as TOML: {e}"));
        }
    }

    #[test]
    fn agent_toml_has_three_required_fields() {
        let rendered = rendered_map();
        for path in [
            ".codex/agents/architect.toml",
            ".codex/agents/worker.toml",
            ".codex/agents/advisory-watch.toml",
            ".codex/agents/boundary-check.toml",
        ] {
            let content = rendered.get(path).unwrap();
            let value: toml::Value = content.parse().unwrap();
            let table = value.as_table().unwrap();
            for field in ["name", "description", "developer_instructions"] {
                assert!(
                    table.contains_key(field),
                    "{path} missing required field `{field}` (report anchor #5)"
                );
            }
        }
    }

    #[test]
    fn config_toml_has_mcp_and_agents_sections() {
        let rendered = rendered_map();
        let content = rendered.get(".codex/config.toml").unwrap();
        let value: toml::Value = content.parse().unwrap();
        assert!(value.get("mcp_servers").and_then(|v| v.get("doctor")).is_some());
        assert!(value.get("agents").is_some());
    }

    #[test]
    fn skill_frontmatter_has_name_and_description() {
        let rendered = rendered_map();
        for path in [
            ".agents/skills/idea/SKILL.md",
            ".agents/skills/forge/SKILL.md",
            ".agents/skills/apply/SKILL.md",
            ".agents/skills/retro/SKILL.md",
        ] {
            let content = rendered.get(path).unwrap();
            assert!(content.starts_with("---\n"), "{path} must open with YAML frontmatter fence");
            let end = content[4..].find("---").unwrap_or_else(|| panic!("{path} missing closing frontmatter fence"));
            let frontmatter = &content[4..4 + end];
            assert!(frontmatter.contains("name:"), "{path} frontmatter missing name:");
            assert!(frontmatter.contains("description:"), "{path} frontmatter missing description:");
        }
    }

    #[test]
    fn agents_md_is_well_formed_and_non_empty() {
        let rendered = rendered_map();
        let content = rendered.get("AGENTS.md").unwrap();
        assert!(!content.trim().is_empty());
        assert!(content.contains("orchestrator"));
        assert!(content.contains("core/ROLES.md#orchestrator"));
    }

    #[test]
    fn every_artifact_contains_its_core_id_pointer() {
        let rendered = rendered_map();
        let expectations: &[(&str, &str)] = &[
            ("AGENTS.md", "core/ROLES.md#orchestrator"),
            (".codex/agents/architect.toml", "core/ROLES.md#architect"),
            (".codex/agents/worker.toml", "core/ROLES.md#worker"),
            (".codex/agents/advisory-watch.toml", "core/ROLES.md#advisory_watch"),
            (".codex/agents/boundary-check.toml", "core/ROLES.md#boundary_check"),
            (".agents/skills/idea/SKILL.md", "core/WORKFLOW.md"),
            (".agents/skills/forge/SKILL.md", "core/WORKFLOW.md"),
            (".agents/skills/apply/SKILL.md", "core/WORKFLOW.md"),
            (".agents/skills/retro/SKILL.md", "core/WORKFLOW.md"),
            (".codex/config.toml", "core/ASSETS.md"),
        ];
        for (path, pointer) in expectations {
            let content = rendered.get(*path).unwrap();
            assert!(
                content.contains(pointer),
                "{path} must contain core-ID pointer `{pointer}` (Decision 2 — pointer, not copy)"
            );
        }
    }

    #[test]
    fn architect_toml_carries_partial_marker_others_stay_honest() {
        let rendered = rendered_map();
        let architect = rendered.get(".codex/agents/architect.toml").unwrap();
        assert!(architect.contains("PARTIAL"), "architect.toml must carry the PARTIAL envelope marker (Decision 3)");
        assert!(architect.contains("workspace-write"));

        let worker = rendered.get(".codex/agents/worker.toml").unwrap();
        assert!(worker.contains("workspace-write"));

        for path in [
            ".codex/agents/advisory-watch.toml",
            ".codex/agents/boundary-check.toml",
        ] {
            let content = rendered.get(path).unwrap();
            assert!(content.contains(r#"sandbox_mode = "read-only""#), "{path} must be honest read-only, not PARTIAL-tool-scoped");
        }
    }

    // NOTE (Decision 1, no-core-fs-read claim): NOT tested via a
    // CWD-mutating unit test — `std::env::set_current_dir` is
    // process-global and would race with other tests running in
    // parallel in the same test binary (flaky by construction). The
    // claim is instead verified by code review: `templates.rs` contains
    // zero `std::fs`/`std::path::Path::new("core"` reads — every
    // `content_for`/`target_path_for` arm is a pure string literal/match.
    // See `docs/discoveries/P078b2.md` for the citation.
}
