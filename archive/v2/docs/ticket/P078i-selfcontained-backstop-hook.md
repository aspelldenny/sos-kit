# PHIẾU P078i: self-contained backstop hook (đóng hook dependency-closure)

---

> **Loại:** Bugfix (security backstop)
> **Ưu tiên:** P0
> **Tầng:** 1 (security backstop — install-time invariant enforcement; sai thì `.env`/code-on-default LỌT ở mọi brownfield repo → AUTO Tầng 1)
> **Lane:** Guarded
> **Ảnh hưởng:** `crates/sos-install/src/engine.rs` (render path), 2 embedded guard scripts, minimal backstop `pre-commit`
> **Dependency:** None (song song P078j — actor-check/path-normalize; KHÔNG overlap surface)

---

## Context

### Vấn đề hiện tại

P079 round-4 dogfood (`docs/adapters/P079-ROUND4-FINDINGS-2026-07-23.md`, Test A3/A4 FAIL, gaps §"New/remaining gaps" #1 dòng 204-211): P078g render hook entrypoint + arm `core.hooksPath`, **NHƯNG hook đó là hook dev `[8/8]` ĐẦY ĐỦ của sos-kit** — delegate mọi phase sang `scripts/*.sh` + `docs-gate` binary, và **fail-OPEN khi script vắng**:

```text
[7/8] Block .env* commit
  scripts/block-env-commit.sh missing — run scripts/install-hooks.sh after bootstrap
Commit allowed.            ← .env LỌT (ENV_COMMIT_EXIT=0)
```

Repo user brownfield (Codex adapter cài vào) KHÔNG có cây `scripts/` đó → `.env` commit + code-on-default commit **lọt hết** (round-4 A3 `ENV_COMMITTED=yes`, A4 `CODE_COMMITTED=yes`).

**Structural-oracle-gap lần thứ 3:** P078f seed hook trong fixture (bỏ lọt "install không render hook"); P078g seed `scripts/block-env-commit.sh` trong fixture (bỏ lọt "install không render script hook GỌI"). Test cứ seed đúng cái install thật thiếu → oracle mù đệ quy. Phiếu này đóng vòng đệ quy bằng **dependency-closure assertion** (oracle grep hook đã render, mọi ref phải nằm trong tập file render).

### Giải pháp

`sos install` (adapter, vào repo tùy ý) render một **hook backstop TỰ CHỨA tối thiểu** — KHÔNG phải hook dev `[8/8]`:

1. Render một `pre-commit` tối thiểu (mới, purpose-built) enforce **CHỈ 2 invariant bảo mật**: `.env` block + no-code-on-default.
2. **Route tái dùng logic đã test:** embed 2 script HIỆN CÓ verbatim (`scripts/block-env-commit.sh` + `scripts/no-code-on-default.sh`) qua `include_str!` làm artifact, render vào target `scripts/`, minimal backstop `pre-commit` gọi chúng.
3. **fail-CLOSED:** guard bắt buộc vắng → BLOCK (exit≠0), KHÔNG fail-open như hook `[8/8]`.
4. Closure = 3 file embedded (minimal pre-commit + 2 guard), KHÔNG kéo `docs-gate`/`trust-gate`/python/type-check.

Hook dev `[8/8]` đầy đủ GIỮ NGUYÊN cho `sos new` (dev project có đủ tooling) — KHÔNG đụng path đó.

### Scope
- CHỈ sửa: `crates/sos-install/src/engine.rs` render path (embed + render 3 backstop artifact); thêm minimal backstop `pre-commit` template.
- KHÔNG sửa: `sos new` full-hook path (new.rs), actor-check/path-normalize (`crates/sos-adapter-codex/**` = P078j), F09 hijack-guard/non-clobber (round-4 A5 PASS), `hooks/pre-commit` dev `[8/8]` bản gốc, docs-gate/trust-gate/security-gate.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `hooks/pre-commit` `[6/8]` gọi `scripts/no-code-on-default.sh`, `[7/8]` gọi `scripts/block-env-commit.sh`, mỗi phase `if [ -f script ]; then …; else echo "missing"; fi` = fail-open (in "missing" rồi cho commit qua) | `grep -n "missing\|block-env-commit\|no-code-on-default" hooks/pre-commit` | ✅ `[verified: hooks/pre-commit:260-291 — [6/8] line 262-271, [7/8] line 280-291, both else-branch echoes "missing" + falls through, no exit]` — round-4 A3/A4 confirm fail-open |
| 2 | `scripts/block-env-commit.sh` (~54 dòng) self-contained: `git diff --cached` + basename regex, tự cd repo-root, có merge-commit escape + override marker; KHÔNG source file khác | `grep -n "source\|\\. \|install-hooks\|docs-gate" scripts/block-env-commit.sh` → expect KHÔNG có transitive dep | ✅ `[verified: grep 0 hits — no source/install-hooks/docs-gate ref]` |
| 3 | `scripts/no-code-on-default.sh` (~117 dòng) self-contained: KHÔNG source/gọi binary ngoài (docs-gate/inv-gate/python) — nếu CÓ transitive dep thì closure vỡ, phải xử | `grep -n "source\|\\. /\|docs-gate\|inv-gate\|python\|\\.sh" scripts/no-code-on-default.sh` → expect no external call | ✅ `[verified: no-code-on-default.sh:79 "python" is a regex-fragment string literal (extension match), :58/:87 "orchestrator-guard.sh" only in comments — 0 real transitive dep. CLOSURE CONFIRMED — matches orchestrator ammo]` |
| 4 | P078g embed hook qua `include_str!` trong `crates/sos-install/src/engine.rs` (`EMBEDDED_PRE_COMMIT`/`EMBEDDED_PRE_PUSH`), render trong `render_embedded_hooks()` | `grep -n "include_str!\|EMBEDDED_PRE_COMMIT\|render_embedded_hooks" crates/sos-install/src/engine.rs` | ✅ `[verified: engine.rs:485-486 const EMBEDDED_PRE_COMMIT/EMBEDDED_PRE_PUSH = include_str!("../../../hooks/pre-commit"/"pre-push") — 3-level ../../../ from crates/sos-install/src/ to repo-root; render_embedded_hooks() fn at :498-523, non-clobber same-content-skip / diff-content-warn-leave pattern to reuse for Task 3]` |
| 5 | `sos new` (new.rs) copy full `scripts/` tree cho dev project qua path KHÁC (không đụng install render path) | `grep -rn "scripts/\|copy.*scripts" crates/sos-install/src/new.rs` | ✅ `[verified — file actually at crates/sos-cli/src/commands/new.rs (Anchor description had wrong crate path, doesn't affect scope): line 312 copy_tree(&kit.join("scripts"), &target_dir.join("scripts"), false) — runtime dir-copy from SOS_KIT_DIR, NOT include_str! embed. Zero overlap with install engine.rs render path.]` |
| 6 | Fresh git repo PRISTINE + `sos install --runtime codex` hiện KHÔNG render `scripts/block-env-commit.sh` / `scripts/no-code-on-default.sh` (18-file plan thiếu) | round-4 findings dòng 62,77 + gap #1 dòng 206-211 | ✅ `[verified: round-4 findings]` |
| 7 (add-on) | Install-plan "created: 18" count không hardcode — tự tăng khi Task 3 thêm 3 file | `grep -n "created.len\|created:" crates/sos-cli/src/commands/install.rs` | ✅ `[verified: install.rs:136 println!("  created: {}", report.created.len()) — dynamic Vec::len(), engine.rs:239 report.created.push(d.target_path) per render — Task 3's 3 new artifacts auto-increment, no separate counter to update]` |

**❌/⏳ handling:** Anchor #3 closure-critical — nếu Worker grep thấy `no-code-on-default.sh` gọi binary ngoài (docs-gate/python), báo Discovery + hoặc (a) inline logic 2 check trực tiếp vào minimal pre-commit, hoặc (b) chỉ embed subset self-contained. Anchor #2/#4/#5 xác nhận before edit.

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

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Worker accepted V1 — no Tầng-1 objections.** All 6 anchors + 1 add-on ✅ (see Task 0 table). Anchor #3 closure-critical CONFIRMED clean (0 transitive dep — "python" is regex-fragment string, not a call; "orchestrator-guard.sh" mention only in a comment) — matches orchestrator pre-verify ammo exactly. Route (a) (minimal pre-commit GỌI 2 embedded script, per Task 2) is feasible and preferred; no evidence route (a) is broken.

**Engine points for EXECUTE:**
- Task 1 embed consts: mirror `engine.rs:485-486` pattern (3-level `../../../` relative path from `crates/sos-install/src/` to repo-root `scripts/`).
- Task 3 render fn: extend `render_embedded_hooks()` at `engine.rs:498-523` — same non-clobber logic (skip if content-identical, warn-leave if different) applies to the 2 new script targets + the new minimal `pre-commit` (note: minimal backstop `pre-commit` target path COLLIDES with dev `[8/8]` embed target `hooks/pre-commit` — Task needs a naming/dispatch decision: is backstop rendered to the SAME `hooks/pre-commit` path as dev hook, or a different one? Phiếu Scope says "hook dev `[8/8]` GIỮ NGUYÊN cho `sos new`" and install path renders backstop — since `sos install` and `sos new` are different commands/paths, no runtime collision expected, but Worker EXECUTE should confirm `render_embedded_hooks()` is only called from the install path, not shared with new.rs, before assuming zero-collision).
- Anchor #5 minor correction: file lives at `crates/sos-cli/src/commands/new.rs`, not `crates/sos-install/src/new.rs` as the phiếu's verify-by command assumed — cosmetic (Tầng 2), doesn't change the "no overlap" conclusion.
- Count logic (`install.rs:136`) is dynamic `Vec::len()` — no manual bump needed when Task 3 adds 3 artifacts.

**Status:** ✅ ACCEPTED — ready for Chủ nhà approval gate.

### Final consensus
- Phiếu version: V1 (accepted as-drafted; 7/7 anchors ✅, route (a) viable, 0 Tầng-1 objections)
- Approved by Chủ nhà: 2026-07-23 — self-approved by Quản đốc under delegated sprint authority (P076→P081, self-approve in-scope; Guarded/Tầng-1, no owner-decision, no Codex run required).
- EXECUTE anchors: embed 2 guard const near `engine.rs:485-486`; render in `render_embedded_hooks()` `:498-523`; new.rs = `crates/sos-cli/src/commands/new.rs:312` (`copy_tree` scripts/, zero overlap with install embed path); plan count `install.rs:136` = `report.created.len()` (auto-increments). Confirm-before-code: `render_embedded_hooks()` reachable only from install path.

---

## Nhiệm vụ

### Task 1: Embed 2 guard script verbatim làm artifact

**File:** `crates/sos-install/src/engine.rs` `[needs Worker verify path/const naming]`

**Tìm:** khối `include_str!` P078g embed hook (const `EMBEDDED_PRE_COMMIT` / `EMBEDDED_PRE_PUSH`) `[needs Worker verify]`.

**Thêm:** 2 const embed script hiện có verbatim (single-source, KHÔNG re-implement):
```
const EMBEDDED_BLOCK_ENV: &str = include_str!("../../../scripts/block-env-commit.sh");
const EMBEDDED_NO_CODE_DEFAULT: &str = include_str!("../../../scripts/no-code-on-default.sh");
```
`[needs Worker verify — relative path từ engine.rs tới repo-root scripts/; dùng cùng độ sâu như include_str! P078g đã có]`

**Lưu ý:** Verbatim embed = single-source với dev `[8/8]`; sửa guard 1 lần, cả 2 path (new + install-backstop) cùng cập nhật. KHÔNG copy-paste logic vào Rust string.

### Task 2: Minimal backstop pre-commit template (purpose-built, fail-CLOSED)

**File:** artifact template mới (Worker chọn vị trí: file riêng embed qua `include_str!` HOẶC inline Rust const) `[needs Worker verify convention P078g dùng]`.

**Nội dung — minimal `pre-commit` enforce CHỈ 2 invariant, fail-CLOSED:**
- Phase 1: `.env` block → gọi `scripts/block-env-commit.sh`.
- Phase 2: no-code-on-default → gọi `scripts/no-code-on-default.sh`.
- **fail-CLOSED:** `if [ ! -f scripts/block-env-commit.sh ]; then echo "BLOCKED: backstop guard missing"; exit 1; fi` — TUYỆT ĐỐI KHÔNG in "missing" rồi `exit 0`.
- KHÔNG có phase docs-gate/trust-gate/type-check/security-gate (đó là dev `[8/8]`, không thuộc backstop).

**Lưu ý:** Đối lập trực tiếp với hook dev `[8/8]` fail-open branch (Anchor #1). Header comment ghi rõ "SOS backstop minimal hook — enforces 2 security invariants only; full [8/8] dev hook via `sos new`".

**Worker chọn:** (a) minimal pre-commit GỌI 2 script embedded [ưu tiên — tái dùng verbatim, ít risk], vs (b) inline logic 2 check TRỰC TIẾP vào pre-commit (bỏ 2 script render, self-contained trong 1 file). Ưu tiên (a) single-source. Ghi lý do Discovery.

### Task 3: Render 3 backstop artifact trong install path

**File:** `crates/sos-install/src/engine.rs` — hàm render hook (`render_embedded_hooks()` hoặc tương đương) `[needs Worker verify fn name]`.

**Tìm:** nơi P078g render `EMBEDDED_PRE_COMMIT` → `hooks/pre-commit` `[needs Worker verify]`.

**Thay bằng:** render backstop set:
- `hooks/pre-commit` ← minimal backstop template (Task 2), executable (mode 0755).
- `scripts/block-env-commit.sh` ← `EMBEDDED_BLOCK_ENV`, executable.
- `scripts/no-code-on-default.sh` ← `EMBEDDED_NO_CODE_DEFAULT`, executable.
- Nếu Worker chọn route (b) inline: chỉ render `hooks/pre-commit`, bỏ 2 script.

**Lưu ý:** Set executable bit cho cả 3 (round-4 A1 confirm P078g đã set cho hook — cùng cơ chế). Non-clobber respect (round-4 A5 PASS — giữ). Nhớ cập nhật install plan count (round-4 "created: 18" → tăng theo số file thêm).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-install/src/engine.rs` | Task 1: embed 2 guard const · Task 3: render 3 backstop artifact + executable |
| minimal backstop `pre-commit` template (vị trí Worker chọn) | Task 2: purpose-built 2-invariant fail-CLOSED hook |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/block-env-commit.sh` | Self-contained (Anchor #2) — embed verbatim, KHÔNG sửa nội dung |
| `scripts/no-code-on-default.sh` | Self-contained, no transitive dep (Anchor #3) — embed verbatim |
| `crates/sos-install/src/new.rs` | `sos new` full-hook path KHÔNG đổi (Anchor #5) |
| `hooks/pre-commit` (dev [8/8]) | Bản gốc GIỮ NGUYÊN — backstop là hook MỚI, không overwrite |
| `crates/sos-adapter-codex/**` | actor-check/path-normalize = P078j, KHÔNG đụng |

---

## Luật chơi (Constraints)

1. Backstop hook enforce ĐÚNG 2 invariant: `.env` block + no-code-on-default. KHÔNG thêm phase.
2. **fail-CLOSED tuyệt đối:** guard vắng → exit≠0. CẤM "missing → commit allowed" (chính là bug round-4).
3. Embed guard VERBATIM qua `include_str!` — CẤM re-implement logic trong Rust string (single-source với dev `[8/8]`).
4. Closure = self-contained: backstop KHÔNG được ref file nào install không render (docs-gate/trust-gate/python/install-hooks.sh). Dependency-closure oracle (Nghiệm thu) phải PASS.
5. `sos new` full-hook path + non-clobber + F09 hijack-guard KHÔNG đổi.
6. Nếu Anchor #3 lộ transitive dep trong `no-code-on-default.sh` → escalate Discovery, chuyển route (b) inline hoặc subset.

---

## Nghiệm thu

### Automated
- [ ] `cargo check` / `cargo build --release` clean.
- [ ] `cargo test` pass.

### Manual Testing — Oracle: fresh PRISTINE repo, KHÔNG seed BẤT KỲ script/hook nào
> Đóng structural-oracle-gap lần 3 — fixture PHẢI pristine, cấm seed cái install thật thiếu.

- [ ] Dựng fresh git repo pristine → `sos install --runtime codex` → `hooks/pre-commit` + 2 guard script render + executable (`ls -l` mode có `x`).
- [ ] **commit thật `.env`** (`echo x > .env; git add -f .env; git commit`) → BLOCK, exit≠0 (round-4 A3 giờ chặn thật).
- [ ] **commit thật product code trên default branch** (`.rs` file, git add, commit trên `main`) → BLOCK, exit≠0 (round-4 A4 giờ chặn thật).
- [ ] **DEPENDENCY-CLOSURE ASSERTION (chặn đệ quy):** grep `hooks/pre-commit` đã render — trích mọi `scripts/xxx` + binary ngoài nó ref; MỌI ref PHẢI ∈ tập file được render. `grep -oE 'scripts/[a-z-]+\.sh|docs-gate|trust-gate|inv-gate|python' hooks/pre-commit` → mỗi hit tồn tại trong repo sau install. Fail nếu ref file install không tạo.
- [ ] **fail-CLOSED khi guard absent:** xoá `scripts/block-env-commit.sh` → commit `.env` → BLOCK (exit≠0), KHÔNG in "commit allowed".
- [ ] **negative-test:** feed `.env` → block; feed file sạch (`README.md`) trên feature branch → commit PASS (không false-block).

### Regression
- [ ] `sos new` full-hook `[8/8]` path unchanged — dev project vẫn render đủ 8 phase.
- [ ] round-4 A5 non-clobber (`core.hooksPath` đã set → refuse) vẫn PASS.
- [ ] F09 hijack-guard vẫn PASS.

### Docs Gate (Tầng 1)
- [ ] `CHANGELOG.md` — entry P078i.
- [ ] `SECURITY.md` — backstop giờ tự-chứa fail-CLOSED enforce 2 invariant (`.env` + no-code-on-default) sau install brownfield; KHÔNG kéo `[8/8]` tooling.
- [ ] `adapters/codex/CAPABILITY.md` — backstop THẬT SỰ enforce sau install brownfield (round-4 A3/A4 giờ chặn).
- [ ] `docs/BACKLOG.md` — mark P078i DONE.

### Discovery Report
- [ ] `docs/discoveries/P078i.md`:
  - Anchor #2-#5 CORRECT/WRONG (file:line).
  - Route chọn (a) gọi-script vs (b) inline — lý do.
  - Anchor #3 kết quả closure-check `no-code-on-default.sh` (có/không transitive dep).
  - Reference round-4 gap A (findings dòng 204-211); note "structural-oracle-gap lần 3 — dependency-closure oracle đóng đệ quy (render X mà X cần Y chưa render)".
  - Tầng-1 docs updated: list.
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
