# Codex artifact → core source ID mapping

> P078b2: 10 declarative artifacts (row below marked "P078b2 DONE") are now physically rendered
> by `CodexAdapter::render()`/`plan()` (`crates/sos-adapter-codex/src/templates.rs`) — structural
> oracle only (`docs/ticket/P078b2-codex-render.md` Decision 5); enforcement rows (b3) remain
> declared-only. "Physical render" column tracks the phiếu that produced bytes, not just intent.

| Artifact (future, declared) | Class | Core source ID | Physical render |
|---|---|---|---|
| `AGENTS.md` (root) | ADAPTER_OWNED | orchestrator contract → `core/ROLES.md#orchestrator`, `core/WORKFLOW.md` | P078b2 DONE |
| `.codex/agents/architect.toml` | ADAPTER_OWNED | `core/ROLES.md#architect` (PARTIAL marker in-artifact — envelope enforced via PreToolUse hook P078b3, NOT structural tool-removal) | P078b2 DONE |
| `.codex/agents/worker.toml` | ADAPTER_OWNED | `core/ROLES.md#worker` | P078b2 DONE |
| `.codex/agents/advisory-watch.toml` | ADAPTER_OWNED | `core/ROLES.md#advisory_watch` (`sandbox_mode="read-only"`, honest structural for filesystem; network-egress gating UNCONFIRMED — escape hatch, behavioral verify = P079) | P078b2 DONE |
| `.codex/agents/boundary-check.toml` | ADAPTER_OWNED | `core/ROLES.md#boundary_check` (`sandbox_mode="read-only"`, honest structural) | P078b2 DONE |
| `.agents/skills/idea/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; frontmatter `caller:` = adapter part | P078b2 DONE |
| `.agents/skills/retro/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; `caller:` = adapter part | P078b2 DONE |
| `.agents/skills/init/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; `caller:` = adapter part | **Deferred, NOT rendered b2** — Decision 4 (`docs/ticket/P078b2-codex-render.md`): Claude's `caller` for `init` is `"sos init CLI (bin/sos.sh prints: run skill /init)"` (`skills/init/SKILL.md:3`), a Claude-CLI-bound invocation path, not a Codex-portable one. Gap declared here, not silently dropped — pending a Codex-native init trigger design before rendering. |
| `.agents/skills/apply/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; `caller:` = adapter part | P078b2 DONE |
| `.agents/skills/forge/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; `caller:` = adapter part | P078b2 DONE |
| `.codex/config.toml` | ADAPTER_OWNED | lifecycle/tool config binding → `core/POLICY.md` (authority/scope), `core/ASSETS.md` (MCP registration) | P078b2 DONE |
| `.codex/hooks.json` | ADAPTER_OWNED | lifecycle event bindings → `core/POLICY.md` (enforcement), `core/WORKFLOW.md` (state gates) | P078b3 |
| `.codex/rules/*.rules` | ADAPTER_OWNED | exec policy (Starlark) → `core/POLICY.md` (authority/scope, outside-sandbox commands) | P078b3 |
| `scripts/codex/*` (rewritten guards) | ADAPTER_OWNED | policy intent → `core/POLICY.md`; host event payload/binding (apply_patch shape, shell-cmd inspection) = adapter part | P078b3 |

## Foundation coverage (P078b1 — code, not artifact rendering)

| Concern | Covered by |
|---|---|
| Adapter trait implementable for Codex | `crates/sos-adapter-codex/src/lib.rs` `impl Adapter for CodexAdapter` |
| Structural capability detection | `CodexAdapter::detect()` — static facts + fail-safe `codex --version` probe |
| Capability-gap declaration (machine) | `CodexAdapter::verify()` — 5 `Finding`s with `FindingStatus` (`Sound`/`Partial`/`Missing`) |
| Capability-gap declaration (human, frozen) | `CAPABILITY.md` (this directory) |
| Composition-root wiring | `crates/sos-cli/src/commands/install.rs` `run_codex()` |

## Render coverage (P078b2 — declarative render, structural oracle)

| Concern | Covered by |
|---|---|
| 10 declarative artifacts, crate-embedded template | `crates/sos-adapter-codex/src/templates.rs` (content-source, no `core/**` filesystem read at render time — Decision 1) |
| `render(asset)→Artifact` per-Asset, `plan()` enumerates fixed 10-Asset set | `crates/sos-adapter-codex/src/lib.rs` `CodexAdapter::plan()`/`render()` |
| Core-ID pointer, not semantic copy | Every template contains a `core/ROLES.md#<id>` / `core/WORKFLOW.md` / `core/ASSETS.md` string pointer only |
| PARTIAL-honest in-artifact marker | `architect.toml` template carries the PARTIAL envelope text; `advisory-watch.toml`/`boundary-check.toml` stay honest `sandbox_mode="read-only"` |
| Structural oracle (TOML parse, frontmatter valid, core-ID pointer present, PARTIAL marker present, artifact count == 10) | `crates/sos-adapter-codex/src/lib.rs` `#[cfg(test)]` module |
| Engine zero-touch | `crates/sos-install/src/engine.rs` diff empty — `ManagedOperation` consumed generically, no Codex-specific branch added |
| Skill set = 4, `init` deferred | Table above + `docs/discoveries/P078b2.md` |
| advisory-watch network-egress vs `sandbox_mode="read-only"` | UNCONFIRMED — escape hatch documented in-template + `docs/discoveries/P078b2.md`; behavioral resolution = P079 |

Every future-artifact row above has a non-empty core source ID (`core/ASSETS.md` line 51
requirement). No row references `adapters/` from `core/` — dependency direction stays one-way
(adapter → core), enforced by `crates/sos-core/tests/dep_direction.rs`.
