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
//! `all_assets()` is the fixed 10+7+1-Asset set `plan()` enumerates (18
//! total after P078d2a's state-skeleton addition).
//!
//! P078b3 EXTENDS this with 7 **enforcement** artifacts (`HOOKS_JSON`,
//! `RULES_EXEC_POLICY`, 5× `GUARD_*`) — `docs/ticket/P078b3-codex-enforcement.md`
//! Context table E1-E7. Guard content is Codex-only (`apply_patch`, `.codex`,
//! `tool_input`, `permissionDecision`) and lives ONLY in this crate
//! (dep-direction — `core/**` carries zero host token). Guard scripts are a
//! REWRITE, not a copy, of the Claude equivalents (`scripts/{architect,
//! orchestrator,block-env-edit,idea-smell}-guard.sh`) — Codex's payload shape
//! (`apply_patch` with the patch body in `tool_input.command`, no `file_path`;
//! shell-based reads instead of Read/Glob) requires different parsing, and the
//! fail-safe default is INVERTED: Claude guards fail-OPEN on unparseable input
//! ("don't block on weird input"); Codex apply_patch guards fail-CLOSED (an
//! unparsed patch could silently write anywhere — Decision 1,
//! `docs/ticket/P078b3-codex-enforcement.md`).

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

/// 7 P078b3 enforcement Asset identities (E1-E7, Context table).
pub const HOOKS_JSON: &str = "codex_hooks_json";
pub const GUARD_ARCHITECT: &str = "codex_guard_architect";
pub const GUARD_ORCHESTRATOR: &str = "codex_guard_orchestrator";
pub const GUARD_BLOCK_ENV: &str = "codex_guard_block_env";
pub const GUARD_APPROVAL_GATE: &str = "codex_guard_approval_gate";
pub const GUARD_IDEA_SMELL: &str = "codex_guard_idea_smell";
pub const RULES_EXEC_POLICY: &str = "codex_rules_exec_policy";

/// P078d2a #5 (approval bootstrap): a rendered SKELETON of the
/// approval-gate's state file. Reuses the install engine's ALREADY-generic
/// checksum/manifest non-clobber logic (`sos-install::engine`, `Decision`
/// enum) -- rendering this once at install time gives fresh installs a
/// file to read (fixing the fail-CLOSED chicken-egg deadlock, `:634` in
/// `guard_approval_gate_sh`); the engine's existing Conflict-on-user-edit
/// rule (content hash differs from both desired AND recorded manifest
/// hash -> never overwrite, stage instead) means once a real
/// `approved_version=` is written by the workflow, re-running install can
/// never clobber it. No engine change needed -- purely additive via this
/// crate's existing render surface.
pub const STATE_SKELETON: &str = "codex_state_skeleton";

/// Fixed 10+7+1-Asset set (18 total, P078d2a adds the state skeleton).
/// `content` is unused by `render()` (identity alone
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
        // P078b3 enforcement artifacts (E1-E7):
        HOOKS_JSON,
        GUARD_ARCHITECT,
        GUARD_ORCHESTRATOR,
        GUARD_BLOCK_ENV,
        GUARD_APPROVAL_GATE,
        GUARD_IDEA_SMELL,
        RULES_EXEC_POLICY,
        // P078d2a #5: state-file skeleton (bootstrap fix).
        STATE_SKELETON,
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
        HOOKS_JSON => ".codex/hooks.json",
        GUARD_ARCHITECT => "scripts/codex/architect-guard.sh",
        GUARD_ORCHESTRATOR => "scripts/codex/orchestrator-guard.sh",
        GUARD_BLOCK_ENV => "scripts/codex/block-env-edit.sh",
        GUARD_APPROVAL_GATE => "scripts/codex/approval-gate.sh",
        GUARD_IDEA_SMELL => "scripts/codex/idea-smell.sh",
        RULES_EXEC_POLICY => ".codex/rules/exec-policy.rules",
        STATE_SKELETON => ".sos-state/ticket-state.env",
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
        HOOKS_JSON => hooks_json(),
        GUARD_ARCHITECT => guard_architect_sh(),
        GUARD_ORCHESTRATOR => guard_orchestrator_sh(),
        GUARD_BLOCK_ENV => guard_block_env_sh(),
        GUARD_APPROVAL_GATE => guard_approval_gate_sh(),
        GUARD_IDEA_SMELL => guard_idea_smell_sh(),
        RULES_EXEC_POLICY => rules_exec_policy(),
        STATE_SKELETON => state_skeleton_env(),
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
- **Spawn caveat (P079 #7, observed on first custom-agent spawn):** a full-history forked agent inherits the PARENT's agent type, breaking the intended role. When spawning `architect`/`worker`/`advisory-watch`/`boundary-check`, either omit `agent_type` on the spawn call or spawn WITHOUT a full-history fork -- do not fork-with-full-history while also setting `agent_type` to the child role.
- **In-subagent enforcement gap (P078d2b, upstream `openai/codex#21753`):** on Codex 0.145.0, role-envelope guards do NOT enforce inside spawned custom-role subagents (their lifecycle/PreToolUse hooks never fire for `architect`/`worker`). The orchestrator (main thread) is therefore responsible for boundary review of every subagent-authored diff before merge -- the Git pre-commit/pre-push gate is the real backstop, not an in-subagent guard. Optionally the orchestrator may `touch .sos-state/<role>-active` before delegating and `rm` it after (best-effort, main-thread only -- this does NOT enforce anything inside the spawned agent itself).

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
    // P078d1 fix #1: TOML table-scope. A bare `key = value` line binds to the
    // MOST RECENT [table] header above it (root, if none yet). The prior
    // render put sandbox_mode/approval_policy AFTER [agents] -- Codex 0.145.0
    // then deserialized them as members of AgentRoleToml (err: "invalid type
    // string \"workspace-write\", expected struct AgentRoleToml"). Root-level
    // settings MUST render before the FIRST table header in the file
    // ([mcp_servers.doctor] is first, not [agents] -- placing root keys
    // merely before [agents] would still nest them under
    // [mcp_servers.doctor]). See docs/discoveries/P078d1.md.
    r#"sandbox_mode = "workspace-write"
approval_policy = "on-request"

[mcp_servers.doctor]
# Pointer: core/ASSETS.md (MCP registration), core/POLICY.md (authority/scope).
# PATH-relative command -- never a per-machine absolute path (mirrors root .mcp.json).
command = "doctor"
args = ["serve"]
enabled_tools = ["*"]

[agents]
enabled = true
"#
    .to_string()
}

