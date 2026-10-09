# PHIẾU P085: Codify phiếu-decomposition heuristic vào ORCHESTRATION.md

> **Loại:** Prompt-only (docs-doctrine, additive)
> **Ưu tiên:** P2
> **Tầng:** 1 — `docs/ORCHESTRATION.md` là contract surface (orchestrator routing spec). CLAUDE.md Rule #9: contract-surface doc edit → qua 1 vòng CHALLENGE dù nhỏ. Sai thì LAN (mọi phiếu-scoping call của Quản đốc + Codex đọc doctrine này). Additive-only, reversible, nhưng consequence lan → Tầng 1.
> **Lane:** Normal
> **Ảnh hưởng:** `docs/ORCHESTRATION.md` (section mới), `agents/orchestrator.md` (1-dòng pointer), `CHANGELOG.md`
> **Dependency:** None

---

## Context

### Vấn đề hiện tại

Doctrine hiện CÓ luật scope **cơ học**: Lane budget (`docs/WORKFLOW_V2.2.md` §1, gate `doctor lane-check`, Normal ≤250 dòng/5 anchor/5 constraint) + one-delivery-unit (`core/WORKFLOW.md:75-81`). NHƯNG **KHÔNG có luật "khi nào tách 1 phiếu thành N sub-phiếu"** — đó đang là judgment thuần. Sprint P077/P078 tách ~7 lần (P077c1-c6, P078d2a/b, b1/b2/b3...) mà tiêu chí split nằm trong đầu orchestrator, không tường minh → không auditable, và **không transfer khi Codex cũng orchestrate (dual-runtime P080)** — judgment không cross-runtime.

Sếp đã chốt 2026-07-22 (chọn (a) sau phân tích lợi/hại): codify để (a) ngầm→tường minh + auditable, (b) nhất quán cross-runtime. "fix nhẹ sau" = giữ mở, iterable.

### Giải pháp

Thêm 1 section **"Phiếu decomposition heuristic (guidance, not a gate)"** vào `docs/ORCHESTRATION.md`, đặt ngay sau section "Tier routing (P036)" (kết ở dòng 170, trước "Trigger phrases" dòng 172) — vì decompose là pre-DRAFT scoping call, cùng họ routing. Nội dung: 5 tiêu chí SPLIT + caveat KEEP-WHOLE, mỗi cái 1 dòng + ví dụ thật từ sprint, + 1 dòng đóng khung "guidance not gate (§0.1)".

**QUAN TRỌNG (§0.1 Luật 3):** đây là JUDGMENT ("2 oracle mâu thuẫn" không grep được) → phải là **guidance, KHÔNG gate**. Lane budget (§1) vẫn là hard gate DUY NHẤT; 5 tiêu chí này chỉ guide khi nào một phiếu "Guarded-but-large" nên thành N sub-phiếu. KHÔNG đặt vào `docs/WORKFLOW_V2.X.md` → KHÔNG cần retro process.

### Scope
- CHỈ sửa `docs/ORCHESTRATION.md` (thêm section), `agents/orchestrator.md` (1-dòng pointer), `CHANGELOG.md`.
- KHÔNG sửa `docs/WORKFLOW_V2.2.md` (không đổi §1 gate — heuristic KHÔNG được mâu thuẫn §1), `core/WORKFLOW.md`, `docs/LAYERS.md`.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `WORKFLOW_V2.2.md` §1 Lane budget = gate (Normal ≤250 dòng/5 anchor/5 constraint, `doctor lane-check`) + §0.1 Luật 3 (mechanical→gate, judgment→guidance) | `grep -n "Lane budgets\|Luật 3" docs/WORKFLOW_V2.2.md` | ✅ §1 dòng 59-96 (budget 67-77, gate 87); §0.1 Luật 3 dòng 50-55 `[verified]` |
| 2 | one-delivery-unit ở `core/WORKFLOW.md:75-81` ("One ticket is one delivery unit") | Read core/WORKFLOW.md:75-81 | ✅ dòng 75-81 `[verified]` |
| 3 | ORCHESTRATION.md có section "Tier routing (P036)" kết ~dòng 170, "Trigger phrases" bắt đầu dòng 172 | Read ORCHESTRATION.md:157-172 | ✅ Tier routing 157-170, Trigger phrases 172 `[verified]` |
| 4 | Ví dụ split thật tồn tại trong docs/ticket/: P077c*, P078d2a/b, P078b3, P078a | `ls docs/ticket/P077c* P078d2* P078b3* P078a*` | ✅ P077c1-c6, P078d2a, P078d2b, P078b3-codex-enforcement, P078a-core-adapter-spec `[verified]` |
| 5 | orchestrator.md có "Tier routing (P036)" section (dòng 45-51) + "Hard rules" (dòng 155) để chèn pointer | Read agents/orchestrator.md:45-51,155-164 | ✅ Tier routing 45-51, Hard rules 155-164 `[verified]` |

