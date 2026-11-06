# Hướng 1–5 sau khi đọc 85 bài về harness, 06/11/2026

> **Trạng thái 06/11/2026:** hướng 1–5 đã xong, phát hành v0.4.1. Hướng 6 và 7 chưa làm, đang chờ điều kiện nêu dưới đây.


Chủ nhà duyệt phương án B ("làm cả 1 đến 5, gồm `[ready]`") ngày 06/11/2026. Hướng 6 và 7 có điều kiện, chưa làm. Mỗi phần dưới đây ghi lỗi đã thấy và quyết định, theo rule 2 của `CLAUDE.md`.

## 1. Đo chữ tự nạp, rồi cắt một lượt

- **Lỗi đã thấy:** BACKLOG của kit có 51 dòng, 17.976 byte, dòng 9 dài 4.256 byte, trong khi banner chỉ in 220 ký tự mỗi dòng. Gate cũ chỉ đếm dòng. Thirty PROGRESS 207 KB, Payquill QA 266 KB.
- **Quyết định:** giới hạn nằm ở `[docs] sized` riêng, vì CHANGELOG của kit (102 KB) là file lịch sử, không phải chữ nạp mỗi phiên. Trống là tắt. Vượt là chặn. Một mục là một dòng vật lý.
- **Đã cắt:** hai câu runtime đã lo (nạp entry, model theo vai); hai đoạn test và chọn check, cùng câu lease tài nguyên dùng chung, sang `harness-lite/VERIFY.md`. Câu lease không xoá được: `sim-guard` chỉ thấy `xcodebuild` gọi trực tiếp, không thấy qua `make` hay script, không thấy database hay thiết bị (Người soát bắt). `CONTRACT.md` từ 6.808 còn 5.152 byte.
- **Không làm:** khối `<important if>`, rule-file do model sinh, cắt nhiều một lần.

## 2. `[ready]`

- **Lỗi đã thấy:** Payquill F35 và F31 mỗi cái xanh riêng, ghép vào main thì đỏ. CONTRACT đã ghi phải chạy E2E nhưng mới là chữ.
- **Quyết định:** một lệnh do app khai, chạy ở ranh giới bàn giao, không thuộc `sos gate all`, không có timeout (chưa có lỗi thật đòi hỏi). Log là văn bản thuần để status của `sos` đọc được mà không cần parser JSON.

