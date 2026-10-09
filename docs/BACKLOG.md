# sos-kit Backlog

> Việc tiếp theo của chính sos-kit. Mục `##` đầu tiên là sprint đang chạy (banner đầu phiên hiện nó). Lịch sử v2 (P001–P088, phiếu, harvest): `archive/v2/docs/BACKLOG-v2.md`.

## Đang chạy — SOS Kit v3 dùng thật

> Nguồn: `docs/research/HARNESS_SURVEY_2026-10-08.md`, `docs/plans/V3_TRIAGE_2026-10-08.md`. Phát hành mới nhất: xem `CHANGELOG.md`.

- [ ] **Áp vào app mới** khi Chủ nhà brainstorm xong: `sos install`, đo token, thời gian, lỗi bắt được theo tầng (gate / advisor / người soát), số lần advisor bắn và đổi hướng thợ. Chốt model và effort cho advisor sau khi có số đo.
- [ ] **Chuyển Thirty / Payquill sang v3** (khi Chủ nhà muốn): `sos install --force-hooks`, `[text].files` lấy từ hook cũ, nối `.claude/settings.json`, sửa `AGENTS.md` trỏ `harness-lite/roles` thay `agents/roles`. Đã thử trên bản clone Thirty. `sos check` xanh chỉ xác nhận wiring, không phát hiện chỉ dẫn cũ mâu thuẫn với `harness-lite/`: đối chiếu `AGENTS.md`/`CLAUDE.md` cũ bằng tay, và thêm mục Design handoff.
- [ ] **Hồ sơ theo loại dự án** (sau brainstorm): `ios-app`, `writing` trước; `web-app`, `tool` sau. Mỗi hồ sơ: lăng kính cho người soát, nội dung `make ready`, gate thêm. `reviewer.md` hiện viết cho app; tách phần riêng ra hồ sơ.
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
