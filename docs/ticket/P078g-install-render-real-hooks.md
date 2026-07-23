# PHIẾU P078g: `sos install` phải RENDER + arm REAL Git hooks (không arm con trỏ rỗng)

---

> **Loại:** Bugfix
> **Ưu tiên:** P1
> **Tầng:** 1 (security backstop surface — install-time hook-arming LAN tới mọi adopter; arm-empty = false-security, tệ hơn tắt hẳn → AUTO Tầng 1)
> **Lane:** Guarded
> **Ảnh hưởng:** `crates/sos-install/src/engine.rs`, (nếu cần) adapter render plan (`crates/sos-adapter-claude/**` + `crates/sos-adapter-codex/**` plan-only), install-smoke test
> **Dependency:** None — independent surface với P078h (actor-check guard), chạy song song.

---

## Context

### Vấn đề hiện tại

**P078f (`d35f462`, đã merge) arm một con trỏ hook RỖNG.** `sos install` set `git config --local core.hooksPath=hooks` NHƯNG adapter render plan (18 file) KHÔNG render `hooks/pre-commit` / `hooks/pre-push` scripts vào target repo → git trỏ vào một dir không có hook → boundary OFF nhưng "trông như armed".

P079 round-3 dogfood (Sếp+Codex @`0108f99`, `docs/adapters/P079-ROUND3-FINDINGS-2026-07-23.md`):
- **Test A1 PASS** — `core.hooksPath` == `hooks` được set.
- **Test A2 FAIL** — `hooks/pre-commit` + `hooks/pre-push` KHÔNG tồn tại sau install (`stat: No such file or directory`).
- **Test A3 FAIL** — commit thật một `.env` → KHÔNG bị chặn (`ENV_COMMIT_EXIT=0`, commit thành công).
- **Test A4 FAIL** — commit code trên default branch → KHÔNG bị chặn (`DEFAULT_CODE_COMMIT_EXIT=0`).
- **Test A5 PASS** — foreign `core.hooksPath` KHÔNG bị clobber (F09 hijack-guard giữ nguyên, KHÔNG đụng).

Backstop "trông như armed" nhưng chạy con số 0 = **false-security, tệ hơn tắt hẳn** (adopter tin có bảo vệ). Đây là fix cho lỗ P078f để lại (arm-empty).

**Lỗi P078f test không bắt được (structural-oracle-gap):** 5 test của P078f **seed hook file sẵn trong fixture** (`crates/sos-install/tests/…` `git init` + tự tạo `hooks/pre-commit`) → không bao giờ chạy đường fresh-install-no-hooks. Phiếu này BẮT BUỘC test đường fresh (xem Oracle).

### Giải pháp

Hai phần:

1. **RENDER** — `sos install` phải đưa `hooks/pre-commit` + `hooks/pre-push` (từ kit source) vào target repo cho **CẢ 2 runtime** (claude + codex), **TRƯỚC** khi arm `core.hooksPath`. Mirror pattern `sos new` (`crates/sos-cli/src/commands/new.rs:315-322` copy hook từ kit-root + `chmod_x`). Non-clobber: đừng đè hook user đã tùy biến (giữ F09 semantics round-3-A5 đã PASS).
2. **HARDEN arm** — `arm_git_hooks()` (P078f, trong `crates/sos-install/src/engine.rs`) KHÔNG được set `core.hooksPath=hooks` khi hook files absent. Hoặc render-trước-rồi-arm (thứ tự đảm bảo files có mặt), hoặc **refuse/warn-loud** (never arm-empty). **Fail-loud > false-armed.**

**Worker quyết chỗ đặt render** (EXECUTE-time, Constraint 1): trong `arm_git_hooks()`/engine copy trực tiếp, hay như một artifact trong adapter plan. Ưu tiên: nơi nào đảm bảo files có mặt TRƯỚC arm và symmetric cho cả 2 runtime.

