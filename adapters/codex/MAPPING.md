# Codex artifact → core source ID mapping

> P078b1 foundation — seed shape. No Codex-native artifact has been physically rendered yet;
> this table declares the intended ownership so P078b2/b3 render against a fixed contract, mirroring
> `adapters/claude/MAPPING.md`'s declarative pattern. "Physical render" column tracks the future
> phiếu, not present state.

| Artifact (future, declared) | Class | Core source ID | Physical render |
|---|---|---|---|
| `AGENTS.md` (root) | ADAPTER_OWNED | orchestrator contract → `core/ROLES.md#orchestrator`, `core/WORKFLOW.md` | P078b2 |
| `.codex/agents/architect.toml` | ADAPTER_OWNED | `core/ROLES.md#architect` | P078b2 |
| `.codex/agents/worker.toml` | ADAPTER_OWNED | `core/ROLES.md#worker` | P078b2 |
| `.codex/agents/advisory-watch.toml` | ADAPTER_OWNED | `core/ROLES.md#advisory_watch` | P078b2 |
| `.codex/agents/boundary-check.toml` | ADAPTER_OWNED | `core/ROLES.md#boundary_check` | P078b2 |
| `.agents/skills/idea/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; frontmatter `caller:` = adapter part | P078b2 |
| `.agents/skills/retro/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; `caller:` = adapter part | P078b2 |
| `.agents/skills/init/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; `caller:` = adapter part | P078b2 |
| `.agents/skills/apply/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; `caller:` = adapter part | P078b2 |
| `.agents/skills/forge/SKILL.md` | ADAPTER_OWNED | portable body → core skill semantics; `caller:` = adapter part | P078b2 |
| `.codex/config.toml` | ADAPTER_OWNED | lifecycle/tool config binding → `core/POLICY.md` (authority/scope), `core/ASSETS.md` (MCP registration) | P078b2 |
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

Every future-artifact row above has a non-empty core source ID (`core/ASSETS.md` line 51
requirement). No row references `adapters/` from `core/` — dependency direction stays one-way
(adapter → core), enforced by `crates/sos-core/tests/dep_direction.rs`.