- **Kit, chạy thật:** `PASS 9954f25+dirty 2026-11-06T08:32:06 48s exit=0` với lệnh ở `.sos.toml`. Sandbox của Người soát không chạy được lệnh này nên dòng trên là do Thợ chạy.
- **Người soát (codex, ba lượt):** bắt PASS cũ hiện như còn đúng khi cây đã đổi, câu lease bị xoá nhầm, `--by skill` im lặng khi không đọc được gì, và lệnh trong ba dấu nháy bị đọc thành trống; cả bốn đã sửa. Lượt ba còn nêu ba ca banner đọc là "có lệnh": `cmd = """` không đóng, `cmd = make`, và `"""\` nối dòng ra chuỗi rỗng. Không sửa: `sos gate ready` đã chạy thật và chặn cả ba (hai ca TOML hỏng, exit 2; ca chuỗi rỗng, exit 1), còn banner chỉ là lời nhắc viết bằng awk, không nên thành parser TOML.

## 3. `sos usage --by skill`

- **Lỗi đã thấy:** không có số nào cho biết skill nào thật sự được dùng, nên không có căn cứ để bỏ hay giữ.
- **Quyết định:** chỉ đọc dữ liệu có sẵn. Tool call `Skill` trong transcript Claude, kết quả Agent mà `sos usage` đã đếm, `.advise-state/calls.jsonl` cho advisor và `sos role`. Giới hạn: lệnh slash người dùng gõ không phải tool call, Codex không có tool `Skill`.

## 4. Bỏ số cố định ở quy tắc resume

Đo ngày 06/11/2026 trên 814 transcript subagent của máy Mac: 51.245 cặp lần gọi liền nhau, chỉ tính ngữ cảnh trên 5.000 token. "Trượt" nghĩa là lần gọi sau đọc từ cache dưới 10% ngữ cảnh lần trước. Hạn cache lấy theo loại ghi chiếm đa số trong từng file.

| Khoảng nghỉ | Cache 5 phút: số cặp, trượt | Cache 1 giờ: số cặp, trượt |
|---|---|---|
| dưới 5 phút | 24.538, 0,2% | 25.762, 0,6% |
| 5–30 phút | 557, 74,0% | 315, 0,6% |
| 30–60 phút | 6, 100% | 9, 0% |
| 1–2 giờ | 1, 100% | 12, 75,0% |
| 2–24 giờ | 15, 100% | 27, 77,8% |

- **Kết luận:** quyết định nằm ở hạn cache, không nằm ở một giờ cố định. Mốc "idle over an hour" chỉ đúng với cache 1 giờ; với cache 5 phút, nghỉ 5–30 phút đã trượt 74%. Các hàng từ 30 phút trở lên có rất ít mẫu. Bảng bỏ hàng trên 1 ngày (3 cặp).
- **Mốc 300K:** dữ liệu không có ngưỡng nào ở đó. Chi phí một lần trượt tăng theo cỡ ngữ cảnh, nên quy tắc viết thành nguyên tắc và không chuyển số vào `.sos.toml`.
- **Giới hạn của phép đo:** khoảng nghỉ gồm cả lúc subagent chờ tool chạy lâu, không chỉ lúc được resume. Chỉ một máy, một người dùng.

## 5. `sos gate links`

- **Lỗi đã thấy:** `sos check-links` có từ v0.4.0 nhưng không gate nào gọi, nên link chết trong doc agent đọc không ai chặn.
- **Quyết định:** chỉ các doc khai ở `[docs] links`, xét trên nội dung sẽ commit. Không kiểm "doc lệch code" bằng model.

## Ai dùng, dùng ở bước nào, đã chạy thật ra sao

Chủ nhà đặt điều kiện ngày 06/11/2026: tính năng phải có người dùng thật, nối vào luồng đang chạy, và chạy thật trên repo thật. Test xanh chưa tính là dogfood. Các lần chạy dưới đây dùng bản dựng của nhánh này; hai app chạy trên bản clone tạm, không đụng repo gốc.

| Hướng | Ai dùng, ở bước nào | Nối vào đâu | Kết quả chạy thật |
|---|---|---|---|
| 1. Đo chữ | Mọi commit sửa doc đã khai; Quản đốc đọc số byte ở `sos check` | `sos gate docs` trong `all`, tức pre-commit; dòng `loaded at session start` | Kit: BACKLOG cũ bị chặn 4 dòng, bản đã rút qua (17.976 còn 11.775 byte). Tarot: BACKLOG 251.965 byte vượt `max_bytes`, chặn. FoamFit: qua. Chữ nạp đầu phiên: kit 26.339, FoamFit 30.834, Tarot 27.666 byte. |
| 2. `[ready]` | Quản đốc, ngay trước báo cáo bàn giao | Vai `orchestrator.md` bắt trích dòng kết quả; banner đầu phiên; cảnh báo ở `sos check` | FoamFit: PASS rồi FAIL đều ghi log và hiện `Ready:` trên banner; `cmd` trống hiện cảnh báo ở FoamFit và Tarot. Kit: xem dòng ở cuối mục này. |
| 3. `--by skill` | Người quyết giữ hay bỏ một skill, vai, advisor | Rule 2 của `CLAUDE.md`. Không đưa vào banner vì một lần quét 14 GB transcript mất khoảng 150 giây ở bản debug | Toàn máy từ 01/10: 10 skill, 10 loại subagent. FoamFit: advisor 6 lần. `architect` 3 lần lỗi là thật (bị plugin chặn model; gọi ở thư mục không có vai). |
| 4. Quy tắc resume | Quản đốc, mỗi lần chọn resume hay mở thợ mới | Câu trong `orchestrator.md`, nạp mỗi phiên | Chỉ đo được phần dữ liệu ở mục 4. Tác dụng lên quyết định thật chưa đo. |
| 5. `links` | Mọi commit sửa doc đã khai | `sos gate links` trong `all`, tức pre-commit | Kit, FoamFit, Tarot: qua. FoamFit thêm thử một link chết và một anchor chết: `all` thoát 1, in đúng dòng và cách sửa. |

**Điểm yếu còn lại:** hướng 3 yếu nhất. Nó nối vào một câu trong rule, không phải một gate, nên vẫn cần người đọc. Skill `retro` là chỗ dùng định kỳ hợp lý hơn nhưng nằm ngoài repo, ở thư mục skill riêng của Chủ nhà. Hướng 2 ở app chỉ có tác dụng khi `[ready] cmd` được điền; mục BACKLOG đã giữ việc đó.
