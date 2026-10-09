# PHIẾU P082: lane-field template drift fix (OA-01)

> **ID format:** `P082` — assigned manually (out-of-sprint, Sếp direct approval; không qua `phieu` counter).
> **Filename:** `docs/ticket/P082-lane-field-template.md`
> **Branch:** `fix/P082-lane-field-template`

---

> **Loại:** Bugfix (integration gap)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — contract surface: ticket schema ↔ `doctor lane-check` ↔ orchestrator pre-CHALLENGE gate. Workflow-gate touch → AUTO Tầng 1 dù diff nhỏ. Def: `docs/LAYERS.md` §2-tier.)
> **Lane:** Normal (budget axis — fix surgical, < 250 dòng. Def: `docs/WORKFLOW_V2.2.md` §1.)
> **Ảnh hưởng:** `phieu/TICKET_TEMPLATE.md`, `scripts/lane-check-contract.sh` (mới), `hooks/pre-commit`, docs Tầng-1.
> **Dependency:** None.
> **⚠️ Out-of-sprint:** Active sprint = runtime-portability (P076-P081). Phiếu này **KHÔNG** thuộc sprint đó — **Chủ nhà direct approval, Sếp-ratified 2026-07-22** (override Architect Rule 0). Audit finding OA-01 🔴, gate đang chết trên MỌI phiếu canonical → xử lý ngay.

---

## Context

### Vấn đề hiện tại

Audit `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md` finding **OA-01 🔴** (reproduced):

- `agents/orchestrator.md` "Lane budget pre-CHALLENGE gate" bắt chạy `doctor lane-check --ticket <path>` trước mỗi Worker CHALLENGE (Tầng-1 phiếu).
- `doctor lane-check` đòi field `**Lane:** Normal|Guarded|Fast` trong header phiếu.
- `phieu/TICKET_TEMPLATE.md` header CHỈ có `**Tầng:** 1|2`, **KHÔNG** có `**Lane:**`.
- Kết quả: `doctor lane-check phieu/TICKET_TEMPLATE.md` → **exit 2 "ticket missing Lane field"** trên mọi phiếu sinh từ template. Gate chống-phình được viết + unit-test tốt nhưng **không chạy được trên phiếu canonical** — integration gap, mỗi component tự-xanh nhưng state machine không nối.

### Chẩn đoán gốc (KHÔNG phải taxonomy nhầm — 2 trục độc lập by-design)

- **`Tầng` (1/2)** = trục **CONSEQUENCE** → quyết định **debate flow** (Tầng 1 CHALLENGE, Tầng 2 skip). Def: `docs/LAYERS.md` §2-tier.
- **`Lane` (Normal/Guarded/Fast)** = trục **BUDGET** → cap **size phiếu** (dòng/anchor/constraint). Def: `docs/WORKFLOW_V2.2.md` §1 (Normal ≤250 dòng/≤5 anchor/≤5 constraint · Guarded no-cap · Fast ≤100 dòng, no-architect).

Cả doctrine §1 lẫn `doctor` xây ĐÚNG. Lỗ hổng = `TICKET_TEMPLATE.md` **chưa bao giờ được thêm field `Lane:`** khi v2.2 §1 ship → template drift khỏi doctrine. **Fix gọn trong sos-kit, KHÔNG đụng repo `~/doctor`.**

### Giải pháp

1. Thêm field `**Lane:** Normal` (token TRẦN) vào header template, cạnh `**Tầng:**`, kèm chú thích phân biệt 2 trục.
2. Thêm contract test cơ học `scripts/lane-check-contract.sh` chạy `doctor lane-check` trên template — assert exit ≠ 2 (field present + parseable), degraded warn-skip khi `doctor` vắng.
3. Wire contract test vào pre-commit như **sub-check trong section [3/8] hiện có** (KHÔNG thêm phase [9/9] → tránh cascade phase-count).
4. Update docs Tầng-1.

### Scope

