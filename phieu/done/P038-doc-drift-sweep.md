# PHIẾU P038: Doc drift + symmetry sweep

> **Loại:** Chore (doc consistency, mechanical edits)
> **Ưu tiên:** P2
> **Tầng:** 2 (lặt vặt — 8-9 anchor files, all surgical edits, no schema/API/auth/new-dep change, no code logic touch)
> **Ảnh hưởng:** `CLAUDE.md`, `README.md`, `docs/PHILOSOPHY.md`, `docs/LAYERS.md`, `docs/HANDOFF.md`, `hooks/pre-commit`, `skills/init/SKILL.md`, `skills/retro/SKILL.md`, `recipes/ai/multi-model-fallback.md`, `recipes/payment/payos-vn.md`
> **Dependency:** None (independent of P005, P006)

---

## Context

### Vấn đề hiện tại

Hai review độc lập ngày 2026-05-05 surface 10 doc-consistency findings across the kit. Cluster đã verified (Sếp grep thực tế trước khi promote vào Active sprint) → không cần Architect re-verify, không cần CHALLENGE round-trip. Mỗi finding self-contained, không cross-cut, ship-as-batch.

Findings (recap):
1. `CLAUDE.md` "Repo structure" tree stale (lists 9 skills, reality is 13; missing `agents/`, `bin/`, `recipes/`, `scripts/`, `templates/` + `CHANGELOG.md`, `INSTALL.md` + `docs/BACKLOG.md`, `docs/COMPARISON.md`, `docs/DISCOVERIES.md`, `docs/GENESIS.md`, `docs/ORCHESTRATION.md`, `docs/ticket/` + `phieu/AUDIT_PROTOCOL.md`, `phieu/GENESIS_TEMPLATE.md`, `phieu/LAUNCH_CHECKLIST.md`).
2. 5 hardcoded personal paths violate `CLAUDE.md` Rule 3 (`/Users/nguyenhuuanh/...` × 4 in recipes, `/c/Users/Admin/...` × 1 in `skills/retro/SKILL.md`).
3. `README.md` architecture tree (line ~393-443) missing `bin/`, `recipes/` despite same README referencing them elsewhere (line 68, 133, 140, 181, 190). (Note: `bootstrap/` listed in Sếp's findings — Worker verifies whether folder exists; Architect glob found no `bootstrap/` folder so likely Sếp's note error.)
4. `docs/PHILOSOPHY.md` count mismatch — line 28 declares "Principle 0", line 50 header says "Six Principles", `CLAUDE.md` line 113 says "5 principles". Three different numbers for same thing.
5. `docs/PHILOSOPHY.md` skills list (~line 71) missing `idea`, `init`, `forge`, `apply` (4 skills added since list last refreshed).
6. `docs/LAYERS.md` line 49 + 62 list `/verify` in BOTH Architect and Worker columns — line 179 acknowledges cross-layer exception but visual scan still confusing.
7. `docs/HANDOFF.md` line 247 cell `/decide (on Worker side)` ambiguous — `/decide` is Chủ-nhà skill, Worker uses it to *frame* multi-choice for Sếp.
8. `hooks/pre-commit` line 102 strict regex `^## .*Active sprint` lacks fallback that `scripts/session-start-banner.sh:22-27` already has (P003 pattern). BACKLOG using `## Now` heading would banner-work but hook-fail.
9. `skills/init/SKILL.md` line 17, 81, 190 reference `/blueprint` slashed (looks like skill) — reality is `sos blueprint` CLI subcommand (line 182 already correct). No `skills/blueprint/` folder exists.
10. `CLAUDE.md` "What this is NOT" rule 1 ("Not a project scaffolder. It doesn't generate your app") drifts from reality — `recipes/` library is acceptable scaffolding form, P032/P033 (Next sprint) explicitly build on `recipes/`.

### Giải pháp

10 surgical edits across 10 files, each self-contained. Tầng 2 → skip CHALLENGE per ORCHESTRATION.md Hard rule #7 (P036). Skip APPROVAL_GATE per `feedback_skip_approval_gate.md` (sub-1h surgical drift fix). DRAFT → EXECUTE thẳng.

Worker reads each task, applies edit, validates with `grep` count assertion at end of each task. Total ~50 phút.

### Scope
- CHỈ sửa (10 files): `CLAUDE.md`, `README.md`, `docs/PHILOSOPHY.md`, `docs/LAYERS.md`, `docs/HANDOFF.md`, `hooks/pre-commit`, `skills/init/SKILL.md`, `skills/retro/SKILL.md`, `recipes/ai/multi-model-fallback.md`, `recipes/payment/payos-vn.md`
- KHÔNG sửa: `docs/ticket/P004-vision-doc-naming-flex.md` (historical record — Worker self-decides, em recommend skip; logging decision to Discovery either way), code logic anywhere, `agents/*.md`, `phieu/TICKET_TEMPLATE.md`, any `skills/*/SKILL.md` other than `init` + `retro`.

