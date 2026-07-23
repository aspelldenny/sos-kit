# PHIẾU P080x: Dev pre-commit hook fail-OPEN parity — port P078i fail-CLOSED cho 2 security invariant

---

> **Loại:** Bugfix (security surface)
> **Ưu tiên:** P1 (HIGH — bảo mật fail-OPEN, chứng minh live)
> **Tầng:** 1 — sửa `hooks/pre-commit` (security backstop). Sai thì secret/product-code lọt commit → KHÔNG-đảo (leak). AUTO Tầng 1.
> **Lane:** Guarded — security surface, Debate đầy đủ.
> **Ảnh hưởng:** `hooks/pre-commit` (2 phase) + CLAUDE.md "Hook chain" + `docs/SETUP.md` (nếu hành vi đổi).
> **Dependency:** P080 (round-1 dogfood phát hiện gap D1). P080 GIỮ gated tới khi phiếu này đóng + re-run xanh.

---

## Context

### Vấn đề hiện tại
P080 round-1 local FAIL với gap D1 (HIGH). Dev `[8/8]` hook `hooks/pre-commit` (render qua `sos new`/`adopt` copy_tree — Claude path) **fail-OPEN** khi guard script phụ absent: phase `[6/8]` no-code-on-default và `[7/8]` block-env in `⏭ ... missing — run scripts/install-hooks.sh` rồi **fall-through KHÔNG `exit`/`FAIL_COUNT++`** → commit lọt. Đây đúng bug P078i đã fix, nhưng fix chỉ nằm ở backstop-minimal hook phía **install-path (Codex)** (`crates/sos-install/src/templates/backstop-pre-commit.sh`), CHƯA port về dev hook gốc. Thợ chứng minh live: `sos new` fixture → xóa `scripts/block-env-commit.sh` → commit `.env` thật → **exit 0** (lẽ ra phải block).

### Giải pháp
Port ngữ nghĩa fail-CLOSED của P078i vào dev `hooks/pre-commit`, PHẠM VI TỐI THIỂU = 2 security invariant (`.env` block + no-code-on-default): guard script absent → `red` LOUD + `FAIL_COUNT++` → summary `exit 1`, KHÔNG `⏭ skip`. Các phase không-thuộc-2-invariant giữ degraded-warn như hiện tại (quyết định per-phase ở bảng dưới). **Phase count `[8/8]` KHÔNG đổi** — chỉ đổi NGỮ NGHĨA missing-script (fail-open → fail-closed) của 2 phase.

### Phase-by-phase decision (Kiến trúc sư đọc `hooks/pre-commit` quyết định)

| Phase | Loại | Missing-script hiện tại | Target | Lý do |
|---|---|---|---|---|
| [1/8] type-check | linter (không-security) | warn-skip | **giữ warn-skip** | linter, không phải security boundary (comment `:59-60` xác nhận chủ đích) |
| [2/8] docs-gate | doc hygiene | warn-skip | **giữ warn-skip** | fresh-install graceful (P006) |
| [3/8] v2 checks (BACKLOG/Discovery/Lane) | process hygiene | warn-skip | **giữ warn-skip** | không secret/code leak |
| [4/8] security-gate INV-009/010 | security surface | `⏭ skip` (fail-open) | **giữ warn-skip — DEFER** | security NHƯNG ngoài phạm vi 2-invariant của P078i backstop; ép fail-closed sẽ brick fresh `sos new` thiếu `install-hooks.sh`. Flag follow-up, KHÔNG mở rộng scope ở đây |
| [5/8] case-collision | CI correctness | `⏭ skip` | **giữ warn-skip** | vỡ-CI không phải secret-leak |
| **[6/8] no-code-on-default** | **SECURITY invariant #2** | `⏭ skip` (fail-open) | **→ fail-CLOSED exit 1** | thuộc 2-invariant P078i; product code lọt default = KHÔNG-đảo |
| **[7/8] block-env** | **SECURITY invariant #1** | `⏭ skip` (fail-open) | **→ fail-CLOSED exit 1** | thuộc 2-invariant P078i; secret leak = KHÔNG-đảo |
| [8/8] trust-gate | security integrity | `⏭ skip` (fail-open) | **giữ warn-skip — DEFER** | như [4/8]: security surface nhưng ngoài 2-invariant; deferred cùng follow-up |

