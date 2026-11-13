# Quy trình làm một app (bản nháp một trang, 12/11/2026)

Trạng thái: Chủ nhà đọc và bảo làm (12/11/2026). Năm chỗ đổi đã vào kit, phát hành ở v0.6.0 (xem `CHANGELOG.md`); sáu chặng vẫn là giả thuyết cho tới khi chạy qua một app thật.
Nguồn: hai gói Hub `NEN-NGHE-UIUX-SOS-KIT-2026-11-12` và `QUY-TRINH-LAM-APP-SOS-KIT-2026-11-12`, cộng lịch sử commit của năm app.

## Đổi gì so với bây giờ

Kit hiện bắt đầu từ lúc xây và chỉ soát sau khi xây. Bản mới thêm phần trước khi xây: định hình, rồi soát bản vẽ. Màu và font lùi về sau.

## Sáu chặng

| # | Chặng | Ai làm | Ra cái gì | Chủ nhà làm gì |
|---|---|---|---|---|
| 1 | Vấn đề | Chủ nhà + Hub | Một trang: app cho ai, họ vướng gì, bằng chứng, mình khác ở đâu | Đưa ý, chốt trang |
| 2 | Định hình | Kiến trúc sư | Bản vẽ: 3–5 luồng chính kèm ví dụ cụ thể, mô hình dữ liệu, những việc không đảo ngược được | Không cần làm gì |
| 3 | Soát bản vẽ | Người soát (model ngoài) | Danh sách chỗ mơ hồ, chỗ hai mảnh hiểu lệch nhau | Đọc danh sách, chốt các câu chỉ Chủ nhà trả lời được |
| 4 | Bộ xương | Thợ | Một luồng chạy thật từ đầu tới cuối trên dữ liệu thật, có test hành trình | Không cần làm gì |
| 5 | Lớn dần | Thợ + thiết kế | Từng luồng còn lại; màu, font; sàn nghề cho từng loại thành phần | Chọn gu khi được hỏi |
| 6 | Soát bản chạy | Người soát, rồi Chủ nhà | Lỗi tìm được trên bản chạy | Cầm máy dùng thử |

Chặng 1 nằm ngoài kit (việc của Hub). Kit bắt đầu từ chặng 2.

## Kit đổi năm chỗ

1. **Kiến trúc sư tách lại khỏi Quản đốc ở chặng 2.** Bản vẽ phải có ba phần ghi ở bảng trên. Thiếu phần nào thì chưa sang chặng 3.
2. **Người soát có thêm một lần gọi: bản vẽ vừa xong.** Mỗi lần gọi, phiếu ghi rõ đang soát cái gì (bản vẽ, diff, hay bản chạy) và mục tiêu viết sẵn để chấm theo. Người soát chỉ nêu vấn đề, không tự sửa.
3. **Bộ xương là cổng.** `sos gate ready` phải xanh trên một luồng trước khi Thợ mở luồng thứ hai.
4. **Sàn nghề màn hình tính theo loại thành phần** (sheet, ô nhập, nút chính, danh sách…), không theo từng màn. Một app có khoảng mười mấy loại. Nội dung lấy từ gói Hub, viết lại bằng lời mình.
5. **Mỗi lỗi ghi thêm chặng nào để lọt.** Dòng `Caught-by:` thêm một chữ: lọt từ bản vẽ, từ bộ xương, từ lúc lớn dần, hay từ lần soát. `sos score` đếm theo chặng.

## Tìm chỗ gãy thế nào

Chủ nhà không cần đọc để đoán chỗ gãy. Chỗ số 5 làm việc đó: sau một app, bảng đếm cho thấy lỗi dồn ở chặng nào, và mình sửa đúng chặng ấy trong kit.

## Chưa có trong bản này

- `sos memory` (kinh nghiệm tích từ app này sang app khác): làm sau, khi bảng đếm ở chỗ 5 cho biết loại lỗi nào lặp lại.
- Cách sinh ra "hướng C": sách không có thủ tục. Tạm thời chỉ là một dòng trong vai Kiến trúc sư: thấy cả A và B đều vướng thì nói ra trước khi xây.
- Sách viết cho đội người. Sáu chặng này với một chủ và nhiều agent là giả thuyết, chưa ai kiểm.

---

