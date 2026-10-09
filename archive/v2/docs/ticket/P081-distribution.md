# PHIẾU P081: Distribution — Stage 1 release pipeline + checksum + curl|sh

---

> **Loại:** Feature (release infra + auto-exec surface)
> **Ưu tiên:** P1
> **Tầng:** 1 — auto-exec surface public (install script + prebuilt binary phân phối). Sai thì supply-chain leak / user cài binary giả → KHÔNG-đảo. AUTO Tầng 1.
> **Lane:** Guarded — release infra + security surface, Debate đầy đủ, no-cap.
> **Ảnh hưởng:** `.github/workflows/release.yml` (mới), `crates/sos-cli/Cargo.toml`, `Cargo.lock`, `tool-manifest.toml`, `install.sh`, `CHANGELOG.md`, `SECURITY.md`, `INSTALL.md`/`docs/SETUP.md`/`README.md`, `.sos-trust-baseline`.
> **Dependency:** P080 (DONE round-2 + Task 3 real-codex PASS, merged `821a102`). Stage 2 (npm/pnpm wrapper + native plugins) GATED sau Stage 1 chạy thật ≥1 release — chỉ Park ở phiếu này.

---

## Context

### Vấn đề hiện tại
Dogfood dual-runtime xanh (P080). Chưa có đường phân phối: repo **KHÔNG có `.github/workflows/`** (Glob = 0 file — không release pipeline nào), `tool-manifest.toml` checksum toàn placeholder `TODO-sha256-<tool>-P081` (`:47-50`...), `sos-cli` version cứng `0.1.0` chưa từng release-tag. Muốn user cài bằng 1 lệnh + verify integrity thì cần: tag `v*` → build binary per-platform → GitHub Release + `.sha256` → `install.sh` tải + verify.

### Giải pháp — Stage 1 (phiếu này)
Release pipeline GitHub Actions: tag `v*` → build `sos` per-platform (macOS arm64 tối thiểu; thêm x86_64-linux nếu runner build được — **BUILD-only, KHÔNG claim test Linux**) → GitHub Release + `.sha256` per asset. Bump version đồng bộ `Cargo.toml`+`Cargo.lock`+`CHANGELOG` (rule F13). Fill real checksum vào `tool-manifest.toml` (FULL fill — cả 10 sister tool đã release, xem O1.2). `install.sh` tải `sos` binary + sha256 vào sidecar `sos-bin` + wrapper export `SOS_RUST_BIN` (route A, xem O1.1). Nghiệm thu = chạy release thật (hoặc dry-run) + tải về verify checksum + `sos --version` đúng.

