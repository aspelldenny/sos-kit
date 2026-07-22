# Codex CLI capability gaps (frozen, human-readable)

> Machine source of truth: `CodexAdapter::verify()` (`crates/sos-adapter-codex/src/lib.rs`).
> This document is seeded FROM that function, per `core/ROLES.md` separation-invariant #5:
> capability absence must be explicit, an integration cannot simulate success with prose.
> Ground truth: `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md` (Codex CLI 0.145.0).
> Vocabulary: `core/POLICY.md` "Oracle-first claims" — SOUND / PARTIAL / MISSING.

Codex CLI 0.145.0 does **not** expose 5 mechanisms the Claude adapter relies on. Each gap below
is declared with its status, why, and its backstop — never silently treated as SOUND.

## 1. Per-role built-in tool allowlist — PARTIAL

**What's missing:** Codex has no equivalent of Claude's `tools: Read,Write,Glob` frontmatter — a
structural, per-agent tool-removal mechanism. Codex CAN set `sandbox_mode=read-only` per agent
(sufficient for `advisory_watch`/`boundary_check`), but CANNOT express "architect gets ticket
Write but no Bash/Grep/Edit".

**Why PARTIAL, not MISSING:** the architect envelope is still enforceable, just via a weaker
mechanism — a `PreToolUse` hook that inspects the tool call plus prose in `AGENTS.md`/
`.codex/agents/architect.toml`, instead of Claude's structural tool-list removal.

**Backstop:** `PreToolUse` hook (P078b3 DONE — `scripts/codex/architect-guard.sh`, rendered to
target project) + Git/CI review gate on architect-authored diffs that touch `crates/**/src`
(see `core/POLICY.md` review-trigger map). Still PARTIAL, not upgraded to SOUND: the guard is
bypassable (untrusted repo / hooks disabled / obfuscated shell command) — see enforcement-weakness
note at the bottom, unchanged by rendering the guard.

## 2. Repo-distributed named slash commands — MISSING

**What's missing:** `.claude/commands/*.md` (e.g. `/security-review`) has no Codex equivalent —
Codex's custom-prompt feature is deprecated and personal-only, not repo-distributable.

**Replacement:** a repo skill invoked by name (`$security-review` under `.agents/skills/`).

**Backstop:** none needed beyond the skill itself — this is a full functional substitute, not a
weakened one; the gap is naming/discovery convention, not enforcement.

## 3. Skill-level `allowed-tools` — PARTIAL

**What's missing:** Codex does not mechanically gate tool calls based on a skill's declared
`allowed-tools` — the declaration is descriptive, not enforced at the tool-call layer.

**Why PARTIAL:** the skill still runs and its frontmatter is readable/auditable; only the
mechanical enforcement of the tool list is absent.

**Backstop:** same `PreToolUse` hook inspection as gap #1; Git/CI review for skill-authored diffs.

## 4. Native semantic ticket-version approval — MISSING

**What's missing:** neither Codex nor Claude has a built-in notion of "this exact ticket version
was approved." Codex's `approval_policy=on-request` approves individual operations, not a
ticket-version binding.

**Replacement:** build via a persisted approved-version marker (see `core/STATE.md` approval
record) checked by a `PreToolUse` guard before EXECUTE — **P078b3 DONE**:
`scripts/codex/approval-gate.sh` reads a `.sos-state/ticket-state.env` projection
(`version`/`approved_version`), read-compares ONLY, never mutates the record (mutation
authority = owner/bounded-delegate only), and fails CLOSED (blocks) when the state file is
missing or the versions mismatch.

**Backstop:** Git/CI — any execute-before-approval slip is still caught at review/merge time
(`core/POLICY.md` review-trigger map). Still MISSING→guard-replaced, not SOUND: the guard
itself is bypassable the same way as gap #1 (untrusted repo / disabled hooks).

## 5. Architect Read/Glob path interception — PARTIAL