# Bản đầy đủ A→K: cả xưởng, không chỉ phần code (13/11/2026)

Trạng thái: Sếp chốt cách 4 (13/11/2026). Viết bởi session Hub.
Vì sao: từng mảnh (Spy, Hub, sos-kit, Claude Design) đều chạy, nhưng thứ nối chúng là trí nhớ của Sếp. Thiết kế lúc gọi lúc không. Thử nhu cầu hay bị bỏ. Bảng dự án ghi sai chặng (Ninety-Five đang code mà bảng vẫn ghi «SOUL»).

## Ba lớp, mỗi lớp một chủ

1. **Spec A→K** (mục này). Trung lập, không nằm trong `.claude`, Claude hay Codex đều đọc được.
2. **File trạng thái trong từng repo.** Repo tự biết mình đang ở chặng nào và đã qua cổng nào. Phần này của sos-kit.
3. **Tổng đài đọc file trạng thái** để cập nhật cột «Chặng» trên bảng và nói bước kế: gọi ai, đưa brief nào. Không ai phải nhớ.

Sáu chặng ở trên vẫn giữ nguyên. Chúng là E, G, H của bản này.

## Mười một chặng

| Chặng | Ai làm | Ra cái gì | Cổng để sang chặng sau | Sáu chặng cũ |
|---|---|---|---|---|
| **A. Spy** | Session Spy + Hub | `SPY-*.md` trong `BOOK/PROJECT-RESEARCH/<app>/` | Hub ra phán quyết làm hay bỏ | 1 |
| **B. SOUL** | Hub | SOUL (vibe, why, triết lý, kết nối) + PRD một trang, có **aha moment viết trong một câu** | Sếp gật SOUL | 1 |
| **C. Thử nhu cầu** | Hub thiết kế, Sếp đăng bài | Số đo so với ngưỡng đã đặt trước | Đạt ngưỡng. Bỏ qua thì phải ghi lý do; «bỏ qua» là một kết quả hợp lệ, không phải lỗ hổng | — |
| **D. Tạo repo** | Quản đốc | Repo đã cài sos-kit; SOUL, PRD, phán quyết Hub nằm trong repo; file trạng thái điền ngược A–C | `sos check` xanh | — |
| **E. UX / BLUEPRINT** | Kiến trúc sư, rồi Người soát | Hành trình kèm ví dụ cụ thể, mô hình dữ liệu, các thao tác không đảo ngược được; hành trình đầu tiên phải dẫn tới aha moment của B | `sos gate blueprint` + Người soát | 2, 3 |
| **F. Thiết kế nhìn** | Claude Design (Hub soạn brief) | Mockup, màu, chữ, thành phần | Sếp duyệt mockup **trước lát đầu tiên của H**. Muốn dùng UI mặc định thì ghi «mặc định» kèm lý do | 5 (phần thiết kế) |
| **G. Bộ xương** | Thợ | Một hành trình chạy trọn trên dữ liệu thật, UI mặc định là được | `sos gate skeleton` | 4 |
| **H. Lớn dần + soát** | Thợ, Người soát (Codex) | Đủ các lát, đúng sàn nghề CRAFT-UI | Sếp cầm máy thật dùng thử | 5, 6 |
| **I. Chuẩn bị ra mắt** | Quản đốc + Hub | Trang store, privacy, chỗ đo đúng số đo đã hứa ở B/C, kế hoạch kênh dựng từ C | Danh sách kiểm đủ dấu tích | — |
| **J. Ra mắt + đo** | Sếp + repo | Số liệu theo luật 60 ngày | Hết 60 ngày | — |
| **K. Quyết định** | Hub + Sếp | Làm lớn, giữ nguyên hay dừng; dừng thì có các bước tắt (gỡ store, báo người dùng, lưu trữ repo) | Ghi vào file trạng thái | — |

## Thiết kế mà không giết aha moment

- Aha moment chốt ở B rồi thành hành trình ở E, đều trước khi có hình.
- Claude Design nhận BLUEPRINT làm đề bài. Chiều ngược lại không xảy ra: Kiến trúc sư không nhìn mockup.
- F chạy song song với G, chỉ chặn lát đầu tiên của H. Vì vậy app nào cũng đi qua F, kể cả khi kết quả là «mặc định».

## File trạng thái (hợp đồng tối thiểu; sos-kit chọn tên và chỗ)

