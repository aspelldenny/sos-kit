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

**Backstop:** `PreToolUse` hook (P078b3) + Git/CI review gate on architect-authored diffs that
touch `crates/**/src` (see `core/POLICY.md` review-trigger map).

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
record) checked by a `PreToolUse` guard before EXECUTE (P078b3).

**Backstop:** Git/CI — any execute-before-approval slip is still caught at review/merge time
(`core/POLICY.md` review-trigger map).

## 5. Architect Read/Glob path interception — PARTIAL

**What's missing:** Codex has no Read/Glob-equivalent tool call to intercept — file reads happen
via shell (`rg`/`sed`/`cat`/etc. inside `apply_patch`-adjacent shell tool calls). The architect's
read-restriction (no reading `crates/**/src`) must inspect shell command TEXT, not a typed
`file_path` parameter — a fundamentally weaker interception point than Claude's tool-level Read
gate.

**Why PARTIAL:** the restriction can still be approximated (pattern-match shell command strings
in a `PreToolUse` hook, P078b3), but it is heuristic, not structural — a sufficiently obfuscated
shell command could evade it.

**Enforcement-weakness note (applies across all 5 gaps):** none of the above hook-based
mitigations are unbypassable. Codex project hooks run only for TRUSTED repos; non-managed hooks
require explicit `/hooks` trust; users can disable hooks entirely; enterprise
`requirements.toml` settings are stronger but not guaranteed present. **Retain Git/CI backstops
for every gap above** — the hook layer is a fast-feedback convenience, not the sole enforcement
boundary.
