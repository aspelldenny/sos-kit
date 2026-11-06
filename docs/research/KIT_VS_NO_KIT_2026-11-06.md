# So sánh các đời kit trên app thật (06/11/2026)

Câu hỏi của Chủ nhà: làm app nhà và app tập luyện có nhiều lúc ức chế vì code sai, bug lọt tới tay. Code bằng SOS Kit và code chay hơn kém nhau ở đâu, xét tốc độ, độ chính xác, tiết kiệm.

**Đính chính của Chủ nhà, cùng ngày.** Bản đầu xếp Lucilius, Thirty, Tally vào nhóm "chưa có kit". Sai: ba app này dùng kit đời cũ từ đầu, trước khi lên v3 ngày 15/10. Code chay thật chỉ có Tập luyện (18–29/8). Bảng dưới đã chia lại theo ba đời.

| Nhóm | App | Ghi chú từ git |
|---|---|---|
| Chay | Tập luyện đến 29/8 | Không có file harness hay agent nào trước 26/10 |
| Kit đời cũ | Lucilius, Thirty, Tally trước 15/10 | Lucilius: SOS v2 từ 02/09. Thirty: harness-lite v0.2 từ 02/09. Tally: mẫu từ Thirty, từ 22/09 |
| v3 | App nhà, TurnSigil, Reel từ đầu; Tập luyện từ 26/10; ba app trên từ 15/10 | TurnSigil lên v3 ngay ngày đầu |

**Kết luận.** So với code chay, kit thắng rõ về độ chính xác ở lỗi logic và dữ liệu; bằng chứng trực tiếp chỉ đến từ một app là Tập luyện. Kit không làm chậm đi. Tiết kiệm chưa chứng minh được, và v3 chưa hơn rõ đời cũ. App nhà lọt lỗi một phần vì chưa đi thử hành trình, một phần vì đụng phần cứng thật mà simulator không thay được.

| Tiêu chí | Kết quả | Bằng chứng chính |
|---|---|---|
| Tốc độ | Kit không chậm hơn | Commit/ngày: chay 9, đời cũ ~20,5, v3 từ đầu ~16 |
| Độ chính xác | Kit thắng | Ít nhất 108 lỗi chặn bị bắt trước Chủ nhà ở hai app |
| Tiết kiệm | Chưa rõ | Ở Tập luyện, theo commit là ngang, theo dòng code đắt hơn 2–3 lần |

## Nguồn và giới hạn

Nguồn: lịch sử git từng repo, transcript Claude và Codex, `.advise-state/calls.jsonl`, `evidence/`, tin nhắn của Chủ nhà (kể cả phiên mẹ `ef70375c`, nơi Chủ nhà nói chuyện về app nhà), đọc tay các lỗi.

Đây là thí nghiệm tự nhiên, không có đối chứng:

- **Chỉ một mẫu chay.** Tập luyện là phép so sánh duy nhất đáng tin giữa chay và kit: chay 18–29/8, cài kit 0.3.12 ngày 26/10. FoamFit chay 2 ngày trên git, quá ít để so. App nhà có kit từ ngày đầu nên không có đối chứng chay.
- **Loại sản phẩm khác nhau.** App nhà điều khiển phần cứng thật; các app kia gần như không. Số lỗi lọt của app nhà không so thẳng được với app khác.
- Giai đoạn sau kit ngắn và lệch pha: dựng mới khác sửa và bảo trì. Tally, FoamFit, Tarot sau kit quá ít việc để so.
- Tỷ lệ commit sửa lỗi phụ thuộc cách đặt tiêu đề.
- Số dòng có cả code sinh ra.
- Giá model đổi giữa các giai đoạn. Đọc cache từ 0,5 xuống 0,2 đơn vị nên tiền giảm phần lớn nhờ model. Đô la theo `crates/sos-cli/src/usage/prices.toml` chỉ để so sánh, không phải hóa đơn.
- Token quy đổi theo trọng số cố định: vào 1, ra 5, ghi cache 1,25, đọc cache 0,1.
- Transcript còn giữ một phần. Reel thiếu dữ liệu (5 ngày đầu chỉ ghi được $7), Tarot chỉ có 26 trên 127 ngày, nên cả hai không dùng để kết luận chi phí.

## Tốc độ

| App | Đời kit | Commit/ngày | Dòng code mới/ngày | Commit có test | Test:code |
|---|---|---|---|---|---|
| App nhà | v3 từ đầu | 25,0 | 6.077 | 86% | 0,64 |
| Reel | v3 từ đầu | 14,0 | 4.657 | 67% | 0,32 |
| TurnSigil | v3 từ đầu | 10,2 | 1.260 | 75% | 0,98 |
| Tally | đời cũ | 21,8 | 2.099 | 88% | 1,26 |
| Thirty | đời cũ | 21,0 | 2.400 | 90% | 1,07 |
| Lucilius | đời cũ | 19,7 | 7.810 | 33% | 0,12 |
| Tập luyện trước kit | chay | 9,0 | 2.308 | 4% | 0,01 |
| Tập luyện sau kit | v3 | 10,0 | 1.356 | 80% | 0,53 |