Tất cả 5 anchor `[verified]` — Architect đã Read các file docs (không phải source). Không có ❌/⚠️. (5 anchor = trong Normal lane cap ≤5.)

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/ — auto-cleaned on phieu-done"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge
**Worker accepted V1 — no challenges.** Anchor verification: all 5 ✅ (§1 lane-budget location + Luật 3 confirmed at WORKFLOW_V2.2.md:59/50-55; core/WORKFLOW.md:75-81 one-delivery-unit confirmed; ORCHESTRATION.md Tier-routing 157-170 / Trigger phrases 172 confirmed; P077c1-c6/P078a/P078b3/P078d2a/P078d2b all exist in `docs/ticket/` and their real content matches the cited examples — P077c c1-c4 parity vs c5 correctness, P078d2a ships now vs d2b waits on SubagentStart probe, P078b3/d2a both explicit security-surface AUTO-Tầng-1, P078a single cohesive spec; orchestrator.md Tier-routing 45-51 / Hard rules 155 confirmed). Framing correctly frames guidance-not-gate with lane budget as sole hard gate — no second gate created.
Ready for Chủ nhà approval gate.

**Status:** ✅ APPROVED V1 — Chủ nhà approved, routed straight to EXECUTE (sprint delegate).

### Final consensus
- Phiếu version: V1
- Total turns: 1
- Approved by Chủ nhà: 2026-07-23

---

## Nhiệm vụ

### Task 1: Thêm section "Phiếu decomposition heuristic" vào ORCHESTRATION.md

**File:** `docs/ORCHESTRATION.md`

**Tìm:** cuối section "Tier routing (P036)" — dòng cuối là:

```
**Default when Architect uncertain:** `Tầng: 1`. Over-tier costs one extra CHALLENGE round-trip; under-tier risks shipping an architecturally wrong fix. Mirror of "default to Tầng 1" rule in DISCOVERY_PROTOCOL.md.
```