**What's missing:** Codex has no Read/Glob-equivalent tool call to intercept — file reads happen
via shell (`rg`/`sed`/`cat`/etc. inside `apply_patch`-adjacent shell tool calls). The architect's
read-restriction (no reading `crates/**/src`) must inspect shell command TEXT, not a typed
`file_path` parameter — a fundamentally weaker interception point than Claude's tool-level Read
gate.

**Why PARTIAL:** the restriction can still be approximated (pattern-match shell command strings
in a `PreToolUse` hook — **P078b3 DONE**: `scripts/codex/architect-guard.sh`'s `Bash` branch
matches read-tool names (`rg`/`sed`/`cat`/`head`/`tail`/`less`/`awk`) plus a source-path
substring (`src/`, `.rs`, vision-doc names)), but it is heuristic, not structural — a
sufficiently obfuscated shell command (e.g. a variable-built path, base64-encoded argument)
could evade it.

**Enforcement-weakness note (applies across all 5 gaps):** none of the above hook-based
mitigations are unbypassable. Codex project hooks run only for TRUSTED repos; non-managed hooks
require explicit `/hooks` trust; users can disable hooks entirely; enterprise
`requirements.toml` settings are stronger but not guaranteed present. **Retain Git/CI backstops
for every gap above** — the hook layer is a fast-feedback convenience, not the sole enforcement
boundary.

## P078b3 enforcement status (guards rendered — still PARTIAL, never claimed SOUND)

All 5 gaps above now have a rendered `PreToolUse`/`UserPromptSubmit` guard
(`crates/sos-adapter-codex/src/templates.rs` E1-E6, `docs/ticket/P078b3-codex-enforcement.md`).
**This upgrades the backstop from "prose only" to "prose + fast-feedback hook" — it does NOT
upgrade any gap's status to SOUND.** Every gap above stays PARTIAL/MISSING-replaced, because:

- Codex project hooks only run for **TRUSTED** repos; a non-managed hook needs explicit
  `/hooks` trust; users can disable hooks entirely (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:22`).
- The `apply_patch` fail-CLOSED design (guard BLOCKs when it cannot extract a safe path from
  the patch body) closes the *parsing* risk, not the *bypass* risk above.
- Hosted/remote tool calls are not hook-visible at all.

**fail-CLOSED design note (Codex-specific inversion):** the Claude guards this rewrite is based
on (`scripts/{architect,orchestrator,block-env-edit}-guard.sh`) fail OPEN on unparseable input
("don't block on weird input" — a deliberate choice for Claude's `file_path`-typed payloads,
which are reliably parseable). The Codex rewrite inverts this: an `apply_patch` payload whose
patch body doesn't match the `*** Add/Update/Delete/Move File:` marker BLOCKS. Rationale: an
unparsed Codex patch could silently write anywhere, whereas an unparsed Claude `file_path` payload
essentially never occurs in practice — the failure modes are asymmetric, so the safe defaults are
too.

**`block-unsafe-merge` — DEFERRED, not rendered (Decision 4, P078b3):** the mechanical class
(force-push, `rm -rf` prefixes) is covered by `.codex/rules/exec-policy.rules` (Starlark
`prefix_rule`, most-restrictive-wins). The semantic class — blocking `gh pr merge <N>` without a
`/security-review` APPROVE sentinel — is NOT rendered for Codex: that logic lives in the external
`claude-hooks` Rust binary, which parses Claude's hook JSON shape specifically
(`scripts/block-unsafe-merge.sh:1-28`). Porting it to parse Codex's `Bash` `tool_input.command`
payload is a binary-side, behavioral change out of this phiếu's scope. **Backstop:** GitHub
branch protection + PR review requirement (Git/CI layer) still gates every merge regardless of
which agent authored the PR — this gap does not remove that boundary, it only means Codex lacks
the client-side fast-feedback Claude has via the ported binary.
