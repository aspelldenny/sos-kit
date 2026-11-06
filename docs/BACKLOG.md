# sos-kit Backlog

> Việc tiếp theo của chính sos-kit. Mục `##` đầu tiên là sprint đang chạy (banner đầu phiên hiện nó). Lịch sử v2 (P001–P088, phiếu, harvest): `archive/v2/docs/BACKLOG-v2.md`.

## Đang chạy — SOS Kit v3 dùng thật

> Nguồn: `docs/research/HARNESS_SURVEY_2026-10-08.md`, `docs/plans/V3_TRIAGE_2026-10-08.md`. Phát hành mới nhất: xem `CHANGELOG.md`.

- [ ] **Rust hoá tool agent (Chủ nhà 04/11/2026), kế hoạch `docs/plans/RUST_ALL_TOOLS_2026-11-04.md`.** Các đợt đã xong và phát hành ở v0.4.0. **Tiếp:** đổi chỗ gọi ở từng repo và ở vault, theo `docs/plans/RUST_HANDOFF_2026-11-05.md` «Việc chờ Sếp». Việc nhỏ còn lại cho `sos usage`: đọc kết quả subagent chạy nền; gom chuỗi trùng để bớt bộ nhớ. Lịch sử: `docs/plans/BACKLOG_DETAIL_2026-11-06.md`.
- [ ] **Áp vào app mới** khi Chủ nhà brainstorm xong: `sos install`, đo token, thời gian, lỗi bắt được theo tầng (gate / advisor / người soát), số lần advisor bắn và đổi hướng thợ. Chốt model và effort cho advisor sau khi có số đo.
- [ ] **Tự dọn worktree và nhánh đã gộp (Chủ nhà 05/11/2026), làm trước.** Lỗi gặp lại nhiều lần: code xong, PR gộp vào main, nhưng worktree (kèm `target/`, `node_modules`) và nhánh nằm lại. Agent không nhớ xoá, Chủ nhà chỉ biết khi SSD báo động rồi mới nhờ một phiên đi mò lại để xoá. Ngày 05/11 vừa dọn tay: 3 worktree và 14 nhánh remote của sos-kit đã gộp từ lâu. v2 từng có `phieu-done` (P038, v2.1.7: `git worktree remove` + `git branch -d`, nhánh chưa gộp thì giữ và cảnh báo), đi vào `archive/v2/` cùng v2. v3 chỉ có `sos bite` dọn worktree tạm của chính nó. Repo cũng chưa bật "Automatically delete head branches" (`delete_branch_on_merge = false`). Theo luật "cơ chế, không phải trí nhớ", cần:
  1. Lệnh con mới của `sos` (tạm gọi là clean): `git fetch --prune`, liệt kê worktree và nhánh local đã nằm trong `origin/<main>` (`git merge-base --is-ancestor`, kể cả squash-merge: so cây hoặc PR đã đóng qua `gh` nếu có), xoá bằng `git worktree remove` (không `--force`) và `git branch -d`; bỏ qua worktree còn file chưa commit, nhánh chưa gộp, worktree đang có phiên chạy; `--dry-run` mặc định khi không có TTY? (chốt khi làm). Phủ cả worktree app Claude tạo (thư mục .claude/worktrees của repo) và worktree anh em kiểu `~/<repo>-xxx`.
  2. Kích tự động, không cần ai nhớ: banner đầu phiên (`sos hook … session-start`) báo "N worktree/nhánh đã gộp, X GB" và gợi lệnh; git hook `post-merge` (sau `git pull` main) chạy phần an toàn; `sos check` báo cùng số liệu.
  3. Báo dung lượng: cộng `target/`, `node_modules` trong các worktree đó, và `~/.cargo-target-shared`; ngưỡng cảnh báo trong `.sos.toml`.
  4. Một lần, ngoài code: bật `delete_branch_on_merge` cho các repo (cần Chủ nhà đồng ý vì đổi cài đặt repo).
  Test: repo giả có worktree đã gộp, chưa gộp, bẩn, squash-merge; chỉ cái đã gộp và sạch bị xoá.
