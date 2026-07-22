# PHIẾU P077e: CUTOVER — Rust `sos` thành canonical cho 6 lệnh nặng

> ⚠️ **IRREVERSIBLE-ish** — phiếu này flip entrypoint dispatch + flip repo contract (`CLAUDE.md` "Not a runtime binary source"). Sau merge, `bin/sos.sh` cho 6 lệnh nặng CHỈ chạy Rust binary. Rollback = `git revert` (xem "Rollback plan"). **BẮT BUỘC founder confirm trước EXECUTE** (decomposition §Founder-decision points).

---

> **Loại:** Feature (cutover)
> **Ưu tiên:** P1
> **Tầng:** 1 — flip repo contract + entrypoint dispatch + irreversible-ish. AUTO Tầng 1 (contract surface + không-đảo).
> **Lane:** Guarded — cutover đa-file, contract-surface, no-cap. (Architect override token: `Guarded`.)
> **Ảnh hưởng:** `bin/sos.sh` (dispatch), `CLAUDE.md` (contract), `docs/PORTABILITY_ARCHITECTURE.md`, `bootstrap/sos-rs/README.md`, `docs/SETUP.md`, `CHANGELOG.md`
> **Dependency:** P077a–P077d3 (tất cả merged — 6 lệnh nặng đã parity/impl trong Rust). **KHÔNG** dependency lên P077f (relocate) — relocate tách ra sau.

---

## Context

### Vấn đề hiện tại

P077a→P077d3 dựng Rust `sos` binary SONG SONG, additive: `bin/sos.sh` (Bash) vẫn canonical, user-facing behavior không đổi. 6 lệnh nặng (`new/adopt/map/sync/install/tools`) nay đã có bản Rust parity/impl (P077c parity suite xanh; P077d2 install engine; P077d3 tool-manifest). Nhưng:

