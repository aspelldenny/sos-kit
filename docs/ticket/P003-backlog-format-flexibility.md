# PHIẾU P003: BACKLOG format flexibility — fallback when "Active sprint" header is missing

> **ID:** P003 — first phiếu file under `docs/ticket/` for sos-kit-self. Convention follows `phieu/TICKET_TEMPLATE.md`.
> **Filename:** `docs/ticket/P003-backlog-format-flexibility.md`
> **Branch:** `fix/P003-backlog-format-flexibility`

---

> **Loại:** Bugfix (drift fix from Tarot dogfood)
> **Ưu tiên:** P1 — must close before shipping single-command installer (per Active sprint goal in BACKLOG.md)
> **Ảnh hưởng:** `scripts/session-start-banner.sh` + `agents/architect.md` (and its sed-mirror `.claude/agents/architect.md`) + `docs/ORCHESTRATION.md` (added in V2 — see Debate Log Turn 1)
> **Dependency:** None. Sibling phiếu P004 (vision doc naming flex) is independent and may ship in either order.

---

## Context

### Vấn đề hiện tại

Three upstream constraints currently force every new project to restructure its `docs/BACKLOG.md` to literally contain a `## ... Active sprint` header — otherwise the SessionStart banner stays silent, the Architect refuses to write any phiếu, and the orchestrator greets with a misleading "BACKLOG chưa có Active sprint" message:

1. **`scripts/session-start-banner.sh` line 22** hard-codes
   `grep -n "^## .*Active sprint" "$BACKLOG"`
   to locate the in-flight section. If the user's BACKLOG uses a different name for the top section (e.g. `## Now`, `## In flight`, `## Sprint 4 — auth fixes`), `HEADER_LINE` is empty → script `exit 0` silently. Sếp gets no banner, no Active sprint context, no orchestrator greeting.

2. **`agents/architect.md` Hard rule 0** (lines 118–121) declares
   > "Only write phiếu for items in `docs/BACKLOG.md` under section **'Active sprint'**. If Sếp's request matches an item in 'Next sprint', 'Open backlog', 'Park', or doesn't match any item — STOP."
   This wording requires the literal phrase "Active sprint." If the project's BACKLOG instead has `## Now` as its first/top section, Architect cannot recognize that section as the active gate, refuses to draft, and Sếp must rename the section to satisfy the regex.

3. **`docs/ORCHESTRATION.md` line 32** hard-codes the orchestrator greeting line for the "no Active sprint header" edge case:
   > 'If BACKLOG has no Active sprint → greet without list: "Em là Kiến trúc sư. BACKLOG chưa có Active sprint — anh có item gì cần viết phiếu không?"'
   After Tasks 1–4 land, a project with `## Now` (fallback path) WILL have an active section detected by the script, but the orchestrator's scripted greeting still says "BACKLOG chưa có Active sprint." Doc and runtime behavior diverge — Sếp is told the BACKLOG is empty when it is not. Worker raised this in Debate Log Turn 1 (O1.1); the phiếu's own Docs Gate (line 343 in V1) explicitly required escalation if any other doc surfaces "Active sprint."

Tarot dogfood (2026-04-26, see `docs/DISCOVERIES.md` v2.1-dogfood entry) worked around constraints 1+2 by **restructuring its BACKLOG.md** to literally include `## 🔥 Active sprint: ...`. That workaround should not be mandatory for any future install — BACKLOG is a Sếp-owned doc, sos-kit consumes it, not the other way around.

### Giải pháp

**Three coordinated changes, one phiếu (atomic — all three must land together so the banner script, the Architect agent, and the orchestrator doc all describe the same fallback semantics):**

