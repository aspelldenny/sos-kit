# PHIẾU P087: Windows runtime bugs — `sos map` scanner dead + `trust-gate.sh` false-BLOCK

> **ID format:** `P087` — assigned manually (Windows bug-fix wave, Sếp direct approval — override Architect Rule 0; Active sprint = runtime-portability).
> **Filename:** `docs/ticket/P087-windows-runtime-fixes.md`
> **Branch:** `fix/P087-windows-runtime-fixes`

---

> **Loại:** Bugfix (cross-platform runtime + security surface)
> **Ưu tiên:** P1 (Windows dogfood 2026-07-24: 6/51 test fail, cả 6 trong `crates/sos-cli/tests/parity.rs`)
> **Tầng:** 1 — `scripts/trust-gate.sh` là security gate (auto-exec baseline surface). Security-surface touch → AUTO Tầng 1 dù diff nhỏ (per `docs/LAYERS.md` §2-tier + CLAUDE.md DOCS-GATE). `map.rs` scanner sai lặng trên Windows đổi output contract (AGENT_MAP surfaces) → LAN sang mọi Windows adopt.
> **Lane:** Guarded — budget axis no-cap (security surface + 2 bug độc lập, ~8 anchor > Normal ≤5-anchor cap). Def: `docs/WORKFLOW_V2.2.md` §1.
> **Ảnh hưởng:** `crates/sos-cli/src/commands/map.rs`, `scripts/trust-gate.sh`, `.sos-trust-baseline` (rebaseline), `SECURITY.md` (nếu baseline FORMAT đổi), CHANGELOG.
> **Dependency:** None để khởi công. **Sequence note:** full `[8/8]` xanh trên Windows có thể còn đợi P088 (`.gitattributes` CRLF fix) — P087 chỉ đóng class `*`-prefix format mismatch của trust-gate, KHÔNG đóng CRLF-hash mismatch (đó là P088). 3 test CRLF-only (`parity_sync/new/adopt_enforced`) nằm ngoài scope P087.

---

## Context

### Vấn đề hiện tại

Windows dogfood 2026-07-24 (Quản đốc chạy `cargo test --workspace` trên Windows 11): linux + macOS đều xanh, Windows **45 pass / 6 fail — cả 6 trong `crates/sos-cli/tests/parity.rs`**. P087 xử 2 trong số đó (BUG 1 + BUG 2); 3 fail CRLF-only để P088.