> **Scope quyết định:** CHỈ [6/8]+[7/8] chuyển fail-CLOSED (khớp CHÍNH XÁC 2 guard mà P078i backstop co-render). [4/8]+[8/8] là security surface nhưng để tránh brick fresh-install thiếu `install-hooks.sh`, deferred như 1 follow-up có chủ đích (ghi Discovery) — KHÔNG mở rộng trong phiếu này.

### Scope
- CHỈ sửa `hooks/pre-commit` phase [6/8] + [7/8] (else-branch missing-script) + docs Tầng 1.
- KHÔNG đụng [1-5]/[8], KHÔNG đổi phase count, KHÔNG sửa `crates/**/src` (backstop-minimal hook install-path đã đúng từ P078i).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `[7/8]` block-env: guard absent → `echo "⏭ ... missing"` KHÔNG `FAIL_COUNT++`/`exit` (fail-OPEN) | đọc `hooks/pre-commit` | ✅ `[verified]` — `hooks/pre-commit:282-291`, else-branch chỉ `echo`, không tăng FAIL |
| 2 | `[6/8]` no-code-on-default: guard absent → `echo "⏭ ... missing"` fail-OPEN | đọc `hooks/pre-commit` | ✅ `[verified]` — `hooks/pre-commit:262-271`, else-branch chỉ `echo` |
| 3 | Summary: `FAIL_COUNT > 0` → `exit 1`; `FAIL_COUNT==0` → commit allowed | đọc `hooks/pre-commit` | ✅ `[verified]` — `hooks/pre-commit:319-328` |
| 4 | P078i backstop-minimal hook đã fail-CLOSED (missing guard → exit 1) — ngữ nghĩa cần port | `grep -n "missing\|exit 1" crates/sos-install/src/templates/backstop-pre-commit.sh` | ⏳ `[needs Worker verify]` — Architect bị architect-guard chặn đọc source (envelope đúng); per BACKLOG P078i + `docs/discoveries/P078i.md` là fail-CLOSED |
| 5 | Gap D1 live-proof (xóa guard → commit `.env` exit 0) ghi trong findings | đọc `docs/adapters/P080-FINDINGS-2026-07-23.md` | ⏳ `[on branch P080-dogfood-round1, needs Worker verify]` — file nằm trên branch đó, working tree hiện tại có thể chưa thấy |
| 6 | 2 guard co-render bởi backstop = `scripts/block-env-commit.sh` + `scripts/no-code-on-default.sh` (đối tượng fail-CLOSED khớp) | `grep -n "block-env-commit\|no-code-on-default" crates/sos-install/src/templates/backstop-pre-commit.sh` | ⏳ `[needs Worker verify]` — per BACKLOG P078i "co-render 2 guard verbatim" |
| 7 | CLAUDE.md "Hook chain" + DOCS GATE mapping ref phase count `[8/8]` (không đổi count → không đổi M) | `grep -n "8/8\|Hook chain\|pre-commit SECTION" CLAUDE.md docs/SETUP.md` | ⏳ `[needs Worker verify]` |

### Pre-phiếu snapshot
Theo TICKET_TEMPLATE (Worker auto first-step trong worktree).

---

## Debate Log

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge
**Worker accepted V1 — no blocking objections.**

