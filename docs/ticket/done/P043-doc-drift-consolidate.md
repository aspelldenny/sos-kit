# PHIẾU P043: Doc drift consolidate — Quản đốc persona codify + alignment engineering + deferred-tool loading

> **Loại:** Docs (foundation doc sweep — persona naming, philosophy rationale, orchestrator spec rewrite, deferred-tool loading instruction)
> **Ưu tiên:** P1 (resolves inconsistency from 2026-05-25 inline edit; foundation for tarot port wave 1; P041 + P042 reference Quản đốc persona)
> **Tầng:** 1 (móng nhà — touches LAYERS / PHILOSOPHY / ORCHESTRATION / HANDOFF / README / CLAUDE / agents/orchestrator — ripple wide; persona naming = contract for downstream subagent docs)
> **Ảnh hưởng:** `docs/LAYERS.md`, `docs/PHILOSOPHY.md`, `docs/ORCHESTRATION.md`, `docs/HANDOFF.md`, `README.md`, `CLAUDE.md`, `agents/orchestrator.md` (+ commits 2 dangling staged files)
> **Dependency:** None (parallel-safe với P041/P042; chỉ touches docs, không touches `bin/sos.sh` / `scripts/parsers/` / `agents/architect.md` / `agents/worker.md`)

---

## Context

### Vấn đề hiện tại

Sprint "Tarot port wave 1" trigger 2026-05-25 sau khi tarot dogfood evolution (P281-287 + Quản đốc rename) reveal 3 lỗ hổng trong sos-kit canon docs:

1. **Persona inconsistency (urgent).** Sếp đã inline-edit `agents/orchestrator.md:9` + `:21` đổi main-session persona `Kiến trúc sư` → `Quản đốc` (commit pending, file staged). Các doc còn lại VẪN nói "Kiến trúc sư persona for the orchestrator" — đặc biệt `docs/ORCHESTRATION.md:34-37` và `:18` (greeting script literal `"Em là Kiến trúc sư project <name>"`). Inconsistency này gây confusion: subagent `Kiến trúc sư` (sandboxed Read/Write/Glob, no Bash) khác với main-session persona — nếu cả 2 cùng tên thì người đọc handbook không phân biệt được "thằng nào nói".
2. **Philosophy gap.** Tarot's `PHILOSOPHY.md` có subsection "alignment engineering / information envelopes" giải thích sâu hơn vì sao role separation chống hallucination LLM. sos-kit có hint ở Principle 6 + Anti-pattern #1 + bullet trong `docs/PHILOSOPHY.md:40-48` ("The deeper principle: information envelopes") nhưng KHÔNG có dedicated subsection sau Principle 6 với tarot-quality framing. Port subsection để rationale rõ ràng cho contributor đọc.
3. **Orchestrator spec underspec.** Tarot's `agents/orchestrator.md` (141 lines, theo BACKLOG spec) có 3 sections sos-kit thiếu: (a) greeting turn template chi tiết hơn 1 đoạn 5-line, (b) tier priority routing logic narrative (vì sao Tầng 2 skip CHALLENGE), (c) session opening script step-by-step. sos-kit hiện có version condensed ở `agents/orchestrator.md:19-23` + `docs/ORCHESTRATION.md:11-37` nhưng thiếu depth.
4. **Deferred-tool loading gap (extra, Sếp added 2026-05-25).** Future Claude Code sessions: Quản đốc/Architect/Worker cần `AskUserQuestion` + `TaskCreate` + `TaskUpdate` ngay từ turn 1 — nhưng những tools này là **deferred** (không auto-load). Direct call → `InputValidationError: tool not loaded`. Workaround manual: invoke `ToolSearch query="select:AskUserQuestion,TaskCreate,TaskUpdate"` ở session start. Instruction này phải persist trong `agents/orchestrator.md` + `CLAUDE.md` để mọi session future tự load không cần Sếp nhắc.
5. **Dangling uncommitted state.** Hiện trên `main` có 2 files staged chưa commit:
   - `agents/orchestrator.md` — Sếp inline-edit line 9 + 21 (`Kiến trúc sư` → `Quản đốc`).
   - `docs/BACKLOG.md` — Quản đốc merge resolve: wave 1 sprint Active, P040 SHIPPED row, paused harvest sprint, Recently shipped entry P040.

   Worker EXECUTE P043 sẽ commit ALL changes (uncommitted staged + new doc edits trong phiếu này) thành 1 PR — không để rotted state trên main.

### Giải pháp

6 surgical doc edits (5 tasks + 1 verification anchor task), all docs-only, no code:

1. **Persona codify (Task 1).** Đổi "Kiến trúc sư persona" → "Quản đốc persona" CHỈ ở chỗ chỉ **main-session orchestrator persona** (UX framing). KHÔNG blanket-rename. Architect SUBAGENT (the sandboxed Read/Write/Glob role) VẪN tên "Kiến trúc sư" — đó là 3-role model identity, không đổi.
2. **LAYERS Layer 0 (Task 2).** Thêm Layer 0 = Quản đốc row vào access matrix + 3-layer ASCII diagram. Quản đốc = main session, spawn-only, no code edit, no vision-doc write.
3. **PHILOSOPHY alignment engineering subsection (Task 3).** Thêm subsection sau Principle 6 port từ tarot's PHILOSOPHY.md (Architect không Read được tarot — port từ skeleton trong phiếu hoặc `[needs Worker verify]`).
4. **ORCHESTRATION rewrite (Task 4).** Rewrite `docs/ORCHESTRATION.md:34-37` "Why Quản đốc persona" + thêm 3 subsections (greeting turn, tier priority routing rationale, session opening script). Port từ tarot's 141-line orchestrator.md — Architect không Read được, port skeleton + `[needs Worker verify]`.
5. **Deferred-tool loading instruction (Task 5).** Thêm section "Deferred-tool loading" vào `agents/orchestrator.md` + `CLAUDE.md`. List mandatory tools: AskUserQuestion (routing/escalation), TaskCreate/TaskUpdate (sprint tracking). Rationale: deferred tools không auto-loaded, direct call fails. **Sếp decision 2026-05-25: raise `CLAUDE.md:149` cap ≤90 → ≤105** (deferred-tool section ~12 lines; compressing elsewhere creates churn).
6. **Cross-ref pass (Task 6).** README.md tables + CLAUDE.md repo structure + HANDOFF.md persona references — confirm "Quản đốc" reflected everywhere main-session persona referenced, "Kiến trúc sư" preserved everywhere subagent referenced.

### Scope

- CHỈ sửa:
  - `docs/LAYERS.md` — Task 2 (Layer 0 row in matrix + ASCII diagram)
  - `docs/PHILOSOPHY.md` — Task 3 (alignment engineering subsection after Principle 6)
  - `docs/ORCHESTRATION.md` — Task 4 (rewrite `:34-37` + add 3 subsections, persona refs at `:18-23`)
  - `docs/HANDOFF.md` — Task 6 (cross-ref persona references in Handoff 0 + Handoff 2.5 chỗ "orchestrator")
  - `README.md` — Task 6 (subagent table, Pipeline diagram, vision docs section persona refs)
  - `CLAUDE.md` — Task 5 + Task 6 (deferred-tool loading section + repo structure note + persona ref consistency + line 149 cap raise ≤90 → ≤105)
  - `agents/orchestrator.md` — Task 5 (deferred-tool loading section) + Task 1 confirm staged edits intact
- KHÔNG sửa:
  - `agents/architect.md` (subagent file — persona "Kiến trúc sư" preserved; out of scope per BACKLOG spec)
  - `agents/worker.md` (subagent file — out of scope)
  - `phieu/TICKET_TEMPLATE.md`, `phieu/RELAY_PROTOCOL.md`, `phieu/DISCOVERY_PROTOCOL.md`, `phieu/AUDIT_PROTOCOL.md`
  - Bất kỳ `phieu/done/*.md` archive (historical phiếu — không rewrite history)
  - `bin/sos.sh`, `scripts/*.sh`, `hooks/pre-commit`
  - `skills/**/SKILL.md` — out of scope (P045 sẽ handle skill drift sweep)
  - `docs/BACKLOG.md` — Quản đốc đã edit (staged), Worker commit nguyên trạng không edit thêm
  - Tier system (Tầng 1 / Tầng 2 rules) — không touched
  - Out-of-wave phiếu: P041, P042, P044, P045, P046, P047

---

## Task 0 — Verification Anchors

