# PHIẾU P077f: WORKSPACE RELOCATE — `bootstrap/sos-rs/` → repo-root

> ✅ **REVERSIBLE** — pure `git mv` + path-fix, KHÔNG đổi logic. Rollback = `git revert` một commit (heavy Bash fns/dispatch/behavior không đụng). Xong = P077 HOÀN TOÀN DONE (workspace layout khớp `docs/PORTABILITY_ARCHITECTURE.md` target tree).

---

> **Loại:** Feature (relocate/infra)
> **Ưu tiên:** P1
> **Tầng:** 1 — build-system root + repo structure + đứng ngay sau cutover P077e (resolver dispatch surface). Sai → LAN (mọi Cargo path + resolver + docs). AUTO Tầng 1 dù reversible.
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/{Cargo.toml,Cargo.lock,crates/,README.md}` → repo-root; `bin/sos.sh` (resolver ws-root); `crates/sos-cli/tests/parity.rs` (CARGO_MANIFEST_DIR depth); `crates/sos-install/src/tools.rs` (include_str! depth — V2 O1.1); `scripts/orchestrator-guard.sh` (allow-list glob — V2 O1.2); `.gitignore`; `.sos-trust-baseline`; docs path-refs; `CHANGELOG.md`
> **Dependency:** P077e (CUTOVER — merged). Relocate là món cuối P077, orthogonal + reversible, tách khỏi cutover irreversible-ish theo founder split.

---

## Context

### Vấn đề hiện tại

P077e flip Rust `sos` binary thành canonical NHƯNG cố ý GIỮ workspace tại `bootstrap/sos-rs/` (transitional root) — relocate tách ra để không hàn churn-path-lớn vào contract-flip irreversible. Hệ quả còn treo:

- **Layout lệch target.** `docs/PORTABILITY_ARCHITECTURE.md` "Target workspace" (dòng 24-39) khai `Cargo.toml @ repo-root` + `crates/ @ repo-root`; `core/` và `tool-manifest.toml` ĐÃ ở root, nhưng workspace vẫn nằm dưới `bootstrap/`. `bootstrap/` = vestige duy nhất còn lại của giai đoạn "bootstrap target".
- **"Runtime monorepo" contract nửa vời.** P077e flip `CLAUDE.md` sang runtime monorepo nhưng repo-structure vẫn khai `bootstrap/sos-rs/` — người đọc CLAUDE.md thấy Rust source nằm ở nhánh phụ, không phải root như một monorepo chuẩn.

### Giải pháp — pure relocate + path-fix (KHÔNG đổi logic)

1. **`git mv`** nội dung workspace `bootstrap/sos-rs/{Cargo.toml, Cargo.lock, crates/, README.md, …}` → repo-root. Xoá `bootstrap/` nếu rỗng sau.
2. **Fix path-coupling điểm** vỡ do đổi depth: (a) `bin/sos.sh` resolver `_sos_workspace_root()` `bootstrap/sos-rs` → repo-root; (b) `parity.rs:246` CARGO_MANIFEST_DIR relative-depth `../../../../` → `../../`; (c) **`tools.rs:29` include_str! relative-depth `../../../../../` → `../../../` (V2 O1.1)**; (d) `.gitignore` `bootstrap/*/target/` → `/target/`; (e) `.sos-trust-baseline` rebaseline (bin/sos.sh + orchestrator-guard.sh hash đổi).
3. **Preserve behavior gate:** `scripts/orchestrator-guard.sh` allow-list glob `bootstrap/*` → thêm `crates/*` (+ `Cargo.toml`/`Cargo.lock`) để Quản đốc VẪN edit Rust CLI source trực tiếp NHƯ TRƯỚC (V2 O1.2 — pure relocate = giữ hành vi).
4. **Update docs path-refs** (NON-historical list — Task 6). KHÔNG rewrite historical (ticket/discoveries/plans/CHANGELOG past — evidence).
5. **KHÔNG đổi behavior:** binary/tests/cutover-dispatch/6-heavy-vs-7-guidance/resolver-precedence/guard-decision-semantics — tất cả giữ nguyên. Chỉ đổi WHERE workspace sống + path/glob trỏ tới nó.

### Scope
- CHỈ sửa: workspace file-tree move (git mv); `bin/sos.sh` (1 helper: ws-root path); `crates/sos-cli/tests/parity.rs` (1 depth constant); `crates/sos-install/src/tools.rs` (1 include_str! depth literal — V2); `scripts/orchestrator-guard.sh` (allow-list glob — V2); `.gitignore`; `.sos-trust-baseline` (rebaseline); `core/ASSETS.md`; `CLAUDE.md` (repo-structure + runtime-monorepo path refs + scripts list nếu guard scope mô tả đổi); `docs/PORTABILITY_ARCHITECTURE.md`; `README.md`; `docs/SETUP.md`; `docs/RUNTIME_BOUNDARY_INVENTORY.md`; `docs/LAYERS.md`/`docs/ORCHESTRATION.md` (NẾU mô tả guard scope — V2 DOCS-GATE verify); `bootstrap/sos-rs/README.md` (→ new location); `tests/README.md` (nếu có ref); `CHANGELOG.md`.
- KHÔNG sửa: Rust **logic** (`crates/**/src/**` — pure move, KHÔNG edit body — **NGOẠI LỆ HẸP V2:** duy nhất `tools.rs:29` path-literal depth-fix, KHÔNG đổi logic); dispatch case-block/resolver-precedence/heavy-vs-guidance split (P077e behavior giữ nguyên); guard decision-semantics (chỉ mở rộng allow-list glob, KHÔNG đổi block-logic); `install.sh` behavior (verify-only — zero-touch trừ khi grep lộ ref path); 7 guidance Bash fns; `docs/ticket/**`, `docs/discoveries/**`, `docs/plans/**`, CHANGELOG entries CŨ (historical — KHÔNG rewrite path).

---

## Task 0 — Verification Anchors

> Architect envelope = docs-only (no Bash/Grep/read-src). Anchors về `bin/sos.sh`/`parity.rs`/`tools.rs`/`orchestrator-guard.sh`/grep-ref-list là `[needs Worker verify]` (Worker CHALLENGE Turn 1 ĐÃ verify #13/#14 by file:line — xem Debate Log). Anchors về `.gitignore`/`.sos-trust-baseline`/`core/ASSETS.md`/`CLAUDE.md`/`PORTABILITY_ARCHITECTURE.md` là `[verified]`. **Cite RANGES, không count.**

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Repo-root COLLISION-FREE: chưa có `Cargo.toml`/`crates/`/`target/` ở root (sạch để `git mv` vào) | `ls Cargo.toml crates target 2>&1` (repo-root) → tất cả "No such file" | ✅ Worker Turn 1: all "No such file" |
| 2 | `bootstrap/sos-rs/` chứa `Cargo.toml`, `Cargo.lock`, `crates/`, `README.md`; `target/` gitignored KHÔNG move; `core/`+`tool-manifest.toml` ĐÃ ở root; `bootstrap/` CHỈ có `sos-rs/` | `ls -A bootstrap/sos-rs/` + `ls -A bootstrap/` | ✅ Worker Turn 1: exact match |
| 3 | 🔴 `crates/sos-cli/tests/parity.rs:246` `CARGO_MANIFEST_DIR` + `../../../../scripts/install-hooks.sh` = 4-hop to-root. Sau relocate phải `../../` (2 hop) | `grep -n 'CARGO_MANIFEST_DIR' crates/sos-cli/tests/parity.rs` | ✅ Worker Turn 1: `:246` confirmed |
| 4 | `parity.rs:52` `/tests/golden` + `dep_direction.rs:34` `/src` = relative-to-crate, depth-invariant → **KHÔNG edit** | grep `CARGO_MANIFEST_DIR` toàn `crates/**/tests/*.rs`; phân loại | ✅ Worker Turn 1: to-crate, excluded |
| 5 | 🔴 `bin/sos.sh` `_sos_workspace_root()` trả `bootstrap/sos-rs`. Precedence: `SOS_RUST_BIN` → `CARGO_TARGET_DIR/{release,debug}/sos` → `<ws>/target/{release,debug}/sos` → build. Đổi ws-root → repo-root | `grep -n '_sos_workspace_root\|bootstrap/sos-rs\|CARGO_TARGET_DIR' bin/sos.sh` | ✅ Worker Turn 1: `:1291-1292`, `:1303` |
| 6 | 🟡 SHARED-CACHE — `CARGO_TARGET_DIR/debug/sos` candidate TRƯỚC `<ws>/target` → binary bắt được bất kể ws | `SOS_RUST_BIN` unset → `bin/sos.sh map <x>` chạy binary OK | ⏳ `[needs Worker verify — escape-hatch nếu resolver mù]` |
| 7 | `.gitignore:8` = `bootstrap/*/target/` → đổi `/target/` | (đã Read :7-9) | ✅ `[verified]` |
| 8 | `.sos-trust-baseline` KHÔNG list `bootstrap/sos-rs/**`; `bin/sos.sh` + `orchestrator-guard.sh` hash đổi → rebaseline BẮT BUỘC | (đã Read :1-21) | ✅ `[verified]` |
| 9 | `core/ASSETS.md` :36,:57 ref `bootstrap/sos-rs/**` → update `crates/...` | (đã Read) | ✅ `[verified]` |
| 10 | `CLAUDE.md` repo-structure tree + runtime-monorepo lines ref `bootstrap/sos-rs/` → repo-root | (session context) | ✅ `[verified]` |
| 11 | Grep-ref list NON-historical: `README.md`, `docs/SETUP.md`, `docs/RUNTIME_BOUNDARY_INVENTORY.md`, `docs/PORTABILITY_ARCHITECTURE.md`, `bootstrap/sos-rs/README.md`(→moved), `scripts/orchestrator-guard.sh`, `tests/README.md`, `install.sh`(verify). KHÔNG rewrite historical dirs | `grep -rn 'bootstrap/sos-rs' --exclude-dir={docs/ticket,docs/discoveries,docs/plans,target,.git}` | ✅ Worker Turn 1: list confirmed |
| 12 | KHÔNG có `docs/AGENT_MAP.yaml` → `validate-map` = N/A | `ls docs/AGENT_MAP.yaml` → absent | ✅ `[verified]` |
| 13 | 🔴 **V2 (O1.1)** — `crates/sos-install/src/tools.rs:29` `include_str!("../../../../../tool-manifest.toml")` = **5-hop** relative-to-source-file (comment :20-22 confirm). Sau relocate file @ `<root>/crates/sos-install/src/tools.rs` → target root = **3 hop** (`src`→`sos-install`→`crates`→root). Phải `../../../../../` → `../../../`. Grep confirmed CHỈ 1 site (`include_str!/include_bytes!/include!` trong `crates/**/src/**`) | `grep -rn 'include_str!\|include_bytes!\|include!(' crates/**/src/**` → đếm hop to-root từng site | ✅ Worker Turn 1: `:28-29` sole site |
| 14 | 🔴 **V2 (O1.2)** — `scripts/orchestrator-guard.sh:71` `bootstrap/*) exit 0 ;;` = allow-list carve-out (comment :67-69: kit-maintenance, allow Quản đốc edit). Sau relocate `crates/*/src/*.rs` rớt sang product-source case `:78 */src/*` → require worker-active marker `:83` → BLOCK. Phải thêm `crates/*` vào allow-list để giữ hành vi | `grep -n 'bootstrap/\*\|\*/src/\*\|worker-active' scripts/orchestrator-guard.sh` → đọc case-block :67-83 | ✅ Worker Turn 1: `:71`, `:78`, `:83` confirmed |

