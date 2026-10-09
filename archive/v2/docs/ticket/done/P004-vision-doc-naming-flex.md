# PHIẾU P004: Vision doc naming flex — glob `docs/CHARACTER*.md` thay vì hard-code `docs/CHARACTER.md`

> **ID:** P004 — sibling phiếu của P003 (BACKLOG format flex). Cùng nguyên tắc: sos-kit consume Sếp-owned docs, không bắt rename.
> **Filename:** `docs/ticket/P004-vision-doc-naming-flex.md`
> **Branch:** `fix/P004-vision-doc-naming-flex`

---

> **Loại:** Bugfix (drift fix from Tarot dogfood — 2026-04-26)
> **Ưu tiên:** P1 — must close before single-command installer ships (per Active sprint goal in `docs/BACKLOG.md` line 13)
> **Ảnh hưởng:** `agents/architect.md` + `agents/worker.md` (+ sed-mirrors `.claude/agents/*.md`) + `docs/SETUP.md` Step 4d + `docs/HANDOFF.md` Handoff 0 (3 sites: lines 21, 28, 33) + `docs/LAYERS.md` (access matrix + inner box prose + responsibilities prose) + `docs/GENESIS.md` Phase 0 row
> **Dependency:** None. Sibling P003 (BACKLOG format flex, shipped commit `87735dc` branch `fix/P003-backlog-format-flexibility`) độc lập — P004 may ship parallel.

---

## Context

### Vấn đề hiện tại

`agents/architect.md` (DRAFT mode "Load context" list, line 55) và `agents/worker.md` (envelope CANNOT list, line 17) đều hard-code đường dẫn vision doc literal:

- Architect line 55: `` - `docs/CHARACTER.md` — voice (only if voice-facing work) ``
- Worker line 17: `` - Read `docs/PROJECT.md`, `docs/SOUL.md`, `docs/CHARACTER.md` — vision docs are Architect's domain ``

Tarot dogfood (2026-04-26) phát hiện: project's character file is named `docs/CHARACTER_CHI_HA.md` (named after the character Chị Hạ — convention "CHARACTER_<NAME>.md" cho project có >1 character hoặc named persona). Khi Architect được spawn:
1. **Architect side:** "Load context" Read step trên `docs/CHARACTER.md` returns "file not found" → Architect skip hoặc proceed without character voice context → phiếu voice-facing có rủi ro mất tiếng nói nhân vật.
2. **Worker side:** Worker envelope nói "CANNOT Read `docs/CHARACTER.md`" — file đó không tồn tại nên rule literal không bind. Nhưng nếu Worker grep / glob ngẫu nhiên trúng `docs/CHARACTER_CHI_HA.md` (cùng folder, cùng prefix) thì envelope intent (Worker không đọc vision docs) bị bypass — Worker có thể đọc vision doc mà không vi phạm literal rule.

Tarot's workaround: tạo **symlink** `docs/CHARACTER.md` → `docs/CHARACTER_CHI_HA.md`. Workaround này phải làm thủ công ở từng project, không scalable, dễ quên khi clone fresh, và ngược nguyên tắc của P003 ("sos-kit consume user-owned docs, không bắt rename").