- **Hai canonical engines** vẫn tồn tại song song — vi phạm PORTABILITY non-goal "Không giữ Bash và Rust là hai canonical implementations sau parity" (OA-06).
- **Bash `sos map` vẫn mang bug OA-02** (false-green coverage) by design — chỉ Rust path đã fix (P077c5). Người dùng `bin/sos.sh` hiện vẫn gặp bug tới cutover.
- **`CLAUDE.md` contract** ("Not a runtime binary source", Rules #1 "No runtime code") mâu thuẫn thực tại: repo NAY chứa runtime source (`bootstrap/sos-rs/` workspace) và sắp dùng nó làm canonical. `bootstrap/sos-rs/README.md` ownership contract cũ cũng mâu thuẫn target (finding #5).

### Giải pháp — approach (A) đã founder-confirm

**6 lệnh nặng → Rust `sos` binary canonical. 7 lệnh guidance → GIỮ Bash trong `bin/sos.sh`** (Claude-flavored, thuộc adapter → render per-runtime ở P078).

1. **Dispatch flip** trong `bin/sos.sh` case block: 6 lệnh nặng → `exec` Rust `sos` binary với nguyên args; 7 lệnh guidance → giữ `sos_<cmd>` Bash function.
2. **Binary resolution** fail-LOUD (KHÔNG silent-fallback về Bash — fallback = ship lại bug OA-02).
3. **Flip repo contract** `CLAUDE.md` + `bootstrap/sos-rs/README.md` ownership + `docs/PORTABILITY_ARCHITECTURE.md` note.
4. **Parity-matrix gate** trước khi coi cutover xong (OA-10 System-level acceptance subset áp dụng cho 6 heavy cmd).

**KHÔNG trong phiếu này (tách ra):**
- **Workspace relocate** `bootstrap/sos-rs/` → repo-root `Cargo.toml`: ORTHOGONAL với cutover, churn path lớn (mọi Cargo path + CLAUDE.md repo-structure + mọi ref `bootstrap/sos-rs/` trong docs/tickets), và **reversible** — KHÔNG nên hàn chung với contract-flip irreversible. → **defer P077f** (founder xác nhận split; xem "Founder-decision" #3). Cutover này GIỮ workspace tại `bootstrap/sos-rs/` (transitional root).
- **`install.sh` prebuilt-binary bootstrap**: PORTABILITY dòng 104 xếp "Bootstrap chỉ tải binary `sos`" vào **P081** (distribution). Cutover này KHÔNG đổi `install.sh` (xem "Founder-decision" #2).

### Scope
- CHỈ sửa: `bin/sos.sh` (dispatch case block + binary-resolver helper), `CLAUDE.md` (contract lines), `bootstrap/sos-rs/README.md` (ownership), `docs/PORTABILITY_ARCHITECTURE.md` (1 status note), `docs/SETUP.md` (dispatch note), `CHANGELOG.md`.
- KHÔNG sửa: `install.sh` (zero-touch — P081), workspace Cargo paths (relocate = P077f), Rust source (đã impl P077a-d), 7 guidance `sos_<cmd>` Bash fns (giữ nguyên).

---

## Task 0 — Verification Anchors

> Architect envelope = docs-only (no Bash/Grep). Anchors về `bin/sos.sh`/`install.sh` là `[needs Worker verify]` — orchestrator recon cấp line-hint, Worker grep xác nhận trước EXECUTE. Anchors về `CLAUDE.md` là `[verified]` (nội dung CLAUDE.md có trong session context).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `bin/sos.sh` có dispatch case block quanh `:1276-1286`, mỗi arm gọi `sos_<cmd> "$@"` | `grep -n 'sos_.*"\$@"' bin/sos.sh` + đọc case block | ✅ VERIFIED — arms at :1275-1289 (grew from :1276-1286), 11 named arms called `sos_<cmd> "$@"` + help/catch-all |
| 2 | Tập lệnh dispatch THỰC TẾ = ? Recon nói "11 cmd" nhưng 6 heavy + 7 guidance = 13. **Worker phải liệt kê ĐỦ mọi arm và phân loại heavy/guidance từng cái** trước khi flip (thiếu 1 arm → lệnh đó route sai) | `grep -nE '^\s*(new\|adopt\|sync\|map\|install\|tools\|init\|blueprint\|contract\|apply\|recipe\|launch\|status\|state)\)' bin/sos.sh` | ✅ VERIFIED — 11 named arms confirmed (NOT 13). `install`/`tools` had ZERO Bash fn/arm — added as NEW arms, not flipped |
| 3 | 6 lệnh nặng ĐỀU có bản Rust trong `sos` binary: `new/adopt/map/sync` (P077c), `install` (P077d2), `tools status` (P077d3) | `sos new --help; sos adopt --help; sos map --help; sos sync --help; sos install --help; sos tools status --help` → exit 0, không "unknown subcommand" | ✅ VERIFIED — all 6 `--help` exit 0 on Rust binary |
| 4 | Rust binary tên `sos` (crate `sos-cli`). Build path dev = `~/.cargo-target-shared/debug/sos` (Denny redirect `CARGO_TARGET_DIR`). Repo chuẩn = `<workspace>/target/{release,debug}/sos` | `ls -la ~/.cargo-target-shared/debug/sos` + `grep -n '^name\|\[\[bin\]\]' bootstrap/sos-rs/crates/sos-cli/Cargo.toml` | ✅ VERIFIED — binary at exact path; Cargo.toml name=sos-cli, [[bin]] name=sos |
| 5 | ⚠️ **Name collision**: launcher `bin/sos.sh` được cài LÀM `sos` trên PATH. Nếu resolver dùng `command -v sos` → tự-đệ-quy vào launcher. Resolver PHẢI dùng path tường minh, KHÔNG `command -v sos` | Đọc `install.sh` xem launcher cài dưới tên gì trên PATH | ✅ VERIFIED — install.sh:151-158 installs launcher AS `sos` on PATH; resolver avoids `command -v sos` |
| 6 | `install.sh` cài `bin/sos.sh` làm launcher + tải sister-tool binaries (`releases/latest`); KHÔNG build/cài Rust `sos` binary | `grep -nE 'sos\.sh\|releases/latest\|cargo (build\|install)' install.sh` | ✅ VERIFIED — install.sh has zero cargo build/install of Rust sos; zero-touch confirmed |
| 7 | `CLAUDE.md` "## What this repo is NOT" mục #1 = "**Not a runtime binary source.** The Rust CLIs (`ship`, `docs-gate`, `guard`, `vps`) live in their own repos. This repo only references them." | (có trong context) | ✅ `[verified]` |
| 8 | `CLAUDE.md` Rules #1 = "`[judgment]` **No runtime code in this repo.** Rust source belongs in their own repos … This repo is documentation, templates, and skill markdown only. `phieu/phieu.sh` is an exception …" | (có trong context) | ✅ `[verified]` |
| 9 | `CLAUDE.md` "## Repo structure" có dòng `│   └── sos-rs/           # Rust CLI source skeleton (bootstrap target)` dưới `bootstrap/` | (có trong context) | ✅ `[verified]` |
| 10 | `bootstrap/sos-rs/README.md` chứa ownership/extraction contract cũ mâu thuẫn target (finding #5) | `grep -niE 'extract\|own\|separate repo' bootstrap/sos-rs/README.md` | ✅ VERIFIED — README.md:1,3,90 had old skeleton/parity-oracle wording; flipped in Task 6 |
| 11 | Parity/fixture test target: `crates/sos-cli/tests/parity.rs` (P077c) + install fixtures `crates/sos-install/tests/install.rs` (P077d2) | `ls bootstrap/sos-rs/crates/*/tests/*.rs` | ✅ VERIFIED — 5 test files exist incl. parity.rs + install.rs + tools.rs |
| 12 | `docs-gate --all` trên sos-kit-self hiện FAIL vì CHANGELOG `changelog_max_age_days` — Worker phải thêm CHANGELOG entry fresh để gate PASS post-flip (audit dòng 43) | `docs-gate --all` | ✅ VERIFIED — docs-gate --all was already 26/26 PASS pre-flip; still 26/26 PASS post-flip (CHANGELOG entry added) |

**❌/⚠️ acknowledged:** Anchor #2 (11 vs 13) và #5 (name collision) là 2 rủi ro CAO. Nhiệm vụ dưới xử lý cả hai tường minh.

### Pre-phiếu snapshot (Worker auto first-step)

> IRREVERSIBLE-ish → snapshot BẮT BUỘC trước mọi edit.

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+[a-z]*')
mkdir -p ".backup/${PHIEU_ID}"
cp bin/sos.sh ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp CLAUDE.md ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Worker accepted V1 — no challenges.** All 12 Task 0 anchors verified ✅ (live grep/binary checks — see CHALLENGE report). **Anchor #2 critical finding confirmed**: dispatch had 11 named arms (not 13) — `install`/`tools` had ZERO existing Bash arm/function; cutover correctly ADDS 2 new arms for them rather than flipping existing ones (per phiếu's own Task 2 contingency note). Anchor #5 (name-collision) confirmed real via `install.sh:151-158` (launcher installs itself as `sos` on PATH) — resolver design correctly avoids `command -v sos`. Ready for founder approval.

**Status:** ✅ APPROVED V1 — founder (Sếp) confirmed EXECUTE, approach (A). See Final consensus.

### Final consensus
- Phiếu version: V1
- Approved by founder (Sếp): 2026-07-22 — IRREVERSIBLE-ish cutover (git-revertible), execution proceeded on branch `P077e-cutover`

---

## Nhiệm vụ

### Task 1: Binary-resolver helper trong `bin/sos.sh`

**File:** `bin/sos.sh`

**Tìm:** vùng đầu file (sau shebang/set-flags, trước dispatch case block). Worker xác định vị trí chèn helper `[needs Worker verify]`.

**Thêm:** một helper resolve Rust `sos` binary theo precedence **fail-LOUD**:

```
# Resolve the canonical Rust `sos` binary (post-P077e cutover).
# MUST NOT resolve via `command -v sos` — this launcher IS `sos` on PATH (recursion). Anchor #5.
_sos_rust_bin() {
  # 1. Explicit override (dev/CI/testing)
  if [ -n "${SOS_RUST_BIN:-}" ] && [ -x "${SOS_RUST_BIN}" ]; then
    printf '%s\n' "${SOS_RUST_BIN}"; return 0
  fi
  # 2. Known build outputs (honor CARGO_TARGET_DIR redirect, then workspace-local target/)
  local ws candidates c
  ws="$(_sos_workspace_root)"   # Worker: helper trả về bootstrap/sos-rs (transitional root này phiếu)
  candidates="${CARGO_TARGET_DIR:-}/release/sos ${CARGO_TARGET_DIR:-}/debug/sos ${ws}/target/release/sos ${ws}/target/debug/sos"
  for c in $candidates; do
    [ -x "$c" ] && { printf '%s\n' "$c"; return 0; }
  done
  # 3. Build-on-demand fallback (dev/dogfood: cargo present). Prints binary path on success.
  if command -v cargo >/dev/null 2>&1; then
    (cd "$ws" && cargo build --quiet --bin sos) >&2 || return 1
    for c in $candidates; do [ -x "$c" ] && { printf '%s\n' "$c"; return 0; }; done
  fi
  return 1
}
```

**Lưu ý:**
- Path chính xác của workspace root + build dir là `[needs Worker verify]` — Denny redirect `CARGO_TARGET_DIR=~/.cargo-target-shared`; probe cả nó lẫn `<ws>/target`. Worker viết `_sos_workspace_root` trỏ `bootstrap/sos-rs` (transitional).
- **KHÔNG** thêm nhánh silent-fallback về `sos_<cmd>` Bash cho lệnh nặng — thà fail loud (xem Task 2 error).
- Exact helper name / var name = **Tầng 2, Worker tự quyết** (local shell detail). Contract cứng = precedence + fail-loud + no `command -v sos`.

### Task 2: Dispatch flip — 6 lệnh nặng `exec` Rust, 7 guidance giữ Bash

**File:** `bin/sos.sh`

**Tìm:** dispatch case block `~:1276-1286` (Anchor #1), mỗi arm hiện gọi `sos_<cmd> "$@"`.

**Thay bằng:** cho 6 arm **nặng** (`new`, `adopt`, `sync`, `map`, `install`, `tools`) — resolve binary rồi `exec`; guidance arms giữ nguyên. Pattern mỗi heavy arm:

```
  new|adopt|sync|map|install|tools)
    _bin="$(_sos_rust_bin)" || {
      echo "sos: Rust binary 'sos' not found — cutover requires it (P077e)." >&2
      echo "  Fix: build in $(_sos_workspace_root) (\`cargo build --bin sos\`) or set SOS_RUST_BIN=/path/to/sos." >&2
      echo "  (Heavy commands no longer fall back to Bash — the Bash 'map' still carries bug OA-02.)" >&2
      exit 127
    }
    exec "$_bin" "$SUBCMD" "$@"
    ;;
```

**Lưu ý:**
- **Anchor #2 reconcile FIRST**: Worker liệt kê ĐỦ mọi arm thật. Nếu tập ≠ {6 heavy + 7 guidance}, hoặc có arm lạ (`state`, `help`, `--version`, catch-all `*)`) → phân loại từng cái: heavy semantics → route Rust; guidance/meta → giữ Bash. **Nếu gặp lệnh không rõ heavy hay guidance → DISCOVERY_REPORT + escalate, KHÔNG tự đoán** (route sai lệnh nặng = ship bug).
- `SUBCMD`/`"$@"` splitting: Worker verify cách case block hiện lấy subcmd name vs args (biến nào giữ subcmd) `[needs Worker verify]` — `exec` phải truyền `<subcmd> <args…>` khớp CLI Rust. Giữ nguyên quoting `"$@"`.
- `exec` thay process → exit code Rust propagate tự nhiên (không cần forward `$?`).
- 7 guidance arms (`init/blueprint/contract/apply/recipe/launch/status`) **KHÔNG đụng**.
- Bash `sos_new/sos_adopt/sos_sync/sos_map/sos_install/sos_tools` functions: **GIỮ DORMANT** (không xóa) — thêm comment `# DEPRECATED post-P077e: dispatched to Rust binary; retained for git-revert safety + oracle ref. Delete at P077f/P081.` Giữ để rollback rẻ + oracle còn sống 1 chu kỳ. (Xóa = Tầng 1 vì chạm rollback surface — KHÔNG xóa phiếu này.)

### Task 3: Flip `CLAUDE.md` "What this repo is NOT" #1

**File:** `CLAUDE.md`

**Tìm (Anchor #7 `[verified]`):**
```
- **Not a runtime binary source.** The Rust CLIs (`ship`, `docs-gate`, `guard`, `vps`) live in their own repos. This repo only references them.
```

**Thay bằng:**
```
- **Runtime monorepo (as of P077e).** This repo now contains the canonical runtime source for the `sos` CLI — a Rust workspace at `bootstrap/sos-rs/` (crates `sos-cli`/`sos-core`/`sos-install`/`sos-adapter-claude`/`sos-hooks`). The 6 heavy `sos` subcommands (`new`/`adopt`/`sync`/`map`/`install`/`tools`) dispatch to this binary; `bin/sos.sh` is a thin launcher that keeps only the 7 Claude-flavored guidance commands (`init`/`blueprint`/`contract`/`apply`/`recipe`/`launch`/`status`) in Bash until P078 renders them per-runtime. The **sister** CLIs (`ship`, `docs-gate`, `guard`, `vps`) STILL live in their own repos (`~/ship` etc.) — this repo references + version-pins them via `tool-manifest.toml`, it does not vendor their source.
```

**Lưu ý:** Nuance — flip chỉ áp cho `sos` binary source; sister tools vẫn external. KHÔNG over-claim "mọi runtime ở đây".

### Task 4: Amend `CLAUDE.md` Rules #1

**File:** `CLAUDE.md`

**Tìm (Anchor #8 `[verified]`):**
```
1. `[judgment]` **No runtime code in this repo.** Rust source belongs in their own repos (`ship`, `docs-gate`, `guard`, `vps`). This repo is documentation, templates, and skill markdown only. `phieu/phieu.sh` is an exception — a single shell function file users source — but it does no computation beyond git and file ops.
```

**Thay bằng:**
```
1. `[judgment]` **The `sos` CLI runtime source lives here; sister-tool source does not.** Since P077e this repo IS the canonical Rust workspace for the `sos` binary (`bootstrap/sos-rs/`). Sister CLIs (`ship`, `docs-gate`, `guard`, `vps`) still belong in their own repos — do not vendor their source here. Beyond the `sos` workspace, this repo stays documentation, templates, and skill markdown. `phieu/phieu.sh` and `bin/sos.sh` are Bash exceptions (a sourced shell function + the thin launcher) doing only git/file ops + dispatch.
```

### Task 5: `CLAUDE.md` "Repo structure" — bootstrap/sos-rs không còn "skeleton/bootstrap target"

**File:** `CLAUDE.md`

**Tìm (Anchor #9 `[verified]`):**
```
│   └── sos-rs/           # Rust CLI source skeleton (bootstrap target)
```

**Thay bằng:**
```
│   └── sos-rs/           # Canonical Rust workspace for the `sos` binary (P077e cutover). Heavy subcommands dispatch here. Relocate to repo-root Cargo.toml = P077f.
```

**Lưu ý:** cũng cập nhật 1 câu ở dòng mô tả `bin/sos.sh` trong repo structure nếu có (Worker verify `[needs Worker verify]`) — ghi rõ nay là "thin launcher: heavy→Rust, guidance→Bash".

### Task 6: `bootstrap/sos-rs/README.md` — flip ownership (finding #5)

**File:** `bootstrap/sos-rs/README.md`

**Tìm:** đoạn ownership/extraction contract cũ (Anchor #10 `[needs Worker verify]` — nói workspace sẽ extract sang repo riêng / là skeleton).

**Thay bằng:** khẳng định workspace là canonical runtime của sos-kit, KHÔNG extract; giữ tại `bootstrap/sos-rs/` transitional, relocate repo-root = P077f; heavy `sos` subcommands dispatch vào binary này kể từ P077e.

**Lưu ý:** wording cụ thể = Worker soạn từ nội dung thật của README (Architect không đọc được). Contract cứng: "canonical, không extract, relocate=P077f".

### Task 7: `docs/PORTABILITY_ARCHITECTURE.md` + `docs/SETUP.md` status note

**File:** `docs/PORTABILITY_ARCHITECTURE.md`

**Tìm:** cụm status-line P077* (quanh dòng 41-52) hoặc migration table dòng 155 (P077 row).

**Thêm:** 1 status note **P077e status (CUTOVER live)**: Rust `sos` canonical cho 6 heavy cmd; `bin/sos.sh` thin launcher (heavy→exec Rust, 7 guidance→Bash tới P078); repo contract flipped; workspace GIỮ `bootstrap/sos-rs/` (relocate=P077f, deferred); `install.sh` prebuilt-binary bootstrap = P081 (chưa đổi). Style khớp các status-line P077b/d hiện có.

**File:** `docs/SETUP.md`

**Tìm/Thêm:** `[needs Worker verify]` — nếu SETUP mô tả `sos <cmd>` hoặc install launcher, thêm 1 note: 6 heavy cmd nay cần Rust `sos` binary (dev: `cargo build --bin sos` trong `bootstrap/sos-rs`, hoặc `SOS_RUST_BIN`); prebuilt bootstrap = P081.

### Task 8: `CHANGELOG.md`

**File:** `CHANGELOG.md`

**Thêm** entry mới nhất trên cùng: `[P077e]` — CUTOVER: Rust `sos` canonical cho 6 heavy subcommand; `bin/sos.sh` thin launcher; repo contract flip (runtime monorepo); workspace tại `bootstrap/sos-rs` (relocate deferred P077f); `install.sh` zero-touch (prebuilt bootstrap P081). Entry fresh giải luôn Anchor #12 (docs-gate changelog age).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `bin/sos.sh` | Task 1 (resolver helper) + Task 2 (dispatch flip 6 heavy → exec Rust; guidance giữ; heavy Bash fns dormant+deprecated) |
| `CLAUDE.md` | Task 3 (What is NOT #1) + Task 4 (Rules #1) + Task 5 (repo structure) |
| `bootstrap/sos-rs/README.md` | Task 6 (ownership flip) |
| `docs/PORTABILITY_ARCHITECTURE.md` | Task 7 (P077e status note) |
| `docs/SETUP.md` | Task 7 (dispatch/build note) |
| `CHANGELOG.md` | Task 8 (`[P077e]` entry) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `install.sh` | ZERO-TOUCH. Xác nhận không cần đổi (launcher + sister-tool download nguyên trạng); prebuilt `sos` bootstrap = P081 |
| `bootstrap/sos-rs/**/*.rs` (source) | KHÔNG đổi — 6 heavy cmd đã impl P077a-d3; cutover chỉ đổi dispatch |
| `bootstrap/sos-rs/Cargo.toml` + mọi path Cargo | KHÔNG relocate — repo-root Cargo.toml = P077f |
| 7 guidance `sos_<cmd>` Bash fns | Vẫn chạy Bash sau cutover (init/blueprint/contract/apply/recipe/launch/status) |

---

## Luật chơi (Constraints)

1. **Fail LOUD, KHÔNG silent-fallback.** Lệnh nặng không resolve được Rust binary → exit non-zero + message rõ. TUYỆT ĐỐI không rơi ngược về `sos_<cmd>` Bash (Bash `map` còn bug OA-02 → fallback = ship bug thầm lặng).
2. **KHÔNG `command -v sos`** trong resolver (launcher tự tên `sos` → recursion). Chỉ path tường minh / `SOS_RUST_BIN` / build dir.
3. **Reconcile Anchor #2 TRƯỚC khi flip.** Liệt kê đủ mọi dispatch arm, phân loại từng cái. Lệnh không rõ heavy/guidance → escalate, KHÔNG đoán.
4. **KHÔNG xóa** Bash heavy fns (dormant + deprecated comment) — rollback safety + oracle. Xóa = Tầng 1, phiếu khác.
5. **KHÔNG relocate workspace** trong phiếu này (P077f).
6. **KHÔNG đổi `install.sh`** (P081).
7. Contract-flip (`CLAUDE.md`) là contract-surface → BẮT BUỘC qua CHALLENGE (Guarded + Tầng 1 auto-flow, CLAUDE.md Rule #9).
8. Guidance commands (7) — behavior không đổi 1 ly.

---

## Nghiệm thu

### Automated
- [ ] `cd bootstrap/sos-rs && cargo build --bin sos` clean.
- [ ] **Parity/flaky gate:** `cargo test --workspace` chạy **×20** = 0 fail, 0 flaky (parity.rs + install.rs). Oracle chống parity regress.
- [ ] `bash -n bin/sos.sh` PASS.
- [ ] `docs-gate --all` **PASS** sau contract flip (Anchor #12 — CHANGELOG entry fresh).

### Manual Testing — Cutover smoke (heavy cmd via launcher == Rust binary == golden)
- [ ] Mỗi lệnh nặng gọi qua `bin/sos.sh <cmd> …` và gọi trực tiếp `$(_sos_rust_bin) <cmd> …` → **output + exit code KHỚP** (đó là bằng chứng đã dispatch sang Rust, không còn Bash).
- [ ] Resolver fail-loud: tạm `SOS_RUST_BIN=/nonexistent` + xóa build dir → `bin/sos.sh new …` exit non-zero + message rõ, **KHÔNG** chạy Bash `sos_new`.
- [ ] 1 guidance cmd (vd `bin/sos.sh status`) vẫn chạy Bash như cũ.
- [ ] Name-collision: xác nhận launcher trên PATH không tự-đệ-quy (resolver không dùng `command -v sos`).

### Fixture matrix (OA-10 System-level acceptance — subset áp dụng 6 heavy cmd)
- [ ] **Greenfield Rust:** `sos new` → repo buildable → verify-setup PASS → docs-gate PASS (nay chạy Rust path).
- [ ] **Brownfield Rust/Python/TS:** `sos adopt` non-clobber, manifest gốc giữ nguyên; `sos map` map **source thật** (`src/**`) + kit assets excluded (OA-02 Rust fix live, KHÔNG còn false-green); `validate-map` PASS.
- [ ] Rows Codex-only / Dual-runtime / Monorepo-code-story = **out of scope** (P078+); Dirty-tree ship = OA-03 (ngoài phiếu).

### Regression
- [ ] 7 guidance commands behavior không đổi.
- [ ] `sos tools status` fail-closed đúng khi required tool drift/missing (P077d3 behavior giữ qua launcher).

### Rollback plan (IRREVERSIBLE-ish)
- Cutover = 1 commit trên branch phiếu. Nếu smoke/gate fail sau merge:
  - `git revert <cutover-commit>` → khôi phục dispatch Bash + contract cũ (heavy Bash fns còn dormant nên revert trivial, không cần re-impl).
  - Trong worktree (chưa merge): `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` HOẶC `cp .backup/${PHIEU_ID}/{sos.sh,CLAUDE.md} …`.
- **NEVER** reset trên main.

### Docs Gate (Tầng 1 — BẮT BUỘC)
- [ ] `CLAUDE.md` — contract (What-is-NOT #1 + Rules #1 + repo structure) updated.
- [ ] `bootstrap/sos-rs/README.md` — ownership flip (finding #5).
- [ ] `docs/PORTABILITY_ARCHITECTURE.md` — P077e status note.
- [ ] `docs/SETUP.md` — dispatch/build note.
- [ ] `CHANGELOG.md` — `[P077e]` entry.
- [ ] Discovery ghi: "Tầng 1 docs updated: CLAUDE.md, bootstrap/sos-rs/README.md, PORTABILITY_ARCHITECTURE.md, SETUP.md, CHANGELOG.md".

### Discovery Report
- [ ] `docs/discoveries/P077e.md`:
  - Anchor #2 reconcile: tập dispatch arm THỰC TẾ + phân loại (11 vs 13 giải ra sao).
  - Anchor #5: resolver dùng cơ chế gì chống recursion.
  - Assumptions CORRECT/WRONG (file:line).
  - Bash heavy fns: giữ dormant hay có lệnh nào buộc phải xử lý khác.
  - Founder decisions confirmed (relocate defer, install.sh zero-touch).
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
