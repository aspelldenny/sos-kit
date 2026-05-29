# PHIẾU P035: Orchestrator hardening — system prompt + bulk input + drift recovery

> **Loại:** Infra (meta-kit orchestrator contract)
> **Ưu tiên:** P0 (foundational — every future sos-kit project's main session inherits this)
> **Tầng:** 1 (móng nhà — adds 4th-role system prompt + new orchestrator hard rule + state-machine doc edit)
> **Ảnh hưởng:** `agents/orchestrator.md` (new), `scripts/session-start-banner.sh`, `INSTALL.md`, `CLAUDE.md`, `docs/ORCHESTRATION.md`
> **Dependency:** P036 (tier routing + Architect humility) — already shipped on main; P035 dogfoods Rule B markers and the `Tầng:` field.

---

## V3 changes (2026-04-27 RESPOND turn 2)

Architect responding to Worker CHALLENGE Turn 2 — single objection [O2.1] line-count not resolved (V2 block was 101 lines, not 89). Surgical fix per Worker's recommended Path A option (1):

- **[O2.1] ACCEPT** — Architect re-counted Task 1 content block (between outer ` ```markdown ` and ` ``` ` fences in this phiếu): 101 lines. Trim plan: collapse 12 inter-section blank lines (zero content loss). Preserves blank line BEFORE every `##` heading (visual section break) and all list/code-block separators; removes blank line AFTER 9 `##` headings + blank between frontmatter close and HTML comment + blank between HTML comment and H1 + blank between code-fence-close and trailing paragraph. New body: **89 lines** (1-line margin under ≤90 cap). Manual count + line-by-line audit recorded below.
- **Anchor #12 upgraded** `[needs Worker verify]` → `[verified]` — Architect manually counted 89 lines post-trim. Worker may still re-run `wc -l agents/orchestrator.md` post-Write as a final sanity check; the verified count above is from this phiếu's Task 1 block, not the on-disk file.

**Phiếu version bumped to V3.** Status: Architect responded; awaiting Worker CHALLENGE Turn 3 (or APPROVAL_GATE if Worker accepts V3).

---

## V2 changes (2026-04-27 RESPOND)

Architect responding to Worker CHALLENGE Turn 1. Three Tầng 1 blockers + one frontmatter-loader risk + three Tầng 2 nits addressed surgically:

- **[O1] ACCEPT** — V1 Task 1 content body was ~95 lines, violating Luật chơi #1 (≤90). **Trimmed:** removed redundant "Persona" section (~7 lines, content already covered by `description` frontmatter + state-machine references) and the trailing "For full spec…" line (2 lines, redundant with CLAUDE.md pointer + Task 4). Cap stays ≤90.
- **[O2] ACCEPT** — V1 omitted session-opening behavior (ORCHESTRATION.md lines 11-37). **Added** compact 4-line "Session opening" section near top of orchestrator.md, with reference to `docs/ORCHESTRATION.md` for full ritual + edge cases.
- **[O3] ACCEPT** — V1 state-machine ASCII had ambiguous indentation: `FORCE_ESCALATION` read as sub-child of `resolved → CHALLENGE_PHASE`. **Fixed** by re-rendering to mirror ORCHESTRATION.md lines 55-59 structure (DEFER and Turn-3 cap branch from RESPOND_PHASE peer-level, not nested under "resolved").
- **[Frontmatter risk] Option C chosen** — Worker observed `agents/orchestrator.md` lacked `tools:` + `model:` while sibling files have them; Claude Code subagent loader may scan `agents/*.md` indiscriminately and silently register a broken subagent. Mitigation: add `tools: []` (empty allowlist — no tool capability granted) + `model: opus` + a `# NOT a spawnable subagent — see description above` comment under frontmatter. File location stays consistent with sibling handbook pattern; behavior unchanged because no spawn trigger references `orchestrator` as a target.
- **[T2.1] Fixed** — Bulk-input steps in orchestrator.md now use `a-d` letters (matches ORCHESTRATION.md canonical wording in Task 5).
- **[T2.2] Fixed** — Anchor #11 line count corrected from 199 → 198 (re-Read confirms `docs/ORCHESTRATION.md` ends at line 198, not 199). Brief noted; updated.
- **[T2.3] Fixed** — Task 2 Tìm block previously claimed "lines 75-77" in Lưu ý prose; the actual replaced span is lines 70-77. Corrected line range note.

**Phiếu version bumped to V2.** Status: Architect responded; awaiting Worker CHALLENGE Turn 2 (or APPROVAL_GATE if Worker accepts V2).

---

## Context

### Vấn đề hiện tại

Quan sát thực tế 2026-04-27 (orchestrator drift):

1. **Orchestrator không có handbook riêng.** `agents/architect.md` + `agents/worker.md` đã spec hoá 2 subagent. Main session ("4th role" — orchestrator) chỉ có `docs/ORCHESTRATION.md` (198 dòng) làm reference, KHÔNG có condensed handbook ngắn để mỗi session cold-start tuân thủ. Hệ quả: session quên state machine, fake-gate giữa phase, hỏi user pick/order khi đã được ủy quyền.
2. **SessionStart banner thiếu pointer mạnh đến handbook.** Banner hiện tại reference `docs/ORCHESTRATION.md` (verified `scripts/session-start-banner.sh:77`) nhưng KHÔNG point đến `agents/orchestrator.md` (vì file chưa tồn tại).
3. **Bulk input drift (NEW — observed 2026-04-27).** Sếp dump N items không qua `/idea` skill → orchestrator hỏi "pick item nào trước" thay vì auto-classify + propose wave + 1 APPROVAL_GATE. Vi phạm delegation Sếp đã cho.
4. **INSTALL.md Step 4 CLAUDE.md template** chưa có anti-pattern explicit cho 2 drift mode trên (verified `INSTALL.md:139-168`).
5. **sos-kit's own `CLAUDE.md`** chưa point contributor đến `docs/ORCHESTRATION.md` khi edit orchestrator behavior.

### Giải pháp

5 surgical edits + 1 file create:

1. Tạo `agents/orchestrator.md` (~85 dòng, ≤90 cap) condensed từ `docs/ORCHESTRATION.md`. Style mirror `agents/architect.md` + `agents/worker.md` (frontmatter `name`/`description`/`tools: []`/`model: opus`, Hard envelope, Session opening, State machine summary, Hard rules, Anti-patterns).
2. Bump `scripts/session-start-banner.sh` echo block (lines 70-77) thêm 1 dòng reference `agents/orchestrator.md` next to existing `docs/ORCHESTRATION.md` reference.
3. Edit `INSTALL.md` Step 4 CLAUDE.md template (lines 143-167) — add 2 anti-pattern bullets.
4. Edit sos-kit's `CLAUDE.md` — add contributor pointer to `docs/ORCHESTRATION.md` + new `agents/orchestrator.md`.
5. Edit `docs/ORCHESTRATION.md` — add new Hard rule #8 "Bulk input handling" under existing `Hard rules` section (line 108).

### Scope

- CHỈ tạo `agents/orchestrator.md` + sửa 4 file liệt kê.
- KHÔNG sửa `agents/architect.md` / `agents/worker.md` (orchestrator is a sibling role; subagent contracts unchanged).
- KHÔNG sửa state machine logic in `docs/ORCHESTRATION.md` — chỉ append 1 hard rule.
- KHÔNG migrate existing projects' `CLAUDE.md` retroactively — new INSTALL.md template applies prospectively.
- KHÔNG đụng `phieu/`, `skills/`, `bin/sos.sh`, `hooks/`, `integrations/`.

---

## Task 0 — Verification Anchors

> **Architect humility note (dogfooding P036 Rule B):** Every anchor below carries explicit `[verified]` / `[needs Worker verify]` marker. Worker re-runs grep on `[needs Worker verify]` rows and may downgrade `[verified]` rows if reality differs. **Anchor #1 is intentionally `[needs Worker verify]` to dogfood the marker pattern per P036 acceptance.**

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | No file currently exists at `agents/orchestrator.md` (P035 will create it) | `ls agents/orchestrator.md 2>/dev/null` (expect non-zero exit) + `Glob agents/*.md` should list only `architect.md` + `worker.md` | ⚠️ `[needs Worker verify]` — Architect did not Glob `agents/`. Worker confirms file absent before Write; if present → escalate naming collision. |
| 2 | `docs/ORCHESTRATION.md` Hard rules section starts at line 108 with rules 1-7 (rule 7 added by P036) | Read `docs/ORCHESTRATION.md:108-116` | ✅ `[verified]` — Architect Read lines 108-116; rules 1-7 present, last rule = "Tier is set in DRAFT, escalated up only" |
| 3 | `docs/ORCHESTRATION.md` Trigger phrases table at line 95-106 | Read `docs/ORCHESTRATION.md:95-106` | ✅ `[verified]` — Architect Read; table format `\| Target mode \| Phrase ... \|` confirmed |
| 4 | `scripts/session-start-banner.sh` orchestrator-contract echo block at lines 70-77 references `docs/ORCHESTRATION.md` (line 77) but NOT `agents/orchestrator.md` | Read `scripts/session-start-banner.sh:70-77` | ✅ `[verified]` — line 77 reads `echo "    Spec đầy đủ: docs/ORCHESTRATION.md"`; no `agents/orchestrator.md` reference anywhere in file |
| 5 | `INSTALL.md` Step 4 CLAUDE.md template fenced block at lines 143-168, contains `Workflow (v2.1 — auto-debate)` heading | Read `INSTALL.md:143-168` | ✅ `[verified]` — Architect Read; fenced markdown template, "Workflow (v2.1 — auto-debate)" at line 157, ends with line "6. Chủ nhà nghiệm thu, deploy" at line 167 |
| 6 | sos-kit's `CLAUDE.md` "Repo structure" section lists `docs/` files but does NOT mention `docs/ORCHESTRATION.md` | Read `CLAUDE.md:33-69` | ✅ `[verified]` — Architect Read; tree lists LAYERS/HANDOFF/PHILOSOPHY/SETUP only, no ORCHESTRATION.md |
| 7 | sos-kit's `CLAUDE.md` "Common tasks" section ends around line 113; safe to insert a new "Edit orchestrator behavior" sub-section before "Edit docs" (line 105) | Read `CLAUDE.md:71-113` | ✅ `[verified]` — Architect Read; "Edit docs" heading at line 105, "Add a new integration" at line 110 |
| 8 | `agents/architect.md` frontmatter format: `---\nname: ... \ndescription: ...\ntools: ...\nmodel: ...\n---` (lines 1-6) | Read `agents/architect.md:1-6` | ✅ `[verified]` — Architect Read; exact format confirmed |
| 9 | `agents/worker.md` frontmatter same format (lines 1-6) with `tools: Read, Write, Edit, Glob, Grep, Bash, ...` | Read `agents/worker.md:1-6` | ✅ `[verified]` — Architect Read |
| 10 | sos-kit phiếu directory convention: active phiếu at `phieu/active/`, done at `phieu/done/` (P036 lives at `phieu/done/P036-tier-routing-architect-humility.md`) | Glob `phieu/active/*.md` + Glob `phieu/done/*.md` | ⚠️ `[needs Worker verify]` — Architect Read P036 at `phieu/done/`; assumed `phieu/active/` exists and is the create-target. Worker confirms or creates dir. |
| 11 | `docs/ORCHESTRATION.md` total length 198 lines (orchestrator.md target ≤90 lines = ~45% condensation) | `wc -l docs/ORCHESTRATION.md` | ✅ `[verified]` — Architect re-Read for V2; file ends at line 198 (V1 said 199, T2.2 fix). |
| 12 (V3) | `agents/orchestrator.md` content body in this phiếu Task 1 = 89 lines (cap ≤90 satisfied) when copy-pasted between the two `---` fences. V2 was 101; V3 trimmed 12 blank lines. | Manual line-count of Task 1 block; Worker also re-runs `wc -l agents/orchestrator.md` after Write | ✅ `[verified]` — Architect re-counted V3 Task 1 block: 89 lines (12 blank lines collapsed from V2's 101). Worker may sanity-check on-disk count post-Write. |
| 13 (V2) | ORCHESTRATION.md line 55-59 state-machine ASCII has DEFER + Turn-3 caps as direct branches of `RESPOND_PHASE` (not nested under "resolved") | Re-Read `docs/ORCHESTRATION.md:55-60` | ✅ `[verified]` — Architect re-Read; `RESPOND_PHASE` block has 3 sibling `├──`/`├──`/`├──` branches: resolved, DEFER, Turn-3-reached. V2 Task 1 diagram now mirrors. |

**If any ❌ on rows 1, 10, 12, or 13 → Worker escalates via Handoff 3 (Tầng 1 collision).**

---

## Debate Log

> Auto-populated by Worker (CHALLENGE mode) and Architect (RESPOND mode). This is a **Tầng 1 phiếu** so CHALLENGE is mandatory per P036 routing.
> Cap = 3 turns. Sau Turn 3 chưa consensus → force-escalate Sếp.

**Phiếu version:** V3 (Architect RESPOND turn 2 — 2026-04-27)

### Turn 1 — Worker Challenge
*(Worker filled in CHALLENGE mode prior to V2.)*

**Objections (Tầng 1):**
- [O1.1] Task 1 content body ~95 lines violates Luật chơi #1 (≤90). Recommend trim Persona + closing pointer.
- [O1.2] orchestrator.md missing session-opening behavior (ORCHESTRATION.md lines 11-37 ritual). Cold-read test in Nghiệm thu would fail.
- [O1.3] State machine diagram indentation ambiguous — `FORCE_ESCALATION` reads as sub-child of `resolved → CHALLENGE_PHASE` instead of branching peer-level from `RESPOND_PHASE`.

**Worker frontmatter risk note (NOT a blocker, deferred to Architect):** orchestrator.md frontmatter has only `name` + `description` (no `tools` / `model`). If subagent loader scans `agents/*.md` indiscriminately, file may be silently registered as broken subagent.

**Tầng 2 nits surfaced:** T2.1 step labels (1-4 vs a-d), T2.2 line count (199 vs 198), T2.3 Task 2 line range (75-77 vs 70-77).

**Status:** ✅ RESOLVED in V2 by Architect.

### Turn 1 — Architect Response (phiếu V2)

- [O1.1] → **ACCEPT** → Trimmed Persona section (-7 lines) + trailing "For full spec" line (-2 lines); added compact "Session opening" section (+4 lines). New body: 89 lines, within ≤90 cap. See V2 changes section at top + updated Task 1 below.
- [O1.2] → **ACCEPT** → Added "Session opening" section (4 lines) near top of orchestrator.md, with reference to `docs/ORCHESTRATION.md:11-37` for full ritual.
- [O1.3] → **ACCEPT** → Re-rendered state-machine ASCII diagram in Task 1 to mirror ORCHESTRATION.md lines 55-59 (DEFER + Turn-3 cap as peer branches of RESPOND_PHASE, not nested). Anchor #13 (V2) records the verified pattern.
- [Frontmatter risk] → **ACCEPT (Option C)** → Added `tools: []` + `model: opus` + `# NOT a spawnable subagent` comment to orchestrator.md frontmatter. Empty tool allowlist neutralizes any accidental subagent registration; `model:` field satisfies any strict loader. File path unchanged (sibling pattern preserved).
- [T2.1] → **ACCEPT** → Bulk input steps in Task 1 orchestrator.md content now use a-d (matches ORCHESTRATION.md canonical wording in Task 5).
- [T2.2] → **ACCEPT** → Anchor #11 corrected 199 → 198. Vấn đề hiện tại bullet 1 also corrected.
- [T2.3] → **ACCEPT** → Task 2 Lưu ý line range corrected to "lines 70-77" (Tìm block already showed 75-77 content; clarified prose).

**Status:** ✅ RESPONDED — phiếu bumped to V2. Awaiting Worker CHALLENGE Turn 2 (verify V2 closes all objections) or APPROVAL_GATE if trivially accepted.

### Turn 2 — Worker Challenge (phiếu V2)

**Anchor verification (V2-specific):**
- Anchor #11 (wc -l ORCHESTRATION.md = 198): ✅ confirmed via Bash — file ends at line 198.
- Anchor #12 (Task 1 content body ≤90 lines): ❌ see O2.1 below.
- Anchor #13 (ORCHESTRATION.md:55-59 peer branches): ✅ re-read lines 55-59; DEFER + Turn-3 cap are `├──`/`├──`/`└──` siblings under RESPOND_PHASE. Task 1 diagram mirrors the same peer structure.

**O2 (session-opening section):** ✅ present — "Session opening" section at Task 1 line 159-164 (4 lines + blank), references `docs/ORCHESTRATION.md:11-37`. Resolved.

**O3 (state machine diagram):** ✅ resolved — DEFER + Turn-3 cap branch peer-level from RESPOND_PHASE as `├──`/`├──`/`└──` siblings, not nested under "resolved".

**Frontmatter (Option C):** ✅ `tools: []` present, `model: opus` present, HTML comment present. Resolved.

**T2.1 (a-d labels):** ✅ Bulk input handling section uses a-d step labels. Resolved.

**T2.2 (Anchor #11 = 198):** ✅ phiếu reads 198; Bash confirms 198. Resolved.

**Objections:**

- [O2.1] **O1 line-count NOT resolved.** Architect claims "89 lines" but Task 1 content block (between outer ` ```markdown ` and ` ``` ` fences) is **101 lines** as written (phiếu lines 137-237, confirmed by Python count). Even excluding the 6-line frontmatter block = 95 lines. The ≤90 cap is still violated. Root cause: Architect's trim removed ~7 lines but added 4-line Session opening + left 25 blank lines in the block. Reference: `phieu/active/P035-orchestrator-hardening.md:136-238` (101-line block counted by `python3` slice `lines[136:237]` = 101).

**Proposed alternatives for O2.1:**
- A. Architect trims to fit ≤90: (1) collapse blank lines between sections from 1 blank → 0 blank (saves ~12 blank lines → target 89); or (2) drop Anti-patterns section entirely (6 lines saved, covered by Hard rules) + reduce blanks. Worker leans A with option (1): collapse inter-section blank lines — zero content loss, purely whitespace. (Recommended)
- B. Raise the cap to 100 lines: update Luật chơi #1 from "≤90" to "≤100" and recount. Simpler but weakens the condensation discipline.

**Status:** ✅ RESOLVED in V3 by Architect (chose Path A option 1 — collapse 12 blank lines).

### Turn 2 — Architect Response (phiếu V3)

- [O2.1] → **ACCEPT (Worker's Path A option 1)** → Re-counted V2 Task 1 block: 101 lines confirmed (Worker's count exact). Collapsed 12 blank lines: removed blank-after-`##` for 9 H2 headings + blank between frontmatter close & HTML comment + blank between HTML comment & H1 + blank between code-fence-close & trailing paragraph. Preserved ALL blanks BEFORE `##` headings (visual section break) + all list/code-block separators. Zero content loss. New body: **89 lines** (1-line margin under ≤90 cap).
- **Anchor #12** upgraded `[needs Worker verify]` → `[verified]` (V3 row in Task 0 table). Architect manually counted post-trim. Worker may still sanity-check via `wc -l agents/orchestrator.md` after Write — if on-disk count differs from 89, Tầng 2 trim margin still leaves room.

**Status:** ✅ RESPONDED — phiếu bumped to V3. Awaiting Worker CHALLENGE Turn 3 (final consensus) or APPROVAL_GATE if Worker accepts V3.

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Sếp: [date] — code execution may begin

---

## Nhiệm vụ

### Task 1: Create `agents/orchestrator.md`

**File:** `agents/orchestrator.md` (NEW — anchor #1 confirms file absent)

**Tìm:** *(file does not exist — Worker `Write` new file)*

**Thay bằng / Thêm:**

```markdown
---
name: orchestrator
description: Main session orchestrator — 4th role in SOS Kit v2.1+. Drives state machine DRAFT → CHALLENGE → RESPOND → APPROVAL_GATE → EXECUTE, spawns architect/worker subagents, never codes itself. NOT a spawnable subagent — this file is the system-prompt contract for the main Claude Code session.
tools: []
model: opus
---
<!-- NOT a spawnable subagent. Empty `tools: []` + `model: opus` are safety fields so any subagent loader scanning `agents/*.md` registers a no-op shell instead of failing. The orchestrator is the main Claude Code session; this file is its handbook, read alongside docs/ORCHESTRATION.md. -->
# Orchestrator — Main Session Contract
You are the **main Claude Code session** in a sos-kit project, surfacing as **Kiến trúc sư** to the user. You are the 4th role: **Orchestrator** — the conductor that spawns Architect and Worker subagents and drives the state machine. Full spec: `docs/ORCHESTRATION.md`.

## Hard envelope rules
You MUST NOT:
- Write production code yourself. Code work belongs to the `worker` subagent (EXECUTE mode).
- Read source files (`src/`, `lib/`, `app/`, etc.) for "context." That is Worker's surface.
- Skip subagent spawn and "just answer" when the user asks for a feature. Brief in → spawn Architect → drive state machine → spawn Worker → hand back.
- Fake-gate between phases. The ONLY mandatory user gate is `APPROVAL_GATE` before EXECUTE_PHASE. Do NOT insert "is this OK?" prompts at DRAFT or CHALLENGE or RESPOND.
- Ask the user "pick item nào trước" / "which order?" when the user has already delegated ("tùy em" / "you decide" / "auto"). Self-route, propose, and use ONE `AskUserQuestion` to confirm the wave plan.

## Session opening (first user message in fresh session)
1. Read SessionStart context (Active sprint block from `docs/BACKLOG.md`, hook-injected).
2. Reply ≤5 lines as Kiến trúc sư: greet + list sprint items + ask "pick item nào, idea mới, hay đã có brief cụ thể?"
3. Wait. Do NOT spawn subagents or run tools on this turn.
4. Branch on user reply: pick item → DRAFT_PHASE; new idea → IDEA_INTAKE; concrete brief → DRAFT_PHASE direct. Edge cases (concrete-brief-on-first-message, empty BACKLOG): see `docs/ORCHESTRATION.md:11-37`.

## State machine (condensed — full spec in `docs/ORCHESTRATION.md`)
```
IDLE → DRAFT_PHASE (spawn architect DRAFT)
        → tầng==2 → APPROVAL_GATE → EXECUTE_PHASE
        → tầng==1 → CHALLENGE_PHASE (spawn worker CHALLENGE)
                    ├── no objections        → APPROVAL_GATE
                    └── objections           → RESPOND_PHASE (spawn architect RESPOND)
                                               ├── all resolved      → CHALLENGE_PHASE (Turn N+1)
                                               ├── any DEFER         → FORCE_ESCALATION
                                               └── Turn 3 reached    → FORCE_ESCALATION
APPROVAL_GATE → AskUserQuestion → approve / amend / abandon
EXECUTE_PHASE → spawn worker EXECUTE → DONE
```
Cap = 3 turns. Hit Turn 3 without consensus → FORCE_ESCALATION (`AskUserQuestion` to Sếp).

## Tier routing (P036)
Architect sets `Tầng: 1` or `Tầng: 2` in phiếu header. You branch:
- **Tầng 2** (lặt vặt, ≤3 files, ≤200 LOC, no schema/API/auth/dep): DRAFT → APPROVAL_GATE → EXECUTE. Skip CHALLENGE_PHASE entirely.
- **Tầng 1** (móng nhà): full debate flow.

Phiếu missing `Tầng:` field → reject, re-spawn Architect with explicit "set Tầng: 1 or 2".
Worker may escalate Tầng 2 → Tầng 1 mid-EXECUTE; you may NEVER demote Tầng 1 → Tầng 2.

## Trigger phrases (when spawning subagents)
| Target | Phrase to include in spawn prompt |
|---|---|
| Architect DRAFT | "Spawn architect viết phiếu cho X" / "plan X" |
| Architect RESPOND | "Architect respond to Debate Log Turn <N> in P<NNN>" |
| Worker CHALLENGE | "Worker challenge phiếu P<NNN>" |
| Worker EXECUTE | "Worker execute phiếu P<NNN>" |

## Marker file hygiene
`.sos-state/architect-active` gates the architect-guard hook. Before EVERY spawn:
- Spawn architect (any mode): `mkdir -p .sos-state && touch .sos-state/architect-active`
- Spawn worker (any mode): `rm -f .sos-state/architect-active`

Never leave a stale marker. Marker lives outside `.claude/` so YOLO mode does not prompt.

## Bulk input handling (P035)
When the user dumps N items NOT via `/idea` skill (e.g. pastes a list of 3+ ideas at once), you MUST:
a. Auto-classify each item: existing BACKLOG match → reference; new → `/idea` triage internally.
b. Append to `docs/BACKLOG.md` (Open backlog or Active sprint per priority).
c. Propose a wave order (which item first, which depends on which).
d. Run `AskUserQuestion` ONCE with the wave plan — options: approve / reorder / drop one / cancel.

You MUST NOT ask "pick item nào trước" before doing a-c. The user already delegated triage by dumping the list.

## Hard rules
1. **Approval gate is mandatory.** Even if Worker accepted V1 with zero objections, run `AskUserQuestion` before EXECUTE.
2. **No silent state.** Narrate every transition: "Worker raised 2 objections → spawning architect RESPOND."
3. **Debate trail in the phiếu file.** No external log. Audit = git history.
4. **Max 3 turns** before force-escalating.
5. **User can interrupt anytime.** State machine is suggestive, not enforced.
6. **One APPROVAL_GATE per phiếu.** Don't add fake-gates between DRAFT/CHALLENGE/RESPOND.
7. **Tier set in DRAFT, escalated up only.** Worker 2→1 escalation = OK; orchestrator 1→2 demotion = forbidden.
8. **Bulk input → auto-triage + 1 gate.** See "Bulk input handling" above.

## Anti-patterns
1. Coding yourself instead of spawning Worker.
2. Asking user "is this OK?" mid-state-machine.
3. Asking user to pick order/priority when "tùy em" was given.
4. Spawning Worker EXECUTE before APPROVAL_GATE.
5. Forgetting to flip the architect-active marker between spawns.
6. Treating bulk input as N separate decisions instead of 1 wave plan.
```

**Lưu ý:**
- Frontmatter has `name` + `description` + `tools: []` + `model: opus`. The `tools: []` empty allowlist is the safety mitigation per V2 frontmatter risk decision (Option C): if Claude Code subagent loader scans `agents/*.md` and registers this file, it cannot accidentally invoke any tools. The HTML comment under the fence documents intent for human readers.
- Total target length: ≤90 lines (heading + frontmatter + content + comment, between the two outer `---` fences). Architect counted 89 lines manually (V3 trim — 12 blank lines collapsed from V2's 101). Worker re-runs `wc -l agents/orchestrator.md` after Write; if over 90 → trim the Anti-patterns list to 5 entries (drop #5 marker hygiene, since it's covered in dedicated section). Anchor #12 tracks.
- Match `agents/architect.md` voice: imperative, mechanical, no filler.
- The "Bulk input handling" section uses a-d step labels (T2.1 fix) MUST stay consistent with `docs/ORCHESTRATION.md` rule #8 (Task 5).
- The state-machine ASCII (T2.3 / O3 fix) MUST keep `├──` / `└──` peer indentation under `CHALLENGE_PHASE` and `RESPOND_PHASE` so Turn-3 cap and DEFER read as siblings of "all resolved", not nested under it. Anchor #13 (V2) verifies the pattern source.
- **V3 whitespace style note:** the trimmed body intentionally omits the blank line AFTER each `##` heading (heading-to-content adjacency) to fit under ≤90 cap with content intact. Renders correctly under CommonMark; sibling files (`agents/architect.md`) keep the blanks because they have no line cap. If a future edit adds content to orchestrator.md, prefer trimming low-value lines first before re-introducing post-`##` blank lines.

### Task 2: Add `agents/orchestrator.md` reference to SessionStart banner

**File:** `scripts/session-start-banner.sh`

**Tìm:** (verified anchor #4 — exact text at lines 70-77; the 3-line replacement target is at lines 75-77)

```
echo "    Deferred tools MANDATORY (load đầu session, KHÔNG fallback markdown 1/2/3):"
echo "        ToolSearch select:AskUserQuestion,TaskCreate,TaskUpdate"
echo "    Spec đầy đủ: docs/ORCHESTRATION.md"
```

**Thay bằng:**

```
echo "    Deferred tools MANDATORY (load đầu session, KHÔNG fallback markdown 1/2/3):"
echo "        ToolSearch select:AskUserQuestion,TaskCreate,TaskUpdate"
echo "    Handbook: agents/orchestrator.md (~85 lines, condensed contract)"
echo "    Spec đầy đủ: docs/ORCHESTRATION.md"
```

**Lưu ý:**
- Surgical 1-line insert before existing `Spec đầy đủ` line. The replaced 3-line block lives at lines 75-77; the surrounding orchestrator-contract block spans lines 70-77 per anchor #4 (T2.3 clarifies V1 prose).
- Keep indentation (4 spaces) to match block.
- Verify Worker: `bash scripts/session-start-banner.sh` runs without error and the new line shows in output.

### Task 3: Add anti-pattern warnings to `INSTALL.md` Step 4 CLAUDE.md template

**File:** `INSTALL.md`

**Tìm:** (verified anchor #5 — fenced template ends at line 167-168)

```
5. Spawn worker (EXECUTE) → Task 0 → code → test → Discovery → commit
6. Chủ nhà nghiệm thu, deploy
```

**Thay bằng:**

```
5. Spawn worker (EXECUTE) → Task 0 → code → test → Discovery → commit
6. Chủ nhà nghiệm thu, deploy

**Anti-patterns (orchestrator MUST NOT):**
- **Không fake-gate giữa phase.** APPROVAL_GATE là gate user DUY NHẤT (trước EXECUTE). Đừng chèn "is this OK?" giữa DRAFT/CHALLENGE/RESPOND.
- **Không hỏi user pick/order khi đã được ủy quyền "tùy em".** Bulk input → auto-classify + propose wave + 1 AskUserQuestion duy nhất confirm wave plan.
- **Không tự code thay vì spawn Worker.** Main session = orchestrator, không phải executor. Code → spawn Worker EXECUTE.
- **Không skip marker hygiene.** `mkdir -p .sos-state && touch .sos-state/architect-active` trước spawn Architect; `rm -f .sos-state/architect-active` trước spawn Worker.

Full contract: `agents/orchestrator.md` (condensed, ~85 lines) + `docs/ORCHESTRATION.md` (full spec).
```

**Lưu ý:**
- Insert AFTER line 167 (the existing "6. Chủ nhà nghiệm thu, deploy" line), INSIDE the fenced markdown block (template ends with the closing ` ``` ` on line 168). Keep the closing fence intact — new content goes BEFORE the closing fence.
- 4 anti-pattern bullets total (matches brief: bullet 1 = fake-gate, bullet 2 = pick/order delegation, bullets 3-4 = additional drift modes worth surfacing).
- Voice: Vietnamese (matches existing template language).

### Task 4: Add contributor pointer to sos-kit's `CLAUDE.md`

**File:** `CLAUDE.md`

**Tìm:** (verified anchor #7 — line 105 region)

```
### Edit docs
- `README.md` — any tool/skill/integration table MUST match actual folders and binaries. Contributor onboarding breaks if they drift.
- `docs/PHILOSOPHY.md` — stable. Don't add a 6th principle without strong justification. The 5 principles are load-bearing.
- `docs/SETUP.md` — must match real binary names and `cargo install` instructions.
```

**Thay bằng:**

```
### Edit orchestrator behavior (`agents/orchestrator.md` + `docs/ORCHESTRATION.md`)
1. `agents/orchestrator.md` is the condensed handbook (~85 lines, ≤90 cap) — system-prompt contract for the main session in every sos-kit project. Keep terse + imperative.
2. `docs/ORCHESTRATION.md` is the full spec (state machine, failure modes, concrete example session). When changing state machine logic, update BOTH.
3. If you add a new orchestrator hard rule, mirror it as a one-liner in `agents/orchestrator.md` "Hard rules" section AND a fuller entry in `docs/ORCHESTRATION.md` "Hard rules".
4. SessionStart banner (`scripts/session-start-banner.sh`) references both files — verify the banner still surfaces them after edit.

### Edit docs
- `README.md` — any tool/skill/integration table MUST match actual folders and binaries. Contributor onboarding breaks if they drift.
- `docs/PHILOSOPHY.md` — stable. Don't add a 6th principle without strong justification. The 5 principles are load-bearing.
- `docs/SETUP.md` — must match real binary names and `cargo install` instructions.
- `docs/ORCHESTRATION.md` — orchestrator full spec; condensed handbook at `agents/orchestrator.md` mirrors hard rules. Edit both together.
```

**Lưu ý:**
- Insert NEW sub-section "Edit orchestrator behavior" BEFORE existing "Edit docs" sub-section (line 105).
- Append 1 bullet to existing "Edit docs" list mentioning `docs/ORCHESTRATION.md` for completeness (covers anchor #6 finding that ORCHESTRATION.md is missing from CLAUDE.md tree — partial fix; full tree refresh is separate Open backlog item per BACKLOG.md line 92).

### Task 5: Add Hard rule #8 (bulk input) to `docs/ORCHESTRATION.md`

**File:** `docs/ORCHESTRATION.md`

**Tìm:** (verified anchor #2 — last existing rule line 116)

```
7. **Tier is set in DRAFT, escalated up only.** Architect's `Tầng` declaration in the phiếu header is the routing key. Worker may escalate Tầng 2 → Tầng 1 mid-EXECUTE with `file:line` evidence; orchestrator may NOT silently demote Tầng 1 → Tầng 2. Phiếu missing the `Tầng` field is rejected pre-spawn — orchestrator re-spawns Architect with explicit "set Tầng: 1 or 2" instruction.

## Failure modes + recovery
```

**Thay bằng:**

```
7. **Tier is set in DRAFT, escalated up only.** Architect's `Tầng` declaration in the phiếu header is the routing key. Worker may escalate Tầng 2 → Tầng 1 mid-EXECUTE with `file:line` evidence; orchestrator may NOT silently demote Tầng 1 → Tầng 2. Phiếu missing the `Tầng` field is rejected pre-spawn — orchestrator re-spawns Architect with explicit "set Tầng: 1 or 2" instruction.
8. **Bulk input → auto-triage + ONE gate.** When user dumps N items not via `/idea` skill (paste a list of 3+ ideas in one message), orchestrator MUST: (a) auto-classify each item (existing BACKLOG match → reference; new → triage as if `/idea` ran internally); (b) append to `docs/BACKLOG.md` under correct section (Open backlog or Active sprint per priority); (c) propose a wave order with rationale; (d) run `AskUserQuestion` ONCE with the wave plan (options: approve / reorder / drop one / cancel). Orchestrator MUST NOT ask "pick item nào trước" before steps a-c — the user already delegated triage by bulk-dumping. Only re-prompt on (d). Failure mode: orchestrator asks the user to pick order before classifying → violates delegation; recovery = redo the auto-triage step then run gate (d).

## Failure modes + recovery
```

**Lưu ý:**
- Insert new rule #8 AFTER existing rule #7 line, BEFORE the `## Failure modes + recovery` heading (verified anchor #2 confirms heading immediately follows rule 7).
- Cross-references `agents/orchestrator.md` "Bulk input handling" section (Task 1) — must stay in sync. Both use a-d step labels (T2.1 fix).
- Voice: English (matches `docs/ORCHESTRATION.md`).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `agents/orchestrator.md` | Task 1 — NEW file, ~85-line condensed orchestrator handbook (≤90 cap) |
| `scripts/session-start-banner.sh` | Task 2 — 1-line insert referencing `agents/orchestrator.md` |
| `INSTALL.md` | Task 3 — append 4 anti-pattern bullets inside Step 4 CLAUDE.md template |
| `CLAUDE.md` (sos-kit root) | Task 4 — new "Edit orchestrator behavior" sub-section + 1 bullet in "Edit docs" |
| `docs/ORCHESTRATION.md` | Task 5 — add Hard rule #8 "Bulk input → auto-triage + ONE gate" |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `agents/architect.md` | Frontmatter format unchanged, no orchestrator content leaks here |
| `agents/worker.md` | Same — orchestrator handbook is sibling, not subagent contract |
| `phieu/TICKET_TEMPLATE.md` | `Tầng:` field still required (P036) — orchestrator.md references it |
| `bin/sos.sh` | NOT the SessionStart banner; do not touch |
| `.claude/settings.json` | Hook config unchanged; banner script content edit is sufficient |
| `phieu/`, `skills/`, `hooks/`, `integrations/`, `configs/` | Out of scope |

---

## Luật chơi (Constraints)

1. `agents/orchestrator.md` MUST be ≤90 lines (target ~85 per V2 trim; V3 sits at 89). Worker tracks line count via `wc -l` in Discovery Report. If over 90 → Tầng 2 self-trim per Task 1 Lưu ý fallback (drop Anti-patterns #5 first).
2. Frontmatter on `agents/orchestrator.md` has `name` + `description` + `tools: []` + `model: opus` (V2 — Option C). The `tools: []` empty allowlist is intentional safety: if Claude Code subagent loader scans `agents/*.md` and tries to register this file, the empty allowlist prevents any tool capability. Worker MUST NOT remove these fields. If Worker discovers Claude Code DOES try to invoke `orchestrator` as a subagent → STOP, escalate Tầng 1 (loader semantics need rework).
3. Voice in `agents/orchestrator.md`: English (mirrors `agents/architect.md` system prompt voice). Body of `INSTALL.md` Step 4 anti-patterns: Vietnamese (matches existing template). `docs/ORCHESTRATION.md` rule #8: English. `CLAUDE.md`: English.
4. Bulk-input rule wording in Task 1 (orchestrator.md) and Task 5 (ORCHESTRATION.md rule #8) MUST be consistent — same enumerated steps a-d (V2 T2.1 fix), same "MUST NOT ask pick order before steps a-c" clause. If Worker finds drift between the two — Tầng 2 self-fix to make ORCHESTRATION.md the canonical wording, mirror to orchestrator.md.
5. Anchor #1 stays `[needs Worker verify]` — that is the dogfood test (P036 acceptance: P037 was supposed to dogfood; brief redirects to P035 anchor #1). Do NOT silently upgrade to `[verified]`.
6. State-machine ASCII in Task 1 orchestrator.md MUST mirror ORCHESTRATION.md lines 55-59 branch structure (V2 O3 fix): DEFER + Turn-3 cap branch peer-level from RESPOND_PHASE, NOT nested under "resolved". Anchor #13 records the canonical pattern.

---

## Nghiệm thu

### Automated
- [ ] `bash scripts/session-start-banner.sh` exits 0 and prints the new "Handbook: agents/orchestrator.md" line in the orchestrator-contract block.
- [ ] `wc -l agents/orchestrator.md` ≤ 90 lines.
- [ ] `grep -c "agents/orchestrator.md" scripts/session-start-banner.sh` = 1 (one new reference added).
- [ ] `grep -c "Bulk input" docs/ORCHESTRATION.md` ≥ 1 (new rule #8 present).
- [ ] `grep -c "Bulk input handling" agents/orchestrator.md` = 1 (mirror section present).
- [ ] `grep -c "Session opening" agents/orchestrator.md` = 1 (V2 O2 fix — session-opening section present).
- [ ] `grep -c "tools: \[\]" agents/orchestrator.md` = 1 (V2 frontmatter safety field present).

### Manual Testing
- [ ] **Bulk-input dry-run procedure** (procedural acceptance — describe in Discovery Report; no live test required for this phiếu since orchestrator behavior is contract-defined, not code-tested):
  1. Hypothetical: in a fresh Claude Code session in a sos-kit project, paste "Em có 3 idea: A, B, C — anh tùy em xếp wave."
  2. Expected per rule #8: orchestrator (a) classifies A/B/C against BACKLOG, (b) appends to BACKLOG, (c) proposes wave order, (d) runs ONE AskUserQuestion. Does NOT ask "pick A/B/C trước" before step (d).
  3. Discovery Report notes whether the rule wording is unambiguous enough for the orchestrator to follow on first read.
- [ ] Cold-read test: open `agents/orchestrator.md` in isolation (no other context) — does it contain enough to drive the state machine correctly, including session-opening behavior (V2 O2 fix)? If a section feels ambiguous → flag in Discovery.
- [ ] State-machine cold-read: with no other context, does the ASCII diagram clearly show DEFER + Turn-3 cap as RESPOND_PHASE peer branches (V2 O3 fix)? Compare visually to ORCHESTRATION.md lines 55-59.

### Regression
- [ ] `agents/architect.md` + `agents/worker.md` content unchanged (`git diff agents/architect.md agents/worker.md` empty after this phiếu).
- [ ] SessionStart banner still runs successfully on a project where `docs/BACKLOG.md` exists (existing tests in INSTALL.md Step 5 still pass).
- [ ] Existing 7 hard rules in `docs/ORCHESTRATION.md` unchanged — only rule #8 appended.

### Docs Gate
- [ ] `CHANGELOG.md` — entry for P035 (1-line: "Orchestrator handbook (`agents/orchestrator.md`) + bulk-input rule + INSTALL anti-patterns").
- [ ] `docs/BACKLOG.md` — move P035 from Active sprint to Recently shipped (per maintenance rule 2).
- [ ] `agents/orchestrator.md` exists at `agents/orchestrator.md`. Verify by `Glob agents/*.md` lists 3 files.

### Discovery Report
- [ ] Append entry to `docs/DISCOVERIES.md` (newest on top):
  - **Assumptions in phiếu — CORRECT:** which anchors held (#2-#9, #11, #13 expected ✅).
  - **Assumptions in phiếu — WRONG:** if any anchor mismatched (especially #1, #10, #12 which were `[needs Worker verify]`).
  - **Tầng 2 self-adapts:** if anchor #1 found a collision and Worker chose a fallback name → log here. If line count target (#12) drove a content trim → log specific section trimmed.
  - **Edge cases / limitations:** Did the orchestrator-handbook condensation lose any load-bearing rule from `docs/ORCHESTRATION.md`? Worker reads both side by side as the final QA gate.
  - **Marker dogfood note:** Confirm anchor #1 stayed `[needs Worker verify]` end-to-end — this is the P036 acceptance dogfood, log explicit "Yes, anchor #1 stayed [needs Worker verify], Worker grep-verified at EXECUTE time and found `<result>`."
  - **V2 RESPOND note:** Confirm V2 closed all 3 Worker objections (O1.1/O1.2/O1.3) cleanly + frontmatter safety Option C did not break subagent loader. If frontmatter `tools: []` caused loader error → escalate Tầng 1.
  - **V3 RESPOND note:** Confirm V3 closed O2.1 (line count) — `wc -l agents/orchestrator.md` returned ≤90. If over 90 despite V3 trim → escalate Tầng 1 (whitespace audit drift).
  - **Docs updated to match reality:** None expected (this phiếu IS the doc update). If Worker found stale doc references during EXECUTE → list them.
