---
description: Scan stack for security advisories via GHSA + vendor pages. Verifies .sos-stack.toml exists, spawns Trinh sát subagent (which runs parsers + queries GHSA), appends results to advisory inbox.
---

# /advisory-scan

You are the orchestrator (Quản đốc) running the advisory-scan slash command. Execute these steps in order — DO NOT skip, DO NOT improvise. Parser invocation happens INSIDE the Trinh sát subagent, NOT in this main session.

## Step 0 — Verify prerequisites

1. Verify `.sos-stack.toml` exists at project root via `Glob(".sos-stack.toml")`.
   - **If missing:** STOP. Tell user: "No `.sos-stack.toml` found. Run `sos init security` first (foundation from P040 — detects stack + writes schema). Then re-run `/advisory-scan`."
2. Verify inbox file exists. Default path: `docs/security/advisory-inbox.md`.
   - **If missing:** Use `Read templates/advisory-inbox.md` then `Write docs/security/advisory-inbox.md` (mkdir-then-write via the file-write tool, NOT via Bash). Tell user the inbox was bootstrapped.

## Step 1 — Spawn Trinh sát subagent

Use `Task` tool with `subagent_type: "advisory-watch"`. Prompt format:

```
You are Trinh sát. Run your full workflow (Bước 0 PyYAML pre-flight → Bước 1 parse stacks → Bước 2 query advisories → Bước 3 grep code → Bước 4 format rows → Bước 5 final report).

Project root: <cwd>
`.sos-stack.toml` path: .sos-stack.toml (or absolute path if cwd unclear)

Return your final report with `<!-- advisory-start -->` ... `<!-- advisory-end -->` block as specified.
```

Wait for subagent return. Subagent handles parser invocation (scoped Bash) + GHSA query + code grep entirely on its own.

## Step 2 — Extract sentinel block from subagent output

Use `Grep` or string parsing to locate the block between `<!-- advisory-start -->` and `<!-- advisory-end -->` in the subagent's return.

- If block is empty (no rows between markers) → tell user "Scan complete. 0 new advisories." STOP.
- If block has N rows → continue to Step 3.

## Step 3 — Append rows to inbox

`Read` current inbox file. Locate the existing `<!-- advisory-start -->` and `<!-- advisory-end -->` markers (between them is the active rows region).

Insert subagent's new rows BETWEEN the existing markers (preserve markers; append below existing rows if any).

`Write` updated inbox file.

## Step 4 — Report to user

Tell user:
- N new advisories appended to `<inbox-path>`.
- Per-row summary (1-line each): `<advisory-ID> <severity> <name@version> — <file:line or indirect>`.
- Next action: review inbox, mark each row `dismissed` or create follow-on phiếu via `phieu <slug>`.

## Hard rules

- Trinh sát is the WORKHORSE. Parser invocation, advisory query, code grep all happen INSIDE the subagent (scoped Bash). Main session ONLY verifies prerequisites + spawns + appends.
- All `Write` happens in this slash command (orchestrator side), never inside the subagent.
- Sentinel markers `<!-- advisory-start -->` / `<!-- advisory-end -->` are LOAD-BEARING. Do not rename, do not duplicate, do not move.
- Schema version check (subagent handles via Bước 1) is a hard gate — schema v1 ONLY in P041. v2 = breaking change requires phiếu.
- If Trinh sát reports "all parsers stubs / PyYAML missing / no stacks" → relay the message verbatim, NOT a silent success.