Anchor verification:
- #1 ✅ `hooks/pre-commit:282-291` — `[7/8]` else-branch is bare `echo`, no `FAIL_COUNT++`/`exit` (fail-OPEN confirmed)
- #2 ✅ `hooks/pre-commit:262-271` — `[6/8]` else-branch same pattern (fail-OPEN confirmed)
- #3 ✅ `hooks/pre-commit:319-328` — summary: `FAIL_COUNT>0` → `exit 1`, else warn/pass
- #4 ✅ `crates/sos-install/src/templates/backstop-pre-commit.sh:25-36` — both guards: `[ ! -f ... ]` → `echo BLOCKED... >&2; exit 1` (fail-CLOSED confirmed, semantics to port: print + hard exit, not just FAIL_COUNT bump — dev hook uses FAIL_COUNT+summary-exit pattern instead, which is equivalent within the dev hook's own architecture, just a different mechanical implementation of the same fail-closed *outcome*)
- #5 N/A this session — historical P080 branch not needed for this fix (D1 already fully described in phiếu Context)
- #6 ✅ `crates/sos-install/src/templates/backstop-pre-commit.sh:25,32` — exactly `scripts/block-env-commit.sh` + `scripts/no-code-on-default.sh`, matching phase [7]/[6] targets
- #7 ✅ `CLAUDE.md:24,117,222` + `docs/SETUP.md:146,180` — phase count `[8/8]` referenced consistently, DOCS GATE mapping row present; edit only changes prose describing [6]/[7] semantics, not the count

Phase-decision table reviewed against real hook: no invariant missed, [4]/[8] defer rationale (avoid bricking fresh `sos new` missing `install-hooks.sh`) matches actual else-branch text at `hooks/pre-commit:230-232,309-311` (identical wording pattern to [6]/[7] before this fix — confirms these two ARE currently symmetric fail-open, correctly scoped to defer). No design flaw, no dangerous side-effect found. Proceeding to EXECUTE.

**Status:** ✅ APPROVED — Worker proceeding to EXECUTE (surgical fix, Sếp rule: skip extra approval gate)

### Final consensus
- Phiếu version: V<N>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1 — `[7/8]` block-env: fail-OPEN → fail-CLOSED

**File:** `hooks/pre-commit`

**Tìm:** (phase [7/8] else-branch, quanh `:289-291`)
```
else
    echo "  ⏭  scripts/block-env-commit.sh missing — run scripts/install-hooks.sh after bootstrap"
fi
```

**Thay bằng:**
```
else
    red "  ❌ scripts/block-env-commit.sh missing — cannot verify .env* not staged (fail-CLOSED)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi
```

**Lưu ý:** khớp ngữ nghĩa P078i (missing guard = block, không "missing → allowed"). `red` helper đã định nghĩa `:22`. KHÔNG đổi dòng `blue "[7/8]..."`.

### Task 2 — `[6/8]` no-code-on-default: fail-OPEN → fail-CLOSED

**File:** `hooks/pre-commit`

**Tìm:** (phase [6/8] else-branch, quanh `:269-271`)
```
else
    echo "  ⏭  scripts/no-code-on-default.sh missing — run scripts/install-hooks.sh after bootstrap"
fi
```

**Thay bằng:**
```
else
    red "  ❌ scripts/no-code-on-default.sh missing — cannot verify no product code on default (fail-CLOSED)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi
```

**Lưu ý:** đối xứng Task 1. KHÔNG đụng phase [4]/[5]/[8] (deferred, giữ `⏭ skip`).

### Task 3 — Cập nhật header comment (tùy chọn, giữ doc đồng bộ)

**File:** `hooks/pre-commit`

**Tìm:** block comment `# Runs in order:` (`:5-13`) — nếu Worker thấy tiện, thêm 1 dòng ghi rõ `[6]/[7]` fail-CLOSED khi guard absent; KHÔNG bắt buộc, KHÔNG đổi số thứ tự.

