# PHIẾU P086: Seed `.sos-trust-baseline` in born-wire — fix first-commit deadlock (dogfood L3)

> **Loại:** Bugfix
> **Ưu tiên:** P1
> **Tầng:** 1 (security-surface adjacency: trust-gate bootstrap + born-wire architecture — sai thì mọi repo mới sinh ra đều chết commit đầu)
> **Lane:** Normal
> **Ảnh hưởng:** `crates/sos-cli/src/commands/new.rs`, `crates/sos-cli/src/commands/adopt.rs` (+ parity goldens `crates/sos-cli/tests/golden/{new,adopt}*.golden`, bash oracle `bin/sos.sh` dormant fns)
> **Dependency:** None

---

## Context

### Vấn đề hiện tại

Linux dogfood 2026-07-24 (finding **L3, HIGH** — `docs/retro/DOGFOOD_LINUX_2026-07-24.md`):

Repo vừa `sos new` xong **không thể tạo commit đầu tiên**. `hooks/pre-commit` phase `[8/8]` (trust gate) fail-CLOSED khi `.sos-trust-baseline` không tồn tại:

```
BLOCKED: trust-gate: .sos-trust-baseline not found.
```

`sos new` arm hooks trong born-wire nhưng KHÔNG seed baseline, và next-steps (`new.rs:489` — "Next: fill # TODO … → git add -A && git commit.") không nhắc `scripts/trust-gate.sh rebaseline`. User mới đi đúng hướng dẫn → đâm tường, không có gợi ý thoát ngoài message của trust-gate.

**`sos adopt` dính y hệt** (phát hiện khi rà anchor, chưa reproduce bằng commit thật — Worker verify ở Nghiệm thu): adopt copy nguyên hook `[8/8]` + `scripts/trust-gate.sh` rồi arm `core.hooksPath` (khi F09 cho phép), nhưng không seed baseline. "Heads-up for your FIRST commit" của adopt nhắc docs-gate + BACKLOG mà không nhắc trust baseline.

(Đường `sos install` KHÔNG dính — P078i render backstop hook 2-invariant tối giản, không có trust-gate phase.)

### Giải pháp

Seed baseline ngay trong bước born-wire, theo đúng thứ tự mà enumerator của trust-gate yêu cầu (`git ls-files` chỉ thấy file đã staged/tracked — gotcha đã document tại `trust-gate.sh:17-19`):

