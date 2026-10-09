# PHIẾU P037: Pre-approve marker file Bash ops in install template

> **Loại:** Chore (DX / install friction)
> **Ưu tiên:** P1
> **Tầng:** 2 (lặt vặt — 2 anchor files, ≤80 LOC, no schema/API/auth/new-dep change)
> **Ảnh hưởng:** `templates/claude-settings.local.json` (NEW), `INSTALL.md` (Step 2.5), maybe `INSTALL.md` Common gotchas
> **Dependency:** P035 + P036 merged (tier routing + skip-CHALLENGE rule live)

---

## Context

### Vấn đề hiện tại

Orchestrator marker hygiene rule (ORCHESTRATION.md Hard rule #6, agents/orchestrator.md "Marker file hygiene") requires `mkdir -p .sos-state && touch .sos-state/architect-active` before each Architect spawn and `rm -f .sos-state/architect-active` before each Worker spawn.

Observed 2026-04-27 on Tarot project P050: every spawn triggers a `Bash(touch …/.sos-state/architect-active)` permission prompt because `.sos-state/` paths are not pre-allowlisted in user's `.claude/settings.local.json`. Sếp must click "Allow" on every state transition (DRAFT spawn, CHALLENGE spawn, RESPOND spawn, EXECUTE spawn — up to 4 prompts per phiếu lifecycle). This defeats the auto-orchestration goal of v2.1 ("user briefs in, approves once at APPROVAL_GATE, nghiệm thu out").

Note: marker lives at `.sos-state/architect-active` (outside `.claude/`) per ORCHESTRATION.md:115 — moving the marker is **not** the fix; that location was chosen specifically so YOLO mode (`--dangerously-skip-permissions`) doesn't gate it. The fix is pre-approving the Bash ops in user-level project settings.

### Giải pháp

**Option A — Ship `templates/claude-settings.local.json` template, instruct copy in INSTALL.md.**

1. Add `templates/claude-settings.local.json` containing a `permissions.allow` list with the three marker-related Bash patterns:
   - `Bash(mkdir -p .sos-state)`
   - `Bash(touch .sos-state/architect-active)`
   - `Bash(rm -f .sos-state/architect-active)`
2. Add INSTALL.md Step 2.5 (between current Step 2 "merge settings.json" and Step 3 "bootstrap docs"): instruct copying template to `.claude/settings.local.json` so marker ops don't prompt.
3. Add a Common gotchas row: "Per-spawn permission prompt for `.sos-state/architect-active` → did you copy the template in Step 2.5?"

**Why A over B (move marker to `/tmp/`):** marker location at `.sos-state/` was a deliberate design decision (ORCHESTRATION.md:115 — outside `.claude/` so YOLO doesn't gate). Re-opening that decision adds risk (worktree hashing, cross-session ambiguity, audit trail loss). Ship friction shifts from per-spawn to per-install (once).

**Why A over C (TaskList in-memory):** loses crash recovery, couples to Claude Code TaskList API. Already rejected by upstream design.

### Scope
- CHỈ sửa: `templates/claude-settings.local.json` (NEW), `INSTALL.md` (insert Step 2.5 + 1 gotcha row)
- KHÔNG sửa: `scripts/architect-guard.sh`, `scripts/session-start-banner.sh`, `agents/orchestrator.md`, `docs/ORCHESTRATION.md`, `.claude/settings.json` (project-level — different file from user's `.local.json`)

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Marker path is `.sos-state/architect-active` (NOT `.claude/.architect-active`) | `grep -n "architect-active" docs/ORCHESTRATION.md agents/orchestrator.md` | ✅ [verified] — ORCHESTRATION.md:115 ("Marker lives outside `.claude/`"), orchestrator.md:57-59 |
| 2 | INSTALL.md Step 2 covers settings.json merge; Step 3 covers docs bootstrap (so Step 2.5 is the right insertion point) | `grep -n "^### " INSTALL.md` | ✅ [verified] — INSTALL.md line 66 (Step 2), line 94 (Step 3) |
| 3 | `scripts/session-start-banner.sh` does not touch the marker (no edit needed there) | grep `architect-active` in scripts/ | ✅ [verified] — Architect just read the full file; only echoes a reminder, no `touch`/`rm` |
| 4 | `templates/` directory exists and has at least one settings template precedent (e.g. `BACKLOG_template.md`) | `ls templates/` + verify `BACKLOG_template.md` referenced from INSTALL.md:98 | ⚠️ [needs Worker verify] — Architect cannot list dirs; INSTALL.md:98 cites `~/sos-kit/templates/BACKLOG_template.md` so dir is highly likely to exist, but Worker confirms with `ls templates/` and reports actual contents |
| 5 | Claude Code's `.claude/settings.local.json` accepts `permissions.allow` array of `Bash(<exact-string>)` patterns and that `Bash(touch .sos-state/architect-active)` is a valid pattern (matches the literal command string Claude Code emits) | grep existing `.local.json` files in user's environment OR Claude Code docs | ⚠️ [needs Worker verify] — Architect has no way to confirm Claude Code's permission-string matching semantics; if Worker finds the actual pattern is e.g. `Bash(touch:*)` or wildcard `Bash(touch .sos-state/*)`, adapt template accordingly and log to Discovery |

**Anchor #4 + #5 are `[needs Worker verify]` per P036 acceptance — P037 demonstrates the marker pattern works on a Tầng 2 phiếu.**

---

## Debate Log

> Tầng 2 phiếu — orchestrator skips CHALLENGE_PHASE per ORCHESTRATION.md Hard rule #7 (P036). Worker self-verifies anchors in EXECUTE mode. This section stays empty unless Worker hits a Tầng 2 → Tầng 1 escalation mid-EXECUTE.

**Phiếu version:** V1 (initial draft)

**Skip-CHALLENGE invoked:** YES — per P036 ORCHESTRATION Hard rule #7. DRAFT → APPROVAL_GATE → EXECUTE. No CHALLENGE round-trip.

### Final consensus
- Phiếu version: V1 (Tầng 2, no debate)
- Total turns: 0
- Approved by Chủ nhà: [date] — code execution may begin

---

## Nhiệm vụ

### Task 1: Create `templates/claude-settings.local.json`

**File:** `templates/claude-settings.local.json` (NEW)

**Thêm (full file content):**
```json
{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "permissions": {
    "allow": [
      "Bash(mkdir -p .sos-state)",
      "Bash(touch .sos-state/architect-active)",
      "Bash(rm -f .sos-state/architect-active)"
    ]
  }
}
```

**Lưu ý:**
- This is the **user-level project settings** file (`.local.json`) — different from `.claude/settings.json` (project-shared, already exists with hooks). The `.local.json` file is per-user and typically gitignored — that's why it's installed via copy, not committed.
- If Worker verifies (Anchor #5) that Claude Code's permission strings use a different format (e.g. `Bash(touch:*)` glob, or `Bash(touch .sos-state/*)` wildcard), **adapt the entries** to whichever format Claude Code actually matches against, and log the actual format to Discovery. The intent (pre-approve the 3 marker ops) holds; the exact string syntax is Tầng 2.
- If Worker verifies that `templates/` directory does not exist (Anchor #4 ⚠️), create it with `mkdir -p templates` first.
- Do NOT add any other allowlist entries beyond the 3 marker ops. Scope creep = future phiếu.

### Task 2: Add INSTALL.md Step 2.5

**File:** `INSTALL.md`

**Tìm** (between Step 2 end and Step 3 start, around line 92-94):
```
Nếu đã có `PreToolUse` hooks khác → merge cùng matcher hoặc thêm entry mới.

### 3. Bootstrap docs (nếu thiếu)
```

**Thay bằng:**
```
Nếu đã có `PreToolUse` hooks khác → merge cùng matcher hoặc thêm entry mới.

### 2.5. Pre-approve marker file Bash ops (skip per-spawn permission prompts)

Orchestrator (main session) touches `.sos-state/architect-active` before spawning Architect and removes it before spawning Worker (marker hygiene per `docs/ORCHESTRATION.md` Hard rule #6). Without pre-approval, Claude Code prompts on every spawn — defeats v2.1 auto-orchestration.

```bash
# Copy template if .claude/settings.local.json doesn't exist
[ ! -f .claude/settings.local.json ] && cp ~/sos-kit/templates/claude-settings.local.json .claude/settings.local.json
```

Nếu `.claude/settings.local.json` đã có, **merge** thêm 3 entry vào `permissions.allow` array:
- `Bash(mkdir -p .sos-state)`
- `Bash(touch .sos-state/architect-active)`
- `Bash(rm -f .sos-state/architect-active)`

`.claude/settings.local.json` là per-user (thường `.gitignore` rồi) — không commit.

### 3. Bootstrap docs (nếu thiếu)
```

**Lưu ý:** Heading is `### 2.5.` to slot between current Step 2 and Step 3 without renumbering 3-5. If preferred to renumber to flat 3,4,5,6 — defer to Sếp; Tầng 2 surgical fix favors least-diff.

### Task 3: Add Common gotchas row

**File:** `INSTALL.md`

**Tìm** (in Common gotchas table, around line 235):
```
| BACKLOG.md không tồn tại | Bootstrap: `cp ~/sos-kit/templates/BACKLOG_template.md docs/BACKLOG.md` |
```

**Thay bằng:**
```
| BACKLOG.md không tồn tại | Bootstrap: `cp ~/sos-kit/templates/BACKLOG_template.md docs/BACKLOG.md` |
| Per-spawn `Bash(touch .sos-state/architect-active)` permission prompt | Bạn chưa làm Step 2.5 — copy `templates/claude-settings.local.json` vào `.claude/settings.local.json` |
```

**Lưu ý:** Append AFTER the BACKLOG row, before the table ends (last row). Preserve the table border.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `templates/claude-settings.local.json` | NEW — Task 1: 3-entry allowlist for marker Bash ops |
| `INSTALL.md` | Task 2: insert Step 2.5 between current Step 2 and Step 3; Task 3: add 1 row to Common gotchas table |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/architect-guard.sh` | Marker check semantics unchanged (still gates on `.sos-state/architect-active` existence) |
| `scripts/session-start-banner.sh` | Does not touch marker — no edit needed (Anchor #3) |
| `agents/orchestrator.md` | Marker hygiene section (line 56-61) still accurate — no edit needed |
| `docs/ORCHESTRATION.md` | Hard rule #6 (line 115) still accurate — no edit needed |
| `.claude/settings.json` (project-level) | NOT same file as `.claude/settings.local.json` (user-level). Project-level has hooks, user-level has permissions. Don't merge them. |

---

## Luật chơi (Constraints)

1. **Tier locked at 2.** No schema/API/auth/dep changes. ≤3 file edits, ≤80 LOC. If Worker discovers the change actually requires editing `scripts/architect-guard.sh` or `agents/orchestrator.md` (móng nhà), STOP and escalate Tầng 2 → Tầng 1 per ORCHESTRATION.md.
2. **No additional allowlist entries.** Only the 3 marker ops. Scope creep (e.g. pre-approving `Bash(grep …)` for Worker) = separate phiếu.
3. **`.local.json` is gitignored by default.** Don't commit the template's *output* (`.claude/settings.local.json` in user's project) — only commit the template (`templates/claude-settings.local.json` in sos-kit repo).
4. **If permission string format differs from assumption (Anchor #5):** adapt template to actual format, log to Discovery; do NOT block on this — the 3 ops + Tầng 2 budget hold either way.

---

## Nghiệm thu

### Automated
- [ ] `templates/claude-settings.local.json` is valid JSON (`python -m json.tool < templates/claude-settings.local.json` exits 0)
- [ ] No other files modified besides the 2 listed in "Files cần sửa"

### Manual Testing (dry-run)
- [ ] **Fresh-install dogfood:** on a clean test project (or fresh worktree of an existing sos-kit project), follow INSTALL.md Steps 1 → 2 → 2.5 → 3. Then run a smoke phiếu (per "First phiếu" section). Observe: **0 permission prompts** for `Bash(touch .sos-state/architect-active)` / `Bash(rm -f .sos-state/architect-active)` / `Bash(mkdir -p .sos-state)` across the full DRAFT → CHALLENGE → RESPOND → EXECUTE lifecycle.
- [ ] **Negative test:** on a project where Step 2.5 was *skipped*, confirm permission prompt still fires (proves the template is the thing doing the work, not some other config).

### Regression
- [ ] `scripts/architect-guard.sh` still blocks `.py/.rs/.ts` reads when marker exists (no change to guard semantics).
- [ ] Existing project-level `.claude/settings.json` hooks (SessionStart banner + PreToolUse architect-guard) still fire as before.

### Docs Gate
- [ ] `CHANGELOG.md` — entry: "P037: pre-approve marker Bash ops via `templates/claude-settings.local.json` template + INSTALL.md Step 2.5. Eliminates per-spawn permission prompt observed on Tarot 2026-04-27."
- [ ] `INSTALL.md` updated (Steps 2.5 + Common gotchas row — covered by Tasks 2-3).

### Discovery Report
- [ ] Append entry to `docs/DISCOVERIES.md` (newest on top):
  - Anchor #4 result: actual contents of `templates/` directory.
  - Anchor #5 result: actual permission string format Claude Code uses (literal exact match? glob? wildcard?). If non-literal, what was the working format?
  - Whether Step 2.5 placement (between 2 and 3) felt natural or whether renumbering INSTALL.md to flat 3,4,5,6 would be cleaner (signal for future phiếu).
  - Skip-CHALLENGE dogfood: did the Tầng 2 path (DRAFT → APPROVAL → EXECUTE, no CHALLENGE) feel correct for this scope, or did the absence of a CHALLENGE round let some bug slip through? Honest signal needed for the P036 retrospective.
