# sos-kit Backlog

> Việc tiếp theo của chính sos-kit. Mục `##` đầu tiên là sprint đang chạy (banner đầu phiên hiện nó). Lịch sử v2 (P001–P088, phiếu, harvest): `archive/v2/docs/BACKLOG-v2.md`.

## Đang chạy — SOS Kit v3 dùng thật

> Nguồn: `docs/research/HARNESS_SURVEY_2026-10-08.md`, `docs/plans/V3_TRIAGE_2026-10-08.md`. Phát hành mới nhất: xem `CHANGELOG.md`.

- [ ] **Áp vào app mới** khi Chủ nhà brainstorm xong: `sos install`, đo token, thời gian, lỗi bắt được theo tầng (gate / advisor / người soát), số lần advisor bắn và đổi hướng thợ. Chốt model và effort cho advisor sau khi có số đo.
- [x] **Thirty / Payquill / Lucilius đã sang v3 (sos 0.3.6), 15/10/2026.** Lucilius bỏ vòng phiếu DRAFT/CHALLENGE/APPROVE (Chủ nhà chốt khi giao việc), vùng nhạy cảm gọi Người soát kèm INVARIANTS của app; luật sản phẩm chuyển vào AGENTS.md để Codex cũng đọc. Thirty và Payquill: `[state] file = docs/PROGRESS.md`, luật riêng của `agents/roles/` vào AGENTS.md "Riêng dự án này". Kiểm chữ cấm bằng probe trên từng repo. Việc theo dõi: đọc lịch sử phiên đầu tiên chạy v3 ở Lucilius (bug Today "Kept") như audit TurnSigil; worktree cũ (payquill-f10/f12/f47, lucilius-p071) chưa có `harness-lite/` nên không chạy hook đến khi rebase lên `main`.
- [x] **Tarot (web) sang v3 + docs theo tầng, 15/10/2026.** Tarot PR #1036 (harness) và #1037 (CLAUDE.md thành bản đồ 66 dòng, BACKLOG chỉ giữ việc mở). Kit 0.3.7: tầng docs trong `harness-lite/README.md`, cảnh báo cỡ lối vào/trạng thái. Còn: Tarot mục đầu BACKLOG vẫn 140 dòng (26 việc mở, nhiều dòng dài); đợt 2 rút gọn các guide (BACKEND, FRONTEND, PROMPTS, DATA, CONVENTION) mỗi guide một PR.
- [ ] **Windows (POD-agent, `E:\Extension`) — giao Claude trên máy Windows, 23/10/2026.** Đã xong từ Mac qua SSH: kit chạy được trên Git for Windows (0.3.9: `python` thay alias Store, UTF-8, CRLF), `[advisor] tests`, toast Windows; POD-agent PR #2 (worktree `E:\Extension-wt\sos-v3`). Còn, theo thứ tự:
  1. `git pull` bản sos-kit ở `C:\Users\Admin\sos-kit` (đang ở v2); bản `sos` phải là bản mới nhất (cài: `cargo install --git https://github.com/aspelldenny/sos-kit --tag <tag> --locked --force sos-cli`).
  2. POD-agent: merge PR #2, kéo về `E:\Extension` (main ở máy hơn origin 5 commit), `sos update`, thêm `[advisor] tests = ["node scripts/check.mjs"]`, xoá worktree `E:\Extension-wt\sos-v3` (junction `run\node_modules` trỏ sang `E:\Extension`). `sos check` phải sạch.
  3. Kiểm trong phiên Claude Code thật: banner đầu phiên có trạng thái kit; sửa `.env` bị chặn; hai lần `node scripts/check.mjs` đỏ → advisor trả lời; toast hiện khi advisor mất đăng nhập (chạy thử `notify()` của `test-watch.py` trong phiên desktop, qua SSH không hiện được).
  4. Codex trên Windows: hook có chạy không, qua bash hay PowerShell (`python "$(git rev-parse --show-toplevel)/…"` hợp cả hai?), bước trust hook. Chưa ai thử.
  5. Alias Store: Chủ nhà tắt `python.exe`/`python3.exe` trong App execution aliases; rồi `python3 harness-lite/scripts/bite` chạy được qua shebang.
  6. Kit: job CI `windows-latest` (Git Bash, gitleaks, `cargo test`, `tests/adapters/run.sh`); bản phát hành `x86_64-pc-windows-msvc` + cách cài không cần Rust; đường dẫn `\\?\E:\…` và `E:\Extension\hooks/pre-commit` in ra cho đẹp.
  - 23/10 trên máy Windows (nhánh `fix/windows-native`): bước 1 và 6 xong (cả bộ test xanh native: cargo, adapters, bite, smoke, docs; CI thêm `windows-2022`; release build `.exe`; `install.sh` chạy trong Git Bash). Sửa thêm: CRLF nhúng vào binary, `sos role`/`advise` gọi `codex.cmd`, `bite` SIGHUP + chạy test qua bash, guard `.env` của adapter fail-open khi không có bash (PATH chỉ có `Git\cmd`). Bước 4 (Codex): `sos install` trên Windows ghi lệnh hook `python -c "…"` chạy được trong cmd, PowerShell và Git Bash; chặn bằng `permissionDecision` "deny" + exit 0 (`powershell -Command` biến exit 2 thành 1). Người soát Codex APPROVE sau 4 vòng (`evidence/roles/`, không commit). **Còn mở, việc của Chủ nhà:** trong một phiên Codex thật trên Windows, `/hooks` trust rồi thử sửa `.env` (phải bị từ chối, file không đổi) và một file thường (phải qua). `.codex/hooks.json` đã có sẵn trong app (như POD-agent) giữ dạng cũ; muốn dạng mới thì xóa file rồi chạy lại `sos install` (file của dự án: phần sửa riêng sẽ mất). POD-agent: `money-guard` chỉ gắn cho Claude, `.codex/hooks.json` không chặn lệnh shell — chờ Chủ nhà quyết.