- CHỈ sửa: `phieu/TICKET_TEMPLATE.md`, tạo `scripts/lane-check-contract.sh`, sửa `hooks/pre-commit` (sub-check trong [3/8]), docs Tầng-1 (`phieu/README.md`, `docs/HANDOFF.md`, `CLAUDE.md`, `docs/SETUP.md`), `CHANGELOG.md`.
- KHÔNG sửa: repo `~/doctor` (bất kỳ file nào), `phieu/phieu.sh`, các phiếu cũ trong `docs/ticket/` (P050/P053/... predate field — không backfill).

---

## Task 0 — Verification Anchors

> Architect docs-only (no grep). Anchor code-level mang marker humility; Thợ verify trước khi code.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Orchestrator bắt `doctor lane-check --ticket` pre-CHALLENGE, exit 2 = missing lane field | `grep -n "lane-check" agents/orchestrator.md` (Architect đọc: block "Lane budget pre-CHALLENGE gate", ~dòng 52-63) `[verified]` | ⏳ TO VERIFY |
| 2 | `phieu/TICKET_TEMPLATE.md` header có `**Tầng:**` (dòng 12) nhưng KHÔNG có `**Lane:**` | `grep -nE "\*\*(Tầng\|Lane):\*\*" phieu/TICKET_TEMPLATE.md` → chỉ ra Tầng, 0 hit Lane `[verified]` | ⏳ TO VERIFY |
| 3 | `phieu.sh` copy template VERBATIM vào phiếu mới, chỉ rewrite dòng 1 (title) → field Lane sẽ propagate | `sed -n '116,124p' phieu/phieu.sh` (`cp "$template"` + `sed "1s|.*|# PHIẾU..."`) `[verified]` | ⏳ TO VERIFY |
| 4 | `doctor lane-check` parse regex `\*\*Lane:\*\*\s*(Normal\|Guarded\|Fast)`; exit 0=OK, 1=budget exceeded, 2=missing/unparseable field | `doctor lane-check --ticket phieu/TICKET_TEMPLATE.md; echo $?` TRƯỚC fix → kỳ vọng `2`. Source `~/doctor` OFF-LIMITS `[needs Worker verify]` | ⏳ TO VERIFY |
| 5 | Template ~181 dòng + 3 example anchor rows (Task 0 table) + 2 example constraints → dưới Normal budget (250/5/5) | `wc -l phieu/TICKET_TEMPLATE.md`; đếm data-row trong "Task 0" table + "Luật chơi". Doctor đếm CÁCH NÀO (incl header/sep row?) `[needs Worker verify]` | ⏳ TO VERIFY |

**Anchor #4 + #5 là điểm cần Worker verify bằng oracle thật** — `[oracle: doctor lane-check]` (SOUND cho claim "field parseable" + "budget count"). Worker CHẠY oracle, KHÔNG suy luận trên giấy (audit OA-01 điểm 5: "Acceptance phải kiểm exit thật, không chỉ unit-test fixture").

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge
*(Worker fills khi CHALLENGE. No objections → "Worker accepted V1 — no challenges.")*

**Anchor verification (recap Task 0):**
- Anchor #N: ✅/⚠️/❌ + 1-line nếu ⚠️/❌

**Objections (Tầng 1 only):**
- [O1.1] …

**Điểm Architect CHỦ ĐỘNG mời CHALLENGE** (2 decision có trade-off thật):
- **D1 — Lane value của template:** chốt `Normal` token trần (§Task 1). Worker verify anchor #4/#5: nếu template khai Normal mà lane-check trả **exit 1** (example rows/line-count trip budget), fallback pre-authorized ở Task 1 Lưu ý. Worker có thể đề xuất Guarded — nhưng Architect DEFEND: Guarded born-default = gate no-cap mọi phiếu = defeat mục đích §1 (xem Constraint 3).
- **D2 — Placement contract test:** chốt sub-check trong pre-commit [3/8] (KHÔNG phase [9/9] mới). Worker có thể đề xuất CI-only / standalone. Architect lean [3/8] vì: enforce-via-mechanism (commit = physical action) + tránh phase-count cascade (CLAUDE.md hook-chain row).