(theo Anchor #3 = dòng ~170, ngay trước `## Trigger phrases`)

**Thêm** (chèn section MỚI giữa dòng trên và `## Trigger phrases (orchestrator → subagent spawn prompt)`):

```markdown
## Phiếu decomposition heuristic (guidance, not a gate) (P085)

*When the orchestrator is scoping a brief pre-DRAFT, these signals say "split this into N sub-phiếu" instead of writing one large phiếu. Judgment, not mechanical — §0.1 Luật 3 (WORKFLOW_V2.2.md:50-55). Only #1 has a gate.*

**SPLIT when any of:**

1. **Lane budget exceeded (mechanical, the only gate).** Phiếu > Normal 250 dòng / 5 anchor / 5 constraint → either split OR declare Guarded/Fast. This is the one criterion with a hard gate — see `docs/WORKFLOW_V2.2.md` §1 + `doctor lane-check`. The other four below are guidance for *when a Guarded-but-large phiếu should instead become N sub-phiếu*.
2. **Incompatible oracles.** Two pass/fail axes that contradict inside one phiếu → split so each gate is self-consistent. VD P077c: c1-c4 assert parity `Rust == Bash` while c5 asserts correctness `Rust BEAT Bash` (fix a Bash bug) — one phiếu holding both = a gate that contradicts itself.
3. **External-input blocker.** One part waits on data only an outside actor can supply (Codex probe / founder decision / real tool version) → split so the unblocked part ships now. VD P078d2 → d2a (guard multipath bootstrap, ships) + d2b (enforcement, waited on a `SubagentStart` probe result).
4. **Security-surface isolation.** A security-critical slice needs concentrated CHALLENGE → split it out so the debate focuses. VD P078b3 (codex enforcement), P078d2a (guards).
5. **Delivery/rollback clarity.** Each split = one independently revertible delivery unit — the one-ticket-one-delivery contract (`core/WORKFLOW.md:75-81`).

**KEEP-WHOLE when:** one coherent oracle + one cohesive file + splitting would only create pointless merge-order pain. VD P078a (core-adapter-spec) stayed a single phiếu — one spec, one oracle, no gain from cutting.

These are heuristics for the orchestrator's pre-DRAFT scoping call — judgment, not a mechanical gate (§0.1). Lane budget is the only hard gate; the rest guide when Guarded-but-large should instead become N sub-phiếu. Iterate freely as evidence accrues.
```

**Lưu ý:** Additive-only. KHÔNG đổi bất kỳ dòng nào của section "Tier routing" hay "Trigger phrases". Section mới nằm TRỌN giữa hai section đó. Giữ nguyên heading style (`##`) khớp các section peer.

### Task 2: Thêm 1-dòng pointer vào orchestrator.md

**File:** `agents/orchestrator.md`

**Tìm:** cuối section "Tier routing (P036)" (Anchor #5 = dòng ~51), dòng:

```
Worker may escalate Tầng 2 → Tầng 1 mid-EXECUTE; you may NEVER demote Tầng 1 → Tầng 2. **LOC is not a Tầng signal — never downgrade because the diff looks small.**
```

**Thêm** (1 dòng ngay sau dòng trên, trong cùng section):

```markdown

**Phiếu decomposition (pre-DRAFT scoping):** when a brief is large, see `docs/ORCHESTRATION.md` "Phiếu decomposition heuristic" for the 5 split signals (incompatible oracles / external-input blocker / security-surface isolation / delivery clarity / lane budget). Guidance, not a gate — only lane budget (§1) blocks.
```

**Lưu ý:** CHỈ pointer, KHÔNG chép nội dung heuristic (heuristic là guidance, không phải hard rule → KHÔNG thêm vào "Hard rules" list, chỉ neo vào Tier-routing section như một con trỏ). Single-source ở ORCHESTRATION.md, tránh drift 2-nơi.

### Task 3: CHANGELOG entry

**File:** `CHANGELOG.md`

**Tìm:** entry mới nhất trên đầu (newest-on-top).

**Thêm:** entry cho P085 phía trên cùng, nội dung: "docs(P085): codify phiếu-decomposition heuristic in ORCHESTRATION.md — 5 split signals (incompatible oracles / external-input blocker / security-surface / delivery clarity / lane budget) + keep-whole caveat. Guidance not gate (§0.1); lane budget stays the only mechanical gate. Pointer mirrored in agents/orchestrator.md."

**Lưu ý:** Format khớp các entry hiện có (Worker verify style header trong file — có thể là `## [Unreleased]` hoặc date-stamped; port theo dòng ngay trên).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `docs/ORCHESTRATION.md` | Task 1: section mới "Phiếu decomposition heuristic" sau "Tier routing (P036)" |
| `agents/orchestrator.md` | Task 2: 1-dòng pointer trong section "Tier routing (P036)" |
| `CHANGELOG.md` | Task 3: entry P085 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `docs/WORKFLOW_V2.2.md` | §1 Lane budget vẫn là gate DUY NHẤT — heuristic mới KHÔNG mâu thuẫn/không thêm gate thứ 2. KHÔNG edit. |
| `core/WORKFLOW.md` | one-delivery-unit dòng 75-81 chỉ được *cite*, KHÔNG sửa. |
| `docs/LAYERS.md` | Không đụng access matrix / 2-tier. |

---

## Luật chơi (Constraints)

1. **Additive-only.** KHÔNG xóa/đổi dòng nào của section "Tier routing" hay "Trigger phrases" trong ORCHESTRATION.md. Section mới chèn nguyên vẹn giữa hai section đó.
2. **Guidance, KHÔNG gate (§0.1 Luật 3).** Framing phải nói rõ đây là judgment; chỉ tiêu chí #1 (lane budget) có gate và nó là gate DUY NHẤT. KHÔNG viết như "MUST split" mechanical.
3. **KHÔNG mâu thuẫn §1.** Không được ngụ ý một gate scope thứ 2 song song lane-check. Lane budget vẫn tối thượng cho size.
4. **Ví dụ phải là phiếu THẬT** (verified Anchor #4): P077c*, P078d2a/b, P078b3, P078a. KHÔNG bịa ID.
5. **orchestrator.md CHỈ pointer** — không nhân đôi nội dung (single-source ORCHESTRATION.md).

---

## Nghiệm thu

### Automated
- [ ] KHÔNG có (docs-only, không type-check/test). `doctor lane-check --ticket docs/ticket/P085-decompose-heuristic.md` → exit 0 (phiếu này ≤ Normal budget: 5 anchor, 5 constraint).

### Manual Testing (oracle docs — Worker verify)
- [ ] `docs/ORCHESTRATION.md` có section "Phiếu decomposition heuristic (guidance, not a gate) (P085)" nằm giữa "Tier routing (P036)" và "Trigger phrases".
- [ ] Section chứa đủ 5 tiêu chí SPLIT + caveat KEEP-WHOLE + 1 dòng đóng khung "guidance not gate".
- [ ] Mỗi ví dụ cite đúng phiếu thật (P077c / P078d2a/b / P078b3 / P078a) — cross-check với `ls docs/ticket/`.
- [ ] Text nói rõ lane budget (§1) là gate DUY NHẤT; KHÔNG mâu thuẫn §1.
- [ ] `agents/orchestrator.md` có 1-dòng pointer tới section (không chép nội dung).
- `[oracle: section-present + consistency-with-§1 + examples-real]` — Worker grep section heading tồn tại, grep 5 ví dụ ID khớp `docs/ticket/`, đọc framing xác nhận không thêm gate thứ 2.

### Regression
- [ ] Section "Tier routing" + "Trigger phrases" trong ORCHESTRATION.md không đổi (diff chỉ thấy chèn thêm).
- [ ] "Hard rules" list trong orchestrator.md KHÔNG có item mới (pointer nằm ở Tier-routing section, không phải hard rule).

### Docs Gate
- [ ] `CHANGELOG.md` — entry P085.
- [ ] `agents/orchestrator.md` — pointer mirror (per CLAUDE.md DOCS-GATE: `docs/ORCHESTRATION.md` thêm section → mirror orchestrator.md nếu là hard rule; ở đây heuristic là guidance → CHỈ pointer, đủ). CLAUDE.md KHÔNG cần sửa (không có row map cho "ORCHESTRATION thêm guidance section"; không đổi phase-count/gate inventory).

### Discovery Report
- [ ] Write to `docs/discoveries/P085.md`:
  - Assumptions CORRECT/WRONG (Task 0 anchors — kỳ vọng all CORRECT, docs-only)
  - Tầng 1 docs updated: ORCHESTRATION.md + orchestrator.md pointer + CHANGELOG (đây là contract-surface add, KHÔNG phải cosmetic)
  - Ví dụ split có cite sai ID nào không
  - Tier escalations: kỳ vọng "None"
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