// ── P078b3 enforcement artifacts (E1-E7) ────────────────────────────────
//
// Ground-truth apply_patch/Bash PreToolUse payload shape (confirmed live
// against Codex CLI gpt-5.6, P078b3 Debate Log Turn 2 -- fixture committed
// at `crates/sos-adapter-codex/tests/fixtures/codex-apply-patch-payloads.jsonl`):
//   {"hook_event_name":"PreToolUse","permission_mode":"...","cwd":"...",
//    "tool_name":"apply_patch","tool_input":{"command":"<patch>"},"tool_use_id":"..."}
// `tool_input.command` for apply_patch is the V4A patch text with embedded
// newlines escaped as literal `\n` in the JSON string:
//   *** Begin Patch\n*** Add File: <path>\n+...\n*** End Patch
//   *** Begin Patch\n*** Update File: <path>\n@@\n-...\n+...\n*** End Patch
//   *** Begin Patch\n*** Delete File: <path>\n*** End Patch
// (defensive: also match `*** Move File:` -- not in the sample, but part of
// the wider V4A spec if Codex ever emits a rename.)
// For `tool_name="Bash"`, `tool_input.command` is a plain shell string.

fn hooks_json() -> String {
    // P078d1 fix #3: Codex 0.145.0 hooks.json top-level schema only accepts
    // `description` / `hooks` (err: "unknown field _provenance, expected
    // description or hooks"). The former `_provenance` + `_partial_note`
    // custom fields are folded into the single top-level `description`
    // string below so the provenance/PARTIAL trace is not lost.
    //
    // P078d2b DEPRECATED-BEST-EFFORT (SubagentStart/SubagentStop below):
    // Codex 0.145.0 does NOT dispatch these hooks for CUSTOM-role subagents
    // (architect/worker) -- only `agent_type="default"` fires (upstream
    // `openai/codex#21753`; probe:
    // docs/adapters/SUBAGENT-HOOK-PROBE-2026-07-22.md; dogfood-confirmed:
    // docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md#4 -- the
    // architect-active/worker-active marker was NEVER created and the
    // forbidden apply_patch succeeded in-subagent). This marker fires for
    // DEFAULT subagents only and is NOT relied upon for any capability
    // claim -- see `CodexAdapter::verify()` Finding #6 (Missing) and
    // `adapters/codex/CAPABILITY.md` §6. Retained (not removed) so it
    // activates automatically if upstream #21753 is ever fixed; retaining
    // it also keeps DEFAULT-subagent marker behavior working today.
    r#"{
  "description": "Provenance: core/POLICY.md (enforcement), core/WORKFLOW.md (state gates) -- P078b3. PARTIAL: Enforcement is bypassable: project hooks only run for TRUSTED repos, non-managed hooks need /hooks trust, and users can disable hooks entirely. Git/CI review-trigger backstops (core/POLICY.md) MUST be retained -- this hook layer is fast-feedback, not the security boundary.",
  "hooks": {
    "SessionStart": [
      { "hooks": [ { "type": "command", "command": "bash scripts/codex/idea-smell.sh" } ] }
    ],
    "UserPromptSubmit": [
      { "hooks": [ { "type": "command", "command": "bash scripts/codex/idea-smell.sh" } ] }
    ],
    "SubagentStart": [
      { "matcher": "architect", "hooks": [ { "type": "command", "command": "mkdir -p .sos-state && touch .sos-state/architect-active" } ] },
      { "matcher": "worker", "hooks": [ { "type": "command", "command": "mkdir -p .sos-state && touch .sos-state/worker-active" } ] }
    ],
    "SubagentStop": [
      { "matcher": "architect", "hooks": [ { "type": "command", "command": "rm -f .sos-state/architect-active" } ] },
      { "matcher": "worker", "hooks": [ { "type": "command", "command": "rm -f .sos-state/worker-active" } ] }
    ],
    "PreToolUse": [
      {
        "matcher": "apply_patch",
        "hooks": [
          { "type": "command", "command": "bash scripts/codex/architect-guard.sh" },
          { "type": "command", "command": "bash scripts/codex/orchestrator-guard.sh" },
          { "type": "command", "command": "bash scripts/codex/block-env-edit.sh" },
          { "type": "command", "command": "bash scripts/codex/approval-gate.sh" }
        ]
      },
      {
        "matcher": "Bash",
        "hooks": [
          { "type": "command", "command": "bash scripts/codex/architect-guard.sh" }
        ]
      }
    ],
    "Stop": []
  }
}
"#
    .to_string()
}

