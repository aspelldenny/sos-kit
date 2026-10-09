# P079 round-4 dogfood checklist (Sếp + Codex) — 2026-07-23

> **Mục tiêu:** confirm 2 fix round-3 (P078g + P078h) usable end-to-end. Xanh → P079 DONE, unblock P080.
> **Fixes under test:** P078g (`69d5b7c` — install RENDERS + arms real hooks, fix P078f arm-empty) · P078h (`2393c1b` — actor-check advance-block + path-normalize).
> **Clone:** main mới nhất (`2393c1b`+). KHÔNG dùng bản round-3 `0108f99`.

## Setup
1. Fresh clone / `git pull` main lên `2393c1b`+. `cargo build --release`.
2. Repo test git **riêng** (KHÔNG phải clone sos-kit — tránh `.sos-state/sos-kit-self`).
3. `sos install --runtime codex` trong repo test đó.

## Test A — P078g: install RENDER + arm hook thật (đây là chỗ round-3 FAIL)
> Round-3: `core.hooksPath=hooks` set nhưng `hooks/pre-commit` KHÔNG tồn tại → commit lọt hết. P078g phải sửa dứt điểm.
- [ ] Sau install: `hooks/pre-commit` **VÀ** `hooks/pre-push` **TỒN TẠI** trong repo test + **executable** (round-3 A2 FAIL — giờ phải PASS).
- [ ] `git config --local core.hooksPath` → `hooks`.
- [ ] Commit thật một file `.env` → **BỊ CHẶN** (exit ≠ 0) — round-3 A3 FAIL, giờ phải block.
- [ ] Commit product code trên default branch → **BỊ CHẶN** — round-3 A4 FAIL, giờ phải block.
- [ ] (refuse-when-absent, optional) nếu xoá `hooks/pre-commit` rồi chạy lại arm-path → install KHÔNG set `core.hooksPath` vào con trỏ rỗng (warn-loud). Không bắt buộc test, chỉ nếu tiện.
- [ ] (non-clobber, round-3 A5 regression) repo có `core.hooksPath` custom sẵn → install non-TTY ABORT không đè.

## Test B — P078h: actor-check advance-block + path-normalize
> ⚠️ **Caveat quan trọng (đọc trước khi test):** trên **real Codex custom-role subagent spawn**, markers `.sos-state/{worker,architect}-active` KHÔNG được tạo (upstream `openai/codex#21753`) → actor-check KHÔNG kích hoạt trong đường subagent thật. Đây là giới hạn đã-biết, KHÔNG phải regression. Test B verify logic qua **manual-marker repro** (như round-3 §B4 đã làm). FULL protection chỉ trên Claude.
- [ ] **Manual-marker advance BLOCK (gap#3 core):** tạo `.sos-state/worker-active` thủ công + state đã approved `V3/V3` → thử patch state-only lên `V4/V4` → **BỊ CHẶN** (round-3: lọt vì guard so state cũ V3==V3 → giờ phải block dứt khoát).
- [ ] **Manual-marker any-write BLOCK:** worker-active present + bất kỳ ghi `ticket-state.env` (create/update/advance) → **BỊ CHẶN**.
- [ ] **Main-thread no-regress (P078e deadlock fix):** KHÔNG có marker → bootstrap `V1/empty` → approval update `V2/V2` → **ALLOW** (không được block nhầm).
- [ ] **Path-normalize (gap#4):** patch dùng **absolute path** cho `ticket-state.env` → xử đúng như relative (round-3: absolute bị false-block). Marked → block; main-thread → allow.
- [ ] **Bundle BLOCK:** patch `ticket-state.env` + code trong 1 apply_patch → vẫn BLOCK (multi-path không regress).

## Kết quả
- **Test A xanh hết** (backstop thật hoạt động) + **Test B manual-marker logic đúng** → **P079 DONE**, unblock **P080** (dual-runtime brownfield). Tạo `docs/adapters/P079-ROUND4-FINDINGS-2026-07-23.md` (PASS/FAIL + output).
- Có FAIL → ghi gap (format round-3), paste về Quản đốc để spec fix.

## Ghi chú honest
Sau round-4, story bảo mật Codex là: **actor-check = best-effort trên Codex real-subagent** (markers absent, upstream), **git backstop (P078g, giờ armed-thật) + human-review-at-commit = net cuối**. Full actor-check chỉ trên Claude (markers fire in-subagent). Điều này đã ghi trong `SECURITY.md` + `adapters/codex/CAPABILITY.md` — round-4 confirm nó khớp reality.