**Status:** ⏳ AWAITING WORKER CHALLENGE

### Turn 1 — Architect Response
*(RESPOND mode — Architect relies on Worker `file:line`.)*
- [O1.1] → ACCEPT / DEFEND / REFRAME / DEFER → action

**Status:** (chưa mở)

### Final consensus
- Phiếu version: V<N> · Total turns: <count> · Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1: Thêm field `**Lane:**` vào header template

**File:** `phieu/TICKET_TEMPLATE.md`

**Tìm:** dòng header `> **Tầng:** 1 (móng — ...) | 2 (lặt vặt — ...). Định nghĩa: docs/LAYERS.md §2-tier. LOC KHÔNG quyết Tầng.` (dòng ~12) `[needs Worker verify]` — grep `\*\*Tầng:\*\*` để định vị chính xác.

**Thay bằng / Thêm:** giữ nguyên dòng `Tầng`, thêm NGAY DƯỚI nó một dòng blockquote mới:

```
> **Lane:** Normal — trục NGÂN SÁCH (khác Tầng = trục hậu quả). Normal ≤250 dòng/≤5 anchor/≤5 constraint · Guarded no-cap · Fast ≤100 dòng (no-architect). Def: `docs/WORKFLOW_V2.2.md` §1. Architect override token per-phiếu. **Token PHẢI trần `Normal`|`Guarded`|`Fast`** — KHÔNG angle-bracket/placeholder (`doctor` regex `**Lane:** (Normal|Guarded|Fast)` fail parse → exit 2). Tầng=debate flow, Lane=size cap — 2 trục độc lập.
```

