# PHIẾU P005: Worker Skill access — codify Option B (Skills are Orchestrator-only)

> **Loại:** Chore (handbook codification)
> **Ưu tiên:** P1
> **Tầng:** 2 (lặt vặt — handbook + template + index doc edits, ≤8 files, no schema/API/auth/dep change, anchor rõ)
> **Ảnh hưởng:** `agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md`, `docs/ORCHESTRATION.md`, `phieu/TICKET_TEMPLATE.md`, `docs/LAYERS.md`, `docs/BACKLOG.md`, `CHANGELOG.md`, `docs/DISCOVERIES.md`, `docs/discoveries/P005.md` (NEW)
> **Dependency:** None (P008 will be re-scoped after P005 ships, see Task 4)
> **Branch:** `fix/P005-skills-orchestrator-only`
> **Date:** 2026-05-10

---

## Context

### Vấn đề hiện tại

Original gap (BACKLOG line 16): `agents/worker.md:4` `tools:` allowlist **không có `Skill`** → Worker không thể gọi Claude Code Skills (`/frontend-design`, `/security-review`, etc.) mid-EXECUTE. Architect cũng không có `Skill` (line 4: `tools: Read, Write, Glob, TaskCreate, TaskUpdate, TaskList, AskUserQuestion`). Cả 2 subagent đều bị chặn — gap thực sự, không phải imagined.

3 options đã debate ~2 tuần (từ 2026-04-26):
- **A.** Add `Skill` vào Worker tools allowlist (1-line edit). Pragmatic nhưng mở envelope: skill output non-deterministic, varies với skill state → reproducibility risk khi audit phiếu sau ship.
- **B.** *(em recommend, **Sếp pick 2026-05-10**)* Architect/Orchestrator run skill **trước** CHALLENGE/EXECUTE, đổ output **frozen** vào phiếu Context. Worker chỉ apply text in-phiếu. Allowlists giữ tight.
- **C.** Hybrid — Worker invoke skill chỉ khi phiếu có flag `requires_skill: <name>`. Thêm field Architect dễ quên → silent gap.

**Sếp lock B 2026-05-10.** Lý do thắng:
1. **Reproducibility** — skill output frozen tại thời điểm Architect spawn → re-run phiếu cho same output. A/C không có guarantee này.
2. **Allowlist tight** — Architect/Worker envelope không phình; Skill chỉ ở Orchestrator (main session) — natural home vì Orchestrator native có Skill tool sẵn (Claude Code main session always has access).
3. **Không thêm field** — không cần `requires_skill:` flag (C) → không có vector để Architect quên.
4. **Codify pattern đã informally đúng** — main session đã có Skill, subagent thì không. Phiếu này chỉ document + enforce explicitly, không change tools list.

### Giải pháp

5 deliverables (handbook + template + index), 0 code, 0 tools-list change:

1. **`agents/orchestrator.md`** — new section "Invoking skills (Skill tool)" (6–10 dòng): when (phiếu cần design tokens / threat model / external pattern), how (run skill main session, capture output verbatim), where to embed (phiếu Context `## Skills consulted` subsection HOẶC Architect spawn prompt).
2. **`docs/ORCHESTRATION.md`** — Hard rule mới: "**Skills are Orchestrator-only.** Architect và Worker MUST NOT invoke Skill. Orchestrator runs skill, captures output, embeds in phiếu Context as frozen artifact." + paragraph trong example session minh họa pattern.
3. **`phieu/TICKET_TEMPLATE.md`** — OPTIONAL subsection trong Context block: `## Skills consulted (optional)` với HTML comment hướng dẫn format. Most phiếu sẽ bỏ qua subsection này.
4. **`agents/architect.md`** + **`agents/worker.md`** — 1 sentence each near tools/inputs section: skill outputs frozen sẵn trong phiếu Context, KHÔNG invoke Skill (also not in allowlist).
5. **`docs/LAYERS.md`** — access matrix table thêm row "Skills (`/frontend-design`, etc.)" với "✏️ runs (main session only)" cho Orchestrator column + footnote về frozen-artifact pattern. (Hiện table chỉ có 3 cột Chủ nhà/Kiến trúc sư/Thợ — Orchestrator là 4th role per `docs/ORCHESTRATION.md` "Why a 4th role"; Worker xác minh format thêm cột vs footnote.)

Plus dependent updates from P005 ship:

6. **`docs/BACKLOG.md`** — flip `[ ] **[P005]**` → `[x] ~~**[P005]**~~ — SHIPPED 2026-05-10 (option B)`. Update P008 entry: "DEPENDS on P005" → "Now scoped to Architect/Orchestrator-side workflow doc (Worker doesn't invoke skill, P008 = handbook section about WHEN Orchestrator invokes `frontend-design` for FE/UI phiếu)".
7. **`CHANGELOG.md`** — entry `[v2.1.10] — 2026-05-10` summarizing option B locked + frozen-artifact pattern.
8. **`docs/discoveries/P005.md`** (NEW) — discovery report: option B reasoning (vs A reproducibility risk, vs C silent-gap risk), note ~2 tuần debate, note pattern đã informally đúng — phiếu codify what was already true.
9. **`docs/DISCOVERIES.md`** — index row P005 newest-on-top (above P006).