fn guard_architect_sh() -> String {
    r#"#!/usr/bin/env bash
# scripts/codex/architect-guard.sh — Codex PreToolUse hook: enforce the Architect envelope.
#
# REWRITE (not a copy) of scripts/architect-guard.sh (Claude), for Codex's payload shape
# (P078b3): Codex writes via apply_patch — {"tool_name":"apply_patch",
# "tool_input":{"command":"<V4A patch>"}} — NOT file_path. Codex reads via shell
# (Bash tool_input.command), NOT Read/Glob — read-restriction inspects command TEXT
# (heuristic, PARTIAL — adapters/codex/CAPABILITY.md gap #5).
#
# fail-CLOSED (Decision 1, P078b3 — INVERTED from Claude's fail-open precedent): while
# architect-active AND tool is apply_patch AND no safe path can be extracted -> BLOCK.
# Claude's guard allows on unparseable input ("don't block on weird input"); for Codex
# apply_patch that would be a silent bypass, so this guard inverts the default.
#
# PARTIAL (honest, adapters/codex/CAPABILITY.md gap #1/#5): bypassable if the repo is
# untrusted, hooks are user-disabled, or the shell command is obfuscated — Git/CI review
# on architect-authored diffs (core/POLICY.md review-trigger map) is the real boundary.
#
# Pointer: core/ROLES.md#architect, core/POLICY.md (edit-and-verify). No jq — pure
# sed/grep for cross-platform parity with the Claude guards.

set -euo pipefail

INPUT_JSON=$(cat)

# cwd binding: Codex payload carries "cwd" (ground-truth sample, P078b3 — no
# $CLAUDE_PROJECT_DIR equivalent confirmed). Fall back to git rev-parse, then $PWD.
# Bind to the repo root deterministically (git-root, else current dir).
# NOTE: we deliberately do NOT cd into the payload's "cwd" field -- Codex
# already launches the hook subprocess with its real working directory, and
# forcing a cd there is unconfirmed/unnecessary behavior this ticket does
# not need to assume (P078b3 mock-payload testing surfaced this: a stale
# "cwd" value from an unrelated captured sample would silently relocate the
# guard away from the project it's meant to protect).
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$REPO_ROOT" 2>/dev/null || true

MARKER_FILE=".sos-state/architect-active"
[ -f "$MARKER_FILE" ] || exit 0

TOOL_NAME=$(printf '%s' "$INPUT_JSON" | sed -n 's/.*"tool_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

case "$TOOL_NAME" in
    apply_patch)
        # Extract EVERY *** Add/Update/Delete/Move File: <path> marker from the raw
        # JSON line (P078d2a #6 fix -- was FIRST-path-only via `head -n1`, a
        # SECURITY HOLE: a multi-file apply_patch could put a ticket-allowed path
        # first and a forbidden path second, and the old code exited ALLOW on the
        # first match without ever looking at the rest). tool_input.command's
        # embedded newlines are literal backslash-n (not real newlines) in the
        # hook's stdin JSON -- [^\\"]+ stops at the escape or the closing quote,
        # giving a clean path per match (ground-truth confirmed, P078b3 Turn 2).
        # `grep -oE` already emits ONE OUTPUT LINE PER MATCH even though every
        # match sits on the same single JSON input line -- removing `head -n1`
        # alone is sufficient to get every path; no new regex is needed.
        # NOTE (open, [needs Worker verify], P079 #6 spec): `*** Move File: <src>`
        # is paired with a separate `*** Move to: <dest>` line in the wider V4A
        # spec, but this was not present in the P078b3 ground-truth sample --
        # only the Move source path is parsed here; the dest line is NOT matched
        # by this regex and is therefore untouched by this guard. Documented gap,
        # not guessed at (Luật chơi #6 -- do not invent unconfirmed format).
        PATCH_PATHS=$(printf '%s' "$INPUT_JSON" \
            | grep -oE '\*\*\* (Add|Update|Delete|Move) File: [^\\"]+' \
            | sed -E 's/^\*\*\* (Add|Update|Delete|Move) File: //' || true)

        if [ -z "$PATCH_PATHS" ]; then
            cat >&2 <<'EOF'
BLOCKED: architect-guard could not extract a safe file path from this apply_patch
payload (fail-CLOSED, P078b3 Decision 1). Unlike the Claude guard (which allows on
unparseable input), the Codex guard BLOCKS -- an unparsed patch could silently write
anywhere. If this is a legitimate patch, check the extraction regex in
scripts/codex/architect-guard.sh against the real payload shape.
EOF
            exit 2
        fi

        # block-if-ANY (P078d2a #6): populate the array via a HERESTRING
        # `while read` (bash 3.2 compatible -- no `readarray`/`mapfile`,
        # macOS ships bash 3.2), then a plain `for` over the array -- NEVER
        # `... | while read; do ...; exit 2; done` (a PIPED while runs in a
        # subshell; `exit 2` there would only exit the subshell, silently
        # keeping the security hole open while still printing "BLOCKED" to
        # stderr). A herestring (`<<<`) does not create a subshell here.
        ALL_PATHS=()
        while IFS= read -r LINE; do
            ALL_PATHS+=("$LINE")
        done <<< "$PATCH_PATHS"
        for RAW_PATH in "${ALL_PATHS[@]}"; do
            [ -z "$RAW_PATH" ] && continue
            NORMALIZED_PATH="${RAW_PATH#./}"
            case "$(basename "$NORMALIZED_PATH")" in
                P[0-9]*-*.md) continue ;;   # phiếu file -> allow this one path
            esac
            cat >&2 <<EOF
