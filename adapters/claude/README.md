# Claude adapter boundary

> Status (P076): **declarative boundary.** No file has moved. This doc + `MAPPING.md` declare ownership; physical extraction/render is P077.

## What this adapter owns

Per `core/ASSETS.md` "Adapter-owned asset classes" (lines 40-51), the Claude integration owns only **serialized representations**:

- host entry instructions (`.claude/settings.json`, `.claude/commands/**`);
- agent and skill registration records (`.claude/agents/**`, `.claude/skills/**` symlinks);
- lifecycle event bindings (hook wiring inside `.claude/settings.json`);
- tool and capability maps (`tools:`/`model:` frontmatter on `agents/*.md`; `caller:` frontmatter on `skills/*/SKILL.md`);
- permission configuration (`templates/claude-settings.local.json`);
- optional protocol-server registration (`.mcp.json`).

An adapter-owned artifact must identify the portable source or policy it represents (`core/ASSETS.md` line 51) — it must not become the only copy of a semantic rule.

## Dependency direction

**Adapter → core, one-way.** The adapter references `core/ROLES.md`, `core/POLICY.md`, `core/WORKFLOW.md` as semantic source of truth. Core does not import or name any adapter (`core/README.md` lines 12-16). No reference of the form `core/** → adapters/**` may exist (see Regression check in `docs/ticket/P076-claude-adapter-parity.md`).

## Transition note (P076)

The boundary declared here is **declarative, not physical**:

- Every Claude artifact keeps its current path (`.claude/**`, `agents/*.md`, `skills/*/SKILL.md`, `templates/claude-settings.local.json`, `.mcp.json`).
- Each artifact is mapped to a stable core source ID in `MAPPING.md` — this is the "owner duy nhất" in declared (not moved) form.
- `agents/*.md` and `skills/*/SKILL.md` bodies carry an inert HTML-comment provenance marker pointing at their `core/ROLES.md#<role_id>` (or core skill semantics) — frontmatter fields read by host tooling (`name`/`model`/`tools`/`caller`) are unchanged.
- Physical file move and rendering (`sos-adapter-claude::render()`) are P077's responsibility (`PORTABILITY_ARCHITECTURE.md` lines 32, 144).

## Semantic source of truth

- `SOS.md` — top-level portable contract.
- `core/README.md` — core module index.
- `core/ROLES.md`, `core/POLICY.md`, `core/WORKFLOW.md`, `core/ASSETS.md` — role, policy, workflow, and asset-ownership semantics this adapter serializes for Claude Code.

See `MAPPING.md` in this directory for the artifact-level mapping table.