- [ ] **`run_limited` giết cả cây process khi hết giờ (sau v0.4.0, mục 5 `docs/plans/RUST_HANDOFF_2026-11-05.md`).** `crates/sos-cli/src/hook/mod.rs` `run_limited` chỉ giết process con trực tiếp; cháu (ví dụ notifier tuỳ chỉnh chạy nền) còn sống và có thể giữ stdout. Người gọi: `guards/sim.rs` (`ps`) và notify trong `watch.rs`. Dùng lại cách của `sos bite` (nhóm process trên Unix, job object trên Windows), như `role::capture` đã làm. Test (Astra đề xuất): notifier đẻ một process con giữ stdout, gồm cả ca process cha thoát sớm; hết giờ thì cả hai chết và hook trả về đúng hạn. Chạy test trên Windows qua máy Win.
- [x] **Thirty / Payquill / Lucilius đã sang v3 (sos 0.3.6), 15/10/2026.** Lucilius bỏ vòng phiếu DRAFT/CHALLENGE/APPROVE (Chủ nhà chốt khi giao việc), vùng nhạy cảm gọi Người soát kèm INVARIANTS của app; luật sản phẩm chuyển vào AGENTS.md để Codex cũng đọc. Thirty và Payquill: `[state] file = docs/PROGRESS.md`, luật riêng của `agents/roles/` vào AGENTS.md "Riêng dự án này". Kiểm chữ cấm bằng probe trên từng repo. Việc theo dõi: đọc lịch sử phiên đầu tiên chạy v3 ở Lucilius (bug Today "Kept") như audit TurnSigil; worktree cũ (payquill-f10/f12/f47, lucilius-p071) chưa có `harness-lite/` nên không chạy hook đến khi rebase lên `main`.
- [x] **Tarot (web) sang v3 + docs theo tầng, 15/10/2026.** Tarot PR #1036 (harness) và #1037 (CLAUDE.md thành bản đồ 66 dòng, BACKLOG chỉ giữ việc mở). Kit 0.3.7: tầng docs trong `harness-lite/README.md`, cảnh báo cỡ lối vào/trạng thái. Còn: Tarot mục đầu BACKLOG vẫn 140 dòng (26 việc mở, nhiều dòng dài); đợt 2 rút gọn các guide (BACKEND, FRONTEND, PROMPTS, DATA, CONVENTION) mỗi guide một PR.
- [ ] **Windows (POD-agent, `E:\Extension`) — giao Claude trên máy Windows, 23/10/2026.** Đã xong từ Mac qua SSH: kit chạy được trên Git for Windows (0.3.9: `python` thay alias Store, UTF-8, CRLF), `[advisor] tests`, toast Windows; POD-agent PR #2 (worktree `E:\Extension-wt\sos-v3`). Còn, theo thứ tự:
  1. `git pull` bản sos-kit ở `C:\Users\Admin\sos-kit` (đang ở v2); bản `sos` phải là bản mới nhất (cài: `cargo install --git https://github.com/aspelldenny/sos-kit --tag <tag> --locked --force sos-cli`).
  2. POD-agent: merge PR #2, kéo về `E:\Extension` (main ở máy hơn origin 5 commit), `sos update`, thêm `[advisor] tests = ["node scripts/check.mjs"]`, xoá worktree `E:\Extension-wt\sos-v3` (junction `run\node_modules` trỏ sang `E:\Extension`). `sos check` phải sạch.
  3. Kiểm trong phiên Claude Code thật: banner đầu phiên có trạng thái kit; sửa `.env` bị chặn; hai lần `node scripts/check.mjs` đỏ → advisor trả lời; toast hiện khi advisor mất đăng nhập (chạy thử `notify()` của `test-watch.py` trong phiên desktop, qua SSH không hiện được).
  4. Codex trên Windows: hook có chạy không, qua bash hay PowerShell (`python "$(git rev-parse --show-toplevel)/…"` hợp cả hai?), bước trust hook. Chưa ai thử.
  5. Alias Store: Chủ nhà tắt `python.exe`/`python3.exe` trong App execution aliases; rồi `python3 harness-lite/scripts/bite` chạy được qua shebang.
  6. Kit: job CI `windows-latest` (Git Bash, gitleaks, `cargo test`, `tests/adapters/run.sh`); bản phát hành `x86_64-pc-windows-msvc` + cách cài không cần Rust; đường dẫn `\\?\E:\…` và `E:\Extension\hooks/pre-commit` in ra cho đẹp.
  - 23/10 trên máy Windows: bước 1 và 6 xong (CI `windows-2022`, release `.exe`), bước 4 đã ghi hook chạy được. **Còn mở, việc của Chủ nhà:** trong một phiên Codex thật trên Windows, `/hooks` trust rồi thử sửa `.env` (phải bị từ chối) và một file thường (phải qua). POD-agent: `money-guard` chỉ gắn cho Claude, chờ Chủ nhà quyết. Chi tiết: `docs/plans/BACKLOG_DETAIL_2026-11-06.md`.