Naming variant phổ biến (research từ Tarot + VISION_TEMPLATES):
- `CHARACTER.md` — single character (template default)
- `CHARACTER_<NAME>.md` — named character (Tarot's `CHARACTER_CHI_HA.md`)
- Optionally co-existing với `VOICE.md` (separate narrator voice, per `phieu/VISION_TEMPLATES/VOICE_template.md`).

Đồng thời docs reference `CHARACTER.md` literal ở các nơi user-facing khác (cần update để doc consistent với hành vi mới của agent — nếu không, user mới đọc onboarding guide sẽ confused, và doc drift là chính lý do P004 sinh ra):
- `docs/SETUP.md` Step 4d line 110 — `cp ... CHARACTER_template.md docs/CHARACTER.md` (instruction copy file đến tên canonical)
- `docs/HANDOFF.md` Handoff 0 line 21, 28, 33 — list vision docs gồm `CHARACTER.md` + commit-flow prose (V3 add — Worker grep at Turn 2 [O2.1])
- `docs/LAYERS.md` line 21 — access matrix row `Vision/strategy docs (PROJECT, SOUL, CHARACTER)`
- `docs/LAYERS.md` line 37 — Layer 1 (Chủ nhà) inner box prose: `Vision docs (PROJECT.md, SOUL.md, CHARACTER.md)` (V2 add — Worker grep at Turn 1 [O1.2])
- `docs/LAYERS.md` line 107 — Chủ nhà responsibility #1 prose: `PROJECT.md ... CHARACTER.md (voice)` (V2 add — Worker grep at Turn 1 [O1.2])
- `docs/GENESIS.md` line 16 — Phase 0 Vision table row Output column: `docs/PROJECT.md, docs/SOUL.md, docs/CHARACTER.md (nếu persona)` (V2 add — Worker grep at Turn 1 [O1.1])

### Giải pháp

**Phương án C — Hybrid (glob-tolerant envelope + canonical-name recommendation in SETUP):**

Cùng nguyên tắc P003 ("format-tolerant, never silent"). Hai thay đổi phối hợp, atomic 1 commit:

**A. Agents — glob `docs/CHARACTER*.md` (primary fix).**
1. **Architect** (`agents/architect.md` line 55, DRAFT mode "Load context" step 1): replace literal `` `docs/CHARACTER.md` `` với glob notation `` `docs/CHARACTER*.md` `` + 1-line behavior note: Architect uses `Glob("docs/CHARACTER*.md")` (Architect already has Glob tool per the agent's frontmatter `tools:` line). Read all matches. If multiple files match (e.g. `CHARACTER_CHI_HA.md` + `CHARACTER_NARRATOR.md`), read each — voice-facing work needs both.
2. **Worker** (`agents/worker.md` line 17, envelope "CANNOT" list): replace literal `` `docs/CHARACTER.md` `` với glob `` `docs/CHARACTER*.md` ``. Behavior: Worker MUST NOT Read any file matching `docs/CHARACTER*.md`. (Worker has Grep/Glob tools — the rule is "do not Read these," not "cannot detect their existence.") Add explicit note: glob covers all variants (`CHARACTER.md`, `CHARACTER_<NAME>.md`).
3. **Sed-mirrors** `.claude/agents/architect.md` + `.claude/agents/worker.md` regenerate via existing `scripts/sync-personal-agents.sh` (per P003 anchor #4 — script does `sed 's/Chủ nhà/Sếp/g' agents/$f > .claude/agents/$f`). No script changes; just rerun after edit.

**B. Docs — update reference + add canonical-name recommendation.**
1. **`docs/SETUP.md` Step 4d** (lines 105–110): keep the canonical `cp ... docs/CHARACTER.md` command as default (most projects = single character, name doesn't matter), but add a 2-line note BELOW the `cp` line explaining: if your project has a named character or multiple character/voice files, use `docs/CHARACTER_<NAME>.md` — Architect globs `docs/CHARACTER*.md` and reads all matches.
2. **`docs/HANDOFF.md` Handoff 0** (lines 21, 28, 33): replace `CHARACTER.md` literal mentions với `CHARACTER*.md` (or `CHARACTER.md` / `CHARACTER_<NAME>.md`) — keep prose readable, just signal that the literal isn't load-bearing. *V3 add per Turn 2 [O2.1] ACCEPT — line 28 was missed in V2 and creates intra-section inconsistency with lines 21+33.*
3. **`docs/LAYERS.md` access matrix** (line 21): change row label from `Vision/strategy docs (PROJECT, SOUL, CHARACTER)` to `Vision/strategy docs (PROJECT, SOUL, CHARACTER*)` — minimal edit, signals the glob.
4. **`docs/LAYERS.md` inner-box prose** (line 37 — Layer 1 Chủ nhà box): `CHARACTER.md` → `CHARACTER*.md` — surgical 1-char addition. *V2 add per Turn 1 [O1.2] ACCEPT.*
5. **`docs/LAYERS.md` Chủ nhà responsibility #1** (line 107): `CHARACTER.md (voice)` → `CHARACTER*.md (voice)` — surgical 1-char addition. *V2 add per Turn 1 [O1.2] ACCEPT.*
6. **`docs/GENESIS.md` Phase 0 row** (line 16): `docs/CHARACTER.md (nếu persona)` → `docs/CHARACTER*.md (nếu persona, e.g. docs/CHARACTER_CHI_HA.md)` — table cell stays single-line, adds 1 illustrative example. *V2 add per Turn 1 [O1.1] ACCEPT.*

**Out of scope (intentionally not touched):**
- `phieu/VISION_TEMPLATES/CHARACTER_template.md` — it's a template skeleton; the filename `CHARACTER_template.md` doesn't change. SETUP guides Sếp on how to copy + rename.
- `docs/PHILOSOPHY.md` — mentions "vision docs" conceptually but no literal `CHARACTER.md` path tied to envelope rule.
- `docs/BACKLOG.md` and `CHANGELOG.md` — no envelope rule there.
- `phieu/RELAY_PROTOCOL.md`, `phieu/DISCOVERY_PROTOCOL.md` — references CHARACTER conceptually only (Worker re-greps to confirm — anchor #6).
- `docs/LAYERS.md` skills map (line 162+) — Worker grep at Turn 1 found ONLY lines 21, 37, 107 as `CHARACTER` mentions in LAYERS.md (per [O1.2]). No further LAYERS sections need touching. Worker Turn 2 grep additionally confirmed lines 165 + 186 are inside skills map / template footnote — out of scope per existing Lưu ý.
- `README.md:112` agent capability table prose — Tầng 2 cosmetic (Worker Turn 2 confirmed not envelope-binding).

### Scope

- CHỈ sửa: `agents/architect.md`, `agents/worker.md`, `.claude/agents/architect.md` (regenerate), `.claude/agents/worker.md` (regenerate), `docs/SETUP.md`, `docs/HANDOFF.md` (3 sites: lines 21, 28, 33), `docs/LAYERS.md` (3 sites: access matrix + inner box + responsibilities prose), `docs/GENESIS.md` (1 site: Phase 0 row).
- KHÔNG sửa: `phieu/VISION_TEMPLATES/CHARACTER_template.md`, `docs/PHILOSOPHY.md`, `docs/BACKLOG.md`, `CHANGELOG.md` (CHANGELOG entry added at commit time, không phải sửa rule), `phieu/TICKET_TEMPLATE.md`, bất kỳ skill markdown nào trong `skills/`.

---

## Task 0 — Verification Anchors

> Architect read trực tiếp các file dưới (Read-only, không grep — Architect không có Grep tool). Worker re-greps + scans repo-wide để confirm không có site nào khác hard-code `docs/CHARACTER.md` literal mà phiếu chưa cover.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `agents/architect.md` line 55 contains `` - `docs/CHARACTER.md` — voice (only if voice-facing work) `` inside DRAFT mode "Load context" list (step 1, items bullet) | `grep -n "CHARACTER" agents/architect.md` (Worker) | ✅ V1 confirmed (Architect Read + Worker re-grep). |
| 2 | `agents/worker.md` line **17** contains literal `` Read `docs/PROJECT.md`, `docs/SOUL.md`, `docs/CHARACTER.md` — vision docs are Architect's domain `` inside the "You CANNOT" envelope list | `grep -n "CHARACTER" agents/worker.md` (Worker) | ✅ V2 — line corrected from 19 → 17 per Worker Turn 1 [O1.3]. Content text matches. Single occurrence. |
| 3 | Architect frontmatter `tools:` line includes `Glob` (so glob-based loading is already authorized — no new tool grant needed) | Read `agents/architect.md` line 4 | ✅ Line 4: `tools: Read, Write, Glob, ...`. |
| 4 | Worker frontmatter `tools:` line includes both `Glob` and `Grep` | Read `agents/worker.md` line 4 | ✅ Line 4: `tools: Read, Write, Edit, Glob, Grep, Bash, ...`. |
| 5 | `scripts/sync-personal-agents.sh` regenerates BOTH `.claude/agents/architect.md` AND `.claude/agents/worker.md` from `agents/*.md` via sed | `cat scripts/sync-personal-agents.sh` (Worker) | ✅ V1 Turn 1: script loops `for f in architect.md worker.md` — both handled. |
| 6 | No file outside the listed scope hard-codes `docs/CHARACTER.md` literal in a way that would silently break when project's file is named differently | `grep -rn "docs/CHARACTER" .` excluding `node_modules`, `.git`, `target/`, `dist/` (Worker) | ⚠️ V1 Turn 1 surfaced 2 additional sites: `docs/GENESIS.md:16` + `docs/LAYERS.md:37,107` (V2 incorporated). V2 Turn 2 surfaced 1 more in-Handoff-0 site: `docs/HANDOFF.md:28` (V3 incorporated). Worker re-grep at EXECUTE: expect rule-binding hits ONLY in 6 in-scope files (architect.md, worker.md, SETUP.md, HANDOFF.md, LAYERS.md, GENESIS.md) + intentional template self-reference in `phieu/VISION_TEMPLATES/CHARACTER_template.md`. Any other rule-binding hit → escalate Turn 3 (last turn — see cap clause below). |
| 7 | `docs/SETUP.md` Step 4d (line 110) literally contains `cp ~/path/to/sos-kit/phieu/VISION_TEMPLATES/CHARACTER_template.md docs/CHARACTER.md` — and the comment line above (line 109) says `# CHARACTER.md only if the product has an AI character / named voice` | Read `docs/SETUP.md` lines 105–115 | ✅ Architect Read confirmed verbatim. |
| 8 | `docs/HANDOFF.md` Handoff 0 references `CHARACTER.md` at lines 21 and 33 | Read `docs/HANDOFF.md` lines 17–37 | ✅ V1 confirmed. **V3 update:** anchor scope widened to lines 21, **28**, 33 — see anchor #16. |
| 9 | `docs/LAYERS.md` access matrix row at line 21 contains literal `Vision/strategy docs (PROJECT, SOUL, CHARACTER)` | Read `docs/LAYERS.md` line 21 | ✅ V1 confirmed. |
| 10 | `docs/PHILOSOPHY.md` does NOT contain envelope-binding `docs/CHARACTER.md` literal | `grep -n "CHARACTER" docs/PHILOSOPHY.md` (Worker) | ✅ V1 Turn 1: zero hits. |
| 11 | `docs/ORCHESTRATION.md` does NOT reference `CHARACTER.md` literal | `grep -n "CHARACTER" docs/ORCHESTRATION.md` (Worker) | ✅ V1 Turn 1: file does not exist. |
| 12 | `phieu/VISION_TEMPLATES/CHARACTER_template.md` first line is `# CHARACTER.md — <Character Name>` (intentional self-reference, NOT touched) | Read template line 1–3 | ✅ V1 confirmed. |
| 13 | `docs/LAYERS.md` line 37 contains the inner-box prose `│    • Vision docs (PROJECT.md, SOUL.md, CHARACTER.md)            │` inside Layer 1 (Chủ nhà) box | `grep -n "CHARACTER" docs/LAYERS.md` (Worker) | ✅ V2 — Architect Read of LAYERS.md lines 25–49 confirmed: line 37 reads `│    • Vision docs (PROJECT.md, SOUL.md, CHARACTER.md)            │`. Single source of truth for that inner box. |
| 14 | `docs/LAYERS.md` line 107 contains Chủ nhà responsibility #1 prose `**Maintain vision docs** — `PROJECT.md` (what it is), `SOUL.md` (why it exists), `CHARACTER.md` (voice). Architect reads these but doesn't write them.` | Read `docs/LAYERS.md` lines 95–115 | ✅ V2 — Architect Read confirmed line 107 verbatim. |
| 15 | `docs/GENESIS.md` line 16 contains the Phase 0 Vision row Output column `docs/PROJECT.md, docs/SOUL.md, docs/CHARACTER.md (nếu persona)` | Read `docs/GENESIS.md` lines 14–22 | ✅ V2 — Architect Read confirmed line 16 verbatim. |
| 16 | `docs/HANDOFF.md` line 28 contains the Chủ nhà workflow prose `  → Chủ nhà edits + commits to PROJECT.md / SOUL.md / CHARACTER.md` inside the Handoff 0 ASCII workflow block (between lines 25–31) | Read `docs/HANDOFF.md` lines 17–37 | ✅ V3 — Architect Read confirmed line 28 verbatim: `  → Chủ nhà edits + commits to PROJECT.md / SOUL.md / CHARACTER.md`. Inside the same Handoff 0 section as anchors #8 (lines 21, 33). |

**⏳ count:** 0 (all anchors resolved at V3 — Architect Read for V3-new #16; Worker Turn 1+2 grep covered #5, #6, #10, #11, #13, #14, #15)
**⚠️ count:** 1 (#6 — V1 expectation was "5-file scope"; V2 expanded to 6-file-scope; V3 expanded HANDOFF.md sites within that scope from 2 to 3; Worker re-greps at EXECUTE to confirm no further sites)
**❌ count:** 0

If Worker's grep at anchor #6 surfaces a rule-binding `docs/CHARACTER.md` reference outside the **6-file** scope (architect.md, worker.md, SETUP.md, HANDOFF.md, LAYERS.md, GENESIS.md), Worker MUST escalate via Debate Log Turn 3 — but Turn 3 is the **cap**: at Turn 3, Architect must either close (DEFEND/ACCEPT all remaining) or DEFER residual objections to a follow-up phiếu. No further refine-loops.

---

## Debate Log

> Auto-populated by Worker (CHALLENGE mode) and Architect (RESPOND mode). Sếp reads at approval gate — no mid-debate intervention unless orchestrator escalates (DEFER or 3-turn cap).
> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Phiếu version V1 → V2 → ... per Architect refine. Cap = 3 turns.

**Phiếu version:** V3 (Architect responded to Turn 2 — HANDOFF.md:28 added to Task 5 scope as sub-edit 5c)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**
- #1 ✅ `agents/architect.md:55` — exact text confirmed
- #2 ⚠️ Phiếu says "line 19" but actual line is **17** (`agents/worker.md:17`). Content matches; line number in Task 2 "Tìm" block is wrong.
- #3 ✅ `agents/architect.md:4` — `Glob` present in tools
- #4 ✅ `agents/worker.md:4` — `Glob` + `Grep` present
- #5 ✅ `scripts/sync-personal-agents.sh` loops `for f in architect.md worker.md` — both files handled
- #6 ⚠️ **SCOPE LEAK FOUND** — see Objection [O1.1] below
- #7 ✅ `docs/SETUP.md:109–110` verbatim confirmed
- #8 ✅ `docs/HANDOFF.md:21,33` both confirmed (re-grepped — no drift from other phiếu)
- #9 ✅ `docs/LAYERS.md:21` verbatim confirmed
- #10 ✅ `docs/PHILOSOPHY.md` — zero CHARACTER hits (grep returned empty)
- #11 ✅ `docs/ORCHESTRATION.md` — file does not exist (no hits possible; anchor expectation met)
- #12 (not re-grepped — template out of scope, already ✅)

**Objections:**

- [O1.1] **`docs/GENESIS.md:16` hard-codes `docs/CHARACTER.md` in a workflow table** — `agents/worker.md:3` frontmatter `description:` field also still says `(PROJECT/SOUL/CHARACTER)` (conceptual, not literal path — Tầng 2 polish only). The GENESIS.md hit is a structured table row: `| 0. Vision | ... | Chủ nhà | docs/PROJECT.md, docs/SOUL.md, docs/CHARACTER.md (nếu persona) |` — this is a *user-visible workflow guide*, not an agent envelope rule, so it won't silently break runtime. But it will be inconsistent documentation after P004 ships. GENESIS.md is not in phiếu's scope list (lines 63–66) and not in "Files KHÔNG sửa" table. Phiếu is silent on it.
  Tác động: post-P004, GENESIS.md still says the canonical name is `CHARACTER.md` with no glob note — new users reading GENESIS.md onboarding flow get conflicting guidance vs HANDOFF.md + SETUP.md.

- [O1.2] **`docs/LAYERS.md` inner boxes (lines 37, 107) also hard-code `CHARACTER.md` literal.** Task 6 only touches line 21 (access matrix row label). Lines 37 (`│ • Vision docs (PROJECT.md, SOUL.md, CHARACTER.md) │`) and 107 (`Maintain vision docs — PROJECT.md … CHARACTER.md (voice).`) are conceptual prose inside layer-detail boxes — not rule-binding. Phiếu's Task 6 Lưu ý (line 257) explicitly says "Do NOT update LAYERS.md skills map (line 162+) or layer-detail boxes (lines 31–76) unless Worker grep…". Worker grep found them. Per that clause: this becomes a scoping question — expand Task 6 scope or accept deliberate asymmetry.

- [O1.3] **`agents/worker.md:17` vs phiếu's stated line 19** — Task 2 "Tìm" block specifies `(line 19, ...)`. Actual line number is 17. Edit will still succeed (phiếu's exact-text match is correct), but stated line number is wrong — future phiếu referencing this as an anchor will cite wrong line.

**Proposed alternatives for [O1.1] — GENESIS.md:**
- A. **Add `docs/GENESIS.md:16` to Task 5 scope** (or a new Task 7): change `docs/CHARACTER.md` in that table cell to `docs/CHARACTER*.md` — 1-line edit, same pattern as HANDOFF.md line 21. (Worker lean — keeps docs consistent with zero architectural complexity; GENESIS.md is user-facing onboarding.)
- B. **Accept deliberate asymmetry**: add GENESIS.md to "Files KHÔNG sửa" with explicit rationale ("GENESIS is a historical narrative doc, naming flexibility is covered in SETUP.md which links to GENESIS flow"). Leaves one stale literal but avoids scope creep.

**Proposed alternatives for [O1.2] — LAYERS.md inner boxes:**
- A. **Expand Task 6 to also update lines 37 and 107**: add `*` to `CHARACTER.md` references in both inner boxes — 2 micro-edits, pure doc consistency. (Worker lean — phiếu's own Task 6 Lưu ý says "if Worker grep finds them → expand scope." Worker found them.)
- B. **Add them to "Files KHÔNG sửa"** with explicit rationale ("inner box prose = conceptual, only access matrix is authoritative").

**For [O1.3] — line number mismatch:**
- Self-correctable at EXECUTE time (exact-text match still works). Tầng 2 — Worker will adapt. No Architect response needed.

**Status:** ✅ ARCHITECT RESPONDED (see Turn 1 — Architect Response below)

### Turn 1 — Architect Response (phiếu V2)

Verdict per objection (Architect Read GENESIS.md:14–22 + LAYERS.md:25–49 + LAYERS.md:95–115 + phiếu V1 Task 6 Lưu ý):

- **[O1.1] GENESIS.md:16 → ACCEPT.** Reason: GENESIS is user-facing 0→1 onboarding doc — leaving literal `docs/CHARACTER.md` while the rest of the docs say "glob-tolerant naming" creates exactly the kind of drift P004 was filed to prevent (Tarot's symlink workaround happened *because* of similar inconsistency). Cost = 1-line surgical edit in a table cell; benefit = onboarding-day consistency. Action: new **Task 7** added below — change line 16's Output cell to `docs/PROJECT.md, docs/SOUL.md, docs/CHARACTER*.md (nếu persona, e.g. docs/CHARACTER_CHI_HA.md)`. Anchor #15 added.

- **[O1.2] LAYERS.md:37,107 → ACCEPT.** Reason: phiếu V1's own Task 6 Lưu ý explicitly delegated this decision to Worker grep ("expand scope if Worker grep finds them") — Worker grep DID find them, so the precondition for expansion fired. Honoring the phiếu's own clause = self-consistent. Cost = 2 surgical 1-char additions (`CHARACTER.md` → `CHARACTER*.md`); benefit = LAYERS.md fully internally consistent. Action: **Task 6 expanded** to include line 37 (Layer 1 inner box prose) and line 107 (Chủ nhà responsibility #1). Anchors #13 and #14 added. Task 6's Lưu ý updated to remove the "if grep finds" clause (it fired; we acted).

- **[O1.3] line 19 vs 17 → ACCEPT (mechanical fix).** Reason: Worker's exact-text match still works (Find/Replace is text-based, not line-based), but phiếu's stated line number is wrong and propagating wrong line numbers degrades anchor quality for future phiếu. Tầng 2 self-correction at EXECUTE was the V1 plan, but V2 fixes it in-place since we're already refining. Action: Task 2 "Tìm (line **17**, ...)" header updated. Anchor #2 line corrected. No semantic change to the edit itself.

Scope discipline: V2 expansion adds 3 micro-edits across 2 new files (1 in GENESIS, 2 in LAYERS) + 1 line-number correction. Total phiếu task count: 6 → 7. All edits surgical (≤2 chars or ≤1 line per site). 1h estimate still holds.

**Status:** ✅ RESPONDED — phiếu V1 → V2. AWAITING WORKER RE-CHALLENGE Turn 2.

*(If Worker Turn 2 finds further scope leaks or disputes V2's verdicts, Architect responds Turn 2 → V3. Cap = 3 turns total.)*

### Turn 2 — Worker Challenge (phiếu V2)

**Anchor verification:**
- #13 ✅ `docs/LAYERS.md:37` — exact text `│    • Vision docs (PROJECT.md, SOUL.md, CHARACTER.md)            │` confirmed. Task 6b Tìm block matches.
- #14 ✅ `docs/LAYERS.md:107` — exact text confirmed verbatim. Task 6c Tìm block matches.
- #15 ✅ `docs/GENESIS.md:16` — exact text confirmed. Task 7 Tìm block matches.
- Anchor #2 ✅ `agents/worker.md:17` — line 17, single occurrence, exact text confirmed.

**Scope leak post-V2 — NEW sites found outside 6-file scope:**
Grep `CHARACTER\.md` across all non-ticket files surfaces 3 categories of NEW hits not covered by any V2 task:

1. `docs/LAYERS.md:165` — skills map row: `PROJECT.md / SOUL.md / CHARACTER.md skeleton` (conceptual — Architect already read these, see phiếu line 68: "Worker Turn 1 grep covered all CHARACTER hits in LAYERS.md and found only lines 21, 37, 107. No further sections touched.") — **BUT Worker Turn 1 grep output was reported in the Debate Log summary, not the raw grep. Actual grep now shows lines 165 + 186 in LAYERS.md that were NOT listed in Turn 1 [O1.2] objection.** Line 165 is inside the skills map (explicitly in "Files KHÔNG sửa" / "Do NOT touch LAYERS.md skills map (line 162+)" — V2 Task 6 Lưu ý). Line 186 is a template reference footnote. Both are in the skills map section — phiếu's own Lưu ý confirms out-of-scope. No objection on these.
2. `docs/HANDOFF.md:28` — `Chủ nhà edits + commits to PROJECT.md / SOUL.md / CHARACTER.md` — this line is **inside Handoff 0 body** (lines 17–37) but is NOT listed in Task 5's two Tìm blocks (Task 5 only touches lines 21 and 33). Post-P004, line 28 will still say the literal `CHARACTER.md` while lines 21 and 33 say `CHARACTER*.md` — internal Handoff 0 inconsistency.
3. `README.md:112` — Worker capability table: "Read PROJECT.md / SOUL.md / CHARACTER.md (vision docs)" — user-facing but inside a "cannot" column in the agents table. Not an envelope rule; documents the restriction to users. Tầng 2 cosmetic.
4. `skills/init/SKILL.md` (lines 45, 109, 170, 181, 194), `skills/insight/SKILL.md` (lines 5, 17, 26, 58, 69, 97), `skills/plan/SKILL.md` (lines 28, 46), `phieu/VISION_TEMPLATES/TEST_CASES_template.md` (6 hits), `phieu/VISION_TEMPLATES/DESIGN_SPEC_template.md` (3 hits), `phieu/VISION_TEMPLATES/SOUL_template.md` (3 hits), `phieu/VISION_TEMPLATES/VOICE_template.md` (2 hits) — all template/skill self-references, no runtime envelope binding. All explicitly in "Files KHÔNG sửa" scope or covered by anchor #6's "template self-reference" exception.

**Objections (Tầng 1 only):**

- [O2.1] **`docs/HANDOFF.md:28` is a missed literal inside Handoff 0** — `agents/worker.md:3` frontmatter description also mentions `CHARACTER` but that is conceptual (anchor #6 exception); line 28 is different: `Chủ nhà edits + commits to PROJECT.md / SOUL.md / CHARACTER.md` is an explicit file-path instruction inside the same Handoff 0 section that Task 5 targets (lines 21 + 33). V2 Task 5 patches lines 21 and 33 but leaves 28 with the old literal. Post-ship, a user reading Handoff 0 in sequence sees line 21 saying `CHARACTER_<NAME>.md` is valid, then line 28 saying "commit to `CHARACTER.md`" — contradiction in the same section. Not a runtime break, but exactly the doc-drift class P004 is filed to prevent (per Constraint 7, "Docs consistency principle").
  Cite: `/Users/nguyenhuuanh/sos-kit/docs/HANDOFF.md:28`

**Proposed alternatives for [O2.1]:**
- A. **Add line 28 to Task 5 scope** (Worker lean): Tìm `CHARACTER.md` on line 28, Thay bằng `CHARACTER*.md` — 1-char addition, same surgical pattern as 6a/6b. Zero architectural complexity; Architect Read of HANDOFF.md already confirmed context. Keeps Constraint 7 satisfied within the same commit.
- B. **Accept deliberate asymmetry on line 28**: add explicit note in "Files KHÔNG sửa" — "HANDOFF.md:28 is prose instruction for Chủ nhà, who understands naming convention from lines 21+33; literal `CHARACTER.md` there is acceptable shorthand." Leaves 1 residual literal with documented rationale.

**Atomicity assessment:** 7 tasks / 1 commit — still coherent. All edits are 1-char additions or 1-line table-cell changes in markdown. 6a/6b/6c in Task 6 adds 3 micro-edits to the same file — normal for a "surgically update all references in LAYERS.md" task. No atomic bloat concern.

**Nghiệm thu regression check — V2 Nghiệm thu grep coverage:**
- Nghiệm thu automated check: `grep -rn "CHARACTER\*" agents/ docs/` — covers architect.md (Task 1), worker.md (Task 2), SETUP.md (Task 4), HANDOFF.md lines 21+33 (Task 5), LAYERS.md lines 21+37+107 (Task 6 3-sites), GENESIS.md line 16 (Task 7). Coverage is complete for all 7 tasks if O2.1 is not accepted. If O2.1-A accepted → HANDOFF.md:28 would also need to appear in this grep.
- Test F added in Nghiệm thu Manual Testing covers 6a/6b/6c + Task 7 visual regression. Adequate.

**Status:** ✅ ARCHITECT RESPONDED (see Turn 2 — Architect Response below)

### Turn 2 — Architect Response (phiếu V3)

Verdict per objection (Architect Read HANDOFF.md:15–37 — line 28 verbatim confirmed: `  → Chủ nhà edits + commits to PROJECT.md / SOUL.md / CHARACTER.md`):

- **[O2.1] HANDOFF.md:28 → ACCEPT (Alternative A).** Reason: Worker's argument is structurally identical to V2's [O1.2] ACCEPT reasoning — line 28 sits *inside the same Handoff 0 section* (lines 17–37) as lines 21 and 33 which Task 5 already patches. Leaving 28 literal while patching 21+33 creates **intra-section** inconsistency, which is strictly worse than inter-doc inconsistency: a user reads Handoff 0 top-to-bottom and sees the convention shift mid-section. Constraint 7 ("Docs consistency principle") was added in V2 specifically for this class of drift. Honoring it requires accepting [O2.1]. Cost = 1-char edit (`CHARACTER.md` → `CHARACTER*.md`) inside an ASCII workflow block; structurally indistinguishable from edits 6a/6b/6c. Alternative B (deliberate asymmetry with rationale) was rejected — the rationale would itself contradict Constraint 7 we just wrote in V2.

  Action:
  1. Task 5 expanded to **3 sub-edits** (5a, 5b, 5c) mirroring Task 6's 3-sub-edit shape. 5a = line 21 (was V1 first edit), 5b = line 33 (was V1 second edit), **5c = line 28 (V3 new)**.
  2. Anchor #16 added: confirms Architect Read of HANDOFF.md lines 17–37 verifies line 28's exact text.
  3. Anchor #8 result column updated: scope widened from "lines 21, 33" to "lines 21, 28, 33" within the same Handoff 0 section.
  4. Header `Ảnh hưởng:` line updated: `docs/HANDOFF.md Handoff 0 (3 sites: lines 21, 28, 33)`.
  5. Files cần sửa table updated: HANDOFF.md row now reads "Task 5: lines 21 + 28 + 33".
  6. Nghiệm thu automated grep updated: `grep -rn "CHARACTER\*" agents/ docs/` expected hit count raised by 1 (HANDOFF.md now contributes 3 hits, not 2).
  7. Solution section B.2 updated to list all three line numbers (21, 28, 33).

Scope discipline: V3 expansion adds 1 micro-edit (1-char addition) in a file already being touched. Zero new files in scope. Total phiếu task count stays at 7 (Task 5 grows internally from 2 sub-edits to 3, parallel to Task 6's 3-sub-edit structure). All edits remain surgical (≤2 chars per site within Task 5). 1h estimate still holds.

**3-turn cap reminder:** This is Turn 2 of 3. If Worker Turn 3 surfaces another in-scope site, Architect MUST either ACCEPT inline (one final refine) or DEFER to a follow-up phiếu (no further turn). If Worker Turn 3 finds zero new objections → consensus reached, ready for Sếp approval gate.

**Status:** ✅ RESPONDED — phiếu V2 → V3. AWAITING WORKER RE-CHALLENGE Turn 3 (final turn — cap).

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Sếp: [date] — code execution may begin

---

## Nhiệm vụ

### Task 1: Architect agent — glob `docs/CHARACTER*.md` instead of literal

**File:** `agents/architect.md`

**Tìm (line 55, inside DRAFT mode "Load context" list, item 1 sub-bullets):**
```
   - `docs/CHARACTER.md` — voice (only if voice-facing work)
```

**Thay bằng:**
```
   - `docs/CHARACTER*.md` — voice (only if voice-facing work). Use `Glob("docs/CHARACTER*.md")` first; Read every match (covers `CHARACTER.md`, `CHARACTER_<NAME>.md`, etc.). Multi-character / multi-voice projects may have several files.
```

**Lưu ý:**
- Architect's `tools:` line (line 4) already includes `Glob` — no frontmatter change needed (anchor #3).
- Order in the list stays: load CLAUDE.md → docs/CLAUDE.md → BACKLOG → PROJECT → SOUL → **CHARACTER\*** → DISCOVERIES → TICKET_TEMPLATE → guides. Glob-then-Read is one logical step (still item 1's sub-bullet).
- The "(only if voice-facing work)" condition stays unchanged — voice-facing detection is still Architect's judgment call.

### Task 2: Worker agent — glob `docs/CHARACTER*.md` in envelope CANNOT list

**File:** `agents/worker.md`

**Tìm (line 17, inside "You CANNOT (this is the symmetric constraint to Architect)" list — V2: line corrected from 19 → 17 per Turn 1 [O1.3]):**
```
- Read `docs/PROJECT.md`, `docs/SOUL.md`, `docs/CHARACTER.md` — vision docs are Architect's domain
```

**Thay bằng:**
```
- Read `docs/PROJECT.md`, `docs/SOUL.md`, or any `docs/CHARACTER*.md` file (`CHARACTER.md`, `CHARACTER_<NAME>.md`, etc.) — vision docs are Architect's domain. Worker MAY use `Glob` / `Grep` to detect these files exist but MUST NOT `Read` their contents.
```

**Lưu ý:**
- Worker has Glob + Grep tools (anchor #4) — they're allowed for *detecting* files; the envelope rule is about not *reading* contents (the leakage vector that lets Worker re-architect from vision).
- Single replacement; no other CHARACTER reference in `agents/worker.md` per anchor #2.
- Other vision docs (`docs/VOICE.md`, `docs/TEST_CASES.md`, `docs/DESIGN_SPEC.md` from `phieu/VISION_TEMPLATES/`) are NOT added to this list in this phiếu — Sếp wants P004 scoped to the CHARACTER glob fix only. If those files also leak vision context, that's a follow-up phiếu (open backlog candidate).

### Task 3: Regenerate sed-mirrors `.claude/agents/architect.md` + `.claude/agents/worker.md`

**File:** none directly — run existing script.

**Tìm:** N/A — execution step.

**Thay bằng:** run `scripts/sync-personal-agents.sh` after Tasks 1+2 are saved. Per P003 anchor #4 (already verified): the script does `sed 's/Chủ nhà/Sếp/g' agents/$f > .claude/agents/$f` for both `architect.md` and `worker.md`. No script edits required — just re-execute.

**Lưu ý:**
- Anchor #5 confirmed (Worker Turn 1) that the script regenerates BOTH files. Just re-run.
- Verify post-run: `diff agents/architect.md .claude/agents/architect.md` should differ ONLY by `Chủ nhà` → `Sếp` substitution (matching P003's verification pattern).
- Same for `agents/worker.md` ↔ `.claude/agents/worker.md`.
- Commit both `agents/*.md` and `.claude/agents/*.md` together — they MUST stay in sync.

### Task 4: `docs/SETUP.md` — add canonical-name recommendation note

**File:** `docs/SETUP.md`

**Tìm (lines 109–110, inside Step 4d "Copy vision doc skeletons"):**
```
# CHARACTER.md only if the product has an AI character / named voice
cp ~/path/to/sos-kit/phieu/VISION_TEMPLATES/CHARACTER_template.md docs/CHARACTER.md
```

**Thay bằng:**
```
# CHARACTER.md only if the product has an AI character / named voice.
# If your project has a named character or multiple character/voice files,
# rename to docs/CHARACTER_<NAME>.md (e.g. docs/CHARACTER_CHI_HA.md).
# Architect globs docs/CHARACTER*.md and reads every match — naming is flexible.
cp ~/path/to/sos-kit/phieu/VISION_TEMPLATES/CHARACTER_template.md docs/CHARACTER.md
```

**Lưu ý:**
- Comment block above the `cp` command (3 lines added). The `cp` command itself stays — `docs/CHARACTER.md` remains the canonical default for projects with a single unnamed character.
- Do NOT add a `mv` step — let Sếp rename if/when they need.
- Do NOT change `phieu/VISION_TEMPLATES/CHARACTER_template.md` filename or its first-line title (anchor #12 — out of scope).

### Task 5: `docs/HANDOFF.md` Handoff 0 — signal naming flex (3 sub-edits)

> **V3 expansion** per Turn 2 [O2.1] ACCEPT — Task 5 now covers 3 sites in HANDOFF.md Handoff 0 (was 2 in V2). Edits are 1-char or short-phrase additions; structural pattern matches Task 6.

**File:** `docs/HANDOFF.md`

**Tìm 5a (line 21, inside Handoff 0's vision-doc list):**
```
- `CHARACTER.md` — voice / persona / tone (if the product has a character like Chị Hạ)
```

**Thay bằng 5a:**
```
- `CHARACTER.md` (or `CHARACTER_<NAME>.md` for named characters, e.g. `CHARACTER_CHI_HA.md`) — voice / persona / tone (if the product has a character like Chị Hạ)
```

**Tìm 5b (line 33, Architect's session-open reading order):**
```
**Architect's response:** on opening a new Claude Web session, read vision docs in order: `CLAUDE.md` → `PROJECT.md` → `SOUL.md` → `CHARACTER.md` → DISCOVERIES.md → request-specific guides. Then confirm "I've loaded context" before writing phiếu.
```

**Thay bằng 5b:**
```
**Architect's response:** on opening a new Claude Web session, read vision docs in order: `CLAUDE.md` → `PROJECT.md` → `SOUL.md` → `CHARACTER*.md` (glob — every match) → DISCOVERIES.md → request-specific guides. Then confirm "I've loaded context" before writing phiếu.
```

**Tìm 5c (line 28, Chủ nhà workflow ASCII block — V3 add per Turn 2 [O2.1]):**
```
  → Chủ nhà edits + commits to PROJECT.md / SOUL.md / CHARACTER.md
```

**Thay bằng 5c:**
```
  → Chủ nhà edits + commits to PROJECT.md / SOUL.md / CHARACTER*.md
```

**Lưu ý:**
- Three surgical edits in HANDOFF.md, all inside Handoff 0 (lines 17–37). Other Handoffs (1–4, 2.5) don't reference CHARACTER literal — verified by Architect Read of full HANDOFF.md + Worker Turn 1 grep.
- Keep "Chị Hạ" example in line 21 — it's a real-world reference that helps Sếp understand naming pattern.
- For 5c: the line is inside an indented ASCII workflow block (lines 25–31). Preserve the leading 2-space indent + `→ ` arrow exactly. Worker matches by exact text — line indent is part of the match.

### Task 6: `docs/LAYERS.md` — signal glob in access matrix + inner box + responsibility prose

> **V2 expansion** per Turn 1 [O1.2] ACCEPT — Task 6 now covers 3 sites in LAYERS.md (was 1 in V1). Edits are 1-char additions; total LAYERS.md change ≈3 chars.

**File:** `docs/LAYERS.md`

**Tìm 6a (line 21, access matrix table row):**
```
| Vision/strategy docs (PROJECT, SOUL, CHARACTER) | ✏️ maintain | 📖 read | 📖 read |
```

**Thay bằng 6a:**
```
| Vision/strategy docs (PROJECT, SOUL, CHARACTER*) | ✏️ maintain | 📖 read | 📖 read |
```

**Tìm 6b (line 37, Layer 1 Chủ nhà inner-box prose — V2 add per [O1.2]):**
```
│    • Vision docs (PROJECT.md, SOUL.md, CHARACTER.md)            │
```

**Thay bằng 6b:**
```
│    • Vision docs (PROJECT.md, SOUL.md, CHARACTER*.md)           │
```

**Tìm 6c (line 107, Chủ nhà responsibility #1 prose — V2 add per [O1.2]):**
```
1. **Maintain vision docs** — `PROJECT.md` (what it is), `SOUL.md` (why it exists), `CHARACTER.md` (voice). Architect reads these but doesn't write them.
```

**Thay bằng 6c:**
```
1. **Maintain vision docs** — `PROJECT.md` (what it is), `SOUL.md` (why it exists), `CHARACTER*.md` (voice — glob covers `CHARACTER.md` and named variants like `CHARACTER_CHI_HA.md`). Architect reads these but doesn't write them.
```

**Lưu ý:**
- Three surgical edits in LAYERS.md. 6a and 6b are 1-char additions (asterisk). 6c adds the asterisk plus a parenthetical clarifying the glob — slightly longer because this is the responsibility definition prose where readers expect the "why."
- For 6b: the trailing space inside the ASCII box must be adjusted (1 char shorter to keep the box border `│` aligned). Box width is 65 chars between borders. After change: `│    • Vision docs (PROJECT.md, SOUL.md, CHARACTER*.md)           │` — Worker counts spaces and aligns; this is Tầng 2 cosmetic detail.
- Do NOT touch LAYERS.md skills map (line 162+) — Worker Turn 1+2 grep covered all CHARACTER hits in LAYERS.md and found lines 21, 37, 107 (in scope) + lines 165, 186 (skills map / template footnote, out of scope per existing exclusion). No further sections touched.

### Task 7: `docs/GENESIS.md` Phase 0 row — signal glob + add example

> **V2 new task** per Turn 1 [O1.1] ACCEPT.

**File:** `docs/GENESIS.md`

**Tìm (line 16, Phase 0 Vision row in pipeline table):**
```
| 0. Vision | `/init` (skill) | Chủ nhà | `docs/PROJECT.md`, `docs/SOUL.md`, `docs/CHARACTER.md` (nếu persona) |
```

**Thay bằng:**
```
| 0. Vision | `/init` (skill) | Chủ nhà | `docs/PROJECT.md`, `docs/SOUL.md`, `docs/CHARACTER*.md` (nếu persona, e.g. `docs/CHARACTER_CHI_HA.md`) |
```

**Lưu ý:**
- Single table-cell edit. Pattern matches Task 5a (line 21 in HANDOFF.md): signal glob + give 1 example.
- Markdown table column count and pipe alignment unchanged (cell width grows by ~25 chars but markdown tables don't enforce column widths, only column count — Worker verifies render).
- Do NOT touch other GENESIS.md sections — Architect Read of GENESIS.md lines 1–22 confirmed no other CHARACTER literal in pipeline table or "Khái niệm cốt lõi." Worker re-grep covers full file via anchor #6.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `agents/architect.md` | Task 1: line 55 — `docs/CHARACTER.md` → `docs/CHARACTER*.md` + Glob-then-Read note |
| `agents/worker.md` | Task 2: line **17** — `docs/CHARACTER.md` → `docs/CHARACTER*.md` + Glob/Grep-detect-only note |
| `.claude/agents/architect.md` | Task 3: regenerate via `scripts/sync-personal-agents.sh` |
| `.claude/agents/worker.md` | Task 3: regenerate via `scripts/sync-personal-agents.sh` |
| `docs/SETUP.md` | Task 4: lines 109–110 — add 3-line comment block recommending `CHARACTER_<NAME>.md` for named characters |
| `docs/HANDOFF.md` | Task 5: lines 21 + **28** + 33 (V3: 3 sub-edits in Handoff 0) — note naming variant + glob in vision-doc list, workflow ASCII block, and reading order |
| `docs/LAYERS.md` | Task 6: line 21 (access matrix) + line 37 (Layer 1 inner box) + line 107 (Chủ nhà responsibility #1) — `CHARACTER` / `CHARACTER.md` → `CHARACTER*` / `CHARACTER*.md` |
| `docs/GENESIS.md` | Task 7: line 16 (Phase 0 Vision row Output cell) — `docs/CHARACTER.md (nếu persona)` → `docs/CHARACTER*.md (nếu persona, e.g. docs/CHARACTER_CHI_HA.md)` |
| `CHANGELOG.md` | Docs Gate: new entry under `## [v2.1.3]` (or next version) — see Nghiệm thu |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `phieu/VISION_TEMPLATES/CHARACTER_template.md` | Title `# CHARACTER.md — <Character Name>` stays unchanged (template's own self-reference, NOT a rule binding — anchor #12) |
| `docs/PHILOSOPHY.md` | No rule-binding `docs/CHARACTER.md` literal exists (anchor #10) — if Worker grep finds one → escalate |
| `docs/ORCHESTRATION.md` | No `CHARACTER.md` reference (anchor #11) — file does not exist |
| `docs/BACKLOG.md` | P004 line 17 stays in Active sprint (will move to "Recently shipped" by Sếp post-merge, manual edit, NOT part of this commit) |
| `docs/DISCOVERIES.md` | New entry written by Worker per Discovery Report section (newest on top) — appended, not replacing |
| `phieu/RELAY_PROTOCOL.md`, `phieu/DISCOVERY_PROTOCOL.md` | Conceptual mentions of CHARACTER (if any) stay as-is — they describe Architect's domain, not enforce a literal path. Worker grep at anchor #6 confirms no rule-binding |
| `skills/*/SKILL.md` | Worker confirms no CHARACTER.md literal binding inside any skill prompt at anchor #6 (Worker Turn 2 already grep'd — only conceptual mentions in init/insight/plan; out of scope) |
| `docs/LAYERS.md` skills map (line 162+) | Worker Turn 1+2 grep covered LAYERS.md repo-wide — only lines 21, 37, 107 are in scope. Lines 165 + 186 (skills map / template footnote) explicitly out of scope per the section exclusion. |
| `README.md:112` agent capability table | Tầng 2 cosmetic — Worker Turn 2 confirmed not envelope-binding (it's a user-facing description of the rule, not the rule itself). Out of P004 scope. |
| `hooks/pre-commit`, `integrations/**` | No CHARACTER references expected (anchor #6 covers via repo-wide grep) |

---

## Luật chơi (Constraints)

1. **Atomic commit** — Tasks 1–7 ship in ONE commit on branch `fix/P004-vision-doc-naming-flex`. The agents and the docs that describe their behavior must move together; otherwise a partial state has docs lying about what agents do (or vice-versa).
2. **Sed-mirror sync mandatory** — `agents/*.md` and `.claude/agents/*.md` MUST stay in lockstep. Verify via `diff` post-`sync-personal-agents.sh` before commit.
3. **No filename rename** — do NOT rename `phieu/VISION_TEMPLATES/CHARACTER_template.md`. Do NOT rename the canonical default `docs/CHARACTER.md` in SETUP.md's `cp` command. The fix is *tolerance for variants*, not *new canonical name*.
4. **Glob, not regex** — use shell glob notation (`docs/CHARACTER*.md`) in markdown, NOT a regex like `docs/CHARACTER.*\.md`. Glob is universally readable; regex confuses non-Bash readers.
5. **Worker `Read`-vs-`Glob` distinction is load-bearing** — Task 2's note clarifies Worker MAY Glob/Grep to detect existence but MUST NOT Read contents. This preserves the "envelope = no vision leak" intent.
6. **Backwards compat** — projects whose CHARACTER file is named exactly `docs/CHARACTER.md` MUST continue to work unchanged. Glob `docs/CHARACTER*.md` matches `docs/CHARACTER.md` (the `*` matches zero characters). Worker verifies in manual test.
7. **Docs consistency principle (V2 add, V3 enforced harder)** — every user-facing doc that names vision files (GENESIS Phase 0, LAYERS access matrix + inner boxes + responsibilities, HANDOFF Handoff 0 — all 3 sites: lines 21, 28, 33, SETUP Step 4d) MUST signal the glob convention. Inconsistency between agents (which glob) and docs (which name literally) — or worse, **intra-section inconsistency within a single Handoff/section** — = exactly the drift class P004 was filed to prevent. Worker grep at Docs Gate enforces this.
8. **Discovery Report mandatory** — Worker writes `docs/DISCOVERIES.md` entry post-execute even if every assumption was correct. Explicit "None" in WRONG section proves Worker checked.

---

## Nghiệm thu

### Automated

- [ ] `bash scripts/sync-personal-agents.sh` runs without error and regenerates BOTH `.claude/agents/architect.md` and `.claude/agents/worker.md`.
- [ ] `diff agents/architect.md .claude/agents/architect.md` shows ONLY `Chủ nhà` → `Sếp` differences (no other drift).
- [ ] `diff agents/worker.md .claude/agents/worker.md` shows ONLY `Chủ nhà` → `Sếp` differences.
- [ ] `grep -rn "docs/CHARACTER\.md" agents/ .claude/agents/ docs/SETUP.md docs/HANDOFF.md docs/LAYERS.md docs/GENESIS.md` returns ZERO rule-binding results (the only remaining `docs/CHARACTER.md` in scope is inside SETUP.md's `cp ... docs/CHARACTER.md` default-target line — which is NOT a rule binding but a default `cp` target with the new glob-tolerance note above it).
- [ ] `grep -rn "CHARACTER\*" agents/ docs/` returns **≥8 hits** (V3 raised from ≥7): architect.md (Task 1), worker.md (Task 2), SETUP.md note block (Task 4), HANDOFF.md line 21 + line 28 + line 33 (Task 5 — **3 hits, V3 added line 28**), LAYERS.md line 21 + line 37 + line 107 (Task 6), GENESIS.md line 16 (Task 7). Plus regenerated `.claude/agents/*.md` mirrors.
- [ ] **V3 add — intra-section consistency check:** `grep -n "CHARACTER" docs/HANDOFF.md` returns hits ONLY on lines containing `CHARACTER*` or `CHARACTER_<NAME>` — no bare `CHARACTER.md` literal anywhere in HANDOFF.md after V3 ships. Specifically, the 3 hits inside Handoff 0 (lines 21, 28, 33) all show the glob form.
- [ ] No new lint/type errors (this phiếu is markdown-only; no test target).

### Manual Testing

- [ ] **Test A — backwards compat (single-character project):**
  Create scratch repo `/tmp/sos-test-A` with `docs/PROJECT.md` (any content) + `docs/SOUL.md` (any) + `docs/CHARACTER.md` (any) + minimal `docs/BACKLOG.md` with `## Active sprint\n- [ ] [P001] dummy`. Spawn Architect (DRAFT) for "P001 dummy." Verify Architect's load-context output includes `docs/CHARACTER.md` (per Glob+Read of `docs/CHARACTER*.md`).
- [ ] **Test B — named-character project (Tarot reproduction):**
  Same scratch repo, RENAME `docs/CHARACTER.md` → `docs/CHARACTER_CHI_HA.md`. Spawn Architect (DRAFT) again. Verify Architect now reads `docs/CHARACTER_CHI_HA.md` (no symlink workaround). No "file not found" error.
- [ ] **Test C — multi-character project:**
  Same scratch repo, add second file `docs/CHARACTER_NARRATOR.md` alongside `docs/CHARACTER_CHI_HA.md`. Spawn Architect (DRAFT). Verify Architect reads BOTH files (Glob returns both, Read each).
- [ ] **Test D — Worker envelope respects glob:**
  Same scratch repo (Test C state). Spawn Worker (CHALLENGE) on a dummy phiếu. Verify Worker does NOT Read either CHARACTER\*.md file's contents during its workflow. Worker MAY Glob to detect they exist (e.g. as part of an "anchor #X — verify file Y" check) — that's allowed.
- [ ] **Test E — empty case:**
  New scratch repo `/tmp/sos-test-E` with NO `docs/CHARACTER*.md` file at all. Spawn Architect (DRAFT). Verify Architect doesn't crash; Glob returns 0 matches; Architect proceeds without character context (logs "no character file" or equivalent — Tầng 2 wording, Worker decides).
- [ ] **Test F (V2 add, V3 expanded) — onboarding doc consistency:**
  Render `docs/GENESIS.md`, `docs/LAYERS.md`, AND `docs/HANDOFF.md` (any markdown viewer or `cat`). Confirm: (a) Phase 0 Vision row + LAYERS access matrix + Layer 1 inner box + Chủ nhà responsibility #1 ALL show `CHARACTER*` or `CHARACTER*.md`; (b) **V3 — HANDOFF.md Handoff 0 reads consistently top-to-bottom: lines 21, 28, 33 all use glob form; no bare `CHARACTER.md` literal anywhere in section**. New user reading these docs in sequence sees consistent glob signal — no mid-section convention shift.

### Regression

- [ ] **P003 unaffected:** banner script still resolves Active sprint per P003 logic; Architect Hard rule 0 still gates phiếu drafting per P003. P004's edits don't touch P003's lines.
- [ ] **Handoff 0 reading order still legible:** human-readable order chain on line 33 stays parseable (the `→` arrows still flow).
- [ ] **V3 add — Handoff 0 ASCII workflow block (lines 25–31) still aligns:** line 28's leading 2-space indent + `→ ` arrow preserved post-edit. Worker eyeballs the workflow block top-to-bottom.
- [ ] **LAYERS.md table still renders:** the `*` added to `CHARACTER` does not break Markdown table column count or alignment.
- [ ] **LAYERS.md ASCII box still aligns (V2 add):** line 37's box border `│` on the right margin stays aligned after `CHARACTER.md` → `CHARACTER*.md` (1 char added → adjust trailing spaces by 1). Worker eyeballs.
- [ ] **GENESIS.md table still renders (V2 add):** line 16 row stays a valid pipe-separated row; column count = 4 (Phase / Skill / Vai / Output).
- [ ] **SETUP.md Step 4 still scriptable copy-paste:** the new comment lines (3 lines, all starting with `#`) are valid shell comments — Sếp can still copy-paste the whole Step 4d block into terminal without error.

### Docs Gate

- [ ] `CHANGELOG.md` — new entry under next version (likely `[v2.1.3]`), section `### Fixed`:
  > **Vision doc naming flex (P004).** `agents/architect.md` and `agents/worker.md` now reference `docs/CHARACTER*.md` (glob) instead of literal `docs/CHARACTER.md` — projects with named characters (e.g. Tarot's `docs/CHARACTER_CHI_HA.md`) work without symlink workaround. Architect globs and reads every match; Worker MUST NOT Read any match (Glob/Grep for detection only). Companion edits in `docs/SETUP.md` (canonical-name recommendation), `docs/HANDOFF.md` (Handoff 0: 3 sites — vision-doc list, workflow ASCII block, session-open reading order), `docs/LAYERS.md` (access matrix + Layer 1 inner box + Chủ nhà responsibility #1), `docs/GENESIS.md` (Phase 0 Vision row). Sibling fix to P003; same principle: sos-kit consumes Sếp-owned docs, doesn't dictate names.
- [ ] **Docs Gate grep coverage (V2 add, V3 strengthened)** — pre-commit hook runs `grep -rn "docs/CHARACTER\.md" docs/ agents/ .claude/agents/` and verifies the only hits are: (a) SETUP.md `cp` default target, (b) the new note text inside SETUP comment block (which mentions both `CHARACTER.md` and `CHARACTER_<NAME>.md`), (c) anchor/example mentions inside this phiếu file itself (out of scope — phiếu IS the spec). Any other hit = doc drift = fail gate. **V3:** Hook also runs `grep -n "CHARACTER" docs/HANDOFF.md` and confirms zero bare `CHARACTER.md` literal hits inside Handoff 0 body (lines 17–37) — intra-section consistency enforced.
- [ ] No need to update `CLAUDE.md` (this repo's contributor doc) — its scope tree does not list every agent envelope rule.
- [ ] No need to update `phieu/README.md` or `phieu/TICKET_TEMPLATE.md` — phiếu format unchanged.

### Discovery Report

- [ ] Append entry to `docs/DISCOVERIES.md` (newest on top):
  - **Assumptions in phiếu — CORRECT:** list each anchor that resolved ✅ at V3 (especially #5, #10, #11 from Worker grep + #13, #14, #15 added at V2 + #16 added at V3).
  - **Assumptions in phiếu — WRONG:** anchor #2 line number was 19 in V1 → corrected to 17 in V2 per Turn 1 [O1.3]. Anchor #6 V1 expectation was "5-file scope" → expanded to "6-file scope" in V2 per Turn 1 [O1.1]. Anchor #8 V2 expectation was "lines 21, 33" → expanded to "lines 21, 28, 33" in V3 per Turn 2 [O2.1] (intra-section scope leak).
  - **Edge cases / limitations found:**
    - Did `scripts/sync-personal-agents.sh` actually handle `worker.md`? (anchor #5)
    - Were there any `docs/CHARACTER.md` rule-bindings outside the **6-file** scope? (anchor #6 — V2 widened expectation)
    - Did Glob on macOS vs Linux behave the same? (filesystem case-sensitivity edge case — `CHARACTER*` vs `character*`)
    - **V2 add:** did LAYERS.md ASCII box stay aligned after the 1-char addition on line 37?
    - **V2 add:** did GENESIS.md Phase 0 row still render as a valid 4-column markdown table?
    - **V3 add:** did HANDOFF.md line 28 ASCII workflow block (lines 25–31) preserve indent + arrow alignment after the 1-char `*` insertion?
    - **V3 add:** did the post-V3 Handoff 0 read consistently top-to-bottom (no mid-section convention shift between lines 21, 28, 33)?
  - **Docs updated to match reality:** at minimum agents/, SETUP.md, HANDOFF.md (3 sites), LAYERS.md (3 sites), GENESIS.md (per Tasks 1, 2, 4, 5, 6, 7) + CHANGELOG.md. If Worker discovered additional sites in anchor #6 → list them too.
  - **Debate Log meta:** record that Turn 1 [O1.1] + [O1.2] both ACCEPTED → V2 expanded scope by 3 micro-edits in 2 new files; anchor #2 line corrected. Turn 2 [O2.1] ACCEPTED → V3 expanded Task 5 to 3 sub-edits within an already-touched file (HANDOFF.md), zero new files in scope. Turn count: 2/3 used. 1h estimate held across all 3 versions.
  - Write "None" explicitly for any sub-section where truly nothing applies — explicit None proves the check happened.