**🔴/🟡 acknowledged:** Anchor #3 (parity depth), #5+#6 (resolver+shared-cache), **#13 (tools.rs include_str depth), #14 (guard allow-list glob)** là rủi ro cao. Nhiệm vụ dưới xử lý tường minh + escape-hatch.

### Pre-phiếu snapshot (Worker auto first-step)

> Relocate chạm build-root + resolver + guard → snapshot BẮT BUỘC trước mọi move.

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+[a-z]*')
mkdir -p ".backup/${PHIEU_ID}"
cp bin/sos.sh ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp scripts/orchestrator-guard.sh ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp .gitignore ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp .sos-trust-baseline ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

> **Dùng `git mv`** (không `mv` trần) để giữ history + để `git revert` khôi phục trivial.

---

## Debate Log

**Phiếu version:** V2 (Architect responded to Turn 1 — 2 objections ACCEPTED)

### Turn 1 — Worker Challenge

**Anchor verification (recap Task 0):**
- Anchor #1: ✅ — `ls Cargo.toml crates target` (repo-root) → all "No such file or directory". Collision-free.
- Anchor #2: ✅ — `ls -A bootstrap/sos-rs/` = `Cargo.lock Cargo.toml crates README.md` (exact match); `ls -A bootstrap/` = `sos-rs` only. `tool-manifest.toml` + `core/` already at repo-root (confirmed).
- Anchor #3/#4: ✅ — `parity.rs:246` `CARGO_MANIFEST_DIR).join("../../../../scripts/install-hooks.sh")` = 4-hop to-root (correct math, needs `../../` post-move). `parity.rs:52` `.join("tests/golden")` and `dep_direction.rs:34` `.join("src")` = to-crate, depth-invariant, correctly excluded from fix.
- Anchor #5/#6: ✅ — `bin/sos.sh:1291-1292` `_sos_workspace_root()` returns `"${SOS_KIT_DIR}/bootstrap/sos-rs"`. Line 1303 candidate order confirms CARGO_TARGET_DIR candidates precede `${ws}/target`. Live env has `CARGO_TARGET_DIR=/Users/nguyenhuuanh/.cargo-target-shared` set — shared-cache scenario is real.
- Anchor #7: ✅ `.gitignore:8` = `bootstrap/*/target/`.
- Anchor #8: ✅ `.sos-trust-baseline` lists `bin/sos.sh` hash, zero `bootstrap` path entries.
- Anchor #11: ✅ mostly complete — grep confirms all listed files. `install.sh` = zero hit, confirmed zero-touch. `tests/README.md` real path is `bootstrap/sos-rs/crates/sos-cli/tests/README.md` (travels with `git mv`). `docs/DISCOVERIES.md`/`docs/BACKLOG.md` hits = generic "bootstrap automation" prose, out of scope.