### Scope
- CHỈ Stage 1 (pipeline + checksum + curl|sh + docs). **Stage 2 = Park** (Task cuối).
- KHÔNG sửa `crates/**/src` logic (chỉ version bump trong Cargo.toml — config, không phải src).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Repo CHƯA có CI/release workflow — `release.yml` phải tạo mới | `ls .github/workflows/` | ✅ `[verified]` — Glob `.github/**` = 0 file, không có `.github/` |
| 2 | `tool-manifest.toml` checksum = placeholder `TODO-sha256-<tool>-P081`, 3 triple/tool | đọc `tool-manifest.toml` | ✅ `[verified]` — `tool-manifest.toml:47-50` (doctor) + tương tự 10 tool; triple `aarch64-apple-darwin`/`x86_64-unknown-linux-gnu`/`x86_64-pc-windows-msvc` (`:16-18`) |
| 3 | `sos --version` phát từ `sos-cli` version = `0.1.0` | đọc `crates/sos-cli/Cargo.toml` | ✅ `[verified]` — `crates/sos-cli/Cargo.toml:3` `version = "0.1.0"`, `[[bin]] name = "sos"` `:8-9` |
| 4 | Workspace root KHÔNG có `[workspace.package] version` — version per-crate | đọc `Cargo.toml` | ✅ `[verified]` — `Cargo.toml:1-16` chỉ có `[workspace]`+deps, không version chung. Bump = sửa `crates/sos-cli/Cargo.toml` |
| 5 | Fleet `release.yml` (draft/publish + sha256) sống ở sister repos — reuse pattern | Worker read `~/ship/.github/workflows/release.yml` | ✅ `[verified]` — `~/ship/.github/workflows/release.yml`: trigger `tag v*`, matrix macos-14/ubuntu-22.04/windows-2022, `cargo build --release --target`, sha256sum/shasum `.sha256` companion, `softprops/action-gh-release@v3` draft + `publish` job flip draft→published. Header (`release.yml:1-4`): asset naming contract `<bin>-<target-triple>[.exe]` = thứ install.sh consume. Copy near-verbatim, `BIN=ship`→`-p sos-cli`, 2-target (drop windows cho Stage 1). |
| 6 | `install.sh` CHƯA tải prebuilt `sos` binary (bin/sos.sh launcher là bash) | `grep -n "sos\|sha256\|BINARIES" install.sh` | ✅ `[verified]` — `install.sh:40` BINARIES (6, no `sos`), `:48` OPTIONAL (4); `install.sh:153-158` chỉ ghi 2-line bash wrapper `$BIN_DIR/sos` = `exec bash .../bin/sos.sh`, KHÔNG fetch Rust binary. |
| 7 | `install.sh` là auto-exec surface trong `.sos-trust-baseline` (P073) — sửa → rebaseline | `grep -n "install.sh" .sos-trust-baseline scripts/trust-gate.sh` | ✅ `[verified]` — `.sos-trust-baseline:6` có sha256 cho `install.sh`; `scripts/trust-gate.sh:44` liệt `"install.sh"`. Edit → `scripts/trust-gate.sh rebaseline` post-review. |
| 8 | Release tag axis: binary-version (`v0.1.0`) ≠ doctrine-version (CHANGELOG "v2.3 forge"); chưa có tag/release nào | đọc CHANGELOG + `git tag -l`/`gh release list` | ✅ `[verified]` — `CHANGELOG.md:5` "v2.3 forge"; Worker xác nhận `git tag -l` empty, `gh release list` empty, repo PUBLIC → `v0.1.0` free, no collision. |

### Pre-phiếu snapshot
Theo TICKET_TEMPLATE (Worker auto first-step trong worktree).

---

## Debate Log

**Phiếu version:** V2 (Architect responded Turn 2)

### Turn 1 — Worker Challenge (phiếu V1)
**Anchor verification:** #1-4 ✅ (unchanged) — #5/#6/#7 ✅ upgraded `[needs Worker verify]`→`[verified]` (Task 0 table). #8 ✅ — no git tag / Release / workflow tồn tại, repo PUBLIC, `v0.1.0` free.

**Additional findings:**
- **Finding #2 — Task 3 FULL fill, not partial:** cả 10 sister tool trong `tool-manifest.toml` đã có published Release tại đúng pinned version + `.sha256` companion (`gh release view v0.1.3 --repo aspelldenny/doctor --json assets` xác nhận `<bin>-<triple>[.exe]`+`.sha256`). Task 3/Constraint #4 nên expect 0 TODO còn lại.
- **Finding #3 — doc drift:** `INSTALL.md:16` prose liệt 9 tool, thiếu `inv-gate` (install.sh:40+48 tải 10). Fix 1 dòng khi Task 5 đụng file.
- **Non-issue:** `CARGO_TARGET_DIR` là local shell env, không commit (`.cargo/config.toml` không tồn tại) → CI không leak.