**BUG 1 — `sos map` scanner chết trên Windows.** `SURFACES[].path_substrs` là substring POSIX-style (`"/routes/"`, `"/handlers/"`, `"/views/"`, `"/controllers/"`, `"/api/"`, `"/models/"`, `"/entities/"`, `"/services/"`, `"/lib/"`) được match qua `path_str.contains(s)` với `path_str = path.to_string_lossy()` = **native separator** (`\` trên Windows) → không bao giờ khớp. Cùng class:
- `NOISE_EXCLUDE` (`"/.git/"`, `"/node_modules/"`, `"/__pycache__/"`, `"/.sos-adopt-incoming/"`, `"/migrations/versions/"`, `"/.venv/"`, `"/venv/"`, `"/dist/"`, `"/build/"`) check trong `is_noise()` cũng so native string → noise KHÔNG bị loại trên Windows (`.git`/`node_modules` bị scan).
- `detect_present_stacks` gọi `is_noise` trên native path — cùng lỗ.
- Output path ĐÃ normalize `\`→`/` (`.replace('\\', "/")`) tại các dòng emit — nên fix = áp **cùng** normalization cho `path_str` TRƯỚC khi match (một class fix, ~3 call site).
- Phụ: dòng stdout xác nhận in `out.display()` → backslash trên Windows; parity golden mong `/` (`<TARGET>/docs/AGENT_MAP.yaml`; failure text hiện `<TARGET>\docs\AGENT_MAP.yaml`).

**BUG 2 — `scripts/trust-gate.sh` false-BLOCK trên Windows (Git Bash).** Test `new_first_commit_passes_all_hooks_zero_seed` fail: first commit sau `sos new` bị chặn ở hook `[8/8]`; mỗi file auto-exec liệt kê HAI lần — một lần plain (`scripts/foo.sh`), một lần có asterisk (`*scripts/foo.sh`).
- Gốc: GNU coreutils `sha256sum` trên Windows/Git Bash phát **binary-mode** format `HASH *PATH` (asterisk trước path), trong khi baseline `.sos-trust-baseline` (seed trên POSIX) là text-mode `HASH  PATH`. Compare path: `diff baseline tmp_current | grep '^[<>]' | awk '{print $NF}'` → mọi dòng khác → BLOCK dù nội dung y hệt. `awk $NF` mang cả `*` theo → doubled listing.
- Fix hướng: normalize hash-line format ở CẢ generation lẫn comparison (strip `*` đầu path field / emit `printf '%s  %s'` từ field đã parse) → baseline format platform-invariant. PHẢI giữ POSIX-sh/bash compat (chạy Git Bash + macOS `shasum` fallback).
- Phụ (NOTE, scope-check): file checkout CRLF trên Windows hash khác nội dung LF-committed — phần đó là `.gitattributes` của P088. P087 CHỈ fix format mismatch. Acceptance P087 = class `*`-prefix biến mất; full `[8/8]` xanh có thể CÒN cần P088 merge (sequence note).

### Giải pháp

1. **BUG 1:** áp `.replace('\\', "/")` cho `path_str` (chuỗi đưa vào `.contains()`) trước mọi match — trong `is_noise()`, trong loop surface-match, và ở call site `detect_present_stacks`. Normalize `out.display()` output sang `/` cho dòng stdout xác nhận (parity-golden dùng `/`).
2. **BUG 2:** normalize hash-line format (strip leading `*` khỏi path field) ở cả nhánh generate baseline lẫn nhánh compare current — POSIX-sh/bash safe, giữ nhánh macOS `shasum` fallback.
3. **Rebaseline:** edit `trust-gate.sh` sẽ tự trip chính trust-gate baseline → task PHẢI kèm bước `scripts/trust-gate.sh rebaseline` (hoặc lệnh rebaseline tương đương) + `git add .sos-trust-baseline`.

### Scope
- CHỈ sửa `crates/sos-cli/src/commands/map.rs` (path normalization + display normalization) + `scripts/trust-gate.sh` (hash-line format) + rebaseline `.sos-trust-baseline` + docs Tầng 1.
- KHÔNG đụng `.gitattributes` (P088), KHÔNG đụng CRLF/EOL layer (P088), KHÔNG đụng `.claude/skills/*` symlink (P088), KHÔNG đổi phase count hook.

---

## Task 0 — Verification Anchors

> Architect bị architect-guard chặn đọc `crates/**/src` + không có Bash/Grep → mọi file:line dưới đây từ Quản đốc insight briefing (verified từ test output + source read). Marker phản ánh: Architect chưa tự Read → `[unverified]`; số dòng dễ drift → `[needs Worker verify]`. Worker grep-xác nhận trước khi Edit.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `path_str = path.to_string_lossy()` (native sep) feed vào surface-match `path_str.contains(s)` | `grep -n "to_string_lossy\|\.contains(" crates/sos-cli/src/commands/map.rs` | ✅ `[verified: map.rs:213 (derive), :257-258 (match)]` — exact match briefing |
| 2 | `SURFACES[].path_substrs` là POSIX substring (`"/routes/"`…`"/lib/"`) | `grep -n "path_substrs\|/routes/\|/lib/" crates/sos-cli/src/commands/map.rs` | ✅ `[verified: map.rs:120,128,136 — routes_handlers/models_schema/services_logic]` |
| 3 | `NOISE_EXCLUDE` POSIX substring + `is_noise()` so native `path_str` | `grep -n "NOISE_EXCLUDE\|fn is_noise" crates/sos-cli/src/commands/map.rs` | ✅ `[verified: NOISE_EXCLUDE const :171, fn is_noise(path_str: &str) :183-184]` — note: `is_noise` takes `&str` (already-stringified), not `&Path` — normalize must happen at each CALLER before passing in, or by wrapping the arg inline; there is no single interior-signature change possible without changing the `&str` param itself |
| 4 | `detect_present_stacks` gọi `is_noise` trên native path (cùng lỗ) | `grep -n "fn detect_present_stacks\|is_noise" crates/sos-cli/src/commands/map.rs` | ✅ `[verified: fn detect_present_stacks :68, is_noise(&path_str) call :83]` (briefing said :82-83, fn body starts :68) |
| 5 | Output path đã normalize qua `.replace('\\', "/")` (pattern có sẵn để tái dùng) | `grep -n "replace('\\\\\\\\'" crates/sos-cli/src/commands/map.rs` | ✅ `[verified: map.rs:234,252,272 exact]` |
| 6 | Dòng stdout xác nhận in `out.display()` → backslash trên Windows | `grep -n "out.display()\|\.display()" crates/sos-cli/src/commands/map.rs` | ✅ `[verified: map.rs:357 (unmapped-stub branch), :371 (mapped branch); briefing said :355-358/:368-372 — off by ~3, same statements]` |
| 7 | trust-gate generate baseline: `... | sort -k2 > tmp_baseline`; compare: `diff ... | grep '^[<>]' | awk '{print $NF}'` | `grep -n "sha256sum\|shasum\|sort -k2\|awk '{print \$NF}'" scripts/trust-gate.sh` | ✅ `[verified: generate hash_file() call :190 → pipe sort -k2 :194; compare hash_file() call :227 → pipe sort -k2 :233; diff+awk NF :237]` — **1 additional read site briefing missed:** line 242 `baseline_paths=$(awk '{print $NF}' "${BASELINE_FILE}")` also NF-extracts from the raw baseline file for added/removed-surface detection (comm -13/-23). See O1.1 below — self-closed, not a blocker. |
| 8 | macOS `shasum` fallback branch tồn tại (fix phải giữ) | `grep -n "shasum" scripts/trust-gate.sh` | ✅ `[verified: trust-gate.sh:78-80 (resolver), :93 (hash_file wrapper branch)]` — shasum default text mode has no `*`-prefix quirk (that's GNU-coreutils-on-Windows-specific default-binary-mode behavior); strip-`*` fix is a no-op on macOS shasum output, confirmed safe |
| 9 | Failing tests: `parity_map_enforced`, `oa02_templates_excluded_from_frontend`, `new_first_commit_passes_all_hooks_zero_seed` | `grep -n "fn parity_map_enforced\|fn oa02_templates_excluded\|fn new_first_commit_passes" crates/sos-cli/tests/parity.rs` | ⚠️ `[verified: actual lines parity.rs:713, :838, :892 — briefing said :724/:862/:910, drifted ~11-18 lines. Test names correct, only line numbers wrong.]` |
| 10 | trust-gate edit tự trip baseline → cần rebaseline lệnh + `git add .sos-trust-baseline` | đọc `scripts/trust-gate.sh` header/usage cho lệnh rebaseline chính xác | ✅ `[verified: subcommand is literally "rebaseline" → cmd_rebaseline() at trust-gate.sh:157; new.rs:499/508 + adopt.rs already invoke "bash scripts/trust-gate.sh rebaseline" this exact way]` |

**❌/⏳ ghi nhận:** mọi anchor `[unverified]` vì Architect envelope chặn source-read — đây là "đá bóng cho Thợ" ĐÚNG mực. Nếu Worker grep thấy lệch (số dòng / tên hàm / danh sách substring) → sửa tại chỗ theo reality + ghi Discovery, KHÔNG cần Architect RESPOND cho lệch số-dòng thuần (oracle: `cargo test` + grep phán được).

### Pre-phiếu snapshot (Worker auto first-step)

> Worker EXECUTE FIRST ACTION (trước mọi edit): rollback point.

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Worker accepted V1 — no Tầng 1 objections.** Anchor verification: 10/10 confirmed real (file:line). Anchor #9 had drifted line numbers (briefing off by ~11-18 lines vs actual `parity.rs:713/838/892`) — corrected in Task 0 table, function names were correct so this is line-drift only (oracle: `grep -n "fn <name>"`, SOUND, self-closed per §2 — no Architect respond needed for pure line-number correction, matches phiếu's own note at line 64 "lệch số-dòng thuần... KHÔNG cần Architect RESPOND").

**Implementation note (Tầng 2, non-blocking — for Worker's own EXECUTE-time judgment call on Task 5):** `hash_file()` wrapper (trust-gate.sh:88-95) is the single call point used by BOTH the generate branch (:190) and the compare branch (:227). Normalizing inside `hash_file()` itself (strip leading `*` from its own output before returning) would also transparently fix the 3rd NF-read site at `baseline_paths=$(awk '{print $NF}' "${BASELINE_FILE}")` (:242, used for added/removed-surface `comm` detection) that Task 5's "Tìm" section doesn't explicitly call out — because a baseline written via a fixed `hash_file()` never contains `*` in the first place. Task 5's two-branch instruction (normalize at generate + compare) is still *correct and sufficient* as written (both branches ultimately call `hash_file`), this is just a DRY-er single-point option Worker may pick at EXECUTE — either way anchor #7's 3rd site is covered, not a gap requiring phiếu change.

**Status:** ✅ Worker accepted V1 — ready for Chủ nhà approval gate (no Architect RESPOND needed).

### Final consensus
- Phiếu version: V1 (no bump — no objections raised)
- Total turns: 1 (Worker CHALLENGE only, no Architect RESPOND required)
- Approved by Chủ nhà: **APPROVED 2026-07-24** at APPROVAL_GATE via `AskUserQuestion` — "Approve cả 2" (P087 + P088 approved in same gate; P087 executed in this spawn, P088 executes next per orchestrator sequencing).

### EXECUTE — Worker (2026-07-24)

Task 0 re-verified at EXECUTE time — all 10 anchors held (see `docs/discoveries/P087.md` for the full table). Tasks 1-6 shipped as specified. One discovery mid-EXECUTE, non-blocking (same class as the phiếu's own named CRLF exclusion, not a new Tầng 1 issue): fixing BUG 1's stdout-path assertion unmasked a second, previously-hidden assertion in `parity_map_enforced` — a pure `.gitattributes`/CRLF golden-checkout mismatch. Did not touch `.gitattributes` per Constraint 5; documented as a 4th CRLF-only expected-red test alongside `parity_new/adopt/sync_enforced`, deferred to P088. Full Discovery Report: `docs/discoveries/P087.md`.

**Status: ✅ SHIPPED.**

---

## Nhiệm vụ

### Task 1: Normalize `path_str` sang forward-slash TRƯỚC mọi substring match (BUG 1)

**File:** `crates/sos-cli/src/commands/map.rs`

**Tìm:** nơi `path_str` (= `path.to_string_lossy()`) được so bằng `.contains(s)` với các `SURFACES[].path_substrs` (POSIX substring) `[needs Worker verify]` — briefing chỉ ~:213 (derive) + :257-258 (match).

**Thay bằng / Thêm:** dùng dạng đã forward-slash của path cho MỌI `.contains()` match. Tái dùng CHÍNH pattern `.replace('\\', "/")` đã có trong file (anchor #5). Ví dụ ngữ nghĩa (Worker đặt tên biến cục bộ theo style file — đó là Tầng 2, Worker tự quyết):
```rust
let path_norm = path.to_string_lossy().replace('\\', "/");
// dùng path_norm cho mọi .contains(substr) trong surface-match loop
```

**Lưu ý:** normalize là ONE-CLASS fix — phải phủ CẢ 3 call site (surface-match loop + `is_noise` + `detect_present_stacks`). Task 2 + Task 3 xử 2 call site còn lại. POSIX behavior BẤT BIẾN (trên POSIX `\`→`/` no-op vì không có `\` trong path). KHÔNG đổi nội dung `path_substrs` (giữ POSIX form — chuẩn hoá bên input, không bên constant).

### Task 2: Normalize path trong `is_noise()` (BUG 1, call site 2)

**File:** `crates/sos-cli/src/commands/map.rs`

**Tìm:** thân `fn is_noise(...)` so `NOISE_EXCLUDE` substring với native path string `[needs Worker verify]` — briefing :183-185.

**Thay bằng / Thêm:** normalize path arg sang forward-slash trong `is_noise` trước khi `.contains()` với `NOISE_EXCLUDE`. Sửa TRONG hàm để mọi caller (bao gồm `detect_present_stacks`) hưởng chung — nếu khả thi theo signature.

**Lưu ý:** nếu `is_noise` nhận `&Path` (không phải `&str`) thì normalize nội bộ; nếu nhận `&str` đã native, normalize tại đây. Worker xác nhận signature. Nếu normalize nội bộ `is_noise` đã phủ `detect_present_stacks` → Task 3 chỉ còn verify-only.

### Task 3: `detect_present_stacks` dùng path đã normalize (BUG 1, call site 3)

**File:** `crates/sos-cli/src/commands/map.rs`

**Tìm:** `fn detect_present_stacks` nơi gọi `is_noise` `[needs Worker verify]` — briefing :82-83.

**Thay bằng / Thêm:** đảm bảo path đưa vào `is_noise` đã forward-slash (nếu Task 2 normalize nội bộ `is_noise` thì đây là verify-only, ghi rõ trong Discovery).

**Lưu ý:** tránh double-normalize vô hại nhưng thừa — chọn MỘT chỗ (khuyến nghị: nội bộ `is_noise`). Worker quyết layer (Tầng 2).

### Task 4: Normalize `out.display()` stdout xác nhận sang `/` (BUG 1, parity golden)

**File:** `crates/sos-cli/src/commands/map.rs`

**Tìm:** dòng stdout xác nhận in `out.display()` `[needs Worker verify]` — briefing :355-358, :368-372 — trên Windows in `<TARGET>\docs\AGENT_MAP.yaml`.

**Thay bằng / Thêm:** in dạng forward-slash (`out.display().to_string().replace('\\', "/")` hoặc tương đương) để khớp golden `<TARGET>/docs/AGENT_MAP.yaml`.

**Lưu ý:** parity test normalize `<TARGET>` NHƯNG KHÔNG normalize separator (failure text xác nhận). Đây là fix để `parity_map_enforced` xanh. POSIX no-op. Nếu có nhiều dòng emit `.display()`, phủ tất cả các dòng thuộc output contract.

### Task 5: Platform-invariant hash-line format trong `trust-gate.sh` (BUG 2)

**File:** `scripts/trust-gate.sh`

**Tìm:** nhánh generate baseline (`sha256sum ... | sort -k2 > tmp_baseline` `[needs Worker verify]` briefing :185-194) VÀ nhánh compare (`diff baseline tmp_current | grep '^[<>]' | awk '{print $NF}'` briefing :207-249, awk :237).

**Thay bằng / Thêm:** normalize format hash-line để loại `*` binary-mode prefix ở path field, áp ở CẢ generate lẫn compare. Hướng (Worker chọn dạng POSIX-sh/bash portable): sau `sha256sum`/`shasum`, pipe qua chuẩn hoá field — ví dụ ngữ nghĩa:
```sh
# strip a leading '*' from the path field, re-emit canonical 'HASH  PATH'
awk '{ h=$1; sub(/^\*/,"",$2); print h"  "$2 }'
```
hoặc `sed 's/ \*/  /'` tương đương — Worker verify khớp cả GNU `sha256sum` (binary `HASH *PATH`) lẫn macOS `shasum` fallback output.

**Lưu ý:** PHẢI giữ POSIX-sh/bash compat (Git Bash + macOS). Áp SYMMETRIC — nếu chỉ normalize một nhánh, diff vẫn lệch. `awk $NF` mang `*` theo gây doubled listing → sau fix, listing single. KHÔNG đổi thuật toán so sánh (chỉ format field). Giữ `sort -k2` để order-invariant.

### Task 6: Rebaseline `.sos-trust-baseline` sau khi sửa `trust-gate.sh`

**File:** `.sos-trust-baseline` (regenerate, KHÔNG sửa tay)

**Tìm:** N/A — đây là bước chạy lệnh.

**Thay bằng / Thêm:** sau Task 5, chạy lệnh rebaseline của trust-gate (`scripts/trust-gate.sh rebaseline` hoặc cú pháp Worker xác nhận từ usage `[needs Worker verify]`), rồi `git add .sos-trust-baseline`.

**Lưu ý:** sửa `trust-gate.sh` (auto-exec surface) TỰ trip baseline → không rebaseline = hook `[8/8]` sẽ block chính commit này. Rebaseline PHẢI chạy trên môi trường có format mới (sau fix) để baseline sinh ra ở canonical `HASH  PATH` form. Verify baseline mới KHÔNG chứa `*`-prefix path.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-cli/src/commands/map.rs` | Task 1-4: normalize `path_str`/`is_noise`/`detect_present_stacks` input + `out.display()` output sang forward-slash |
| `scripts/trust-gate.sh` | Task 5: platform-invariant hash-line format (strip `*` binary-mode prefix) ở generate + compare |
| `.sos-trust-baseline` | Task 6: regenerate qua rebaseline lệnh (KHÔNG sửa tay) |
| `SECURITY.md` | Docs Gate: nếu baseline FORMAT đổi → update rebaseline workflow / INV-TRUST note |
| `CHANGELOG.md` | Docs Gate: entry P087 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-cli/tests/parity.rs` | `parity_map_enforced`, `oa02_templates_excluded_from_frontend`, `new_first_commit_passes_all_hooks_zero_seed` xanh sau fix; goldens byte-stable trên POSIX |
| `.gitattributes` | KHÔNG đụng (P088 territory) |
| `hooks/pre-commit` | Phase count `[8/8]` KHÔNG đổi; `[8/8]` trust-gate vẫn gọi `scripts/trust-gate.sh` như cũ |

---

## Luật chơi (Constraints)

1. POSIX behavior BẤT BIẾN — mọi normalization là no-op trên POSIX (không có `\` trong path; hash-line strip `*` chỉ tác động binary-mode output). Nghiệm thu: POSIX goldens byte-stable, không regression.
2. `trust-gate.sh` PHẢI giữ POSIX-sh/bash compat + nhánh macOS `shasum` fallback nguyên vẹn.
3. KHÔNG chuẩn hoá bên constant (`path_substrs`/`NOISE_EXCLUDE` giữ POSIX form) — chỉ chuẩn hoá bên input path. Single-source, không tạo 2 dạng constant.
4. Sửa `trust-gate.sh` → BẮT BUỘC rebaseline `.sos-trust-baseline` cùng commit (Task 6). Security surface — không rebaseline = phiếu CHƯA XONG.
5. KHÔNG mở scope sang CRLF/EOL/symlink (P088). Nếu full `[8/8]` còn đỏ trên Windows sau P087 do CRLF-hash mismatch → ghi Discovery "expected, P088 dependency", KHÔNG fix ở đây.

---

## Nghiệm thu

### Automated
- [ ] `cargo check --workspace` clean.
- [ ] Trên Windows: `cargo test --workspace` → `parity_map_enforced`, `oa02_templates_excluded_from_frontend`, `new_first_commit_passes_all_hooks_zero_seed` **PASS** (3 CRLF-only fail có thể còn đỏ tới khi P088 merge — ghi rõ).
- [ ] Trên POSIX (linux + macOS): `cargo test --workspace` KHÔNG regression (goldens byte-stable).

### Manual Testing
- [ ] Windows: `sos map` trên repo có `.git/`+`node_modules/` → noise bị loại, surfaces (routes/lib/…) được detect (không rỗng).
- [ ] Windows: dòng stdout xác nhận in `<TARGET>/docs/AGENT_MAP.yaml` (forward-slash).
- [ ] Windows Git Bash: first commit sau `sos new` qua hook `[8/8]` không bị false-BLOCK; trust-gate listing KHÔNG doubled (`*`-prefix biến mất).
- [ ] `.sos-trust-baseline` mới KHÔNG chứa path field bắt đầu `*`.

### Regression
- [ ] POSIX trust-gate vẫn bắt tamper thật (thêm/sửa auto-exec file → BLOCK) — fail-CLOSED nguyên.
- [ ] `sos map` trên POSIX cho output y hệt trước fix.

### Docs Gate
- [ ] `CHANGELOG.md` — entry P087.
- [ ] `SECURITY.md` — cập nhật nếu baseline FORMAT đổi (rebaseline workflow / trust-gate line-format note). Nếu format hiển-thị-baseline KHÔNG đổi (vẫn `HASH  PATH`) → ghi "Tầng 1 N/A — format canonical không đổi, chỉ normalize input Windows-side" trong Discovery.

### Discovery Report
- [ ] `docs/discoveries/P087.md` — assumptions CORRECT/WRONG (file:line citations thực sau grep), scope (CRLF còn đỏ = P088 dependency, note), edge cases (signature `is_noise`, cú pháp rebaseline), docs updated, tier escalations.
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