---

## Task 0 — Verification Anchors

> **Em note:** Sếp đã grep tất cả 10 finding trước khi promote vào Active sprint 2026-05-05. Architect không tự grep được (no Bash, no Grep tool). Em mark `[verified — Sếp grep 2026-05-05]` cho mỗi anchor; Worker spot-check khi EXECUTE, escalate nếu phát hiện drift.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `CLAUDE.md` line 33-69 has stale ASCII tree (9 skills, missing folders) + line 20 says "9 total (3+1+5)" | `grep -n "9 total\|skills/" CLAUDE.md` + read line 33-69 | ✅ [verified — Sếp grep 2026-05-05] |
| 2 | Reality has 13 skills: apply, decide, forge, idea, init, insight, plan, qa, retro, review, route, ship, verify | `ls skills/` | ✅ [verified — Architect glob `skills/*/SKILL.md` confirmed 13 entries 2026-05-05] |
| 3 | Folders that exist at repo root: `agents/`, `bin/`, `configs/`, `docs/`, `hooks/`, `integrations/`, `phieu/`, `recipes/`, `scripts/`, `skills/`, `templates/` | `ls -d */` | ⚠️ [needs Worker verify] — Architect glob confirmed `agents/`, `bin/`, `recipes/`, `scripts/`, `templates/` exist; did NOT find `bootstrap/` (Sếp's findings note suggests `bootstrap/` exists but glob returned no files). Worker `ls -d */` to confirm full folder list before rewriting tree |
| 4 | `recipes/ai/multi-model-fallback.md` line 290-291 has `/Users/nguyenhuuanh/tarot/lib/ai/*` and `/Users/nguyenhuuanh/tarot/docs/DISCOVERIES.md` | `grep -n "/Users/nguyenhuuanh" recipes/ai/multi-model-fallback.md` | ✅ [verified — Sếp grep 2026-05-05] |
| 5 | `recipes/payment/payos-vn.md` line 273-274 has `/Users/nguyenhuuanh/tarot/app/api/payment/payos/*` and `/Users/nguyenhuuanh/tarot/docs/DISCOVERIES.md` | `grep -n "/Users/nguyenhuuanh" recipes/payment/payos-vn.md` | ✅ [verified — Sếp grep 2026-05-05] |
| 6 | `skills/retro/SKILL.md` line 156 has `/c/Users/Admin/$proj` (Windows path, orphan from another project) | `grep -n "/c/Users/Admin" skills/retro/SKILL.md` | ✅ [verified — Sếp grep 2026-05-05] |
| 7 | `README.md` architecture tree at line 393-443 missing `bin/`, `recipes/` (and possibly `bootstrap/` per Sếp note) | `sed -n '393,443p' README.md` | ✅ [verified — Sếp grep 2026-05-05]. Worker confirms exact line range when EXECUTE — README may have shifted since 2026-05-05. |
| 8 | `docs/PHILOSOPHY.md` line 28 has `## Principle 0`, line 50 has `## Six Principles`, line 32 has phrase "Every other principle in this kit serves this one" | `grep -n "Principle 0\|Six Principles\|serves this one" docs/PHILOSOPHY.md` | ✅ [verified — Sếp grep 2026-05-05] |
| 9 | `CLAUDE.md` line 113 says "5 principles are load-bearing" | `grep -n "5 principles\|five principles" CLAUDE.md` | ✅ [verified — Sếp grep 2026-05-05] |
| 10 | `docs/PHILOSOPHY.md` skills list (~line 71) missing `idea`, `init`, `forge`, `apply` | `grep -n "skills:\|/idea\|/init\|/forge\|/apply" docs/PHILOSOPHY.md` | ✅ [verified — Sếp grep 2026-05-05]. Worker confirms exact line + current list before replacement. |
| 11 | `docs/LAYERS.md` line 49 + 62 list `/verify` in both Architect and Worker columns; line 179 has phrase "(or cross-layer gate as with /verify)" | `grep -n "/verify" docs/LAYERS.md` | ✅ [verified — Sếp grep 2026-05-05] |
| 12 | `docs/HANDOFF.md` line 247 contains the table cell `/decide (on Worker side)` | `grep -n "decide.*Worker side" docs/HANDOFF.md` | ✅ [verified — Sếp grep 2026-05-05] |
| 13 | `hooks/pre-commit` line 102 has strict regex `^## .*Active sprint` (no fallback) | `grep -n "Active sprint" hooks/pre-commit` | ✅ [verified — Sếp grep 2026-05-05] |
| 14 | `scripts/session-start-banner.sh` line 22-27 has fallback awk pattern (strict match first → fallback first `^## ` if absent) | `sed -n '22,27p' scripts/session-start-banner.sh` | ✅ [verified — Sếp grep 2026-05-05]. Worker reads exact awk pattern before porting. |
| 15 | `skills/init/SKILL.md` line 17, 81, 190 reference `/blueprint` (slashed); line 182 already uses `sos blueprint` (CLI form) | `grep -n "blueprint" skills/init/SKILL.md` | ✅ [verified — Sếp grep 2026-05-05] |
| 16 | No `skills/blueprint/` folder exists (so `/blueprint` slashed is misleading) | `ls skills/ \| grep blueprint` | ✅ [verified — Architect glob `skills/*/SKILL.md` 2026-05-05 returned no `blueprint` entry] |
| 17 | `CLAUDE.md` "What this is NOT" rule 1 says "Not a project scaffolder. It doesn't generate your app; it ships your app." | `grep -n "Not a project scaffolder" CLAUDE.md` | ✅ [verified — Architect Read CLAUDE.md 2026-05-05, line 26-30] |

**Summary:** 14 ✅ verified anchors + 2 ⚠️ Worker spot-check items + 1 anchor (Sếp's `bootstrap/` mention) flagged as likely note-error. No ❌. Worker proceeds with EXECUTE; if any anchor fails actual grep at EXECUTE-time, escalate via Discovery.

---

## Debate Log

> Tầng 2 phiếu — orchestrator skips CHALLENGE_PHASE per ORCHESTRATION.md Hard rule #7 (P036). Skip APPROVAL_GATE per `feedback_skip_approval_gate.md` (sub-1h surgical drift fix, no kiến trúc/API/schema/auth touch). DRAFT → EXECUTE thẳng.

**Phiếu version:** V1 (initial draft)

**Skip-CHALLENGE invoked:** YES — per P036 ORCHESTRATION Hard rule #7. Mechanical doc edits, no architecture decision.
**Skip-APPROVAL-GATE invoked:** YES — per `feedback_skip_approval_gate.md`. Sub-1h surgical, drift fix, Sếp đã APPROVE cluster trước khi promote vào BACKLOG.

### Final consensus
- Phiếu version: V1 (Tầng 2, no debate, no approval gate)
- Total turns: 0
- Approved by Chủ nhà: 2026-05-05 (cluster approve via BACKLOG promotion) — code execution may begin

---

## Nhiệm vụ

> Worker order = Task 1 → 10 (independent edits, can be applied in any order, but listed in finding order for traceability). Each task ends with a `grep` validation that Worker runs to confirm fix landed.

### Task 1: `CLAUDE.md` — Repo structure tree refresh + skill count

**File:** `CLAUDE.md`

**Tìm 1** (line ~20, in "What's inside" bullet list):
```
- `skills/` — Claude Code skills grouped by layer (3 Chủ nhà + 1 Kiến trúc sư + 5 Thợ = 9 total)
```

**Thay bằng 1:**
```
- `skills/` — Claude Code skills grouped by layer (13 total: 4 Chủ nhà + 2 Kiến trúc sư + 7 Thợ — see `docs/LAYERS.md` skills map for canonical assignment)
```

**Lưu ý 1:** Số 4/2/7 = Architect best estimate from glob output (apply/decide/idea/insight/route → Chủ nhà-leaning; forge/plan → Architect; verify/review/qa/ship/retro/init → Worker; mapping varies — Worker reads `docs/LAYERS.md` skills map and uses ACTUAL canonical numbers if different. If LAYERS.md table differs, source-of-truth is LAYERS.md and Worker syncs CLAUDE.md to it. Log to Discovery.

**Tìm 2** (line 33-69, full ASCII tree starting `sos-kit/`):
Replace the entire fenced block (line 33-69 inclusive) with a refreshed tree. Worker constructs the new tree by:
1. `ls -1 *` at repo root → enumerate top-level files + folders
2. `ls -1 docs/ phieu/ skills/ recipes/ scripts/ templates/ integrations/ hooks/ configs/ agents/ bin/` → enumerate each subfolder's notable contents
3. Render in same ASCII style as current tree (unicode box chars `├──` `│   ` `└──`, brief comment after `#`)
4. Include AT MINIMUM these entries that current tree lacks:
   - Top level: `CHANGELOG.md`, `INSTALL.md`, `agents/`, `bin/`, `recipes/`, `scripts/`, `templates/`
   - `docs/`: `BACKLOG.md`, `COMPARISON.md`, `DISCOVERIES.md`, `GENESIS.md`, `ORCHESTRATION.md`, `ticket/`
   - `phieu/`: `AUDIT_PROTOCOL.md`, `GENESIS_TEMPLATE.md`, `LAUNCH_CHECKLIST.md`
   - `skills/`: all 13 (apply, decide, forge, idea, init, insight, plan, qa, retro, review, route, ship, verify) — current tree only lists 9
5. KHÔNG include `bootstrap/` if `ls` shows it doesn't exist (Sếp's findings mentioned it, Architect glob did not find it; Worker is final authority).

**Lưu ý 2:** Tree comment lines (after `#`) stay short. Match existing terse style. If a folder has many files (e.g. `phieu/VISION_TEMPLATES/` has 5+), use `...` ellipsis and the comment "templates copied + filled by Chủ nhà day 1" — don't enumerate every template file.

**Validate:**
- `grep -c "9 total" CLAUDE.md` → must return `0` (old phrase gone).
- `grep -c "13 total\|13 skills" CLAUDE.md` → must return `≥1`.
- `grep -c "agents/\|bin/\|recipes/\|scripts/\|templates/" CLAUDE.md` → must return `≥5` (each folder appears in tree at least once).
- `grep -c "CHANGELOG.md\|INSTALL.md\|BACKLOG.md\|ORCHESTRATION.md" CLAUDE.md` → ≥4.

---

### Task 2: Hardcoded personal paths sweep

**Files & edits:**

**File 2a:** `recipes/ai/multi-model-fallback.md`

**Tìm** (line 290-291 area):
```
/Users/nguyenhuuanh/tarot/lib/ai/*
/Users/nguyenhuuanh/tarot/docs/DISCOVERIES.md
```

**Thay bằng:**
```
~/<your-app>/lib/ai/*
~/<your-app>/docs/DISCOVERIES.md
```

**Lưu ý:** Preserve surrounding markdown context (code fence, bullet, prose) exactly — only the path strings change. If line numbers shifted slightly since 2026-05-05, locate by the literal `/Users/nguyenhuuanh/tarot/` substring, not by line number.

---

**File 2b:** `recipes/payment/payos-vn.md`

**Tìm** (line 273-274 area):
```
/Users/nguyenhuuanh/tarot/app/api/payment/payos/*
/Users/nguyenhuuanh/tarot/docs/DISCOVERIES.md
```

**Thay bằng:**
```
~/<your-app>/app/api/payment/payos/*
~/<your-app>/docs/DISCOVERIES.md
```

---

**File 2c:** `skills/retro/SKILL.md`

**Tìm** (line 156 area):
```
/c/Users/Admin/$proj
```

**Thay bằng:**
```
~/$proj
```

**Lưu ý:** Windows-style path orphan (likely copied from another project's example). The `~/` form works on macOS/Linux/Windows-WSL alike. If the surrounding context references Git Bash specifically, Worker may instead use `~/proj` without the `$` — judgment call, log to Discovery.

---

**Validate (after all 3 sub-edits):**
- `grep -rn "/Users/nguyenhuuanh" recipes/ skills/ | wc -l` → must return `0` (all 5 instances gone from these dirs).
- `grep -rn "/c/Users/Admin" skills/ | wc -l` → must return `0`.
- `grep -rn "/Users/nguyenhuuanh" docs/ticket/` → may return historical hits in `P004-vision-doc-naming-flex.md` line 189; Worker self-decides whether to fix (em recommend SKIP — historical phiếu record, not active reference). Log decision to Discovery.

---

### Task 3: `README.md` architecture tree — add missing folders

**File:** `README.md`

**Tìm** (line ~393-443, the architecture ASCII tree block — locate by the opening line that starts with `sos-kit/` inside a fenced code block):
The current tree under "Architecture" section. Worker reads the actual block (lines may have shifted) and identifies entries that should appear but don't.

**Thay bằng / Thêm:**
Inject 2 entries (or 3 if `bootstrap/` exists per Anchor #3 ⚠️ Worker verify):
- `bin/` — with comment like `# helper scripts (sos.sh entrypoint)`
- `recipes/` — with comment like `# DNA snippets — patterns /apply consumes (AI fallback, payment, etc.)`
- (conditional) `bootstrap/` — only if `ls bootstrap/` shows files exist

Insertion order: alphabetical within the tree (so `bin/` after `agents/`, `recipes/` after `phieu/`).

**Lưu ý:** Match the existing tree's comment style (terse `# one-line description`). Don't reformat unrelated lines. The other README sections (line 68, 133, 140, 181, 190) that already reference `bin/`/`recipes/` stay untouched — they're correct, only the tree block lacks them.

**Validate:**
- After edit, `grep -c "^├── bin/\|^│   ├── bin/\|│ bin/" README.md` (or whichever ASCII style README uses) → ≥1.
- `grep -c "recipes/" README.md` → unchanged or +1 (existing references stay; tree gets one more).

---

### Task 4: `docs/PHILOSOPHY.md` — principles count phrasing + sync `CLAUDE.md`

**File 4a:** `docs/PHILOSOPHY.md`

**Tìm** (line ~50):
```
## Six Principles
```

**Thay bằng:**
```
## Six Operational Principles
```

**Lưu ý:** Worker chooses one of two phrasings:
- **Option A (em recommend):** "Six Operational Principles" — frames Principle 0 as the meta-principle (accountability) and the 6 numbered ones as operational rules. Minimal diff, no renumbering.
- **Option B:** "Seven Principles" — counts Principle 0 as #1; requires renumbering all 6 below. Higher diff, breaks any external link to "Principle N".

If Worker picks Option B, also renumber `## Principle 1` through `## Principle 6` headers and any "Principle N" references in body text. Em strongly suggest A unless Worker finds existing cross-references that already treat 0 as a regular numbered principle.

**File 4b:** `CLAUDE.md`

**Tìm** (line ~113):
```
The 5 principles are load-bearing.
```

**Thay bằng (matching Option A — recommended):**
```
The 6 operational principles (plus Principle 0 = accountability) are load-bearing.
```

**Thay bằng (alternative, if Worker picks Option B in 4a):**
```
The 7 principles are load-bearing.
```

**Validate:**
- `grep -c "Six Principles" docs/PHILOSOPHY.md` → 0 (old header gone).
- `grep -c "5 principles\|five principles" CLAUDE.md` → 0.
- `grep -c "6 operational\|Six Operational\|7 principles\|Seven Principles" docs/PHILOSOPHY.md CLAUDE.md` → ≥2 (one in each file, consistent phrasing).

---

### Task 5: `docs/PHILOSOPHY.md` — refresh skills list

**File:** `docs/PHILOSOPHY.md`

**Tìm** (line ~71 area, the skills-per-layer list. Worker reads ~line 65-85 to find the actual list — could be in prose, table, or bullet form):
The current list of skills, which is missing `idea`, `init`, `forge`, `apply`.

**Thay bằng:**
Sync the list to match `docs/LAYERS.md` skills map AND `README.md` "Claude Code Skills" table. All 13 skills present, grouped by layer:
- Chủ nhà: `idea`, `route`, `decide`, `insight` (verify against LAYERS.md — if LAYERS assigns differently, LAYERS wins)
- Kiến trúc sư: `plan`, `forge`
- Thợ: `verify`, `apply`, `review`, `qa`, `ship`, `retro`, `init` (verify against LAYERS — `init` may be cross-layer)

**Lưu ý:**
- LAYERS.md is the canonical source. If LAYERS disagrees with this Architect-best-guess grouping, Worker syncs PHILOSOPHY to LAYERS.
- Preserve the surrounding prose context (the list likely sits under a heading like "Skills per layer" or similar) — only the list contents change.
- If the list is in table form (`|` pipe-separated), preserve column structure.

**Validate:**
- `grep -c "/idea\|/init\|/forge\|/apply" docs/PHILOSOPHY.md` → ≥4 (each skill name appears at least once).
- `grep -c "/route\|/decide\|/insight\|/plan\|/verify\|/review\|/qa\|/ship\|/retro" docs/PHILOSOPHY.md` → ≥9.

---

### Task 6: `docs/LAYERS.md` — `/verify` cross-layer marker

**File:** `docs/LAYERS.md`

**Tìm** (line 49 — Architect column):
```
Skills: /plan /forge /verify
```

**Thay bằng:**
```
Skills: /plan /forge /verify *
```

**Tìm** (line 62 — Worker column):
```
Skills: /verify /apply /review /qa /ship /retro
```

**Thay bằng:**
```
Skills: /verify * /apply /review /qa /ship /retro
```

**Tìm** (line 179 — already has the cross-layer phrase):
```
One skill = one layer (or cross-layer gate as with /verify)
```

**Thay bằng:**
```
One skill = one layer (or cross-layer gate as with /verify *)

\* `/verify` is a cross-layer gate: Architect specifies what must be verified (Task 0 anchors); Worker runs the verification. Listed in both columns above for that reason.
```

**Lưu ý:** Footnote uses literal `*` character (escaped as `\*` in the body to render). Place the footnote line immediately after line 179's existing phrase, with one blank line before it. Don't disturb other content nearby.

**Validate:**
- `grep -c "/verify \*" docs/LAYERS.md` → ≥2 (one in Architect col, one in Worker col).
- `grep -c "cross-layer gate" docs/LAYERS.md` → ≥1.

---

### Task 7: `docs/HANDOFF.md` — clarify `/decide` on Worker side

**File:** `docs/HANDOFF.md`

**Tìm** (line ~247, in summary table):
```
Worker → Chủ nhà → Architect | Tầng 1 blocker | Multi-choice escalation | /decide (on Worker side)
```

**Thay bằng:**
```
Worker → Chủ nhà → Architect | Tầng 1 blocker | Multi-choice escalation | Worker frames choices → Chủ nhà invokes /decide
```

**Lưu ý:** Preserve table column structure (pipes, alignment). The change is purely the right-most cell text. If the actual table uses different cell delimiter or has more columns, Worker locates by the literal substring `/decide (on Worker side)` and replaces the cell content while keeping all other columns intact.

**Validate:**
- `grep -c "/decide (on Worker side)" docs/HANDOFF.md` → 0 (old phrasing gone).
- `grep -c "Worker frames choices\|Chủ nhà invokes /decide" docs/HANDOFF.md` → ≥1.

---

### Task 8: `hooks/pre-commit` — port Active sprint fallback from session-start-banner

**File:** `hooks/pre-commit`

**Tìm** (around line 102, the strict regex block. Worker reads `sed -n '95,115p' hooks/pre-commit` to see the surrounding shell logic):
The current strict-match implementation that locates the Active sprint heading via regex `^## .*Active sprint`.

**Thay bằng:**
Port the fallback pattern from `scripts/session-start-banner.sh` line 22-27. Worker reads the banner script's exact awk/grep pattern and adapts it to pre-commit's shell idiom. The semantic:
1. Try strict match: first `^## ` line whose content contains `Active sprint` (case-insensitive substring).
2. If strict match returns empty → fallback: first `^## ` line in the file (any heading).
3. Use the matched heading's line number as the start of the active section.

**Lưu ý:**
- Match the banner script's idiom exactly (same awk variables, same NR comparison) so the two stay in sync. If banner uses `awk '/^## .*[Aa]ctive sprint/{print NR; exit}'` with a follow-up fallback, pre-commit uses the identical construct.
- Preserve all surrounding pre-commit shell logic (variable names like `HEADER_LINE`, exit codes, error messages) — only the matching block changes.
- Test mentally: BACKLOG with `## Now` (no "Active sprint" text) → strict miss → fallback to `## Now` → hook proceeds. BACKLOG with `## 🔥 Active sprint: ...` (current case) → strict hit → unchanged behavior.
- If pre-commit currently produces a user-visible error message ("BACKLOG missing Active sprint heading") on miss, that error message goes away in the new path (since fallback always finds a heading) — that's the desired behavior, not a regression.

**Validate:**
- `grep -c "Active sprint" hooks/pre-commit` → ≥1 (still references the strict-match attempt).
- `diff <(awk '/^## /{print NR; exit}' docs/BACKLOG.md) <(awk '/^## /{print NR; exit}' docs/BACKLOG.md)` — sanity that fallback finds *something*.
- Run pre-commit hook on a synthetic BACKLOG renamed to `## Now` (in a scratch worktree, not actual repo) → must NOT fail. Run on current BACKLOG → must still succeed (regression).

---

### Task 9: `skills/init/SKILL.md` — `/blueprint` slashed → `sos blueprint` CLI

**File:** `skills/init/SKILL.md`

**Tìm 1** (line ~17):
The literal substring `/blueprint` (slashed form, looks like a skill invocation).

**Thay bằng 1:**
`sos blueprint` (CLI subcommand form, matches line 182's existing correct usage).

**Apply same find/replace to:** line ~81, line ~190.

**Lưu ý:**
- Worker greps `grep -n "/blueprint" skills/init/SKILL.md` to locate ALL slashed occurrences (could be more than 3 if drift accumulated). Replace each with `sos blueprint`.
- Line 17 specifically reads `Phase 1 ('/blueprint' — Kiến trúc sư)` — if Worker finds the prose flows better as `Phase 1 (Kiến trúc sư bootstrap, run \`sos blueprint\`)`, that rephrase is acceptable. Tầng 2 sentence-level wording is Worker's call.
- Line 182 already uses `sos blueprint` correctly — DO NOT touch.
- Other `/skill-name` references in the file (e.g. `/insight`, `/route`) are correct and stay slashed — only `/blueprint` is the false slash because there's no `skills/blueprint/` folder.

**Validate:**
- `grep -c "/blueprint" skills/init/SKILL.md` → 0 (all slashed forms gone).
- `grep -c "sos blueprint" skills/init/SKILL.md` → ≥4 (was ≥1 at line 182; +3 minimum from the replacements).

---

### Task 10: `CLAUDE.md` — "What this is NOT" rephrase rule 1

**File:** `CLAUDE.md`

**Tìm** (line 26-30 area, "What this repo is NOT" section, rule 1):
```
- **Not a project scaffolder.** It doesn't generate your app; it ships your app.
```

**Thay bằng:**
```
- **Not a boilerplate project scaffolder.** `recipes/` provides battle-tested **patterns** (DNA snippets, decision rationale) that `/apply` consumes — but the kit doesn't generate full app templates from a blank directory. SOS Kit picks up after "code is ready," not "project is empty."
```

**Lưu ý:**
- Preserve the bullet's `- **bold**` markdown style and surrounding bullet siblings.
- Worker may polish phrasing if a tighter version reads better — em propose this version is ~33 words and reads well; <40 words is the budget. Don't expand to a paragraph.
- The other "NOT" bullets (Rule 2 "Not a runtime binary source", Rule 3 "Not a planning methodology", Rule 4 "Not a place for experimental features") stay untouched.

**Validate:**
- `grep -c "Not a project scaffolder\." CLAUDE.md` → 0 (old phrasing gone).
- `grep -c "boilerplate project scaffolder\|recipes/.*patterns" CLAUDE.md` → ≥1.
- `grep -c "code is ready" CLAUDE.md` → ≥1 (semantic kept).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `CLAUDE.md` | Task 1: refresh tree + skill count (line ~20, ~33-69). Task 4b: sync principles count (line ~113). Task 10: rephrase "Not a project scaffolder" (line ~26-30) |
| `README.md` | Task 3: add `bin/`, `recipes/` (and optionally `bootstrap/`) to architecture tree (line ~393-443) |
| `docs/PHILOSOPHY.md` | Task 4a: rename "Six Principles" → "Six Operational Principles" (line ~50). Task 5: refresh skills list (line ~71) |
| `docs/LAYERS.md` | Task 6: add `*` cross-layer marker on `/verify` (line 49 + 62) + footnote at line 179 |
| `docs/HANDOFF.md` | Task 7: clarify `/decide (on Worker side)` cell (line ~247) |
| `hooks/pre-commit` | Task 8: port Active sprint fallback from session-start-banner.sh (around line 102) |
| `skills/init/SKILL.md` | Task 9: replace 3+ `/blueprint` slashed with `sos blueprint` (line ~17, 81, 190) |
| `skills/retro/SKILL.md` | Task 2c: `/c/Users/Admin/$proj` → `~/$proj` (line ~156) |
| `recipes/ai/multi-model-fallback.md` | Task 2a: 2 hardcoded paths → `~/<your-app>/...` (line ~290-291) |
| `recipes/payment/payos-vn.md` | Task 2b: 2 hardcoded paths → `~/<your-app>/...` (line ~273-274) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `docs/ticket/P004-vision-doc-naming-flex.md` | Worker self-decides if line 189 hardcoded path needs fix (em recommend SKIP — historical record). Either way, log decision to Discovery. |
| `agents/architect.md`, `agents/worker.md`, `agents/orchestrator.md`, `agents/README.md` | No edits. State machine + role contracts unchanged. |
| `phieu/TICKET_TEMPLATE.md` | No edits. Format unchanged. |
| `scripts/session-start-banner.sh` | Source of truth for fallback pattern (Task 8). READ to copy idiom, do NOT edit. |
| `skills/blueprint/` | Confirms folder does NOT exist (Anchor #16). If Worker finds it does exist → escalate, this changes Task 9 framing. |
| All other `skills/*/SKILL.md` (apply, decide, forge, idea, insight, plan, qa, review, route, ship, verify) | No edits. Only `init` and `retro` touched per Task 9 + 2c. |

---

## Luật chơi (Constraints)

1. **Tier locked at 2.** Mechanical doc edits only. NO logic in `hooks/pre-commit` beyond porting the existing fallback idiom from banner. NO new skills, NO architecture changes, NO API/schema/auth touch. If Worker discovers any task actually requires editing `agents/*.md` or `phieu/TICKET_TEMPLATE.md` (móng nhà), STOP and escalate Tầng 2 → Tầng 1.
2. **Doc-source-of-truth ordering for cross-doc syncs:**
   - Skill list reality: `ls skills/` (filesystem) > `docs/LAYERS.md` skills map > `README.md` skill table > `docs/PHILOSOPHY.md` skills list > `CLAUDE.md` repo structure tree.
   - Principles count: `docs/PHILOSOPHY.md` is canonical; `CLAUDE.md` syncs to it.
   - Architecture folder list: filesystem (`ls -d */`) is canonical; `CLAUDE.md` tree, `README.md` tree both sync to filesystem.
3. **Worker spot-check anchors before each task.** If line numbers in this phiếu drift from reality (file edits since 2026-05-05), Worker locates by literal substring, not by line number, and updates the line ref in Discovery report.
4. **No batched commits.** One commit per task (or one commit per logical group: Task 2a/b/c can be one commit "chore: replace hardcoded personal paths"). Use `git add -p` for surgical staging if needed. Branch = `phieu/P038-doc-drift-sweep`.
5. **Task 8 (`hooks/pre-commit`) is the only behavioral change.** All other tasks are pure prose/tree/path string replacements with zero runtime effect. Task 8 must pass the regression test (current BACKLOG `## 🔥 Active sprint:` heading still resolves to active section line number — same as before the fallback was added).
6. **Validate-grep at end of each task is mandatory** (the `Validate:` blocks). Worker runs them before moving to next task. If any validation fails, fix the task before proceeding.
7. **Skip-CHALLENGE + skip-APPROVAL_GATE.** No CHALLENGE round. No mid-execute approval prompt. Worker EXECUTE → Discovery report → done.

---

## Nghiệm thu

### Automated
- [ ] All 10 tasks' `Validate:` grep assertions pass (each task has its own).
- [ ] `git diff --stat phieu/P038-doc-drift-sweep` shows ≤10 files changed (matches "Files cần sửa" table). If `docs/ticket/P004-...` Worker decided to fix, +1 OK.
- [ ] No file outside the "Files cần sửa" list modified (run `git diff --name-only main...phieu/P038-doc-drift-sweep` and audit).
- [ ] `hooks/pre-commit` is executable (`test -x hooks/pre-commit`) — no permission drift.
- [ ] If repo has docs-gate / type-check available: pass cleanly. If not (sos-kit fresh repo without `.docs-gate.toml` per P006), skip — Tầng 2 doc-only doesn't gate on it.

### Manual Testing
- [ ] **Task 8 regression — current BACKLOG.** Run `hooks/pre-commit` (or invoke its Active-sprint-finder section directly) against current `docs/BACKLOG.md` (which has `## 🔥 Active sprint: ...`) → resolves to same line number as before. No behavior change for the current case.
- [ ] **Task 8 fallback — synthetic BACKLOG.** In a scratch dir, write a BACKLOG with first heading `## Now` (no "Active sprint" substring). Run the same finder logic → resolves to that `## Now` line, does NOT error. Confirms fallback works.
- [ ] **Task 1 + 5 + 3 visual scan.** Render the 3 ASCII trees / lists in a markdown previewer (or just read in editor). All 13 skills present, all real folders listed, no fake folders, alphabetical order within each section.
- [ ] **Task 9 — `skills/init/SKILL.md` reads naturally** after `/blueprint` → `sos blueprint` swap. No grammatical breakage from the literal replacement.

### Regression
- [ ] `scripts/session-start-banner.sh` unchanged (read-only source for Task 8). `git diff scripts/session-start-banner.sh` → empty.
- [ ] All `agents/*.md` unchanged. `git diff agents/` → empty.
- [ ] `phieu/TICKET_TEMPLATE.md` unchanged. `git diff phieu/TICKET_TEMPLATE.md` → empty.
- [ ] No new `/Users/nguyenhuuanh` or `/c/Users/Admin` introduced anywhere. `grep -rn "/Users/nguyenhuuanh\|/c/Users/Admin" . --include="*.md" --include="*.sh" --include="*.json" --include="*.toml"` → returns only historical hits in `docs/ticket/` (if Worker chose to skip P004) and zero elsewhere.

### Docs Gate
- [ ] `CHANGELOG.md` — new entry at top: "P038: doc drift + symmetry sweep — refreshed CLAUDE.md tree (13 skills, all folders), removed 5 hardcoded personal paths, synced PHILOSOPHY/LAYERS/HANDOFF cross-doc inconsistencies, ported pre-commit Active sprint fallback from session-start-banner. Tầng 2 surgical, no behavior change except pre-commit fallback."
- [ ] No new doc files created (Tầng 2 sweep only).

### Discovery Report

Append entry to `docs/DISCOVERIES.md` (newest on top, like CHANGELOG). Required fields:

- [ ] **Anchor reality check.** For each of the 17 anchors in Task 0 — was the assumption correct at EXECUTE time? Note any drift since 2026-05-05 (line numbers shifted, content already partially fixed by another phiếu, etc.).
- [ ] **Anchor #3 resolution.** Did `bootstrap/` folder exist at EXECUTE time? (Architect glob said no; Sếp's findings note suggested yes.) Final answer + how it affected Task 1 + Task 3 trees.
- [ ] **Task 4 phrasing choice.** Did Worker pick Option A ("Six Operational Principles") or Option B ("Seven Principles")? Why? Any external link to "Principle N" found that influenced the choice?
- [ ] **Task 5 skill grouping.** What does `docs/LAYERS.md` actually assign each of the 13 skills to (Chủ nhà / Kiến trúc sư / Thợ)? Did Architect's guess in Task 1 + Task 5 match? List any mismatches.
- [ ] **Task 8 idiom transfer.** Quote the exact awk pattern copied from `scripts/session-start-banner.sh:22-27`. Confirm it landed in `hooks/pre-commit` verbatim (or note any adaptation required for shell vs script context).
- [ ] **Task 9 occurrence count.** How many `/blueprint` slashed instances did `grep -n "/blueprint" skills/init/SKILL.md` actually return? (Architect predicted 3 at lines 17, 81, 190; reality may be more.)
- [ ] **`docs/ticket/P004-...` decision.** Did Worker fix line 189's hardcoded path or skip? Reasoning.
- [ ] **Skip-CHALLENGE + skip-APPROVAL_GATE dogfood.** This is the second Tầng 2 phiếu after P037. Did the absence of CHALLENGE + approval gate feel right for this scope? Did any task surface a hidden Tầng 1 issue that should've gone through CHALLENGE? Honest signal needed for the P036 retrospective accumulating evidence.
- [ ] **Total time + tokens.** Architect estimated ~50 phút. Actual? (For future Tầng 2 phiếu sizing calibration.)
