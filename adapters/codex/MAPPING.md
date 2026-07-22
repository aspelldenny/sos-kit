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
| `.codex/hooks.json` | ADAPTER_OWNED | lifecycle event bindings → `core/POLICY.md` (enforcement), `core/WORKFLOW.md` (state gates) | P078b3 DONE |
| `.codex/rules/*.rules` | ADAPTER_OWNED | exec policy (Starlark) → `core/POLICY.md` (authority/scope, outside-sandbox commands) | P078b3 DONE |
| `scripts/codex/*` (rewritten guards) | ADAPTER_OWNED | policy intent → `core/POLICY.md`; host event payload/binding (apply_patch shape, shell-cmd inspection) = adapter part | P078b3 DONE |

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

## Enforcement coverage (P078b3 — 7 enforcement artifacts, structural + mock-payload oracle)

**b3 = the LAST piece of P078b. P078b (b1+b2+b3) is DONE.**

Ground-truth apply_patch envelope confirmed live against Codex CLI (gpt-5.6, P078b3 Debate
Log Turn 2): `{"tool_name":"apply_patch","tool_input":{"command":"<V4A patch>"},...}`, patch
body = `*** Begin Patch\n*** Add|Update|Delete File: <path>\n...*** End Patch` with embedded
newlines escaped as literal `\n`. Fixture committed:
`crates/sos-adapter-codex/tests/fixtures/codex-apply-patch-payloads.jsonl` (4 real payloads).

| Concern | Covered by |
|---|---|
| 7 enforcement artifacts, crate-embedded template | `crates/sos-adapter-codex/src/templates.rs` (`HOOKS_JSON`/`RULES_EXEC_POLICY`/5× `GUARD_*` identities) |
| apply_patch path extraction (fail-CLOSED) | `grep -oE '\*\*\* (Add\|Update\|Delete\|Move) File: [^\\"]+'` on the raw hook stdin JSON line — every guard BLOCKs (exit 2) if this yields no path, inverting the Claude fail-open precedent |
| Architect envelope (write-allowlist + shell-read heuristic) | `scripts/codex/architect-guard.sh` — apply_patch write-allowlist `P[0-9]*-*.md`; Bash-read heuristic on `src/`/`.rs`/vision-doc substrings (PARTIAL, gap #5) |
| Orchestrator envelope (product-source gate) | `scripts/codex/orchestrator-guard.sh` — blocks apply_patch product-source writes unless `.sos-state/worker-active` present |
| Secret guard | `scripts/codex/block-env-edit.sh` — blocks apply_patch on `.env*` except `.env.example` |
| Approval gate (Codex native gap #4, guard-BUILT) | `scripts/codex/approval-gate.sh` — reads `.sos-state/ticket-state.env` projection (`version`/`approved_version`), read-compares ONLY, never mutates; fail-CLOSED when state file missing |
| Idea-smell intake pointer | `scripts/codex/idea-smell.sh` — UserPromptSubmit regex reminder (`core/WORKFLOW.md` intake) |
| Mechanical exec policy | `.codex/rules/exec-policy.rules` — Starlark `prefix_rule` force-push/destructive-command deny; PR-merge-sentinel gate DEFERRED (semantic, external `claude-hooks` binary — Git/CI backstop retained) |
| Mock-payload structural oracle (block/allow correctness) | `crates/sos-adapter-codex/src/lib.rs` `mock_payload_oracle` test module — feeds real + path-substituted-real fixture payloads to each rendered guard, asserts exit-code block(2)/allow(0); `#[cfg(unix)]`-gated (bash-exec), content-pattern assertions stay cross-platform |
| PARTIAL-honest (3-surface) | `.codex/hooks.json` `_partial_note` field, every guard-script header, this MAPPING.md, `CAPABILITY.md`, `SECURITY.md` all state: bypassable (untrusted repo / disabled hooks / obfuscated command) → Git/CI backstop retained |
| Additive, not committed to sos-kit | `.codex/`/`scripts/codex/` never appear in `git status` of this repo — render only targets a project via `install --runtime codex` |

**Behavioral verification (Codex CLI actually enforcing these hooks/rules at runtime) is
OUT OF SCOPE — deferred to P079.** b3's oracle is structural: artifact validity + guard
parse-correctness against ground-truth payload shape, not a live Codex dogfood run.
