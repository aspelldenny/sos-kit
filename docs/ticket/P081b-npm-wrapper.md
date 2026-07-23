# PHIẾU P081b: Distribution Stage 2 — npm wrapper (1 lệnh trọn bộ)

---

> **Loại:** Feature (distribution — supply-chain surface)
> **Ưu tiên:** P1
> **Tầng:** 1 — npm package chạy `postinstall` auto-exec public + kéo full toolset. Sai thì user cài toolchain giả / half-install → KHÔNG-đảo. AUTO Tầng 1.
> **Lane:** Guarded — supply-chain surface, Debate đầy đủ, no-cap.
> **Ảnh hưởng:** `package.json` (mới), `scripts/npm-postinstall.sh` (mới), `bin/sos-npm` wrapper (mới hoặc trỏ `bin/sos.sh`), `INSTALL.md`, `README.md`, `SECURITY.md`, `.sos-trust-baseline`.
> **Dependency:** P081 Stage 1 (release `v0.1.0` LIVE + e2e verified — BACKLOG Park entry). Đây là "cái cuối" (Sếp).

---

## Context

### Vấn đề hiện tại
P081 Stage 1 shipped: release `v0.1.0` có `sos-<triple>`+`.sha256`, `install.sh` fetch 10 sister tool + `sos-bin` sidecar + wrapper (fail-CLOSED). Nhưng entry-point vẫn là `curl|sh`. Sếp muốn npm = "cái cuối": `npm install <pkg>` → có TRỌN bộ (sos-bin + 10 sister tool doctor/ship/docs-gate/inv-gate... + wrapper), trải nghiệm 1 lệnh. Chưa có `package.json` nào trong repo (Glob = 0).

### Giải pháp (THIN — 1 nguồn sự thật install)
npm package **postinstall tải + chạy chính `install.sh` pinned tag `v0.1.0`** (KHÔNG `main` — supply-chain: pin tag + pin sha256 của `install.sh` trong package, verify TRƯỚC khi chạy, fail-CLOSED). KHÔNG fork logic install sang JS → sister tools đi kèm tự nhiên, `install.sh` là single-source. `bin` field trỏ wrapper `sos`. Package version == Cargo version == tag (F13 mở rộng).

**Decision (Architect chốt — CHALLENGE-able):** **(a) postinstall auto-run full install** — đúng ý Sếp "1 lệnh trọn bộ". Trade-off ghi rõ: npm ecosystem dị ứng postinstall nặng (corporate proxy chặn network trong postinstall; CI `--ignore-scripts`). Mitigation: postinstall in rõ đang làm gì + honor `--ignore-scripts` (nếu skip → in lệnh `npx sos-kit-setup` explicit để chạy tay = fallback (b) tự nhiên). Vậy (a) là default, (b) là fallback khi script bị skip — không loại trừ nhau.

