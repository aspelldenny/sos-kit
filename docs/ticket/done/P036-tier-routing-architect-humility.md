# PHIẾU P036: Workflow tier routing + Architect humility rule

> **Loại:** Infra (meta-kit workflow rule)
> **Ưu tiên:** P0 (foundational — affects every future phiếu in every dogfooding project)
> **Tầng:** 1 (móng nhà — modifies state machine routing + agent contracts)
> **Ảnh hưởng:** `phieu/TICKET_TEMPLATE.md`, `docs/ORCHESTRATION.md`, `phieu/DISCOVERY_PROTOCOL.md`, `agents/architect.md`, `agents/worker.md`
> **Dependency:** None (extends existing Tầng 1/2 boundary in DISCOVERY_PROTOCOL.md)

---

## V2 changes (2026-04-27 RESPOND)

This phiếu was bumped V1 → V2 by Architect (RESPOND mode) in response to Worker CHALLENGE Turn 1.

**Verdicts:**
- **O1.1 (path drift in `agents/worker.md`):** ACCEPT + EXPAND SCOPE (Decision A). Task 5 grows to also fix the two hardcoded `docs/ticket/P<NNN>-<slug>.md` references at `agents/worker.md:48` and `agents/worker.md:90`. New Task 5c added.
- **O1.2 (stale path in `agents/architect.md:57`):** ACCEPT + EXPAND SCOPE (Decision A). Task 4 grows: new Task 4b fixes `docs/ticket/TICKET_TEMPLATE.md` → `phieu/TICKET_TEMPLATE.md` at `agents/architect.md:57`.
- **O1.3 (Task 2 conflates 4 edits):** ACCEPT. Task 2 split into Task 2a (state-machine fenced block), 2b (new "Tier routing" section), 2c (Hard rule #7), 2d (failure-modes table row). Each has its own Tìm/Thay bằng.
- **O2.1 (path issue):** Subsumed by O1.1.
- **O2.2 (renumbering math architect.md Hard rules):** ACCEPT — V1 already accounted for the bump (rule 6 → rule 7). Manual cross-check item added to Nghiệm thu.
- **O2.3 (CHANGELOG.md may not exist):** ACCEPT. Nghiệm thu Docs Gate row downgraded to `[needs Worker verify]` with explicit "create if absent" branch.

**Anchor mismatches Worker flagged:**
- #1 (TICKET_TEMPLATE.md fields) — V1 was actually correct: anchor #1 said the metadata block has fields `Loại / Ưu tiên / Ảnh hưởng / Dependency` (no `Tầng`). Re-verified against `phieu/TICKET_TEMPLATE.md:10-13` — those are the exact 4 fields. Marked `[verified]` (re-confirmed in V2).
- #2 (state-machine line 41 vs 45) — V1 said "lines 41-76" in anchor row #2 but Task 2's prose said "lines 41-76, the fenced state machine block". The fence opens at line 41, the inner `DRAFT_PHASE` text at line 45. Off-by-one in description only — `Tìm` content match still works. Task 2a now references "the fenced state machine block (opens line 41)" `[verified]`.
- #4 (architect.md:57 stale path) — confirmed pre-existing drift; absorbed into Task 4b.

**Decision rationale (A vs B for path drift):** Chose (A) Expand scope. The two stale-path edits are 2 single-line replacements in files Task 4 and Task 5 already open and edit — not "new files" by any reasonable scope-creep definition. Leaving the drift would mean Worker (post-P036) reads the freshly-edited agent file and immediately sees a broken path reference, which contradicts the very humility/honesty principle P036 introduces. (C) was rejected for this phiếu because dual-path repo-detection logic is a separate architectural concern deserving its own DRAFT.

**Anchors added in V2:** 4 new anchors (#11, #12, #13, #14), all carrying explicit verification markers per Rule B.

---

## Context

### Vấn đề hiện tại

Quan sát thực tế 2026-04-27 (Tarot project, phiếu P050 — billing surgical fix):
- Phiếu Tầng 2 (≤200 LOC, anchor đã rõ, không đụng schema/API) tốn **25m51s + 300k+ tokens**.
- Worker CHALLENGE chạy mọi phiếu bất kể tier → wasted round-trip.
- Architect over-read 153k tokens vì không có rule "biết → bảo biết, không chắc → defer Worker grep".

Hai gap chưa được spec hoá trong meta-kit:

**Gap A — CHALLENGE phase chạy unconditionally.** `docs/ORCHESTRATION.md` state machine (fenced block opens line 41, inner transitions lines 45-76) bắt buộc DRAFT → CHALLENGE → APPROVAL → EXECUTE cho mọi phiếu. Tầng 2 surgical fix không cần debate — anchor đã rõ, scope nhỏ, Worker tự verify Task 0 là đủ.

**Gap B — Architect không có anti-hallucination clause.** `agents/architect.md` (lines 134-140) chỉ nói "source from docs not imagination" — không có ký hiệu rõ ràng cho phép Architect ghi `[needs Worker verify]` thay vì bịa file:line. Hệ quả: Architect viết anchor giả vờ chắc → Worker tốn round-trip CHALLENGE để bóc.

### Giải pháp

**Rule A — Tier routing trong state machine:**
- Architect set `tầng: 1 | 2` trong header phiếu khi DRAFT.
- Heuristic Tầng 2 mặc định: ≤3 anchor files, ≤200 LOC change, không sửa schema/API contract/auth, không thêm dependency mới.
- Orchestrator branch:
  - `tầng==2`: DRAFT → APPROVAL_GATE → EXECUTE (skip CHALLENGE_PHASE + RESPOND_PHASE).
  - `tầng==1`: full flow (current behavior).
- Worker escalate-path 2→1: nếu mid-EXECUTE phát hiện đụng móng (schema/API/auth/dependency), STOP, append Debate Log Turn 1 với evidence, return → orchestrator chạy CHALLENGE flow như Tầng 1.

**Rule B — Architect humility marker:**
- Anchor markers chuẩn hoá:
  - `[verified]` — Architect đã Read file và confirm.
  - `[unverified]` — Architect reference theo docs/intuition nhưng chưa Read.
  - `[needs Worker verify]` — Architect không biết, Worker bắt buộc grep trước khi apply; nếu sai → DISCOVERY_REPORT.
- Architect MUST khowledge surface: kiến trúc tổng thể, luồng API, data flow, module boundary, schema shape.
- Architect MAY skip: code lặt vặt, helper internal, formatting, local var, CSS class.
- "Đá bóng cho Thợ" hợp lệ + được khuyến khích thay vì bịa.

### Scope

- CHỈ sửa 5 file liệt kê dưới (template + state machine + 2 agent contracts + protocol).
- KHÔNG sửa skill files (`skills/*/SKILL.md`) — tier routing is an orchestration concern, not a skill responsibility.
- KHÔNG sửa Rust tool repos (ship/docs-gate/guard/vps) — these don't see tier.
- KHÔNG migrate existing Tarot/sos-kit phiếu retroactively — new rule applies prospectively.
- **V2 scope expansion:** Tasks 4 + 5 also fix two pre-existing stale-path references (`agents/architect.md:57` and `agents/worker.md:48,90`) since those files are already being edited. This is NOT new-file scope creep; these are 2-line surgical fixes co-located with the planned edits.

---

## Task 0 — Verification Anchors

> **Architect humility note (dogfooding Rule B):** Every anchor below carries an explicit verification marker. Worker must re-confirm `[needs Worker verify]` rows via Bash/Grep before Edit; `[verified]` rows Architect already Read.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `phieu/TICKET_TEMPLATE.md` has a metadata block at lines 10-13 with fields `Loại`, `Ưu tiên`, `Ảnh hưởng`, `Dependency` (no `Tầng` field yet) | Read `phieu/TICKET_TEMPLATE.md:10-13` | ✅ `[verified]` — re-confirmed in V2 RESPOND, exact 4 fields in that order |
| 2 | `docs/ORCHESTRATION.md` state machine fenced block opens at line 41, inner `DRAFT_PHASE` transition at line 45, fence closes line 76 | Read `docs/ORCHESTRATION.md:41-76` | ✅ `[verified]` — re-confirmed in V2 RESPOND |
| 3 | `phieu/DISCOVERY_PROTOCOL.md` has "Tầng 2 mismatch" + "Tầng 1 mismatch" sub-headings around lines 28-48 | Read `phieu/DISCOVERY_PROTOCOL.md:28-48` | ✅ `[verified]` — confirmed (Tầng 2 at 28-36, Tầng 1 at 37-48) |
| 4 | `agents/architect.md` "Source your assumptions from docs, not imagination" section at lines 134-140 | Read `agents/architect.md:134-140` | ✅ `[verified]` — confirmed |
| 5 | `agents/worker.md` CHALLENGE mode workflow at lines 44-84, EXECUTE mode at lines 86-112, Tầng 1/2 table at 114-131 | Read `agents/worker.md:44-131` | ✅ `[verified]` — confirmed |
| 6 | No existing phiếu file collides at `phieu/active/P036-*.md` or `docs/ticket/P036-*.md` | `Glob phieu/active/P036* && Glob docs/ticket/P036*` | ⚠️ `[needs Worker verify]` — Architect did not Glob; Worker confirms before Write. If collision → escalate naming. |
| 7 | `agents/architect.md` "Hard rules" Rule 5 (line ~131) currently says "Tầng 1 vs Tầng 2 ... let Thợ decide" — exists as a single rule, not split | Read `agents/architect.md:131` | ✅ `[verified]` — line 131 |
| 8 | `agents/worker.md` Tầng 1/2 decision table at line 120-130 lists 8 example rows | Read `agents/worker.md:120-130` | ✅ `[verified]` — 8 rows confirmed |
| 9 | `docs/ORCHESTRATION.md` "Hard rules" section starts at line 91 with 6 numbered rules | Read `docs/ORCHESTRATION.md:91-98` | ✅ `[verified]` — 6 rules confirmed |
| 10 | Project counter for next phiếu — sos-kit uses `phieu/active/` directory (not `docs/ticket/`) per repo convention | Read `phieu/README.md` + Glob `phieu/active/*.md` | ⚠️ `[needs Worker verify]` — Architect read `phieu/README.md` (uses `docs/ticket/` for downstream projects) but the brief explicitly directed `phieu/active/P036-*.md` for sos-kit's own meta-phiếu. Worker confirms `phieu/active/` exists or creates it. |
| 11 (V2) | `agents/architect.md:57` reads `docs/ticket/TICKET_TEMPLATE.md — the format you must follow` (stale path; actual file is at `phieu/TICKET_TEMPLATE.md`) | Read `agents/architect.md:57` | ✅ `[verified]` — re-confirmed in V2 RESPOND, exact text matches |
| 12 (V2) | `agents/worker.md:48` reads `Read the phiếu file** — \`docs/ticket/P<NNN>-<slug>.md\`` (hardcoded path; sos-kit meta-phiếu live at `phieu/active/`) | Read `agents/worker.md:48` | ✅ `[verified]` — re-confirmed in V2 RESPOND |
| 13 (V2) | `agents/worker.md:90` reads `Read the phiếu file** — \`docs/ticket/P<NNN>-<slug>.md\`. This is your contract.` (hardcoded path, EXECUTE mode) | Read `agents/worker.md:90` | ✅ `[verified]` — re-confirmed in V2 RESPOND |
| 14 (V2) | sos-kit root has `CHANGELOG.md` (Docs Gate target) | `ls CHANGELOG.md` | ⚠️ `[needs Worker verify]` — Architect did not Read; if absent, Worker creates it with header `# Changelog\n\n` and the P036 entry as first row. |

**If any ❌ on row 6, 10, 11, 12, 13, or 14 → Worker escalates via Handoff 3 (path/naming is Tầng 1 — Chủ nhà's call) UNLESS the row's noted fallback applies (e.g. row 14 → create file).**

---

## Debate Log

> Auto-populated by Worker (CHALLENGE mode) and Architect (RESPOND mode). This is a **Tầng 1 phiếu** so CHALLENGE is mandatory.
> Cap = 3 turns. Sau Turn 3 chưa consensus → force-escalate Chủ nhà.

**Phiếu version:** V2 (Architect responded to Worker CHALLENGE Turn 1)

### Turn 1 — Worker Challenge
*(Worker fills this when invoked in CHALLENGE mode.)*

**Anchor verification (recap from Task 0):**
- Anchors #1-#10 verified per Worker's CHALLENGE pass; #1 field-order confirmed correct, #2 noted state-machine fence opens line 41 (V1 prose said 45 for inner content — reconciled in V2).

**Objections (Tầng 1 only — phiếu cần sửa):**
- [O1.1] `agents/worker.md:48` and `agents/worker.md:90` hardcode phiếu path as `docs/ticket/P<NNN>-<slug>.md`, but sos-kit's own meta-phiếu live in `phieu/active/`. P036 Task 5 edits worker.md without reconciling. Post-P036, Worker would still route to wrong directory for sos-kit meta-phiếu.
- [O1.2] `agents/architect.md:57` references `docs/ticket/TICKET_TEMPLATE.md` but the file is at `phieu/TICKET_TEMPLATE.md`. Pre-existing drift in the same file P036 Task 4 edits.
- [O1.3] P036 Task 2 conflates 4 distinct edits to `docs/ORCHESTRATION.md` (state-machine fenced block + new "Tier routing" section + Hard rule #7 + failure-mode row) into one Tìm/Thay bằng block. Ambiguous for Worker EXECUTE — needs to be split into 4 explicit sub-edits.

**Tầng 2 fixups (non-blocking, noted for the record):**
- [O2.1] Path issue (subsumed by O1.1).
- [O2.2] Renumbering math in `agents/architect.md` Hard rules — verify rule 6 → rule 7 bump cleanly.
- [O2.3] Nghiệm thu references `CHANGELOG.md` at sos-kit root which may not exist. Mark `[needs Worker verify]` or specify creation if absent.

**Status:** ✅ ARCHITECT RESPONDED (V2)

### Turn 1 — Architect Response (phiếu V2)

- **[O1.1]** → **ACCEPT (scope-expansion path A)** → Task 5 grows: new sub-step 5c fixes `agents/worker.md:48` and `agents/worker.md:90` to drop the hardcoded `docs/ticket/` and route per-project (sos-kit reads `phieu/active/`, downstream reads `docs/ticket/`). Fix is surgical: replace `docs/ticket/P<NNN>-<slug>.md` with the per-project phiếu directory referenced in `phieu/README.md`. See Task 5c below for exact Tìm/Thay bằng.
- **[O1.2]** → **ACCEPT (scope-expansion path A)** → Task 4 grows: new sub-step 4b fixes `agents/architect.md:57` from `docs/ticket/TICKET_TEMPLATE.md` to `phieu/TICKET_TEMPLATE.md`. Single-line surgical edit.
- **[O1.3]** → **ACCEPT** → Task 2 split into Task 2a (state-machine fenced block, surgical Tìm/Thay bằng on the DRAFT_PHASE arrow), Task 2b (new "Tier routing" section, Insert-after Tìm anchor), Task 2c (Hard rule #7, Insert-after Tìm anchor), Task 2d (failure-modes table row, Insert-after Tìm anchor). Each carries its own marker.
- **[O2.1]** → SUBSUMED by O1.1.
- **[O2.2]** → **ACCEPT** → Manual cross-check item already in Nghiệm thu Manual Testing ("Hard rules numbering in `agents/architect.md` is now 0→1→...→7"). Worker re-runs after edit.
- **[O2.3]** → **ACCEPT** → Nghiệm thu Docs Gate row updated: `[needs Worker verify]` whether `CHANGELOG.md` exists; if absent, Worker creates it with header `# Changelog\n\n` plus P036 entry. New Anchor #14 added to Task 0.

**Status:** ✅ RESPONDED — phiếu bumped to V2.  Note for Worker re-CHALLENGE (Turn 2 if spawned): all O1.x ACCEPT, no DEFER, no DEFEND. Re-grep new anchors #11-#14 + the split Task 2a/b/c/d Tìm strings before commit.

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date] — code execution may begin

---

## Nhiệm vụ

### Task 1: Add `Tầng` field to phiếu template header

**File:** `phieu/TICKET_TEMPLATE.md`  `[verified]` location

**Tìm:** (lines 10-13, the metadata block)
```
> **Loại:** Feature / Bugfix / Prompt-only / Hotfix
> **Ưu tiên:** P0 / P1 / P2
> **Ảnh hưởng:** [main files affected]
> **Dependency:** [which phiếu must finish first, or "None"]
```

**Thay bằng:**
```
> **Loại:** Feature / Bugfix / Prompt-only / Hotfix
> **Ưu tiên:** P0 / P1 / P2
> **Tầng:** 1 (móng nhà — kiến trúc/API/schema/auth/new dep) | 2 (lặt vặt — ≤3 files, ≤200 LOC, anchor rõ)
> **Ảnh hưởng:** [main files affected]
> **Dependency:** [which phiếu must finish first, or "None"]
```

**Lưu ý:**
- Field order: `Tầng` between `Ưu tiên` and `Ảnh hưởng`. Visual scan: priority → tier → blast radius.
- One-line heuristic embedded in the field itself so Architect không cần lookup separate doc.
- Default presumption: nếu Architect không chắc → ghi `Tầng: 1` (over-tier safer than under-tier, mirrors existing "default to Tầng 1" rule in DISCOVERY_PROTOCOL.md:62-63).

---

### Task 2a: Add tier-routing branch to state-machine fenced block

**File:** `docs/ORCHESTRATION.md`  `[verified]` location

**Tìm:** (the DRAFT_PHASE arrow inside the fenced block; fence opens line 41, inner text at line 45-48)
```
DRAFT_PHASE                                spawn Architect (DRAFT)
 │ Architect writes phiếu V1 with Debate Log section initialized
 ▼
CHALLENGE_PHASE                            spawn Worker (CHALLENGE)
```

**Thay bằng:**
```
DRAFT_PHASE                                spawn Architect (DRAFT)
 │ Architect writes phiếu V1 with Debate Log + sets `Tầng: 1|2` in header
 ├── tầng==2 (lặt vặt) ─────────────────────► APPROVAL_GATE  (skip CHALLENGE)
 ├── tầng==1 (móng nhà) ────────────────────► CHALLENGE_PHASE
 ▼
CHALLENGE_PHASE                            spawn Worker (CHALLENGE)
```

**Lưu ý:**
- Surgical: only edits 4 lines inside the fenced block. Surrounding fence and other transitions untouched.
- Verify the fence still closes at line 76 after the edit (3 new lines added → fence closes ~79).

---

### Task 2b: Insert "Tier routing" section after the state-machine fence

**File:** `docs/ORCHESTRATION.md`  `[verified]` location

**Tìm:** (the line immediately after the state-machine fence closes — currently line 78, header `## Trigger phrases`)
```
## Trigger phrases (orchestrator → subagent spawn prompt)
```

**Thay bằng (Insert-before — prepend a new section, then the existing header):**
```markdown
## Tier routing (P036)

Architect sets `Tầng: 1` or `Tầng: 2` in the phiếu header during DRAFT. Orchestrator branches:

| Tầng | Path | Reason |
|---|---|---|
| 2 (lặt vặt) | DRAFT → APPROVAL_GATE → EXECUTE | Surgical fix, anchor clear, ≤3 files, ≤200 LOC, no schema/API/auth/new-dep change. Worker self-verifies Task 0 in EXECUTE mode. CHALLENGE round-trip is pure overhead. |
| 1 (móng nhà) | DRAFT → CHALLENGE → [RESPOND ⇄ CHALLENGE] → APPROVAL → EXECUTE | Touches kiến trúc, API contract, data flow, schema, auth boundary, or adds dependency. Worker MUST CHALLENGE before code. |

**Tầng 2 → Tầng 1 escalation (mid-EXECUTE):** If Worker discovers during EXECUTE that the change actually touches móng nhà (schema/API/auth/new dep) — STOP. Append Debate Log Turn 1 with `file:line` evidence of the móng-nhà collision. Return to orchestrator. Orchestrator re-routes through CHALLENGE_PHASE as if phiếu had been Tầng 1 from the start. Update phiếu header `Tầng: 1` and note in Discovery Report ("escalated 2→1 mid-execute, reason: …").

**No Tầng 1 → Tầng 2 demotion mid-flow.** Once Architect declared Tầng 1, the debate runs even if it turns out trivial — sunk cost is fine, silent demotion is not (audit trail).

**Default when Architect uncertain:** `Tầng: 1`. Over-tier costs one extra CHALLENGE round-trip; under-tier risks shipping an architecturally wrong fix. Mirror of "default to Tầng 1" rule in DISCOVERY_PROTOCOL.md.

## Trigger phrases (orchestrator → subagent spawn prompt)
```

**Lưu ý:**
- Insert-before: keep existing `## Trigger phrases` line intact at the bottom of the Thay bằng block.
- New section sits between state-machine fence close and Trigger phrases — visual flow: state machine → tier routing context → spawn-phrase reference.

---

### Task 2c: Add Hard rule #7 (tier set in DRAFT, escalate up only)

**File:** `docs/ORCHESTRATION.md`  `[verified]` location

**Tìm:** (the existing Hard rule 6 at line 98)
```
6. **Marker file hygiene.** Architect-guard hook uses `.sos-state/architect-active` marker. Orchestrator must `mkdir -p .sos-state && touch .sos-state/architect-active` before spawning Architect (any mode), `rm -f .sos-state/architect-active` before spawning Worker. Never leave stale markers. (Marker lives outside `.claude/` so YOLO mode doesn't prompt — `.claude/` is gated even with `--dangerously-skip-permissions`.)
```

**Thay bằng:**
```
6. **Marker file hygiene.** Architect-guard hook uses `.sos-state/architect-active` marker. Orchestrator must `mkdir -p .sos-state && touch .sos-state/architect-active` before spawning Architect (any mode), `rm -f .sos-state/architect-active` before spawning Worker. Never leave stale markers. (Marker lives outside `.claude/` so YOLO mode doesn't prompt — `.claude/` is gated even with `--dangerously-skip-permissions`.)
7. **Tier is set in DRAFT, escalated up only.** Architect's `Tầng` declaration in the phiếu header is the routing key. Worker may escalate Tầng 2 → Tầng 1 mid-EXECUTE with `file:line` evidence; orchestrator may NOT silently demote Tầng 1 → Tầng 2. Phiếu missing the `Tầng` field is rejected pre-spawn — orchestrator re-spawns Architect with explicit "set Tầng: 1 or 2" instruction.
```

**Lưu ý:**
- Append-after rule 6, no other rule renumbering.
- Wording mirrors Constraint 5 + 7 below for consistency.

---

### Task 2d: Add failure-modes table row

**File:** `docs/ORCHESTRATION.md`  `[verified]` location

**Tìm:** (the last row of the failure-modes table at line 108)
```
| Same objection raised in 2 consecutive Worker turns | Indicates Architect didn't actually fix the underlying issue. Force-escalate. |
```

**Thay bằng:**
```
| Same objection raised in 2 consecutive Worker turns | Indicates Architect didn't actually fix the underlying issue. Force-escalate. |
| Phiếu missing `Tầng` field in header | Orchestrator rejects, re-spawns Architect with explicit "set Tầng: 1 or 2" instruction. Second failure → escalate. |
| Worker silently demoted Tầng 1 → Tầng 2 (skipped CHALLENGE on a phiếu marked Tầng 1) | Refuse — orchestrator escalates as a bug in Worker output. Tier escalation is one-way (2→1 only). |
```

**Lưu ý:**
- Append-after the existing last row. No header change.
- Two new rows: missing-field case + silent-demotion case.

---

### Task 3: Strengthen Tầng 1/2 boundary in DISCOVERY_PROTOCOL

**File:** `phieu/DISCOVERY_PROTOCOL.md`  `[verified]` location

**Tìm:** (line 64, after the existing decision tree fenced block ends)
```
      └── Default to Tầng 1. Over-escalating is fixable; silent drift is not.
```
```

**Thay bằng / Thêm:** (insert a new sub-section AFTER the closing fence of the decision tree, BEFORE "## Where it goes" at line 66)

```markdown

### Tier as a routing key (P036)

Tầng is no longer only a *Discovery-Report classification* — it is now the **routing key set in the phiếu header during DRAFT** (`Tầng: 1 | 2`). See `docs/ORCHESTRATION.md` "Tier routing".

The decision tree above still applies *during execute* (Worker found a mismatch — is it Tầng 1 or 2?). The new rule layered on top:

**Worker mid-execute escalation 2 → 1:**

If a phiếu was marked `Tầng: 2` by Architect but Worker discovers during EXECUTE that the change actually touches:
- A schema/migration
- An API contract (request/response shape, status codes, auth header)
- An auth/security boundary
- A new external dependency
- Cross-module data flow

→ STOP coding. Do NOT silently complete. Append a Debate Log Turn 1 with `file:line` evidence of the móng-nhà collision. Return to orchestrator. The phiếu re-routes through full CHALLENGE flow.

**Why this matters:** A "small" billing fix that touches `auth.ts` is not Tầng 2, even if the diff is 20 LOC. The tier is about **blast radius of what could break**, not lines changed.

**Heuristic Tầng 2 (sufficient conditions):**
- ≤3 anchor files
- ≤200 LOC change
- No schema/API contract/auth modification
- No new dependency
- All Task 0 anchors `[verified]` or surgical-only `[needs Worker verify]`

If ANY condition fails → Tầng 1.
```

**Lưu ý:**
- The existing "Quick decision tree" stays as-is — it's about classifying mismatches at execute-time, which is still valid.
- New section is purely additive: introduces tier-as-routing-key concept and the 2→1 escalation path.

---

### Task 4: Add humility rule + verification markers to Architect agent

**File:** `agents/architect.md`  `[verified]` location

**Tìm:** (lines 134-140, the "Source your assumptions from docs, not imagination" section — keep this section, append a new sub-section after it)

Current end of that section (line 140):
```
If `DISCOVERIES.md` previously flagged that a doc was wrong about something — USE the discovery correction, not the stale doc.
```

**Thêm** (insert AFTER line 140, BEFORE "## Anti-patterns" at line 142):

```markdown

## Humility markers — biết → bảo biết, không chắc → ghi rõ (P036)

Every anchor / file-path / function-name / line-number you write in a phiếu MUST carry one of three markers:

| Marker | Meaning | When to use |
|---|---|---|
| `[verified]` | I Read the file and confirmed the assumption | After actually opening the file via the `Read` tool |
| `[unverified]` | I reference per docs/intuition but did not Read | When docs strongly imply but you didn't open the file (low cost to mark — Worker re-checks anyway) |
| `[needs Worker verify]` | I do not know — Worker MUST grep before applying | When you cannot tell from docs; explicitly punt to Worker |

**Rule (anti-hallucination):** If you find yourself writing a file:line, function name, or constant name without a marker — STOP. You're about to bịa. Either Read the file (then mark `[verified]`) or downgrade to `[needs Worker verify]` and let Worker grep.

**"Đá bóng cho Thợ" is not a failure mode — it's the correct behavior.** Example:

> Task 3, File `src/lib/billing.ts`. Tìm: function `applyDiscount` `[needs Worker verify]` — Worker grep `applyDiscount\b` in `src/lib/`; if found, Edit there; if not, DISCOVERY_REPORT and ask which file holds the discount logic.

This is **better** than:

> ~~Task 3, File `src/lib/billing.ts:142`. Tìm: function `applyDiscount(amount: number, code: string)`. Thay bằng: `applyDiscount(amount: number, code: string, ctx: Ctx)`.~~

— the second example invents a line number and a signature. If either is wrong, Worker wastes a CHALLENGE round-trip discovering the lie.

### What Architect MUST know (cannot punt)

- Overall architecture / module boundaries
- API surface (routes, request/response shape)
- Data flow between modules
- Schema shape (table/column names at the conceptual level)
- Which guide doc covers what

### What Architect MAY skip (legitimate "Worker verify" territory)

- Exact line numbers
- Internal helper file paths (e.g. `lib/utils.ts` vs `lib/helpers.ts`)
- Local variable names, CSS class names, log wording
- Function signatures of helpers Architect didn't design
- Whether a constant is named `FOO` or inlined as `"foo"`

If you'd need to Read source code to know it → it's MAY-skip territory. Mark `[needs Worker verify]`.
```

**Tìm:** (line 131, Hard rule 5)
```
5. **Tầng 1 vs Tầng 2.** Your phiếu specifies Tầng 1 (architecture: file structure, function signatures, schema, API shape). Tầng 2 (local var names, CSS classes, internal helpers, error wording dev-only) — let Thợ decide, log to Discovery.
```

**Thay bằng:**
```
5. **Tầng 1 vs Tầng 2 — set the field.** Every phiếu header MUST include `Tầng: 1` or `Tầng: 2`. Tầng 1 = móng nhà (kiến trúc, API contract, schema, auth, new dep). Tầng 2 = lặt vặt (≤3 files, ≤200 LOC, no schema/API/auth/dep change, anchor rõ). Default uncertainty → Tầng 1. See `docs/ORCHESTRATION.md` "Tier routing" for state-machine impact (Tầng 2 skips CHALLENGE).
6. **Humility markers mandatory.** Every code-level anchor (file path, function name, line number, constant) carries `[verified]` / `[unverified]` / `[needs Worker verify]`. No bare anchors. See "Humility markers" section below.
```

**Lưu ý:**
- This bumps existing rule 6 (Voice in phiếu) to rule 7 — renumber.
- New rule 6 (humility markers) cross-references the new section added above.

---

### Task 4b (V2 — O1.2 ACCEPT): Fix stale TICKET_TEMPLATE path in architect.md

**File:** `agents/architect.md`  `[verified]` location at line 57

**Tìm:** (line 57, inside the DRAFT mode "Load context" list)
```
   - `docs/ticket/TICKET_TEMPLATE.md` — the format you must follow
```

**Thay bằng:**
```
   - `phieu/TICKET_TEMPLATE.md` — the format you must follow (canonical location in sos-kit; downstream projects may symlink or copy to `docs/ticket/TICKET_TEMPLATE.md`)
```

**Lưu ý:**
- Single-line edit. Anchor #11 confirms the exact V1 text.
- The parenthetical clarifies that downstream-project conventions still allow `docs/ticket/` — the file *itself* lives at `phieu/TICKET_TEMPLATE.md` in sos-kit (template source), and downstream projects may point to either location.
- This fix is co-located with the Task 4 humility-markers edit (same file) — true scope-creep would be touching new files.

---

### Task 5: Add tier-aware CHALLENGE selectivity to Worker agent

**File:** `agents/worker.md`  `[verified]` location

**Tìm:** (line 39-40, the Invocation modes table CHALLENGE row)
```
| **CHALLENGE** | "Worker challenge phiếu P<NNN>", "review phiếu pre-code", "verify phiếu against code" | Read phiếu + verify Task 0 + read real code → write Debate Log Turn N → **DO NOT code, DO NOT commit, return** |
```

**Thay bằng:**
```
| **CHALLENGE** | "Worker challenge phiếu P<NNN>", "review phiếu pre-code", "verify phiếu against code" | **Only spawned for Tầng 1 phiếu.** Read phiếu + verify Task 0 + read real code → write Debate Log Turn N → **DO NOT code, DO NOT commit, return**. For Tầng 2 phiếu, orchestrator routes DRAFT → APPROVAL → EXECUTE directly (skip CHALLENGE). |
```

**Tìm:** (line 86, "## EXECUTE mode workflow" header) — keep header, replace the body's step 4 to add 2→1 escalation.

Current step 4 (line 93-94):
```
4. **Run Task 0 verification** — even in EXECUTE mode. The Debate Log may have aged; re-check that anchors still hold.
   - If ANY ❌ or ⚠️ that wasn't already addressed in Debate Log → STOP. Write a new Debate Log Turn (or escalate via Handoff 3). Do NOT code.
```

**Thay bằng:**
```
4. **Run Task 0 verification** — even in EXECUTE mode. The Debate Log may have aged; re-check that anchors still hold.
   - For each anchor: if marker is `[verified]` and grep confirms → proceed. If `[unverified]` or `[needs Worker verify]` → grep now, mark result in Result column.
   - If ANY ❌ or ⚠️ that wasn't already addressed in Debate Log → STOP. Write a new Debate Log Turn (or escalate via Handoff 3). Do NOT code.
4a. **Tier escalation check (Tầng 2 phiếu only).** Before writing any code, scan the actual diff scope:
   - Touches schema/migration? → STOP, escalate 2→1.
   - Modifies API contract (route, request/response, auth header)? → STOP, escalate 2→1.
   - Adds a new dependency to package.json / Cargo.toml / requirements.txt? → STOP, escalate 2→1.
   - Touches auth/security boundary? → STOP, escalate 2→1.
   - Changes cross-module data flow? → STOP, escalate 2→1.

   To escalate: append Debate Log Turn 1 with `file:line` evidence of móng-nhà collision, update phiếu header `Tầng: 1`, return to orchestrator. Note in Discovery Report: "escalated 2→1 mid-execute, reason: <which trigger fired>".
```

**Tìm:** (line 131, the "When in doubt, default to Tầng 1" sentence — keep, but add a note after the existing Tầng 1/2 table)

Current line 131:
```
**When in doubt, default to Tầng 1.** Over-escalating is fixable; silent drift is not.
```

**Thêm** (insert AFTER line 131, BEFORE "## Hand-back format" at line 133):

```markdown

### Tier escalation 2 → 1 (P036)

If phiếu was marked `Tầng: 2` but mid-EXECUTE you discover the change touches móng nhà → STOP, escalate. The triggers in step 4a above are exhaustive — if none fire, the phiếu stays Tầng 2 and you ship.

**You may NEVER demote Tầng 1 → Tầng 2.** If Architect declared Tầng 1, the debate already happened (or will). Worker's only escalate direction is upward.

### Anchor markers — verifying Architect's humility (P036)

Phiếu anchors carry `[verified]` / `[unverified]` / `[needs Worker verify]` markers. Your verification protocol:

| Marker | Worker action |
|---|---|
| `[verified]` | Re-grep anyway (Task 0 is mandatory); flag mismatch as Tầng 1 if found |
| `[unverified]` | Re-grep; same mismatch handling |
| `[needs Worker verify]` | **Architect explicitly punted — your job to grep + decide.** If anchor found → apply. If not found → DISCOVERY_REPORT with what you actually found, do NOT silently invent a path. |

**The marker is informational — Task 0 verification is unconditional.** Markers tell you *Architect's confidence*, not whether you can skip verifying.
```

**Lưu ý:**
- The existing Tầng 1/2 table (lines 120-130) stays as-is — it's about *mid-execute mismatch classification*, which is still valid alongside the new tier-as-routing-key concept.
- New step 4a in EXECUTE mode runs ONLY for `Tầng: 2` phiếu (Tầng 1 already had CHALLENGE — móng nhà concerns surfaced there).

---

### Task 5c (V2 — O1.1 ACCEPT): Generalise hardcoded phiếu paths in worker.md

**File:** `agents/worker.md`  `[verified]` location at lines 48 and 90

**Tìm 1:** (line 48, CHALLENGE mode workflow step 1)
```
1. **Read the phiếu file** — `docs/ticket/P<NNN>-<slug>.md`. Note the Phiếu version (V1, V2, ...) in the Debate Log section.
```

**Thay bằng 1:**
```
1. **Read the phiếu file** — at the project's phiếu directory (sos-kit: `phieu/active/P<NNN>-<slug>.md`; downstream projects: `docs/ticket/P<NNN>-<slug>.md` per `phieu/README.md`). Note the Phiếu version (V1, V2, ...) in the Debate Log section.
```

**Tìm 2:** (line 90, EXECUTE mode workflow step 1)
```
1. **Read the phiếu file** — `docs/ticket/P<NNN>-<slug>.md`. This is your contract. Read the Debate Log so you know which decisions Architect already responded to.
```

**Thay bằng 2:**
```
1. **Read the phiếu file** — at the project's phiếu directory (sos-kit: `phieu/active/P<NNN>-<slug>.md`; downstream projects: `docs/ticket/P<NNN>-<slug>.md` per `phieu/README.md`). This is your contract. Read the Debate Log so you know which decisions Architect already responded to.
```

**Lưu ý:**
- Two single-line edits, both inside the same file already touched by Task 5.
- The wording matches Task 4b's pattern (canonical-location parenthetical) so both agent files stay consistent.
- A future phiếu may introduce repo-detection logic (path C from RESPOND decision matrix) — explicitly OUT OF SCOPE here.
- After this edit, `grep -n "docs/ticket/P<NNN>" agents/worker.md` should return zero hits (sanity check in Nghiệm thu).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `phieu/TICKET_TEMPLATE.md` | Task 1: add `Tầng:` field to header metadata block (line 12 area) |
| `docs/ORCHESTRATION.md` | Task 2a: add tier branch to state-machine fenced block; Task 2b: insert "Tier routing" section before Trigger phrases; Task 2c: add Hard rule 7; Task 2d: add 2 failure-modes rows |
| `phieu/DISCOVERY_PROTOCOL.md` | Task 3: add "Tier as a routing key" sub-section + 2→1 escalation rule + heuristic |
| `agents/architect.md` | Task 4: add "Humility markers" section + split Hard rule 5 into 5+6 (tier set + markers mandatory); Task 4b (V2): fix stale `docs/ticket/TICKET_TEMPLATE.md` path at line 57 |
| `agents/worker.md` | Task 5: CHALLENGE row notes Tầng 1 only + EXECUTE step 4a tier-escalation check + marker-handling table; Task 5c (V2): generalise hardcoded `docs/ticket/P<NNN>-<slug>.md` paths at lines 48 and 90 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `skills/plan/SKILL.md` | Skill content references phiếu structure — no break expected since new field is additive. Worker greps `Tầng` to confirm no stale assumptions. `[needs Worker verify]` |
| `skills/verify/SKILL.md` | Task 0 skill — should still work; markers are additive context. `[needs Worker verify]` |
| `docs/LAYERS.md` | 3-role model — tier routing doesn't change layer separation. Worker confirms no contradiction. `[needs Worker verify]` |
| `docs/HANDOFF.md` | Handoff 2 (Architect → Worker) format — tier field is metadata, doesn't change handoff. `[needs Worker verify]` |
| `phieu/phieu.sh` | Shell function creates ticket file from template — auto-picks up new `Tầng:` field, no script change needed. `[needs Worker verify]` |
| `phieu/VISION_TEMPLATES/*.md` | Vision templates don't reference phiếu structure. No change. |
| `README.md` | Top-level — may want a 1-line mention of tier routing under "Phiếu workflow", but Tầng 2 (Worker self-decides if to add). |

---

## Luật chơi (Constraints)

1. **Backward compat:** existing phiếu without `Tầng:` field → orchestrator treats as Tầng 1 (safe default). Migration is prospective only; do not retro-edit.
2. **No new tools required.** All changes are markdown edits. Phiếu shell function untouched.
3. **No skill files touched.** Tier is orchestration concern; skills stay layer-pure.
4. **Markers are informational, verification is unconditional.** Worker MUST run Task 0 grep regardless of marker (`[verified]` doesn't mean "skip checking" — it means "Architect read the file once").
5. **Tier escalation is one-way (2→1 only).** Once Architect declares Tầng 1, debate runs. No silent demotion.
6. **Default to Tầng 1 on uncertainty** — mirrors existing DISCOVERY_PROTOCOL.md:62-63 rule.
7. **Worker CHALLENGE phase only spawns for Tầng 1 phiếu.** Orchestrator skip rule is enforced in `docs/ORCHESTRATION.md` state machine — agent files inherit.
8. **Humility markers required on every code-level anchor in every future phiếu.** Bare anchors (file:line with no marker) → phiếu rejected at orchestrator's marker-lint check (Worker enforces during CHALLENGE for Tầng 1; Worker enforces during EXECUTE Task 0 for Tầng 2).
9. **(V2)** Path-drift fixes in Tasks 4b + 5c are surgical co-located edits ONLY. Do NOT extend to other files referencing `docs/ticket/` — those are P038 territory (separate phiếu, repo-detection design).

---

## Nghiệm thu

### Automated
- [ ] Markdown lint clean on all 5 changed files (no broken table syntax, no orphan headers).
- [ ] `grep -n "Tầng" phieu/TICKET_TEMPLATE.md` returns ≥1 hit (field added).
- [ ] `grep -n "tầng==2" docs/ORCHESTRATION.md` returns ≥1 hit (state machine branch added — Task 2a).
- [ ] `grep -n "Tier routing (P036)" docs/ORCHESTRATION.md` returns ≥1 hit (Task 2b section added).
- [ ] `grep -nE "^7\. \*\*Tier is set in DRAFT" docs/ORCHESTRATION.md` returns ≥1 hit (Task 2c rule 7 added).
- [ ] `grep -n "Phiếu missing \`Tầng\` field" docs/ORCHESTRATION.md` returns ≥1 hit (Task 2d failure-mode row added).
- [ ] `grep -n "needs Worker verify" agents/architect.md` returns ≥1 hit (humility marker section added).
- [ ] `grep -n "phieu/TICKET_TEMPLATE.md" agents/architect.md` returns ≥1 hit AND `grep -n "docs/ticket/TICKET_TEMPLATE.md" agents/architect.md` returns 0 hits (Task 4b path fix).
- [ ] `grep -n "Only spawned for Tầng 1" agents/worker.md` returns ≥1 hit (CHALLENGE row updated).
- [ ] `grep -nE "docs/ticket/P<NNN>-<slug>\.md" agents/worker.md` returns 0 hits (Task 5c hardcoded-path removal).
- [ ] `grep -n "Tier as a routing key" phieu/DISCOVERY_PROTOCOL.md` returns ≥1 hit (new section added).

### Manual Testing
- [ ] Re-read each of the 5 modified files end-to-end — check no section was orphaned, no broken cross-reference.
- [ ] Cross-check: Hard rules numbering in `agents/architect.md` is now 0→1→2→3→4→5→6→7 (was 0→1→...→6) — sequential, no gaps. (O2.2 verification.)
- [ ] Cross-check: `docs/ORCHESTRATION.md` Hard rules numbering 1→2→3→4→5→6→7 sequential.
- [ ] Cross-check: `docs/ORCHESTRATION.md` failure-modes table now has the 2 new rows (missing-field + silent-demotion) appended after the original 5.

### Regression (dogfood test — REQUIRED for acceptance)
- [ ] **P037 dry-run:** spawn Architect to draft a small Tầng 2 phiếu (e.g., "fix typo in `phieu/README.md` line 12"). Architect MUST set `Tầng: 2` in header. Orchestrator MUST route DRAFT → APPROVAL_GATE → EXECUTE (skip CHALLENGE). End-to-end token cost should drop ≥40% vs current Tầng-1-by-default flow.
- [ ] **P037 humility marker test:** the Tầng 2 phiếu MUST contain ≥1 anchor marked `[needs Worker verify]`. Worker (in EXECUTE Task 0) successfully greps + applies + logs to Discovery Report.
- [ ] **2→1 escalation test:** spawn Architect to draft a Tầng 2 phiếu that secretly touches schema (e.g., "add a field to a Prisma model"). Worker mid-EXECUTE detects schema change in step 4a, STOPS, escalates 2→1, orchestrator re-routes through CHALLENGE. Verify Debate Log shows the escalation evidence.

### Docs Gate
- [ ] **`CHANGELOG.md` (sos-kit root) `[needs Worker verify]`** — Anchor #14: if file exists, append entry `- P036: tier routing in state machine + Architect humility markers + path-drift fixes (V2)`. If file does NOT exist, create it with header `# Changelog\n\n` and the P036 entry as the first row. (O2.3 ACCEPT — explicit creation branch.)
- [ ] `README.md` (sos-kit root) — section "Phiếu workflow" or equivalent: 1-line note that phiếu carry `Tầng: 1|2` and Tầng 2 skip CHALLENGE. `[needs Worker verify]` whether README has this section already.

### Discovery Report
- [ ] Append entry to `docs/DISCOVERIES.md` (sos-kit's own — `[needs Worker verify]` if file exists; create if not):
  - Assumptions in phiếu — CORRECT / WRONG (per anchor #1-#14, including V2-added #11-#14)
  - Path collision (anchor #6) result
  - `phieu/active/` vs `docs/ticket/` resolution (anchor #10)
  - CHANGELOG.md existed / was created (anchor #14)
  - V2 scope expansion: did Tasks 4b + 5c land cleanly without secondary-edit fallout?
  - Any rules numbering gotcha discovered while editing
  - Cross-module impact found (skill files, README) — was Constraint 3 right?
