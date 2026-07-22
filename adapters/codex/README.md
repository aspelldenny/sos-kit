# Codex adapter boundary

> Status (P078b1): **foundation.** `crates/sos-adapter-codex` exists — `detect()` and `verify()`
> are live (structural). No Codex-native artifact has been rendered yet (`AGENTS.md`, `.codex/**`,
> `.agents/skills/**`) — physical rendering is P078b2/b3. Mirrors `adapters/claude/` shape
> (declarative-first pattern, P076).

## What this adapter owns

Per `core/ASSETS.md` "Adapter-owned asset classes" (lines 40-51), the Codex integration owns only
**serialized representations** — the same semantic source of truth as the Claude adapter, rendered
into Codex-native form:

- orchestrator contract entry (`AGENTS.md` — root, precedence global→repo→subtree, concatenated,
  32KiB limit, rebuilt per session);
- named subagent registration (`.codex/agents/<role>.toml` — name/description/
  developer_instructions + overrides);
- skill registration (`.agents/skills/<name>/SKILL.md` — frontmatter name+description only are
  mechanical);
- runtime configuration (`.codex/config.toml` — `[mcp_servers]`, `[agents]`, `sandbox_mode`,
  `approval_policy`);
- lifecycle event bindings (`.codex/hooks.json` — SessionStart/SubagentStart/SubagentStop/
  PreToolUse/PermissionRequest/PostToolUse/PreCompact/PostCompact/UserPromptSubmit/Stop);
- exec policy rules (`.codex/rules/<name>.rules` — Starlark `prefix_rule`, most-restrictive wins);
- enforcement scripts (`scripts/codex/*` — rewritten guards, since Codex uses `apply_patch`
  (`tool_name="apply_patch"`, patch in `tool_input.command`), NOT Claude's `file_path`-shaped
  tool calls; Codex reads happen via shell, not Read/Glob).

An adapter-owned artifact must identify the portable source or policy it represents
(`core/ASSETS.md` line 51) — it must not become the only copy of a semantic rule.

## Capability gaps (Codex CLI 0.145.0)

See `CAPABILITY.md` in this directory for the frozen, human-readable declaration of the 5 known
Codex capability gaps (per `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:15-22`). The
machine source of truth for these gaps is `CodexAdapter::verify()`
(`crates/sos-adapter-codex/src/lib.rs`) — `CAPABILITY.md` is seeded from it, per
`core/ROLES.md` separation-invariant #5: capability absence must be explicit, an integration
cannot simulate success with prose.

## Dependency direction

**Adapter → core, one-way.** The adapter references `core/ROLES.md`, `core/POLICY.md`,
`core/WORKFLOW.md` as semantic source of truth. Core does not import or name any adapter
(`core/README.md` lines 12-16; enforced by `crates/sos-core/tests/dep_direction.rs`). No
reference of the form `core/** → adapters/**` may exist.

## Foundation note (P078b1)

The boundary declared here has a **live but minimal** code counterpart:

- `crates/sos-adapter-codex/src/lib.rs` — `CodexAdapter` implements the core `Adapter` trait.
  `detect()` is structural (static Codex 0.145.0 facts + a fail-safe `codex --version` probe —
  absence of the binary never panics). `verify()` reports the 5 known capability gaps below with
  an explicit `FindingStatus` (`Sound`/`Partial`/`Missing`, `crates/sos-core/src/adapter.rs`) —
  never `Sound` for a gap.
- `plan()` / `render()` / `uninstall()` are **minimal-honest stubs** — no artifact bytes are
  produced yet. Real rendering into `AGENTS.md`/`.codex/**`/`.agents/skills/**` is P078b2; the
  hook/rule/guard-script enforcement layer is P078b3.
- `sos install --runtime codex` is wired into the composition root
  (`crates/sos-cli/src/commands/install.rs`) and no longer errors "not yet available" — it drives
  the same install ENGINE (`sos-install::engine`) that `--runtime claude` uses, purely through the
  `Adapter` trait (zero engine change was required).
- Oracle boundary (Decision 5, `docs/ticket/P078b1-codex-adapter-foundation.md`): **structural
  only** — this foundation is verified without Codex CLI installed. Whether Codex actually
  executes the rendered artifacts correctly is **behavioral**, deferred to P079.

## Semantic source of truth

- `SOS.md` — top-level portable contract.
- `core/README.md` — core module index.
- `core/ROLES.md`, `core/POLICY.md`, `core/WORKFLOW.md`, `core/ASSETS.md` — role, policy,
  workflow, and asset-ownership semantics this adapter serializes for Codex CLI.

See `MAPPING.md` in this directory for the artifact-level mapping table (seeded shape — most rows
fill in P078b2/b3) and `CAPABILITY.md` for the frozen 5-gap declaration.