- [ ] **Hồ sơ theo loại dự án** (sau brainstorm): `ios-app`, `writing` trước; `web-app`, `tool` sau. Mỗi hồ sơ: lăng kính cho người soát, nội dung `make ready`, gate thêm. Hồ sơ `ios-app` thêm lease simulator thành khóa file mà `make check`/UI test tôn trọng (TurnSigil 14/10: hai suite chạy chung một simulator, 4/4 hành trình đỏ giả). `reviewer.md` hiện viết cho app; tách phần riêng ra hồ sơ.
- [ ] **Repo tool cũ:** ghi chú "đã gộp vào `sos gate`" ở README của quality-gate, doctor, doc-rotate, claude-hooks.

## Ý tưởng mang sang từ v2 (chưa làm)

- **Gate đầu vào** (P084): kiểm thứ AI đọc vào (tài liệu trỏ file không tồn tại, tài liệu lệch code) trước khi nó suy luận. Chỗ chứa tự nhiên: `sos gate`.
- **Kiểm riêng tiếng Việt** (P013): dấu, VND, GMT+7, font, xuất PDF — ứng viên cho hồ sơ app/web tiếng Việt.
- **Swift/macOS** (P048): kiểm `make ready` cho iOS (build, test, simulator lease).

## Đã xong (v3)

- 08/10 — Bước 0–2.2: cất trạng thái v2 (tag `v2-final`), phân loại tool/hook, archive workflow v2, role Lite có advisor và người soát nhìn từ ngoài vào.
- 09/10 — 2.3a tách tầng: CLI trung lập (`scripts/`) + adapter Claude/Codex chỉ dịch payload; fixture thật; chạy thật cả hai agent.
- 09/10 — 2.3b gộp quality-gate, doc-rotate cap, features-guard, runtime-scan vào `sos gate`.
- 09/10 — 2.4 `sos install / update / check`; v2 CLI và crates vào archive; `install.sh` chỉ tải `sos`.
- 09/10 — 2.5 phát hành v0.3.0. `main` được push thẳng bằng quyền admin, bỏ qua luật PR; từ đó mọi thay đổi đi qua PR.
- 09/10 — dọn repo: tài liệu, mẫu, cấu hình v2 vào `archive/v2/`; BACKLOG, PHILOSOPHY, skill `apply` viết lại cho v3.

## Ngoài phạm vi sos-kit

- `guard` còn 569 dòng chưa commit (repo riêng). Thirty còn việc dở không thuộc harness (DESIGN/SOUL/hồ sơ App Store).