**Objections (Tầng 1 only):**
- [O1.1] **Anchor #3/#4 missed a hardcoded-depth site outside the `CARGO_MANIFEST_DIR` family.** `bootstrap/sos-rs/crates/sos-install/src/tools.rs:28-29`: `const EMBEDDED_TOOL_MANIFEST: &str = include_str!("../../../../../tool-manifest.toml");` — a **5-hop relative-to-source-file** path. Doc comment (:20-22) states "5 levels up ... `crates/sos-install/src/tools.rs` -> `bootstrap/sos-rs/crates/sos-install` -> kit root". After relocate → `<repo-root>/crates/sos-install/src/tools.rs` → only **3 hops** reach root. Left at 5 hops, `include_str!` resolves OUTSIDE repo → **compile error** (`cargo build --workspace` would catch, but only at build time). Contradicts "Files KHÔNG sửa" row `crates/**/src/**` = PURE MOVE. Grep pattern `CARGO_MANIFEST_DIR` doesn't cover `include_str!`. Recommend Task 2b: fix `../../../../../` → `../../../`; grep-confirmed sole site.
- [O1.2] **`scripts/orchestrator-guard.sh:71` is a functional gate glob, not just a path reference.** Line `bootstrap/*) exit 0 ;;` is an allow-list carve-out (comment :67-69: "kit's OWN bundled tooling ... kit-maintenance ... allow") letting Quản đốc edit kit's Rust CLI source without `worker-active` marker. After relocate `crates/*/src/*.rs` no longer matches `bootstrap/*`; falls into product-source case `:78 */src/*` → requires marker `:83` → BLOCKS main session editing moved Rust source. Genuine behavior change on a security/discipline gate — violates Constraint #1 (PURE RELOCATE) + #8 (Contract-surface). Allow-list glob must be updated (`bootstrap/*` → also match `crates/*` / `Cargo.toml`) to preserve the pre-relocate exemption.

**Status:** ✅ RESPONDED (Architect Turn 1)

### Turn 1 — Architect Response (phiếu V2)