### Scope
- CHỈ sửa: `crates/sos-install/src/engine.rs` (render + harden arm), (nếu Worker chọn plan-route) adapter plan cho hook artifact, install-smoke test.
- KHÔNG sửa: actor-check / approval guard logic (`crates/sos-adapter-codex/**` guard) = **P078h** surface.
- KHÔNG đổi F09 hijack-guard behavior (round-3 A5 PASS — non-clobber giữ nguyên).
- KHÔNG delete/rewrite `scripts/install-hooks.sh` (reference-only).

---

## Task 0 — Verification Anchors

> **⚠️ Architect KHÔNG đọc code (no Grep / no source-read envelope).** Mọi anchor code-level `[needs Worker verify]` — Worker grep confirm rồi mới edit.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `sos new` COPY `hooks/pre-commit`+`pre-push` từ kit-root vào target rồi `chmod_x` — pattern tham chiếu để port | `sed -n '300,330p' crates/sos-cli/src/commands/new.rs` — confirm copy+chmod_x logic quanh L315-322 (kit-source path resolve + copy fn + chmod fn tên gì) `[needs Worker verify line]` | ⏳ TO VERIFY |
| 2 | `arm_git_hooks()` (P078f) trong `crates/sos-install/src/engine.rs` chạy trong `engine::apply` trước `Ok(report)`; hiện `chmod_x_if_exists(hooks/pre-commit)` skip lặng khi file absent → arm-rỗng | `grep -n "arm_git_hooks\|hooksPath\|chmod" crates/sos-install/src/engine.rs` — confirm arm fn + skip-silent branch `[needs Worker verify]` | ⏳ TO VERIFY |
| 3 | Adapter render plan KHÔNG render git hook scripts — **cả 2 runtime**. Codex: `crates/sos-adapter-codex/src/lib.rs` `fn plan()` (~L92) render 18 Codex-native artifact, không có hooks | `grep -n "pre-commit\|pre-push\|hooks/" crates/sos-adapter-codex/src/lib.rs crates/sos-adapter-claude/src/lib.rs` — confirm CẢ 2 plan có/không render hooks `[needs Worker verify — round-3 chỉ test codex]` | ⏳ TO VERIFY |
| 4 | Kit-source hook path (`hooks/pre-commit`, `hooks/pre-push`) accessible từ install context (giống cách `new.rs` resolve kit-root) | Confirm cách engine/install biết kit-source root (env / bundled / arg) — cùng cơ chế `new.rs` Anchor #1 dùng `[needs Worker verify]` | ⏳ TO VERIFY |
| 5 | `hooks/pre-push` tồn tại trong kit source (render phải cover cả pre-push, không chỉ pre-commit) | `ls -la hooks/pre-commit hooks/pre-push` | ⏳ TO VERIFY |
| 6 | Install-smoke test hiện (P078f) seed hook file trong fixture → KHÔNG exercise fresh-install-no-hooks path | `grep -rn "hooks/pre-commit\|git init\|TempFixture" crates/sos-install/tests/` — confirm fixture seed hooks; phiếu này thêm fresh-no-seed case `[needs Worker verify]` | ⏳ TO VERIFY |

**Nếu Anchor #3 cho thấy claude adapter ĐÃ render hooks còn codex thì không** → asymmetry là root-cause; Worker note trong Discovery + fix cho parity. Nếu CẢ HAI đều không render → render-route áp cho cả hai.

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

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Anchor verification (grep-confirmed):**
- #1 ✅ `new.rs:251-252` (`kit = SOS_KIT_DIR env`) + `L315-322` copy hooks + `chmod_x`.
- #2 ✅ `engine.rs:323-372` `arm_git_hooks()`: `chmod_x_if_exists` no-op if absent (`L390-400`) then UNCONDITIONAL `git_config_set(hooksPath,"hooks")` `L372` → arms empty path, confirmed.
- #3 ⚠️ **Worse than assumed:** `ClaudeAdapter.plan()` is a total STUB (`sos-adapter-claude/src/lib.rs:23-26`, `Plan::default()`, 0 assets) — `--runtime claude` renders NOTHING today (pre-existing, out-of-scope). `CodexAdapter` renders 17-18 string assets, none hooks.
- #4 ❌ **CONFIRMED WRONG:** `grep SOS_KIT_DIR crates/sos-install crates/sos-cli/.../install.rs` = 0 hits. Install has ZERO kit-source resolution — meant to run standalone in arbitrary target repo (unlike new/adopt/sync which require SOS_KIT_DIR). Mirroring `new.rs` env-copy would silently break that.
- #5 ✅ `hooks/pre-commit`+`pre-push` exist, executable, in kit source.
- #6 ✅ `tests/install.rs:348` seeds `op("hooks/pre-commit",...)` into synthetic Plan → never exercises real (absent) render path.

