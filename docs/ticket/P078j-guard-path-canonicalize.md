# PHIẾU P078j: guard path canonicalization — symlinked repo roots (macOS /tmp → /private/tmp)

---

> **Loại:** Bugfix (security — actor-check / approval-gate guard path-matching)
> **Ưu tiên:** P1 — round-4 B4 FAIL: forbidden advance LỌT + legit approval false-BLOCK khi path đi qua symlink alias. Security integrity + usability.
> **Tầng:** 1 — chạm path-matching feed vào actor-check / exemption (security boundary, nối P078h) = AUTO Tầng 1 dù nhỏ. LOC KHÔNG quyết.
> **Lane:** Guarded — security guard logic, no-cap (real symlink fixture matrix + negative-test bắt buộc, không always-green). SECURITY → Worker CHALLENGE bắt buộc.
> **Ảnh hưởng:** `crates/sos-adapter-codex/src/templates.rs` (guard content-fn: path-normalize bước — nâng lexical strip → canonicalize); `crates/sos-adapter-codex/src/lib.rs` test module.
> **Dependency:** None — độc lập, SONG SONG với **P078i** (install/hook-render, gap#1 round-4). Nền = **P078h** (`74edc20`/`dd4594d` merge) ship lexical path-normalize + marked-actor early-BLOCK. P078j chỉ nâng bước path-canonicalize; KHÔNG đổi early-BLOCK logic.

---

## Context

### Vấn đề hiện tại

P079 **round-4** live-dogfood (`docs/adapters/P079-ROUND4-FINDINGS-2026-07-23.md`, **Test B4 FAIL**, `:149-191` + §"New/remaining gaps" #2 `:212-216`) phơi lỗ trong path-normalize P078h ship:

- P078h normalize absolute→relative bằng **strip literal prefix `$REPO_ROOT/`** (REPO_ROOT = `git rev-parse --show-toplevel`, macOS canonical hoá thành `/private/tmp/...`). Normalize = **lexical string-strip, KHÔNG resolve symlink/`..`** (findings `:212` "lexical, not filesystem-equivalent"; P078h Worker Turn 1 confirm `rg normalize|realpath = 0`).
- macOS: `/tmp` là **symlink → `/private/tmp`**. Apply_patch path `/tmp/proj/.sos-state/ticket-state.env` KHÔNG khớp prefix `/private/tmp/proj/` → guard coi là path lạ → normalize không ra relative `.sos-state/ticket-state.env`.
- **Hậu quả HAI CHIỀU** (findings B4):
  1. **marked-actor** dùng `/tmp/...` → không match state-file → early-BLOCK (P078h Task 1) KHÔNG fire → forbidden advance `V3/V3 → V4/V4` **LỌT** (`:160-168`).
  2. **main-thread** dùng `/tmp/...` legit approval → không match state-file → không được state-file-only exemption → fail-closed **false-BLOCK** (`:177-187`).
  - Canonical `/private/tmp/...` xử ĐÚNG cả 2 (`:170-175`) → chứng minh lỗ CHỈ ở path-equivalence, không ở early-BLOCK/exemption logic.
- Findings §"gaps" #2: *"Normalize BOTH the candidate path and repo root consistently (including symlink resolution) before actor/exemption comparisons."*

### Giải pháp

Additive, 1 area (`sos-adapter-codex`), nâng **1 bước** trong guard content-fn (`templates.rs`) từ lexical-strip → **canonicalize BOTH sides**:
- **Task 1:** trong rendered guard bash, canonicalize **CẢ** `REPO_ROOT` **VÀ** mỗi candidate patch path (resolve symlink) TRƯỚC khi compare/strip → `/tmp/proj/x` và `/private/tmp/proj/x` cùng resolve về một canonical form → match đúng. Xử **target chưa tồn tại** (file đang được TẠO) — `realpath`/`readlink -f` trên path chưa tồn tại có thể fail; Worker chọn cơ chế portable.
- **Task 2:** real symlink fixture matrix (tái dùng round-4 /tmp→/private/tmp case) + negative-test.

**Giữ nguyên (KHÔNG đổi):** marked-actor early-BLOCK (P078h Task 1), main-thread exemption, d2a multi-path all-path loop — chỉ sửa bước path feed vào chúng. Path-matching chính xác hơn → early-BLOCK + exemption fire đúng.

**Conservative fail-closed (giữ P078h):** path không resolve an toàn về repo-root canonical (`../` escape / symlink trỏ NGOÀI repo) → coi là path lạ → KHÔNG exempt → guard block. Canonicalize KHÔNG được mở đường bypass multi-path guard.

### Scope

- **CHỈ sửa:** `crates/sos-adapter-codex/src/templates.rs` (guard content-fn — bước path-normalize); `crates/sos-adapter-codex/src/lib.rs` test module; docs (CHANGELOG/SECURITY/CAPABILITY nếu ref/Discovery/BACKLOG).
- **KHÔNG sửa:** `crates/sos-install/**`, `crates/sos-cli/**`, engine, hook-render (= **P078i** surface, gap#1 round-4 — song song); actor-check early-BLOCK LOGIC (P078h Task 1 — chỉ sửa path feed vào, KHÔNG đổi block quyết định); d2a multi-path guard core; SubagentStart marker (d2b); `scripts/orchestrator-guard.sh`; `sos-core`, `sos-adapter-claude`.

---

## Task 0 — Verification Anchors

> Architect docs-only. Round-4 fact = `[verified: P079-ROUND4-FINDINGS-2026-07-23.md]`. P078h-shipped code-site = P078h Worker Turn 1 confirmed line ranges NHƯNG P078h merge (`74edc20`) vừa CHÈN early-BLOCK + normalize → **mọi line number DRIFT**. Worker re-grep post-P078h-merge, đánh `[needs Worker verify]`. **Cite RANGE, không count.**

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | P078h path-normalize bước = **lexical prefix-strip** của `$REPO_ROOT/` (KHÔNG resolve symlink). Đây là site P078j nâng. `[verified: findings :212; P078h Turn 1 "rg normalize\|realpath = 0"; needs Worker verify exact site post-P078h merge]` | `rg -n "REPO_ROOT\|#\*/\|\$\{.*#\|strip\|normalize" crates/sos-adapter-codex/src/templates.rs` trong guard content-fn → tìm chỗ strip repo-root prefix khỏi candidate path | ⏳ TO VERIFY |
| 2 | `REPO_ROOT` compute = `git rev-parse --show-toplevel \|\| pwd` trong rendered guard bash (P078h Turn 1: pre-merge `:746`, giờ drift). macOS `--show-toplevel` trả canonical `/private/tmp/...`. `[needs Worker verify exact line post-merge]` | `rg -n "REPO_ROOT=\|show-toplevel\|rev-parse" crates/sos-adapter-codex/src/templates.rs` | ⏳ TO VERIFY |
| 3 | Guard content-fn render **BASH** (canonicalize chạy ở hook runtime trong shell, KHÔNG Rust `fs::canonicalize`) → portability = bash-level (`realpath`/`readlink -f`/`pwd -P`), macOS BSD ≠ GNU. `[needs Worker verify — fn trả String bash template]` | Đọc `guard_approval_gate_sh()` (P078h Turn 1: `guard_approval_gate_sh` ~`:667-828` pre-merge) → xác nhận output = bash script string | ⏳ TO VERIFY |
| 4 | Candidate path extract từ apply_patch header có thể là **file chưa tồn tại** (patch đang TẠO `.sos-state/ticket-state.env` — bootstrap case) → `realpath`/`readlink -f` strict trên path chưa tồn tại FAIL trên một số platform. `[verified: bootstrap create case findings B1 :103-124; needs Worker verify extract site + có `-m`/`-e` flag không]` | `rg -n "for RAW_PATH\|while read\|Update File\|Add File\|apply.patch\|path" templates.rs` → path-extract loop; kiểm bootstrap tạo file mới | ⏳ TO VERIFY |
| 5 | Normalize áp cho MỌI candidate TRƯỚC cả state-file-allow (exemption) VÀ d2a all-path multi-path loop — 1 chỗ normalize, 2 consumer. Canonicalize thay đúng chỗ lexical-strip cũ (Task 1), KHÔNG regress multi-path. `[verified: P078h Turn 1 "normalize insert per-path inside/before for RAW_PATH loop"; needs Worker verify post-merge]` | `rg -n "for RAW_PATH\|NON_TICKET_PATHS\|STATE_FILE\|ticket-state.env" templates.rs` → xác nhận normalize feed cả 2 consumer | ⏳ TO VERIFY |
| 6 | Test harness `run_guard()` + `with_worker_active`/`with_architect_active` + fixture `tests/fixtures/codex-apply-patch-payloads.jsonl` (`include_str!`) trong `lib.rs`. Cần khả năng dựng **symlink dir → repo** trong test (`std::os::unix::fs::symlink`). `[verified: P078h Turn 1 :91-94; needs Worker verify + symlink support]` | `rg -n "run_guard\|with_worker_active\|synthetic_apply_patch\|symlink\|fixtures" crates/sos-adapter-codex/src/lib.rs` | ⏳ TO VERIFY |

**Nếu ❌ (P078h normalize đã resolve symlink / lỗ B4 không tái hiện) → Worker DISCOVERY_REPORT + dừng, KHÔNG "sửa mò".**
**Nếu canonicalize làm marked-actor early-BLOCK / main-thread exemption đổi HÀNH VI (không chỉ path chuẩn hơn) → ESCALATE: chệch scope P078j, có thể regress P078h security.**

### Pre-phiếu snapshot (Worker auto first-step)

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

> Cap = 3 turns. SECURITY → Worker CHALLENGE bắt buộc. **Append-only.**

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Anchor verification (grep-confirmed, post-merge `2393c1b`):**
- #1 ✅ `templates.rs:795` `case "$RAW_PATH" in "$REPO_ROOT"/*) NORM_PATH="${RAW_PATH#"$REPO_ROOT"/}" ;; esac` — pure lexical prefix-strip, no `realpath`/`canonicalize` in fn (only comment at `:782`). Gap B reproduces as findings describe, not already fixed.
- #2 ✅ `:748` `REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)`.
- #3 ✅ `guard_approval_gate_sh()` `:667-869` returns a bash-script String → canonicalize happens at shell runtime, not Rust.
- #5 ✅ single normalize loop `:787-801` feeds state-file-alone exemption (`:842-846`), marked-actor early-BLOCK (`:825-832`), and `NON_TICKET_PATHS` for d2a multi-path — one site, all consumers.
- #6 ⚠️ `run_guard()` (`lib.rs:634-668`) spawns bash in a plain non-git temp dir with NO symlink scaffolding — Task 2 must add it (confirms Task 2 scope, not a spec error).

**Objections (Tầng 1 only):** Worker accepted V1 — no challenges. Gap B confirmed real; canonicalize only feeds cleaner paths into P078h's early-BLOCK/exemption, does not alter those decisions.

**Status:** ✅ ACCEPTED V1 — no Architect response needed

### Final consensus
- Phiếu version: V1 (accepted as-drafted)
- Total turns: 1 (Worker accepted, 0 objections)
- Approved by Chủ nhà: 2026-07-23 — self-approved by Quản đốc under delegated sprint authority (Guarded/Tầng-1 security guard, no owner-decision, no Codex run required).
- **PORTABLE MECHANISM (Worker-confirmed via `man realpath` on macOS — BSD realpath has NO `-m`/`-e`/`-s`, fails on not-yet-created path):** resolve DIRNAME only — `RESOLVED_DIR=$(cd "$(dirname "$RAW_PATH")" 2>/dev/null && pwd -P)` (POSIX `cd && pwd -P`, no GNU flags), re-append `basename`; if `cd` fails → fall through to raw path unchanged (preserves P078h conservative fail-closed — unresolved path never wins exemption). Canonicalize `REPO_ROOT` the same way. Normalize insert = loop `:787-801`.

---

## Nhiệm vụ

### Task 1: canonicalize BOTH REPO_ROOT + candidate path (resolve symlink) TRƯỚC compare/strip

**File:** `crates/sos-adapter-codex/src/templates.rs` — guard content-fn, bước path-normalize (P078h lexical-strip site, `[needs Worker verify exact fn+line post-P078h merge]`).

**Tìm:** bước hiện tại strip literal prefix `$REPO_ROOT/` khỏi candidate path (lexical, findings `:212`). `REPO_ROOT` = `git rev-parse --show-toplevel || pwd`. Candidate `/tmp/...` KHÔNG khớp canonical `$REPO_ROOT`=`/private/tmp/...` → normalize sai.

**Thay bằng / Thêm:** nâng thành **canonicalize cả 2 vế** TRƯỚC strip (trong rendered bash):
- Canonicalize `REPO_ROOT` → canonical form (idempotent nếu git đã trả canonical; xử case `git rev-parse` fail → `pwd` non-canonical `/tmp` cũng resolve). `[needs Worker verify cơ chế]`
- Canonicalize MỖI candidate patch path → cùng canonical form → strip repo-root prefix. Sao cho `/tmp/proj/.sos-state/ticket-state.env` và `/private/tmp/proj/.sos-state/ticket-state.env` cùng resolve về relative `.sos-state/ticket-state.env`.
- **Target chưa tồn tại (bắt buộc xử):** candidate có thể là file đang TẠO (chưa có trên disk) → strict `realpath`/`readlink -f` FAIL. Worker chọn cơ chế portable (vd: resolve dirname `cd "$(dirname X)" && pwd -P` rồi re-append basename; hoặc `realpath -m`; hoặc canonicalize existing-ancestor). `[needs Worker verify — chọn 1, ghi rationale Discovery]`
- Áp normalize cho MỌI candidate TRƯỚC cả state-file-allow (exemption) VÀ d2a all-path loop (Task 0 #5) — thay đúng chỗ lexical-strip cũ, 1 chỗ, 2 consumer.

**Lưu ý:**
1. **Portable (macOS BSD ≠ GNU):** guard render bash chạy runtime trên máy user (macOS/Linux) → KHÔNG giả định GNU `readlink -f` / `realpath -m` có sẵn. BSD `realpath` cũ thiếu `-m`; macOS `readlink` không có `-f`. Ưu tiên POSIX (`cd … && pwd -P` cho dir portion) HOẶC feature-detect. `[needs Worker verify — đây là câu hỏi mở chính, xem CHALLENGE]`
2. **Conservative fail-closed (giữ P078h):** path canonicalize xong nằm NGOÀI repo-root canonical (`../` escape / symlink trỏ ngoài) → coi path lạ → KHÔNG exempt → để guard block. Canonicalize KHÔNG mở bypass.
3. **KHÔNG đổi early-BLOCK/exemption LOGIC** — chỉ path feed vào chuẩn hơn. Marked-actor→BLOCK, main-thread→ALLOW quyết định GIỮ (P078h Task 1). Nếu thấy phải sửa quyết định → chệch scope → ESCALATE.
4. **d2a multi-path KHÔNG regress:** bundle (state+code) sau canonicalize vẫn multi-path → BLOCK.

### Task 2: real symlink fixture matrix + negative-test

**File:** `crates/sos-adapter-codex/src/lib.rs` test module (`run_guard()` + `with_worker_active` closure + symlink dựng, P078h/P078e-established).

**Thêm test (mock apply_patch payload REAL b3 shape + marker fixture + repo-root là symlinked dir, qua `run_guard`):**
1. **gap B core — marked-actor advance qua symlink path → BLOCK (đỏ trước fix):** REPO_ROOT canonical `/private/…`, patch path dùng `/tmp/…` alias, `.sos-state/worker-active` SET, state approved `V3/V3` → advance `V4/V4` → **BLOCK**. (round-4 B4: LỌT.)
2. **gap B main-thread — legit approval qua symlink path → ALLOW (đỏ trước fix):** BOTH marker ABSENT, patch path `/tmp/…` cho ticket-state.env, state `V5/empty` → approval `V6/V6` → **ALLOW** (round-4 B4 false-block `:177-187`).
3. **canonical no-regress P078h:** `/private/tmp/…` path — marked→BLOCK; main-thread→ALLOW (P078h behavior giữ nguyên).
4. **relative no-regress:** relative `.sos-state/ticket-state.env` — marked→BLOCK; main-thread→ALLOW.
5. **bundle qua symlink (multi-path no-regress):** patch ticket-state.env (`/tmp/…`) **kèm** code path (`/tmp/…/src/**`) → **BLOCK**.
6. **path-trick fail-closed:** candidate canonicalize ra NGOÀI repo-root (`../` escape / symlink dir trỏ ngoài repo) → KHÔNG exempt (không được state-file-allow) → guard xử conservative (không bypass multi-path).
7. **negative-test (răng, bắt buộc — ghi Discovery):** revert Task 1 canonicalize (về lexical-strip) → test #1 BLOCK→ALLOW (advance lọt) = FAIL **VÀ** test #2 ALLOW→BLOCK (false-block) = FAIL. Canonicalize load-bearing 2 chiều.

**Lưu ý:**
1. **Dựng symlink THẬT:** test env tạo symlink dir → git repo mô phỏng /tmp→/private/tmp (`std::os::unix::fs::symlink(real_dir, link_dir)`). Absolute-path fixture dùng LINK path; REPO_ROOT (`git rev-parse`) trả REAL path → tái hiện mismatch. **CI Linux không có /tmp-symlink sẵn** → tạo symlink thủ công trong tempdir (KHÔNG rely /tmp là symlink). `[needs Worker verify symlink test helper]`
2. Fixture = REAL b3 apply_patch shape (`tests/fixtures/codex-apply-patch-payloads.jsonl` nếu còn, Task 0 #6), KHÔNG string bịa. Marker = file thật temp `.sos-state`.
3. KHÔNG always-green: test #1 + #2 PHẢI đỏ trước fix (2 chiều gap B). Negative-test bắt buộc.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-adapter-codex/src/templates.rs` | Task 1: canonicalize BOTH REPO_ROOT + candidate path (resolve symlink, portable, target-not-exist-safe) thay lexical prefix-strip; conservative fail-closed |
| `crates/sos-adapter-codex/src/lib.rs` | Task 2: symlink fixture matrix (marked-advance BLOCK + main-thread ALLOW qua /tmp alias + canonical/relative no-regress + bundle BLOCK + path-trick fail-closed + negative 2-chiều) |
| `SECURITY.md` | control-plane path-matching giờ **symlink-safe** (canonicalize BOTH sides trước actor/exemption compare); conservative fail-closed path-trick |
| `adapters/codex/CAPABILITY.md` | (nếu ref path-normalize) actor-check path giờ symlink-equivalent, không chỉ lexical |
| `CHANGELOG.md` | entry P078j (path canonicalization symlinked roots — round-4 B4 fix) |
| `docs/discoveries/P078j.md` | Discovery Report (mới) |
| `docs/BACKLOG.md` | mark P078j DONE (active sprint) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `templates.rs` — marked-actor early-BLOCK (P078h Task 1) | Quyết định BLOCK/ALLOW KHÔNG đổi — chỉ path feed vào chuẩn hơn (test #3/#4 chứng minh no-regress) |
| `templates.rs` — main-thread exemption + d2a multi-path all-path loop | Exemption + bundle-BLOCK KHÔNG regress sau canonicalize (test #4/#5) |
| `templates.rs` — SubagentStart marker touch (d2b) | Marker path khớp actor-check; KHÔNG đổi |
| `scripts/orchestrator-guard.sh` | Đọc semantics — KHÔNG sửa |
| `crates/sos-install/**`, `crates/sos-cli/**`, engine, hook-render | **P078i** (gap#1 round-4) — song song, KHÔNG chạm |
| `crates/sos-core/**`, `crates/sos-adapter-claude/**` | Untouched |

---

## Luật chơi (Constraints)

1. **Canonicalize BOTH sides:** REPO_ROOT VÀ candidate patch path cùng resolve symlink TRƯỚC compare/strip → `/tmp/x` ≡ `/private/tmp/x`. Chỉ 1 vế = tái lỗ B4.
2. **Target-not-exist-safe:** file đang TẠO (chưa trên disk) phải canonicalize được (resolve dirname re-append, hoặc `-m`, hoặc existing-ancestor). Strict realpath fail trên bootstrap create = regress B1.
3. **Portable macOS BSD ≠ GNU:** guard bash chạy runtime user-machine → KHÔNG giả định GNU `readlink -f`/`realpath -m`. POSIX-first hoặc feature-detect. (Câu hỏi mở chính.)
4. **Conservative fail-closed (giữ P078h):** canonicalize ra NGOÀI repo-root (`../`/symlink ngoài) → path lạ → KHÔNG exempt → guard block. Canonicalize KHÔNG mở bypass multi-path.
5. **KHÔNG đổi early-BLOCK/exemption LOGIC (P078h Task 1):** chỉ path-matching chính xác hơn. Marked→BLOCK, main-thread→ALLOW GIỮ. Đổi quyết định = chệch scope → ESCALATE.
6. **1 chỗ normalize, 2 consumer:** canonicalize feed cả state-file-allow + d2a multi-path; multi-path KHÔNG regress (bundle qua symlink vẫn BLOCK).
7. **Oracle real, 2-chiều, không always-green:** Task 2 fixture = symlink dir THẬT + REAL b3 payload qua `run_guard`; test #1 (advance lọt) + #2 (false-block) PHẢI đỏ trước fix. Negative-test bắt buộc (revert → cả 2 chiều FAIL). CI Linux tạo symlink thủ công.
8. **Additive + dep-direction:** adapter→core, no core→adapter, KHÔNG đổi public signature. Độc lập P078i — KHÔNG chạm install/cli/engine/hook-render.
9. **Dựa round-4 findings THẬT:** lỗ không khớp spec → DISCOVERY + báo, KHÔNG sửa mò. Lane Guarded declared.

---

## Nghiệm thu

### Automated
- [ ] `cargo check` clean
- [ ] `cargo test -p sos-adapter-codex` pass
- [ ] **Oracle (real symlink guard fixtures — repo-root là symlinked dir):**
  - marked-actor + approved V3/V3 + advance V4/V4 qua `/tmp/…` alias → **BLOCK** (gap B, đỏ trước fix)
  - main-thread + legit approval qua `/tmp/…` alias → **ALLOW** (gap B false-block, đỏ trước fix)
  - canonical `/private/tmp/…` → marked BLOCK / main-thread ALLOW (P078h no-regress)
  - relative path → marked BLOCK / main-thread ALLOW (no-regress)
  - bundle (state+code) qua `/tmp/…` → **BLOCK** (multi-path no-regress)
  - path-trick (`../`/symlink ngoài repo) → KHÔNG exempt (fail-closed)
- [ ] **Negative-test (2-chiều):** revert canonicalize → advance BLOCK→ALLOW = FAIL **VÀ** false-block ALLOW→BLOCK = FAIL. Ghi Discovery
- [ ] Flake gate: `cargo test -p sos-adapter-codex` ×20 → 0-flaky
- [ ] Dep-direction guard xanh (adapter→core)

### Manual Testing
- [ ] (nếu có macOS + Codex 0.145.0) repro round-4 B4 trong `/tmp/…` repo: marked advance → BLOCK; main-thread approval → ALLOW (defer nếu không có instance)

### Regression
- [ ] P078h marked-actor early-BLOCK (canonical + relative path) KHÔNG đổi
- [ ] P078h main-thread exemption + P078e deadlock fix KHÔNG regress
- [ ] d2a multi-path (bundle BLOCK) + bootstrap create (B1) KHÔNG regress
- [ ] SubagentStart marker (d2b) + 3 startup render (d1) output KHÔNG đổi

### Docs Gate
- [ ] `SECURITY.md` — path-matching symlink-safe (canonicalize BOTH sides trước actor/exemption compare) + conservative fail-closed. **Trust-gate:** verify SECURITY.md content plain ASCII (INV-TRUST-02); rebaseline CHỈ nếu SURFACE_GLOBS đổi — ghi Discovery có/không (path-normalize sửa nội bộ guard content-fn, SURFACE_GLOBS thường KHÔNG đổi)
- [ ] `adapters/codex/CAPABILITY.md` — nếu ref path-normalize (verify có ref không; N/A explicit nếu không)
- [ ] `CHANGELOG.md` — entry P078j
- [ ] `docs/discoveries/P078j.md`
- [ ] `docs/BACKLOG.md` — P078j DONE

### Discovery Report
- [ ] Write to `docs/discoveries/P078j.md`
  - Anchor #1–6 — CORRECT / WRONG (file:RANGE thật cho P078h normalize site post-merge + REPO_ROOT compute + path-extract loop + run_guard harness)
  - **Cơ chế canonicalize chọn (bắt buộc):** portable primitive dùng (`pwd -P` dirname re-append / `realpath -m` / feature-detect), rationale macOS-BSD-vs-GNU + target-not-exist xử sao
  - **2-chiều fix note:** gap B là HAI hậu quả (advance lọt + false-block); canonicalize sửa cả 2 — chứng minh bằng test #1 + #2
  - **No-regress note:** early-BLOCK/exemption LOGIC (P078h Task 1) KHÔNG đổi, chỉ path feed chuẩn hơn (test #3/#4)
  - **Fail-closed note:** path-trick (`../`/symlink ngoài) → KHÔNG exempt (test #6)
  - Symlink test helper: cách dựng symlink dir → repo trong test (portable CI Linux)
  - Negative-test kết quả (revert → cả 2 chiều FAIL)
  - trust-gate SECURITY.md rebaseline — có chạy không, tại sao
  - Docs updated (SECURITY/CAPABILITY hoặc "N/A" explicit)
  - Tier escalations (None expected — canonicalize đổi HÀNH VI early-BLOCK/exemption / path-trick bypass được → escalate)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