> **Architect humility note:** Architect Read `docs/LAYERS.md` + `docs/PHILOSOPHY.md` + `docs/ORCHESTRATION.md` + `docs/HANDOFF.md` + `README.md` + `CLAUDE.md` + `agents/orchestrator.md` + `phieu/done/P040-bootstrap-stack-detection.md` (giọng reference) end-to-end trong DRAFT session 2026-05-25. All `[verified]` anchors from real Read. `[needs Worker verify]` items: (a) tarot file content (Architect không có access tarot repo từ sos-kit envelope), (b) cross-doc string greps (Architect không có Grep tool).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `docs/LAYERS.md` access matrix table at lines 19-26 currently has 3 columns: Chủ nhà / Kiến trúc sư / Thợ. Adding Layer 0 = adding a 4th column (Quản đốc) OR adding a new row labeled "Quản đốc (Layer 0)" depending on Worker's structural choice. | `sed -n '19,30p' docs/LAYERS.md` | ✅ [verified] — Architect Read 2026-05-25, table at lines 19-26, 3 columns confirmed. Architect recommends ROW addition (cleaner — see Task 2 spec). |
| 2 | `docs/LAYERS.md:35-78` ASCII art has 3 boxes: Layer 1 Chủ nhà / Layer 2 Kiến trúc sư / Layer 3 Thợ. Layer 0 Quản đốc box must be added ABOVE Layer 1 (chronological: Quản đốc orchestrates first). | `sed -n '35,78p' docs/LAYERS.md` | ✅ [verified] — Architect Read 2026-05-25, 3-layer ASCII confirmed at exactly those lines. |
| 3 | `docs/PHILOSOPHY.md:40-48` already has subsection "The deeper principle: information envelopes (alignment engineering)" — short version of what BACKLOG asks to port. Decision: EXPAND existing subsection vs. ADD new one after Principle 6. Architect recommends EXPAND (avoid duplication) + add subheading + worked example. | `sed -n '40,75p' docs/PHILOSOPHY.md` | ✅ [verified] — Architect Read 2026-05-25; existing subsection present at lines 40-48, principles at lines 50-75. Worker confirms expand-vs-replace at EXECUTE. |
| 4 | `docs/ORCHESTRATION.md:34-37` currently subsection "Why 'Kiến trúc sư' persona for the orchestrator" with 3 bullet rationale. To rewrite as "Why Quản đốc persona" — preserve rationale skeleton, swap persona name + adjust framing. | `sed -n '34,38p' docs/ORCHESTRATION.md` | ✅ [verified] — Architect Read 2026-05-25, exact 3-bullet rationale at lines 34-37 confirmed. |
| 5 | `docs/ORCHESTRATION.md:18-23` greeting script literal contains `Em là Kiến trúc sư project <name>` — must change to `Em là Quản đốc project <name>`. Same fix needed at line 32 ("Em là Kiến trúc sư. BACKLOG chưa có item nào..."). | `sed -n '17,33p' docs/ORCHESTRATION.md` | ✅ [verified] — Architect Read 2026-05-25, greeting script at lines 18-22, edge-case fallback at line 32 both reference "Kiến trúc sư" persona literal. |
| 6 | `agents/orchestrator.md:9 + :21` ALREADY edited by Sếp inline 2026-05-25 — line 9 says "surfacing as **Quản đốc**", line 21 says "as Quản đốc". File is currently STAGED (uncommitted on `main`). Worker confirms via `git status` and `git diff --cached agents/orchestrator.md`. | `git status` + `git diff --cached agents/orchestrator.md` | ✅ [verified] — Architect Read 2026-05-25 reading current file state shows "Quản đốc" at lines 9 + 21 (staged state). Confirmed via session git status: `M agents/orchestrator.md`. |
| 7 | `docs/BACKLOG.md` ALREADY edited by Quản đốc — wave 1 sprint Active, P040 SHIPPED row at line 16, paused harvest sprint at line 28, recently shipped P040 entry at line 130. File is STAGED. Worker must commit nguyên trạng KHÔNG edit thêm trong P043. | `git status` shows `M docs/BACKLOG.md` | ✅ [verified] — Architect Read 2026-05-25, BACKLOG content matches spec (wave 1 Active + P040 SHIPPED + recently shipped row). Session git status confirms `M`. |
| 8 | Tarot's `~/tarot/docs/PHILOSOPHY.md` "alignment engineering" subsection content. Architect attempted Read but Architect envelope blocks reads outside sos-kit repo (no path access to `~/tarot/`). Skeleton + `[needs Worker verify]` deferred to Worker. | (Architect cannot Read tarot files) | ⚠️ [Architect cannot verify] — Worker EXECUTE Read `~/tarot/docs/PHILOSOPHY.md` if access exists; otherwise Worker WRITES subsection from skeleton in Task 3 + flags in Discovery Report. **Skeleton provided in Task 3 below** — sufficient if tarot Read fails. |
| 9 | Tarot's `~/tarot/agents/orchestrator.md` 141-line file — greeting turn + tier priority + session opening script content. Architect cannot Read. Skeleton + `[needs Worker verify]` for Worker. | (Architect cannot Read tarot files) | ⚠️ [Architect cannot verify] — Worker EXECUTE Read tarot's orchestrator.md if access. Architect provides skeleton structure (3 subsection headings + bullet placeholders) in Task 4 — Worker port concrete content from tarot OR fill skeleton từ tarot dogfood memory bank entries Sếp can paste. |
| 10 | `README.md` "Components" section line 105-114 has subagent table with rows for `architect` + `worker` (subagents). Persona refs in `README.md:9-14` (Why section: "Chủ nhà / Kiến trúc sư / Thợ"). No Layer 0 Quản đốc mention currently. Cross-ref needed: add Layer 0 reference in Why section + (optional) in Pipeline diagram. | `sed -n '8,15p' README.md` + `sed -n '105,115p' README.md` | ✅ [verified] — Architect Read 2026-05-25, exact lines confirmed. README currently only documents 3 layers — adding Layer 0 mention (NOT a new role, just naming the orchestrator persona) ý nghĩa cho new readers. |
| 11 | `CLAUDE.md:8` says "**3-role workflow** for one-person software teams: **Chủ nhà** ... **Kiến trúc sư** ... **Thợ**". Layer 0 Quản đốc = main session orchestrator = NEW persona but NOT new role (still 3-role model). CLAUDE.md needs subtle reframe: "3-role workflow + orchestrator (Quản đốc) main session". | `sed -n '6,10p' CLAUDE.md` | ✅ [verified] — Architect Read 2026-05-25. Reframe is Tầng 1 wording (one-line edit). |
| 12 | `docs/HANDOFF.md:6-7` says "Kiến trúc sư (Claude Web Project) and Thợ (Claude Code) are **separate sessions**". In v2.1 Subagent mode, orchestrator (Quản đốc) is the relay — already documented at line 87-130 (Handoff 2.5 Architect ↔ Worker debate). HANDOFF persona refs to "orchestrator" need Quản đốc label introduced once (e.g. Handoff 2.5 mention "main session orchestrator (Quản đốc)"). | `sed -n '86,95p' docs/HANDOFF.md` | ✅ [verified] — Architect Read 2026-05-25, Handoff 2.5 at lines 87-130 mentions "orchestrator" 4 times without persona label. One-time label add suffices. |
| 13 | `agents/orchestrator.md` current line count ≤90 per `CLAUDE.md:149` "Keep terse + imperative" rule. Adding "Deferred-tool loading" section may push over 90 if verbose. **Sếp DECIDED 2026-05-25: raise cap ≤90 → ≤105** (per V2 update — see Task 5). | `wc -l agents/orchestrator.md` | ✅ [verified + DECIDED] — Architect Read full file 2026-05-25, file ends at line 91. Adding ~12-line deferred-tool section pushes to ~103. Sếp confirmed cap raise to ≤105 (Option A in V2 update); Worker executes concrete `CLAUDE.md:149` edit in Task 5 sub-step. |
| 14 | `CLAUDE.md` insertion point for deferred-tool instruction: most natural location = new subsection under "## Maintainer-only conventions" (line 192) OR new top-level section between "## Language" (186) and "## Maintainer-only conventions" (192). Architect recommends: NEW top-level section "## Deferred-tool loading (Claude Code session start)" before "## Maintainer-only conventions" — discoverable, not buried under maintainer-only. | `sed -n '186,199p' CLAUDE.md` | ✅ [verified] — Architect Read 2026-05-25, structure confirmed. Worker self-decide exact heading wording (Tầng 2). |
| 15 | "Quản đốc" is the SAME orchestrator role specified in `agents/orchestrator.md`. NOT a 4th role addition. Layer 0 in LAYERS.md = NAMING the persona for what main session already does (spawn subagents, drive state machine). Confirms via `agents/orchestrator.md:8-9` "You are the **main Claude Code session** ... **4th role: Orchestrator**". | `sed -n '7,10p' agents/orchestrator.md` | ✅ [verified] — Architect Read 2026-05-25. P043 = naming consistency, not new role introduction. |