**Objections (Tầng 1 only):** None blocking. Anchor #4's wrongness is exactly what Constraint 1 punts to Worker (route-decision authority) — self-resolved, no Architect respond turn needed.

**Status:** ✅ ACCEPTED V1 — route corrected below (within Constraint 1)

### Turn 1 — Architect Response
- Anchor #4 route correction ACCEPTED into consensus (Constraint 1 authority) — see Final consensus.

**Status:** ✅ RESOLVED (no version bump — route was already Worker's to decide)

### Final consensus
- Phiếu version: V1 (accepted; route decided per Constraint 1)
- Total turns: 1 (Worker accepted, route self-resolved)
- Approved by Chủ nhà: 2026-07-23 — self-approved by Quản đốc under delegated sprint authority (P076→P081, self-approve in-scope; Guarded/Tầng-1, no owner-decision, no Codex run required).
- **RENDER-ROUTE DECIDED (overrides Task 1/2 `new.rs`-copy prose):** EMBEDDED-ARTIFACT, engine-side, compile-time `include_str!` of `hooks/pre-commit`+`pre-push` INTO the `sos-install` crate, rendered adapter-agnostically inside `engine::apply()` success path RIGHT BEFORE `arm_git_hooks()` (call-site already runtime-agnostic `engine.rs:279-286`). **NOT** SOS_KIT_DIR-copy (doesn't exist in install), **NOT** per-adapter-plan (ClaudeAdapter is a stub). Symmetric for both runtimes trivially (engine path identical). Note claude-stub asymmetry as pre-existing/out-of-scope in Discovery.

---

## Nhiệm vụ

### Task 1: Verify render gap — cả 2 runtime + new.rs pattern (verify-before-impl)

**File (read-only reference):** `crates/sos-cli/src/commands/new.rs` (~L315-322), `crates/sos-adapter-codex/src/lib.rs`, `crates/sos-adapter-claude/src/lib.rs`, `crates/sos-install/src/engine.rs`

**Tìm:**
- `new.rs` copy-hook pattern: cách resolve kit-source root, copy fn, `chmod_x` fn (Anchor #1, #4).
- CẢ 2 adapter `plan()`: có render `hooks/` không (Anchor #3). Ghi rõ asymmetry nếu có.
- `arm_git_hooks()` skip-silent branch khi hook absent (Anchor #2).

**Lưu ý:** Đây là gate cho quyết định render-route (Constraint 1). Ghi enumerate vào Discovery. Nếu asymmetry claude-vs-codex → đó là ground-truth root cause.

### Task 2: RENDER hook scripts vào target TRƯỚC khi arm

**File:** `crates/sos-install/src/engine.rs` `[needs Worker verify — chèn trước bước arm `core.hooksPath`]` (hoặc adapter plan nếu Worker chọn plan-route)

**Thêm:** routine copy `hooks/pre-commit` + `hooks/pre-push` từ kit source vào target repo root, mirror `new.rs:315-322`:
1. Resolve kit-source hook path (cùng cơ chế `new.rs` Anchor #1/#4).
2. Copy `hooks/pre-commit` (+ `hooks/pre-push` nếu Anchor #5 confirm) vào target `hooks/`.
3. `chmod_x` (`#[cfg(unix)]` set-executable — Windows no-op/skip).
4. **Non-clobber:** nếu target đã có `hooks/pre-commit` KHÁC kit source (user tùy biến) → theo F09 semantics đã có, KHÔNG đè lặng (round-3 A5 behavior). Worker confirm F09 guard đã cover hay cần thêm.
5. Chạy cho **CẢ** `--runtime claude` VÀ `--runtime codex` — nếu render nằm trong engine core-path (không rẽ runtime) thì symmetric tự động.

**Lưu ý:** thứ tự BẮT BUỘC render-TRƯỚC-arm. Nếu render fail (kit source absent) → xem Task 3 (harden arm refuse).

### Task 3: HARDEN `arm_git_hooks()` — never arm-empty

**File:** `crates/sos-install/src/engine.rs` — `arm_git_hooks()` `[needs Worker verify]`

**Thay đổi:** trước khi `git config --local core.hooksPath hooks`, assert `hooks/pre-commit` (+ `pre-push`) TỒN TẠI + executable trong target. Nếu absent:
- **KHÔNG set `core.hooksPath`** (never false-arm), VÀ
- **warn-loud** (stderr rõ: "hook scripts not rendered — refusing to arm empty hooksPath; boundary NOT active") hoặc return Err tùy Worker (Constraint 2 fail-loud).

**Lưu ý:** đây là backstop cho trường hợp Task 2 render fail vì lý do bất ngờ. Fail-loud > false-armed. KHÔNG revert F09 guard (A5 PASS).

### Task 4: install-smoke test — fresh git repo KHÔNG seed hooks (oracle)

**File:** install-smoke test `[needs Worker verify vị trí — `crates/sos-install/tests/` reuse `TempFixture`, thêm case fresh-no-seed]`

**Lưu ý:** test PHẢI dựng **fresh git repo KHÔNG có `hooks/` sẵn** (KHÁC P078f fixture — không seed hook file) rồi assert END-TO-END. Chi tiết ở Nghiệm thu. Đây là bài học structural-oracle-gap: P078f seed hook trong fixture nên bỏ lọt lỗi.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-install/src/engine.rs` | Task 2: render hook scripts trước arm; Task 3: harden `arm_git_hooks()` never-arm-empty |
| adapter plan (`crates/sos-adapter-{claude,codex}/src/lib.rs`) | CHỈ nếu Worker chọn plan-route cho hook artifact (Constraint 1) — plan-only, KHÔNG touch guard logic |
| install-smoke test file | Task 4: fresh-no-seed git repo assert render + arm + `.env` block end-to-end + refuse-when-absent + negative |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-cli/src/commands/new.rs` | Reference-only — copy-hook pattern L315-322 để port |
| `scripts/install-hooks.sh` | Reference-only — KHÔNG delete/rewrite |
| `crates/sos-adapter-codex/**` guard | KHÔNG touch — P078h surface (actor-check) |
| F09 hijack-guard (P078f) | Giữ nguyên non-clobber (round-3 A5 PASS) |

---

## Luật chơi (Constraints)

1. **Render-route = Worker decides at EXECUTE** — copy trong engine `arm_git_hooks()` hay adapter plan artifact. Đọc `new.rs` pattern (Task 1) rồi chọn. Ưu tiên nơi đảm bảo files có mặt TRƯỚC arm + symmetric cả 2 runtime. Ghi lý do trong Discovery.
2. **Fail-loud > false-armed** — hook absent → KHÔNG set `core.hooksPath` (warn-loud hoặc Err). Never arm-empty.
3. **Render-TRƯỚC-arm** — thứ tự cứng: files phải có mặt trước khi trỏ `core.hooksPath`.
4. **Symmetric** — render + arm chạy cho CẢ claude VÀ codex; verify không có early-return bỏ sót runtime (root-cause round-3: codex plan thiếu hooks).
5. **Non-clobber giữ F09** — KHÔNG đè hook user tùy biến; round-3 A5 PASS phải vẫn PASS.
6. **Windows-portable** — `chmod +x` qua Rust-native `#[cfg(unix)]` set-executable (Windows skip); copy qua std fs (không shell-out).

---

## Nghiệm thu

### Automated
- [ ] `cargo check` clean
- [ ] `cargo test` pass
- [ ] **install-smoke (oracle `[oracle: install-smoke fresh-no-seed temp-git — end-to-end block]`)** — dựng **fresh git repo KHÔNG seed `hooks/`** (khác P078f fixture), cho MỖI runtime (claude + codex), assert:
  - [ ] sau `sos install`: `hooks/pre-commit` + `hooks/pre-push` TỒN TẠI trong target + executable (`#[cfg(unix)]`)
  - [ ] nội dung hook khớp kit source
  - [ ] `git config --local core.hooksPath` == `hooks`
  - [ ] **commit thật một `.env`** trong temp repo → **bị chặn** (exit ≠ 0) — hook THẬT SỰ chạy+chặn, không chỉ "file tồn tại". *(Trade-off: nếu chạy full pre-commit chain quá nặng trong unit test không khả thi, tối thiểu assert file-tồn-tại+executable+nội-dung-khớp-kit-source, ghi rõ trade-off trong Discovery. **Ưu tiên end-to-end block nếu khả thi.**)*
  - [ ] **refuse/warn-loud:** ép hook source absent → `arm_git_hooks()` KHÔNG set `core.hooksPath` (hoặc loud warn / Err), KHÔNG false-arm
  - [ ] **non-clobber (F09 regression):** seed foreign `core.hooksPath=custom-dir` trước install → non-TTY install KHÔNG đè (round-3 A5)
  - [ ] **negative-test:** guard THẬT SỰ chặn — không always-green (feed `.env`→assert block; feed clean file→assert pass)

### Manual Testing
- [ ] `sos install --runtime claude` trong fresh git repo → `hooks/` render + `core.hooksPath=hooks`; commit `.env` bị chặn
- [ ] `sos install --runtime codex` → cùng kết quả (symmetric — đây là runtime FAIL ở round-3)

### Regression
- [ ] Re-install trong repo đã có kit → hook chain không hỏng, F09 non-clobber giữ
- [ ] P078f behavior còn lại (F09 guard, `.bak` rename, non-git warn-skip) không regress

### Docs Gate (Tầng 1 — security surface AUTO Tầng 1)
- [ ] `CHANGELOG.md` — entry P078g (install renders + arms REAL hooks; fix P078f arm-empty gap)
- [ ] `SECURITY.md` — sửa story: install giờ **render+arm** hook scripts (không còn arm-rỗng con trỏ); Git backstop THẬT SỰ active sau install
- [ ] `adapters/codex/CAPABILITY.md` — honest-MISSING update: backstop giờ THẬT SỰ armed (không chỉ `core.hooksPath` set vào dir rỗng); confirm câu chữ khớp round-3→round-4 reality
- [ ] `adapters/claude/README.md` hoặc MAPPING — nếu ref hook-render/install, sync (verify — có thể N/A; nếu Anchor #3 lộ claude đã render thì note asymmetry)
- [ ] `docs/BACKLOG.md` — active-sprint mark **P078g DONE** + resume pointer → P079 round-4
- [ ] `docs/discoveries/P078g.md` (xem dưới)

### Discovery Report
- [ ] Write `docs/discoveries/P078g.md`:
  - Task-0 anchor CORRECT/WRONG (file:line) — đặc biệt Anchor #3 (claude-vs-codex render asymmetry root-cause)
  - **Render-route quyết định + lý do** (Constraint 1: engine copy vs adapter plan artifact)
  - `hooks/pre-push` tồn tại trong kit source hay không (Anchor #5)
  - End-to-end block test đạt được hay fallback file-assert (Nghiệm thu trade-off)
  - Edge cases / limitations (Windows chmod no-op, refuse-path, non-clobber)
  - Tầng 1 docs updated: <list> (hoặc "N/A" explicit)
  - Tier escalations (write "None")
  - Reference: fix cho lỗ P078f (arm-empty) — link `docs/adapters/P079-ROUND3-FINDINGS-2026-07-23.md` Test A2/A3/A4
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`
