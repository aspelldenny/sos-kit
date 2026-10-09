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

**Phiếu version:** V2

### Turn 1 — Worker Challenge (phiếu V1)
**Anchor verification:** #1 ✅ · #5 ⚠️ (existence confirmed, but Task 3 plan under-specified) · #6 ✅ (flow confirmed, capture.sh scope note non-blocking)
**Objections:**
- [O1.1] `adopt.rs:228,246,298,310,547,555,563,585,596,604,630,637,656` — `added: Vec<String>` is a pre-formatted stdout DISPLAY list, not a raw-path list (mixes clean paths with trailing description text; `:656` "hooks ARMED (core.hooksPath → ...)" isn't a path at all). Task 3's "git add giới hạn: các path adopt vừa ADDED" cannot iterate `added` directly for `git add`.
  Claim: `added` cannot be used verbatim as `git add` args.
  Oracle: direct code read of adopt.rs push call sites — SOUND (exact source text).
  Soundness: SOUND for existence-of-gap; NOT sound for which fix to pick (design choice) → needs Architect respond.
  Verdict: needs Architect respond.
**Proposed alternatives:**
- A. (Worker lean) Thread a parallel `added_paths: Vec<PathBuf>` accumulator through adopt_item/adopt_skills/wire_mcp_json/wire_settings_local + inline pushes; Task 3 git-adds that list + `.sos-trust-baseline`. Mechanical, no string parsing.
- B. Parse `added` strings (strip "    + " prefix, split on " ("), skip non-path-looking entries. Cheaper diff, but silently drops special-case lines (e.g. hooks-armed) — maintenance trap.
- C. Diff `git status --porcelain --untracked-files=all` before/after the adopt body instead of tracking paths in Rust; avoids touching accumulators but couples correctness to git-status parsing, different mechanism than phiếu's stated approach.
**Status:** ✅ RESPONDED (below)

### Turn 1 — Architect Response
- [O1.1] → **ACCEPT — chọn phương án A** (parallel `added_paths: Vec<PathBuf>` accumulator). Lý do: B là string-parsing trap đúng như Worker chỉ (silently drops hooks-armed class); C đổi cơ chế sang parse git-status — thêm class rủi ro mới + lệch tinh thần "Rust tự bookkeeping". A mechanical, khớp pattern `&mut Vec<T>` sẵn có. **Task 3 respec:** mỗi call site ghi file thật push path tương ứng vào `added_paths` (side-effect không phải file — như hooks-armed `:656`, jq-merge `.mcp.json` merge-nhánh — vẫn push path file bị ghi nếu có ghi, bỏ qua nếu chỉ là git-config). Bash oracle dormant `sos_adopt` mirror bằng mảng `_added_paths`.
- Anchor #6 note → **Tầng 2, chốt tree-only:** KHÔNG thêm `.sos-trust-baseline` vào `NEW_GEN_FILES`/`ADOPT_GEN_FILES` (nội dung baseline chứa hash biến thiên theo nội dung spine — gen-hash golden sẽ giòn vô ích; tree-golden proof-of-existence là đủ). `capture.sh` KHÔNG thuộc "Files cần sửa" — chỉ chạy để regenerate.

**Status:** ✅ RESPONDED — phiếu bumped to V2

### Final consensus
- Phiếu version: V2
- Total turns: 1
- Approved by Chủ nhà: 2026-07-24 — standing approval ("em làm full flow đi") ghi nhận từ chat; execute may begin.

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

**Task 3 (V2 — per O1.1/phương án A):** `adopt.rs` — tương tự sau leg arm-hooks, nhưng CHỈ khi arm thật sự xảy ra (F09 decline → không seed, không stage — repo người ta, đừng đụng staging khi mình không arm). Cơ chế path: thread accumulator `added_paths: Vec<PathBuf>` song song `added` qua `adopt_item`/`adopt_skills`/`wire_mcp_json`/`wire_settings_local` + inline pushes (mỗi call site GHI FILE thật push path; side-effect không-file như hooks-armed thì bỏ qua). `git add` = `added_paths` + `.sos-trust-baseline` (KHÔNG `add -A` trên brownfield — Luật chơi #3). Bash oracle mirror bằng mảng `_added_paths`.

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
