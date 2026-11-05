# Rust hoá — bàn giao cho session làm đợt tiếp theo (05/11/2026, cập nhật sau đợt 3)

Session 2.1 dừng ở đây theo chỉ đạo «giữ ngữ cảnh ngắn» (Sếp duyệt 05/11). Ghi chú này đủ để làm tiếp mà không cần đọc transcript cũ.

## Chỗ đứng

- **Nhánh:** `claude/rust-all-tools-d1e85b`, worktree `~/sos-kit/.claude/worktrees/rust-all-tools-d1e85b`. Đã push, PR https://github.com/aspelldenny/sos-kit/pull/49. Không merge vào main khi chưa có lệnh.
- **Các đợt đã xong trên nhánh** (cũ → mới):

| Đợt | Commit | Nội dung |
|---|---|---|
| 1a | `eb13cfa` và các commit trước | `sos hook claude\|codex …` thay `adapters/*/hook.py`, `env-guard.sh`, `sim-guard`, `status.sh` |
| 1b+1c | `d53e8b0` | mẫu hook gọi `sos hook … --protocol 1` qua launcher fail-closed; `sos gate pre-commit\|pre-push`; `sos update --wiring` |
| 2.4 | `b917dc2`, `912dc09` | `sos usage`: token theo vai × model, từ transcript Claude và rollout Codex |
| tách `hook/` | `29fc065` | `hook/{mod,claude,codex}.rs`, `guards/{env,sim}.rs`; không đổi hành vi |
| 2.1 | `978cda4` | `sos advise` (`advise.rs`) và bộ kích advisor chạy trong `sos hook post-bash` (`watch.rs`): hook không còn gọi Python. Astra soát 4 lượt, MERGE |
| 1d | commit «sos-rules: declarative hook rules» | crate `crates/sos-rules`, `sos hook [claude\|codex] rules --file F`, `--lint/--cases/--explain`; 5 luật dịch sang TOML trong `tests/rules/<tên>/rules.toml`, 111 ca có kết quả bản cũ. Astra soát 6 lượt |
| 2.2 | commit «sos bite: the mutation check in Rust» | `sos bite` (`crates/sos-cli/src/bite.rs`) thay `scripts/bite` (bản Python để cạnh, chưa đổi chỗ gọi). `tests/bite.sh` chạy cả hai bản (`SOS_BITE_BIN=…/sos`), `tests/bite_parsers.py --sos BIN`. Khác có chủ ý: CHANGELOG mục Unreleased. Astra soát 3 lượt, MERGE |
| 2.3 | commit «sos-dev: the kit's trust gate, docs check and hook installer in Rust» | crate `crates/sos-dev` (không phát hành): `sos-dev trust-gate\|docs-check\|install-hooks`; `hooks/pre-commit` gọi bản Rust, bản cũ để cạnh; `tests/sos-dev.sh` so hai bản. Astra soát 5 lượt, MERGE |
| 2.5 | commit «sos sim: boot, settle and destination for one simulator» | `sos sim boot\|settle\|destination` (`crates/sos-cli/src/sim.rs`) thay `sim-boot.sh`, `wait-settled.sh`, `resolve-simulator-destination.sh`; test với `xcrun`/`sysctl` giả. Astra soát 2 lượt, MERGE |
| 3.1 | commit «sos predict: written predictions and their age» | `sos predict` (`crates/sos-cli/src/predict.rs`) thay `skills/depth-check/scripts/predictions.py` (script để cạnh, skill gọi `sos predict` trước); chỉ đọc, không chạy lệnh từ git config, không fetch trong partial clone. Astra soát 6 lượt, MERGE |
| 3.2 | commit «sos check-links: relative links and anchors in Markdown» | `sos check-links <dir>` (`crates/sos-cli/src/check_links.rs`) thay `references/check-links.py` của skill product-description (chỗ gọi chưa đổi: nằm ở `~/.claude/skills`). Astra soát 3 lượt, MERGE |
| 4.3, 4.4 | `25d8f22` | `sos sessions idle\|count` (`crates/sos-cli/src/sessions.rs`): đọc file session của app, `count --hook --max 3` cho hook `start_session`. Astra soát 2 lượt, MERGE; `6ae10a0` sửa `idle` không `--over` |
| 4.6 | `4017da3` | `sos hook word-limit` (`hook/word_limit.rs`): đo chữ tin cuối lượt, `--log` TSV; `--block` có nhưng không bật |
| 4.1, 4.2, 4.3, 4.5, 4.7 | crate `~/code/tong-dai` (git ở máy, không push) | `chan.toml` + 53 ca; `tong-dai kiem` (`--cu` khớp `kiem.py`), Sổ giao ca phải có id/đường dẫn, ĐỨNG IM, `--hook` chặn `clear_session`; `tong-dai brief`. 4 commit ở máy (`12e2977`…`4798c51`). Astra soát 4 lượt, MERGE |