**Lưu ý:** đây là Tầng-2 cosmetic — Worker tự quyết, không block phiếu.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `hooks/pre-commit` | Task 1+2: phase [6/8]+[7/8] missing-guard else-branch → fail-CLOSED (`red`+`FAIL_COUNT++`) |
| `CLAUDE.md` | DOCS GATE: ghi rõ [6/7] fail-CLOSED semantics; **phase count [8/8] KHÔNG đổi** (chỉ ngữ nghĩa) |
| `docs/SETUP.md` | nếu mô tả hook behavior — cập nhật; nếu chỉ liệt kê phase count thì no-op (count không đổi) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-install/src/templates/backstop-pre-commit.sh` | install-path backstop ĐÃ fail-CLOSED (P078i) — phiếu này KHÔNG đụng; chỉ đối chiếu ngữ nghĩa để port |
| `hooks/pre-commit` phase [1-5],[8] | giữ nguyên hành vi (warn-skip / degraded) |

---

## Luật chơi (Constraints)

1. **CHỈ [6/8]+[7/8] chuyển fail-CLOSED** — khớp chính xác 2-invariant P078i backstop. [4]/[8] security-surface deferred có chủ đích (ghi Discovery), KHÔNG mở rộng scope.
2. **Phase count [8/8] KHÔNG đổi** — chỉ đổi ngữ nghĩa missing-script. KHÔNG add/remove section (nếu không sẽ trigger phase-count DOCS GATE khác — CLAUDE.md P062).
3. **Oracle pristine-no-seed BẮT BUỘC** (bài học 4 vòng structural-oracle-gap P078f/g/i): fixture `sos new` xong → XÓA 1 guard script → commit `.env` thật → phải BLOCK exit 1. KHÔNG chỉ grep nội dung hook.
4. KHÔNG sửa `crates/**/src`.

---

## Nghiệm thu

### Automated (oracle — pristine, KHÔNG seed)
- [ ] Fixture: `sos new` repo git riêng `/tmp` → `hooks/pre-commit` + 2 guard render.
- [ ] **D1 repro fix:** XÓA `scripts/block-env-commit.sh` → `git commit` file `.env` thật → **BỊ CHẶN exit 1** (round-1: exit 0). Output verbatim `❌ ... missing ... (fail-CLOSED)`.
- [ ] XÓA `scripts/no-code-on-default.sh` → commit product code trên default → **BỊ CHẶN exit 1**.
- [ ] Negative control: 2 guard CÓ mặt + commit sạch (không `.env`, feature branch) → **exit 0** (không false-block).
- [ ] Regression: 2 guard CÓ mặt + commit `.env` → vẫn BỊ CHẶN (hành vi cũ không vỡ).
- [ ] `bash -n hooks/pre-commit` (syntax) clean.

### Manual Testing
- [ ] Grep `hooks/pre-commit` phase [6]/[7] else-branch → `FAIL_COUNT` + `red`, KHÔNG còn `⏭ ... missing`.
- [ ] Phase [4]/[5]/[8] else-branch → còn `⏭ skip` (deferred đúng chủ đích).

### Regression
- [ ] Chạy full hook trên sos-kit repo (2 guard present) → không FAIL mới do thay đổi này.

### Docs Gate (Tầng 1 — sửa hooks/pre-commit)
- [ ] `CLAUDE.md` "Hook chain" / DOCS GATE mapping — ghi [6/7] fail-CLOSED; xác nhận phase count [8/8] KHÔNG đổi (chỉ ngữ nghĩa).
- [ ] `docs/SETUP.md` hook section — no-op nếu chỉ liệt kê count; cập nhật nếu mô tả behavior.
- [ ] `CHANGELOG.md` — entry P080x.

### Discovery Report
- [ ] `docs/discoveries/P080x.md` — anchor #1-7 CORRECT/WRONG (file:line), quyết định defer [4]/[8] (lý do + follow-up đề xuất), D1 repro before/after (exit 0 → exit 1), phase-count-unchanged xác nhận.
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
- [ ] Sau khi merge → P080 round-2 re-run D1 phải xanh (unblock P080 → P081).