**Lưu ý:**
- Token `Normal` đứng NGAY sau `**Lane:** ` (khoảng trắng) để regex `\*\*Lane:\*\*\s*(Normal|Guarded|Fast)` match. Trailing text sau token OK (regex không anchor cuối dòng) — nhưng Worker CONFIRM anchor #4 rằng doctor dùng search (không full-line-anchored). Nếu doctor đòi token SẠCH (full-line) → tách chú thích xuống dòng riêng, để dòng `Lane` chỉ có `> **Lane:** Normal`.
- **Default = `Normal` (KHÔNG Guarded):** vì `phieu.sh` copy verbatim (anchor #3) → mọi phiếu mới born `Normal` = budget enforced by default. Phiếu lớn quên bump → lane-check exit 1 block pre-CHALLENGE → ép Architect khai Guarded có ý thức (fail-toward-enforcement). Guarded born-default = no-cap mọi phiếu = defeat gate.
- **Fallback nếu anchor #5 cho exit 1** (template Normal vượt budget do doctor đếm example rows): (F1) giảm example anchor rows trong "Task 0" table từ 3 → 2 để về dưới cap; nếu vẫn exit 1 do line-count → (F2) escalate Chủ nhà (trim template HAY chấp nhận assertion "exit ≠ 2"). Ghi count breakdown vào Discovery.

### Task 2: Tạo contract test `scripts/lane-check-contract.sh`

**File:** `scripts/lane-check-contract.sh` (mới, `chmod +x`)

**Nội dung (spec — Worker viết bash, khớp pattern shim `scripts/block-unsafe-merge.sh` degraded mode):**
- Nếu `doctor` binary KHÔNG có trên PATH (`command -v doctor`) → in `⚠️ doctor absent — lane contract SKIPPED` + `exit 0` (warn-skip, KHÔNG hard-fail — fresh-env friendly).
- Nếu có: chạy `doctor lane-check --ticket phieu/TICKET_TEMPLATE.md`; capture exit.
  - exit `2` → **FAIL LOUD** (`❌ template missing/unparseable Lane field — OA-01 regression`), `exit 1`.
  - exit `1` → WARN (`⚠️ template over Normal budget`) — không block (budget drift ≠ schema drift), in count.
  - exit `0` → `✅ template Lane field parseable + within budget`.
- Optional arg `$1` = path phiếu thật → check thêm (dùng ở acceptance với P082 chính nó).

**Lưu ý:** script phải chạy được từ repo root. KHÔNG hardcode path tuyệt đối. Exit-code contract phải khớp assertion ở Nghiệm thu.

### Task 3: Wire contract test vào pre-commit [3/8]

**File:** `hooks/pre-commit`

**Tìm:** cuối section `[3/8] sos-kit v2 checks` — sau block `3e. Worker commit pattern` (dòng ~186-200), TRƯỚC `echo ""` đóng section 3 `[needs Worker verify]`.

**Thay bằng / Thêm:** thêm sub-check `3f`:
```bash
# 3f. Lane-field contract (OA-01 regression guard — only when template staged)
TEMPLATE_STAGED=$(git diff --cached --name-only --diff-filter=AM | grep -E '^phieu/TICKET_TEMPLATE\.md$' || true)
if [ -n "$TEMPLATE_STAGED" ] && [ -f "scripts/lane-check-contract.sh" ]; then
    if bash scripts/lane-check-contract.sh; then
        green "  ✅ Lane-field contract OK"
    else
        red "  ❌ Lane-field contract failed (OA-01 — template lost Lane field)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
fi
```

**Lưu ý:**
- Đây là **sub-check TRONG [3/8]**, KHÔNG phải phase mới → phase count VẪN `[8/8]`, KHÔNG cascade phase-count doc updates. Nếu Worker CHALLENGE và Chủ nhà quyết đổi sang phase [9/9] riêng → PHẢI update mọi `[N/8]→[N/9]` label + `# Runs in order` header + CLAUDE.md "Hook chain" + docs/SETUP.md (per CLAUDE.md mapping row) — cascade nặng, lý do lean sub-check.
- Chỉ chạy khi template staged (drift trigger) → proportionate, không thêm wall-time mỗi commit.

### Task 4: Docs Gate Tầng-1

**File 4a:** `phieu/README.md` — thêm mô tả field `Lane` (cạnh chỗ nói về header/format phiếu, HOẶC section "Naming convention"/gotchas) `[needs Worker verify]` chỗ chính xác. Nội dung: Lane = trục ngân sách, token trần Normal|Guarded|Fast, doctor lane-check parse.

**File 4b:** `docs/HANDOFF.md` Handoff 2 — dòng liệt kê header fields hiện là `- Header (Loại, Ưu tiên, Ảnh hưởng, Dependency)` (dòng ~73) `[verified]`. Thay bằng: `- Header (Loại, Ưu tiên, **Tầng, Lane**, Ảnh hưởng, Dependency)` — bổ sung CẢ Tầng (cũng đang thiếu) lẫn Lane cho khớp template thật.

**File 4c:** `CLAUDE.md` — scripts list (repo structure block) thêm dòng `scripts/lane-check-contract.sh` (mô tả: OA-01 lane-field contract, degraded warn-skip). Và "Hook chain"/pre-commit mention: [3/8] nay gồm sub-check `3f` lane-contract (KHÔNG đổi phase count).

**File 4d:** `docs/SETUP.md` — hook/scripts section: thêm `lane-check-contract.sh` vào inventory `[needs Worker verify]` section chính xác.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `phieu/TICKET_TEMPLATE.md` | Task 1: thêm `**Lane:** Normal` + chú thích 2-trục |
| `scripts/lane-check-contract.sh` | Task 2: contract test mới (degraded warn-skip) |
| `hooks/pre-commit` | Task 3: sub-check `3f` trong [3/8] (KHÔNG đổi phase count) |
| `phieu/README.md` | Task 4a: mô tả field Lane |
| `docs/HANDOFF.md` | Task 4b: Handoff 2 header field list += Tầng, Lane |
| `CLAUDE.md` | Task 4c: scripts list + hook-chain mention |
| `docs/SETUP.md` | Task 4d: scripts inventory += lane-check-contract |
| `CHANGELOG.md` | entry P082 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `phieu/phieu.sh` | Copy verbatim (chỉ rewrite dòng 1) — Lane field propagate đúng vào phiếu mới. KHÔNG sửa. |
| `agents/orchestrator.md` | Lane-check pre-CHALLENGE gate call vẫn đúng sau khi template có field. KHÔNG sửa. |
| `~/doctor/**` | OFF-LIMITS tuyệt đối — fix nằm hoàn toàn trong sos-kit. |
| `docs/ticket/P050..P075` | Phiếu cũ predate field — KHÔNG backfill. |

---

## Luật chơi (Constraints)

1. **KHÔNG đụng repo `~/doctor`** (bất kỳ file/source nào). Fix hoàn toàn trong sos-kit.
2. **Lane token TRẦN `Normal`|`Guarded`|`Fast`** — KHÔNG angle-bracket/placeholder (doctor regex fail parse → exit 2).
3. **Template default = `Normal`**, KHÔNG Guarded (born-default phải enforce budget; Guarded = defeat gate).
4. **Contract test = sub-check trong pre-commit [3/8]**, KHÔNG thêm phase `[9/9]` (tránh phase-count cascade) — trừ khi Chủ nhà quyết khác ở CHALLENGE.
5. **`doctor` absent → warn-skip `exit 0`**, KHÔNG hard-fail (fresh-env friendly, khớp shim pattern).

---

## Nghiệm thu

### Automated
- [ ] `bash -n scripts/lane-check-contract.sh` + `bash -n hooks/pre-commit` clean.
- [ ] `doctor lane-check --ticket phieu/TICKET_TEMPLATE.md; echo $?` → **exit 0** (target: field parseable + within budget). MUST NOT = 2 (bug đang fix). Nếu = 1 → áp Task 1 fallback F1/F2, ghi count.
- [ ] `doctor lane-check --ticket docs/ticket/P082-lane-field-template.md; echo $?` → **exit 0** (phiếu thật, có Lane field — audit OA-01 "≥1 phiếu thật").
- [ ] `bash scripts/lane-check-contract.sh` (doctor present) → exit 0. Giả lập doctor absent (PATH tạm bỏ doctor) → warn-skip exit 0.
- [ ] Pre-commit: stage `phieu/TICKET_TEMPLATE.md` → sub-check `3f` chạy + PASS; phase count vẫn `[8/8]`.

### Manual Testing
- [ ] Tạo phiếu thử qua `phieu` (hoặc cp template thủ công) → phiếu mới có dòng `**Lane:** Normal` (verbatim propagate, anchor #3).
- [ ] Regression: xoá thử dòng Lane khỏi template + stage → `3f` FAIL LOUD (guard hoạt động).

### Regression
- [ ] Orchestrator lane budget gate (`agents/orchestrator.md`) giờ chạy được trên phiếu canonical (không còn exit 2 mù).
- [ ] Các section pre-commit [1/8]..[8/8] khác không đổi hành vi.

### Docs Gate (Tầng 1 — BẮT BUỘC, per CLAUDE.md mapping)
- [ ] `phieu/README.md` — mô tả field Lane (Task 4a).
- [ ] `docs/HANDOFF.md` Handoff 2 — header field list += Tầng, Lane (Task 4b).
- [ ] `CLAUDE.md` — scripts list += `lane-check-contract.sh` + hook-chain mention (Task 4c).
- [ ] `docs/SETUP.md` — scripts inventory (Task 4d).
- [ ] `CHANGELOG.md` — entry P082.
- [ ] Ghi rõ trong Discovery: "Tầng-1 docs updated: <list>" HOẶC lý do N/A từng file.

### Discovery Report
- [ ] Write `docs/discoveries/P082.md`:
  - Anchor #1-5: CORRECT/WRONG (file:line citations). Đặc biệt #4 (doctor exit codes) + #5 (budget count method) = oracle result thật.
  - Claim/Oracle/Soundness cho anchor #4, #5 (§2 3-field).
  - D1/D2 decision outcome (Lane value, contract placement) sau CHALLENGE.
  - Fallback F1/F2 có kích hoạt không + count breakdown template.
  - Tầng-1 docs updated list.
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