- **Việc tiếp theo (sau 2.2):**
  1. **Đổi chỗ gọi ở từng repo, mỗi repo một nhánh riêng**, chép `tests/rules/<tên>/rules.toml` sang repo đó, hook gọi `sos hook claude rules --file <đường dẫn>` (Codex: `sos hook codex rules …`), bản cũ để cạnh đến bản phát hành sau:
     - vault: `chan.toml` thay `chan.py`;
     - BOOK: `win-heavy.toml` thay `win_heavy_guard.py`;
     - soulsign-marketing: `worker-guard.toml` thay `worker-guard.sh` (bản cũ fail-open dưới bash 3.2 + locale UTF-8, nên đổi sớm);
     - tarot: `merge-guard.toml` thay `block-unsafe-merge.sh`, kèm một helper `scripts/merge-check <PR>` (phần gọi `gh`; trong ca kiểm nó là `fake`);
     - mọi repo: `hub-audit.toml` thay `~/.claude/hooks/hub-audit.sh`.
     Điều kiện: bản `sos` có `sos hook … rules` đã phát hành và cài (bản 0.3.13 trả exit 2 với lệnh lạ = chặn mọi thứ). Không đụng lucilius, reel.
  2. Đợt 3 xong phần trong sos-kit (3.1 `sos predict`, 3.2 `sos check-links`; 3.3–3.5 chỉ ghi). Đợt 4 xong phần code (đổi chỗ gọi: «Việc chờ Sếp» mục 7). Đợt 5 không làm ở đây: Tổng đài mở dự án riêng.
- **Bản 1d hẹp hơn đặc tả §2** (không allow/scope, first_positional, case_insensitive, when.file_exists/env; ask bị từ chối): ghi ở `RUST_ARCHITECTURE_2026-11-05.md`, đoạn «Bản 1d hẹp hơn đặc tả». Thêm khi một luật thật cần.

## File cần đọc

- Kế hoạch và tiến độ: `docs/plans/RUST_ALL_TOOLS_2026-11-04.md` (mục «Tiến độ» ở cuối).
- Kiến trúc đích và lý do thứ tự: `docs/plans/RUST_ARCHITECTURE_2026-11-05.md`. Mục 2 là 1d, mục 7 là `sos usage`.
- Soát kiến trúc và soát 1b: `docs/plans/RUST_REVIEW_2026-11-05.md`.
- Việc đang mở: `docs/BACKLOG.md`, mục đầu, dòng «Rust hoá tool agent».
- Các bản soát của Astra cho 2.4 và 2.1 nằm trong scratchpad của session cũ (sẽ mất). Phần cốt lõi đã ghi vào CHANGELOG (mục Unreleased) và vào commit message.

## Cách làm đã quen (giữ nguyên)

- **Mac chỉ có 8 GB RAM.** Ở Mac chỉ chạy `cargo check`/`clippy`, không chạy `cargo build`/`test` nặng. Build và test trên máy Win, theo `~/VibeNotes/30_Resources/luat-may-win.md`:
  - gửi tar mã nguồn: `ssh windows 'cat > /home/sep/rusthoa/src.tgz' < src.tgz`;
  - chạy một file job qua `ssh windows 'cat > /tmp/job-x.sh && bash /tmp/job-x.sh; rm -f /tmp/job-x.sh' < job.sh`, trong tmux, bọc `/mnt/e/may-win/may-win chay --ten … --han … --`.
- **Thư mục trên Win:**
  - `/home/sep/rusthoa` (WSL, `CARGO_TARGET_DIR=/home/sep/rusthoa/target`);
  - `C:\Users\Admin\rusthoa` (Windows native; `cargo test` qua `cmd.exe`; Git Bash ở `/mnt/c/Program Files/Git/bin/bash.exe`).
  - Dọn cả hai khi xong toàn bộ Rust hoá.