BLOCKED: Architect envelope violation (apply_patch write).

Architect may ONLY write phiếu files (P<NNN>-<slug>.md): $NORMALIZED_PATH is outside
the envelope. Write a Task 0 anchor instead -- a spawned Worker verifies it.
EOF
            exit 2
        done
        exit 0
        ;;
    Bash)
        # Heuristic (PARTIAL, gap #5): pattern-match the RAW tool_input.command text
        # (embedded inside INPUT_JSON) for read-tool names + a source-ish path. We
        # deliberately do NOT try to cleanly extract an unescaped "command" value --
        # Bash commands routinely embed escaped quotes (ground-truth sample line 4:
        # `sed -n \"1p\" read-target.txt`), and a greedy sed capture on that is
        # unreliable. Substring-matching the raw JSON text is simpler AND sufficient
        # for a heuristic gate -- both sides of the check only need presence/absence,
        # not a clean variable.
        BLOCKED=0
        case "$INPUT_JSON" in
            *rg*|*sed*|*cat*|*head*|*tail*|*less*|*awk*)
                case "$INPUT_JSON" in
                    *src/*|*crates/*src*|*.rs*|*docs/CHARACTER*|*docs/SOUL.md*|*docs/PROJECT.md*)
                        BLOCKED=1 ;;
                esac
                ;;
        esac
        if [ "$BLOCKED" = "1" ]; then
            cat >&2 <<EOF
BLOCKED: Architect envelope violation (shell read).

This shell command appears to read source code. Write a Task 0 anchor in the phiếu
instead -- a spawned Worker grep-verifies it. (Heuristic, PARTIAL, gap #5 --
pattern-matches shell command text, not a structural interception point; a
sufficiently obfuscated command could evade it.)
EOF
            exit 2
        fi
        exit 0
        ;;
    *)
        exit 0
        ;;
esac
"#
    .to_string()
}

fn guard_orchestrator_sh() -> String {
    r#"#!/usr/bin/env bash
# scripts/codex/orchestrator-guard.sh — Codex PreToolUse hook: block product-source
# apply_patch writes when no worker-active marker is present.
#
# REWRITE (not a copy) of scripts/orchestrator-guard.sh (Claude) for Codex's
# apply_patch payload shape (P078b3) -- see scripts/codex/architect-guard.sh header
# for the payload-shape note.
#
# fail-CLOSED (Decision 1): apply_patch + unparseable path -> BLOCK (Codex inverts
# the Claude fail-open precedent for write-guards).
#
# Pointer: core/ROLES.md#orchestrator, core/POLICY.md.

set -euo pipefail

INPUT_JSON=$(cat)

# Bind to the repo root deterministically (git-root, else current dir).
# NOTE: we deliberately do NOT cd into the payload's "cwd" field -- Codex
# already launches the hook subprocess with its real working directory, and
# forcing a cd there is unconfirmed/unnecessary behavior this ticket does
# not need to assume (P078b3 mock-payload testing surfaced this: a stale
# "cwd" value from an unrelated captured sample would silently relocate the
# guard away from the project it's meant to protect).
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$REPO_ROOT" 2>/dev/null || true

TOOL_NAME=$(printf '%s' "$INPUT_JSON" | sed -n 's/.*"tool_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
[ "$TOOL_NAME" = "apply_patch" ] || exit 0

# Extract EVERY path (P078d2a #6 fix -- see architect-guard.sh comment for the
# full rationale: `head -n1` was a SECURITY HOLE letting a second, product-source
# path ride through behind a first docs/kit-tooling path).
PATCH_PATHS=$(printf '%s' "$INPUT_JSON" \
    | grep -oE '\*\*\* (Add|Update|Delete|Move) File: [^\\"]+' \
    | sed -E 's/^\*\*\* (Add|Update|Delete|Move) File: //' || true)

if [ -z "$PATCH_PATHS" ]; then
    echo "BLOCKED: orchestrator-guard could not parse apply_patch path -- fail-CLOSED (P078b3 Decision 1)." >&2
    exit 2
fi

# block-if-ANY (P078d2a #6): populate the array via a HERESTRING `while read`
# (bash 3.2 compatible -- no `readarray`/`mapfile`), then a plain `for` over
# the array -- NEVER `... | while read; do ...; exit 2; done` (a PIPED while
# runs in a subshell; `exit 2` there would silently keep the hole open).
ALL_PATHS=()
while IFS= read -r LINE; do
    ALL_PATHS+=("$LINE")
done <<< "$PATCH_PATHS"
for RAW_PATH in "${ALL_PATHS[@]}"; do
    [ -z "$RAW_PATH" ] && continue
    NORMALIZED_PATH="${RAW_PATH#./}"

    # Docs are never product source (mirror Claude guard).
    case "$NORMALIZED_PATH" in
        *.md) continue ;;
    esac

    # The kit's own bundled tooling is kit-maintenance, not the product it ships.
    case "$NORMALIZED_PATH" in
        bootstrap/*|crates/*|Cargo.toml|Cargo.lock) continue ;;
    esac

    # Is this product source? Anything else -> allow this one path.
    case "$NORMALIZED_PATH" in
        *.swift|*.pbxproj|src/*|*/src/*) ;;
        *) continue ;;
    esac

    [ -f ".sos-state/worker-active" ] && continue

    cat >&2 <<EOF
BLOCKED: Orchestrator envelope violation (apply_patch).

Product source $NORMALIZED_PATH is being written without a worker-active marker.
Set .sos-state/worker-active before spawning the worker subagent (rm -f it after).
EOF
    exit 2
done
exit 0
"#
    .to_string()
}

fn guard_block_env_sh() -> String {
    r#"#!/usr/bin/env bash
# scripts/codex/block-env-edit.sh — Codex PreToolUse hook: block apply_patch touching
# .env* (except .env.example).
#
# REWRITE (not a copy) of scripts/block-env-edit.sh (Claude) for the apply_patch
# payload shape (P078b3). fail-CLOSED: an unparseable apply_patch path while this
# guard fires -> BLOCK (secret-leak guard, WORKFLOW_V2.2.md Sub-mech F).

set -euo pipefail

INPUT_JSON=$(cat)

TOOL_NAME=$(printf '%s' "$INPUT_JSON" | sed -n 's/.*"tool_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
[ "$TOOL_NAME" = "apply_patch" ] || exit 0

# Extract EVERY path (P078d2a #6 fix -- see architect-guard.sh comment for the
# full rationale: `head -n1` was a SECURITY HOLE letting a second .env-touching
# path ride through behind a first, innocuous-looking path).
PATCH_PATHS=$(printf '%s' "$INPUT_JSON" \
    | grep -oE '\*\*\* (Add|Update|Delete|Move) File: [^\\"]+' \
    | sed -E 's/^\*\*\* (Add|Update|Delete|Move) File: //' || true)

if [ -z "$PATCH_PATHS" ]; then
    echo "BLOCKED: block-env-edit could not parse apply_patch path -- fail-CLOSED (secret guard, P078b3)." >&2
    exit 2
fi

# block-if-ANY (P078d2a #6): populate the array via a HERESTRING `while read`
# (bash 3.2 compatible -- no `readarray`/`mapfile`), then a plain `for` over
# the array -- NEVER `... | while read; do ...; exit 2; done` (a PIPED while
# runs in a subshell; `exit 2` there would silently keep the hole open).
ALL_PATHS=()
while IFS= read -r LINE; do
    ALL_PATHS+=("$LINE")
done <<< "$PATCH_PATHS"
for RAW_PATH in "${ALL_PATHS[@]}"; do
    [ -z "$RAW_PATH" ] && continue
    BASE=$(basename "$RAW_PATH")
    [ "$BASE" = ".env.example" ] && continue

    if echo "$BASE" | grep -qE '^\.env($|\.)'; then
        cat >&2 <<EOF
BLOCKED: apply_patch touching $RAW_PATH.

.env* holds real secrets -- do not edit via the agent tool. Edit .env.example
(template) instead, or edit the real .env by hand outside the agent.
EOF
        exit 2
    fi
done
exit 0
"#
    .to_string()
}

fn guard_approval_gate_sh() -> String {
    r#"#!/usr/bin/env bash
# scripts/codex/approval-gate.sh — Codex PreToolUse hook: block EXECUTE-phase
# apply_patch writes to source until the exact ticket version carries an approval
# record (E5 -- Codex native gap #4, adapters/codex/CAPABILITY.md: Codex's
# approval_policy=on-request approves individual OPERATIONS, not a ticket VERSION).
#
# This guard is the guard-BUILT replacement -- it reads a machine-state projection
# (core/STATE.md fields) and READ-COMPARES ONLY. It never mutates approved_version
# (mutation authority = owner / bounded delegate only, core/STATE.md "Mutation
# authority"; the ticket's Debate Log stays the authoritative record -- this is a
# derived projection, core/STATE.md "Authoritative-vs-projection rule").
#
# State-file layout (Worker Tầng-2 layout decision, P078b3 -- core/STATE.md leaves
# the concrete file/format integration-defined; no prior convention existed in this
# codebase to grep against): `.sos-state/ticket-state.env`, plain key=value lines:
#   version=<ticket version string, e.g. V2>
#   approved_version=<approved version string, empty/absent if none>
#
# fail-CLOSED: state file missing/unparseable + apply_patch on a non-ticket path ->
# BLOCK (treat "cannot prove approval" the same as "not approved"). Ticket files
# themselves (docs/ticket/P<NNN>-*.md) are ALWAYS allowed through this guard --
# drafting/amending the ticket doesn't require its own prior approval.
#
# State-file-alone exemption (P078d2a #5 create-when-missing; P078e Task 1
# extends it to create-OR-update): a patch that touches ONLY
# .sos-state/ticket-state.env (and touches NO other path) is allowed through --
# otherwise there is no way to ever initialize the state file (chicken-egg
# deadlock at fresh install, P078d2a #5) NOR to ever write a legitimate
# approval-transition update afterwards (the V1->V2 deadlock P079 round-2
# found: bootstrap creates version=V1/approved_version=empty, then the very
# FIRST legit approval write -- version=V2, approved_version=V2 -- was BLOCKed
# by the version-match check below, because writing the approval record is
# itself gated on the record already existing). This exemption is SAFE only
# because of TWO layers together:
#
#   (1) actor-check (P078e Task 1, NEW -- mirrors `orchestrator-guard.sh`'s
#       `[ -f ".sos-state/worker-active" ] && continue` pattern): the
#       exemption fires ONLY when NEITHER `.sos-state/worker-active` NOR
#       `.sos-state/architect-active` is present. Those markers are set only
#       while a worker/architect subagent turn is in flight (SubagentStart/
#       Stop hooks, `templates.rs` orchestrator config). Approval authority
#       belongs to the owner/orchestrator (core/STATE.md "Mutation
#       authority") -- the actor-check keeps the exemption from ever firing
#       while the SAME agent under review (worker) is the one holding the
#       apply_patch pen, which would otherwise let it self-approve its own
#       ticket version. If either marker is set, this guard now BLOCKS the
#       write outright (P078h Task 1, gap #3 fix, below) -- a
#       worker/architect-authored write to ticket-state.env gets NO special
#       treatment, and NO fall-through to the version-match check (that
#       fall-through used to let an already-approved version self-advance,
#       e.g. V3/V3 -> V4/V4).
#
#   (2) the #6 all-path check below, which sees every path in the patch: an
#       attacker cannot smuggle a second path (.env, src/**, a forged
#       approved_version alongside a code change) in the same patch as
#       ticket-state.env -- the moment a second path is present, the set is
#       no longer "exactly ticket-state.env alone" and this guard falls
#       through to the normal fail-CLOSED BLOCK. Do NOT loosen this to
#       "contains" semantics -- it must stay "IS EXACTLY the one path."
#
# Codex caveat (BOTH layers are best-effort on Codex, unlike Claude): Codex
# in-subagent PreToolUse enforcement is a known gap (E5/#4, d2b honest-MISSING
# declaration) -- a worker subagent's apply_patch call may not even route
# through this hook in-session on Codex. The actor-check above is correct
# and complete on Claude (subagent hooks fire reliably there); on Codex it is
# defense-in-depth only -- the real backstop for approval-record integrity on
# Codex is human-review-at-the-git-commit-boundary, NOT this in-session gate.
# Do NOT claim full self-approve protection on Codex.

set -euo pipefail

INPUT_JSON=$(cat)

# Bind to the repo root deterministically (git-root, else current dir).
# NOTE: we deliberately do NOT cd into the payload's "cwd" field -- Codex
# already launches the hook subprocess with its real working directory, and
# forcing a cd there is unconfirmed/unnecessary behavior this ticket does
# not need to assume (P078b3 mock-payload testing surfaced this: a stale
# "cwd" value from an unrelated captured sample would silently relocate the
# guard away from the project it's meant to protect).
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$REPO_ROOT" 2>/dev/null || true

TOOL_NAME=$(printf '%s' "$INPUT_JSON" | sed -n 's/.*"tool_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
[ "$TOOL_NAME" = "apply_patch" ] || exit 0

# Extract EVERY path (P078d2a #6 fix -- see architect-guard.sh comment for the
# full rationale). This is the all-path check the #5 bootstrap exemption below
# depends on for its safety.
PATCH_PATHS=$(printf '%s' "$INPUT_JSON" \
    | grep -oE '\*\*\* (Add|Update|Delete|Move) File: [^\\"]+' \
    | sed -E 's/^\*\*\* (Add|Update|Delete|Move) File: //' || true)

if [ -z "$PATCH_PATHS" ]; then
    echo "BLOCKED: approval-gate could not parse apply_patch path -- fail-CLOSED (P078b3 Decision 1)." >&2
    exit 2
fi

ALL_PATHS=()
while IFS= read -r LINE; do
    ALL_PATHS+=("$LINE")
done <<< "$PATCH_PATHS"

# Ticket files are always allowed individually; build the set of paths that
# still need an approval check. If nothing remains, every path in this patch
# was a ticket file -- allow.
#
# Path-normalize (P078h Task 2, gap #4): apply_patch path headers are
# sometimes repo-root-absolute instead of relative (Codex prompt-shape
# dependent, P079 round-3 "Additional usability observation"), which used to
# false-block against the exact-relative-string STATE_FILE compare below.
# Normalize HERE -- the single site both the state-file-allow exemption and
# the multi-path count below consume -- by stripping a literal "$REPO_ROOT/"
# prefix. Conservative on purpose: this is a plain string-prefix strip, NOT a
# realpath/symlink resolution, so a path that is absolute but NOT under
# REPO_ROOT (or that still contains "..") is left exactly as-is, which means
# it can never equal STATE_FILE below and therefore can never win the
# exemption -- an unresolved/foreign-looking path always falls through to
# fail-CLOSED rather than being trusted.
NON_TICKET_PATHS=()
for RAW_PATH in "${ALL_PATHS[@]}"; do
    [ -z "$RAW_PATH" ] && continue
    case "$(basename "$RAW_PATH")" in
        P[0-9]*-*.md) continue ;;   # editing/drafting the ticket itself never needs its own approval
    esac
    NORM_PATH="$RAW_PATH"
    case "$RAW_PATH" in
        "$REPO_ROOT"/*) NORM_PATH="${RAW_PATH#"$REPO_ROOT"/}" ;;
    esac
    case "$NORM_PATH" in
        *..*) NORM_PATH="$RAW_PATH" ;;   # path-trick guard: never trust a normalized form containing ".."
    esac
    NON_TICKET_PATHS+=("$NORM_PATH")
done

[ ${#NON_TICKET_PATHS[@]} -eq 0 ] && exit 0

STATE_FILE=".sos-state/ticket-state.env"

# Actor-check (P078e Task 1, mirror orchestrator-guard.sh's
# `[ -f ".sos-state/worker-active" ] && continue`): the state-file-alone
# exemption below is safe ONLY when neither subagent marker is present --
# i.e. only the orchestrator/main-thread is the one writing.
ACTOR_IS_MAIN_THREAD=1
[ -f ".sos-state/worker-active" ] && ACTOR_IS_MAIN_THREAD=0
[ -f ".sos-state/architect-active" ] && ACTOR_IS_MAIN_THREAD=0

# Marked-actor early-BLOCK (P078h Task 1, gap #3): a marked actor (worker or
# architect subagent marker present) is rejected OUTRIGHT for ANY write that
# touches STATE_FILE -- create, update, or advance -- with NO fall-through to
# the version-match check further below. P079 round-3 §B4 found that merely
# removing the state-file-alone exemption for a marked actor was NOT enough:
# the fall-through version-match check only compares PRE-PATCH state, so an
# already-approved version (e.g. V3/V3) satisfied `APPROVED == VERSION` and
# was allowed to advance to V4/V4 -- a self-approve hole. Version-match must
# never be an escape hatch for a marked actor; only main-thread writes reach
# the checks below.
if [ "$ACTOR_IS_MAIN_THREAD" -eq 0 ]; then
    for CHECK_PATH in "${NON_TICKET_PATHS[@]}"; do
        if [ "$CHECK_PATH" = "$STATE_FILE" ]; then
            echo "BLOCKED: control-plane write by marked actor: blocked (.sos-state/worker-active or .sos-state/architect-active present while $STATE_FILE is touched -- no version-match escape hatch)." >&2
            exit 2
        fi
    done
fi

# State-file-alone exemption (#5 create-when-missing, P078e-extended to
# create+update): allow ONLY if (a) the entire non-ticket path set is
# exactly {.sos-state/ticket-state.env} -- one element, that exact value
# (checked via the #6 all-path list above) -- AND (b) the actor-check
# passed. Fires regardless of whether the state file currently exists
# (bootstrap) or already exists (approval transition, the V1->V2 deadlock
# fix). Any other path present, or either marker set, falls through to the
# normal fail-CLOSED / version-match checks below.
if [ "$ACTOR_IS_MAIN_THREAD" -eq 1 ] \
    && [ ${#NON_TICKET_PATHS[@]} -eq 1 ] \
    && [ "${NON_TICKET_PATHS[0]}" = "$STATE_FILE" ]; then
    exit 0
fi

if [ ! -f "$STATE_FILE" ]; then
    echo "BLOCKED: approval-gate found no $STATE_FILE -- treating as UNAPPROVED, fail-CLOSED." >&2
    exit 2
fi

VERSION=$(sed -n 's/^version=//p' "$STATE_FILE" | head -n1)
APPROVED=$(sed -n 's/^approved_version=//p' "$STATE_FILE" | head -n1)

if [ -z "$APPROVED" ] || [ "$APPROVED" != "$VERSION" ]; then
    cat >&2 <<EOF
BLOCKED: ticket version '$VERSION' is not approved (approved_version='$APPROVED').

apply_patch on ${NON_TICKET_PATHS[0]} is blocked until the exact version carries an
approval record (core/STATE.md approval record; only the owner or a bounded
delegate may create one -- this guard reads, it does not mutate).
EOF
    exit 2
fi
exit 0
"#
    .to_string()
}

fn guard_idea_smell_sh() -> String {
    r#"#!/usr/bin/env bash
# scripts/codex/idea-smell.sh — Codex UserPromptSubmit hook: idea-smell detector.
#
# REWRITE-pointer (same strategy, not a copy) of scripts/idea-smell.sh (Claude) --
# regex-matches the raw stdin payload TEXT for idea-smell phrases, portable because
# it string-matches regardless of the exact field name carrying the user's message.
# Pointer: core/WORKFLOW.md (intake). Tight pattern set on purpose (false-positive =
# noise) -- extend only when a real miss is observed, don't speculate.

INPUT=""
if [ ! -t 0 ]; then INPUT=$(cat || true); fi
[ -z "$INPUT" ] && exit 0

case "$INPUT" in
  *"ghi vào backlog"*|*"thêm vào backlog"*|*"ghi backlog"*|*"anh nghĩ ra"*|*"anh vừa nghĩ"*|*"tự nhiên anh nghĩ"*|*"ý tưởng mới"*|*"new idea"*|*"add to backlog"*|*"log this idea"*)
    echo "Idea-smell in message -- invoke the idea skill (dedup BACKLOG, tag/section, date stamp). Do not write BACKLOG.md directly."
    ;;
esac
exit 0
"#
    .to_string()
}

fn state_skeleton_env() -> String {
    // P078d2a #5 -- rendered ONCE at install time so approval-gate.sh
    // (`:634` STATE_FILE check) has something to read on a fresh install
    // instead of hitting its fail-CLOSED "missing state file" BLOCK
    // immediately. Non-clobber is handled entirely by the pre-existing
    // generic install-engine checksum/manifest logic (`sos-install::engine`
    // `Decision::Conflict`) -- once the workflow writes a real
    // `approved_version=`, this file's on-disk content diverges from both
    // the skeleton AND the manifest-recorded hash, so a re-install
    // Conflicts (stages incoming, never overwrites) rather than clobbering
    // real approval state. This crate makes zero engine changes -- purely
    // additive via the existing render surface.
    r#"# .sos-state/ticket-state.env — SOS Kit ticket approval-state projection.
# Machine-written/read by scripts/codex/approval-gate.sh — DO NOT edit by
# hand except to bootstrap a fresh install (this skeleton IS that
# bootstrap). Authoritative record = the ticket's own Debate Log
# (core/STATE.md "Authoritative-vs-projection rule"); this file is a
# derived, mechanically-read projection only.
#
#   version=<ticket version string, e.g. V2>
#   approved_version=<approved version string, empty/absent if none>
version=
approved_version=
"#
    .to_string()
}

fn rules_exec_policy() -> String {
    r#"# .codex/rules/exec-policy.rules — Starlark exec policy (P078b3 E7).
#
# Pointer: core/POLICY.md (authority/scope, outside-sandbox commands). Governs
# commands OUTSIDE the sandbox, NOT the in-sandbox tool allowlist
# (docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:12). Most-restrictive rule
# wins across matches.
#
# DEFER (Decision 4, P078b3): PR-merge-without-security-APPROVE is a SEMANTIC gate
# (reads a PR sentinel, scripts/block-unsafe-merge.sh) that cannot be rewritten here
# -- its logic lives in an external `claude-hooks` binary parsing Claude's payload
# shape. This file covers only the MECHANICAL force-push/destructive-command class.
# The PR-merge-sentinel gap is retained via the Git/CI backstop (branch protection +
# review requirement on GitHub), noted in adapters/codex/CAPABILITY.md.
#
# PARTIAL (honest): bypassable if the repo is untrusted or the user disables rules --
# Git/CI backstops (branch protection, push-protection) are the real boundary; this
# is fast-feedback only.
#
# P078d1 fix #2: `pattern` MUST be a token LIST, not a bare string -- Codex
# 0.145.0 rejected the prior string form (err: "pattern doesn't match,
# expected list, actual string"). Each argv token is one list element.

prefix_rule(pattern = ["git", "push", "--force"], decision = "forbidden")
prefix_rule(pattern = ["git", "push", "-f"], decision = "forbidden")
prefix_rule(pattern = ["git", "push", "--force-with-lease"], decision = "prompt")
prefix_rule(pattern = ["rm", "-rf", "/"], decision = "forbidden")
prefix_rule(pattern = ["rm", "-rf", "~"], decision = "forbidden")
prefix_rule(pattern = ["git", "reset", "--hard"], decision = "prompt")
"#
    .to_string()
}