**Summary:** 13 ✅ verified + 2 ⚠️ (Anchor #8 + #9: tarot file content — Worker port if access, else use Architect skeletons in Task 3 / Task 4). No ❌. Tier 1 — Worker MUST CHALLENGE before EXECUTE per ORCHESTRATION.md Hard rule #7.

### Pre-phiếu snapshot (Worker auto first-step)

> Worker EXECUTE FIRST ACTION (before any doc edit, before Task 0 grep verification): snapshot for rollback.

```bash
# Run from worktree root for P043:
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/ — auto-cleaned on phieu-done"
```

If P043 hits ❌ mid-execute: `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (within phiếu worktree only).

---

## Debate Log

> Tầng 1 phiếu — Worker MUST CHALLENGE before EXECUTE. Persona naming = contract (P041/P042 + future phiếu reference). 2 ⚠️ anchors (tarot file Read failures) — Worker CHALLENGE will report whether tarot content available; if not, Architect skeletons used.

**Phiếu version:** V2 (Architect RESPOND mode 2026-05-25 — concrete Thay bằng for CLAUDE.md:149 cap raise added per Worker CHALLENGE; Task 5 + validate threshold + Lưu ý promoted to DECIDED)

### Turn 1 — Worker Challenge

*(Worker fills this when invoked in CHALLENGE mode. If no objections, write "Worker accepted V1 — no challenges. Ready for Chủ nhà approval." and skip to Final consensus.)*

**Anchor verification (recap from Task 0):**
- 13/15 anchors clean (Quản đốc edits, ORCHESTRATION:34-37, greeting script, scope). 2 ⚠️ deferred (tarot file Read).

**Objections (Tầng 1 only — phiếu cần sửa):**
- [O1.1] Sếp approved option (b) — raise CLAUDE.md cap ≤90 → ≤105 — but the phiếu has NO concrete Thay bằng block for this edit. Task 5 leaves it as Architect "recommendation" in Lưu ý, not Worker instruction. Worker hitting `wc -l ≤100` validate will fail (~103 lines post-insert) with no explicit instruction to edit CLAUDE.md:149. Architect must add concrete Thay bằng: find `(~85 lines, ≤90 cap)` in `CLAUDE.md:149`, replace with `(~85 lines, ≤105 cap)`, so Worker has unambiguous instruction + validate check updated to `wc -l agents/orchestrator.md ≤105`.

**Proposed alternatives** (Worker recommends 1):
- A. Add concrete Thay bằng block in Task 5 for `CLAUDE.md:149` cap raise; update validate threshold to ≤105; promote Lưu ý "recommendation" to DECIDED (Sếp 2026-05-25).

**Status:** ✅ ADDRESSED BY ARCHITECT V2

### Turn 1 — Architect Response (phiếu V2)

- [O1.1] → **ACCEPT** → Added concrete Thay bằng block in Task 5 (new sub-step "Tìm 3 / Thay bằng 3" targeting `CLAUDE.md:149`). Updated Task 5 Validate threshold from `≤100` to `≤105`. Rewrote Task 5 Lưu ý "Architect recommendation: option (b)" → **DECIDED (Sếp 2026-05-25): Option A — raise cap to ≤105**. Removed Worker-self-decide ambiguity. Removed Task 6b `CLAUDE.md:149` parenthetical "if Task 5 option (a)" — only Option A path remains.

**Status:** ✅ RESPONDED — phiếu bumped to V2

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date] — code execution may begin

---

## Nhiệm vụ

> Worker order: Task 0 snapshot → Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6 → Nghiệm thu. Tasks 1-5 are independent file edits; Task 6 is cross-ref pass that confirms everything aligns.

### Task 1: Persona codify — sync "Kiến trúc sư" → "Quản đốc" for **main session persona ONLY**

**WARNING (do not blanket-rename):** Architect SUBAGENT (file `agents/architect.md`, role "Kiến trúc sư" in 3-role model) stays "Kiến trúc sư". Out of scope for P043. Only edit chỗ nói về **main-session orchestrator persona** (the visible role surfacing to Sếp in chat = Quản đốc).

**Files affected by Task 1:**
- `agents/orchestrator.md` — already edited by Sếp inline 2026-05-25, lines 9 + 21 say "Quản đốc". Worker confirm staged state intact (no rollback). NO new edits in Task 1 to this file (Task 5 will add deferred-tool section).
- `docs/ORCHESTRATION.md` — Task 4 covers (rewrites line 18-23 greeting + 34-37 rationale + line 32 edge case).
- `docs/HANDOFF.md` — Task 6 covers (one-time persona label add).
- `README.md` — Task 6 covers (Why section reframe + add Layer 0 mention).
- `CLAUDE.md` — Task 6 covers (line 8 reframe).
- `docs/LAYERS.md` — Task 2 covers (Layer 0 = Quản đốc row + ASCII box).
- `docs/PHILOSOPHY.md` — Task 3 covers (no persona rename, but principle 6 may reference Quản đốc for full 4-role coverage).

**Validate (Task 1):**
- `grep -rn "Kiến trúc sư" agents/architect.md` returns ≥1 (subagent identity preserved).
- `grep -n "Kiến trúc sư persona" docs/ORCHESTRATION.md` returns 0 (rewritten in Task 4).
- `git diff --cached agents/orchestrator.md` shows Sếp's 2-line edits intact (lines 9 + 21 = "Quản đốc").
- `grep -n "Em là Kiến trúc sư" docs/ORCHESTRATION.md` returns 0 (greeting script updated in Task 4).

**Lưu ý:** Task 1 itself = NO edits (it's a coordination task — orchestrating which subsequent task touches what). The discriminator "main-session persona vs subagent persona" is THE rule Worker applies in Task 4 + Task 6. If Worker finds ambiguous reference (e.g. "Kiến trúc sư reads docs" — could be either), default = **subagent**, do not rename. Escalate via Discovery Report if uncertain.

---

### Task 2: `docs/LAYERS.md` — add Layer 0 Quản đốc

**File:** `docs/LAYERS.md`

**Tìm 1** (access matrix at line 19-26):

```markdown
## Access matrix — who can see what

| | Chủ nhà | Kiến trúc sư | Thợ |
|---|---|---|---|
| Vision/strategy docs (PROJECT, SOUL, CHARACTER*) | ✏️ maintain | 📖 read | 📖 read |
| Code (src/, tests/) | 📖 read optional | ❌ **NO access** | ✏️ read+edit |
| Tickets (phiếu) | 📖 read, approve | ✏️ write | 📖 read, execute |
| Discovery Reports | 📖 read | 📖 read before next phiếu | ✏️ write |
| Running commands (bash, pnpm, git) | ❌ delegates | ❌ cannot | ✏️ runs |
| Skills (`/frontend-design`, `/security-review`, etc.) | ❌ delegates | ❌ NO access | ❌ NO access |
```

**Thay bằng / Thêm 1** — add a 4th column "Quản đốc" (orchestrator persona, main session, Layer 0) between Chủ nhà and Kiến trúc sư:

```markdown
## Access matrix — who can see what

| | Chủ nhà | Quản đốc (Layer 0, main session) | Kiến trúc sư | Thợ |
|---|---|---|---|---|
| Vision/strategy docs (PROJECT, SOUL, CHARACTER*) | ✏️ maintain | 📖 read (briefing context only) | 📖 read | 📖 read |
| Code (src/, tests/) | 📖 read optional | ❌ **NO access** (spawn-only) | ❌ **NO access** | ✏️ read+edit |
| Tickets (phiếu) | 📖 read, approve | 📖 read, route between subagents | ✏️ write | 📖 read, execute |
| Discovery Reports | 📖 read | 📖 read (next-phiếu briefing) | 📖 read before next phiếu | ✏️ write |
| Running commands (bash, pnpm, git) | ❌ delegates | ⚠️ marker file ops only (mkdir/touch/rm `.sos-state/`) | ❌ cannot | ✏️ runs |
| Skills (`/frontend-design`, `/security-review`, etc.) | ❌ delegates | ✏️ invoke (Orchestrator-only per P005) | ❌ NO access | ❌ NO access |
```

**Tìm 2** — line 28 "Skills note" subsection just below access matrix, currently says:

```
**Skills note:** `Skill` tool is **Orchestrator-only** (main Claude Code session, the 4th role per `docs/ORCHESTRATION.md` "Why a 4th role"). Subagents (Architect / Worker) cannot invoke skills — outputs come pre-frozen in phiếu Context per `phieu/TICKET_TEMPLATE.md` `### Skills consulted` (P005, option B).
```

**Thay bằng 2** — update to use new "Quản đốc" persona name:

```
**Skills note:** `Skill` tool is **Quản đốc-only** (the main Claude Code session, Layer 0 orchestrator per `docs/ORCHESTRATION.md`). Subagents (Architect / Worker) cannot invoke skills — outputs come pre-frozen in phiếu Context per `phieu/TICKET_TEMPLATE.md` `### Skills consulted` (P005, option B).
```

**Tìm 3** (3-layer ASCII at lines 34-79):

The current ASCII box has 3 layers starting `┌──...── Layer 1 — CHỦ NHÀ ...`. Add a NEW box ABOVE Layer 1 for Layer 0 Quản đốc.

**Thay bằng / Thêm 3** — insert NEW box before existing Layer 1:

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 0 — QUẢN ĐỐC (Orchestrator / Main session)               │
│  Tools: Read, Write, Glob, Grep, Bash (marker file ops only),   │
│         Task, AskUserQuestion, Skill                            │
│  Owns:                                                          │
│    • State machine (DRAFT → CHALLENGE → RESPOND → APPROVAL      │
│      → EXECUTE)                                                 │
│    • Subagent spawn (architect + worker), marker file hygiene   │
│    • Skill invocation (Orchestrator-only per P005), output      │
│      capture into phiếu Context as frozen artifact              │
│    • Approval gate via AskUserQuestion (one mandatory gate      │
│      before EXECUTE_PHASE)                                      │
│    • Narration of every state transition (no silent state)     │
│  Does NOT:                                                      │
│    • Write production code (Worker's surface)                   │
│    • Read source files (src/, lib/, etc.) for "context"         │
│    • Edit vision docs (Chủ nhà's surface)                       │
│    • Skip APPROVAL_GATE — even for V1-accepted phiếu            │
│  Full spec: `docs/ORCHESTRATION.md`                             │
├─────────────────────────────────────────────────────────────────┤
```

(Then existing Layer 1 / Layer 2 / Layer 3 boxes follow unchanged — Worker connects the new box's bottom border to existing Layer 1 top border.)

**Lưu ý (Task 2):**
- "Quản đốc" is NOT a 4th *role* in the 3-role model; it's the orchestrator persona (the visible surface of the main Claude Code session). Document this disclaimer clearly: maybe a 1-line note above the diagram: "Note: Quản đốc = Layer 0 = main-session orchestrator persona. Still 3-role model — Quản đốc orchestrates, doesn't replace Chủ nhà / Kiến trúc sư / Thợ."
- Skill access column row: per `docs/LAYERS.md:26` + P005 (Skills are Orchestrator-only), Quản đốc has ✏️. This is the ONLY layer with skill access — confirms LAYERS already aligned with ORCHESTRATION Hard rule #9. Task 2 makes this VISIBLE in the table (Quản đốc no longer hidden under "Orchestrator-only note").
- "Quản đốc" Vietnamese gloss: foreman, supervisor — the one who orchestrates work between sub-roles. Maps to "Orchestrator" 1:1.
- Add Layer 0 to the "Skills map (13 total)" section at line 164 if context demands; Tầng 2 — Worker self-decide if a new column "Layer 0 (Quản đốc)" needed in that table or if "Orchestrator-only" annotation suffices. **Recommended:** keep skills map unchanged (skills only belong to Chủ nhà / Kiến trúc sư / Thợ — Quản đốc INVOKES, doesn't OWN — the table tracks ownership, not invocation).

**Validate (Task 2):**
- `grep -c "Quản đốc" docs/LAYERS.md` ≥3 (matrix column + ASCII box + at least 1 prose mention).
- ASCII art renders correctly in markdown preview (no broken `│` characters).
- 3-role model preserved (search for "3-role" should return ≥1 with disclaimer that Quản đốc is orchestrator persona, not new role).

---

### Task 3: `docs/PHILOSOPHY.md` — alignment engineering subsection after Principle 6

**File:** `docs/PHILOSOPHY.md`

**Tìm 1** (existing subsection at lines 40-48, "The deeper principle: information envelopes"):

```markdown
## The deeper principle: information envelopes (alignment engineering)

The 3-role split isn't only about workflow. It's about **information envelope engineering for LLM alignment**.

LLMs hallucinate in proportion to how much *irrelevant* context they see. An Architect-LLM with grep access invents implementations that "look right" but cite phantom functions. A Worker-LLM with full vision-doc access silently re-architects "while it's there." Both failures are caused by **information leakage across role boundaries**, not by lack of skill.

SOS Kit prevents these failures *structurally*: each role has a different `allowedTools` envelope — Architect reads docs but cannot grep code; Worker reads code but cannot see vision strategy. The same human drives all three, but the AI assisting each role sees only what that role needs. Three envelopes, three accountability surfaces.

This is why we don't share context "for efficiency." Shared context is exactly the leak we're preventing.
```

**Thay bằng 1** — EXPAND this subsection (do NOT add a duplicate after Principle 6 — Architect's verdict: existing placement before Principles is the better narrative position; expansion is the right move):

```markdown
## The deeper principle: information envelopes (alignment engineering)

The 3-role split isn't only about workflow. It's about **information envelope engineering for LLM alignment**.

LLMs hallucinate in proportion to how much *irrelevant* context they see. An Architect-LLM with grep access invents implementations that "look right" but cite phantom functions. A Worker-LLM with full vision-doc access silently re-architects "while it's there." Both failures are caused by **information leakage across role boundaries**, not by lack of skill.

### How envelopes are enforced

SOS Kit prevents these failures *structurally*. Each role has a different `allowedTools` envelope:

- **Quản đốc (Layer 0, orchestrator persona for the main Claude Code session)** — spawns subagents, drives state machine, invokes Skills. NO source-code reads (envelope guard); NO production code edits. Sees the phiếu, the BACKLOG, the Debate Log — enough to route, not enough to second-guess.
- **Kiến trúc sư (Architect subagent)** — `Read`, `Write`, `Glob`. NO Bash, NO Grep, NO Edit on source. Reads docs (PROJECT/SOUL/CHARACTER/guides/BACKLOG/DISCOVERIES) but cannot grep source code. Writes phiếu with Task 0 anchors — every assumption framed as "Worker verify at file:line."
- **Thợ (Worker subagent)** — full code tools (`Read`, `Write`, `Edit`, `Glob`, `Grep`, `Bash`). Cannot Read vision docs (PROJECT.md / SOUL.md / CHARACTER*.md) — prevents silent re-architecture from "knowing" the why beyond the phiếu.

Three envelopes, three accountability surfaces. Plus Layer 0 (Quản đốc) routing between them. The same human drives all four mental modes; the AI assisting each one sees only what that mode needs.

### Why "share context for efficiency" is the trap

The intuitive optimization — give every role more context "so it can help better" — is exactly the leak we prevent. Architect with code access invents anchors. Worker with vision access drifts scope. Quản đốc with source-code access starts coding instead of spawning Worker.

The envelopes are not a workflow inconvenience; they are the **alignment surface**. Removing them removes the alignment.

### Why role separation, not just prompt discipline

Prompt discipline ("please don't read code, Architect") fails because LLMs reach for what they have access to. The fix is structural: don't ship the tool. `allowedTools: [Read, Write, Glob]` in Architect's frontmatter, plus a `PreToolUse` hook (`scripts/architect-guard.sh`) hard-blocking `Read` on `src/` paths when the architect marker is active. Even a misbehaving model cannot bypass.

This is also why we don't lean on "trust the model": the hallucination-by-irrelevant-context failure mode is **load-bearing**, not occasional. The 3-role split is the minimum viable structure for catching it.

> *[needs Worker verify]* — If Worker has access to `~/tarot/docs/PHILOSOPHY.md` "alignment engineering" subsection, port additional concrete failure examples from tarot's lessons (named: prompt-injection drift, vision-doc leakage into Worker, code-context leakage into Architect). If tarot inaccessible, Architect's skeleton above suffices.
```

**Tìm 2** (Principle 6 at line 68, "Separate Roles, Separate Brains"):

Currently:
```markdown
### 6. Separate Roles, Separate Brains
One person running a software business wears three hats: **Chủ nhà** (owner — what to build, what to reject, maintain vision), **Kiến trúc sư** (architect — how to spec it, docs-only access), **Thợ** (worker — execute, ship, report reality back). When one brain does all three at once, you get half-finished features, scope explosions, and architectural drift.
```

**Thay bằng 2** — update to acknowledge Layer 0 Quản đốc:

```markdown
### 6. Separate Roles, Separate Brains
One person running a software business wears three hats: **Chủ nhà** (owner — what to build, what to reject, maintain vision), **Kiến trúc sư** (architect — how to spec it, docs-only access), **Thợ** (worker — execute, ship, report reality back). When one brain does all three at once, you get half-finished features, scope explosions, and architectural drift.

In v2.1+ Subagent mode, a 4th persona — **Quản đốc** (Layer 0, the main Claude Code session as orchestrator) — automates the relay between Kiến trúc sư and Thợ. Quản đốc is not a 4th *human* role; it's the AI persona surfacing the orchestrator state machine to Sếp. The human still wears three hats. See [`LAYERS.md`](./LAYERS.md) for Layer 0 specifics.
```

**Lưu ý (Task 3):**
- Expand-not-duplicate decision: prevents two competing "alignment engineering" subsections (one at line 40, one after Principle 6). Single source of truth.
- The `[needs Worker verify]` block at the end of subsection allows Worker to slot in tarot-specific concrete examples if access exists. Architect's skeleton is sufficient stand-alone — Worker tarot port is enhancement, not block.
- Principle 6 reframe is 1 added paragraph — does not bump principle count (still 6 + Principle 0). Quản đốc explicitly NOT counted as a 4th role per CLAUDE.md / LAYERS.md framing ("3-role model + orchestrator persona").

**Validate (Task 3):**
- `grep -c "alignment engineering" docs/PHILOSOPHY.md` ≥1.
- `grep -c "Quản đốc" docs/PHILOSOPHY.md` ≥2 (subsection + Principle 6 update).
- Principle 6 still labeled "6. Separate Roles, Separate Brains" (no renumbering).

---

### Task 4: `docs/ORCHESTRATION.md` — rewrite "Why Quản đốc persona" + add 3 tarot-ported subsections

**File:** `docs/ORCHESTRATION.md`

**Tìm 1** (greeting script literal at lines 18-22):

```markdown
2. Reply briefly (max 5 lines), greeting as the visible "Architect" persona:
   ```
   Em là Kiến trúc sư project <name>.
   Sprint hiện có {N} item: <short list>.
   Anh muốn pick item nào, có idea mới, hay đã có công việc cụ thể?
   ```
```

**Thay bằng 1** — update greeting script to Quản đốc persona:

```markdown
2. Reply briefly (max 5 lines), greeting as **Quản đốc** (the visible orchestrator persona):
   ```
   Em là Quản đốc project <name>.
   Sprint hiện có {N} item: <short list>.
   Anh muốn pick item nào, có idea mới, hay đã có công việc cụ thể?
   ```
```

**Tìm 2** (edge case literal at line 32):

```markdown
- If BACKLOG has no recognizable section (no `## ` headings at all → SessionStart banner stayed silent) → greet without list: "Em là Kiến trúc sư. BACKLOG chưa có item nào — anh có việc gì cần viết phiếu không?" (After P003: ...
```

**Thay bằng 2**:

```markdown
- If BACKLOG has no recognizable section (no `## ` headings at all → SessionStart banner stayed silent) → greet without list: "Em là Quản đốc. BACKLOG chưa có item nào — anh có việc gì cần viết phiếu không?" (After P003: ...
```

**Tìm 3** (subsection at lines 34-37):

```markdown
**Why "Kiến trúc sư" persona for the orchestrator:**
- Solo workflow has 1 human (Chủ nhà) + 1 visible AI counterpart + 1 invisible Worker subagent. Surfacing the orchestrator as a 4th distinct role bloats the mental model.
- Internally the main session is still the orchestrator. It still delegates ticket writing to the `architect` subagent (sandboxed, no code access) when DRAFT_PHASE fires. The persona is UX framing, not a role merger.
- The 8-câu checklist, debate loop, and envelope guard all still run in the subagent — the persona does not let main session bypass them.
```

**Thay bằng 3** — rewrite with Quản đốc framing + clarify subagent distinction (consolidates inline-edit 2026-05-25):

```markdown
**Why "Quản đốc" persona for the orchestrator (consolidates inline-edit 2026-05-25):**

The main Claude Code session (the visible AI surface to Sếp) is *named* "Quản đốc" — Vietnamese for foreman / supervisor — to make the orchestrator role legible without claiming a separate seat in the 3-role model. The persona naming serves 3 purposes:

- **Disambiguates from Kiến trúc sư subagent.** Earlier framing surfaced the orchestrator as "Kiến trúc sư" — same name as the docs-only subagent that writes phiếu. Two roles with one name = handbook confusion. "Quản đốc" names the orchestrator distinctly, so when Sếp reads `agents/orchestrator.md` it's clear which entity is speaking.
- **Matches the actual function.** Quản đốc routes work, doesn't do the work. It spawns Architect subagent for ticket writing, spawns Worker subagent for execution, runs Skills, drives state machine. Foreman semantics fit; Architect semantics don't (Architect *writes* phiếu — that's the subagent's job).
- **Preserves the 3-role model.** Quản đốc is not a 4th *human* role. Sếp still wears 3 hats (Chủ nhà / Kiến trúc sư mental mode / Thợ mental mode). Quản đốc is the AI persona for the main session; the underlying orchestrator role (per Layer 0 in `docs/LAYERS.md`) is the same one v2.0 introduced.

Internally the main session still runs the orchestrator state machine. It still delegates ticket writing to the `architect` subagent (sandboxed, no code access) when DRAFT_PHASE fires. Persona naming is UX framing, NOT a role merger. The 8-câu checklist, debate loop, envelope guard, and marker-file hygiene all still run in the subagents — the Quản đốc persona does not let the main session bypass them.

### Greeting turn template (session opening detail)

> *[needs Worker verify]* — Worker port from `~/tarot/agents/orchestrator.md` if access exists. Otherwise use this Architect skeleton.

The session opening (per Hard rule documented at `docs/ORCHESTRATION.md` "Session opening (first user message)") must follow this template structure:

1. **Read SessionStart context.** Hook injects Active sprint from `docs/BACKLOG.md`. If banner stayed silent (no `##` headings → fallback fires), no Active block to surface — Quản đốc greets without list.
2. **Compose greeting (≤5 lines).** Required elements: persona label ("Em là Quản đốc project <name>"), sprint summary (item count + short list), open-ended branch ask ("Anh muốn pick item nào, có idea mới, hay đã có brief cụ thể?"). Do NOT spawn subagents or run Read/Bash/Grep on this turn — first turn is greet-only.
3. **Wait for Sếp's reply.** Branch:
   - Pick existing BACKLOG item → DRAFT_PHASE (spawn Architect DRAFT)
   - New idea Y → IDEA_INTAKE (`/idea` skill or append to BACKLOG)
   - Concrete brief on first message → skip greet, go DRAFT_PHASE directly (edge case)

Why a dedicated greeting turn: SessionStart hook stdout is injected into the model's context only — it does not render to Sếp's terminal UI. Without explicit greeting, Sếp has no signal that the session is alive and BACKLOG-aware. The greeting turn is the persistent human-visible "I'm here, here's what I see, what do you want?" handshake.

### Tier priority routing rationale

> *[needs Worker verify]* — Worker port from tarot if access; this is the Architect skeleton based on existing P036 tier system documented at `docs/ORCHESTRATION.md` lines 80-93.

Tier routing exists because not every phiếu deserves a multi-turn debate. Architect declares `Tầng: 1` or `Tầng: 2` in the phiếu header during DRAFT, and Quản đốc branches:

- **Tầng 2 (lặt vặt)** — surgical fix, anchor clear, ≤3 files, ≤200 LOC, no schema/API/auth/new-dep change. Skip CHALLENGE_PHASE entirely. DRAFT → APPROVAL_GATE → EXECUTE. The CHALLENGE round-trip is pure overhead for changes Worker can self-verify at EXECUTE time. Cost saved: 1 subagent spawn + Architect RESPOND round-trip (~30-60s + 5-15k tokens per skip).
- **Tầng 1 (móng nhà)** — touches kiến trúc, API contract, data flow, schema, auth boundary, or adds dependency. Worker MUST CHALLENGE before code. The cost of shipping an architecturally-wrong fix dwarfs the CHALLENGE round-trip cost.

**Tier escalation is one-way** (Tầng 2 → Tầng 1 mid-EXECUTE allowed; Tầng 1 → Tầng 2 demotion forbidden). Audit trail integrity: once Architect declared Tầng 1, the debate runs even if it turns out trivial. Silent demotion = lost signal for retro / next-phiếu calibration.

**Default when Architect uncertain:** `Tầng: 1`. Over-tier costs one extra CHALLENGE round-trip; under-tier risks shipping a móng-nhà-wrong fix. The asymmetry favors over-tiering.

### Session opening script (explicit step-by-step)

> *[needs Worker verify]* — Worker port from tarot's 141-line orchestrator.md if access; Architect skeleton below derived from current `agents/orchestrator.md:19-23` + ORCHESTRATION.md:11-37.

When Quản đốc opens a fresh session, the canonical script:

```
1. SessionStart hook fires (scripts/session-start-banner.sh)
   → Reads docs/BACKLOG.md, surfaces Active sprint block into model context
   → Also surfaces P038 cleanup nudges if any merged-but-not-closed phiếu exist
   → Banner stdout goes to model context (NOT user terminal)

2. First user message arrives.

3. Quản đốc reads injected context.
   - If Active sprint block present: extract item titles, count.
   - If banner silent (empty BACKLOG / malformed): note for greeting fallback.

4. Quản đốc composes greeting reply (≤5 lines, Vietnamese):
   ```
   Em là Quản đốc project <name>.
   Sprint hiện có {N} item: <short list>.
   Anh muốn pick item nào, có idea mới, hay đã có brief cụ thể?
   ```

5. Quản đốc DOES NOT:
   - Spawn Architect or Worker subagent on this turn.
   - Read source files (envelope rule).
   - Run Bash beyond marker-file hygiene (mkdir/touch/rm `.sos-state/`).
   - Self-route ("OK I'll start with item 1") — wait for Sếp's pick.

6. On Sếp's reply, Quản đốc branches per state machine:
   - "Pick item X" → DRAFT_PHASE
   - "New idea Y" → IDEA_INTAKE
   - Concrete brief → DRAFT_PHASE directly
   - Off-topic / chat → respond casual, no state transition
```

**Why scripted greeting:** without the script, models default to either (a) over-greeting with verbose context dumps or (b) under-greeting by silently waiting. Both fail the "I'm alive, I see BACKLOG, what's next?" handshake. Scripted = consistent.
```

**Lưu ý (Task 4):**
- Three new subsections (Greeting turn template / Tier priority routing rationale / Session opening script) inserted AFTER the rewritten "Why Quản đốc persona" subsection. Placement: between line 37 (end of rewrite) and line 39 (start of "State machine" section). Worker preserves "State machine" section unchanged.
- Each new subsection starts with `[needs Worker verify]` — Worker, if has access to `~/tarot/agents/orchestrator.md`, port concrete content; otherwise Architect skeleton stands. Both paths valid; Discovery Report logs which path taken.
- Tarot port enhancement OPTIONAL — phiếu does not block on Worker having tarot access. If Worker reports "no tarot access," Architect skeletons are sufficient (they're derived from current sos-kit docs, internally consistent).
- Greeting script in Tìm 1: `<name>` placeholder stays — it's the literal template, project name interpolates at runtime.
- Edge case greeting in Tìm 2: persona rename consistent with Tìm 1.

**Validate (Task 4):**
- `grep -n "Em là Kiến trúc sư" docs/ORCHESTRATION.md` returns 0 (no main-session persona "Kiến trúc sư" greeting literals remain).
- `grep -n "Em là Quản đốc" docs/ORCHESTRATION.md` returns ≥2 (greeting + fallback).
- `grep -n "Why .Quản đốc. persona" docs/ORCHESTRATION.md` returns ≥1 (rewritten subsection heading).
- `grep -c "needs Worker verify" docs/ORCHESTRATION.md` ≥3 (one per new subsection).
- 3 new subsection headings present: "Greeting turn template", "Tier priority routing rationale", "Session opening script".
- "State machine" section at line 39 onwards unchanged (Worker spot-check via `git diff docs/ORCHESTRATION.md`).

---

### Task 5: Deferred-tool loading instruction

**Files:** `agents/orchestrator.md` + `CLAUDE.md`

**Why this task exists:** Future Claude Code sessions need `AskUserQuestion` + `TaskCreate` + `TaskUpdate` ngay turn 1 (Quản đốc routing) + Architect subagents need same tools (per `agents/architect.md` frontmatter line 4 — already has TaskCreate/TaskUpdate/TaskList/AskUserQuestion listed). Direct invocation fails because these tools are **deferred** (lazy-loaded, not auto-loaded). Workaround: invoke `ToolSearch query="select:AskUserQuestion,TaskCreate,TaskUpdate"` at session start. This instruction must persist in both handbooks (orchestrator.md = main session contract; CLAUDE.md = project-wide rule visible to any contributor).

**Tìm 1** (`agents/orchestrator.md`, current file ends ~line 91):

Locate the trailing section (after "Anti-patterns" at line 84-90). Insert new section BEFORE "Anti-patterns" so the new instruction is part of the operational contract, not anti-pattern catalog.

**Thay bằng / Thêm 1** — new section in `agents/orchestrator.md`:

```markdown
## Deferred-tool loading (mandatory session-start step)

Tools `AskUserQuestion`, `TaskCreate`, `TaskUpdate`, `TaskList` are **deferred** — not auto-loaded in fresh sessions. Direct invocation fails with `InputValidationError: tool not loaded`. Quản đốc MUST load them on session start before any state-machine transition.

**Session-start invocation (first turn, before greeting):**

Use `ToolSearch` to register the tools as available:
```
ToolSearch query="select:AskUserQuestion,TaskCreate,TaskUpdate,TaskList"
```

If `ToolSearch` is itself unavailable, the session is in degraded mode — narrate to Sếp and proceed with available tools only (avoid the orchestrator features that require deferred tools: approval gate, sprint tracking).

**Why this matters:**
- `AskUserQuestion` = mandatory for APPROVAL_GATE (Hard rule #1) and FORCE_ESCALATION.
- `TaskCreate` / `TaskUpdate` = sprint tracking visibility for Sếp during multi-step phiếu.
- Architect subagent also needs these (its frontmatter declares them at `agents/architect.md:4`) — but subagent spawn re-loads tools per `tools:` allowlist, so this concern is Quản đốc-specific.
```

**Tìm 2** (`CLAUDE.md`, between line 186 "## Language" and line 192 "## Maintainer-only conventions"):

**Thay bằng / Thêm 2** — insert new top-level section:

```markdown
## Deferred-tool loading (Claude Code session start)

Claude Code's `AskUserQuestion`, `TaskCreate`, `TaskUpdate`, `TaskList` are **deferred** tools — they don't auto-load in fresh sessions. Direct invocation fails with `InputValidationError: tool not loaded`.

The main-session orchestrator (Quản đốc) and Architect subagent both rely on these:
- `AskUserQuestion` — APPROVAL_GATE + FORCE_ESCALATION (orchestrator), multi-choice escalation (Architect)
- `TaskCreate` / `TaskUpdate` / `TaskList` — sprint tracking visibility (both)

**On every new Claude Code session in a sos-kit project**, the first turn MUST invoke `ToolSearch` to register them:

```
ToolSearch query="select:AskUserQuestion,TaskCreate,TaskUpdate,TaskList"
```

This is documented as a mandatory orchestrator session-start step in `agents/orchestrator.md` "Deferred-tool loading" section. Contributors editing the orchestrator handbook must preserve this instruction — removing it causes silent state-machine failure on later turns (approval gate cannot fire = phiếu cannot be approved = workflow halts).
```

**Tìm 3** (`CLAUDE.md:149`, orchestrator handbook size cap rule — interlocked with Tìm 1 above so the new ~12 lines don't violate cap):

Current line:
```markdown
1. `agents/orchestrator.md` is the condensed handbook (~85 lines, ≤90 cap) — system-prompt contract for the main session in every sos-kit project. Keep terse + imperative.
```

**Thay bằng 3** — raise cap from ≤90 to ≤105:

```markdown
1. `agents/orchestrator.md` is the condensed Quản đốc handbook (~95 lines after P043 deferred-tool section, ≤105 cap) — system-prompt contract for the main session in every sos-kit project. Keep terse + imperative.
```

**Lưu ý (Task 5):**
- **DECIDED (Sếp, 2026-05-25): Option A — raise cap to ≤105.** Reason: deferred-tool section is permanent useful addition (~12 lines); compressing other terse content creates churn + degrades readability. NO Worker self-decide on cap path — Tìm 3 above is mandatory edit.
- `CLAUDE.md` insertion position (Tìm 2): between "## Language" and "## Maintainer-only conventions" makes the rule discoverable for external contributors (not buried under maintainer-only).
- `ToolSearch` syntax: Worker confirms exact `query` parameter format at EXECUTE — if Claude Code expects different syntax (e.g. `query:` vs `query =`), Worker fixes both files. Architect's syntax is best-guess from Anthropic platform documentation patterns; Worker has Bash to actually test.
- Tools listed: include `TaskList` (read-only counterpart) per `agents/architect.md:4` which already lists it — completeness.
- Tìm 3 (CLAUDE.md:149 cap raise) is interlocked with Task 6b CLAUDE.md edits — Worker handles in same session so Task 5 Tìm 3 + Task 6b Tìm 3 don't conflict. If Worker hits a conflict (e.g., Task 6b previously had alternate wording), Task 5 Tìm 3 wins (it's the explicit Sếp-decided wording).

**Validate (Task 5):**
- `grep -c "ToolSearch" agents/orchestrator.md` ≥1.
- `grep -c "ToolSearch" CLAUDE.md` ≥1.
- `grep -c "Deferred-tool" agents/orchestrator.md` + `CLAUDE.md` total ≥2.
- `wc -l agents/orchestrator.md` ≤105 (per Sếp-decided cap raise; if Worker file overruns ≤105 → escalate Discovery, do NOT silent-compress).
- `grep -n "≤105 cap" CLAUDE.md` ≥1 (confirms Tìm 3 applied — CLAUDE.md:149 reflects new cap).
- `grep -n "≤90 cap" CLAUDE.md` returns 0 (old cap fully replaced — no orphan reference).

---

### Task 6: Cross-ref pass — README + CLAUDE + HANDOFF

**Files:** `README.md`, `CLAUDE.md`, `docs/HANDOFF.md`

**Why this task exists:** Tasks 1-5 edit primary docs. Task 6 sweeps secondary docs to make sure references align — "Quản đốc" appears where main-session persona is named, "Kiến trúc sư" preserved where subagent referenced, no orphaned references to obsolete terminology.

#### 6a — `README.md`

**Tìm 1** (line 8-14, "Why" section):

```markdown
Building software alone means wearing three hats every day:
- **Chủ nhà** (Owner) — deciding what's worth doing, vetoing scope creep, approving plans, maintaining vision docs
- **Kiến trúc sư** (Architect) — reading docs (not code), writing phiếu, specifying architecture
- **Thợ** (Worker) — reading code, executing the phiếu, running tests, shipping, monitoring, reporting discoveries back
```

**Thay bằng 1** — add note about Quản đốc persona without renumbering the 3 hats:

```markdown
Building software alone means wearing three hats every day:
- **Chủ nhà** (Owner) — deciding what's worth doing, vetoing scope creep, approving plans, maintaining vision docs
- **Kiến trúc sư** (Architect) — reading docs (not code), writing phiếu, specifying architecture
- **Thợ** (Worker) — reading code, executing the phiếu, running tests, shipping, monitoring, reporting discoveries back

In v2.1+ Subagent mode, the main Claude Code session surfaces as **Quản đốc** (Layer 0 orchestrator persona) — automates the relay between Kiến trúc sư and Thợ subagents, runs the state machine, gates approval. Still 3 hats for the human; Quản đốc is the AI orchestrator persona. See [`docs/LAYERS.md`](./docs/LAYERS.md#layer-0--quản-đốc-orchestrator).
```

**Tìm 2** (line 105-114, subagent table):

The table header says "Claude Code Subagents (v2 — Subagent mode)". Currently lists `architect` + `worker`. Add a row for `orchestrator` (= Quản đốc, the main session) but note distinctly as NOT a spawnable subagent.

**Thay bằng 2** — extend table:

```markdown
### Claude Code Subagents (v2 — Subagent mode)

Two role-bound subagents live in `.claude/agents/` and run inside the same Claude Code session, alongside the main-session orchestrator (Quản đốc):

| Subagent | File | Tools allowed | Cannot |
|---|---|---|---|
| **orchestrator** (Quản đốc) | `agents/orchestrator.md` (handbook for main session) | Read, Write, Glob, Grep, Bash (marker ops), Task*, AskUserQuestion, Skill | Read source code for "context"; write production code; edit vision docs; skip APPROVAL_GATE |
| **architect** | `.claude/agents/architect.md` | Read, Write, Glob, TaskCreate/Update/List, AskUserQuestion | Bash, Grep, Edit, read source files (blocked by hook) |
| **worker** | `.claude/agents/worker.md` | Read, Write, Edit, Glob, Grep, Bash, TaskCreate/Update/List, AskUserQuestion | Read PROJECT.md / SOUL.md / CHARACTER.md (vision docs) |

Quản đốc is NOT a spawnable subagent — it's the main Claude Code session itself, with `agents/orchestrator.md` serving as its system-prompt handbook. The two `.claude/agents/*.md` subagents (architect + worker) are spawned by Quản đốc as work demands.

Enforcement is structural: a `PreToolUse` hook (`scripts/architect-guard.sh`) hard-blocks Read/Glob on `src/` paths when the architect marker is active, so even a misbehaving model cannot bypass the envelope.
```

**Tìm 3** (Pipeline diagram around line 39-44):

The diagram currently has stages mapped to Chủ nhà / Kiến trúc sư / Thợ. Layer 0 / Quản đốc not visible. Add a header note (Architect's preferred minimal touch) — diagram itself is fine, just annotate.

**Thay bằng 3** — add 1 sentence after the diagram (currently around line 45 "Each stage belongs to exactly one layer..."):

```markdown
Each stage belongs to exactly one layer. Crossing layers without a handoff is the anti-pattern SOS Kit is built to prevent. **In Subagent mode, Quản đốc (the main-session orchestrator persona) sits across all stages — it routes between layers and runs the state machine but does no stage work itself.** See [`docs/GENESIS.md`](./docs/GENESIS.md) for 0→1 details.
```

#### 6b — `CLAUDE.md`

**Tìm 1** (line 8):

```markdown
SOS Kit = "Solo Operating System" — a distribution center that packages a **3-role workflow** for one-person software teams: **Chủ nhà** (owner / vision / routing), **Kiến trúc sư** (architect / ticket writer / docs-only), **Thợ** (worker / code executor).
```

**Thay bằng 1**:

```markdown
SOS Kit = "Solo Operating System" — a distribution center that packages a **3-role workflow + orchestrator persona** for one-person software teams: **Chủ nhà** (owner / vision / routing), **Kiến trúc sư** (architect / ticket writer / docs-only), **Thợ** (worker / code executor), plus **Quản đốc** (Layer 0 — the main Claude Code session's orchestrator persona in v2.1+ Subagent mode). See `docs/LAYERS.md` for layer specifics.
```

**Tìm 2** (line 42, repo structure — `agents/` block):

Currently:
```markdown
├── agents/                 # Orchestrator + role subagent definitions
│   ├── orchestrator.md     # Condensed orchestrator handbook (≤90 lines, session contract)
│   ├── architect.md        # Kiến trúc sư subagent (Read/Write/Glob, no Bash/Grep/Edit)
│   ├── worker.md           # Thợ subagent (full code tools, no vision docs)
│   └── README.md           # Agent setup instructions
```

**Thay bằng 2** — clarify orchestrator = Quản đốc persona, also adjust line cap per Task 5 Sếp-decided cap raise:

```markdown
├── agents/                 # Orchestrator + role subagent definitions
│   ├── orchestrator.md     # Quản đốc handbook (main-session orchestrator persona, ≤105 lines, session contract — includes deferred-tool loading section)
│   ├── architect.md        # Kiến trúc sư subagent (Read/Write/Glob, no Bash/Grep/Edit)
│   ├── worker.md           # Thợ subagent (full code tools, no vision docs)
│   └── README.md           # Agent setup instructions
```

**Tìm 3** (line 149, the orchestrator handbook size cap rule):

> Note: this edit is the SAME edit as Task 5 Tìm 3. Worker applies it ONCE — listed here for cross-ref completeness so Task 6b stays self-contained. If Task 5 Tìm 3 already applied, validate the line reads as the Thay bằng 3 below; no re-edit needed.

Currently:
```markdown
1. `agents/orchestrator.md` is the condensed handbook (~85 lines, ≤90 cap) — system-prompt contract for the main session in every sos-kit project. Keep terse + imperative.
```

**Thay bằng 3** (Sếp-decided 2026-05-25 — cap raised to ≤105):

```markdown
1. `agents/orchestrator.md` is the condensed Quản đốc handbook (~95 lines after P043 deferred-tool section, ≤105 cap) — system-prompt contract for the main session in every sos-kit project. Keep terse + imperative.
```

#### 6c — `docs/HANDOFF.md`

**Tìm 1** (line 7, intro paragraph):

```markdown
**Critical context**: Kiến trúc sư (Claude Web Project) and Thợ (Claude Code) are **separate sessions**. They cannot ping each other. Chủ nhà (the human) is the only bridge. Every handoff that crosses Architect ↔ Worker goes through Chủ nhà as a manual paste.
```

**Thay bằng 1** — add Subagent mode note:

```markdown
**Critical context (v1 Web Project mode)**: Kiến trúc sư (Claude Web Project) and Thợ (Claude Code) are **separate sessions**. They cannot ping each other. Chủ nhà (the human) is the only bridge. Every handoff that crosses Architect ↔ Worker goes through Chủ nhà as a manual paste.

**Critical context (v2.1 Subagent mode)**: Both Kiến trúc sư and Thợ run as subagents inside the same Claude Code session. The main-session orchestrator persona — **Quản đốc** — automates the relay (see Handoff 2.5 below). Chủ nhà is no longer the courier; Chủ nhà only enters at brief-in and APPROVAL_GATE.
```

**Tìm 2** (line 87 onwards, Handoff 2.5 — "Architect ↔ Worker debate"):

Currently the subsection mentions "orchestrator" 4 times without persona label. Add a one-time label at the first mention.

**Thay bằng 2** — line 89 ("Before Worker EXECUTEs, orchestrator spawns Worker..."):

```markdown
**Trigger:** Architect just wrote phiếu V1 in DRAFT mode. Before Worker EXECUTEs, the main-session orchestrator (persona name: **Quản đốc**) spawns Worker in CHALLENGE mode to verify phiếu's assumptions against real code.
```

(Subsequent uses of "orchestrator" in the subsection — lines 117-121 — remain unchanged. One label introduction sufficient.)

**Lưu ý (Task 6):**
- `README.md` Pipeline diagram is ASCII art; Worker self-decides if minimum-touch (just the trailing sentence) is enough or if the diagram itself needs Quản đốc annotation. **Recommended:** minimum-touch trailing sentence; diagram preserves clean visual.
- `CLAUDE.md` line 149 cap update: identical edit to Task 5 Tìm 3. Apply once. Cap raised to ≤105 per Sếp 2026-05-25 decision (Option A) — NO conditional path remains.
- `HANDOFF.md` other persona refs (e.g. line 7 "Kiến trúc sư (Claude Web Project)") refer to the subagent + Web Project mode — keep "Kiến trúc sư". Only the orchestrator mention gets "Quản đốc" label.

**Validate (Task 6):**
- `grep -c "Quản đốc" README.md` ≥3 (Why section addition + subagent table + Pipeline note).
- `grep -c "Quản đốc" CLAUDE.md` ≥3 (intro line + repo structure + line 149 update).
- `grep -c "Quản đốc" docs/HANDOFF.md` ≥1 (Handoff 2.5 label introduction).
- `grep -c "Kiến trúc sư" agents/architect.md` ≥1 (subagent identity preserved — symmetric sanity check).
- No leftover "Em là Kiến trúc sư" main-session greeting literal anywhere: `grep -rn "Em là Kiến trúc sư" .` returns 0 across whole repo.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `docs/LAYERS.md` | Task 2: add Layer 0 Quản đốc to access matrix (new column) + ASCII diagram (new box above Layer 1) + Skills note rewording |
| `docs/PHILOSOPHY.md` | Task 3: expand "alignment engineering" subsection + Principle 6 reframe with Quản đốc note |
| `docs/ORCHESTRATION.md` | Task 4: rewrite greeting script (lines 18-22) + edge case (line 32) + "Why Quản đốc persona" subsection (lines 34-37) + add 3 new subsections (Greeting turn template / Tier priority routing rationale / Session opening script) |
| `agents/orchestrator.md` | Task 5: add "Deferred-tool loading" section before "Anti-patterns" (+ confirm Sếp's 2-line edits at lines 9 + 21 intact) |
| `CLAUDE.md` | Task 5 + Task 6: add "Deferred-tool loading" section before "Maintainer-only conventions" + line 8 reframe (3-role + orchestrator) + line 42 repo structure + line 149 cap raise ≤90 → ≤105 (Sếp-decided) |
| `README.md` | Task 6: Why section Quản đốc note + subagent table extension + Pipeline diagram trailing sentence |
| `docs/HANDOFF.md` | Task 6: intro paragraph Subagent-mode split + Handoff 2.5 first-mention Quản đốc label |
| `agents/orchestrator.md` (already staged) | Sếp's 2-line edits at lines 9 + 21 — Worker commits this state along with Task 5 additions |
| `docs/BACKLOG.md` (already staged) | Quản đốc's wave 1 sprint + P040 SHIPPED + recently shipped entry — Worker commits nguyên trạng (NO new edits in P043) |

(8 files total; 2 already staged + 6 new doc edits = 1 PR consolidating all.)

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `agents/architect.md` | NO edits. Subagent identity "Kiến trúc sư" stays. Persona name change is main-session only. Worker spot-check: `grep "Kiến trúc sư" agents/architect.md` returns ≥1, file otherwise unchanged. |
| `agents/worker.md` | NO edits. Worker subagent's "Thợ" identity preserved; not relevant to P043. |
| `phieu/TICKET_TEMPLATE.md`, `phieu/RELAY_PROTOCOL.md`, `phieu/DISCOVERY_PROTOCOL.md`, `phieu/AUDIT_PROTOCOL.md`, `phieu/GENESIS_TEMPLATE.md`, `phieu/LAUNCH_CHECKLIST.md` | NO edits. Phiếu workflow contract docs out of scope for P043. |
| `phieu/done/*.md` (historical phiếu) | NO edits. Don't rewrite history. |
| `phieu/active/P043-*.md` | This phiếu itself — only Debate Log section edited by Worker/Architect in CHALLENGE/RESPOND mode. Nhiệm vụ section is the contract — preserved. |
| `bin/sos.sh`, `scripts/*.sh`, `hooks/pre-commit` | NO edits. Foundation tools unchanged. |
| `skills/**/SKILL.md` | NO edits. P045 (Open backlog) handles skill drift sweep separately. Verify `skills/init/SKILL.md` / `skills/plan/SKILL.md` / etc. still reference "Kiến trúc sư" / "Chủ nhà" / "Thợ" as appropriate role names. |
| `docs/SETUP.md`, `docs/COMPARISON.md`, `docs/GENESIS.md`, `docs/DISCOVERIES.md` | NO edits in P043 scope. If Worker spots blocking inconsistency (e.g., SETUP.md references "Em là Kiến trúc sư" literal) → escalate via Discovery Report, NOT silent fix. |
| `templates/*` | NO edits. |
| `recipes/**/*.md` | NO edits. |
| `INSTALL.md` | NO edits in P043. May need follow-up sweep but Architect estimates current INSTALL.md doesn't have main-session-persona literals to fix. Worker spot-checks. |

---

## Luật chơi (Constraints)

1. **Tier locked at 1 (móng nhà).** Foundation doc touch with ripple. MUST complete CHALLENGE before EXECUTE per ORCHESTRATION.md Hard rule #7 / P036. Persona naming = contract (P041/P042/future phiếu reference) — mismatch here costs every downstream phiếu.
2. **NO blanket rename.** "Kiến trúc sư" stays in: `agents/architect.md` (subagent identity), `docs/LAYERS.md` (3-role table — Layer 2), `phieu/TICKET_TEMPLATE.md`, all `phieu/done/*.md` archive, `docs/HANDOFF.md` Web Project mode references, `README.md` 3-role mentions. "Quản đốc" replaces "Kiến trúc sư" ONLY where the main-session orchestrator persona is meant. The discriminator: **subagent that writes phiếu** = Kiến trúc sư; **main Claude Code session that spawns subagents** = Quản đốc.
3. **Three-role model preserved.** P043 does NOT introduce a 4th *role*. It NAMES the orchestrator persona (Layer 0) for the main session. The 3-role human framework (Chủ nhà / Kiến trúc sư mental mode / Thợ mental mode) is unchanged. Every doc that says "3-role" stays "3-role" — with optional clarifier "plus Quản đốc orchestrator persona in Subagent mode."
4. **No code touched.** P043 is docs-only. Worker uses `Read` / `Write` / `Edit` / `Glob` / `Grep` but does NOT touch `bin/sos.sh`, `scripts/*.sh`, `hooks/*`, `agents/architect.md`, `agents/worker.md`, `templates/*`, `recipes/**`.
5. **Tarot port = enhancement, not block.** Anchors #8 + #9 are `[needs Worker verify]` for tarot file Read. Worker port from tarot if access exists; if not, Architect skeletons in Task 3 + Task 4 are sufficient stand-alone. Discovery Report logs which path taken. **Phiếu does not block on Worker having tarot repo access.**
6. **Dangling staged files = 1 PR.** Worker's EXECUTE commits `agents/orchestrator.md` (Sếp's staged 2-line edit) + `docs/BACKLOG.md` (Quản đốc's staged sprint update) + all P043 new edits in a SINGLE PR. NOT 2 PRs. Commit message format: `docs(P043): consolidate Quản đốc persona codify + alignment engineering + deferred-tool loading` (Worker may refine wording — Tầng 2).
7. **Line cap raise — DECIDED (Sếp, 2026-05-25).** `agents/orchestrator.md` cap raised from ≤90 → ≤105 to accommodate Task 5 deferred-tool section (~12 lines). Worker MUST apply Task 5 Tìm 3 edit to `CLAUDE.md:149` (single source of truth — Task 6b Tìm 3 references same edit). NO Worker self-decide on cap path. If post-edit `wc -l agents/orchestrator.md > 105` → escalate Discovery, do NOT silent-compress or further raise cap.
8. **Pre-commit hook compliance.** P043 changes affect: CHANGELOG.md (new entry required), DISCOVERIES.md (per-phiếu file `docs/discoveries/P043.md` + index entry per docs-gate). docs-gate runs against this repo's own `.docs-gate.toml` (P006 dogfood). Worker ensures both files updated before commit.
9. **No re-edit of BACKLOG.md.** `docs/BACKLOG.md` is already staged with Quản đốc's sprint update (wave 1 Active, P040 SHIPPED, paused harvest). Worker commits as-is. The P043 → "Recently shipped" move happens AFTER PR merge (Sếp/orchestrator handles post-merge, NOT Worker mid-EXECUTE) per BACKLOG.md "Maintenance rules" #2.
10. **Vietnamese persona name preserved.** "Quản đốc" stays as Vietnamese — do NOT translate to "Foreman" / "Supervisor" / "Orchestrator" in user-facing prose. Only in technical glosses (e.g. "Quản đốc (orchestrator persona)") is the English term used parenthetically. Matches project convention (Chủ nhà / Kiến trúc sư / Thợ also stay Vietnamese throughout).

---

## Nghiệm thu

### Automated
- [ ] `grep -rn "Em là Kiến trúc sư" .` returns 0 across entire repo (no leftover main-session greeting literals).
- [ ] `grep -c "Quản đốc" docs/LAYERS.md` ≥3.
- [ ] `grep -c "Quản đốc" docs/PHILOSOPHY.md` ≥2.
- [ ] `grep -c "Quản đốc" docs/ORCHESTRATION.md` ≥5 (rewritten subsection + 3 new subsections + greeting literal).
- [ ] `grep -c "Quản đốc" docs/HANDOFF.md` ≥1.
- [ ] `grep -c "Quản đốc" README.md` ≥3.
- [ ] `grep -c "Quản đốc" CLAUDE.md` ≥3.
- [ ] `grep -c "Quản đốc" agents/orchestrator.md` ≥2 (Sếp's edits at lines 9 + 21).
- [ ] `grep -c "Kiến trúc sư" agents/architect.md` ≥1 (subagent identity preserved — symmetric sanity check).
- [ ] `grep -c "ToolSearch" agents/orchestrator.md` ≥1.
- [ ] `grep -c "ToolSearch" CLAUDE.md` ≥1.
- [ ] `wc -l agents/orchestrator.md` ≤105 (matches CLAUDE.md cap as raised per Sếp 2026-05-25 decision — Task 5 Tìm 3).
- [ ] `grep -n "≤90 cap" CLAUDE.md` returns 0 (old cap fully replaced; new `≤105 cap` only).
- [ ] No broken markdown: open each edited file in any markdown previewer; all headings render, tables align, ASCII art unbroken.

### Manual Testing
- [ ] Fresh Claude Code session in this repo: open, observe SessionStart banner fires + greets as "Em là Quản đốc project sos-kit" (NOT "Kiến trúc sư"). If banner doesn't trigger main session naming, that's a separate issue — log to Discovery.
- [ ] Read `docs/LAYERS.md` end-to-end: 4-layer narrative coherent (Layer 0 → 1 → 2 → 3), access matrix readable, no orphan references to "Kiến trúc sư orchestrator."
- [ ] Read `docs/ORCHESTRATION.md` end-to-end: rewritten "Why Quản đốc persona" subsection at lines ~34-50 makes sense to a fresh contributor; 3 new subsections (Greeting / Tier / Session opening) are discoverable.
- [ ] Read `agents/orchestrator.md` end-to-end: deferred-tool loading section actionable; `ToolSearch` syntax matches reality (Worker tested at EXECUTE).
- [ ] Cross-check with `agents/architect.md` + `agents/worker.md`: confirm subagent identities unchanged.
- [ ] Spot-check `phieu/done/P040-bootstrap-stack-detection.md`: any reference to orchestrator should NOT have been retroactively rewritten (historical phiếu = archive, immutable).

### Regression
- [ ] State machine semantics unchanged. `docs/ORCHESTRATION.md` "State machine" section (lines 39-78) byte-identical to pre-P043: `git diff docs/ORCHESTRATION.md` → only changes in lines 18-37 + new subsections appended around line 38-130.
- [ ] Hard rules at `docs/ORCHESTRATION.md` "Hard rules" section unchanged.
- [ ] Tier routing rules at `docs/ORCHESTRATION.md` "Tier routing (P036)" section unchanged (Task 4's "Tier priority routing rationale" subsection is NEW added content explaining existing rules, NOT rewriting them).
- [ ] Phiếu format (TICKET_TEMPLATE.md) unchanged.
- [ ] Subagent allowlists in `.claude/agents/architect.md` + `.claude/agents/worker.md` unchanged (envelope rules preserved). Note: file may be at `.claude/agents/architect.md` (project-local) OR `agents/architect.md` (global) — Worker checks both, edits NEITHER.
- [ ] All 3 layer skill maps (Chủ nhà / Kiến trúc sư / Thợ skills) in LAYERS.md and README.md unchanged. P043 does NOT touch skills.
- [ ] `docs/BACKLOG.md` content matches the staged state — Worker commits without modifying.
- [ ] Pre-commit hook passes: type-check N/A (docs-only), docs-gate succeeds with CHANGELOG.md updated + per-phiếu Discovery file present + BACKLOG.md present (already staged) + DISCOVERIES.md index entry.

### Docs Gate
- [ ] `CHANGELOG.md` — new entry at top: "P043: doc drift consolidate — Quản đốc persona codify (main-session orchestrator persona named distinctly from Kiến trúc sư subagent), alignment engineering subsection expanded in PHILOSOPHY.md, ORCHESTRATION.md rewritten with greeting / tier / session-opening subsections, deferred-tool loading instruction added to orchestrator.md + CLAUDE.md, CLAUDE.md:149 orchestrator handbook cap raised ≤90 → ≤105 per Sếp 2026-05-25 decision. Touches 7 files; 1 PR consolidating Sếp's staged orchestrator.md + Quản đốc's staged BACKLOG.md."
- [ ] `docs/BACKLOG.md` — no Worker edit (Sếp/orchestrator post-merge moves P043 to Recently shipped).
- [ ] `agents/orchestrator.md` cap update reflected in `CLAUDE.md:149` (Task 5 Tìm 3 / Task 6b Tìm 3 — SAME edit, applied once).

### Discovery Report
- [ ] Write to `docs/discoveries/P043.md` (per-phiếu file, P038 pattern):
  - **Tarot Read access at EXECUTE.** Did Worker have access to `~/tarot/docs/PHILOSOPHY.md` + `~/tarot/agents/orchestrator.md` for content port (anchors #8 + #9)? If yes: which specific content was ported, line/section refs. If no: Architect skeletons stand; flag for follow-up tarot recon phiếu if Sếp wants tarot-quality fidelity.
  - **Persona discriminator edge cases.** Did Worker hit ambiguous "Kiến trúc sư" references where context didn't make it clear if main-session or subagent was meant? If yes: list with file:line, default rule applied (subagent), and recommend Architect clarify in next sweep.
  - **`ToolSearch` syntax verified.** Worker confirms exact syntax via bash test at EXECUTE — does `ToolSearch query="select:AskUserQuestion,..."` work as expected? Any deviation flagged for Architect's CLAUDE.md / orchestrator.md edit.
  - **Line cap decision applied.** Confirm Task 5 Tìm 3 + Task 6b Tìm 3 (same edit) landed in `CLAUDE.md:149` reading `≤105 cap`. Document final `wc -l agents/orchestrator.md` count post-edit. If cap exceeded (>105), flag escalation.
  - **Cross-ref completeness.** Did Worker spot any doc with main-session-persona "Kiến trúc sư" leftover that P043 didn't cover (e.g. `docs/SETUP.md`, `INSTALL.md`, `docs/COMPARISON.md`, `docs/GENESIS.md`)? If yes: flag as P044 candidate (DO NOT silent-fix — Tầng 1 scope expansion).
  - **Dangling staged files commit.** Confirm `agents/orchestrator.md` + `docs/BACKLOG.md` committed nguyên trạng with new P043 edits in single PR. Quote the final commit message.
  - **CHALLENGE round value.** Did the CHALLENGE round (Worker → Architect) for this Tầng 1 doc-drift phiếu catch anything? Honest signal for P036 retrospective.
  - **Total time + tokens.** Architect estimate: half-day. Actual?
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