- **Bộ chạy trên Win:** build, `cargo test -p sos-cli`, `tests/adapters/run.sh` hai bản (Python, và `SOS_HOOK_BIN=…/sos`), `tests/bite.sh` hai bản (Python, và `SOS_BITE_BIN=…/sos`), `tests/docs-check.py <sos>`, clippy; Windows native `cargo test` + clippy; Git Bash `run.sh` và `bite.sh` với `sos.exe`.
- **Soát độc lập:** `sos role reviewer --from claude --brief <file> --out <file>` (Codex gpt-6-astra), chạy nền. Bắt buộc theo luật 4 khi chạm `harness-lite/` hoặc `templates/app/`.
- **Báo Tổng đài** đúng hai mẫu, mỗi mẫu một dòng:
  - `[Rust hoá] XONG: … · file: …`
  - `[Rust hoá] CẦN SẾP: … · lựa chọn: A / B`
- **Sau mỗi đợt, ghi sổ ở bốn chỗ:**
  - BACKLOG mục đầu;
  - «Tiến độ» trong kế hoạch;
  - một dòng đầu «Nhật ký bảng» trong `~/VibeNotes/10_Projects/README.md`;
  - một dòng trong ghi chú chung `rust-hoa-dot-1a.md` (thư mục side-session-notes của Tổng đài).

## Việc chờ Sếp (repo khác, session Rust hoá không sửa)

1. **2.5 — đổi chỗ gọi simulator** sang `sos sim …` khi bản có `sos sim` đã phát hành và cài:
   - tally và turnsigil: `tools/sim-boot.sh <udid> [log]` → `sos sim boot <udid> [--log F]`; `tools/wait-settled.sh <label> [log]` → `sos sim settle <label> [--log F]` (biến `SETTLE_*` vẫn đọc được);
   - lucilius: `scripts/resolve-simulator-destination.sh` → giữ một wrapper nhỏ đọc `LUCILIUS_DESTINATION`, còn lại gọi `sos sim destination --name "${LUCILIUS_SIMULATOR_NAME:-Lucilius P004 Delivery Oracle}"`. Lưu ý: bản Rust khớp nguyên tên, không khớp tiền tố.

2. **2.6 — turnsigil bỏ `tools/agent_role.py`** (kế hoạch ghi «bỏ»: trùng `sos role`). Chỗ gọi: `tools/agent-bin/claude`, `tools/agent_hook.py`, `.codex/config.toml`, `AGENTS.md`, test `tools/tests/test_agent_harness.py`. Đổi sang `sos role`, so lệnh sinh ra bằng `sos role --dry-run` trước khi xoá.
3. **2.7 — tarot** (`scripts/security-gate.sh`, `scripts/check-hardcoded-secrets.py`, `scripts/refresh-docs-stats.sh`; gọi từ `package.json`, `docs/TOOLS.md`, `docs/security/INVARIANTS.md`, `scripts/test-inv004-history.sh`): bỏ phần tìm khoá trùng `sos gate secrets` (gitleaks); INV-001→010 còn lại thành luật `sos-rules` nếu đủ, không thì Rust trong tarot. Chứng minh: chạy INV cả hai bản trên cùng cây và trên các commit cũ từng bị bắt.

4. **3.2 — skill product-description** (`~/.claude/skills/product-description/SKILL.md`, dòng 18 và 83 gọi `python3 {skill dir}/references/check-links.py {repo}`): đổi sang `sos check-links <dir>` khi bản có lệnh này đã phát hành và cài; script để cạnh làm dự phòng như depth-check. Cùng dòng báo, số đếm, mã thoát; khác có chủ ý ở CHANGELOG.
5. **3.3 — thanhtra** (`thanhtra-pre-scan.py` + CLI `thanhtra`, repo công khai riêng): Rust hoá trong repo thanhtra, bộ test của thanhtra chạy cả hai bản trước khi đổi.
6. **3.4 — bỏ plugin Codex `codex-claude-bridge`** (`bridge.py`, `ghostty_bridge.py`): trùng start_session/SendMessage; Sếp gỡ plugin, mã chuyển vào lưu trữ. 3.5 (skill đồng bộ của Anthropic, skill hệ thống Codex) **giữ**, không có việc.

