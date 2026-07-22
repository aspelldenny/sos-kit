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

    // NOTE: P078b3 extends the fixed set from 10 -> 17 (7 enforcement
    // artifacts added, Context table E1-E7). See
    // `plan_renders_seventeen_artifacts` below for the current count
    // assertion — kept as ONE test, not duplicated, per Decision 5 "count
    // test tracks" note.

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

    // ── P078b3 structural oracle (enforcement artifacts, Decision 5) ────
    // Runs on ANY machine, no `codex` binary required.

    #[test]
    fn plan_renders_eighteen_artifacts() {
        // 10 (b2) + 7 (b3 enforcement: hooks.json, rules, 5 guards) + 1
        // (d2a #5: .sos-state/ticket-state.env bootstrap skeleton).
        let adapter = CodexAdapter;
        let capabilities = adapter.detect();
        let plan = adapter.plan(&capabilities);
        assert_eq!(
            plan.operations.len(),
            18,
            "plan() must render 10 (b2) + 7 (b3 enforcement) + 1 (d2a state skeleton) = 18 artifacts"
        );
    }

    #[test]
    fn hooks_json_is_valid_json_with_expected_events() {
        let rendered = rendered_map();
        let content = rendered.get(".codex/hooks.json").unwrap();
        let value: serde_json::Value =
            serde_json::from_str(content).expect("hooks.json must be valid JSON");
        let hooks = value.get("hooks").expect("hooks.json must have a `hooks` key");
        for event in [
            "SessionStart",
            "UserPromptSubmit",
            "SubagentStart",
            "SubagentStop",
            "PreToolUse",
            "Stop",
        ] {
            assert!(hooks.get(event).is_some(), "hooks.json missing event `{event}`");
        }
        assert!(content.contains("scripts/codex/architect-guard.sh"));
        assert!(content.contains("scripts/codex/orchestrator-guard.sh"));
        assert!(content.contains("scripts/codex/block-env-edit.sh"));
        assert!(content.contains("scripts/codex/approval-gate.sh"));
        assert!(content.contains("scripts/codex/idea-smell.sh"));
    }

    #[test]
    fn enforcement_artifacts_carry_partial_honest_note() {
        let rendered = rendered_map();
        let hooks = rendered.get(".codex/hooks.json").unwrap();
        assert!(hooks.contains("bypassable"), "hooks.json must declare bypassable honestly");

        let architect_guard = rendered.get("scripts/codex/architect-guard.sh").unwrap();
        assert!(architect_guard.contains("PARTIAL"));
        assert!(architect_guard.contains("fail-CLOSED") || architect_guard.contains("fail-closed"));

        let rules = rendered.get(".codex/rules/exec-policy.rules").unwrap();
        assert!(rules.contains("PARTIAL"));
        assert!(rules.contains("prefix_rule"));
    }

    // ── P078d1 schema-shape oracle (Codex 0.145.0 live-dogfood, P079) ──────
    //
    // The tests above (`toml_artifacts_parse_ok`, `hooks_json_is_valid_json_
    // with_expected_events`) only assert generic valid-TOML / valid-JSON +
    // key presence -- that oracle PASSED while all 3 startup-blockers below
    // were still live against real Codex 0.145.0 (P079 dogfood). These 3
    // tests assert the Codex-SPECIFIC shape each blocker violated.
    //
    // Honest limit (ghi rõ, b2/b3-gap lesson): structural-valid + shape-assert
    // PASS here still does NOT prove Codex 0.145.0 accepts the output --
    // only a live Codex run is ground-truth. This oracle is a hand-coded
    // approximation of the 3 confirmed error messages from P079, not the
    // real Codex deserializer.

    #[test]
    fn config_toml_root_settings_are_not_nested_in_a_table() {
        // Bug #1 (P079): sandbox_mode/approval_policy rendered AFTER [agents]
        // -> TOML table-scope binds them into [agents] -> Codex 0.145.0:
        // "invalid type string \"workspace-write\", expected struct AgentRoleToml".
        let rendered = rendered_map();
        let content = rendered.get(".codex/config.toml").unwrap();
        let value: toml::Value =
            content.parse().expect("config.toml must be valid TOML");

        assert!(
            value.get("sandbox_mode").is_some(),
            "sandbox_mode must be a ROOT key, not nested under a table"
        );
        assert!(
            value.get("approval_policy").is_some(),
            "approval_policy must be a ROOT key, not nested under a table"
        );

        // Negative-check the specific failure mode: it must NOT have landed
        // inside [agents] (that was the exact bug).
        if let Some(agents) = value.get("agents") {
            assert!(
                agents.get("sandbox_mode").is_none(),
                "sandbox_mode must NOT be nested under [agents] (P079 bug #1)"
            );
            assert!(
                agents.get("approval_policy").is_none(),
                "approval_policy must NOT be nested under [agents] (P079 bug #1)"
            );
        }
    }

    #[test]
    fn hooks_json_top_level_keys_are_description_and_hooks_only() {
        // Bug #3 (P079): `_provenance` + `_partial_note` top-level fields ->
        // Codex 0.145.0: "unknown field _provenance, expected description or hooks".
        let rendered = rendered_map();
        let content = rendered.get(".codex/hooks.json").unwrap();
        let value: serde_json::Value =
            serde_json::from_str(content).expect("hooks.json must be valid JSON");
        let obj = value.as_object().expect("hooks.json top level must be an object");

        for key in obj.keys() {
            assert!(
                key == "description" || key == "hooks",
                "hooks.json top-level key `{key}` is not accepted by Codex 0.145.0 \
                 (only `description`/`hooks` allowed) -- P079 bug #3"
            );
        }
        assert!(!obj.contains_key("_provenance"), "_provenance is not a valid top-level field (P079 bug #3)");
        assert!(!obj.contains_key("_partial_note"), "_partial_note is not a valid top-level field (P079 bug #3)");
        // Provenance/PARTIAL trace must survive, folded into `description`.
        let description = obj.get("description").and_then(|v| v.as_str()).unwrap_or("");
        assert!(description.contains("bypassable"), "provenance/PARTIAL note must be folded into `description`");
    }

    #[test]
    fn rules_exec_policy_pattern_is_list_form_not_bare_string() {
        // Bug #2 (P079): `pattern = "git push --force"` (bare string) ->
        // Codex 0.145.0: "pattern doesn't match, expected list, actual string".
        //
        // Oracle note (honest, Worker CHALLENGE Turn 1 finding + Architect
        // ACCEPT V2): `.rules` content is STARLARK function-call syntax
        // (`prefix_rule(pattern = [...], decision = "...")`), NOT TOML/JSON --
        // `toml::from_str` on this content deterministically ERRORS (syntax
        // mismatch, confirmed empirically in Worker CHALLENGE). Adding a real
        // Starlark parser crate for a test-only assertion is Tầng 1 new-dep
        // overkill (WORKFLOW_V2.2.md Sub-mech B). This is a STRUCTURAL-STRING
        // oracle -- weaker than a real parser, but sufficient to catch the
        // exact bug class (bare-string vs list) and to negative-test.
        let rendered = rendered_map();
        let content = rendered.get(".codex/rules/exec-policy.rules").unwrap();

        assert!(
            content.contains("pattern = [\"git\", \"push\", \"--force\"]"),
            "pattern must be rendered as a token LIST, e.g. pattern = [\"git\", \"push\", \"--force\"]"
        );
        assert!(
            !content.contains("pattern = \""),
            "pattern must NOT be a bare string (P079 bug #2) -- found `pattern = \"...\"` form"
        );
    }

    #[cfg(unix)]
    mod mock_payload_oracle {
        //! Core oracle (Decision 5): feed REAL/derived Codex apply_patch+Bash
        //! stdin payloads to each rendered guard script and assert the
        //! block(exit 2)/allow(exit 0) outcome. `#[cfg(unix)]`-gated (anchor
        //! #11) — mirrors the existing precedent in
        //! `crates/sos-install/tests/tools.rs` / `crates/sos-cli/tests/parity.rs`
        //! for bash-exec tests; content-pattern assertions above stay
        //! cross-platform.
        use super::*;
        use std::fs;
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        use std::path::PathBuf;
        use std::process::{Command, Stdio};
        use std::sync::atomic::{AtomicU64, Ordering};

        static COUNTER: AtomicU64 = AtomicU64::new(0);

        /// The 4 REAL Codex hook payloads (ground-truth, P078b3 Debate Log Turn
        /// 2) — captured live from Codex CLI gpt-5.6 by Sếp, NOT invented.
        const REAL_FIXTURES: &str =
            include_str!("../tests/fixtures/codex-apply-patch-payloads.jsonl");

        fn real_fixture_line(idx: usize) -> &'static str {
            REAL_FIXTURES.lines().nth(idx).expect("fixture line must exist")
        }

        /// Renders `identity`'s guard script into a throwaway temp dir, feeds
        /// `stdin_json` on stdin, and returns the exit code. `setup` may
        /// pre-create marker files / state files in the temp dir before the
        /// guard runs (mirrors the real `.sos-state/` convention).
        fn run_guard(identity: &str, stdin_json: &str, setup: impl FnOnce(&PathBuf)) -> i32 {
            let n = COUNTER.fetch_add(1, Ordering::SeqCst);
            let dir = std::env::temp_dir().join(format!(
                "sos-codex-guard-test-{}-{}-{}",
                std::process::id(),
                n,
                identity
            ));
            fs::create_dir_all(&dir).unwrap();
            setup(&dir);

            let script_path = dir.join("guard.sh");
            fs::write(&script_path, templates::content_for(identity)).unwrap();
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();

            let mut child = Command::new("bash")
                .arg(&script_path)
                .current_dir(&dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("spawn bash guard");
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(stdin_json.as_bytes())
                .unwrap();
            let status = child.wait().expect("wait on guard process");
            let _ = fs::remove_dir_all(&dir);
            status.code().unwrap_or(-1)
        }

        fn no_setup(_dir: &PathBuf) {}

        fn with_architect_active(dir: &PathBuf) {
            fs::create_dir_all(dir.join(".sos-state")).unwrap();
            fs::write(dir.join(".sos-state/architect-active"), "").unwrap();
        }

        fn with_worker_active(dir: &PathBuf) {
            fs::create_dir_all(dir.join(".sos-state")).unwrap();
            fs::write(dir.join(".sos-state/worker-active"), "").unwrap();
        }

        fn synthetic_apply_patch(action: &str, path: &str) -> String {
            format!(
                r#"{{"hook_event_name":"PreToolUse","permission_mode":"bypassPermissions","cwd":"/tmp/x","tool_name":"apply_patch","tool_input":{{"command":"*** Begin Patch\n*** {action} File: {path}\n+x\n*** End Patch"}},"tool_use_id":"t"}}"#
            )
        }

        fn synthetic_bash(command_note: &str) -> String {
            format!(
                r#"{{"hook_event_name":"PreToolUse","permission_mode":"bypassPermissions","cwd":"/tmp/x","tool_name":"Bash","tool_input":{{"command":"{command_note}"}},"tool_use_id":"t"}}"#
            )
        }

        /// P078d2a #6 multi-path bypass fixture: a SINGLE apply_patch with TWO
        /// `*** ... File:` markers in ONE patch body -- (action1, path1) FIRST,
        /// (action2, path2) SECOND. This is the exact shape the pre-fix
        /// `head -n1` extraction only ever saw the first of.
        fn synthetic_apply_patch_multi(action1: &str, path1: &str, action2: &str, path2: &str) -> String {
            format!(
                r#"{{"hook_event_name":"PreToolUse","permission_mode":"bypassPermissions","cwd":"/tmp/x","tool_name":"apply_patch","tool_input":{{"command":"*** Begin Patch\n*** {action1} File: {path1}\n+x\n*** {action2} File: {path2}\n+y\n*** End Patch"}},"tool_use_id":"t"}}"#
            )
        }

        #[test]
        fn architect_guard_allows_when_marker_absent() {
            // No architect-active marker -> allow anything, even the real
            // apply_patch(Add) fixture line.
            let code = run_guard(
                templates::GUARD_ARCHITECT,
                real_fixture_line(0),
                no_setup,
            );
            assert_eq!(code, 0);
        }

        #[test]
        fn architect_guard_blocks_apply_patch_on_non_ticket_file_real_fixture() {
            // Real fixture line 0 (Add File: foo.txt) — architect-active,
            // foo.txt is not a phiếu file -> BLOCK.
            let code = run_guard(
                templates::GUARD_ARCHITECT,
                real_fixture_line(0),
                with_architect_active,
            );
            assert_eq!(code, 2);
        }

        #[test]
        fn architect_guard_blocks_apply_patch_update_and_delete_real_fixture() {
            // Real fixture lines 1 (Update) and 2 (Delete) — same envelope,
            // same non-ticket path -> BLOCK.
            for idx in [1usize, 2] {
                let code = run_guard(
                    templates::GUARD_ARCHITECT,
                    real_fixture_line(idx),
                    with_architect_active,
                );
                assert_eq!(code, 2, "fixture line {idx} must BLOCK");
            }
        }

        #[test]
        fn architect_guard_allows_apply_patch_on_ticket_file() {
            let payload = synthetic_apply_patch("Add", "docs/ticket/P099-example.md");
            let code = run_guard(templates::GUARD_ARCHITECT, &payload, with_architect_active);
            assert_eq!(code, 0);
        }

        #[test]
        fn architect_guard_fails_closed_on_unparseable_apply_patch() {
            let payload = r#"{"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{"command":"not a real patch at all"},"tool_use_id":"t"}"#;
            let code = run_guard(templates::GUARD_ARCHITECT, payload, with_architect_active);
            assert_eq!(code, 2, "unparseable apply_patch must fail-CLOSED");
        }

        #[test]
        fn architect_guard_allows_real_bash_read_fixture_non_src() {
            // Real fixture line 3: Bash `sed -n "1p" read-target.txt` -- does
            // not touch src/ -> ALLOW even with architect-active.
            let code = run_guard(
                templates::GUARD_ARCHITECT,
                real_fixture_line(3),
                with_architect_active,
            );
            assert_eq!(code, 0);
        }

        #[test]
        fn architect_guard_blocks_bash_read_on_src() {
            let payload = synthetic_bash("rg TODO crates/sos-core/src/lib.rs");
            let code = run_guard(templates::GUARD_ARCHITECT, &payload, with_architect_active);
            assert_eq!(code, 2);
        }

        #[test]
        fn architect_guard_allows_bash_read_on_docs() {
            let payload = synthetic_bash("rg TODO docs/BACKLOG.md");
            let code = run_guard(templates::GUARD_ARCHITECT, &payload, with_architect_active);
            assert_eq!(code, 0);
        }

        #[test]
        fn orchestrator_guard_blocks_product_source_without_worker_marker() {
            let payload = synthetic_apply_patch("Add", "src/main.rs");
            let code = run_guard(templates::GUARD_ORCHESTRATOR, &payload, no_setup);
            assert_eq!(code, 2);
        }

        #[test]
        fn orchestrator_guard_allows_product_source_with_worker_marker() {
            let payload = synthetic_apply_patch("Add", "src/main.rs");
            let code = run_guard(templates::GUARD_ORCHESTRATOR, &payload, with_worker_active);
            assert_eq!(code, 0);
        }

        #[test]
        fn orchestrator_guard_allows_docs_without_marker() {
            let payload = synthetic_apply_patch("Update", "docs/DISCOVERIES.md");
            let code = run_guard(templates::GUARD_ORCHESTRATOR, &payload, no_setup);
            assert_eq!(code, 0);
        }

        #[test]
        fn orchestrator_guard_fails_closed_on_unparseable() {
            let payload = r#"{"tool_name":"apply_patch","tool_input":{"command":"garbage"},"tool_use_id":"t"}"#;
            let code = run_guard(templates::GUARD_ORCHESTRATOR, payload, no_setup);
            assert_eq!(code, 2);
        }

        #[test]
        fn block_env_edit_blocks_dotenv() {
            let payload = synthetic_apply_patch("Update", ".env");
            let code = run_guard(templates::GUARD_BLOCK_ENV, &payload, no_setup);
            assert_eq!(code, 2);
        }

        #[test]
        fn block_env_edit_allows_dotenv_example() {
            let payload = synthetic_apply_patch("Update", ".env.example");
            let code = run_guard(templates::GUARD_BLOCK_ENV, &payload, no_setup);
            assert_eq!(code, 0);
        }

        #[test]
        fn block_env_edit_allows_unrelated_file() {
            let payload = synthetic_apply_patch("Add", "src/config.rs");
            let code = run_guard(templates::GUARD_BLOCK_ENV, &payload, no_setup);
            assert_eq!(code, 0);
        }

        #[test]
        fn approval_gate_blocks_on_version_mismatch() {
            let payload = synthetic_apply_patch("Update", "src/main.rs");
            let code = run_guard(templates::GUARD_APPROVAL_GATE, &payload, |dir| {
                fs::create_dir_all(dir.join(".sos-state")).unwrap();
                fs::write(
                    dir.join(".sos-state/ticket-state.env"),
                    "version=V2\napproved_version=V1\n",
                )
                .unwrap();
            });
            assert_eq!(code, 2);
        }

        #[test]
        fn approval_gate_allows_on_version_match() {
            let payload = synthetic_apply_patch("Update", "src/main.rs");
            let code = run_guard(templates::GUARD_APPROVAL_GATE, &payload, |dir| {
                fs::create_dir_all(dir.join(".sos-state")).unwrap();
                fs::write(
                    dir.join(".sos-state/ticket-state.env"),
                    "version=V2\napproved_version=V2\n",
                )
                .unwrap();
            });
            assert_eq!(code, 0);
        }

        #[test]
        fn approval_gate_fails_closed_when_state_file_missing() {
            let payload = synthetic_apply_patch("Update", "src/main.rs");
            let code = run_guard(templates::GUARD_APPROVAL_GATE, &payload, no_setup);
            assert_eq!(code, 2);
        }

        #[test]
        fn approval_gate_allows_editing_ticket_file_regardless_of_state() {
            let payload = synthetic_apply_patch("Update", "docs/ticket/P078b3-x.md");
            let code = run_guard(templates::GUARD_APPROVAL_GATE, &payload, no_setup);
            assert_eq!(code, 0);
        }

        // ── P078d2a #6 multi-path bypass -- the core security oracle ──────
        // Pre-fix, every guard extracted ONLY the first `*** ... File:` path
        // (`head -n1`). A patch with an allowed path FIRST and a forbidden
        // path SECOND would exit ALLOW on the first match without ever
        // looking at the second. These tests assert BLOCK for exactly that
        // shape across every guard that has an allow-list (anchor #6).

        #[test]
        fn architect_guard_blocks_multipath_bypass_ticket_first_forbidden_second() {
            let payload = synthetic_apply_patch_multi(
                "Update",
                "docs/ticket/P099-example.md",
                "Add",
                ".env",
            );
            let code = run_guard(templates::GUARD_ARCHITECT, &payload, with_architect_active);
            assert_eq!(code, 2, "multi-path bypass (ticket-first, .env-second) must BLOCK");
        }

        #[test]
        fn architect_guard_blocks_multipath_bypass_ticket_first_state_file_second() {
            let payload = synthetic_apply_patch_multi(
                "Update",
                "docs/ticket/P099-example.md",
                "Add",
                ".sos-state/ticket-state.env",
            );
            let code = run_guard(templates::GUARD_ARCHITECT, &payload, with_architect_active);
            assert_eq!(code, 2, "multi-path bypass (ticket-first, state-file-second) must BLOCK");
        }

        #[test]
        fn architect_guard_allows_multipath_all_ticket_files() {
            // No-regress: multiple paths, ALL exempt -> still ALLOW.
            let payload = synthetic_apply_patch_multi(
                "Add",
                "docs/ticket/P099-a.md",
                "Update",
                "docs/ticket/P099-b.md",
            );
            let code = run_guard(templates::GUARD_ARCHITECT, &payload, with_architect_active);
            assert_eq!(code, 0);
        }

        #[test]
        fn orchestrator_guard_blocks_multipath_bypass_docs_first_source_second() {
            let payload = synthetic_apply_patch_multi(
                "Update",
                "docs/DISCOVERIES.md",
                "Add",
                "src/evil.rs",
            );
            let code = run_guard(templates::GUARD_ORCHESTRATOR, &payload, no_setup);
            assert_eq!(code, 2, "multi-path bypass (docs-first, src-second, no worker marker) must BLOCK");
        }

        #[test]
        fn block_env_edit_blocks_multipath_bypass_src_first_env_second() {
            let payload = synthetic_apply_patch_multi(
                "Add",
                "src/config.rs",
                "Update",
                ".env",
            );
            let code = run_guard(templates::GUARD_BLOCK_ENV, &payload, no_setup);
            assert_eq!(code, 2, "multi-path bypass (src-first, .env-second) must BLOCK");
        }

        #[test]
        fn approval_gate_blocks_multipath_bypass_ticket_first_source_second() {
            let payload = synthetic_apply_patch_multi(
                "Update",
                "docs/ticket/P078b3-x.md",
                "Add",
                "src/evil.rs",
            );
            // No state file -> src/evil.rs is a non-ticket path that is NOT
            // ticket-state.env alone -> fail-CLOSED BLOCK (not the #5 exemption).
            let code = run_guard(templates::GUARD_APPROVAL_GATE, &payload, no_setup);
            assert_eq!(code, 2, "multi-path bypass (ticket-first, src-second) must BLOCK");
        }

        // ── P078d2a #5 approval bootstrap (coupled with #6 above) ─────────

        #[test]
        fn approval_gate_allows_bootstrap_when_only_state_file_touched() {
            // State file missing, patch touches ONLY .sos-state/ticket-state.env
            // -> self-bootstrap exemption -> ALLOW.
            let payload = synthetic_apply_patch("Add", ".sos-state/ticket-state.env");
            let code = run_guard(templates::GUARD_APPROVAL_GATE, &payload, no_setup);
            assert_eq!(code, 0, "bootstrap-only patch (state file missing) must ALLOW");
        }

        #[test]
        fn approval_gate_blocks_bootstrap_when_state_file_plus_other_path() {
            // State file missing, patch touches ticket-state.env AND another
            // path in the SAME patch -> #6 all-path check sees the second
            // path -> exemption does NOT apply -> BLOCK.
            let payload = synthetic_apply_patch_multi(
                "Add",
                ".sos-state/ticket-state.env",
                "Add",
                "src/evil.rs",
            );
            let code = run_guard(templates::GUARD_APPROVAL_GATE, &payload, no_setup);
            assert_eq!(code, 2, "state-file + a second path bundled in one patch must BLOCK");
        }

        #[test]
        fn all_guard_scripts_are_bash_syntax_clean() {
            for identity in [
                templates::GUARD_ARCHITECT,
                templates::GUARD_ORCHESTRATOR,
                templates::GUARD_BLOCK_ENV,
                templates::GUARD_APPROVAL_GATE,
                templates::GUARD_IDEA_SMELL,
            ] {
                let n = COUNTER.fetch_add(1, Ordering::SeqCst);
                let dir = std::env::temp_dir()
                    .join(format!("sos-codex-guard-syntax-{}-{}-{}", std::process::id(), n, identity));
                fs::create_dir_all(&dir).unwrap();
                let script_path = dir.join("guard.sh");
                fs::write(&script_path, templates::content_for(identity)).unwrap();
                let status = Command::new("bash")
                    .arg("-n")
                    .arg(&script_path)
                    .status()
                    .expect("run bash -n");
                assert!(status.success(), "{identity} failed `bash -n` syntax check");
                let _ = fs::remove_dir_all(&dir);
            }
        }

        #[test]
        fn idea_smell_guard_detects_smell_phrase() {
            let payload = r#"{"prompt":"anh nghĩ ra một ý tưởng mới"}"#;
            // Not exit-code gated (idea-smell always exits 0) — assert stdout instead.
            let n = COUNTER.fetch_add(1, Ordering::SeqCst);
            let dir = std::env::temp_dir()
                .join(format!("sos-codex-idea-smell-{}-{}", std::process::id(), n));
            fs::create_dir_all(&dir).unwrap();
            let script_path = dir.join("guard.sh");
            fs::write(&script_path, templates::content_for(templates::GUARD_IDEA_SMELL)).unwrap();
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
            let mut child = Command::new("bash")
                .arg(&script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            child.stdin.as_mut().unwrap().write_all(payload.as_bytes()).unwrap();
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success());
            assert!(String::from_utf8_lossy(&output.stdout).contains("Idea-smell"));
            let _ = fs::remove_dir_all(&dir);
        }
    }
}