### Scope
- CHỈ package.json + postinstall + wrapper + docs. **KHÔNG publish npm registry** (outward-facing — Sếp/Quản đốc bắn sau như tag v0.1.0).
- KHÔNG fork install logic sang JS. KHÔNG sửa `install.sh` behavior (chỉ consume nó). KHÔNG sửa `crates/**/src`.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Chưa có `package.json` nào trong repo | `Glob **/package.json` | ✅ `[verified]` — Glob = 0 file |
| 2 | `install.sh` fetch 6 required + 4 optional + `sos-bin` sidecar, fail-CLOSED, tự đủ (không cần cargo) | đọc `install.sh` | ✅ `[verified]` — `install.sh:40` BINARIES(6), `:48` OPTIONAL(4), `:150-154` `fetch_bin "sos" required "sos-kit" "sos-bin"` fail-CLOSED, `:168-177` wrapper export `SOS_RUST_BIN` |
| 3 | `install.sh` env override: `SOS_KIT_DIR` + `SOS_BIN_DIR` — dùng để cô lập prefix khi test | đọc `install.sh` | ✅ `[verified]` — `install.sh:18-19,24-25` `SOS_KIT_DIR`/`SOS_BIN_DIR` |
| 4 | Release asset naming `sos-<triple>`+`.sha256`; chỉ mac-arm64 + linux-x64 (KHÔNG Windows) | đọc `install.sh` | ✅ `[verified]` — `install.sh:53-54` 2 triple; `:55-56` win triple resolve nhưng release KHÔNG build Win → fetch fail-closed trên Win (đã note INSTALL.md) |
| 5 | `sos tools` Rust CLI CHỈ có `status`, KHÔNG `tools install` | Quản đốc verified `main.rs:113-118` | ✅ `[verified per Quản đốc]` — sister-tool fetch chỉ ở `install.sh`, KHÔNG trong `sos` binary. npm PHẢI qua install.sh, không thể gọi `sos tools install` |
| 6 | Package name `@aspelldenny/sos-kit` (hoặc `sos-kit`) available trên npm registry | `npm view @aspelldenny/sos-kit` / `npm view sos-kit` | ⏳ `[needs Worker verify]` — Architect không chạy được npm; Worker check availability, nếu taken → đề xuất tên (khuyến nghị scoped `@aspelldenny/sos-kit` giảm nguy cơ collision) |
| 7 | Tag `v0.1.0` release LIVE + `install.sh` tại tag đó có route-A sos-bin | `gh release view v0.1.0 --repo aspelldenny/sos-kit`; `git show v0.1.0:install.sh \| grep sos-bin` | ⏳ `[needs Worker verify]` — pin postinstall vào TAG (không main); Worker xác nhận install.sh@v0.1.0 đã có sos-bin (Stage 1 merge trước hay sau tag?) |
| 8 | `install.sh` trong `.sos-trust-baseline` (P073) — npm-postinstall.sh mới cũng là auto-exec → thêm baseline | `grep -n "install.sh" .sos-trust-baseline scripts/trust-gate.sh` | ✅ `[verified]` (install.sh) — `.sos-trust-baseline:6` + `scripts/trust-gate.sh:44`; new `scripts/npm-postinstall.sh` PHẢI add vào baseline surface |

### Pre-phiếu snapshot
Theo TICKET_TEMPLATE (Worker auto first-step trong worktree).

---

## Debate Log

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge
**Anchor verification:**
- #6: `npm view @aspelldenny/sos-kit` → 404 (available). `npm view sos-kit` → also 404 (available, but unscoped riskier for collision later). **Chốt: `@aspelldenny/sos-kit`** (scoped, per Architect recommendation) — publish sau này cần `--access public` (ghi vào Task 5/BACKLOG note).
- #7 (blocking-check per instruction): `git show v0.1.0:install.sh | grep sos-bin` → **4 hits** (`fetch_bin "sos" required "sos-kit" "sos-bin"` line 150, sidecar comment line 89, `SOS_RUST_BIN` export line 173, final echo line 189) → **route-A sos-bin đã có tại tag v0.1.0, KHÔNG cần đổi pin.** Đồng thời `git diff v0.1.0 main -- install.sh` = NO DIFF → main hiện tại (`65bc6c6`) và tag v0.1.0 identical trên file này, không có drift risk. `sha256` của `install.sh@v0.1.0` computed: `63813f0dded8afb5f3ec2a3ad2abe9f02c757f9a19f1bf0ac3379c951f109a9a` — sẽ dùng làm pinned hash cho `scripts/install-sh.sha256` (Task 2).
- Version sync: `crates/sos-cli/Cargo.toml` = `0.1.0`, tag `v0.1.0` → khớp `package.json` version `0.1.0` per plan.

**No Tầng 1 objections.** Both `[needs Worker verify]` anchors resolved cleanly, no design change needed. Proceeding to EXECUTE with decision (a) auto-postinstall + (b) fallback as drafted.

**Status:** ✅ WORKER ACCEPTED — no challenges, ready for EXECUTE