**Objection:**
- [O1.1] Task 4 dest path đụng bash-wrapper: `install.sh:153-158` đã ghi 2-line wrapper vào `$BIN_DIR/sos` (load-bearing cho 7 guidance subcmd Bash). `bin/sos.sh:_sos_rust_bin()` (`:1296-1314`) chỉ resolve `SOS_RUST_BIN` env / `CARGO_TARGET_DIR` / workspace target — KHÔNG nhìn `$BIN_DIR`. Prebuilt binary tải cùng path = đè wrapper hoặc dispatcher lơ. Alt: A (sidecar `$BIN_DIR/sos-bin` + wrapper export `SOS_RUST_BIN`, chỉ đụng install.sh) / B (thêm `$BIN_DIR` vào `_sos_rust_bin()` candidate — robust hơn nhưng đụng dispatch contract P077e, Rule #9). Oracle: grep SOUND cho collision-exists; fix A/B = design → routes Architect.

**Status:** ✅ RESPONDED (Turn 2 below)

### Turn 2 — Architect Response (phiếu V2)
- [O1.1] → **ACCEPT route A.** Rationale: A giữ diff cục bộ trong `install.sh` (auto-exec surface đã trong scope + rebaseline), KHÔNG đụng `bin/sos.sh` dispatch contract (P077e cutover surface — Rule #9 đòi CHALLENGE riêng, ngoài scope Task 4). B robust hơn nhưng mở contract-surface = 1 phiếu riêng nếu cần sau. Precedence phải giữ: **user tự set `SOS_RUST_BIN` PHẢI thắng** — wrapper chỉ export khi env CHƯA set (`: "${SOS_RUST_BIN:=$BIN_DIR/sos-bin}"` rồi `export SOS_RUST_BIN`, KHÔNG hard-override). Task 4 spec sửa tương ứng bên dưới.
- [Finding #2] → **ACCEPT.** Task 3 đổi "scoped/partial" → FULL fill, 0 TODO. Giữ honest-fallback code path phòng 1 triple asset thiếu, nhưng đi vào với expect full.
- [Finding #3] → **ACCEPT.** Task 5 thêm sửa `INSTALL.md:16` prose 9→10 tool (thêm `inv-gate`).

**Status:** ✅ RESPONDED — phiếu bumped to V2 · **APPROVED-FOR-EXECUTE** (Sếp delegation "làm P081 luôn" còn hiệu lực, Quản đốc xác nhận in-scope)

### Final consensus
- Phiếu version: V2
- Approved by Chủ nhà: 2026-07-23 (delegation "làm P081 luôn") — code execution may begin

---

## Nhiệm vụ

### Task 1 — `.github/workflows/release.yml` (tạo mới)
**File:** `.github/workflows/release.yml` (mới — anchor #1)
- Trigger: push tag `v*`.
- Build matrix: `aarch64-apple-darwin` (BẮT BUỘC, macOS runner) + `x86_64-unknown-linux-gnu` (**build-only** nếu ubuntu runner build được — KHÔNG chạy test Linux, KHÔNG claim Linux-tested). **Drop windows-2022 row** khỏi fleet template (Stage 1 không đòi Windows).
- Mỗi target: `cargo build --release -p sos-cli` → rename theo triple (`sos-aarch64-apple-darwin` ...).
- Compute `.sha256` per asset (macOS `shasum -a 256` / Linux `sha256sum`).
- Tạo GitHub Release cho tag, attach binary + `.sha256` companion per asset.
- **Lưu ý:** copy near-verbatim từ `~/ship/.github/workflows/release.yml` (anchor #5 verified), swap `BIN=ship`→`-p sos-cli`, giữ asset naming contract `<bin>-<triple>[.exe]` (= thứ install.sh consume). Giữ draft→publish 2-job pattern.

### Task 2 — Version sync (rule F13) + release tag
**File:** `crates/sos-cli/Cargo.toml` + `Cargo.lock` + `CHANGELOG.md`
- Bump `crates/sos-cli/Cargo.toml:3` version cho release đầu; cập nhật `Cargo.lock` (`cargo build` regen).
- Thêm CHANGELOG entry cho release (dưới header hiện tại).
- **Lưu ý (anchor #8 — DECISION):** tag = `v<sos-cli-version>` = **`v0.1.0`** cho release đầu (khớp `sos --version`, Worker xác nhận free/no-collision). Doctrine-version ("v2.3 forge") là trục KHÁC, KHÔNG dùng làm git tag. F13: CHANGELOG bump PHẢI sync Cargo.toml — Worker verify `grep '^version' crates/sos-cli/Cargo.toml` khớp tag.

### Task 3 — Fill real checksum vào `tool-manifest.toml` (FULL fill)
**File:** `tool-manifest.toml`
- Thay TẤT CẢ `TODO-sha256-<tool>-P081` bằng sha256 thật của asset release đã publish, per triple. **Expect 0 TODO còn lại** — cả 10 sister tool đã có Release+`.sha256` tại pinned version (Finding #2 verified).
- **Honest-fallback:** nếu 1 triple asset cụ thể thiếu (vd advisory-cron Windows = compile_error by design, `:176`) → GIỮ TODO + comment lý do; KHÔNG bịa hash. Worker liệt kê fill-được vs skip trong Discovery.
- **Lưu ý:** checksum của SISTER tool, KHÔNG phải `sos` (self-integrity qua `.sha256` companion Task 1). Fetch từ `gh release view v<pin> --repo aspelldenny/<tool>` → download asset → recompute sha256 → paste.

### Task 4 — `install.sh` tải + verify `sos` binary (route A — sidecar + wrapper export)
**File:** `install.sh` (KHÔNG đụng `bin/sos.sh`)
- Thêm bước fetch prebuilt `sos-<triple>` từ GitHub Release + companion `.sha256`, recompute + compare, **fail-CLOSED** (mismatch/download-fail → abort exit≠0). Đặt binary vào **sidecar `$BIN_DIR/sos-bin`** (KHÔNG `$BIN_DIR/sos` — tránh đè wrapper), `chmod +x`.
- Sửa wrapper-gen (`install.sh:153-158`): trước dòng `exec bash .../bin/sos.sh "$@"`, thêm **precedence-safe export** — chỉ default khi user chưa set:
  ```sh
  : "${SOS_RUST_BIN:=$BIN_DIR/sos-bin}"
  export SOS_RUST_BIN
  ```
  → `_sos_rust_bin()` (`bin/sos.sh:1296-1314`) resolve `SOS_RUST_BIN` đầu tiên, wrapper tự đủ (no cargo/dev machine). **User tự set `SOS_RUST_BIN` vẫn thắng** (`:=` không override).
- Tái dùng sha256-verify helper sẵn có của install.sh (P071) cho `sos-bin`. Giữ sister-tool fetch + wrapper 7-guidance-subcmd không vỡ.
- **Lưu ý:** fail-CLOSED bắt buộc (auto-exec public). Route B (sửa `bin/sos.sh` candidate list) BỊ TỪ CHỐI ở Turn 2 — không đụng dispatch contract trong phiếu này.

### Task 5 — Docs + security surface (DOCS GATE Tầng 1)
- `SECURITY.md` — ghi release/distribution auto-exec surface (binary phân phối + install.sh fetch), threat model integrity.
- `.sos-trust-baseline` — install.sh đổi (anchor #7) → `scripts/trust-gate.sh rebaseline` sau review, commit baseline mới.
- `INSTALL.md` — (a) xác nhận curl|sh (`:13`) khớp release pipeline; (b) **sửa prose `:16` 9→10 tool, thêm `inv-gate`** (Finding #3).
- `docs/SETUP.md` + `README.md` — lệnh cài + verify khớp thật.
- `CHANGELOG.md` — entry P081.

### Task 6 — Stage 2 PARK (không làm ở phiếu này)
- Ghi vào `docs/BACKLOG.md` (Next/Park): npm/pnpm wrapper (thin, downloads binary) + native plugins — **GATED sau Stage 1 chạy thật ≥1 release**. KHÔNG implement ở P081.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `.github/workflows/release.yml` | MỚI — Task 1: tag v* → build 2-target + sha256 + Release |
| `crates/sos-cli/Cargo.toml` + `Cargo.lock` | Task 2: version bump → tag `v0.1.0` |
| `CHANGELOG.md` | Task 2+5: release entry + P081 entry |
| `tool-manifest.toml` | Task 3: FULL fill checksum thật (0 TODO expected) |
| `install.sh` | Task 4: fetch `sos-bin` sidecar + verify sha256 (fail-CLOSED) + wrapper export SOS_RUST_BIN |
| `SECURITY.md`, `INSTALL.md`, `docs/SETUP.md`, `README.md`, `.sos-trust-baseline` | Task 5: docs khớp + prose 9→10 + rebaseline |
| `docs/BACKLOG.md` | Task 6: Park Stage 2 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/**/src/**` | KHÔNG đụng logic — chỉ version trong Cargo.toml |
| `bin/sos.sh` | dispatch không vỡ; `_sos_rust_bin()` (`:1296-1314`) resolve `SOS_RUST_BIN` do wrapper export — KHÔNG sửa file này (route A) |

---

## Luật chơi (Constraints)

1. **KHÔNG sửa `crates/**/src`** — chỉ Cargo.toml version bump. **KHÔNG sửa `bin/sos.sh`** (route A giữ dispatch contract nguyên).
2. **Linux = build-only, KHÔNG claim tested.** Discovery ghi "Linux built, not dogfood-tested" (như P080 E2 DEFERRED). macOS arm64 = surface verify thật.
3. **install.sh fail-CLOSED** — auto-exec public; verify fail → abort, không warn-skip. Đổi install.sh → rebaseline `.sos-trust-baseline` + review (P073).
4. **Checksum FULL fill** — cả 10 tool có Release+hash; fill hết, 0 TODO. Chỉ giữ TODO nếu 1 triple asset thật sự thiếu (+comment). KHÔNG bịa hash.
5. **`SOS_RUST_BIN` precedence** — wrapper dùng `:=` default, KHÔNG hard-override; user env thắng.
6. **Stage 2 KHÔNG làm** — chỉ Park. Gate = Stage 1 chạy thật ≥1 release.
7. Rule F13 — Cargo.toml version PHẢI sync CHANGELOG/tag; Worker grep verify khớp.

---

## Nghiệm thu

### Automated / release (đo được)
- [ ] `cargo build --release -p sos-cli` clean; `Cargo.lock` sync.
- [ ] `.github/workflows/release.yml` valid (yaml lint / `act` dry-run HOẶC push tag test `v0.1.0-rc` → workflow chạy tới bước tạo Release + sha256).
- [ ] **Release thật hoặc dry-run:** GitHub Release cho tag có asset `sos-aarch64-apple-darwin` + `.sha256` companion (Linux asset nếu build được).
- [ ] **Tải binary về máy → `shasum -a 256` khớp `.sha256` published** (integrity end-to-end).
- [ ] Tải binary chạy `sos --version` → in đúng version = tag (anchor #3, F13 sync).

### Manual Testing
- [ ] `install.sh` fetch `sos-bin` sidecar + verify sha256 → `sos --version` đúng qua wrapper (SOS_RUST_BIN export). Mismatch giả (sửa 1 byte) → abort exit≠0 (fail-CLOSED).
- [ ] **Precedence:** user `export SOS_RUST_BIN=/custom/sos` trước khi chạy `sos` → wrapper KHÔNG override (dùng /custom/sos).
- [ ] `tool-manifest.toml`: 0 TODO còn lại (trừ triple thiếu có comment); parse đúng.
- [ ] `curl|sh` URL trong INSTALL.md:13 khớp release pipeline; INSTALL.md:16 prose = 10 tool.

### Regression
- [ ] install.sh sister-tool fetch (10 tool) không vỡ; `bin/sos.sh` dispatch + 7 guidance subcmd không vỡ.
- [ ] trust-gate pass sau rebaseline.

### PASS/FAIL rule
- [ ] **PASS** = release asset + sha256 verify end-to-end + `sos --version` đúng + install.sh fail-CLOSED + precedence-safe. → Stage 1 DONE, Stage 2 unblocked (Park → sprint sau).
- [ ] **FAIL** = mở gap ticket (format P080x), ghi + paste Quản đốc. KHÔNG publish release public khi verify chưa xanh.

### Docs Gate (Tầng 1 — auto-exec surface + install command)
- [ ] `SECURITY.md` — distribution surface + threat model.
- [ ] `.sos-trust-baseline` rebaselined (`scripts/trust-gate.sh rebaseline`).
- [ ] `INSTALL.md`/`docs/SETUP.md`/`README.md` — lệnh cài + verify khớp thật (prose 9→10).
- [ ] `CHANGELOG.md` — release entry + P081 (sync Cargo.toml, F13).

### Discovery Report
- [ ] `docs/discoveries/P081.md` — anchor #1-8 CORRECT/WRONG (file:line), fleet release.yml pattern nguồn, install.sh route-A trước/sau, checksum 10-tool fill (list + triple skip nếu có), Linux build-only note, tag `v0.1.0` decision.
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
- [ ] Tick `[P081]` BACKLOG + Park Stage 2 + resume pointer.
