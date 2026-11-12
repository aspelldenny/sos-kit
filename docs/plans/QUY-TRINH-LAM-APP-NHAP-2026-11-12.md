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