- **[O1.1] → ACCEPT.** Đúng — grep sàng `CARGO_MANIFEST_DIR` không phủ họ `include_str!/include_bytes!/include!`; `tools.rs:29` là hardcoded relative-to-source-file 5-hop nằm ngoài lưới. Post-relocate 5-hop resolve ngoài repo → `cargo build --workspace` FAIL. → **Task 2b mới** (fix `../../../../../` → `../../../`, 5→3 hop; đếm hop thật, KHÔNG hardcode mù). Amend Constraint #1 + "Files KHÔNG sửa" row: NGOẠI LỆ HẸP — duy nhất path-literal depth (include_str! relative), KHÔNG đổi logic. Ghi grep-confirmed-only-site (Worker Turn 1: `:28-29` sole `include*` site trong `crates/**/src/**`); nếu grep lộ thêm site → escalate.
- **[O1.2] → ACCEPT.** Đúng — `:71 bootstrap/*` là allow-list chức năng, không phải path-ref cosmetic; sau relocate `crates/*/src/*.rs` rớt vào product-source case `:78` → require `worker-active` → BLOCK Quản đốc = behavior-change trên security surface (vi phạm Constraint #1 pure-relocate). → **Task mới (guard glob)**: thêm `crates/*) exit 0 ;;` (+ cân `Cargo.toml`/`Cargo.lock`/`crates/**/*.rs`) vào allow-list. **Preserve-behavior approach:** GIỮ CẢ `bootstrap/*` lẫn `crates/*` (bootstrap/ sẽ gỡ nên `bootstrap/*` thành dead glob vô hại — an toàn hơn là xoá). Mục tiêu: Quản đốc edit Rust CLI source trực tiếp NHƯ TRƯỚC, KHÔNG bị block. CHỈ mở rộng allow-list, KHÔNG đổi block-logic/decision-semantics.
  - **Doctrine question NOTED, NOT resolved trong P077f:** post-P077e-cutover Rust CLI source giờ LÀ runtime/product — có nên chuyển sang Worker-gated (product-source) thay vì kit-maintenance allow-list? Đây là doctrine-decision RIÊNG, KHÔNG nhét vào relocate. P077f CHỈ preserve hành vi hiện tại. → Ghi Discovery + BACKLOG như open question (xem Discovery Report).
- **DOCS-GATE (V2):** `orchestrator-guard.sh` đổi = security-surface. Verify + update nếu `docs/LAYERS.md`/`docs/ORCHESTRATION.md` mô tả guard allow-list scope + `CLAUDE.md` scripts list. `.sos-trust-baseline` rebaseline nay bắt cả `orchestrator-guard.sh` hash (Anchor #8 amended).

**Status:** ✅ RESPONDED — phiếu bumped to V2. Cả 2 objection ACCEPT, không DEFER. Orchestrator: spawn Worker CHALLENGE lại verify consensus HOẶC (nếu response trivially-correct) proceed approval gate.

### Turn 2 — Worker Accept V2

Worker accepted V2 — no further challenges. Both O1.1 (tools.rs include_str 5→3 hop, Task 2b) and O1.2 (orchestrator-guard.sh crates/* allow-list addition) resolutions match the Turn 1 recommendation exactly. Preserve-behavior approach (keep `bootstrap/*` as dead glob, add `crates/*`) is correct — no decision-semantics change. Proceeding to EXECUTE mode per Chủ nhà delegation (marker `worker-active` set, branch `P077f-workspace-relocate`).

### Final consensus
- Phiếu version: V2
- Total turns: 2
- Approved by Chủ nhà: 2026-07-22 (delegated EXECUTE — sprint handoff)

---

## Nhiệm vụ

### Task 1: Enumerate + `git mv` workspace → repo-root

**File:** file-tree (`bootstrap/sos-rs/` → repo-root)

**Tìm:** nội dung `bootstrap/sos-rs/` (Anchor #2). Xác nhận collision-free root (Anchor #1).

**Thực hiện:**
- `git mv` từng top-level entry: `bootstrap/sos-rs/Cargo.toml` → `Cargo.toml`, `Cargo.lock` → `Cargo.lock`, `crates` → `crates`, `README.md` → (Task 5). File khác ở Anchor #2 → move theo, TRỪ `target/` (gitignored).
- Sau move: `bootstrap/` rỗng → `git rm`/xoá. Còn file lạ → DISCOVERY + KHÔNG xoá, escalate.

**Lưu ý:**
- `Cargo.toml` `[workspace] members = ["crates/*"]` relative → move cùng `crates/` thì members KHÔNG đổi. Verify `cargo metadata` không lỗi.
- KHÔNG edit nội dung `.rs` (pure move — trừ Task 2b path-literal). `Cargo.toml` path-dependency `../` ra ngoài workspace → verify + báo (dự kiến KHÔNG có).
- `core/` + `tool-manifest.toml` ĐÃ ở repo-root (KHÔNG move).

### Task 2: 🔴 Fix `parity.rs` CARGO_MANIFEST_DIR depth (relative-to-root site)

**File:** `crates/sos-cli/tests/parity.rs` (sau move)

**Tìm (Anchor #3, :246):** `CARGO_MANIFEST_DIR` ghép `../../../../scripts/install-hooks.sh` (4 hop).

**Thay bằng:** giảm 2 hop → `../../scripts/install-hooks.sh`. Worker ĐẾM hop thật tới `scripts/install-hooks.sh` sau relocate — mục tiêu resolve `<repo-root>/scripts/install-hooks.sh`.

**Lưu ý:**
- CHỈ site relative-to-repo-root vỡ. Site to-crate (`:52`, `dep_direction.rs:34`) → KHÔNG đụng (Anchor #4).
- Grep lộ THÊM site relative-to-root ngoài :246 → fix hết + Discovery.
- Oracle: `cargo test --workspace` từ root.

### Task 2b: 🔴 Fix `tools.rs` include_str! relative depth (V2 — O1.1)

**File:** `crates/sos-install/src/tools.rs` (sau move; trước = `bootstrap/sos-rs/crates/sos-install/src/tools.rs`)

**Tìm (Anchor #13, :28-29):** `const EMBEDDED_TOOL_MANIFEST: &str = include_str!("../../../../../tool-manifest.toml");` — 5-hop relative-to-source-file (comment :20-22 giải thích 5-level).

**Thay bằng:** giảm 2 hop → `include_str!("../../../tool-manifest.toml")` (3 hop: `src`→`sos-install`→`crates`→root). Worker ĐẾM hop thật từ `crates/sos-install/src/tools.rs` tới `<repo-root>/tool-manifest.toml` sau relocate — KHÔNG hardcode mù. **CẬP NHẬT comment :20-22** cho khớp depth mới (3-level, path mới) — comment giờ nói 5-level sẽ sai.

**Lưu ý:**
- **NGOẠI LỆ HẸP duy nhất** với "KHÔNG edit `crates/**/src/**` body": chỉ path-literal + comment mô tả path, KHÔNG đổi logic/const-name/type.
- **Grep-confirmed CHỈ 1 site** (Worker Turn 1: sole `include_str!/include_bytes!/include!` trong `crates/**/src/**`). Nếu grep lộ THÊM site `include*` với hardcoded `../` relative-to-repo-root → fix cùng nguyên tắc + Discovery; nếu KHÁC nguyên tắc (không tự tin đếm) → **STOP + escalate** (escape-hatch).
- Oracle: `cargo build --workspace` từ root — 5-hop resolve ngoài repo = compile error, fix đúng = PASS.

### Task 3: Fix `bin/sos.sh` resolver workspace-root

**File:** `bin/sos.sh`

**Tìm (Anchor #5):** `_sos_workspace_root` (:1291-1292) trả `bootstrap/sos-rs`.

**Thay bằng:** trả repo-root. Worker chọn biểu diễn root đúng chuẩn resolver hiện có (git-toplevel, hoặc dir script) `[needs Worker verify]` — miễn `<ws>/target/{release,debug}/sos` + `cd "$ws" && cargo build` trỏ repo-root.

**Lưu ý:**
- KHÔNG đổi precedence, heavy-arm list, fail-loud, no-`command -v sos`. Chỉ ws-root value.
- Shared-cache (Anchor #6): `CARGO_TARGET_DIR/debug/sos` TRƯỚC `<ws>/target` → binary vẫn bắt. Fix ws-root cho path non-redirect + build fallback.
- Exact ws-root biểu diễn = **Tầng 2, Worker tự quyết**. Contract cứng: resolver resolve về binary đúng (smoke).

### Task 3b: Fix `scripts/orchestrator-guard.sh` allow-list glob (V2 — O1.2, PRESERVE behavior)

**File:** `scripts/orchestrator-guard.sh`

**Tìm (Anchor #14, case-block :67-83):** allow-list carve-out `:71 bootstrap/*) exit 0 ;;` (comment :67-69 = kit-maintenance allow) + product-source case `:78 */src/*` require `worker-active` marker `:83`.

**Thay bằng:** thêm allow-list arm cho new location — `crates/*) exit 0 ;;` (+ cân thêm `Cargo.toml`/`Cargo.lock` nếu guard cũng gate 2 file đó cho kit-maintenance). **GIỮ NGUYÊN `bootstrap/*) exit 0 ;;`** (bootstrap/ gỡ → dead glob vô hại; giữ để an toàn). Đặt arm `crates/*` TRƯỚC product-source case `:78 */src/*` (case-glob first-match wins) để `crates/*/src/*.rs` được allow như bootstrap trước đây.

**Lưu ý:**
- **PRESERVE-BEHAVIOR only.** CHỈ mở rộng allow-list glob, KHÔNG đổi block-logic/decision-semantics/marker-check `:83`. Mục tiêu: Quản đốc edit `crates/*/src/*.rs` trực tiếp NHƯ TRƯỚC relocate (kit-maintenance), KHÔNG bị block.
- **KHÔNG resolve doctrine-question** (Rust-source-giờ-là-product → có nên Worker-gate?) trong phiếu này — chỉ giữ hành vi hiện tại. Doctrine-decision ghi Discovery/BACKLOG (open question).
- Worker verify first-match order: `crates/*` phải match TRƯỚC `*/src/*` (nếu case-block dùng `;;&` fallthrough hoặc thứ tự khác → điều chỉnh + Discovery).
- Guard là auto-exec script → đổi = rebaseline bắt (Task 4).

### Task 4: `.gitignore` + `.sos-trust-baseline` rebaseline

**File:** `.gitignore`

**Tìm (Anchor #7, :8):**
```
bootstrap/*/target/
```
**Thay bằng:**
```
/target/
```
**Lưu ý:** anchored `/target/` = root-level target. Build sinh target path khác → điều chỉnh + Discovery.

**File:** `.sos-trust-baseline`

**Thực hiện (Anchor #8):** sau khi fix MỌI auto-exec-surface (`bin/sos.sh` Task 3, **`orchestrator-guard.sh` Task 3b — V2**) → chạy `scripts/trust-gate.sh rebaseline` cập nhật hash. KHÔNG sửa tay từng dòng hash.

**Lưu ý:** rebaseline = bước CUỐI (sau mọi edit script). V2: nay bắt CẢ `bin/sos.sh` (:3) LẪN `orchestrator-guard.sh` hash đổi.

### Task 5: `bootstrap/sos-rs/README.md` → repo-root + wording

**File:** `bootstrap/sos-rs/README.md` → đích mới (Worker quyết: root `README.md` CÓ SẴN → KHÔNG overwrite; workspace README → `crates/README.md` HOẶC merge "Module layout" section vào root) `[needs Worker verify]`.

**Thực hiện:** move file, đổi self-reference path `bootstrap/sos-rs/...` → new. Đích trùng file tồn tại → KHÔNG overwrite; đặt tại `crates/README.md` hoặc `docs/` + Discovery lựa chọn.

**Lưu ý:** wording ownership (P077e "canonical, không extract") giữ; chỉ đổi path-refs. Contract: không mất "Module layout" doc P077b tạo.

### Task 6: Update docs path-refs (NON-historical)

**File:** mỗi file ở Anchor #11 grep-list + V2 DOCS-GATE additions.

**Tìm/Thay:** đổi `bootstrap/sos-rs/` → new root layout trong: `CLAUDE.md` (repo-structure tree: gỡ `├── bootstrap/`+`│   └── sos-rs/`, thêm `├── Cargo.toml` + `├── crates/` đúng alphabet; runtime-monorepo contract lines; **+ scripts list nếu mô tả `orchestrator-guard.sh` scope — V2**), `core/ASSETS.md` (:36, :57), `docs/PORTABILITY_ARCHITECTURE.md` (status-line ~41-54 + **P077f status note**: relocate DONE, layout khớp target, `bootstrap/` gỡ), `README.md`, `docs/SETUP.md`, `docs/RUNTIME_BOUNDARY_INVENTORY.md`, `scripts/orchestrator-guard.sh` (path-ref TRONG comment nếu nhắc `bootstrap/sos-rs` — glob-logic đã fix ở Task 3b), `tests/README.md`, `install.sh` (verify — dự kiến zero ref).

**+ V2 DOCS-GATE verify (guard security-surface):** grep `docs/LAYERS.md` + `docs/ORCHESTRATION.md` cho mô tả guard allow-list scope / "kit-maintenance vs product-source" — NẾU mô tả glob cụ thể (`bootstrap/*`) → update sang `crates/*` khớp Task 3b. NẾU chỉ mô tả nguyên tắc (không path cụ thể) → KHÔNG đụng + ghi Discovery "guard scope doc mô tả nguyên tắc, path-agnostic, N/A".

**Lưu ý:**
- KHÔNG rewrite historical: `docs/ticket/**` (gồm P077a-e), `docs/discoveries/**`, `docs/plans/**`, CHANGELOG cũ. Phân loại từng hit; nghi ngờ → giữ + Discovery.
- `CLAUDE.md` DOCS-GATE row "file move" nhắc AGENT_MAP → N/A (Anchor #12).
- `PORTABILITY_ARCHITECTURE.md` "Target workspace" tree (:24-39) ĐÃ khai repo-root = target → thêm status note ĐẠT, KHÔNG viết lại tree.

### Task 7: `CHANGELOG.md`

**File:** `CHANGELOG.md`

**Thêm** entry mới nhất trên cùng: `[P077f]` — RELOCATE: Rust workspace `bootstrap/sos-rs/` → repo-root (`Cargo.toml`+`crates/` @ root, `bootstrap/` gỡ); layout khớp PORTABILITY target tree; path-fix resolver (`bin/sos.sh` ws-root) + `parity.rs` CARGO_MANIFEST_DIR depth + **`tools.rs` include_str! depth (V2)** + `.gitignore` `/target/` + **`orchestrator-guard.sh` allow-list preserve (V2)** + trust-baseline rebaseline; pure relocate, behavior/dispatch/cutover/guard-semantics KHÔNG đổi; reversible (git revert). **P077 HOÀN TOÀN DONE.** Entry fresh giữ `docs-gate --all` PASS.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `bootstrap/sos-rs/{Cargo.toml,Cargo.lock,crates/,…}` → repo-root | Task 1: `git mv` lên root; gỡ `bootstrap/` nếu rỗng |
| `crates/sos-cli/tests/parity.rs` | Task 2: CARGO_MANIFEST_DIR to-root depth `../../../../`→`../../` (CHỈ site to-root) |
| `crates/sos-install/src/tools.rs` | **Task 2b (V2):** include_str! depth `../../../../../`→`../../../` (5→3 hop) + comment :20-22. NGOẠI LỆ HẸP src-body: chỉ path-literal |
| `bin/sos.sh` | Task 3: `_sos_workspace_root` → repo-root (precedence/heavy-arms/fail-loud giữ) |
| `scripts/orchestrator-guard.sh` | **Task 3b (V2):** allow-list glob thêm `crates/*) exit 0 ;;` (+`Cargo.toml`/`Cargo.lock` nếu cần), giữ `bootstrap/*`; PRESERVE behavior, KHÔNG đổi block-logic |
| `.gitignore` | Task 4: `bootstrap/*/target/` → `/target/` |
| `.sos-trust-baseline` | Task 4: `scripts/trust-gate.sh rebaseline` (bin/sos.sh + orchestrator-guard.sh hash đổi) |
| `bootstrap/sos-rs/README.md` → new location | Task 5: move + self-path fix (không overwrite root README) |
| `CLAUDE.md` | Task 6: repo-structure tree + runtime-monorepo refs + scripts list (guard scope V2) |
| `core/ASSETS.md` | Task 6: :36 + :57 path refs |
| `docs/PORTABILITY_ARCHITECTURE.md` | Task 6: status-line refs + P077f status note |
| `docs/LAYERS.md`, `docs/ORCHESTRATION.md` | **Task 6 (V2):** verify guard allow-list scope mô tả — update NẾU có glob cụ thể, N/A nếu path-agnostic |
| `README.md`, `docs/SETUP.md`, `docs/RUNTIME_BOUNDARY_INVENTORY.md`, `tests/README.md` | Task 6: path refs (mỗi cái grep-verify NON-historical) |
| `CHANGELOG.md` | Task 7: `[P077f]` entry |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/**/src/**` (Rust logic) | PURE MOVE — KHÔNG edit body. **NGOẠI LỆ HẸP V2:** duy nhất `tools.rs:29` path-literal + comment (Task 2b), KHÔNG đổi logic. Site `include*` khác → escalate |
| `crates/**/tests/*.rs` site relative-to-crate (`parity.rs:52`, `dep_direction.rs:34`) | KHÔNG đổi depth (to-crate — Anchor #4) |
| `bin/sos.sh` dispatch case-block + resolver precedence + heavy/guidance split | P077e behavior GIỮ; chỉ ws-root value đổi |
| `scripts/orchestrator-guard.sh` block-logic + marker-check `:83` + decision-semantics | V2: CHỈ mở rộng allow-list glob (Task 3b); KHÔNG đổi block-logic |
| `install.sh` | Grep `bootstrap/sos-rs` → dự kiến zero; nếu ref → update (Task 6). Behavior zero-touch |
| `tool-manifest.toml`, `core/**` (content) | ĐÃ ở repo-root — KHÔNG move |
| `docs/ticket/**`, `docs/discoveries/**`, `docs/plans/**`, CHANGELOG cũ | HISTORICAL — KHÔNG rewrite path (evidence) |

---

## Luật chơi (Constraints)

1. **PURE RELOCATE.** KHÔNG đổi logic/behavior/dispatch/resolver-precedence/guard-decision-semantics. Chỉ WHERE workspace sống + path/glob trỏ tới nó. Diff `.rs` body = 0 — **NGOẠI LỆ HẸP V2:** duy nhất `parity.rs` depth-constant (Task 2) + `tools.rs:29` include_str! path-literal & comment (Task 2b); cả 2 chỉ đổi path depth, KHÔNG đổi logic.
2. **`git mv`, KHÔNG `mv` trần** — giữ history + revert trivial.
3. **Reconcile Anchor #3+#4 TRƯỚC khi sửa `parity.rs`:** phân loại mọi `CARGO_MANIFEST_DIR` site to-root(fix) vs to-crate(giữ). Sửa nhầm to-crate = break test.
4. **Reconcile Anchor #13 TRƯỚC khi sửa `tools.rs` (V2):** đếm hop thật tới `<root>/tool-manifest.toml`; grep-confirmed sole `include*` site — site khác lộ ra → escalate KHÔNG tự fix mù.
5. **Guard preserve-behavior (V2):** `crates/*` allow arm phải match TRƯỚC product-source `*/src/*` (first-match). CHỈ mở rộng allow-list, KHÔNG đổi block-logic. Doctrine-question (Rust=product→Worker-gate?) KHÔNG resolve ở đây — ghi Discovery/BACKLOG.
6. **Resolver (#5/#6):** đổi ws-root, KHÔNG đổi precedence/fail-loud/no-`command -v sos`. Sau relocate `bin/sos.sh map <x>` PHẢI resolve Rust binary (smoke).
7. **Rebaseline trust-baseline LÀ BƯỚC CUỐI** — sau MỌI edit auto-exec script (`bin/sos.sh` + `orchestrator-guard.sh` — V2).
8. **KHÔNG rewrite historical path** (ticket/discoveries/plans/CHANGELOG cũ). Nghi ngờ → giữ + Discovery.
9. **Escape-hatch:** (a) `git mv` conflict / root không collision-free → STOP, DISCOVERY, escalate; (b) resolver mù sau relocate → STOP, escalate — KHÔNG silent-fallback Bash; (c) **grep lộ thêm hardcoded-depth site (`include*`/CARGO_MANIFEST_DIR to-root) ngoài `tools.rs:29`+`parity.rs:246`** → STOP, escalate (V2); (d) **guard case-block order khác dự kiến (`;;&` fallthrough / arm order) → điều chỉnh + Discovery, verify first-match** (V2).
10. Contract-surface (`CLAUDE.md` repo-structure, **`orchestrator-guard.sh` allow-list = security surface — V2**) → Guarded + Tầng 1 auto CHALLENGE (CLAUDE.md Rule #9).

---

## Nghiệm thu

### Automated
- [ ] `cargo build --workspace` **từ repo-root** clean — **bắt CẢ `parity.rs` depth LẪN `tools.rs` include_str! depth (V2): 5-hop resolve ngoài repo = compile error**.
- [ ] **Flaky gate:** `cargo test --workspace` từ root **×20** = 0 fail, 0 flaky. Oracle bắt CARGO_MANIFEST_DIR depth sai.
- [ ] `cargo metadata --format-version 1 >/dev/null` từ root PASS.
- [ ] `bash -n bin/sos.sh` PASS + `bash -n scripts/orchestrator-guard.sh` PASS (V2).
- [ ] `docs-gate --all` PASS (CHANGELOG fresh).
- [ ] `scripts/trust-gate.sh` (pre-commit) PASS sau rebaseline (hash `bin/sos.sh` + `orchestrator-guard.sh` khớp baseline mới).

### Manual Testing — relocate smoke
- [ ] **Cutover vẫn chạy:** `bin/sos.sh map <crate>` → dispatch Rust binary, resolver tìm được (Anchor #6). Output == gọi thẳng `$(_sos_rust_bin) map <crate>`.
- [ ] Resolver shared-cache: `SOS_RUST_BIN` unset, `CARGO_TARGET_DIR=~/.cargo-target-shared` → `bin/sos.sh new/adopt/map/sync/install/tools --help` exit 0.
- [ ] Resolver non-redirect fallback: unset `CARGO_TARGET_DIR`+`SOS_RUST_BIN` → resolver dùng `<repo-root>/target/{release,debug}/sos` hoặc build-on-demand từ root, KHÔNG trỏ `bootstrap/sos-rs`.
- [ ] **Guard preserve-behavior (V2):** mô phỏng input path `crates/sos-cli/src/main.rs` (Quản đốc edit, KHÔNG có `worker-active` marker) → `orchestrator-guard.sh` **exit 0 (allow) NHƯ `bootstrap/*` trước relocate**. Mô phỏng input path product-source thật (nếu có ngoài crates) → vẫn block đúng (block-logic KHÔNG đổi).
- [ ] `bootstrap/` KHÔNG còn tồn tại (hoặc rỗng-đã-gỡ).
- [ ] 1 guidance cmd (`bin/sos.sh status`) vẫn chạy Bash như cũ.

### Regression
- [ ] 6 heavy + 7 guidance dispatch behavior KHÔNG đổi so P077e.
- [ ] `parity.rs` install-hooks path test PASS (depth đúng).
- [ ] **`tools.rs` EMBEDDED_TOOL_MANIFEST resolve đúng (V2)** — binary embed tool-manifest.toml nội dung khớp (test đọc manifest nếu có, hoặc build PASS = include resolve).
- [ ] **Guard: input path chưa-move `bootstrap/*` (nếu còn ai gọi) vẫn allow** — dead glob giữ, vô hại (V2).
- [ ] `.gitignore` vẫn ignore build artifact: `git status` sau `cargo build` KHÔNG list `target/`.

### Docs Gate (Tầng 1 — BẮT BUỘC)
- [ ] `CLAUDE.md` — repo-structure tree + runtime-monorepo refs + scripts list (guard scope V2) updated.
- [ ] `core/ASSETS.md` — :36 + :57.
- [ ] `docs/PORTABILITY_ARCHITECTURE.md` — status refs + P077f status note.
- [ ] **`docs/LAYERS.md` + `docs/ORCHESTRATION.md` (V2)** — guard allow-list scope: updated nếu mô tả glob cụ thể, HOẶC Discovery ghi "path-agnostic, N/A".
- [ ] `README.md`, `docs/SETUP.md`, `docs/RUNTIME_BOUNDARY_INVENTORY.md`, `bootstrap/sos-rs/README.md`(moved), `tests/README.md` — path refs.
- [ ] `CHANGELOG.md` — `[P077f]` entry.
- [ ] AGENT_MAP `validate-map` = N/A — ghi rõ Discovery.
- [ ] Discovery ghi: "Tầng 1 docs updated: CLAUDE.md, core/ASSETS.md, PORTABILITY_ARCHITECTURE.md, LAYERS.md/ORCHESTRATION.md (guard scope), README.md, SETUP.md, RUNTIME_BOUNDARY_INVENTORY.md, CHANGELOG.md, (moved README)".

### Rollback plan (REVERSIBLE)
- Relocate = 1 commit trên branch phiếu. Fail sau merge → `git revert <relocate-commit>` khôi phục layout + resolver + guard cũ (pure move).
- Trong worktree: `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` HOẶC restore `.backup/${PHIEU_ID}/{sos.sh,orchestrator-guard.sh,.gitignore,.sos-trust-baseline}`.
- **NEVER** reset trên main.

### Discovery Report
- [ ] `docs/discoveries/P077f.md`:
  - Anchor #2: nội dung move; `bootstrap/` gỡ được không.
  - Anchor #3/#4: mọi `CARGO_MANIFEST_DIR` site + phân loại; depth cuối `parity.rs:246`.
  - **Anchor #13 (V2):** `tools.rs` include_str! hop cuối; grep có lộ thêm `include*` site không.
  - **Anchor #14 (V2):** guard case-block order thực tế; `crates/*` arm match TRƯỚC `*/src/*` xác nhận; input-path smoke kết quả.
  - Anchor #5/#6: resolver ws-root sửa ra sao; shared-cache resolve OK; có mù không.
  - Anchor #11: grep-hit list + phân loại historical/live.
  - **Doctrine open-question (V2):** "post-cutover Rust CLI source = runtime/product — có nên chuyển guard sang Worker-gated thay vì kit-maintenance allow-list? P077f PRESERVE hành vi hiện tại, KHÔNG resolve. → BACKLOG open question." Append 1-line vào `docs/BACKLOG.md` (Open/Park section).
  - Assumptions CORRECT/WRONG (file:line ranges).
  - Trust-baseline rebaseline: script nào đổi hash (bin/sos.sh + orchestrator-guard.sh).
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