Gợi ý là bảng `[stage]` trong `.sos.toml`. Đã có file đó thì không đẻ thêm file.

```toml
[stage]
current = "G"
[stage.gates]   # chặng = "ngày | bằng chứng (đường dẫn)" hoặc "bỏ qua | lý do"
A = "2026-11-10 | BOOK/PROJECT-RESEARCH/APP-5-6-2026-11-12/evidence/SPY-NICHE5.md"
B = "2026-11-12 | SOUL.md"
C = "bỏ qua | Sếp chọn build ngay, luật 60 ngày (HUB-VERDICT vòng 4)"
D = "2026-11-12 | sos check xanh"
E = "2026-11-12 | docs/BLUEPRINT.md"
F = "mặc định | Sếp chọn UI mặc định 13/11"
```

Kit cần:
- (a) Một lệnh in chặng hiện tại, cổng còn thiếu và bước kế (ai làm, đọc gì). Gợi ý `sos stage` / `sos next`.
- (b) Một cổng chặn **lùi lại**: không ghi chặng sau khi chặng trước chưa có ngày hoặc «bỏ qua | lý do». Kit kiểm có mặt, không kiểm chất lượng; chất lượng vẫn là việc của các cổng sẵn có.
- (c) `sos install` tạo sẵn bảng này. `sos update` không đè lên.

## Tổng đài cần

- Đọc `[stage]` của mọi repo trên bảng (`kiem.py` hoặc bản Rust) và sửa cột «Chặng» theo đó, không ghi tay.
- Chào sáng có thêm một dòng «bước kế theo chặng» cho từng app 💰.
- Chặng A–C chưa có repo thì ghi ở `BOOK/PROJECT-RESEARCH/<app>/` (Hub giữ). Sang D, Quản đốc chép ngược vào repo.

## Thử trên app thật

Ninety-Five là app đầu tiên dùng. Điền ngược theo ví dụ ở trên. Đang ở G, C và F đã ghi lý do. Đo xem bảng có còn lệch nữa không.

## Sổ tay từng chặng: A→K là xương sống, chi tiết ở đâu (13/11/2026)

A→K chỉ nói thứ tự, cổng và chặng hiện tại. Cách làm từng chặng nằm ở sổ tay riêng, không viết lại ở đây.

| Chặng | Sổ tay chi tiết | Khâu và chặng trong PLAYBOOK bán app |
|---|---|---|
| A–C | `BOOK/PROJECT-RESEARCH/<app>/` (phán quyết Hub), `~/code/aso-spy` | Khâu 1 |
| D–H | sos-kit (doc này, `harness-lite/`) | Khâu 2 |
| I | `BOOK/PROJECT-RESEARCH/BAN-APP-PLAYBOOK/PLAYBOOK.md` §2, CHECKLIST-REPO.md | Khâu 3 |
| J | PLAYBOOK §3 (Apple Ads), §4 (clip social), §5a (mốc, luật xoay vòng) | Khâu 4–5 |
| K | PLAYBOOK §1a (chặng ①–⑧), «Chấm cả chuỗi» | Khâu 6 |

Bài học PLAYBOOK đã thành cổng A→K (đều là giả thuyết; Lucilius là ca gốc):
- **Cổng A:** chưa có số độ phổ biến tìm kiếm thì không chọn ngách. Lucilius: spy bản 1 chấm 29 điểm, chấm lại theo cách mới thì 0 điểm.
- **Cổng E:** BLUEPRINT phải có chỗ đếm chặng ④–⑥ (làm xong việc chính, giữ chân, thấy paywall), không thu nội dung người dùng, và có màn xin đánh giá sau một việc vừa xong. Để tới chặng I mới thêm thì sẽ phải sửa kiến trúc.
- **Cổng I:** 3 ảnh đầu đọc được khi thu nhỏ, ảnh 1 là luồng chính (ảnh 1–2 của Lucilius là khối chữ). Phụ đề chứa đúng những chữ sẽ chạy quảng cáo.
- **Chấm ở K:** mỗi lỗi ghi «lọt từ chặng nào của A→K», giống `Escaped-from:` nhưng tính cho cả xưởng. Buổi chấm cả chuỗi Lucilius ngày 14/11 là lần chạy thử đầu tiên.