7. **Đợt 4 — đổi chỗ gọi của Tổng đài** (vault, `~/.claude`; session Rust hoá không sửa). Điều kiện chung: bản `sos` có `sos sessions`, `sos hook word-limit`, `sos hook rules` đã phát hành và cài; `tong-dai` build từ `~/code/tong-dai` (`cargo install --path ~/code/tong-dai`).
   - `chan.py` → `sos hook claude rules --file ~/code/tong-dai/chan.toml` ở settings của vault (cùng matcher); `chan.py` để cạnh một thời gian.
   - PreToolUse matcher `mcp__ccd_session__start_session` → `sos sessions count --hook --max 3` (thay câu «tối đa 3 session con» trong LUAT bằng một dòng trỏ tới hook).
   - PreToolUse matcher `mcp__ccd_session_mgmt__clear_session` → `tong-dai kiem --hook` (bảng lỗi thì chặn xoá lịch sử).
   - Stop → `sos hook word-limit --max 100 --log ~/.claude/word-limit.tsv`, **không** `--block`; vài ngày sau đếm dòng `over=1` rồi Sếp quyết có bật chặn không.
   - `python3 10_Projects/tong-dai/kiem.py` → `tong-dai kiem` (khác: thêm kiểm Sổ giao ca và mục ĐỨNG IM; `--cu` cho đầu ra cũ).
   - Brief cho session con: `tong-dai brief --du-an … --viec … --xong … --doc …`, hoặc `tong-dai brief --kiem FILE` cho brief viết tay.
   - Bảng thật 05/11 đang có 2 dòng Sổ giao ca thiếu id/đường dẫn («Sếp đã chốt 05/11», «Desktop không dính lỗi…»): thêm chỗ lần lại trước khi bật hook `clear_session`, không thì hook chặn ngay.

## Việc còn mở (không chặn đợt sau)

1. **WSL thiếu gitleaks.** Vì vậy test `install_sets_a_reviewer_from_the_other_family_when_both_clis_exist` (`crates/sos-cli/tests/kit.rs`) đỏ ở WSL. Đây là chuyện môi trường, không phải lỗi code. Cài gitleaks vào WSL, hoặc để nguyên.
2. ~~**4 ca Codex trên Windows thật**~~ — xong 05/11 khi phát hành v0.4.0: `codex exec` thật trên Windows 11 (codex 0.162, `--dangerously-bypass-hook-trust`, `-s danger-full-access`). sos mới: sửa `README.md` đi qua, sửa `.env` bị env guard chặn. sos 0.3.13 ("sos exited 2 … Upgrade sos"), thiếu sos ("sos is not on PATH"), sos.exe hỏng ("SOS Kit hook failed: OSError(22 …)"): mọi `apply_patch` bị chặn, `.env` và `README.md` không đổi. SessionStart: Completed với sos mới, Failed với ba ca còn lại (không chặn phiên).
3. **Settings của chính kit.** `.claude/settings.json` của sos-kit vẫn gọi adapter Python. Chỉ đổi sang `sos hook` sau khi bản phát hành có `sos hook` đã cài trên Mac: bản 0.3.13 trả exit 2 với lệnh lạ, và Claude Code coi exit 2 là chặn.
4. **`sos usage`, hai việc đã ghi trong BACKLOG:**
   - đọc kết quả subagent chạy nền (về qua task-notification, chưa đếm);
   - bộ nhớ tăng theo số lần gọi (177 MB với 70 nghìn lần gọi), cần gom chuỗi trùng.
5. **`run_limited` (hook/mod.rs) không giết process cháu** (`sos bite` đã dùng job object của 1d trên Windows, nhóm process trên Unix; `run_limited` có thể dùng lại cách đó) khi hết giờ. `role::capture` thì có giết. Chỉ `sim-guard` (`ps`) và notify còn dùng `run_limited`. Xem lại trước khi phát hành.
6. **Git Bash trên Windows, 7 ca đỏ không thuộc advisor** (05/11, chưa kiểm xem có từ trước 2.1 hay không):
   - Rust: 4 ca `same status` (so từng byte status Python với Rust; chữ hiện ra giống nhau, nghi khác ở xuống dòng hoặc encoding);
   - Python: 3 ca `.env.local\x` (tên POSIX, Windows coi `\` là dấu phân cách).

   Mọi ca advisor đều xanh ở Git Bash. CI có job `windows-2022`, sẽ thấy các ca này khi mở PR.
7. **Python vẫn được ship** (`scripts/test-watch.py`, `scripts/advise`, `adapters/*/hook.py`) cho app còn wiring cũ. Bỏ khi mọi app đã chạy `sos update --wiring` (BACKLOG).
