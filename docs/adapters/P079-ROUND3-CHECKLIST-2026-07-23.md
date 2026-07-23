# P079 round-3 dogfood checklist (Sếp + Codex) — 2026-07-23

> **Mục tiêu:** confirm 2 fix round-2 usable end-to-end trên Codex thật. Nếu cả 2 xanh → P079 DONE, unblock P080.
> **Fixes under test:** P078e (`dd4594d` — approval transition + actor-check) · P078f (`d35f462` — `sos install` arms Git hooks).
> **Clone:** dùng main mới nhất (`d35f462` trở lên) — KHÔNG clone @72ff1c9 cũ.

## Setup
1. Fresh clone main (hoặc `git pull` lên `d35f462`+). Build `sos` binary (`cargo build --release` hoặc dùng đã có).
2. Tạo repo test git thật (KHÔNG phải sos-kit clone — tránh self-exempt marker `.sos-state/sos-kit-self`).
3. `sos install --runtime codex` trong repo test đó.

## Test A — P078f: install tự arm Git hooks (KHÔNG chạy install-hooks.sh tay)
- [ ] Ngay sau `sos install --runtime codex`, chạy `git config --local core.hooksPath` → **phải in `hooks`** (round-2: unset → FAIL). KHÔNG cần chạy `scripts/install-hooks.sh` tay.
- [ ] `hooks/pre-commit` + `hooks/pre-push` executable.
- [ ] Thử commit 1 file `.env` → **pre-commit BLOCK** (Git boundary armed).
- [ ] Thử commit product code trên default branch → **no-code-on-default BLOCK**.
- [ ] (Non-clobber) Nếu repo test đã có `core.hooksPath` custom sẵn: install non-TTY → **ABORT không đè**; hoặc TTY → hỏi `[y/N]`.

## Test B — P078e: approval transition deadlock GONE + worker self-approve BLOCKED
- [ ] Bootstrap state (version=V1, approved_version=empty) tạo được.
- [ ] **First legit approval update** (main-thread: version→V2, approved_version=V2 ghi vào `.sos-state/ticket-state.env`) → **KHÔNG còn bị approval-gate chặn** (round-2: deadlock, phải sửa state tay). Đây là fix chính.
- [ ] EXECUTE + REVIEW sau approval chạy thông, KHÔNG cần manual state edit.
- [ ] **Actor-check:** spawn Worker rồi để Worker thử tự ghi `approved_version=` vào `ticket-state.env` (self-approve) → **BLOCK** (marker `.sos-state/worker-active` present = không phải main-thread).
  - ⚠️ **Caveat Codex:** in-subagent enforcement MISSING (P078d2b — Codex custom-role hooks không fire, openai/codex#21753). Nếu marker không được tạo trong spawned agent → actor-check best-effort; Git backstop (Test A) + human-review-at-commit là net cuối. Ghi rõ marker có được tạo trong subagent Codex không.
- [ ] Bundle patch (ticket-state.env + code cùng 1 apply_patch) → vẫn **BLOCK** (multi-path guard d2a không regress).

## Kết quả → điền vào đâu
- Xanh hết → tạo `docs/adapters/P079-ROUND3-FINDINGS-2026-07-23.md`: PASS/FAIL từng test + evidence (command output). Mark P079 DONE trong BACKLOG → unblock P080.
- Có FAIL → ghi gap mới (như round-1/round-2 format), quay lại Quản đốc để spec fix phiếu.

## Câu hỏi ground-truth cần Codex trả lời (nếu tiện)
- Trong spawned agent (architect/worker), SubagentStart hook có tạo `.sos-state/{architect,worker}-active` không? (đây là gap #4 / P078d2b vẫn treo — round-3 là cơ hội probe live; nếu marker tạo được → có thể unblock d2b).
