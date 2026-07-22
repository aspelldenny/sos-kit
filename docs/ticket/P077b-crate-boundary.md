# PHIẾU P077b: Crate boundary carve + dependency-direction rule

> Sub-phiếu THỨ 2 của P077 decomposition (`docs/plans/P077-decomposition.md`). Additive/reversible; Bash `bin/sos.sh` GIỮ canonical (P077e mới cutover). Xây trên P077a (workspace shell + parity harness đã landed — `docs/discoveries/P077a.md`).

---

> **Loại:** Feature (infra)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — build-system boundary; dependency direction sai thì LAN xuống P077c–e và ngấm vào mọi crate. Kiến trúc/contract surface → AUTO Tầng 1 dù diff nhỏ)
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/Cargo.toml` (→ virtual workspace), `bootstrap/sos-rs/crates/**` (mới: sos-core/sos-cli/sos-install/sos-adapter-claude/sos-hooks), lift `src/state.rs`→sos-core + `src/{main,commands}`→sos-cli, `bootstrap/sos-rs/README.md`
> **Dependency:** P077a merged (workspace shell `members=["."]`, parity harness `tests/parity.rs`, golden fixtures). None khác.

---

## Context

### Vấn đề hiện tại

P077a dựng workspace shell (`bootstrap/sos-rs/Cargo.toml` = `[workspace] members=["."]` + crate `sos` đơn) nhưng **chưa carve boundary** — toàn bộ code vẫn là một crate `sos` phẳng (`main.rs`, `state.rs`, `commands/`). Target (`docs/PORTABILITY_ARCHITECTURE.md` dòng 24-38) là monorepo nhiều crate với **dependency direction một chiều**: core sở hữu semantic (state/policy/roles/workflow/config schema), adapter phụ thuộc core contract, **core KHÔNG import adapter**, CLI là composition root (`PORTABILITY_ARCHITECTURE.md` dòng 68; `core/README.md` dòng 12-16 `portable core ─X─> host integration`).

Nếu impl `new/adopt/map/sync` (P077c) trên nền một-crate-phẳng thì dependency direction chưa có ranh giới cơ học → code P077c sẽ trộn host-concern vào core, và refactor boundary về sau chồng lên code parity đã test. Phải **carve boundary + enforce direction bằng gate cơ học TRƯỚC khi impl** (thứ tự P077-decomposition dòng 34: "carve crate boundary trước khi impl để dependency direction được enforce từ đầu, tránh refactor chồng ở P077c").

### Giải pháp

Additive/reversible, 4 mảnh, KHÔNG đổi hành vi user-facing (users vẫn dùng `bin/sos.sh`; binary Rust vẫn tên `sos`):

1. **Carve 5 crate skeleton** dưới `bootstrap/sos-rs/crates/`:
   - `sos-core` — semantic: nhận `state.rs` (state.toml mgmt, spec_hash, config schema). KHÔNG chứa host token / clap / CLI concern.
   - `sos-cli` — binary `sos` (composition root): nhận `main.rs` + `commands/`. Deps: sos-core + sos-install + sos-adapter-claude + sos-hooks.
   - `sos-install` / `sos-adapter-claude` / `sos-hooks` — **skeleton rỗng** (`lib.rs` stub `//! P077d`), logic thật ở P077d. Có mặt để **wiring dependency graph thật** cho gate kiểm.
2. **Dependency-direction gate cơ học** (Tầng 1 quyết): hai lớp —
   - **Compiler graph** (structural): sos-core Cargo.toml khai báo **zero** adapter/install/hooks/cli dep → mọi `use sos_adapter_*` / `use sos_install` trong core = compile error.
   - **Guard test** (`cargo test`, catch regression compiler câm): một `#[test]` scan `sos-core/src/**` tìm token cấm (`sos_adapter`, `sos_install`, `sos_hooks`, `sos_cli`) → fail loud nếu thấy. Bắt được ca "ai đó thêm dep vào core Cargo.toml" mà compiler vẫn xanh.
3. **Lift source** đúng boundary: `state.rs`→sos-core; `main.rs`+`commands/`→sos-cli (đổi `use crate::state`→`use sos_core::…`). Build vẫn ra binary `sos`.
4. **Workspace root GIỮ tại `bootstrap/sos-rs/`** (relocate lên repo-root = P077e vì chạm repo contract "Not a runtime binary source"). `bootstrap/sos-rs/Cargo.toml` chuyển từ `members=["."]` (mixed package+workspace) sang **virtual workspace** `members=["crates/*"]`.

### Scope

- CHỈ: carve 5 crate skeleton + dependency-direction gate + lift state/commands + README module-layout + note deviation.
- KHÔNG impl `new/adopt/map/sync` (P077c). KHÔNG install framework/manifest (P077d). KHÔNG cutover / relocate repo-root / flip `CLAUDE.md` contract (P077e). KHÔNG đụng `bin/sos.sh` (oracle — freeze). KHÔNG tạo `sos-adapter-codex` (P078).

---

## Task 0 — Verification Anchors

> Architect docs-only — không grep `src/`, không chạy cargo/Bash. Anchors code-level = `[needs Worker verify]`, Worker verify ở EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `bootstrap/sos-rs/Cargo.toml` hiện = `[workspace] members=["."]` + `[package] name="sos"` + deps clap/serde/toml/sha2/chrono/anyhow/walkdir | Đã Read (P077a landed) | ✅ `[verified]` (Cargo.toml dòng 1-26) |
| 2 | Target crate list = sos-cli/sos-core/sos-install/sos-adapter-claude/sos-hooks (+ sos-adapter-codex = P078 out-of-scope); direction: adapter→core one-way, CLI = composition root | `docs/PORTABILITY_ARCHITECTURE.md` dòng 24-38 + 68, `core/README.md` 12-16 | ✅ `[verified]` (doc read) |
| 3 | `src/` layout = `main.rs`, `state.rs`, `commands/{apply,blueprint,contract,init,launch,mod,recipe,status}.rs` (recon); CHƯA có new/adopt/map/sync | `ls bootstrap/sos-rs/src bootstrap/sos-rs/src/commands` | ⚠️ `[needs Worker verify]` (recon-provided; Architect không Read src) |
| 4 | `state.rs` host-NEUTRAL (state.toml/spec_hash/config schema) → hợp `sos-core`; KHÔNG chứa `CLAUDE_*`/host path/permission serialization | `grep -nE "CLAUDE\|\.claude\|permission\|clap" bootstrap/sos-rs/src/state.rs` | ⚠️ `[needs Worker verify]` — nếu state.rs chứa host token / clap → carve không sạch → DISCOVERY trước khi lift |
| 5 | `commands/` + `main.rs` là CLI concern (clap parse, dispatch) → hợp `sos-cli`; import `state` qua `use crate::state` (đổi thành `use sos_core::…` sau lift) | `grep -rn "use crate::state\|crate::state::" bootstrap/sos-rs/src/` | ⚠️ `[needs Worker verify]` — nếu coupling 2 chiều (state.rs `use crate::commands`) thì carve state→core không sạch → DISCOVERY |
| 6 | Dependency-direction gate khả thi bằng cargo-native (guard `#[test]` grep core src + compiler graph) — KHÔNG cần external tool (cargo-deny) | Worker viết guard test + chạy `cargo test`; negative-test (thêm tạm `use sos_adapter_claude;` vào 1 file core → `cargo build` fail) chứng minh one-way | ⚠️ `[needs Worker verify]` — nếu cargo-native không đủ, DISCOVERY đề xuất cargo-deny `[bans]` |
| 7 | Parity harness `tests/parity.rs` (P077a) vẫn chạy sau carve — nó invoke binary `sos` (giờ do sos-cli sinh); path harness có thể phải move về workspace-level `tests/` hoặc vào `crates/sos-cli/tests/` | `cargo test` sau carve → `parity_skeleton_informational` vẫn PASS + in "not yet parity" | ⚠️ `[needs Worker verify]` — nếu harness đứt do đổi crate, fix path (Tầng 2 self-decide) nhưng phải vẫn xanh |

**⚠️ Anchor 3-7 `[needs Worker verify]`.** Nếu Anchor 4 hoặc 5 lộ coupling bẩn (state↔commands 2 chiều, hoặc state.rs chứa host token) → carve state→core không sạch tại phiếu này → Worker DISCOVERY, đề xuất minimal-split (chỉ lift phần thật host-neutral) hoặc escalate Architect nếu boundary cần re-spec.

---

## Debate Log

> Cap = 3 turns. Sau Turn 3 chưa consensus → force-escalate Chủ nhà.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Worker accepted V1 — no challenges.**

**Anchor verification (recap Task 0):**
- Anchor #3: ✅ — `src/` layout matches recon exactly (`main.rs`, `state.rs`, `commands/{apply,blueprint,contract,init,launch,mod,recipe,status}.rs`); no new/adopt/map/sync present.
- Anchor #4: ✅ — `state.rs` host-neutral confirmed (`grep -nE "CLAUDE|\.claude|permission|clap" state.rs` → 0 hits); only anyhow/serde/std::fs/std::path/chrono. Safe to lift into sos-core.
- Anchor #5: ✅ — one-way coupling confirmed: `commands/{init,contract,apply,blueprint,launch}.rs` all `use crate::state::...`; `grep "commands" state.rs` → 0 hits, no reverse coupling.
- Anchor #6: ✅ — cargo-native (compiler graph + guard `#[test]` + negative-test) is coherent, sound, cheapest mechanism per §0.1; no cargo-deny needed.
- Anchor #7: ⚠️ — moving `tests/parity.rs` into `crates/sos-cli/tests/` keeps `CARGO_BIN_EXE_sos` valid (same-crate binary); `CARGO_MANIFEST_DIR` will then resolve to `crates/sos-cli` so `tests/golden/` needs to move alongside it (git-mv both together) rather than a path-string edit — flagged for EXECUTE, not a blocker.

Ready for Chủ nhà approval gate.

**Status:** ✅ CHALLENGE COMPLETE — APPROVED V1

### Final consensus
- Phiếu version: V1 (no revisions needed)
- Approved by Chủ nhà: 2026-07-22 — CHALLENGE APPROVED V1 → EXECUTE delegated

---

## Nhiệm vụ

### Task 1: Chuyển `Cargo.toml` sang virtual workspace + tạo 5 crate skeleton

**File:** `bootstrap/sos-rs/Cargo.toml` (sửa) + `bootstrap/sos-rs/crates/{sos-core,sos-cli,sos-install,sos-adapter-claude,sos-hooks}/Cargo.toml` (mới).

**Tìm:** khối hiện tại `[workspace] members=["."]` + `[package] name="sos"` + `[[bin]]` + `[dependencies]` (Cargo.toml dòng 1-26).

**Thay bằng / Thêm:** biến root thành **virtual workspace** (không còn `[package]` ở root):
```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
# shared version pin — deps hiện có, chia cho crate cần
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
sha2 = "0.10"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
walkdir = "2"
```
Mỗi crate skeleton một `Cargo.toml`. **Dependency graph (Tầng 1 — direction là móng):**
- `sos-core`: deps CHỈ từ {serde, toml, sha2, chrono, anyhow, walkdir} nếu cần — **ZERO** dep tới bất kỳ crate sos-* nào.
- `sos-adapter-claude`: `sos-core = { path = "../sos-core" }` (chứng minh adapter→core allowed). KHÔNG dep sos-cli/sos-install/sos-hooks.
- `sos-install`, `sos-hooks`: có thể dep `sos-core` (path) nếu cần; KHÔNG dep adapter/cli.
- `sos-cli`: composition root — dep `sos-core` + `sos-install` + `sos-adapter-claude` + `sos-hooks` (path) + clap. `[[bin]] name="sos"`.

**Lưu ý:**
- Binary vẫn phải tên `sos` (`crates/sos-cli` giữ `[[bin]] name="sos" path="src/main.rs"`) — regression: `cargo build` ra `target/debug/sos` như trước.
- KHÔNG đặt workspace ở repo-root `/Cargo.toml` (flip contract = P077e).
- `resolver="2"` giữ nguyên; nếu warning edition, Worker verify `cargo build` clean.

### Task 2: Lift `state.rs` → sos-core (semantic boundary)

**File:** `bootstrap/sos-rs/src/state.rs` → `bootstrap/sos-rs/crates/sos-core/src/` (`lib.rs` + module).

**Tìm:** N/A (move file). Nội dung state.rs = state.toml mgmt / spec_hash / config schema (recon; Worker Read xác nhận host-neutral per Anchor 4).

**Thay bằng / Thêm:** đặt `state.rs` làm module trong `sos-core` (vd `crates/sos-core/src/lib.rs` `pub mod state;` + `state.rs`). Export những gì `commands/` cần (`pub use`).

**Lưu ý:**
- **Anchor 4 GATE:** nếu Worker Read thấy state.rs chứa host token (`CLAUDE_*`, `.claude/**` path, permission serialization) hoặc `use clap` → KHÔNG thuộc core theo `core/README.md` dòng 27-32 → DISCOVERY, tách phần host ra (giữ ở sos-cli) trước khi lift, KHÔNG kéo host token vào core.
- **Anchor 5 GATE:** nếu state.rs `use crate::commands::…` (coupling ngược) → DISCOVERY: core không được biết command. Đề xuất invert (command truyền data vào state fn) hoặc escalate Architect nếu cần re-spec.
- Semantic Markdown (`core/*.md`) KHÔNG đụng — đây chỉ là Rust state crate.

### Task 3: Lift `main.rs` + `commands/` → sos-cli + rewire import

**File:** `bootstrap/sos-rs/src/main.rs` + `bootstrap/sos-rs/src/commands/*` → `bootstrap/sos-rs/crates/sos-cli/src/`.

**Tìm:** `use crate::state` / `crate::state::…` trong `main.rs` + `commands/*.rs` (Anchor 5).

**Thay bằng / Thêm:** move `main.rs`+`commands/` vào `crates/sos-cli/src/`; đổi mọi `crate::state` → `sos_core::state` (hoặc symbol export tương ứng). `main.rs` là composition root — khi cần install/adapter/hooks (P077d) mới gọi; ở đây skeleton stub chưa gọi nên chỉ cần declare dep (Task 1) để wiring graph.

**Lưu ý:**
- Sau lift `src/` cũ (`bootstrap/sos-rs/src/`) phải rỗng/xóa — KHÔNG để code trùng hai nơi (build ambiguity).
- Binary output vẫn `sos`, dispatch init/blueprint/contract/apply/recipe/launch/status y hệt trước (regression).
- KHÔNG impl new/adopt/map/sync (P077c).

### Task 4: Dependency-direction gate (guard test + negative proof)

**File:** guard test — Worker chọn vị trí (Tầng 2): `crates/sos-core/tests/dep_direction.rs` HOẶC workspace-level `tests/`. Đề xuất trong sos-core để chạy cùng `cargo test`.

**Tìm:** N/A (tạo mới).

**Thay bằng / Thêm:** `#[test]` scan đệ quy `crates/sos-core/src/**/*.rs`, assert KHÔNG có token nào trong {`sos_adapter`, `sos_install`, `sos_hooks`, `sos_cli`} (import path). Fail loud với message chỉ file vi phạm.

**Lưu ý (Tầng 1 — đây là gate cơ học của phiếu):**
- Hai lớp defense: (a) **compiler graph** (Task 1 — core zero adapter dep → `use sos_adapter_*` = compile error) là structural primary; (b) **guard test này** bắt regression compiler câm (ai thêm dep vào core Cargo.toml).
- **Negative-test oracle (bắt buộc chứng minh, ghi Discovery):** Worker thêm TẠM `use sos_adapter_claude;` vào một file `sos-core/src` → chạy `cargo build` → PHẢI fail (unresolved crate) → gỡ ra. Đây là bằng chứng one-way, không phải chỉ "build xanh".
- Cấm dùng external tool (cargo-deny) trừ khi cargo-native chứng minh không đủ (Anchor 6 DISCOVERY) — WORKFLOW §0.1 "cơ chế rẻ nhất".

### Task 5: Parity harness vẫn chạy sau carve

**File:** `bootstrap/sos-rs/tests/parity.rs` (P077a) — có thể phải relocate.

**Tìm:** harness hiện invoke binary `sos` + đọc `tests/golden/*.golden`.

**Thay bằng / Thêm:** đảm bảo harness vẫn tìm đúng binary `sos` (giờ do sos-cli sinh) + golden fixtures. Nếu vị trí `tests/` cũ đứt do đổi thành virtual workspace, move harness về `crates/sos-cli/tests/` hoặc workspace-level `tests/` sao cho `cargo test` vẫn chạy nó.

**Lưu ý:**
- Harness GIỮ informational (`HARD_FAIL=false` — P077c mới flip). KHÔNG đổi ngữ nghĩa harness, chỉ đảm bảo path/binary resolve đúng sau carve.
- Golden fixtures (`tests/golden/*.golden`) KHÔNG re-capture (oracle freeze) — chỉ đảm bảo harness đọc được.

### Task 6: README module layout + note deviation so target

**File:** `bootstrap/sos-rs/README.md`.

**Tìm:** phần "Status" / "Build" mô tả một-crate.

**Thay bằng / Thêm:** thêm mục "Module layout" liệt kê 5 crate + dependency direction (adapter→core one-way, cli=composition root). Ghi rõ **deviation so target** (`docs/PORTABILITY_ARCHITECTURE.md` dòng 24-38):
- Workspace root GIỮ `bootstrap/sos-rs/` (target = repo-root) → crates tại `bootstrap/sos-rs/crates/` (target `sos-kit/crates/`) — relocate = P077e.
- `sos-adapter-codex` CHƯA tạo (target liệt kê) — thuộc P078.
- install/adapter-claude/hooks là skeleton rỗng — logic P077d.

**Lưu ý:** KHÔNG viết lại toàn README. KHÔNG tuyên bố Rust canonical (chưa — P077e). Nếu crate layout thực khác target nhiều hơn 3 điểm trên → cập nhật thêm note trong `docs/PORTABILITY_ARCHITECTURE.md` (Docs Gate).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `bootstrap/sos-rs/Cargo.toml` | Task 1: `[package]` → virtual workspace `members=["crates/*"]` + `[workspace.dependencies]` |
| `bootstrap/sos-rs/crates/sos-core/{Cargo.toml,src/lib.rs}` (mới) | Task 1+2: crate core, zero adapter dep; nhận state.rs |
| `bootstrap/sos-rs/crates/sos-cli/{Cargo.toml,src/}` (mới) | Task 1+3: composition root, binary `sos`; nhận main.rs+commands/ |
| `bootstrap/sos-rs/crates/{sos-install,sos-adapter-claude,sos-hooks}/{Cargo.toml,src/lib.rs}` (mới) | Task 1: skeleton stub; adapter-claude dep sos-core |
| `bootstrap/sos-rs/src/state.rs` (move→sos-core) | Task 2: lift ra khỏi crate phẳng |
| `bootstrap/sos-rs/src/{main.rs,commands/*}` (move→sos-cli) | Task 3: lift + rewire `crate::state`→`sos_core::` |
| guard test (`crates/sos-core/tests/dep_direction.rs` hoặc tương đương) (mới) | Task 4: dependency-direction gate |
| `bootstrap/sos-rs/tests/parity.rs` (có thể relocate) | Task 5: vẫn chạy sau carve |
| `bootstrap/sos-rs/README.md` | Task 6: module layout + deviation note |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `bin/sos.sh` | ORACLE — behavior KHÔNG đổi; 11 subcommand chạy như trước |
| `bootstrap/sos-rs/tests/golden/*.golden` | Freeze — KHÔNG re-capture; chỉ harness đọc lại |
| `CLAUDE.md` "Not a runtime binary source" | KHÔNG đụng (flip = P077e); virtual workspace vẫn tại `bootstrap/sos-rs/` giữ contract root nguyên |
| `core/*.md`, `adapters/claude/**` | Semantic Markdown / P076 adapter — không thuộc carve này |

---

## Luật chơi (Constraints)

1. **Additive/reversible.** User chạy `bin/sos.sh` KHÔNG thấy khác. Binary Rust vẫn tên `sos`, dispatch command cũ y hệt.
2. **Dependency direction một chiều = móng, enforce cơ học từ đầu.** core ZERO adapter/install/hooks/cli dep. adapter→core allowed. cli=composition root. Gate = compiler graph + guard test; chứng minh bằng negative-test.
3. **Bash `bin/sos.sh` + golden fixtures là oracle — freeze.** Nhu cầu sửa = DISCOVERY.
4. **Workspace root GIỮ `bootstrap/sos-rs/`** — KHÔNG repo-root, KHÔNG flip `CLAUDE.md` contract (P077e).
5. **KHÔNG impl new/adopt/map/sync** (P077c), **KHÔNG install/manifest** (P077d), **KHÔNG tạo sos-adapter-codex** (P078). install/adapter/hooks = skeleton rỗng.
6. **Carve phải sạch** — nếu state.rs coupling bẩn với commands hoặc chứa host token (Anchor 4/5) → DISCOVERY, KHÔNG kéo host token vào core, KHÔNG tạo dep ngược.
7. **Parity harness vẫn xanh** (informational, `HARD_FAIL=false`) sau carve.
8. **Cơ chế rẻ nhất** — cargo-native gate (guard test + compiler), KHÔNG thêm external tool trừ khi chứng minh không đủ (§0.1).

---

## Nghiệm thu

### Automated
- [ ] `cd bootstrap/sos-rs && cargo build` — xanh, ra binary `sos` (từ sos-cli) như trước
- [ ] `cd bootstrap/sos-rs && cargo test` — xanh; guard test `dep_direction` PASS; parity harness in "not yet parity" (KHÔNG đỏ)
- [ ] **Negative-test (oracle):** thêm tạm `use sos_adapter_claude;` vào 1 file `sos-core/src` → `cargo build` FAIL (unresolved crate) → gỡ ra. Ghi vào Discovery.
- [ ] `bash -n bin/sos.sh` — vẫn PASS (oracle chưa đụng)

### Manual Testing
- [ ] `cargo tree -p sos-core` — KHÔNG xuất hiện sos-adapter-*/sos-install/sos-hooks/sos-cli (direction proof)
- [ ] `cargo tree -p sos-adapter-claude` — CÓ sos-core, KHÔNG có sos-cli (adapter→core one-way)
- [ ] `git diff bin/sos.sh` rỗng (oracle untouched); golden fixtures untouched
- [ ] Binary `sos` dispatch init/blueprint/contract/apply/recipe/launch/status y hệt trước carve