Thirty và Tally đã có tỷ lệ test cao ở đời cũ, nên bước nhảy lớn về test là từ chay lên có kit, không phải riêng công của v3.

Người soát Codex tốn 124 phút ở app nhà (32 lượt) và 146 phút ở Tập luyện (53 lượt), khoảng 11–13% số giờ máy chạy.

## Độ chính xác

**Kit bắt được**

- Lỗi chặn bắt trước Chủ nhà, đếm thấp nhất từ tiêu đề commit: app nhà ≥62, Tập luyện ≥46.
- Ở Tập luyện, 41 trên 45 báo cáo trong `evidence/roles` nhắc lỗi chặn, 2 báo cáo được ghi là sạch.
- `sos bite` lộ ba mutant sống ở Tập luyện, tức ba chỗ test chưa bắt được code sai.
- Trước kit, Tập luyện lọt lỗi ngay trên app: nhảy ra khi mở thư viện, không tắt được bàn phím, lịch lặp vô tận. Góp ý 30/10 sau kit chủ yếu về nội dung.

**Kit không chặn được**

Khoảng 8–10 lỗi tới tay Chủ nhà trong khoảng 2 ngày dùng app nhà thật (02–03/11):

- Code không nối vào đâu: `Room.mergedItems` không ai gọi (`cccdb63`).
- Dữ liệu giả khác thật: `GET /devices` thật không có `capabilities`, bản demo có sẵn nên ảnh simulator trông đúng.
- Lệnh mất, bấm không phản hồi, mở app không thấy điều hòa.
- Phần cứng: một gói WoL không đủ đánh thức TV.

Ở Tập luyện, ngày 02/11 màn hình chính mở bản nháp cũ từ 18/8 và Chủ nhà tập cả buổi trong đó, dù đã qua 222 test và các vòng soát (`00bdd4a`).

Lỗi không tắt được bàn phím lặp lại ở Tập luyện, Lucilius trước kit và app nhà có kit. Test đơn vị và soát code đều không nhìn thấy loại này.

**Khác biệt quyết định: có đi thử cả hành trình hay không**

- Tập luyện: agent đi simulator theo J01–J08 ngày 30/10, tìm ra 25 lỗi (F1–F25) trước khi Chủ nhà đụng vào.
- App nhà: 903 test đơn vị, không có test giao diện hay chạy thử đầu cuối. Bàn giao ngày đầu chốt "chưa sim/e2e, chỉ code + unit test đến khi thiết bị về", nên 7 ngày đầu chưa chạm thiết bị thật. Chủ nhà thành người kiểm thử cuối.
- Hai app không ngang nhau: Tập luyện chạy trọn trên simulator, app nhà thì không. Đi simulator có thể bắt loại lỗi giao diện và code chưa nối (`mergedItems`, bấm không phản hồi, bàn phím đơ). Lỗi cần thiết bị thật thì không (mất lệnh, hộp FPT, đánh thức TV); dữ liệu `GET /devices` thật cũng chỉ lộ khi có bridge thật hoặc bản ghi từ nó.
- `[ready] cmd` trống ở mọi app đã xem, nên chưa gì kiểm toàn bộ sản phẩm trước bàn giao.

## Tiết kiệm

Commit code là commit có đụng code, không tính commit chỉ sửa tài liệu.

| App | Đời kit | $/commit code | Token/commit code (giá cố định) |
|---|---|---|---|
| Lucilius | đời cũ | 3,6 | 0,51M |
| Tập luyện | chay | 14,4 | 1,83M |
| Thirty | đời cũ | 11,0 | 2,3M |
| Tally | đời cũ | 21,5 | 7,1M |
| TurnSigil | v3 từ đầu | 3,2 | 1,19M |
| App nhà | v3 từ đầu | 4,1 (có phiên mẹ $78) | từ 1,26M, chưa tính Codex và phiên mẹ |
| Tập luyện | v3 | 3,6 | 1,25M, tới 1,75M nếu tính Codex |

Cùng một app, Tập luyện: token/commit gần như ngang trước kit khi tính cả Codex. Theo 100 dòng code, 0,36M trước kit thành 0,75–1,07M sau kit, vì commit nhỏ hơn và nhiều vòng soát hơn. Giữa các đời kit không có người thắng rõ: đời cũ từ 0,51M (Lucilius) tới 7,1M (Tally), v3 khoảng 1,2M. Các bước lên v3 cùng app cũng không so được, vì Lucilius và Thirty sau 15/10 chủ yếu là bảo trì.

## Việc rút ra

Theo thứ tự ưu tiên, đã ghi vào `docs/BACKLOG.md`:

1. Bắt buộc đi thử cả hành trình trước bàn giao: điền `[ready] cmd` cho từng app. Không thay được thiết bị thật ở app phần cứng.
2. Với app có phần cứng hoặc API ngoài: test bằng dữ liệu thật (như `RealBridgeTests`), chạm thiết bị thật sớm.
3. Gate phát hiện code không ai gọi, chặn sớm lỗi kiểu `mergedItems`.
