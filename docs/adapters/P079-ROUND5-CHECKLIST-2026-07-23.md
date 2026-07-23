# P079 round-5 dogfood checklist (Sếp + Codex) — 2026-07-23

> **Mục tiêu:** confirm 2 fix round-4 (P078i + P078j) usable end-to-end. Xanh → P079 DONE, unblock P080.
> **Fixes under test:** P078i (`9646a56` — install renders a self-contained fail-CLOSED backstop hook + its 2 guard scripts) · P078j (`a0df82e` — guard canonicalizes symlinked paths, fixes macOS /tmp).
> **Clone:** main mới nhất (`a0df82e`+). KHÔNG dùng round-4 `0d458b6`.

## Setup
1. `git pull` main lên `a0df82e`+. `cargo build --release`.
2. Repo test git **riêng** (không phải clone sos-kit).
3. `sos install --runtime codex`.

## Test A — P078i: backstop hook TỰ-CHỨA, fail-CLOSED (round-4 A3/A4 FAIL vì hook thiếu script phụ)
> Round-4: hook render nhưng gọi `scripts/block-env-commit.sh` + `scripts/no-code-on-default.sh` KHÔNG tồn tại → fail-open → commit lọt. P078i render hook backstop tối thiểu + 2 script đó.
- [ ] Sau install: `hooks/pre-commit` **VÀ** `scripts/block-env-commit.sh` **VÀ** `scripts/no-code-on-default.sh` đều TỒN TẠI + executable.
- [ ] `git config --local core.hooksPath` → `hooks`.
- [ ] Commit thật `.env` → **BỊ CHẶN** (exit ≠ 0) — round-4 A3 FAIL, giờ phải block.
- [ ] Commit product code trên default branch → **BỊ CHẶN** — round-4 A4 FAIL, giờ phải block.
- [ ] (fail-CLOSED, optional) xoá `scripts/block-env-commit.sh` rồi commit `.env` → vẫn **BỊ CHẶN** (KHÔNG in "missing → commit allowed"). Đây là điểm P078i sửa dứt: guard vắng = block, không phải allow.
- [ ] (non-clobber, round-4 A5 regression) repo có `core.hooksPath` custom sẵn → install non-TTY ABORT không đè.
- [ ] Xác nhận hook là **minimal backstop** (2 invariant), KHÔNG phải hook dev [8/8] — grep `hooks/pre-commit` không ref `docs-gate`/`trust-gate`/`type-check`/`install-hooks.sh`.

## Test B — P078j: path canonicalize symlinked root (round-4 B4 FAIL với /tmp)
> Round-4: macOS git root = `/private/tmp/...`; path `/tmp/...` không match → forbidden advance lọt + legit approval false-block. P078j canonicalize cả 2 phía.
- [ ] **Manual-marker advance qua `/tmp/...` path (round-4 lọt):** tạo `.sos-state/worker-active` + state approved `V3/V3`, patch dùng `*** Update File: /tmp/<repo>/.sos-state/ticket-state.env` lên `V4/V4` → **BỊ CHẶN** (round-4: `patch: completed` — giờ phải block).
- [ ] **Main-thread legit approval qua `/tmp/...` path (round-4 false-block):** KHÔNG marker, state `V5/empty`, approval patch dùng `/tmp/...` path → **ALLOW** (round-4: bị chặn nhầm — giờ phải cho qua).
- [ ] **Canonical `/private/tmp/...` no-regress:** cùng thao tác với `/private/tmp/...` → marked BLOCK, main-thread ALLOW (round-4 đã đúng, không được vỡ).
- [ ] **Relative path no-regress:** relative header → marked BLOCK, main-thread ALLOW.
- [ ] **Bundle qua `/tmp/...`:** state + code trong 1 patch với `/tmp/...` path → BỊ CHẶN (multi-path).

## Ghi chú caveat (không đổi từ round-4)
- Codex real custom-role subagent markers vẫn KHÔNG fire (upstream `openai/codex#21753`) → actor-check best-effort trên Codex real-subagent; verify Test B qua **manual-marker**. Git backstop (Test A, giờ enforce thật) + human-review = net cuối.

## Kết quả
- **Test A + Test B xanh** → **P079 DONE**, unblock **P080** (dual-runtime brownfield). Tạo `docs/adapters/P079-ROUND5-FINDINGS-2026-07-23.md` (PASS/FAIL + output).
- Có FAIL → ghi gap (format round-4), paste về Quản đốc.