### Regression
- [ ] `bin/sos.sh` 11 subcommand chạy như trước phiếu (additive verify)
- [ ] `bootstrap/sos-rs/src/` cũ rỗng/xóa — không còn code trùng hai nơi

### Docs Gate
- [ ] `CHANGELOG.md` — entry P077b
- [ ] `bootstrap/sos-rs/README.md` — module layout + 3-điểm deviation (root, codex, skeleton) (Task 6)
- [ ] `docs/PORTABILITY_ARCHITECTURE.md` — note deviation NẾU crate layout thực khác target quá 3 điểm đã biết (else N/A, ghi rõ)
- [ ] `CLAUDE.md` "Not a runtime binary source" — **KHÔNG đổi ở P077b** (flip = P077e); xác nhận N/A trong Discovery. Virtual workspace tại `bootstrap/sos-rs/` giữ contract root nguyên; nếu Worker đánh giá scaffold vi phạm contract → DISCOVERY escalate founder.

### Discovery Report
- [ ] Write to `docs/discoveries/P077b.md`
  - Anchor 3-7 verify results (src layout, state.rs host-neutral?, commands coupling, gate cơ chế, harness sau carve)
  - Carve sạch không? (state.rs có coupling bẩn / host token không — nếu có, xử lý gì)
  - Dependency-direction gate: cơ chế chốt (cargo-native vs cargo-deny fallback) + negative-test kết quả
  - Deviation so target layout (root tại bootstrap/, codex chưa tạo, skeleton rỗng) — CONFIRM hay phát sinh thêm
  - Docs updated (list) — README + PORTABILITY note nếu có
  - Repo-contract touch: xác nhận "N/A — virtual workspace tại bootstrap/, repo-root contract untouched" HOẶC escalate
  - Tier escalations (None nếu không)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