### Final consensus
- Phiếu version: V<N>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1 — `package.json` (mới, THIN)
**File:** `package.json` (mới — anchor #1)
- `name`: `@aspelldenny/sos-kit` (anchor #6 — Worker confirm available; taken → propose). `version`: **== tag == Cargo `sos-cli` version** (`0.1.0`; F13 mở rộng — Worker verify khớp `grep '^version' crates/sos-cli/Cargo.toml`).
- `bin`: `{ "sos": "bin/sos-npm" }` (wrapper trỏ `$BIN_DIR/sos` do postinstall tạo — xem Task 3).
- `scripts.postinstall`: `"sh scripts/npm-postinstall.sh"`.
- `os`: `["darwin","linux"]` (anchor #4 — KHÔNG Windows; npm cảnh báo/skip trên Win đúng reality install.sh). `files`: whitelist chỉ `scripts/npm-postinstall.sh` + `bin/sos-npm` + `install.sh`(pinned copy? — xem Task 2 decision).
- **Lưu ý:** KHÔNG `dependencies` JS (thin, zero npm dep). KHÔNG `preinstall`.

### Task 2 — `scripts/npm-postinstall.sh` (mới — pin tag + verify install.sh)
**File:** `scripts/npm-postinstall.sh` (mới)
- Honor `--ignore-scripts`: nếu npm không chạy postinstall thì no-op tự nhiên; script chạy = in rõ "sos-kit: fetching full toolset via install.sh@v0.1.0".
- **Pin tag:** tải `install.sh` từ `https://raw.githubusercontent.com/aspelldenny/sos-kit/v0.1.0/install.sh` (TAG, KHÔNG main — supply-chain).
- **Verify install.sh sha256 TRƯỚC khi chạy (fail-CLOSED):** package ship 1 file `scripts/install-sh.sha256` (pinned hash của install.sh@v0.1.0); postinstall recompute sha256 file tải về, compare, **mismatch → exit≠0 abort** (KHÔNG chạy). Nguồn hash = Worker compute từ `git show v0.1.0:install.sh` (anchor #7).
- Chạy `sh install.sh` (env passthrough `SOS_KIT_DIR`/`SOS_BIN_DIR` nếu user set — cho phép prefix cô lập).
- Fail → in lệnh fallback tay `npx sos-kit-setup` (hoặc `sh scripts/npm-postinstall.sh`) + non-zero exit (postinstall fail = npm install fail, đúng fail-CLOSED "1 lệnh trọn bộ").
- **Lưu ý:** KHÔNG fork bất kỳ fetch/verify logic — chỉ orchestrate: verify-then-exec install.sh. install.sh tự lo 10 tool + sos-bin + wrapper.

### Task 3 — `bin/sos-npm` wrapper (mới)
**File:** `bin/sos-npm` (mới)
- npm `bin` link: khi user gọi `sos` qua npm PATH → exec `$BIN_DIR/sos` (wrapper install.sh đã tạo, đã export `SOS_RUST_BIN`). Nếu `$BIN_DIR/sos` chưa tồn tại (postinstall skipped/failed) → in hướng dẫn chạy postinstall tay + exit≠0.
- **Lưu ý:** KHÔNG duplicate dispatch — chỉ delegate sang wrapper canonical. Giữ 1 nguồn (route A).

### Task 4 — Docs + security (DOCS GATE Tầng 1)
- `INSTALL.md` — thêm npm path song song curl|sh: `npm install -g @aspelldenny/sos-kit` (1 lệnh trọn bộ). Note macOS/Linux only + `--ignore-scripts` fallback.
- `README.md` — install section thêm npm option.
- `SECURITY.md` — npm supply-chain surface: pin-tag + pin-install.sh-sha256 + postinstall fail-CLOSED threat model.
- `.sos-trust-baseline` — thêm `scripts/npm-postinstall.sh` (+`bin/sos-npm`) vào auto-exec surface → `scripts/trust-gate.sh rebaseline` post-review (anchor #8).
- `CHANGELOG.md` — entry P081b.

### Task 5 — Publish PARK
- Ghi BACKLOG: `npm publish` = outward-facing, Sếp/Quản đốc bắn tay sau (như tag v0.1.0). KHÔNG publish trong phiếu này.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `package.json` | MỚI — thin, bin+postinstall, version==tag |
| `scripts/npm-postinstall.sh` | MỚI — pin tag v0.1.0 + verify install.sh sha256 fail-CLOSED + exec |
| `scripts/install-sh.sha256` | MỚI — pinned hash install.sh@v0.1.0 |
| `bin/sos-npm` | MỚI — delegate sang `$BIN_DIR/sos` wrapper |
| `INSTALL.md`, `README.md`, `SECURITY.md`, `.sos-trust-baseline` | Task 4: npm path + threat model + rebaseline |
| `CHANGELOG.md` | entry P081b |
| `docs/BACKLOG.md` | Task 5: Park npm publish + resume pointer |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `install.sh` | KHÔNG đổi behavior — postinstall CONSUME nó (pinned tag); route-A sos-bin + 10-tool fetch không vỡ |
| `bin/sos.sh` + `crates/**/src` | KHÔNG đụng — 1 nguồn dispatch |

---

## Luật chơi (Constraints)

1. **THIN — 1 nguồn install.** KHÔNG fork fetch/verify logic sang JS; postinstall chỉ verify-then-exec `install.sh`. install.sh = single source (10 tool + sos-bin + wrapper).
2. **Pin TAG, KHÔNG main.** postinstall tải install.sh@`v0.1.0` + verify pinned sha256 TRƯỚC exec, fail-CLOSED (mismatch → abort, không chạy).
3. **Version sync (F13 mở rộng):** `package.json` version == `crates/sos-cli/Cargo.toml` version == git tag. Worker grep verify.
4. **KHÔNG publish npm registry** — chỉ `npm pack` + local install test. Publish = Sếp/Quản đốc sau.
5. **Auto-exec surface:** `scripts/npm-postinstall.sh` + `bin/sos-npm` → thêm `.sos-trust-baseline` + rebaseline post-review.
6. **Honor `--ignore-scripts`** — skip = no-op + in lệnh setup tay (fallback b); không silently half-install.
7. KHÔNG sửa `install.sh`/`bin/sos.sh`/`crates/**/src`.

---

## Nghiệm thu

### Automated (đo được — prefix cô lập)
- [ ] `npm pack` tạo tarball; `npm install -g` từ tarball vào **prefix cô lập** (`npm install --prefix /tmp/sos-npm-test ...` HOẶC `SOS_KIT_DIR`/`SOS_BIN_DIR` trỏ tmp).
- [ ] Sau install: **đủ 10 tool** (6 required + 4 optional tùy availability) + `sos-bin` + wrapper `sos` trong prefix.
- [ ] `sos tools status` → **exit 0** (toolset đủ).
- [ ] `sos --version` → in đúng version = tag (qua wrapper SOS_RUST_BIN).
- [ ] `package.json` version == Cargo `sos-cli` version == `v0.1.0` (F13).

### Manual Testing
- [ ] **Checksum-tamper fixture:** sửa 1 byte `scripts/install-sh.sha256` (hoặc mock install.sh tải về khác hash) → postinstall **abort exit≠0**, KHÔNG chạy install.sh (fail-CLOSED).
- [ ] `npm install --ignore-scripts` → no-op sạch + in lệnh setup tay (không half-install).
- [ ] `sos` qua npm PATH khi postinstall skipped → in hướng dẫn + exit≠0 (không dispatch nửa vời).
- [ ] anchor #6 npm name available (hoặc tên thay thế đã chốt); #7 install.sh@v0.1.0 có sos-bin.

### Regression
- [ ] `install.sh` (curl|sh path) không vỡ — postinstall chỉ consume, không sửa.
- [ ] trust-gate pass sau rebaseline.

### PASS/FAIL rule
- [ ] **PASS** = npm pack→local install→10 tool+sos-bin+wrapper, `sos tools status` exit 0, `sos --version` đúng, tamper abort. → Stage 2 DONE; publish = Sếp bắn sau.
- [ ] **FAIL** = mở gap ticket (format P080x), paste Quản đốc. KHÔNG publish npm khi chưa xanh.

### Docs Gate (Tầng 1 — supply-chain + install command)
- [ ] `SECURITY.md` — npm surface + pin-tag + pin-sha256 threat model.
- [ ] `.sos-trust-baseline` rebaselined (npm-postinstall.sh + bin/sos-npm).
- [ ] `INSTALL.md`/`README.md` — npm path khớp thật (macOS/Linux only note).
- [ ] `CHANGELOG.md` — P081b (sync version).

### Discovery Report
- [ ] `docs/discoveries/P081b.md` — anchor #1-8 CORRECT/WRONG (file:line), npm name chốt, decision (a)-auto vs (b)-fallback rationale, tamper-abort proof, prefix-isolated install result (tool count), Windows-skip note.
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
- [ ] Tick BACKLOG un-park P081b + Park npm-publish + resume pointer.
