# PHIẾU P077d3: tool-manifest.toml — pin sister-tool version+asset+checksum + `sos tools status` + doctor fail-clear (OA-07)

> **ID:** P077d3 (sub-phiếu CUỐI của P077d — sau d1 adapter-contract + d2 install-engine SHIPPED)
> **Filename:** `docs/ticket/P077d3-tool-manifest.md`
> **Branch:** `feat/P077d3-tool-manifest`

---

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 1 — fill install-engine seam (step-5 tool-resolve) + distribution integrity (checksum verify) + version-pin contract mà cả `sos install` LẪN downstream giả định. Security-adjacent (checksum/verify) → AUTO Tầng 1. Sai thì LAN (install engine consume sai → mọi install-runtime nhận binary sai version).
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/` (`crates/sos-install`, `crates/sos-cli`), `tool-manifest.toml` (kit-root, MỚI). `install.sh` + `bin/sos.sh` KHÔNG đổi (additive).
> **Dependency:** P077d2 (install engine + `resolve_tools()` step-5 stub) — SHIPPED.

---

## Context

### Vấn đề hiện tại

OA-07 (`docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md:278-302`): installer dùng `releases/latest` **unpinned** → version drift không phát hiện được. Evidence trên máy audit: installed `doctor 0.1.1` vs source `0.1.3`; `ship 0.1.0` vs `0.1.1`; `docs-gate 0.1.0` vs `0.1.1`; `inv-gate` **required nhưng absent trên PATH**. Repo contract có thể giả định tool behavior mới nhưng runtime chạy binary cũ — checksum hiện tại (P071/P073) chống corruption/tamper của asset ĐÃ CHỌN, KHÔNG bảo đảm reproducible version selection.

d2 để lại **step-5 tool-resolve = seam stub**: `resolve_tools()` trong `crates/sos-install/src/engine.rs` trả `Vec::new()`, wired đúng transaction position nhưng verify/assert nothing (`docs/discoveries/P077d2.md:30,85-87`). d3 fill nó bằng một manifest pin thật.

### Giải pháp

Thêm `tool-manifest.toml` (committed KIT config — pin `version` + per-platform `asset` + per-platform `checksum` cho 10 sister-tool, đánh dấu required/optional khớp install.sh) + một core `check_tools()` thuần (parse manifest → resolve installed version qua `<tool> --version`/`which` → so drift) surfaced qua **HAI** mặt: `sos tools status` (report + exit-code) và install step-5 tool-resolve gate + doctor fail-clear. **Một cơ chế (core check), hai surface** — không dựng 2 engine drift riêng.

**GIỚI HẠN d3 (KHÔNG kéo vào):** atomic upgrade + previous-version rollback (OA-07 upgrade-direction bullet 4) = **FUTURE (P081 hoặc sau)**. d3 CHỈ pin + status + verify + fail-clear. `sos tools install|upgrade` KHÔNG impl ở d3.

### Phân biệt CỐT LÕI (đừng lẫn — recon)

| File | Phiếu | Bản chất | Nơi | Nội dung |
|---|---|---|---|---|
| `.sos-manifest.toml` | d2 | **generated** artifact-tracking (managed ARTIFACTS đã cài VÀO project) | **project** root | `[[managed]]` owner/hash/rollback per-file |
| `tool-manifest.toml` | **d3** | **committed** config (external sister-tool version pin) | **kit** root | `[[tool]]` version/asset/checksum per external binary |

Hai file KHÁC NHAU hoàn toàn: một cái theo dõi cái ta ĐÃ RẢI RA, một cái pin cái ta TẢI VÀO.

### Scope

- CHỈ sửa/thêm: `tool-manifest.toml` (kit root, MỚI) · `crates/sos-install/src/engine.rs` (fill `resolve_tools()`) · `crates/sos-install/src/` (core parse+check, có thể module mới `tools.rs`) · `crates/sos-cli/src/commands/` (`tools status` subcommand + doctor fail-clear surface) · `crates/sos-cli/src/main.rs` + `commands/mod.rs` (clap wiring) · test fixtures.
- KHÔNG sửa: `install.sh`, `bin/sos.sh` (additive — path mới song song), `.sos-manifest.toml` schema (d2, khác domain), `Adapter` trait / `ManagedManifest` (d1 shape).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | install.sh required = `doctor claude-hooks docs-gate ship advisory-inbox inv-gate` (6); optional = `guard vps doc-rotate advisory-cron` (4) → tool-manifest `required` flag phải khớp | `grep -nE '^(BINARIES\|OPTIONAL_BINARIES)=' install.sh` | ✅ **[verified]** L40 (BINARIES, 6) + L48 (OPTIONAL_BINARIES, 4) — Architect đã Read install.sh |
| 2 | 3 platform target = `aarch64-apple-darwin` / `x86_64-unknown-linux-gnu` / `x86_64-pc-windows-msvc` (win có `.exe`) → asset+checksum keys phải là 3 target-triple này | `grep -nE 'TARGET=' install.sh` | ✅ **[verified]** L52-60 (Darwin-arm64/Linux-x86_64/MINGW→msvc.exe; Intel Mac→arm64 via Rosetta, KHÔNG target riêng) |
| 3 | d2 để lại `resolve_tools()` stub trả `Vec::new()`, marker `// SEAM P077d3`, wired đúng transaction position trong `engine.rs` | `grep -n 'resolve_tools\|SEAM P077d3' crates/sos-install/src/engine.rs` | ⏳ **[needs Worker verify]** — `docs/discoveries/P077d2.md:30,85-87` khẳng định; exact signature/return-type Worker grep |
| 4 | Asset URL pattern hiện tại = `github.com/<owner>/<bin>/releases/latest/download/<bin>-<target>[.exe]` + companion `.sha256` (P071) → manifest thay `latest` bằng pinned tag/version | `grep -n 'releases/latest\|\.sha256' install.sh` | ✅ **[verified]** L88,96-99 (fetch_bin URL + `.sha256` recompute+string-compare) |
| 5 | PORTABILITY spec: step-5 = "Resolve tool manifest, tải đúng version/checksum"; `sos tools status\|install\|upgrade` namespace; `sos khóa version/checksum trong tool-manifest.toml` | `sed -n '111,145p' docs/PORTABILITY_ARCHITECTURE.md` | ✅ **[verified]** L111-137 — step-5 L117, namespace L134, khóa L137 |
| 6 | sos-cli command-wiring pattern = `commands/<name>.rs` + clap variant trong `main.rs` + dispatch trong `commands/mod.rs` (precedent: d2's `commands/install.rs`) | `grep -rn 'Install\|pub fn run' crates/sos-cli/src/commands/mod.rs crates/sos-cli/src/main.rs` | ⏳ **[needs Worker verify]** — `docs/discoveries/P077d2.md:39-40` mô tả; exact enum/dispatch shape Worker grep |
| 7 | `sos doctor` — là subcommand MỚI trong sos-cli, hay dispatch sang external `doctor` binary (PORTABILITY:126 namespace)? Quyết placement của fail-clear surface | `grep -rn 'Doctor\|doctor' crates/sos-cli/src/commands/mod.rs crates/sos-cli/src/main.rs` | ⏳ **[needs Worker verify]** — nếu `sos doctor` subcommand tồn tại → fold tool-check vào; nếu dispatch external → surface fail-clear qua `sos tools status` exit + install step-5 gate, KHÔNG tạo subcommand xung đột |
| 8 | semver-compare khả dụng để so `installed < pinned` (older = fail) | `grep -n 'semver' crates/*/Cargo.toml bootstrap/sos-rs/Cargo.toml` | ⏳ **[needs Worker verify]** — nếu `semver` crate là workspace dep → dùng; nếu KHÔNG → exact-string match (drift = mismatch, không phân older/newer) là fallback, ghi Discovery |

**Escape hatch (escalate, ĐỪNG bịa):**
- **E1 — version-detection non-deterministic:** `<tool> --version` output format KHÁC nhau giữa 10 tool (vd `doctor 0.1.3` vs `docs-gate v0.1.1` vs không có `--version`). Nếu ≥1 required tool KHÔNG parse được version deterministic → **DISCOVERY_REPORT + escalate** (đừng regex-guess format bừa). Ghi rõ tool nào + output thật.
- **E2 — checksum-per-platform không pin được:** nếu chưa có prebuilt release asset cho một (tool × platform) → sha256 thật KHÔNG lấy được. **Verify MECHANISM (schema+parse+verify path) ship + fixture-test bằng checksum synthetic**; cell checksum thật nào thiếu prebuilt → để trống/placeholder + escalate "checksum-fill cần release-asset" (đừng bịa hash). Version-pin (derivable từ source `Cargo.toml`/`CHANGELOG`) vẫn fill được độc lập.

---

## Debate Log

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Anchor verification (recap Task 0):**
- #1 ✅ CORRECT — `install.sh:40` `BINARIES="doctor claude-hooks docs-gate ship advisory-inbox inv-gate"` (6), `install.sh:48` `OPTIONAL_BINARIES="guard vps doc-rotate advisory-cron"` (4). Exact match to phiếu's required/optional split.
- #2 ✅ CORRECT — `install.sh:52-60` confirms 3 target-triple (`aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` w/ `.exe`); Intel Mac routes to arm64/Rosetta, no 4th triple.
- #3 ✅ CORRECT — `crates/sos-install/src/engine.rs:296-302`: `pub fn resolve_tools() -> Vec<String> { Vec::new() }`, comment literally says `SEAM P077d3 (OA-07)`, wired at transaction step 5 per doc comment. Signature is the simplest possible (`() -> Vec<String>`) — Worker notes this will need to change shape (likely `-> Vec<ToolStatus>` or similar) to actually carry drift info per Task 5; that's expected/in-scope, not an objection.
- #4 ✅ CORRECT — `install.sh:88,96-99` confirms `releases/latest/download/...` URL pattern + companion `.sha256` fetch-recompute-compare.
- #5 ✅ CORRECT — Architect-verified doc read, not independently re-checked (docs-only anchor, low risk).
- #6 ✅ CORRECT — `crates/sos-cli/src/main.rs:88` (`Install { ... }` clap variant) + `:123` (`Cmd::Install => commands::install::run(...)`) + `commands/mod.rs` (`pub mod install;`). Precedent is clean and directly copyable for a `Tools` variant.
- #7 ✅ CORRECT, resolved — grepped `crates/sos-cli/src/**` for `doctor`/`Doctor`: **no `sos doctor` subcommand exists anywhere in sos-cli.** Every `doctor` reference is sos-cli *shelling out* to the external `doctor` binary (`Command::new(&doctor_bin)` in `adopt.rs:678`, `new.rs:403`) for `verify-setup`/`validate-map`. This settles anchor #7 cleanly: Task 4's second branch applies — **surface fail-clear via `sos tools status` exit + install step-5 gate, do NOT create a `doctor` subcommand** (that would collide with the external binary's own name/identity). Phiếu's own Task 4 wording already anticipates this branch correctly.
- #8 ✅ CORRECT — grepped all `Cargo.toml` in the workspace (`crates/*/Cargo.toml` + root `bootstrap/sos-rs/Cargo.toml`): **no `semver` crate anywhere.** Fallback confirmed: exact-string version compare (Drift on any mismatch, no older/newer distinction) per phiếu's own §Task 2 note.

**E1 — version-detection determinism (CRITICAL, live-tested):**
Ran `<tool> --version` for all 9 tools present on PATH (`inv-gate` absent — see below):
```
doctor 0.1.1 · docs-gate 0.1.0 · ship 0.1.0 · claude-hooks 0.9.0 ·
advisory-inbox 0.1.0 · guard 0.1.0 · vps 0.1.0 · doc-rotate 0.21.0 · advisory-cron 0.1.0
```
Format is **uniform** across all 9: `<binary-name><space><x.y.z>` — no `v` prefix, no extra text, single space, all valid semver-shaped strings (splitting on whitespace + taking last token parses cleanly). This actually contradicts the phiếu's own worry (`docs-gate v0.1.1` hypothetical) — real output has no `v` prefix. `inv-gate` (required) is **not installed on this machine at all** (`command not found`) — this is the `Missing` verdict case working as designed, not a parse-format problem; it's literally the OA-07 evidence reproduced live. **E1 does NOT fire** — no escalation needed. Caveat: this is an n=9 sample on one dev machine (clap-derived `--version` is likely the shared mechanism across all sos-kit-authored tools, hence the uniformity) — cross-platform/future-tool drift is a residual risk but not a blocker for V1.

**E2 — checksum synthetic-fixture:**
Confirmed honest by design — `tool-manifest.toml` doesn't exist yet (Task 1 creates it), and no per-tool release pipeline currently computes/publishes a sha256 keyed to this new manifest shape (existing `.sha256` companions in `install.sh` are keyed to `releases/latest` assets, not to this pin format). Task 6's synthetic-checksum fixture verifies the **mechanism** (parse/pin/compare/sabotage→fail) independently of real hash values — phiếu already mandates placeholder + Discovery escalation for real values (no fabrication). This is sound and honest.

**required/optional split vs install.sh:** ✅ confirmed exact match (see anchor #1 above, live-grepped not just Architect-claimed).

**Manifest distinction (`tool-manifest.toml` vs `.sos-manifest.toml`):** ✅ clear — different root (kit vs project), different lifecycle (committed config vs generated artifact), different schema (`[[tool]]` version/asset/checksum pin vs `[[managed]]` owner/hash/rollback). `engine.rs`'s current `resolve_tools()` stub and `load_manifest()` (for `.sos-manifest.toml`) are already separate functions — no shared code path to accidentally conflate.

**Objections:** None — no Tầng 1 issues found. All 4 `[needs Worker verify]` anchors resolved cleanly with code citations; both critical escape-hatch risks (E1/E2) investigated live and neither blocks. Phiếu V1 is well-scoped and accurately grounded in current code.

**Status:** ✅ WORKER ACCEPTED V1 — no challenges. Ready for Chủ nhà approval gate.

### Final consensus
- Phiếu version: V<N>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1: `tool-manifest.toml` (kit root, MỚI)

**File:** `tool-manifest.toml` (sos-kit repo root — cạnh `.sos-trust-baseline`; committed config, KHÔNG generated, KHÔNG gitignore).

**Schema (CHỐT):** array-of-tables `[[tool]]`, mỗi tool:

```toml
# Pin external sister-tool version + per-platform asset + checksum (OA-07).
# required = true → thiếu/cũ làm sos doctor + install FAIL. required = false → warn-skip.
# Keys của [tool.asset]/[tool.checksum] = 3 target-triple (khớp install.sh L52-60).

[[tool]]
name     = "doctor"
version  = "0.1.3"          # pinned (thay releases/latest). Fill từ source CHANGELOG/Cargo.toml.
required = true

[tool.asset]
"aarch64-apple-darwin"      = "doctor-aarch64-apple-darwin"
"x86_64-unknown-linux-gnu"  = "doctor-x86_64-unknown-linux-gnu"
"x86_64-pc-windows-msvc"    = "doctor-x86_64-pc-windows-msvc.exe"

[tool.checksum]             # sha256 hex per platform. E2: thiếu prebuilt → placeholder + escalate.
"aarch64-apple-darwin"      = "sha256:<fill-or-TODO>"
"x86_64-unknown-linux-gnu"  = "sha256:<fill-or-TODO>"
"x86_64-pc-windows-msvc"    = "sha256:<fill-or-TODO>"
```

**Nội dung (10 tool, required flag khớp install.sh — anchor #1):**
- `required = true` (6): `doctor`, `claude-hooks`, `docs-gate`, `ship`, `advisory-inbox`, `inv-gate`.
- `required = false` (4): `guard`, `vps`, `doc-rotate`, `advisory-cron`.

**Lưu ý:** version-pin fill từ source (verifiable, độc lập E2). checksum thật cần release-asset — nếu thiếu, placeholder `TODO` + Discovery note (E2), KHÔNG bịa hash. Schema field name (`asset` vs `assets`, `checksum` vs `sha256`) là Tầng-2 — Worker chọn nhất quán, miễn serde round-trip test xanh.

### Task 2: Core `check_tools()` — parse + resolve + drift (thuần, không mutate)

**File:** `crates/sos-install/src/tools.rs` (module MỚI — hoặc nơi Worker thấy khớp crate layout, `[needs Worker verify]`; `sos-install` deps chỉ `sos-core` — giữ direction).

**Thêm:**
1. Serde struct `ToolManifest { tools: Vec<ToolPin> }` + `ToolPin { name, version, required, asset: Map<target,String>, checksum: Map<target,String> }` — TOML deserialize.
2. `check_tools(manifest, platform) -> Vec<ToolStatus>` **thuần**: mỗi tool → resolve installed (chạy `<tool> --version` + `which`/PATH-probe) → parse installed version → verdict enum: `Ok` (installed == pinned) / `Drift{expected,found}` (installed < pinned, older — anchor #8) / `Newer{expected,found}` (installed > pinned, warn) / `Missing` (không trên PATH).
3. Version-compare: semver nếu workspace dep (#8), else exact-string (fallback → Drift on mismatch không phân older/newer). E1: parse fail → `ToolStatus::Unparseable{raw}` + trigger escalate ở caller.

**Lưu ý:** đây là MỘT cơ chế cho cả 3 surface (status/doctor/step-5). Chạy subprocess `<tool> --version` = side-effect I/O nhưng KHÔNG mutate fs — test qua fixture: inject một fake-PATH dir chứa stub script in version giả (KHÔNG phụ thuộc tool thật cài trên máy CI → deterministic + collision-safe như d2 `TempFixture`).

### Task 3: `sos tools status` — report + exit-code

**File:** `crates/sos-cli/src/commands/tools.rs` (MỚI) + wiring `main.rs` clap `Tools { status }` + `commands/mod.rs` dispatch (pattern anchor #6).

**Thêm:** `sos tools status` → đọc `tool-manifest.toml` (resolve path qua SOS_KIT_DIR env — `[needs Worker verify]` cơ chế resolve: env-var vs embedded vs `--manifest` flag; d2's `.sos-manifest.toml` dùng project-root, nhưng tool-manifest = KIT config nên KHÁC) → gọi `check_tools()` → in **table**: `name | required? | expected | installed | verdict`.

**Exit code (CHỐT):**
- `0` = KHÔNG required-drift (mọi required = `Ok`/`Newer`; optional bất kỳ).
- `1` = ≥1 required = `Drift`/`Missing`/`Unparseable`.
- optional `Drift`/`Missing` → in warn line, KHÔNG flip exit (fail-closed reserve cho required).

**Lưu ý:** format khớp OA-07 evidence (doctor: expected 0.1.3, installed 0.1.1, DRIFT). Table wording/màu = Tầng-2.

### Task 4: `sos doctor` fail-clear (required-tool gate)

**File:** placement theo anchor #7 (`[needs Worker verify]`):
- Nếu `sos doctor` subcommand tồn tại trong sos-cli → **fold** tool-check vào (thêm một section gọi `check_tools()`).
- Nếu `sos doctor` dispatch external `doctor` binary → **KHÔNG tạo subcommand xung đột**; surface fail-clear qua (a) `sos tools status` exit + (b) install step-5 gate (Task 5). Ghi Discovery quyết định nào.

**Hành vi fail-clear:** required tool `Missing` HOẶC `Drift` (installed < pinned) → **exit ≠ 0** + message nêu rõ `tool + expected + found` (vd `✗ inv-gate: required, MISSING (expected 0.1.0) — run installer`). Optional thiếu → warn-skip (exit 0). E1 unparseable required → fail + escalate note.

**Lưu ý:** dùng CHUNG `check_tools()` với Task 3 — KHÔNG viết lại drift-logic (một bệnh một cơ chế).

### Task 5: Fill d2 step-5 tool-resolve seam

**File:** `crates/sos-install/src/engine.rs` — `resolve_tools()` (anchor #3, hiện `Vec::new()` stub `// SEAM P077d3`).

**Thay bằng:** consume `tool-manifest.toml` → gọi `check_tools()` → tại install transaction step-5: **verify** required tool version+checksum (thay `releases/latest` unpinned assumption). Required drift/missing → install FAIL rõ + rollback (dùng d2's rollback path, step-7 doctor gate semantics: "lỗi required component làm install fail rõ ràng và rollback" — PORTABILITY:119).

**Lưu ý d3 GIỚI HẠN:** step-5 CHỈ **resolve + verify** (đọc manifest, check version+checksum khớp). **KHÔNG tải/atomic-upgrade/rollback external binary** (= P081 future). Nếu required tool drift → FAIL + báo user chạy installer, KHÔNG tự động fetch. Giữ `resolve_tools()` signature tương thích caller d2 (đừng vỡ transaction position).

### Task 6: Oracle fixtures (hard-fail)

**File:** `crates/sos-install/tests/tools.rs` (MỚI) hoặc mở rộng `tests/install.rs` — dùng collision-safe `TempFixture` pattern (d2 `install.rs:127-137` range precedent) + fake-PATH stub-script harness (Task 2 lưu ý):

1. **manifest-pin-verify fixture:** parse `tool-manifest.toml` fixture (synthetic checksum) → `ToolManifest` round-trip + verify path khớp pinned version+checksum của một fake tool. Sabotage checksum → fail loud.
2. **status-drift fixture:** fake-PATH stub in `0.1.1`, manifest pin `0.1.3` → `check_tools()` trả `Drift{expected:0.1.3, found:0.1.1}` (reproduce OA-07 doctor evidence) + `sos tools status` exit 1. Required `Missing` (stub absent) → exit 1. Optional missing → exit 0.
3. **doctor fail-clear fixture:** required tool Missing/older → gate exit ≠ 0 + message chứa tool+expected+found. Optional missing → exit 0 (warn only).

**Lưu ý:** negative-test mỗi fixture (sabotage → fail → revert). KHÔNG phụ thuộc tool thật cài trên CI (stub-PATH deterministic).

### Task 7: Docs (DOCS-GATE Tầng 1)

- `docs/PORTABILITY_ARCHITECTURE.md` — "P077d3 status" line: tool-manifest.toml path + schema + step-5 filled + status/doctor exit semantics + d3 GIỚI HẠN (upgrade/rollback = P081).
- `bootstrap/sos-rs/README.md` — module-layout row (`tools.rs` LIVE) + "tool-manifest (P077d3)" section.
- `docs/SETUP.md` — nếu tool-install doc đổi (thêm `sos tools status` mục); nếu KHÔNG đổi user-facing install flow → ghi "N/A" trong Discovery.
- `CHANGELOG.md` — `[P077d3]` entry trên `[P077d2]`.
- `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md` — OA-07 → note **RESOLVED trong Rust path** (pin+status+fail-clear+step-5 verify); `install.sh` legacy `releases/latest` giữ nguyên tới P077e cutover; atomic upgrade/rollback = P081.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `tool-manifest.toml` (kit root, MỚI) | Task 1: pin 10 tool version+asset+checksum, required flag |
| `crates/sos-install/src/tools.rs` (MỚI) | Task 2: `ToolManifest`/`ToolPin`/`check_tools()` core |
| `crates/sos-install/src/engine.rs` | Task 5: fill `resolve_tools()` seam |
| `crates/sos-install/src/lib.rs` | Task 2: `mod tools;` export (nếu cần) |
| `crates/sos-cli/src/commands/tools.rs` (MỚI) | Task 3: `sos tools status` |
| `crates/sos-cli/src/commands/mod.rs` + `main.rs` | Task 3/4: clap `Tools` variant + dispatch; Task 4 doctor surface (anchor #7) |
| `crates/sos-install/tests/tools.rs` (MỚI) | Task 6: 3 oracle fixtures |
| `docs/PORTABILITY_ARCHITECTURE.md`, `bootstrap/sos-rs/README.md`, `docs/SETUP.md`, `CHANGELOG.md`, `docs/retro/OUTSIDER_AUDIT...md` | Task 7 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `install.sh`, `bin/sos.sh` | KHÔNG đổi — additive. `git diff` phải empty |
| `.sos-manifest.toml` schema (d2) | KHÁC domain — KHÔNG đụng |
| `crates/sos-core/src/adapter.rs`, `manifest.rs` | d1 shape (trait 5-method, manifest 6-field) UNCHANGED |
| `dep_direction.rs` guard | `sos-install` vẫn chỉ dep `sos-core` (KHÔNG thêm host token) — guard xanh |

---

## Luật chơi (Constraints)

1. **Additive tuyệt đối** — `install.sh` + `bin/sos.sh` KHÔNG đổi. `sos install`/`sos tools` = path mới song song bootstrap hiện tại.
2. **Một cơ chế, hai/ba surface** — `check_tools()` core DÙNG CHUNG cho `sos tools status` + `sos doctor` + install step-5. KHÔNG viết lại drift-logic 3 lần.
3. **d3 GIỚI HẠN pin+status+verify+fail-clear** — atomic upgrade/rollback/auto-fetch external binary = P081 FUTURE. Required drift → FAIL + báo user, KHÔNG tự fetch.
4. **required flag phải khớp install.sh** (anchor #1) — 6 required / 4 optional. Drift với install.sh = bug.
5. **KHÔNG bịa checksum** (E2) — thiếu prebuilt → placeholder + escalate. KHÔNG bịa version-format regex (E1) → escalate.
6. **Fail-closed cho required** — required missing/older = exit≠0. Optional = warn, exit 0.
7. **`sos-install` giữ dep-direction** — chỉ `sos-core`, zero host token (Claude/Codex/`.claude`). `dep_direction.rs` guard xanh.
8. **tool-manifest.toml ≠ .sos-manifest.toml** — committed KIT config vs generated project artifact. KHÔNG lẫn path/schema.

---

## Nghiệm thu

### Automated
- [ ] `cargo build --workspace` clean (từ `bootstrap/sos-rs/`)
- [ ] `cargo test --workspace` xanh (fixtures d1/d2 + 3 fixture d3)
- [ ] **Oracle:** `[oracle: manifest-pin-verify fixture + status-drift fixture + doctor fail-clear fixture]` — hard-fail, 3/3 PASS
- [ ] **×20 TRUE parallel** (`seq 1 20 | xargs -P 20 -I{} cargo test --workspace`) = **0 flaky** (collision-safe TempFixture + fake-PATH, không phụ thuộc tool thật)
- [ ] `cargo clippy --workspace` — no NEW warning
- [ ] `dep_direction.rs` guard xanh (`sos-install` host-neutral)

### Manual Testing
- [ ] `sos tools status` trong scratch dir: in table, exit 1 khi required drift/missing, exit 0 khi clean
- [ ] doctor fail-clear: required older/missing → exit≠0 + message tool+expected+found (reproduce OA-07 doctor 0.1.1 vs 0.1.3)
- [ ] `sos install --dry-run`: step-5 resolve chạy (không còn no-op stub), KHÔNG mutate fs

### Regression
- [ ] `git diff install.sh bin/sos.sh` empty (additive)
- [ ] d2 install fixtures (6) + d1 trait/manifest test vẫn xanh
- [ ] `sos install` d2 behavior không đổi ngoài step-5 nay verify thật

### Docs Gate (Tầng 1)
- [ ] `CHANGELOG.md` — `[P077d3]` entry
- [ ] `docs/PORTABILITY_ARCHITECTURE.md` — P077d3 status line
- [ ] `bootstrap/sos-rs/README.md` — tool-manifest section
- [ ] `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md` — OA-07 resolved (Rust path) note
- [ ] `docs/SETUP.md` — updated HOẶC "N/A" ghi rõ trong Discovery

### Discovery Report
- [ ] Write `docs/discoveries/P077d3.md`:
  - Anchors #3/#6/#7/#8 CORRECT/WRONG (file:line) — `resolve_tools()` signature, sos-cli wiring, `sos doctor` placement quyết định, semver-vs-string
  - E1/E2 có fire không (tool nào unparseable version, checksum nào thiếu prebuilt)
  - `sos doctor` placement: fold-vào-subcommand hay tools-status+step-5-surface
  - Manifest-path resolve: SOS_KIT_DIR env vs embedded vs flag
  - Docs updated (hoặc "None"/"N/A" explicit)
  - Tier escalations (None nếu không)
  - **P077d CLOSED?** → xác nhận d1+d2+d3 done → next P077e cutover
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