- [ ] **Bật `bite` ở các app, 23/10/2026.** Tarot và Reel đã có `[bite]`. Còn: POD-agent (Claude bên Windows, sau khi kéo kit mới); các app iOS (Lucilius, Thirty, TurnSigil, Payquill, FoamFit) cần lệnh riêng: `xcodebuild test` không `-quiet`, `-only-testing` hẹp, `-derivedDataPath` cố định, lease simulator; đo một đột biến trên một app trước khi điền cho cả nhóm (máy 8 GB). Tarot `src/lib/iap/` còn 3 chỗ chưa có test trên đường tiền. Chi tiết: `docs/plans/BACKLOG_DETAIL_2026-11-06.md`.
- [ ] **Điền `[ready] cmd` ở các app (Chủ nhà duyệt 06/11/2026).** `sos gate ready` đã có; banner và `sos check` báo khi trống. Điền lệnh hành trình xuyên suốt ở từng app khi đụng tới. Còn lại của mục E2E cũ: mẫu gói Hub (người thiết kế khác họ model, kết quả mong đợi từ nguồn ngoài app), gộp vào hồ sơ theo loại dự án. Bằng chứng: `docs/plans/BACKLOG_DETAIL_2026-11-06.md`.
- [ ] **Comment cạnh code: đo trước khi thêm luật.** Ý từ bài "ban comments" (agent chép comment workaround cũ, comment cũ lan như cỏ dại). Đếm trên diff gần đây của agent ở Tarot và Lucilius: comment kể lại code, workaround không có điều kiện gỡ, comment đã sai so với code. Đo lần 1 (23/10, diff code không phải test từ 23/09): comment bằng 20–31% số dòng code ở Tarot, Lucilius, Payquill, 6% ở TurnSigil; workaround/TODO thật: 0 (các chỗ khớp từ khoá là luật D&D "temporary HP" và cỡ chữ `xxxLarge`). Kiểu lặp thật là mã phiếu trong comment (`P576:`, `F30`, `ADR 29`): 25% dòng comment ở Tarot, 21% Payquill, 16% Lucilius, 2% TurnSigil; đó là lịch sử, thuộc về git và CHANGELOG. Chưa thấy lỗi nào do comment gây ra, nên theo luật 2 chưa thêm luật. Nếu có, thêm một dòng vào CONTRACT (giữ comment giải thích điều code không tự nói được; workaround ghi điều kiện gỡ); gate được chỉ phần TODO/HACK mới không có điều kiện gỡ.
- [ ] **Hồ sơ theo loại dự án** (sau brainstorm): `ios-app`, `writing` trước; `web-app`, `tool` sau. Mỗi hồ sơ: lăng kính cho người soát, nội dung `make ready`, gate thêm. Hồ sơ `ios-app` thêm lease simulator thành khóa file mà `make check`/UI test tôn trọng (TurnSigil 14/10: hai suite chạy chung một simulator, 4/4 hành trình đỏ giả). Đã có một phần: `sim-guard` chặn `xcodebuild test` chạy song song hoặc nhân bản simulator (tap-luyen 31/10); UI test qua `make`/script và công cụ simulator MCP chưa được chặn. `reviewer.md` hiện viết cho app; tách phần riêng ra hồ sơ.
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