**A. `scripts/session-start-banner.sh` — add fallback to "first `## ` section after document title."**
Logic order:
1. Try the existing strict regex `^## .*Active sprint` (current behavior, preserves backwards compat for sos-kit's own BACKLOG).
2. If empty, fall back to: **the first `^## ` section in the file** (the document's top-most H2). This is treated as "the user's de-facto active section."
3. If still empty (no `^## ` headings at all → BACKLOG is empty or malformed), exit 0 silently as before.

When the fallback path is taken, the banner output appends a one-line note so Sếp knows which header was used:
```
📌 Treating "<header text>" as Active sprint (no "Active sprint" header found).
```

**B. `agents/architect.md` Hard rule 0 — soften the wording to match the script's fallback.**
Replace the literal-string requirement with a "BACKLOG resolution" sub-rule:
1. **Active section resolution:** Architect treats as the active section either (a) the first `## ` section whose heading matches `Active sprint` (case-insensitive substring), or (b) if no such heading exists, the first `## ` section in the file. The active section is the gate.
2. Items in any other section (typically labeled "Next sprint", "Open backlog", "Park", or any other H2 below the active section) require explicit promotion before phiếu drafting.
3. The four section labels currently named in Hard rule 0 ("Next sprint", "Open backlog", "Park", "doesn't match") are kept as **examples**, not as a closed list — the rule's intent is "non-active sections require promotion," not "these four exact labels."

**C. `docs/ORCHESTRATION.md` line 32 — rewrite the "no Active sprint" edge-case greeting to cover the fallback case.**
The "no Active sprint header" branch fires only when the script has NO `## ` headings at all (truly empty/malformed BACKLOG — script exits silently in that case anyway, so the orchestrator is greeting Sếp without injected SessionStart context). The wording must reflect that, not falsely claim the user's BACKLOG has no active sprint when fallback resolved one. New wording: greet acknowledging the BACKLOG has no recognized sections, prompt Sếp to add an item — no false "Active sprint" claim. (Exact replacement string in Task 6.)

All three changes are **format-tolerant, never silent** — the banner reports which header it used; Architect's rule body explains what fallback occurred; the orchestrator greeting no longer lies about BACKLOG state.

### Scope

- CHỈ sửa: `scripts/session-start-banner.sh`, `agents/architect.md`, `.claude/agents/architect.md` (regenerate via `scripts/sync-personal-agents.sh`), `docs/ORCHESTRATION.md` (line 32 edge-case greeting only — added in V2 per Worker O1.1).
- KHÔNG sửa: `docs/BACKLOG.md` itself (sos-kit's own BACKLOG already passes both old and new logic — it has `## 🔥 Active sprint: ...` which matches strict regex). Do not edit `agents/worker.md` (Worker doesn't enforce Rule 0). Do not touch `phieu/TICKET_TEMPLATE.md`. Do not touch other sections of `docs/ORCHESTRATION.md` — only the line 32 edge-case bullet, scope minimal.

---

## Task 0 — Verification Anchors

> Architect read these directly via `Read` tool. Worker should re-grep to confirm before EXECUTE — line numbers may shift if other phiếu land first.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `scripts/session-start-banner.sh` line 22 contains `HEADER_LINE=$(grep -n "^## .*Active sprint" "$BACKLOG" 2>/dev/null \| head -1 \| cut -d: -f1)` | `grep -n "Active sprint" scripts/session-start-banner.sh` | ✅ Line 22 — confirmed via direct Read of the file |
| 2 | `scripts/session-start-banner.sh` uses `set -uo pipefail` (no `-e`) and `awk -v start="$HEADER_LINE" 'NR > start && /^## / {print NR; exit}'` to find the next section boundary | Read full file | ✅ Lines 13–14 (set) and line 28 (awk) — confirmed |
| 3 | `agents/architect.md` Hard rule 0 lives at lines 118–121 with literal text `"Active sprint"` quoted | `grep -n "Active sprint" agents/architect.md` | ✅ Lines 118 and 120 — confirmed via direct Read |
| 4 | `.claude/agents/architect.md` is sed-generated from `agents/architect.md` via `scripts/sync-personal-agents.sh` (per CHANGELOG v2.1 entry: "`.claude/agents/architect.md` and `.claude/agents/worker.md` are now sed-generated from `agents/*.md`") | `cat scripts/sync-personal-agents.sh` (Worker) | ✅ Worker Turn 1 confirmed: script does `sed 's/Chủ nhà/Sếp/g' agents/$f > .claude/agents/$f` for both architect.md and worker.md. |
| 5 | `docs/BACKLOG.md` line 10 has `## 🔥 Active sprint: Drift fixes after Tarot dogfood` — strict regex still matches after our change (backwards compat) | `grep -n "^## .*Active sprint" docs/BACKLOG.md` | ✅ Line 10 — confirmed via direct Read |
| 6 | `docs/DISCOVERIES.md` v2.1-dogfood entry mentions Tarot's BACKLOG restructure as workaround | Read DISCOVERIES.md | ⚠️ Not explicitly worded as "BACKLOG restructured" in DISCOVERIES — the workaround is documented in `docs/BACKLOG.md` line 16 (P003 item itself) and in `MEMORY.md` `project_v21_tarot_dogfood_gaps.md`. Worker can cite BACKLOG line 16 as the source of truth for the workaround claim. |
| 7 | The shell script does NOT currently emit any "fallback used" message — adding one is a new line, not a modification | Read script lines 45–60 (banner output block) | ✅ Confirmed — current banner has no fallback-detection branch. New conditional output goes between `SPRINT_BLOCK=$(...)` extraction and the `echo` block. |
| 8 | No other shell script in `scripts/` greps `Active sprint` (so this fix is single-site for the script side) | `grep -rn "Active sprint" scripts/` (Worker) | ✅ Worker Turn 1 confirmed: only `session-start-banner.sh` contains it. |
| 9 | No file under `agents/` other than `architect.md` references "Active sprint" gating language | `grep -rn "Active sprint" agents/` (Worker) | ✅ Worker Turn 1 confirmed: only `agents/architect.md` contains it; `agents/worker.md` has zero mentions. |
| 10 | `docs/ORCHESTRATION.md` line 32 contains the literal greeting string `"BACKLOG chưa có Active sprint — anh có item gì cần viết phiếu không?"` inside a bullet under "Edge cases:" (line 30) | `grep -n "BACKLOG chưa có Active sprint" docs/ORCHESTRATION.md` | ✅ Architect re-read in V2 (Read of `docs/ORCHESTRATION.md` lines 25–40): line 32 contains the exact string verbatim, single occurrence in the doc. Worker re-greps before edit to confirm line number unchanged. |

**⚠️ count:** 1 (anchor #6 — wording in DISCOVERIES is indirect)
**⏳ count:** 0 (all V1 ⏳ resolved by Worker Turn 1; V2 anchor #10 added and pre-verified by Architect Read)
**❌ count:** 0

---

## Debate Log

> Auto-populated by Worker (CHALLENGE mode) and Architect (RESPOND mode). Sếp reads at approval gate — no mid-debate intervention unless orchestrator escalates.
> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Phiếu version bumps V1 → V2 → ... per Architect refine. Cap = 3 turns.

**Phiếu version:** V2 (Architect refined after Turn 1 — Task 6 added, ORCHESTRATION.md scoped in)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**
- #1 ✅ `session-start-banner.sh` line 22 — exact grep regex confirmed.
- #2 ✅ `set -uo pipefail` lines 13-14, `awk` line 28 — confirmed (Architect already read directly).
- #3 ✅ `agents/architect.md` lines 118-121 "Active sprint" confirmed — note: the canonical file says "Chủ nhà" (not "Sếp") because agents/ is English-neutral voice; the phiếu quoted "Sếp" but the functional content is identical. Tầng 2 discrepancy, does not affect task.
- #4 ✅ `scripts/sync-personal-agents.sh` EXISTS and confirmed behavior: `sed 's/Chủ nhà/Sếp/g' agents/$f > .claude/agents/$f` for both architect.md and worker.md. Task 4 is valid.
- #5 ✅ `docs/BACKLOG.md` line 10 has the strict-match header — confirmed (Architect already read).
- #6 ⚠️ `docs/BACKLOG.md` line 16 DOES contain the workaround mention: "Tarot worked around by restructuring its BACKLOG; that workaround should not be mandatory." This is the source of truth the phiếu cites. DISCOVERIES.md does not have a standalone "BACKLOG restructured" entry, but the BACKLOG line 16 + MEMORY.md file together cover it. Acceptable as cited — no action needed.
- #7 ✅ No fallback-detection branch in current banner output block — confirmed (Architect already read).
- #8 ✅ `grep -rn "Active sprint" scripts/` — only `session-start-banner.sh` contains it. Fix IS single-site for the script side.
- #9 ✅ `grep -rn "Active sprint" agents/` — only `agents/architect.md` contains it. `agents/worker.md` has zero mentions. No scope expansion needed for the agents/ side.

**Objections (Tầng 1 only):**

- [O1.1] `docs/ORCHESTRATION.md` line 32 contains literal gate language: `"If BACKLOG has no Active sprint → greet without list: 'Em là Kiến trúc sư. BACKLOG chưa có Active sprint — anh có item gì cần viết phiếu không?'"` — this string is hard-coded in the orchestration doc and would be incorrect after P003 lands: a project with `## Now` instead of `## Active sprint` DOES have an active section (fallback path), but the orchestrator would still greet "BACKLOG chưa có Active sprint." The doc and the script would diverge semantically. `docs/ORCHESTRATION.md:32` — code reference.

  The phiếu scope section (line 57-58) says "Do not touch ... `docs/HANDOFF.md`" etc., and the Nghiệm thu Docs Gate (line 333) says "Worker greps docs/ for 'Active sprint' to confirm. If any other doc surfaces, add to docs gate or escalate via CHALLENGE." This is exactly that case. The phiếu explicitly instructed escalation if any doc surfaces.

  **Proposed alternatives:**
  - A. Expand scope of P003 to also patch `docs/ORCHESTRATION.md` line 32 — replace the hard-coded "chưa có Active sprint" message with wording that covers the fallback case, e.g. "BACKLOG chưa có section nào — anh có item gì...". This keeps script + orchestration doc in sync. Worker recommends A — the fix is 1 line in ORCHESTRATION.md, low-risk, and staying silent about the doc drift would leave a misleading Docs Gate result. (Worker recommendation)
  - B. Explicitly exclude `docs/ORCHESTRATION.md` from P003 scope — add it to the "Files KHÔNG sửa" table with a note "ORCHESTRATION.md line 32 acknowledged, P005 tracks doc update." This keeps P003 minimal but creates a known doc drift until P005 ships.

**Status:** ✅ RESOLVED IN TURN 1 — see Architect Response below.

### Turn 1 — Architect Response (phiếu V2)

- [O1.1] → **ACCEPT alternative A** → expanded P003 scope to include `docs/ORCHESTRATION.md` line 32.

  **Reasoning:**
  1. Worker is correct on substance. After Tasks 1–4 land, the script + Architect agent both treat `## Now`-style BACKLOGs as having an active section via fallback. The orchestrator greeting at ORCHESTRATION.md:32 then either (a) never fires (because SessionStart did inject a sprint block — fallback path resolves a header), or (b) fires only in the truly-empty case (no `## ` headings at all). Either way, the current wording "BACKLOG chưa có Active sprint" is wrong: in case (a) it would lie about state, and in case (b) "no Active sprint" is technically true but misleading because the more accurate reason is "BACKLOG chưa có section nào / chưa có item nào" — the user's mental model is "I haven't set up a sprint yet," not "I'm missing a literal label."
  2. Phiếu V1's own Docs Gate (Nghiệm thu line referencing "If any other doc surfaces, add to docs gate or escalate via CHALLENGE") explicitly authorized this expansion. Worker followed protocol — Architect accepts the protocol's outcome.
  3. Cost vs benefit: 1-line edit to ORCHESTRATION.md vs deferring to P005 (not yet on the active sprint). Atomic landing keeps script + agent + orchestrator doc semantically aligned in a single commit; deferring creates a known-misleading greeting that Sếp would hit on the very next fresh project (Tarot v2.2 onward). The "format-tolerant, never silent" principle in the Giải pháp section already commits to consistency across all three surfaces — leaving ORCHESTRATION.md drift would violate that principle from day 1.
  4. Alternative B (defer to P005) was considered and rejected: P005 is not yet listed in BACKLOG.md "Active sprint" so it cannot be drafted under Hard rule 0. Carrying a doc drift across an unbounded delay is worse than a 1-line in-scope fix.

  **Action taken in V2:**
  - Added Task 6 (patch `docs/ORCHESTRATION.md` line 32) to "Nhiệm vụ".
  - Added anchor #10 in Task 0 covering the line 32 string + line number, pre-verified by Architect via direct Read of `docs/ORCHESTRATION.md` lines 25–40.
  - Added `docs/ORCHESTRATION.md` to "Files cần sửa" table with Task 6 reference.
  - Updated "Ảnh hưởng" header line, "Vấn đề hiện tại" (now lists 3 constraints, not 2), "Giải pháp" (now A/B/C, not A/B), and "Scope" (CHỈ sửa list).
  - Added a Manual Testing case for the rewrite + a Docs Gate check that ORCHESTRATION.md no longer contains the old string.
  - Bumped phiếu version V1 → V2.

**Status:** ✅ RESPONDED — phiếu bumped to V2. Awaiting Worker re-CHALLENGE on Turn 2 to confirm V2 has no new anchor mismatches before APPROVAL_GATE.

*(Repeat Turn 2, Turn 3 if needed. Cap = 3.)*

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Sếp: [date] — code execution may begin

---

## Nhiệm vụ

### Task 1: Add fallback header resolution to `session-start-banner.sh`

**File:** `scripts/session-start-banner.sh`

**Tìm** (current line 22, the strict-regex header lookup):

```bash
# Find the Active sprint header line number
HEADER_LINE=$(grep -n "^## .*Active sprint" "$BACKLOG" 2>/dev/null | head -1 | cut -d: -f1)

# Silent if no Active sprint section
[ -z "$HEADER_LINE" ] && exit 0
```

**Thay bằng:**

```bash
# Find the Active sprint header line number — strict match first
HEADER_LINE=$(grep -n "^## .*Active sprint" "$BACKLOG" 2>/dev/null | head -1 | cut -d: -f1)
FALLBACK_USED=0

# Fallback: first ^## section in the file (treats the top H2 as the active section)
if [ -z "$HEADER_LINE" ]; then
    HEADER_LINE=$(grep -n "^## " "$BACKLOG" 2>/dev/null | head -1 | cut -d: -f1)
    FALLBACK_USED=1
fi

# Silent if no ^## headings at all (malformed / empty BACKLOG)
[ -z "$HEADER_LINE" ] && exit 0

# Capture the actual header text for the fallback note (strip leading "## " + emoji whitespace)
HEADER_TEXT=$(sed -n "${HEADER_LINE}p" "$BACKLOG" | sed 's/^## *//')
```

**Lưu ý:**
- `set -uo pipefail` is in effect — `grep` exit 1 (no match) is normal here because we then check `[ -z "$HEADER_LINE" ]`. Do not add `set -e`.
- The `sed 's/^## *//'` is intentionally permissive — keeps emoji and extra text intact; only strips the markdown header marker. Tầng 2: Worker may use a different stripper (`awk '{$1=""; print}'`, etc.) if cleaner — Discovery report.
- The two `grep -n` calls are deliberate (not one merged regex) to preserve original strict-match precedence — sos-kit's own BACKLOG keeps its current banner behavior verbatim.

### Task 2: Emit a fallback-detection note in the banner output

**File:** `scripts/session-start-banner.sh`

**Tìm** (current lines 53–59, the closing banner block):

```bash
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Active sprint: $OPEN_COUNT items đang treo, $DONE_COUNT đã xong"
echo ""
echo "📌 Architect Rule 0: chỉ viết phiếu cho item trong Active sprint."
echo "    Idea mới → /idea skill (intake vào BACKLOG.md)."
echo "    Pick item hay add idea?"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
```

**Thay bằng:**

```bash
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Active sprint: $OPEN_COUNT items đang treo, $DONE_COUNT đã xong"
if [ "$FALLBACK_USED" = "1" ]; then
    echo ""
    echo "📌 Treating \"$HEADER_TEXT\" as Active sprint (no \"Active sprint\" header found)."
fi
echo ""
echo "📌 Architect Rule 0: chỉ viết phiếu cho item trong Active sprint (or first ^## section if absent)."
echo "    Idea mới → /idea skill (intake vào BACKLOG.md)."
echo "    Pick item hay add idea?"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
```

**Lưu ý:**
- The new conditional only prints when fallback was used — sos-kit's own BACKLOG won't show the note (preserves current UX).
- The "(or first ^## section if absent)" parenthetical in Rule 0 line is to keep the model's context aligned with the new behavior. Wording is dev-facing context only — Tầng 2: Worker may polish exact phrasing — Discovery report.

### Task 3: Soften `agents/architect.md` Hard rule 0 wording

**File:** `agents/architect.md`

**Tìm** (current lines 118–121, the Hard rule 0 block):

```markdown
0. **BACKLOG.md is the gate.** Only write phiếu for items in `docs/BACKLOG.md` under section "Active sprint". If Sếp's request matches an item in "Next sprint", "Open backlog", "Park", or doesn't match any item — STOP. Use `AskUserQuestion` to ask Sếp:
   - "This item is in section X of BACKLOG. Promote to Active sprint?" (options: yes / pick different active item / add as new idea via /idea / cancel)
   - Do NOT write phiếu until Sếp confirms the item is in Active sprint.
   - Exception: P0 hotfix (production down, user-impacting bug) — write phiếu, then immediately update BACKLOG.md "Active sprint" to include it post-hoc.
```

**Thay bằng:**

```markdown
0. **BACKLOG.md is the gate.** Only write phiếu for items in the **active section** of `docs/BACKLOG.md`. The active section is resolved as follows:
   - **Strict match first:** the first `## ` section whose heading contains "Active sprint" (case-insensitive substring).
   - **Fallback:** if no such heading exists, the **first `## ` section** in the file is treated as the active section. (The matching SessionStart banner script uses the same fallback — they stay in sync.)

   If Sếp's request matches an item in any **non-active** section (e.g. "Next sprint", "Open backlog", "Park", or any H2 below the active one), or doesn't match any item — STOP. Use `AskUserQuestion` to ask Sếp:
   - "This item is in section X of BACKLOG (active section is Y). Promote to active section?" (options: yes / pick different active item / add as new idea via /idea / cancel)
   - Do NOT write phiếu until Sếp confirms the item is in the active section.
   - Exception: P0 hotfix (production down, user-impacting bug) — write phiếu, then immediately update BACKLOG.md active section to include it post-hoc.
```

**Lưu ý:**
- The rule's authority is unchanged: only active-section items get phiếu. What changed: the **identification mechanism** for "active section." Strict label "Active sprint" is preserved as the preferred form; fallback is the safety net.
- The "(active section is Y)" prompt extension lets Sếp see which section the resolver picked — important for transparency on first install when a new project has e.g. `## Now` instead of `## Active sprint`.
- Tầng 2: Worker may rephrase the AskUserQuestion option labels for clarity — Discovery report. Section-resolution semantics are Tầng 1 and must match the script.

### Task 4: Regenerate `.claude/agents/architect.md` from canonical via sync script

**File:** `.claude/agents/architect.md` (do not edit directly)

**Action:** Worker runs `bash scripts/sync-personal-agents.sh` after Task 3 lands. Per CHANGELOG v2.1 Changed-section: ".claude/agents/architect.md and .claude/agents/worker.md are now sed-generated from agents/*.md. Do not edit .claude/agents/ directly — changes will be overwritten by sync-personal-agents.sh."

**Lưu ý:**
- If the sync script does not exist or errors, escalate via CHALLENGE (anchor #4 was ⏳ TO VERIFY).
- Verify post-sync: `diff <(grep "BACKLOG.md is the gate" agents/architect.md) <(grep "BACKLOG.md is the gate" .claude/agents/architect.md | sed 's/Sếp/Chủ nhà/g')` should produce zero diff. (The sync script swaps Sếp ↔ Chủ nhà; rule body otherwise identical.)

### Task 5: CHANGELOG entry

**File:** `CHANGELOG.md`

**Action:** Add a new top entry below `## [v2.1.1]` and above the existing `## [v2.1]`. Use a `[v2.1.2]` heading or, if the maintainer prefers, group under a continuation of `[v2.1.1]` — Tầng 2 decision: Worker picks based on what is currently the most recent version block at edit time.

**Suggested content (Worker may polish wording):**

```markdown
## [v2.1.2] — <date Worker ships>

### Fixed
- **BACKLOG format flexibility (P003).** `scripts/session-start-banner.sh` now falls back to the first `## ` section when no `## ... Active sprint` header is present (previously: silent exit, no banner). `agents/architect.md` Hard rule 0 wording softened to match — the active section is resolved by case-insensitive substring "Active sprint" first, then by first `## ` section. `docs/ORCHESTRATION.md` edge-case greeting (line 32) rewritten to no longer falsely claim "BACKLOG chưa có Active sprint" after fallback resolves a header. Sếp no longer needs to rename their BACKLOG sections to satisfy a literal regex. Tarot's restructured-BACKLOG workaround (2026-04-26 dogfood) is no longer required for new installs.
```

**Lưu ý:**
- Date is the day Worker commits. Do not write `<date>` — fill it in.
- Discovery Report (separate file `docs/DISCOVERIES.md`) is below in Nghiệm thu.

### Task 6: Patch `docs/ORCHESTRATION.md` line 32 edge-case greeting

> Added in V2 per Worker O1.1 (Debate Log Turn 1). Keeps the orchestrator greeting consistent with the script's new fallback behavior — script + agent + orchestrator doc all agree.

**File:** `docs/ORCHESTRATION.md`

**Tìm** (current line 32, the "Edge cases:" bullet under "Session opening"):

```markdown
- If BACKLOG has no Active sprint → greet without list: "Em là Kiến trúc sư. BACKLOG chưa có Active sprint — anh có item gì cần viết phiếu không?"
```

**Thay bằng:**

```markdown
- If BACKLOG has no recognizable section (no `## ` headings at all → SessionStart banner stayed silent) → greet without list: "Em là Kiến trúc sư. BACKLOG chưa có item nào — anh có việc gì cần viết phiếu không?" (After P003: a project whose top section is e.g. `## Now` instead of `## Active sprint` resolves via fallback and DOES get a sprint block — this edge case fires only for truly empty/malformed BACKLOGs.)
```

**Lưu ý:**
- This is a 1-line bullet replacement — preserve the surrounding "Edge cases:" header (line 30) and the preceding bullet (line 31). Only the line 32 bullet changes.
- The new wording removes the false claim "BACKLOG chưa có Active sprint" (which would lie about state in fallback-path projects) and reframes the trigger as "no `## ` headings at all" — the only case where the orchestrator actually has no SessionStart context to draw on.
- The parenthetical "(After P003: ...)" is intentional — it documents the post-fix semantics inline so future readers don't have to chase the CHANGELOG. Tầng 2: Worker may shorten the parenthetical or rephrase the user-facing greeting; Tầng 1 (must match phiếu): the greeting must NOT claim "BACKLOG chưa có Active sprint" when the fallback path may have resolved one.
- After this task, no file in the repo should contain the literal string `"BACKLOG chưa có Active sprint"`. Worker greps to confirm in Docs Gate.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `scripts/session-start-banner.sh` | Tasks 1 + 2: add fallback header resolution + fallback-used note in banner output |
| `agents/architect.md` | Task 3: soften Hard rule 0 with active-section resolution sub-rule |
| `.claude/agents/architect.md` | Task 4: regenerate via `scripts/sync-personal-agents.sh` (do not edit directly) |
| `CHANGELOG.md` | Task 5: new entry for P003 (mention all three surfaces — script, agent, orchestrator doc) |
| `docs/ORCHESTRATION.md` | Task 6: rewrite line 32 edge-case greeting to drop false "BACKLOG chưa có Active sprint" claim |
| `docs/DISCOVERIES.md` | Nghiệm thu: Discovery Report entry (newest on top) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `docs/BACKLOG.md` | After all tasks land, the existing `## 🔥 Active sprint: Drift fixes after Tarot dogfood` heading still triggers the strict-match path of the banner (no fallback note shown). Sếp's day-to-day banner UX must be unchanged. |
| `agents/worker.md` | Worker greps for "Active sprint" — should NOT contain BACKLOG-gate language. If it does, escalate via CHALLENGE — out of scope for this phiếu. |
| `phieu/TICKET_TEMPLATE.md` | Untouched. The template is project-template, not BACKLOG-resolution logic. |
| `scripts/architect-guard.sh` | Untouched. Only enforces source-read prohibition; does not parse BACKLOG. |
| `scripts/sync-personal-agents.sh` | Worker runs it (Task 4) but does NOT modify it. After running, `.claude/agents/architect.md` reflects the new Hard rule 0. |
| `docs/ORCHESTRATION.md` (other sections) | Only line 32 bullet changes. Sections "Why a 4th role", "Session opening" header/numbered steps 1–3, "State machine", "Why Kiến trúc sư persona", and any text below the "Edge cases:" block must remain byte-identical. |

---

## Luật chơi (Constraints)

1. **Backwards compatibility is mandatory.** sos-kit's own `docs/BACKLOG.md` (which has `## 🔥 Active sprint: ...`) MUST behave identically after this phiếu lands — same banner, no fallback note. Tested by running the SessionStart hook locally on a fresh shell open and observing banner is byte-identical to v2.1.1 behavior in the active-section block.
2. **No silent failures.** When fallback is used, Sếp sees the note. When neither path resolves a header, banner exits 0 silently (existing behavior — BACKLOG-less projects stay quiet).
3. **Single source of truth: canonical `agents/architect.md`.** `.claude/agents/architect.md` is regenerated, never hand-edited. Per CHANGELOG v2.1.
4. **No `set -e` in the bash script.** Existing comment on line 14 explains why; preserve it.
5. **Active-section semantics must match across script + agent + orchestrator doc.** If they drift (e.g. script picks first `^## ` but Architect's wording says first `^### `, or ORCHESTRATION.md still claims "no Active sprint" after fallback fires), Sếp loses. Worker's Task 0 re-grep must confirm all three surfaces refer to the same H2-prefix (`^## `) and the same edge-case trigger (truly empty BACKLOG, not "literal label missing").
6. **No new dependencies.** No `jq`, no `yq`, no Python helper. Pure shell + sed/grep/awk (script header line 12 explicitly states this).
7. **Tầng 2 decisions Worker self-decides + logs to Discovery:** exact wording of fallback note ("Treating … as Active sprint" vs alternatives), CHANGELOG version block grouping (`[v2.1.2]` vs continuation of `[v2.1.1]`), `sed`-strip choice for header text, exact Vietnamese phrasing of the new ORCHESTRATION.md greeting (must drop "Active sprint" claim — that's Tầng 1 — but exact word choice for "BACKLOG chưa có item nào" vs "BACKLOG trống" etc. is Tầng 2). Tầng 1 (must match phiếu): the resolution algorithm itself, the rule body's semantics, the orchestration doc's edge-case trigger condition.

---

## Nghiệm thu

### Automated

- [ ] Bash script lints clean: `shellcheck scripts/session-start-banner.sh` (if shellcheck available — sos-kit doesn't gate on it currently; do not block ship if unavailable).
- [ ] Sync script regenerates `.claude/agents/architect.md` without error: `bash scripts/sync-personal-agents.sh && echo OK`.
- [ ] Diff of canonical vs `.claude/` after sync, ignoring Sếp ↔ Chủ nhà swap, must be empty (verify command in Task 4 Lưu ý).

### Manual Testing

- [ ] **Strict-match path (regression).** In sos-kit repo root, run `bash scripts/session-start-banner.sh`. Expected: banner shows the existing `🔥 Active sprint: Drift fixes after Tarot dogfood` block, item count "2 items đang treo, 0 đã xong" (or current values), NO "Treating … as Active sprint" note.
- [ ] **Fallback path.** In a temp directory, create a minimal `docs/BACKLOG.md` containing only:
   ```markdown
   # My Project Backlog

   ## Now
   - [ ] **[X1]** Some item
   - [ ] **[X2]** Another item

   ## Later
   - [ ] **[X3]** Future thing
   ```
   `cd` into that temp dir and run `bash <path-to-sos-kit>/scripts/session-start-banner.sh`. Expected: banner shows the `## Now` block (header text + 2 open items), AND the "Treating \"Now\" as Active sprint (no \"Active sprint\" header found)." note appears once.
- [ ] **Empty/malformed path.** In a temp directory with a `docs/BACKLOG.md` containing only `# Title\n` (no `## ` headings at all), run the script. Expected: silent exit 0, no banner. (Identical to the no-BACKLOG-file case.)
- [ ] **Architect Rule 0 reading test.** In a fresh subagent invocation (or by re-reading `agents/architect.md`), confirm the new Hard rule 0 text is unambiguous — a reader unfamiliar with this phiếu can correctly identify the active section in (a) sos-kit's own BACKLOG and (b) the temp `## Now` BACKLOG without asking clarifying questions.
- [ ] **ORCHESTRATION.md edge-case greeting reading test (V2, Task 6).** Re-read `docs/ORCHESTRATION.md` "Edge cases:" block (around line 30–32). The bullet that previously claimed "BACKLOG chưa có Active sprint" must be gone. The replacement bullet must (a) trigger only on "no `## ` headings at all," (b) not falsely imply the user's BACKLOG lacks an active sprint when fallback would resolve one, and (c) preserve the bullet structure (still under the "Edge cases:" header, still next to the "concrete brief" bullet on line 31). A reader simulating the orchestrator should know which greeting to use in three cases: (i) sos-kit's own BACKLOG (active sprint block injected, normal greeting), (ii) `## Now`-style BACKLOG (sprint block injected via fallback, normal greeting), (iii) empty BACKLOG (no headings, edge-case greeting fires).

### Regression

- [ ] sos-kit's own BACKLOG.md banner output is byte-identical to v2.1.1 behavior in the active-section block (item list, count line). Only difference allowed: the "Architect Rule 0:" closing line gains "(or first ^## section if absent)" parenthetical — that change is intentional and accepted.
- [ ] No skill or agent file other than `agents/architect.md` and `.claude/agents/architect.md` references the literal phrase `"Active sprint"` as a hard requirement. Worker greps to confirm.
- [ ] `phieu` shell function still works end-to-end (creates worktree, ticket file from `phieu/TICKET_TEMPLATE.md`, no BACKLOG dependency in that path).
- [ ] `docs/ORCHESTRATION.md` sections OTHER than the line 32 bullet are byte-identical to pre-phiếu state. Worker diffs to confirm.

### Docs Gate

- [ ] `CHANGELOG.md` — entry added per Task 5 (mentions all three surfaces: script, agent, orchestrator doc).
- [ ] `docs/BACKLOG.md` — Active sprint item P003 marked `[x]` and moved to "Recently shipped" with this phiếu's commit SHA + date.
- [ ] `docs/ORCHESTRATION.md` line 32 — verified rewritten per Task 6.
- [ ] **No file in the repo contains the literal string `"BACKLOG chưa có Active sprint"`.** Worker runs `grep -rn "BACKLOG chưa có Active sprint" .` (excluding `.git/` and this phiếu file `docs/ticket/P003-backlog-format-flexibility.md` which legitimately quotes the old string in the Debate Log + Tìm/Thay-bằng blocks). Expected: zero hits outside this phiếu file.
- [ ] No other doc requires update — `docs/HANDOFF.md`, `docs/LAYERS.md`, `docs/PHILOSOPHY.md`, `docs/SETUP.md`, `phieu/README.md` do not currently mention "Active sprint" gating mechanics. Worker greps `docs/` and `phieu/` for "Active sprint" to confirm. If any other doc surfaces beyond `BACKLOG.md`, `ORCHESTRATION.md`, and this phiếu, escalate via CHALLENGE.

### Discovery Report

- [ ] Append entry to `docs/DISCOVERIES.md` (newest on top, like CHANGELOG). Required fields:
  - Phiếu ID + date.
  - **Assumptions in phiếu — CORRECT** (which Task 0 anchors held, including #10 added in V2).
  - **Assumptions in phiếu — WRONG / Adapted** (any anchor result that differed; how Worker resolved).
  - **Scope expansions** (V1 → V2: ORCHESTRATION.md added per Worker O1.1; Worker should note the debate-loop mechanic worked — Docs Gate caught a missing scope item before EXECUTE). Plus any further expansion Worker discovers in EXECUTE phase — escalate first via CHALLENGE; document outcome here.
  - **Edge cases / limitations found** — at minimum, note: "Fallback resolves to the FIRST `## ` section, not the user's intended active section if their BACKLOG places e.g. `## Future waves` before `## Now`. Mitigation: the fallback note tells Sếp which header was picked; Sếp reorders BACKLOG if wrong." Plus any other edge case Worker discovers.
  - **Docs updated to match reality** — list of files actually changed.
  - **Notes for future phiếu** — e.g. P004 (sibling drift fix) should follow the same "format-tolerant, never silent" pattern. Also: future doc-touching phiếu should pre-grep `docs/` for ALL strings whose semantics they change — V1 missed ORCHESTRATION.md only because the Architect anchor table didn't enumerate doc-side surfaces.
