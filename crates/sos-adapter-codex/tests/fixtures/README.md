# Fixtures — Codex apply_patch/Bash hook payloads (P078b3)

`codex-apply-patch-payloads.jsonl` — 4 REAL PreToolUse hook stdin payloads, captured live
from Codex CLI (gpt-5.6) by Sếp on 2026-07-22 (P078b3 Debate Log Turn 2, escape-hatch
anchor #1 resolution). Each line is exactly what a `PreToolUse` hook receives on stdin:

1. `apply_patch` — Add File
2. `apply_patch` — Update File
3. `apply_patch` — Delete File
4. `Bash` — a plain shell read command

These are the GROUND TRUTH for the `*** Add|Update|Delete|Move File: <path>` marker
extraction regex used by every `scripts/codex/*` guard rewritten in P078b3
(`crates/sos-adapter-codex/src/templates.rs`). Do NOT hand-edit this file to "improve"
coverage — synthetic variants used in `crates/sos-adapter-codex/src/lib.rs` unit tests
substitute only the target *path* inside the same confirmed envelope shape; they never
invent a different envelope. If Codex's payload shape changes in a future CLI version,
re-capture and replace this file, noting the CLI version in this README.