1. `git add -A` (mọi thứ trong repo lúc đó đều là spine mà chính `sos new`/`adopt` vừa ghi — stage an toàn; với adopt: repo đã có commit sẵn, `git add` chỉ các file adopt vừa ADDED — xem Luật chơi #3)
2. shell ra `bash scripts/trust-gate.sh rebaseline` (script này nằm trong spine vừa copy)
3. `git add .sos-trust-baseline`
4. Đổi next-steps: "… → git commit." (bỏ `git add -A` vì born-wire đã stage)

Giữ nguyên nguyên tắc "no auto-commit — the bootstrap commit is the user's" (`new.rs:9`).

**Alternative đã cân nhắc (KHÔNG chọn):** tính baseline trực tiếp trong Rust (sha2 có sẵn trong workspace-deps) — bị loại vì phải duplicate `SURFACE_GLOBS` + format của `trust-gate.sh` → drift risk giữa 2 nguồn chân lý; trust-gate.sh là single source.

### Scope

- CHỈ sửa: `crates/sos-cli/src/commands/new.rs`, `crates/sos-cli/src/commands/adopt.rs`, goldens new/adopt tương ứng, `bin/sos.sh` (dormant `sos_new`/`sos_adopt` — giữ parity oracle đồng bộ), CHANGELOG.
- KHÔNG sửa: `scripts/trust-gate.sh` (không đổi semantics/format), `hooks/pre-commit` `[8/8]` (fail-closed là ĐÚNG — chính nó bắt được bug này), `crates/sos-install/*` (đường install không dính).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Born-wire của new: git init + arm hooks, no auto-commit, KHÔNG có `git add` nào | `grep -n "git add\|symbolic-ref\|Next:" crates/sos-cli/src/commands/new.rs` | ✅ :9 comment no-auto-commit; :475 symbolic-ref; :489 Next-line; grep `git add` chỉ hit chuỗi trong println :489 |
| 2 | trust-gate enumerator = `git ls-files` (untracked bị bỏ qua im lặng) → rebaseline TRƯỚC `git add` sẽ ghi baseline RỖNG | `grep -n "ls-files\|SURFACE_GLOBS" scripts/trust-gate.sh` | ✅ :151 `git ls-files ${SURFACE_GLOBS[@]}`; :17-19 gotcha documented; :163-174 untracked warning |
| 3 | Hook `[8/8]` fail-CLOSED khi baseline vắng | Linux dogfood transcript 4a (round 1) | ✅ `BLOCKED: trust-gate: .sos-trust-baseline not found` — commit chặn |
| 4 | Workaround thủ công đã verify chạy được: rebaseline (sau khi có staged files) + add → cả 8 phase xanh | Linux dogfood transcript 4g/4a2 | ✅ baseline 20 surfaces → first commit PASS |
| 5 | adopt arm hooks + copy trust-gate.sh nhưng không seed baseline | Worker grep `adopt.rs` (arm-hooks call site + không có rebaseline) | ⏳ WORKER VERIFY — nếu adopt.rs cấu trúc khác giả định, ghi vào Debate Log |
| 6 | Parity goldens new/adopt sẽ đổi stdout (thêm dòng seed + đổi Next-line) | `crates/sos-cli/tests/golden/{new,adopt}*.golden` + `capture.sh` | ⏳ WORKER VERIFY — golden phải regenerate qua capture.sh từ bash oracle đã sửa đồng bộ |

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

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge
*(pending)*

### Final consensus
- Phiếu version: —
- Approved by Chủ nhà: —

---

## Nhiệm vụ

### Task 1: `new.rs` — seed baseline trong born-wire

**File:** `crates/sos-cli/src/commands/new.rs`

Sau bước arm hooks (quanh `:475` symbolic-ref + arm), TRƯỚC println Next-line:
1. `git add -A` (cwd = target repo)
2. run `bash scripts/trust-gate.sh rebaseline` (cwd = target repo); nếu script vắng/lỗi → in `⚠` warn + hướng dẫn chạy tay, KHÔNG fail cả sos new (bootstrap phải hoàn thành — trust-gate hook sẽ fail-closed ở commit, không mất an toàn)
3. `git add .sos-trust-baseline`
4. In 1 dòng xác nhận kiểu `✓ trust baseline seeded (N surfaces) + spine staged`

**Task 2:** Sửa Next-line `:489` → "Next: fill # TODO (…) → git commit." (spine đã staged).

**Task 3:** `adopt.rs` — tương tự sau leg arm-hooks, nhưng CHỈ khi arm thật sự xảy ra (F09 decline → không seed, không stage — repo người ta, đừng đụng staging khi mình không arm). `git add` giới hạn: các path adopt vừa ADDED + `.sos-trust-baseline` (KHÔNG `add -A` trên brownfield — xem Luật chơi #3).

**Task 4:** Sửa dormant `sos_new`/`sos_adopt` trong `bin/sos.sh` y hệt (bash oracle cho parity), regenerate goldens qua `crates/sos-cli/tests/golden/capture.sh`.

**Task 5:** Adopt "Heads-up for your FIRST commit" thêm dòng baseline (chỉ nhánh F09-decline cần nhắc chạy tay sau khi arm bằng install-hooks.sh).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-cli/src/commands/new.rs` | Task 1+2: seed baseline + stage + Next-line |
| `crates/sos-cli/src/commands/adopt.rs` | Task 3+5: seed khi arm, scoped staging, heads-up |
| `bin/sos.sh` (dormant fns) | Task 4: đồng bộ bash oracle |
| `crates/sos-cli/tests/golden/*` | Task 4: regenerate |
| `CHANGELOG.md` | entry |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/trust-gate.sh` | rebaseline vẫn là single source; format baseline không đổi |
| `hooks/pre-commit` | `[8/8]` fail-closed giữ nguyên |
| `crates/sos-install/**` | backstop hook không trust-gate — không đụng |

---

## Luật chơi (Constraints)

1. KHÔNG auto-commit — bootstrap commit vẫn của user (`new.rs:9` doctrine giữ nguyên).
2. Thứ tự BẮT BUỘC: stage → rebaseline → stage baseline (đảo = baseline rỗng, deadlock quay lại — anchor #2).
3. Adopt: KHÔNG `git add -A` — chỉ stage file adopt vừa tạo + baseline. Brownfield có thể có work-in-flight của user (bài học `sos sync` dirty-warn, incident inv-gate).
4. `trust-gate.sh` lỗi/vắng khi seed → warn + degrade, không chặn bootstrap.
5. Parity goldens regenerate qua `capture.sh` từ bash oracle — không sửa golden bằng tay.

---

## Nghiệm thu

### Automated
- [ ] `cargo check --workspace` clean
- [ ] `cargo test --workspace --no-fail-fast` — parity new/adopt xanh với goldens mới
- [ ] Test mới: fixture `sos new` xong → real `git commit` đầu PASS cả 8 phase KHÔNG cần thao tác tay (pristine, zero-seed — theo bài học oracle P078i)
- [ ] Test mới: adopt vào brownfield có file un-staged của user → file đó KHÔNG bị stage

### Manual Testing
- [ ] Linux: `sos new /tmp/t --stack python && cd /tmp/t && git commit -m x` → commit thành công ngay
- [ ] Adopt nhánh F09-decline: KHÔNG seed, heads-up nhắc baseline
- [ ] Reproduce adopt-first-commit deadlock TRƯỚC khi fix (xác nhận anchor #5 giả định đúng), rồi confirm fix đóng

### Regression
- [ ] `sos new` vẫn không auto-commit; `sos tools status`/verify-setup không đổi
- [ ] Xóa guard script sau new → commit vẫn BLOCKED (fail-closed [6/8]/[7/8]/[8/8] như dogfood 4e2/4f2)

### Docs Gate
- [ ] `CHANGELOG.md` — entry P086
- [ ] `INSTALL.md` — nếu có đoạn hướng dẫn first-commit thì sync Next-line mới

### Discovery Report
- [ ] `docs/discoveries/P086.md` + 1-line index vào `docs/DISCOVERIES.md`
