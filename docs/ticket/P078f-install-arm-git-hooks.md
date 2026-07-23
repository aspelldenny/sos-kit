# PHIẾU P078f: `sos install` phải arm Git hooks (core.hooksPath + F09 hijack-guard)

---

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 1 (security surface — install-time hook-arming LAN tới mọi adopter; sai thì Git boundary OFF hoặc clobber adopter's hook chain → AUTO Tầng 1)
> **Lane:** Guarded
> **Ảnh hưởng:** `crates/sos-install/src/engine.rs`, `crates/sos-cli/src/commands/install.rs`, install-smoke test
> **Dependency:** P078e (SHIPPED `dd4594d`) — **independent surface** (install-engine vs adapter guard); có thể chạy song song / sau. Không có coupling code.

---

## Context

### Vấn đề hiện tại

`sos install` KHÔNG arm Git hooks. Sau install, `git config --local core.hooksPath` vẫn unset → repo `hooks/{pre-commit,pre-push}` không bao giờ chạy → **Git boundary OFF-by-default**.

Điều này rút ruột "honest-MISSING" story của P078d2b: adapter tuyên bố "in-subagent enforcement MISSING trên Codex (custom-role hooks không fire, openai/codex#21753) → rely on universal Git backstop (pre-commit/pre-push)" — nhưng backstop đó tắt. Declared-backstop mà off = bảo vệ giả.

Found by P079 round-2 dogfood (`docs/adapters/P079-ROUND2-FINDINGS-2026-07-23.md`). Split khỏi P078e per P085 security-isolation heuristic (`docs/plans/P078d-decomposition.md` §"Why split round-2 usability → P078e / P078f" + §"P078f items").

### Giải pháp

Port security-arming của `scripts/install-hooks.sh` vào `sos install` — arm-by-default, symmetric cho cả `--runtime claude` và `--runtime codex`. 4 arming step (Worker đọc `install-hooks.sh` full để enumerate chính xác trước khi impl — xem Task 0 anchor #3):

1. **`chmod +x hooks/pre-commit`** (+ `hooks/pre-push` nếu tồn tại) — Git bỏ qua hook không executable.
2. **F09 hijack-guard:** nếu `git config --local core.hooksPath` đã set sang giá trị KHÁC `hooks` → TTY: confirm `[y/N]`; non-TTY: **ABORT (exit 1)**. KHÔNG silently clobber adopter's hook chain.
3. **`git config core.hooksPath hooks`** (local).
4. **Rename** existing `.git/hooks/{pre-commit,pre-push}` → `*.pre-hookspath.bak` (rename, KHÔNG delete — escape hatch cho adopter).

Non-clobber (không đè custom hooksPath), non-git-repo → **warn-skip** (không fail install).

**Engine-native vs invoke-script — Worker decides at EXECUTE** (xem Constraint 1).

### Scope
- CHỈ sửa: `crates/sos-install/src/engine.rs`, `crates/sos-cli/src/commands/install.rs`, install-smoke test.
- KHÔNG sửa: adapter guard logic (`crates/sos-adapter-codex/**`, `crates/sos-adapter-claude/**`) = P078e/d2b surface. KHÔNG pull approval actor-check scope.
- KHÔNG delete/rewrite `scripts/install-hooks.sh` (reference-only; nếu native-port khiến nó redundant, note trong Discovery — đừng xoá trong phiếu này).

---

## Task 0 — Verification Anchors

> Worker đọc `install-hooks.sh` FULL trước khi chọn cách impl. Task 0 verify TỪNG anchor trước edit.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `crates/sos-install/src/engine.rs` tồn tại (~311 dòng), là nơi add arming step sau khi files render/copy xong | `grep -n "fn " crates/sos-install/src/engine.rs` — tìm hàm install/apply entrypoint `[needs Worker verify]` | ⏳ TO VERIFY |
| 2 | `crates/sos-cli/src/commands/install.rs` (~186 dòng) wire `--runtime {claude,codex}` flag → engine | `grep -n "runtime\|claude\|codex" crates/sos-cli/src/commands/install.rs` `[needs Worker verify]` | ⏳ TO VERIFY |
| 3 | `scripts/install-hooks.sh` chứa 4 arming step (chmod / F09 hijack-guard TTY-confirm+non-TTY-abort / `git config core.hooksPath hooks` / rename `.bak`) | `cat scripts/install-hooks.sh` — enumerate mọi step + edge (đặc biệt F09 guard branch + TTY detect) `[needs Worker verify — Architect không đọc source]` | ⏳ TO VERIFY |
| 4 | `hooks/pre-push` tồn tại (arming phải cover cả pre-push, không chỉ pre-commit) | `ls hooks/pre-push` — nếu absent, arming chỉ chmod pre-commit + note | ⏳ TO VERIFY (Architect thấy `hooks/pre-commit` refs trong CLAUDE.md nhưng KHÔNG confirm `pre-push`) |
| 5 | `git config core.hooksPath` semantics: local-scope, giá trị `hooks` relative repo-root | `install-hooks.sh` dùng local hay global? confirm `--local` `[needs Worker verify]` | ⏳ TO VERIFY |
| 6 | Install engine có sẵn cách detect "target repo root" + biết đang trong git repo hay không | `grep -n "git\|repo_root\|is_git" crates/sos-install/src/engine.rs` `[needs Worker verify]` | ⏳ TO VERIFY |

**⚠️ Architect KHÔNG đọc code (no Grep/no source-read envelope).** Mọi anchor code-level là `[needs Worker verify]` — Worker grep confirm rồi mới edit. Nếu `install-hooks.sh` có step thứ 5 Architect không liệt kê → Worker enumerate đủ + DISCOVERY_REPORT.

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
- #1 ✅ `engine.rs:186 pub fn apply(...)` = single write-entrypoint (called `install.rs:128`); insert arming after `save_manifest` success branch, before `Ok(report)` at `engine.rs:277`.
- #2 ✅ `install.rs:80 fn run_adapter` = ONE shared call-site for both `run_claude`/`run_codex` → `engine::apply` — symmetric by construction, no runtime early-return.
- #3 ✅ `install-hooks.sh` = exactly 4 steps (chmod L30-31, F09 guard L39-52, `git config core.hooksPath hooks` L59, rename `.bak` L65-69). No hidden 5th step. Script uses bare `git config` (default local) — Worker uses explicit `--local` = strict improvement.
- #4 ✅ `hooks/pre-push` exists → arming covers both.
- #5 ⚠️ script uses bare `git config` not `--local`; phiếu's explicit `--local` is correct (strict-improvement).
- #6 ⚠️ No existing git-repo/repo-root helper in `engine.rs` → Worker builds fresh (`git rev-parse --is-inside-work-tree` or `.git` check).
- Task 4 ⚠️ No existing install-smoke temp-git test; `crates/sos-install/tests/install.rs` has `TempFixture` helper → reuse + add `git init`.

**Objections (Tầng 1 only):** Worker accepted V1 — no Tầng-1 challenges. F09 guard semantics (fires only when `hooksPath` set AND ≠ `hooks`) already satisfy idempotency constraint #6; non-git/non-TTY handling unambiguous.

**Status:** ✅ ACCEPTED V1 — no Architect response needed

### Final consensus
- Phiếu version: V1 (accepted as-drafted)
- Total turns: 1 (Worker accepted, 0 objections)
- Approved by Chủ nhà: 2026-07-23 — self-approved by Quản đốc under delegated sprint authority (P076→P081, self-approve in-scope; Guarded/Tầng-1, no owner-decision, no Codex run required)

---

## Nhiệm vụ

### Task 1: Enumerate arming steps từ `install-hooks.sh` (verify-before-impl)

**File:** `scripts/install-hooks.sh` (read-only reference)

**Tìm:** toàn bộ arming logic — 4 step ở §Giải pháp + mọi edge (TTY detect method, F09 guard exact branch, `.bak` naming convention, local-vs-global `git config`).

**Lưu ý:** Đây là bước gate cho quyết định engine-native vs invoke-script (Constraint 1). Ghi kết quả enumerate vào Discovery. Nếu có step Architect không liệt kê → đó là ground-truth, port theo `install-hooks.sh`, không theo phiếu.

### Task 2: Add arming step vào install engine

**File:** `crates/sos-install/src/engine.rs` `[needs Worker verify — chèn sau bước render/copy files, trước khi engine return success]`

**Thêm:** một arming routine chạy khi install target là git repo, thực hiện 4 step theo thứ tự:
1. `chmod +x hooks/pre-commit` (+ `hooks/pre-push` nếu Anchor #4 confirm tồn tại).
2. F09 hijack-guard: đọc `git config --local core.hooksPath`; nếu set và ≠ `hooks` → TTY confirm `[y/N]` (default N), non-TTY → return Err/abort exit 1 với message rõ ("core.hooksPath already set to <X>; refusing to clobber — run install-hooks.sh manually or unset").
3. `git config --local core.hooksPath hooks` (confirm `--local` per Anchor #5).
4. Nếu `.git/hooks/pre-commit` (và/hoặc `pre-push`) tồn tại → rename → `*.pre-hookspath.bak` (KHÔNG delete).

**Lưu ý:**
- **Non-git-repo → warn-skip, KHÔNG fail install** (log warn "not a git repo — skipping hook arming", tiếp tục success).
- **Windows-portable:** nếu chọn engine-native, dùng Rust std (không shell out `chmod`/`git` qua bash). `chmod +x` trên Windows là no-op/skip — dùng cách Rust-native set-executable (Unix perms) guard bằng `#[cfg(unix)]`. `git config` gọi qua `git` binary (cross-platform) HOẶC lib — Worker chọn.
- **TTY detect** cross-platform (std `IsTerminal` / atty-equiv). Non-TTY = CI/piped install → abort-not-prompt (fail-closed cho F09 guard).

### Task 3: Symmetric wiring cho cả claude + codex runtime

**File:** `crates/sos-cli/src/commands/install.rs` `[needs Worker verify]`

**Lưu ý:** arming phải chạy cho **cả** `--runtime claude` VÀ `--runtime codex` (cả hai dựa vào Git backstop). Nếu arming nằm trong engine core-path (không rẽ theo runtime) thì symmetric tự động — confirm không có early-return bỏ qua arming cho một runtime.

### Task 4: install-smoke test (oracle — xem Nghiệm thu)

**File:** install-smoke test `[needs Worker verify vị trí — grep existing install-smoke/temp-git test trong crates/sos-install/tests hoặc engine.rs #[cfg(test)]]`

**Lưu ý:** test PHẢI chạy trong **temp git repo thật** (`git init` trong tempdir) rồi assert git-config + file state — KHÔNG chỉ assert "install ran". Chi tiết assertion ở Nghiệm thu.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-install/src/engine.rs` | Task 2: add hook-arming routine (4 step + non-git warn-skip + F09 guard) |
| `crates/sos-cli/src/commands/install.rs` | Task 3: confirm symmetric claude+codex arming (có thể no-op nếu engine core-path đã symmetric) |
| install-smoke test file | Task 4: temp-git-repo assert git-config + chmod + `.bak` + non-clobber + non-git + negative |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/install-hooks.sh` | Reference-only — enumerate steps; KHÔNG delete/rewrite kể cả khi native-port khiến redundant (note trong Discovery) |
| `crates/sos-adapter-codex/**`, `crates/sos-adapter-claude/**` | KHÔNG touch — P078e/d2b surface |

---

## Luật chơi (Constraints)

1. **Engine-native vs invoke `install-hooks.sh` = Worker decides at EXECUTE.** Đọc `install-hooks.sh` full (Task 1) rồi chọn. **Precedent P059/P072 nghiêng native** (bash-on-Windows portability risk — install phải chạy Windows Git Bash + Linux + macOS). Ghi rõ lý do chọn trong Discovery.
2. **F09 hijack-guard fail-closed:** non-TTY + custom hooksPath = ABORT (exit 1), KHÔNG silent clobber, KHÔNG prompt-vào-void.
3. **Rename không delete** — existing `.git/hooks/*` → `.pre-hookspath.bak`. Adopter phải recover được hook chain cũ.
4. **Non-git-repo warn-skip** — install vẫn success, chỉ log warn.
5. **Symmetric** — arming chạy cho cả hai runtime; không rẽ nhánh bỏ sót một cái.
6. **Idempotent** — chạy `sos install` lần 2 khi `core.hooksPath` đã = `hooks` (do ta set trước đó) KHÔNG được trigger F09 guard (giá trị `hooks` = của ta, không phải custom). Guard chỉ fire khi ≠ `hooks`.

---

## Nghiệm thu

### Automated
- [ ] `cargo check` clean
- [ ] `cargo test` pass
- [ ] **install-smoke (oracle `[oracle: install-smoke temp-git assert git-config + hijack-guard behavior]`)** — trong temp git repo, assert:
  - [ ] `git config --local core.hooksPath` == `hooks`
  - [ ] `hooks/pre-commit` executable (`#[cfg(unix)]`); `hooks/pre-push` executable nếu tồn tại
  - [ ] existing `.git/hooks/pre-commit` (seed trước install) → được rename `*.pre-hookspath.bak` (file cũ còn, không mất)
  - [ ] **non-clobber:** seed `git config core.hooksPath custom-dir` trước install → non-TTY install ABORT exit 1, KHÔNG đè thành `hooks`
  - [ ] **non-git-dir:** install trong tempdir KHÔNG `git init` → warn-skip, install success (exit 0), không panic
  - [ ] **idempotent:** install 2 lần liên tiếp → lần 2 không abort (giá trị `hooks` của ta không trigger F09 guard)
  - [ ] **negative-test:** assert guard THẬT SỰ chặn — không phải test luôn-xanh (feed custom hooksPath, assert Err; feed clean repo, assert Ok)

### Manual Testing
- [ ] `sos install --runtime claude` trong repo git thật → `git config core.hooksPath` = hooks, commit thử → pre-commit fire
- [ ] `sos install --runtime codex` → cùng kết quả (symmetric)

### Regression
- [ ] `sos install` trong repo đã có kit (re-install) không hỏng hook chain hiện có
- [ ] Install trên non-git dir (nếu use-case tồn tại) vẫn hoàn tất

### Docs Gate (Tầng 1 — security surface AUTO Tầng 1)
- [ ] `CHANGELOG.md` — entry P078f (arm Git hooks by default)
- [ ] `SECURITY.md` — install-time hook-arming là auto-exec/boundary surface: document `core.hooksPath=hooks` armed-by-default + F09 hijack-guard behavior (boundary touch)
- [ ] `adapters/codex/CAPABILITY.md` — cập nhật honest-MISSING story: Git backstop giờ **armed-by-default** (không còn "off after install"); confirm câu chữ khớp reality
- [ ] `adapters/claude/README.md` hoặc MAPPING — nếu có reference tới hook-arming/install, sync (verify — có thể N/A)
- [ ] `docs/plans/P078d-decomposition.md` — mark P078f DONE (nếu convention)
- [ ] `docs/BACKLOG.md` — active-sprint P078 row + resume pointer: P078f DONE → next P079 round-3

### Discovery Report
- [ ] Write `docs/discoveries/P078f.md`:
  - Task-0 anchor CORRECT/WRONG (file:line citations) — đặc biệt `install-hooks.sh` step enumeration vs phiếu 4-step
  - **Engine-native vs invoke-script — quyết định + lý do** (Constraint 1)
  - `hooks/pre-push` tồn tại hay không (Anchor #4 resolve)
  - Scope expansions (nếu có)
  - Edge cases / limitations (Windows chmod no-op, TTY detect, idempotent guard)
  - Tầng 1 docs updated: <list> (hoặc "N/A" explicit)
  - Tier escalations (write "None")
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`
