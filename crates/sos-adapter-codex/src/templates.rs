//! P078b2 — declarative render templates.
//!
//! Crate-embedded template/format-strings for the 10 Codex-native
//! artifacts (`docs/ticket/P078b2-codex-render.md` Context table).
//! **Decision 1** (that phiếu): content is generated from string literals
//! in THIS crate — `render()` never reads `core/**` off the filesystem at
//! render time. Every template carries a **pointer** to a stable core ID
//! (`core/ROLES.md#<role_id>`, `core/WORKFLOW.md`, ...) — it does NOT copy
//! role/skill semantics (`core/ROLES.md` sep-inv #5, `core/ASSETS.md:51`).
//!
//! `asset_identity(..)` values are the stable keys `render()` matches on;
//! `all_assets()` is the fixed 10-Asset set `plan()` enumerates.

use sos_core::adapter::Asset;

/// The 10 stable Asset identities b2 renders. Order matches the Context
/// table in `docs/ticket/P078b2-codex-render.md`.
pub const AGENTS_MD: &str = "agents_md";
pub const AGENT_ARCHITECT: &str = "codex_agent_architect";
pub const AGENT_WORKER: &str = "codex_agent_worker";
pub const AGENT_ADVISORY_WATCH: &str = "codex_agent_advisory_watch";
pub const AGENT_BOUNDARY_CHECK: &str = "codex_agent_boundary_check";
pub const SKILL_IDEA: &str = "codex_skill_idea";
pub const SKILL_FORGE: &str = "codex_skill_forge";
pub const SKILL_APPLY: &str = "codex_skill_apply";
pub const SKILL_RETRO: &str = "codex_skill_retro";
pub const CONFIG_TOML: &str = "codex_config_toml";

/// Fixed 10-Asset set. `content` is unused by `render()` (identity alone
/// selects the crate template) — kept as a stable echo of `identity` so
/// the generic core `Asset` type has a non-empty, deterministic payload.
pub fn all_assets() -> Vec<Asset> {
    [
        AGENTS_MD,
        AGENT_ARCHITECT,
        AGENT_WORKER,
        AGENT_ADVISORY_WATCH,
        AGENT_BOUNDARY_CHECK,
        SKILL_IDEA,
        SKILL_FORGE,
        SKILL_APPLY,
        SKILL_RETRO,
        CONFIG_TOML,
    ]
    .into_iter()
    .map(|identity| Asset {
        identity: identity.to_string(),
        content: identity.to_string(),
    })
    .collect()
}

/// `target_path` (relative to project root) for a given Asset identity.
pub fn target_path_for(identity: &str) -> &'static str {
    match identity {
        AGENTS_MD => "AGENTS.md",
        AGENT_ARCHITECT => ".codex/agents/architect.toml",
        AGENT_WORKER => ".codex/agents/worker.toml",
        AGENT_ADVISORY_WATCH => ".codex/agents/advisory-watch.toml",
        AGENT_BOUNDARY_CHECK => ".codex/agents/boundary-check.toml",
        SKILL_IDEA => ".agents/skills/idea/SKILL.md",
        SKILL_FORGE => ".agents/skills/forge/SKILL.md",
        SKILL_APPLY => ".agents/skills/apply/SKILL.md",
        SKILL_RETRO => ".agents/skills/retro/SKILL.md",
        CONFIG_TOML => ".codex/config.toml",
        other => panic!("templates::target_path_for: unknown asset identity `{other}`"),
    }
}

/// Rendered content for a given Asset identity. Pure string generation —
/// no filesystem I/O, no `core/**` read (Decision 1).
pub fn content_for(identity: &str) -> String {
    match identity {
        AGENTS_MD => agents_md(),
        AGENT_ARCHITECT => agent_architect_toml(),
        AGENT_WORKER => agent_worker_toml(),
        AGENT_ADVISORY_WATCH => agent_advisory_watch_toml(),
        AGENT_BOUNDARY_CHECK => agent_boundary_check_toml(),
        SKILL_IDEA => skill_md(
            "idea",
            "Chủ nhà skill — capture a new idea/request, classify it, append to docs/BACKLOG.md in the right section.",
        ),
        SKILL_FORGE => skill_md(
            "forge",
            "Kiến trúc sư mode — write a new recipe when a library pattern is missing, or update an outdated one, into recipes/<category>/<name>.md.",
        ),
        SKILL_APPLY => skill_md(
            "apply",
            "Thợ mode — apply one recipe from the recipes/ library into the current project via a sub-phiếu.",
        ),
        SKILL_RETRO => skill_md(
            "retro",
            "Weekly engineering retrospective — shipping velocity, code quality, patterns.",
        ),
        CONFIG_TOML => config_toml(),
        other => panic!("templates::content_for: unknown asset identity `{other}`"),
    }
}