### Scope

- CHỈ sửa: 9 file trên (8 modified + 1 new).
- KHÔNG sửa: `Skill` trong **bất kỳ** `tools:` allowlist của agent nào (đó là điểm option B). KHÔNG sửa `phieu/phieu.sh`, `hooks/pre-commit`, `scripts/`, skills folder, recipes, integrations, `docs/HANDOFF.md`, `docs/PHILOSOPHY.md`.
- KHÔNG mở envelope mới, KHÔNG thêm `requires_skill:` flag (đó là option C, đã reject), KHÔNG redesign state machine.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `agents/worker.md:4` tools line **không** có `Skill` | `grep -n "^tools:" agents/worker.md` | ✅ verified — `tools: Read, Write, Edit, Glob, Grep, Bash, TaskCreate, TaskUpdate, TaskList, AskUserQuestion` (no `Skill`) `[verified]` |
| 2 | `agents/architect.md:4` tools line **không** có `Skill` | `grep -n "^tools:" agents/architect.md` | ✅ verified — `tools: Read, Write, Glob, TaskCreate, TaskUpdate, TaskList, AskUserQuestion` (no `Skill`) `[verified]` |
| 3 | `agents/orchestrator.md` line 4 `tools: []` (intentionally empty per comment line 7 — orchestrator is main session, not spawnable) | `sed -n '4,7p' agents/orchestrator.md` | ✅ verified `[verified]` — main session natively has Skill via Claude Code, không phải qua frontmatter allowlist |
| 4 | `agents/orchestrator.md` cap ≤90 lines (per file: hiện 91 dòng, thêm section phải tuân cap → có thể cần condense `## Marker file hygiene` hoặc `## Phiếu cleanup nudge` để đổi chỗ) | `wc -l agents/orchestrator.md` | ✅ 91 dòng `[verified]`. **Lưu ý:** P035 spec ≤90 cap; CHANGELOG v2.1.5 nói "~88 lines, ≤90 cap"; P038 v2.1.7 đẩy lên 90. Worker phải condense thành 6-10 dòng max + verify cap không bị phá. Insertion point gợi ý: sau `## Phiếu cleanup nudge (P038)` (line 62-63), trước `## Bulk input handling (P035)` (line 65) — Worker decide chính xác |
| 5 | `docs/ORCHESTRATION.md` Hard rules section đánh số tới #8 (P035 thêm rule #8 bulk input) | `grep -nE "^[0-9]+\. \*\*" docs/ORCHESTRATION.md` | ✅ verified — `1.` (Max 3 turns) ... `8.` (Bulk input) `[verified]` line 110-117. New rule = #9 "Skills are Orchestrator-only" |
| 6 | `docs/ORCHESTRATION.md` example session bắt đầu line 152 ("## Concrete example session") | `grep -n "^## " docs/ORCHESTRATION.md` | ✅ verified `[verified]` — sections list line 1, 5, 11, 39, 80, 95, 108, 119, 131, 150, 216 |
| 7 | `phieu/TICKET_TEMPLATE.md` Context section ở line 18 (`## Context`), kết thúc trước `## Task 0` line 32 | `grep -n "^## " phieu/TICKET_TEMPLATE.md` | ✅ verified `[verified]` — `## Context` 18, `### Vấn đề hiện tại` 20, `### Giải pháp` 23, `### Scope` 26, `## Task 0` 32. Insertion point: sau `### Scope` block, trước `---` separator |
| 8 | `docs/LAYERS.md` access matrix có 5 rows hiện tại (vision/code/tickets/discovery/bash) cột header `\| \| Chủ nhà \| Kiến trúc sư \| Thợ \|` (3 cột role + 1 col label) | `sed -n '17,27p' docs/LAYERS.md` | ✅ verified `[verified]` line 19-25. Hiện không có cột Orchestrator. Worker decide: (a) thêm cột thứ 4 "Orchestrator" (table widening — affects all rows), HOẶC (b) thêm row "Skills" với cell "Orchestrator only (main session)" + footnote — option (b) ít invasive. Architect lean (b) |
| 9 | `docs/BACKLOG.md` line 16 P005 entry exact text + line 59 P008 entry | `sed -n '16,20p; 59,60p' docs/BACKLOG.md` | ✅ verified `[verified]` (đã đọc 2026-05-10) — P005 line 16-20, P008 line 59 |
| 10 | `CHANGELOG.md` top-most entry là `[v2.1.9] — 2026-05-10` (P006) | `head -20 CHANGELOG.md` | ✅ verified `[verified]` — line 5. Next version = `[v2.1.10] — 2026-05-10` (same day double-release vẫn OK per Keep-a-Changelog) |
| 11 | `docs/DISCOVERIES.md` index format (table với cột Phiếu \| Date \| 1-line summary), top entry = P006 link tới `discoveries/P006.md` | `sed -n '9,15p' docs/DISCOVERIES.md` | ✅ verified `[verified]` line 11-13. Insertion point: row mới P005 ngay trên P006 (newest-on-top per file header) |
| 12 | `docs/discoveries/` directory tồn tại (P038 đã tạo) | `test -d docs/discoveries && ls docs/discoveries/` | ✅ verified `[verified]` — chứa `P006.md`, `P038.md`, `P039.md`. Worker tạo `P005.md` mới |
| 13 | `agents/architect.md` có "Inputs" section (Sếp's prompt nhắc) HOẶC tương đương | Worker greps `grep -nE "^## .*[Ii]nput\|^## DRAFT mode workflow" agents/architect.md` | `[needs Worker verify]` — Architect không grep tools cho phép, scan visual line 49-58 thấy "Load context" trong DRAFT workflow nhưng không có literal "Inputs" header. Worker decide insertion point: dưới line 58 (cuối Load-context list) HOẶC mới sub-bullet trong DRAFT workflow step 1. 1-sentence integration only |
| 14 | `agents/worker.md` "tools allowlist" reference điểm: line 4 frontmatter HOẶC `## Hard envelope rules` (line 12) | Worker greps `grep -n "^## \|^tools:" agents/worker.md` | ✅ verified `[verified]` — frontmatter line 4, `## Hard envelope rules` line 12, `You have full code tools:` line 14. Worker insertion point: sau line 14 ("Read, Write, Edit, ... Bash") — natural fit cho 1 sentence "Skills are Orchestrator-only" |
| 15 | `phieu/TICKET_TEMPLATE.md` không có existing `## Skills consulted` section | `grep -n "Skills consulted" phieu/TICKET_TEMPLATE.md` | ✅ verified `[verified]` (đã đọc full template) — section chưa có, an toàn add |
| 16 | sos-kit's `docs-gate` config ở root `.docs-gate.toml` (P006 ship) handle CHANGELOG.md staged + Discovery Report new file | Worker `cat .docs-gate.toml` (P006 artifact) | `[needs Worker verify]` — pre-commit phải pass với CHANGELOG entry mới + new `docs/discoveries/P005.md`. Nếu hook fail nghĩa P006 config gap; Tier 2 escalation 2→1 nếu schema mismatch |
| 17 | Vietnamese vs English convention cho handbook files (CLAUDE.md "Language" section: handbook + agent files English; phiếu body OK Vietnamese) | `grep -A 5 "^## Language" CLAUDE.md` | ✅ verified `[verified]` (đã đọc CLAUDE.md) — `agents/*.md`, `docs/*.md` English; phiếu body OK Vietnamese. Sếp directive xác nhận: "Vietnamese for body text in handbook sections, English for handbook structure" — Worker khi viết section mới: section title English, body có thể có Vietnamese inline cho clarity nhưng main flow English (match existing handbook voice) |

**Anchors `[needs Worker verify]`: #13, #16.** Architect không peek source per `feedback_architect_no_hallucination.md`. Worker grep-first ở EXECUTE để xác nhận insertion point chính xác.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp agents/orchestrator.md ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp agents/architect.md ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp agents/worker.md ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp docs/ORCHESTRATION.md ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp phieu/TICKET_TEMPLATE.md ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp docs/LAYERS.md ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

Rollback (worktree only): copy back từ `.backup/${PHIEU_ID}/` + `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)`.

---

## Debate Log

> Tier 2 phiếu — orchestrator routing per `docs/ORCHESTRATION.md` "Tier routing": DRAFT → APPROVAL_GATE → EXECUTE (skip CHALLENGE_PHASE). Debate Log section vẫn được khởi tạo cho audit trail; Worker EXECUTE dùng để log Tier escalation 2→1 nếu trigger fires (xem rules step 4a `agents/worker.md`).

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge
*(Tier 2 — skip CHALLENGE expected. Nếu Worker phát hiện tier-escalation trigger mid-EXECUTE [touches schema / API / auth / new dep / cross-module data flow], STOP, append Turn 1 với `file:line` evidence, tier 2→1, return to orchestrator.)*

### Final consensus
- Phiếu version: V1 (no debate expected)
- Total turns: 0 (Tier 2 skip-CHALLENGE)
- Approved by Chủ nhà: [date pending — orchestrator runs APPROVAL_GATE]

---

## Nhiệm vụ

### Task 1: Update agent handbook trio (orchestrator + architect + worker)

**File 1.1:** `agents/orchestrator.md`

**Tìm:** Section `## Phiếu cleanup nudge (P038)` (around line 62-63 per anchor #4). After this section, before `## Bulk input handling (P035)`.

**Thay bằng / Thêm:** New section `## Invoking skills (Skill tool) (P005)`. 6–10 dòng max (cap ≤90 lines tổng — Worker phải verify post-edit `wc -l agents/orchestrator.md` ≤ 90; nếu vượt, condense `Marker file hygiene` hoặc `Phiếu cleanup nudge` 1 dòng để bù).

Content (English, mechanical, mirror existing voice):
```
## Invoking skills (Skill tool) (P005)
Skills (`/frontend-design`, `/security-review`, etc.) are **Orchestrator-only**. When a phiếu needs skill output (design tokens, threat model, external pattern):
1. Run the skill in the main session BEFORE spawning Architect (or before APPROVAL_GATE if mid-flow).
2. Capture output verbatim. Embed in phiếu Context under `## Skills consulted` subsection (per `phieu/TICKET_TEMPLATE.md`) — frozen artifact, audit trail.
3. Subagents (Architect / Worker) read skill output FROM phiếu — they MUST NOT invoke Skill themselves (not in their allowlist anyway).
```

**Lưu ý:**
- Cap check: post-edit `wc -l agents/orchestrator.md` MUST ≤ 90. Nếu > 90, condense ưu tiên section `## Phiếu cleanup nudge (P038)` (line 62-63 hiện 2 dòng — có thể merge vào 1 dòng) để giành chỗ.
- KHÔNG đụng frontmatter line 4 `tools: []` — comment line 7 đã giải thích empty là intentional.

**File 1.2:** `agents/architect.md`

**Tìm:** Cuối DRAFT workflow step 1 "Load context" list (line ~58, sau `Any guide doc relevant to the request...`) HOẶC equivalent insertion point Worker xác minh (anchor #13 `[needs Worker verify]`).

**Thay bằng / Thêm:** Add 1 bullet to the load-context list:
```
   - Skill outputs (if any) appear in phiếu Context under `## Skills consulted` — read them as part of phiếu, do not invoke skills yourself (not in allowlist).
```

**Lưu ý:**
- KHÔNG đụng frontmatter line 4 `tools:` — `Skill` cố ý absent là điểm cốt lõi của option B.
- 1 sentence only — không expand thành sub-section.

**File 1.3:** `agents/worker.md`

**Tìm:** Line 14 (sau `You have full code tools: ...`).

**Thay bằng / Thêm:** Add 1 sentence after line 14:
```
Skills are Orchestrator-only. If a phiếu's spec depends on skill output, that output is already frozen in the Context section under `## Skills consulted`. Do not invoke `Skill` (not in your allowlist anyway).
```

**Lưu ý:**
- KHÔNG đụng frontmatter line 4 `tools:` — same reason.
- 1 sentence only.

---

### Task 2: Update `docs/ORCHESTRATION.md` (Hard rule + example session paragraph)

**File:** `docs/ORCHESTRATION.md`

**Tìm 2.1:** Hard rules list, after rule `8. **Bulk input → auto-triage + ONE gate.**` (around line 117 per anchor #5).

**Thay bằng / Thêm:** New rule #9:
```
9. **Skills are Orchestrator-only.** Architect and Worker MUST NOT invoke `Skill`. Orchestrator runs the skill in the main session BEFORE spawning Architect (or before APPROVAL_GATE if mid-flow), captures output verbatim, embeds in phiếu Context under `## Skills consulted` subsection as frozen artifact. Reproducibility: re-running the phiếu yields the same output. Allowlist: subagent `tools:` lists do NOT include `Skill` — this is intentional, not an oversight (P005, option B).
```

**Tìm 2.2:** `## Concrete example session` block (line ~150 per anchor #6). After "Read CLAUDE.md, BACKLOG.md, PROJECT.md, DISCOVERIES.md." in `[ARCHITECT DRAFT]` block (around line 158).

**Thay bằng / Thêm:** Insert example invocation BEFORE `[ARCHITECT DRAFT]` to show the skill-first pattern. Worker decides exact placement; pattern:
```
USER: build a phiếu cho item "Add user export" — phiếu này có UI form, cần design tokens

ORCHESTRATOR: phiếu touches UI → running /frontend-design first to capture design tokens, freezing into phiếu Context.

  [SKILL /frontend-design output captured 2026-05-10 — pasted into Architect spawn prompt + phiếu Context]

ORCHESTRATOR: spawning architect (DRAFT) with frozen design-token context...

  [ARCHITECT DRAFT]
  Read CLAUDE.md, BACKLOG.md, PROJECT.md, DISCOVERIES.md.
  Read phiếu Context `## Skills consulted` — design tokens already frozen.
  ...
```

**Lưu ý:**
- Example phải ngắn (≤10 dòng thêm). Đừng rewrite full session — chỉ chèn 1 block trước `[ARCHITECT DRAFT]` minh họa pattern.
- KHÔNG đổi state machine box (line 41-78). Skill-invocation là pre-DRAFT hoặc pre-EXECUTE side step, không phải state mới.

---

### Task 3: Update `phieu/TICKET_TEMPLATE.md` (optional Context subsection) + `docs/LAYERS.md` (access matrix)

**File 3.1:** `phieu/TICKET_TEMPLATE.md`

**Tìm:** End of `### Scope` block (line ~28-29 per anchor #7), before the `---` separator at line ~30.

**Thay bằng / Thêm:** New OPTIONAL subsection (HTML comment cho Architect biết khi nào dùng):
```markdown
### Skills consulted (optional)

<!-- Architect: nếu Orchestrator đã chạy skill (e.g., /frontend-design, /security-review) trước khi spawn em, paste output VERBATIM here. Frozen artifact for reproducibility. Most phiếu won't need this section — leave blank or delete it if no skill consulted. -->

<!-- Format example:
**Skill:** `/frontend-design` — invoked by Orchestrator 2026-05-10
**Output:**
[verbatim skill output here — design tokens, color palette, component spec, etc.]
-->
```

**Lưu ý:**
- OPTIONAL section — most phiếu skip. Không phải mandatory section như Task 0.
- Heading level `###` (subsection of `## Context`) — không phải `##` để tránh phình outline.
- Worker xác nhận heading level fits Context block hierarchy (Context → Vấn đề hiện tại / Giải pháp / Scope / Skills consulted) — nếu pattern hiện dùng `### ` cho 3 sub thì giữ; nếu dùng `## ` thì follow.

**File 3.2:** `docs/LAYERS.md`

**Tìm:** Access matrix table, line 19-25 per anchor #8 (rows: Vision/code/tickets/discovery/bash).

**Thay bằng / Thêm:** Add 1 row to the table (Worker decide: (a) extend table với 4th col Orchestrator [more invasive] HOẶC (b) add row "Skills" với cell value pointing to Orchestrator + footnote [recommended]).

Recommended (b):
```markdown
| Skills (`/frontend-design`, `/security-review`, etc.) | ❌ delegates | ❌ NO access | ❌ NO access |
```

Add footnote below table:
```markdown
**Skills note:** `Skill` tool is **Orchestrator-only** (main Claude Code session, the 4th role per `docs/ORCHESTRATION.md` "Why a 4th role"). Subagents (Architect / Worker) cannot invoke skills — outputs come pre-frozen in phiếu Context per `phieu/TICKET_TEMPLATE.md` `## Skills consulted` (P005, option B).
```

**Lưu ý:**
- Worker tự decide (a) vs (b) khi Read file — anchor #8 noted Architect lean (b) nhưng Tầng 2 = Worker's call. Either lựa chọn phải giữ table consistency.
- Footnote phải reference `docs/ORCHESTRATION.md` (4th role doc) — không phải định nghĩa lại Orchestrator role tại LAYERS.md.

---

### Task 4: Backlog flip + CHANGELOG entry + Discovery Report + index row

**File 4.1:** `docs/BACKLOG.md`

**Tìm:** Line 16 `- [ ] **[P005]** Worker Skill access — ...` and full P005 block (lines 16-20).

**Thay bằng:**
```markdown
- [x] ~~**[P005]** Worker Skill access~~ — **SHIPPED 2026-05-10 (option B locked).** Skills are Orchestrator-only; subagent allowlists kept tight (no `Skill` added). Outputs frozen in phiếu Context under `## Skills consulted`. Codified in `agents/orchestrator.md` "Invoking skills" section + `docs/ORCHESTRATION.md` Hard rule #9 + `phieu/TICKET_TEMPLATE.md` optional subsection. Discovery: `docs/discoveries/P005.md`.
```

**Tìm:** Line 59 P008 entry (`- [ ] **[P008]** Frontend-design plugin workflow doc...`).

**Thay bằng:**
```markdown
- [ ] **[P008]** Frontend-design plugin workflow doc — when phiếu touches FE/UI/UX → **Orchestrator** invokes `frontend-design` plugin (claude-plugins-official) BEFORE spawning Architect/Worker, freezes design tokens + component spec into phiếu Context under `## Skills consulted`. **RE-SCOPED 2026-05-10 post-P005 ship:** original draft assumed Worker invokes skill; option B inverts that — workflow doc now documents Orchestrator (main session) trigger criteria + invocation pattern, not Worker handbook entry. Target file: `phieu/FRONTEND_WORKFLOW.md` or section in `docs/ORCHESTRATION.md`.
```

**File 4.2:** `CHANGELOG.md`

**Tìm:** Top of file, before `## [v2.1.9] — 2026-05-10` (line 5 per anchor #10).

**Thay bằng / Thêm:** New version block:
```markdown
## [v2.1.10] — 2026-05-10

### Changed
- **P005: Worker Skill access — option B locked (Skills are Orchestrator-only).** ~2 weeks of A/B/C debate (started 2026-04-26) closed 2026-05-10. Option B: Orchestrator (main Claude Code session) invokes skills BEFORE spawning Architect/Worker, captures output verbatim, embeds in phiếu Context under `## Skills consulted` subsection as frozen artifact. Subagent `tools:` allowlists unchanged — `Skill` intentionally absent from both `agents/architect.md` and `agents/worker.md` (audit trail: option B = handbook codification, NOT tools-list change). Reproducibility: re-running a phiếu yields the same skill output.
- Files changed: `agents/orchestrator.md` (new section), `agents/architect.md` (1 sentence), `agents/worker.md` (1 sentence), `docs/ORCHESTRATION.md` (Hard rule #9 + example session paragraph), `phieu/TICKET_TEMPLATE.md` (optional `## Skills consulted` subsection), `docs/LAYERS.md` (access matrix Skills row + footnote), `docs/BACKLOG.md` (flip P005 + re-scope P008), `docs/discoveries/P005.md` (new), `docs/DISCOVERIES.md` (index row).
```

**File 4.3 (NEW):** `docs/discoveries/P005.md`

**Thay bằng / Thêm:** New file. Structure mirror P006 discovery format:
```markdown
# P005 — Worker Skill access: option B locked (Skills are Orchestrator-only)

**Date:** 2026-05-10
**Branch:** fix/P005-skills-orchestrator-only
**Tier:** 2 (handbook codification, skip-CHALLENGE)

---

## Decision: option B vs A vs C

3 options debated ~2 weeks (2026-04-26 → 2026-05-10):

| Option | Mechanism | Reject reason |
|---|---|---|
| **A** | Add `Skill` to Worker `tools:` allowlist (1-line edit) | Reproducibility risk — skill output varies with skill state at invocation time. Audit replay of phiếu post-ship cannot guarantee same output. Pragmatic but opens envelope. |
| **B** *(picked)* | Orchestrator (main session) runs skill, captures output, embeds in phiếu Context as frozen artifact. Subagents read but don't invoke. | **Picked 2026-05-10.** Allowlists tight, output frozen, no new field needed, codifies pattern that was already informally true (main session always has Skill, subagents don't). |
| **C** | Hybrid — Worker invokes skill only when phiếu has `requires_skill: <name>` flag | Adds field that Architect can forget → silent gap. Same envelope-opening issue as A when flag set. |

**Why B won:**
1. **Reproducibility** — skill output frozen at Architect-spawn time. Re-running phiếu gives same Context. A/C don't guarantee.
2. **Allowlist hygiene** — `Skill` stays out of `agents/architect.md` and `agents/worker.md` `tools:`. Envelope tight (the whole point of subagent role separation).
3. **No new field** — option C's `requires_skill:` adds a vector for Architect to forget. B uses existing `## Skills consulted` Context subsection (optional, no new schema).
4. **Codifies existing reality** — Claude Code main session natively has Skill tool; this was already true in practice. P005 just documents + enforces explicitly.

## Implementation

5 deliverables (all docs, 0 code, 0 `tools:` change):

1. `agents/orchestrator.md` — new section "Invoking skills (Skill tool) (P005)" (≤10 lines, respects ≤90 cap).
2. `docs/ORCHESTRATION.md` — Hard rule #9 + example session paragraph showing pre-DRAFT skill invocation.
3. `phieu/TICKET_TEMPLATE.md` — OPTIONAL `### Skills consulted` subsection in Context block.
4. `agents/architect.md` + `agents/worker.md` — 1 sentence each pointing to Context subsection.
5. `docs/LAYERS.md` — access matrix row "Skills" + footnote referencing Orchestrator (4th role).

Plus: BACKLOG flip (P005 closed) + P008 re-scope (Worker→Orchestrator invocation) + CHANGELOG v2.1.10 + this discovery + DISCOVERIES.md index row.

## Anchors `[needs Worker verify]` — resolved

| # | Assumption | Actual | Source |
|---|-----------|--------|--------|
| #13 | `agents/architect.md` "Inputs" section insertion point | [Worker fills with grep result + line number] | `grep -n "^## " agents/architect.md` |
| #16 | `.docs-gate.toml` config handles new `docs/discoveries/P005.md` + CHANGELOG staged | [Worker fills with pre-commit hook output] | `bash hooks/pre-commit` after staging |

## Notes

- **Pattern that was informally true:** main session has `Skill` natively in Claude Code; subagents register `tools:` allowlist without `Skill`. P005 surfaces this from "implicit / undocumented" to "explicit / handbook rule". No actual behavior change — Worker invocation of `Skill` would have failed at runtime anyway (not in allowlist).
- **P008 re-scope:** original P008 spec'd "Worker invokes `/frontend-design` for FE/UI phiếu". Option B inverts — Orchestrator invokes BEFORE spawning subagents. P008 now = Orchestrator handbook section + trigger criteria, not Worker handbook.
- **No tools-list edit anywhere.** That's the audit-trail signature of option B vs A/C: `git diff agents/*.md` should show frontmatter `tools:` line UNCHANGED. If diff shows `Skill` added, ship is wrong.
- **Tier escalation 2→1:** none. Phiếu remained Tầng 2 throughout — handbook codification, no schema/API/auth/dep change.

## Docs updated to match reality

`agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md`, `docs/ORCHESTRATION.md`, `phieu/TICKET_TEMPLATE.md`, `docs/LAYERS.md`, `docs/BACKLOG.md`, `CHANGELOG.md`, `docs/DISCOVERIES.md` (index row).
```

**File 4.4:** `docs/DISCOVERIES.md`

**Tìm:** Index table, after header row `|---|---|---|` (line 12), before `| [P006](discoveries/P006.md) ...` (line 13 per anchor #11).

**Thay bằng / Thêm:** Insert row newest-on-top:
```markdown
| [P005](discoveries/P005.md) | 2026-05-10 | Skills are Orchestrator-only — option B locked (frozen-artifact pattern in phiếu Context) |
```

**Lưu ý:**
- Newest-on-top per index file header instruction.
- Same date as P006 (2026-05-10) — order: P005 above P006 per "newest" definition (P005 ships AFTER P006 chronologically in this session).

---

### Task 5: Smoke test + commit + PR

**Steps:**
1. Run `bash hooks/pre-commit` (or `git commit --dry-run` equivalent) to verify docs-gate passes with:
   - New file `docs/discoveries/P005.md`
   - Modified `CHANGELOG.md` (entry v2.1.10 staged, < 1 day old per `changelog_max_age_days = 1`)
   - 8 modified docs files (orchestrator/architect/worker/ORCHESTRATION/TICKET_TEMPLATE/LAYERS/BACKLOG/DISCOVERIES)
2. If hook FAILS — `[needs Worker verify]` anchor #16 fired. Investigate: docs-gate config gap (P006 follow-up?) or staged-file rule mismatch. STOP, escalate via Discovery Report (Tier 2 boundary — config gap is config debt, not architecture).
3. Commit with message: `chore(P005): worker-skill-access codify option B (skills orchestrator-only)`. Branch `fix/P005-skills-orchestrator-only`.
4. Push + open PR. PR title: `chore(P005): Skills are Orchestrator-only (option B locked)`. PR body: paste CHANGELOG v2.1.10 entry verbatim.
5. Hand back to Sếp for nghiệm thu.

**Lưu ý:**
- Branch type prefix: `fix/` per Sếp's directive (BACKLOG note "branch fix/P005-...") — but commit type is `chore` since handbook codification, not bugfix. Slight inconsistency OK; phiếu workflow accepts.
- KHÔNG `git push --force` (P038 safety rail).
- KHÔNG merge — Sếp manual via `/ultrareview` + GitHub UI per CLAUDE.md "Recurring routines: Pre-merge any PR".

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `agents/orchestrator.md` | Task 1.1: new section "Invoking skills (Skill tool) (P005)" — 6-10 lines, cap ≤90 |
| `agents/architect.md` | Task 1.2: 1 bullet in DRAFT load-context list pointing to phiếu `## Skills consulted` |
| `agents/worker.md` | Task 1.3: 1 sentence after line 14 ("You have full code tools...") |
| `docs/ORCHESTRATION.md` | Task 2: Hard rule #9 + example session paragraph |
| `phieu/TICKET_TEMPLATE.md` | Task 3.1: optional `### Skills consulted` subsection in Context block |
| `docs/LAYERS.md` | Task 3.2: access matrix Skills row + footnote |
| `docs/BACKLOG.md` | Task 4.1: flip P005 to ✅ + re-scope P008 entry |
| `CHANGELOG.md` | Task 4.2: new entry `[v2.1.10] — 2026-05-10` |
| `docs/discoveries/P005.md` | Task 4.3: NEW file — discovery report |
| `docs/DISCOVERIES.md` | Task 4.4: index row newest-on-top above P006 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `agents/architect.md:4` `tools:` line | UNCHANGED — `Skill` MUST stay absent (option B audit signature) |
| `agents/worker.md:4` `tools:` line | UNCHANGED — same reason |
| `agents/orchestrator.md:4` `tools: []` | UNCHANGED — main session natively has Skill, frontmatter empty intentional |
| `phieu/phieu.sh` | UNCHANGED — no shell function changes for option B |
| `hooks/pre-commit` | UNCHANGED — docs-gate already handles new discovery file via P006 config |
| `docs/HANDOFF.md` | UNCHANGED — Sếp directive note: option B integrated into existing Handoff 2 (Architect → Worker) implicitly via phiếu Context; no new "Handoff 6" needed |
| `docs/PHILOSOPHY.md` | UNCHANGED — option B doesn't add 7th principle |
| Skills folder (`skills/*/SKILL.md`) | UNCHANGED — option B is about WHO invokes skills, not HOW skills work |

---

## Luật chơi (Constraints)

1. **NO `tools:` line edit anywhere.** Audit signature of option B vs A/C: `git diff agents/*.md | grep "^[+-]tools:"` MUST return zero matches. If `Skill` appears in any subagent allowlist diff, ship is wrong — revert + re-do per option B.
2. **Tier 2 boundary.** Phiếu touches handbook + template + index docs only. Triggers for 2→1 escalation (per `agents/worker.md` step 4a): schema (none), API contract (none), new dep (none), auth (none), cross-module data flow (none). Stay Tầng 2.
3. **Cap ≤90 lines for `agents/orchestrator.md`.** P035 spec, P038 confirmed. Worker MUST `wc -l` post-edit; if >90, condense `## Phiếu cleanup nudge (P038)` (currently 2 lines, can become 1).
4. **Voice:** handbook files (`agents/*.md`, `docs/*.md`) English per CLAUDE.md "Language" section. Phiếu body Vietnamese OK (this file). Discovery report English-leaning (mirrors P006 style).
5. **Frozen-artifact discipline.** When example session in `docs/ORCHESTRATION.md` shows skill invocation, output must be VERBATIM (or marked `[skill output 2026-MM-DD — see phiếu Context]` placeholder). No paraphrasing — that defeats reproducibility argument.
6. **OPTIONAL Context subsection.** `phieu/TICKET_TEMPLATE.md` `### Skills consulted` is OPTIONAL. Architect leaves it blank or removes the subsection if no skill consulted. NOT mandatory like Task 0.
7. **No `Skill` invocation by Architect/Worker** — even though phiếu adds documentation, neither subagent gets the tool. The frontmatter `tools:` lines are the enforcement.
8. **Tier 2 skip-CHALLENGE.** Per `docs/ORCHESTRATION.md` Tier routing: DRAFT → APPROVAL_GATE → EXECUTE. No Worker CHALLENGE expected. Debate Log seeded for Tier-escalation only.

---

## Nghiệm thu

### Automated
- [ ] `wc -l agents/orchestrator.md` ≤ 90 (cap respected post-Task 1.1)
- [ ] `bash hooks/pre-commit` passes (docs-gate handles new files + staged CHANGELOG)
- [ ] `git diff agents/architect.md agents/worker.md agents/orchestrator.md | grep "^[+-]tools:"` returns ZERO matches (option B audit signature)

### Manual Testing
- [ ] Read `agents/orchestrator.md` new section — 6-10 lines, mechanical voice, mirrors existing handbook style
- [ ] Read `docs/ORCHESTRATION.md` Hard rule #9 — concise, references P005, mentions "frozen artifact"
- [ ] Read `phieu/TICKET_TEMPLATE.md` `### Skills consulted` subsection — OPTIONAL marker clear, format example in HTML comment
- [ ] Read `docs/LAYERS.md` Skills row + footnote — references `docs/ORCHESTRATION.md` "Why a 4th role" for Orchestrator definition
- [ ] Read `docs/BACKLOG.md` P005 entry — flipped to ✅ with 2026-05-10 ship date + option-B note
- [ ] Read `docs/BACKLOG.md` P008 entry — re-scoped to Orchestrator-side, dependency note updated
- [ ] Read `CHANGELOG.md` v2.1.10 — entry concise, lists all 9 modified + 1 new files
- [ ] Read `docs/discoveries/P005.md` — option B reasoning explicit, A/C reject reasons listed, anchors resolved table

### Regression
- [ ] Spawn architect subagent (test invocation, e.g. `/idea` workflow) — frontmatter `tools:` still excludes `Skill`, agent loads OK
- [ ] Spawn worker subagent (next phiếu, real or test) — frontmatter `tools:` still excludes `Skill`, agent loads OK
- [ ] Existing phiếu (e.g., P006 done) still readable — `### Skills consulted` subsection backwards-compat (absent in old phiếu = OK, optional)
- [ ] `phieu-list` / `phieu-done` shell function unaffected (no `phieu.sh` edit)

### Docs Gate
- [ ] `CHANGELOG.md` — entry for v2.1.10 (Task 4.2)
- [ ] `docs/discoveries/P005.md` — NEW file (Task 4.3)
- [ ] `docs/DISCOVERIES.md` — index row P005 (Task 4.4)
- [ ] `docs/BACKLOG.md` — P005 flipped + P008 re-scoped (Task 4.1)

### Discovery Report
- [ ] Write to `docs/discoveries/P005.md` (per-phiếu file, P038 pattern) — Task 4.3 already specs full content
  - Decision: option B vs A vs C reasoning
  - Implementation: 5 deliverables + dependent updates
  - Anchors `[needs Worker verify]` — resolved (#13, #16)
  - Pattern note: option B codifies what was already informally true
  - P008 re-scope note
  - Tier escalations: None (stayed Tầng 2)
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md` — Task 4.4 already specs row format