fn agents_md() -> String {
    r#"# AGENTS.md — SOS Kit orchestrator contract (Codex-native)

<!-- SOS-ADAPTER-PROVENANCE: role semantics canonical -> core/ROLES.md#orchestrator, core/WORKFLOW.md, core/POLICY.md, core/STATE.md; adapter = Codex CLI (adapters/codex/MAPPING.md). Physical render -> P078b2. -->

You are the orchestrator (Quan doc) for this SOS Kit project, running on Codex CLI.

Load, in order: `SOS.md`, `core/ROLES.md`, `core/WORKFLOW.md`, `core/POLICY.md`, `core/STATE.md`.

Hard rules (see `core/WORKFLOW.md`, `core/POLICY.md` for the canonical text -- this file is a pointer, not a copy):
- The main thread is the orchestrator. It MUST NOT implement the active ticket itself -- spawn the `architect` and `worker` subagents per phase (`.codex/agents/architect.toml`, `.codex/agents/worker.toml`).
- No EXECUTE phase begins before the exact ticket version has been approved (`core/WORKFLOW.md` approval gate).
- Spawn `advisory-watch` / `boundary-check` subagents for their scoped read-only checks (`.codex/agents/advisory-watch.toml`, `.codex/agents/boundary-check.toml`).

Full role contract: `core/ROLES.md#orchestrator`.
"#
    .to_string()
}

fn agent_architect_toml() -> String {
    r#"name = "architect"
description = "SOS Kit Kien truc su (architect) -- docs-only ticket author. See core/ROLES.md#architect."
developer_instructions = """
Role contract: core/ROLES.md#architect (canonical semantics -- do not duplicate here).

PARTIAL envelope: Codex CLI has no per-role built-in tool allowlist (unlike
Claude's `tools: Read,Write,Glob`). This file cannot structurally remove
Bash/Grep/Edit from the architect role. The "docs-only, no code read/write"
envelope is enforced via a PreToolUse hook (P078b3) plus this prose --
NOT a structural tool-removal like Claude's. See adapters/codex/CAPABILITY.md
gap #1 and this adapter's verify() Finding for the machine-readable version
of this same declaration.
"""
sandbox_mode = "workspace-write"
"#
    .to_string()
}

fn agent_worker_toml() -> String {
    r#"name = "worker"
description = "SOS Kit Tho (worker) -- full code execution. See core/ROLES.md#worker."
developer_instructions = """
Role contract: core/ROLES.md#worker (canonical semantics -- do not duplicate here).

Codex CLI has no per-role built-in tool allowlist (adapters/codex/CAPABILITY.md
gap #1) -- worker has full code-execution access by design, so this absence
does not narrow the worker envelope, but the same absence applies
symmetrically across every Codex-rendered role.
"""
sandbox_mode = "workspace-write"
"#
    .to_string()
}

fn agent_advisory_watch_toml() -> String {
    r#"name = "advisory-watch"
description = "SOS Kit Trinh sat (advisory-watch) -- read-only security-advisory scout. See core/ROLES.md#advisory_watch."
developer_instructions = """
Role contract: core/ROLES.md#advisory_watch (canonical semantics -- do not
duplicate here).

Honest structural envelope: sandbox_mode = "read-only" below matches this
role's read-only, no-filesystem-write contract structurally (unlike
architect/worker above, no PARTIAL marker is needed for the filesystem
dimension -- read-only IS enforced by the sandbox, not just prose).
"""
sandbox_mode = "read-only"
# ESCAPE HATCH (P078b2 anchor #10, unresolved -- flag PARTIAL, do not claim
# SOUND): "read-only" above is a filesystem/exec sandbox setting. This
# adapter's discovery report (docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md)
# does not document whether Codex CLI's sandbox_mode also gates network
# egress. advisory-watch needs outbound network to query GHSA advisories.
# Rendered here as read-only (honest for the filesystem contract); whether
# this setting additionally blocks the network call this role needs is
# UNCONFIRMED -- must be verified against a real Codex CLI instance
# (behavioral, P079). If network is blocked, an additional
# approval_policy/config.toml allow may be required.
"#
    .to_string()
}

fn agent_boundary_check_toml() -> String {
    r#"name = "boundary-check"
description = "SOS Kit Giam sat (boundary-check) -- read-only invariant checker. See core/ROLES.md#boundary_check."
developer_instructions = """
Role contract: core/ROLES.md#boundary_check (canonical semantics -- do not
duplicate here).

Honest structural envelope: sandbox_mode = "read-only" below matches this
role's read-only, local-only (git+grep) contract structurally -- no network
dependency, no PARTIAL marker needed.
"""
sandbox_mode = "read-only"
"#
    .to_string()
}

fn skill_md(name: &str, description: &str) -> String {
    format!(
        r#"---
name: {name}
description: {description}
---

<!-- SOS-ADAPTER-PROVENANCE: canonical semantics -> core skill contract (see core/WORKFLOW.md); adapter = Codex CLI (adapters/codex/MAPPING.md). Physical render -> P078b2. -->

See `core/WORKFLOW.md` for this skill's full semantic contract. This file is
a Codex-native pointer only -- body semantics live in core, not duplicated
here.
"#
    )
}

fn config_toml() -> String {
    r#"[mcp_servers.doctor]
# Pointer: core/ASSETS.md (MCP registration), core/POLICY.md (authority/scope).
# PATH-relative command -- never a per-machine absolute path (mirrors root .mcp.json).
command = "doctor"
args = ["serve"]
enabled_tools = ["*"]

[agents]
enabled = true

sandbox_mode = "workspace-write"
approval_policy = "on-request"
"#
    .to_string()
}
