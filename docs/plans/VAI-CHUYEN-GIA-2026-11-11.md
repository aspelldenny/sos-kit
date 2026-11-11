# Bớt lời, thêm giác quan, thêm trọng tài: hướng v0.5 (11/11/2026)

Chủ nhà chốt hướng ngày 11/11/2026 sau khi đọc bài Felix Rieseberg ("How to be more ambitious") và bài Fable 5.2 của Reel. Năm câu hỏi của bản bàn ban đầu được thay bằng hướng này; phần Chi tiết bên dưới giữ bằng chứng của bản bàn đó.

## Trang đầu

**Kết luận.** Model cỡ Fable 5.2 đã đủ sức thiết kế các app nhỏ của Chủ nhà (vài trăm tới vài nghìn người dùng). Cái kit còn thiếu không phải trí khôn mà là **mắt** (công cụ đo thật) và **trọng tài** (thứ chấm ở ngoài model). Ba lỗi lọt ở app nhà (đo 06/11) đều thuộc loại này: code không màn hình nào gọi (`mergedItems`), dữ liệu giả khác API thật (`capabilities`), không có lệnh chạy trọn hành trình (`[ready] cmd` trống). Hai bài cùng nói một điều: Felix (nguồn thô, đồ nghề thật, người chấm đứng ngoài) và Fable 5.2 (khung đỡ mỏng đi; gộp subagent; advisor +1,7 điểm mà tốn 2,1×).

**Chỗ nặng của kit hiện nay không nằm ở prompt** (`harness-lite/` cả thảy 181 dòng) mà ở bề mặt: 14 lệnh `sos`, 13 gate, 11 mục `.sos.toml`, hai đường dây song song (`scripts/` + `adapters/` cũ và `sos hook` mới), bốn vai mà Kiến trúc sư <2,4% và Người soát ~3% usage.

**Nguyên tắc (vào `docs/PHILOSOPHY.md`).** Model suy luận và phán đoán. Thứ tool truy được thì đừng bắt model nhớ; thứ tool đo được thì đừng bắt model ước lượng; thứ một phép thử độc lập kiểm được thì đừng để model tự chứng nhận. Ngược lại, đừng cơ khí hoá phán đoán cần hiểu vấn đề. Mỗi thứ trong kit phải chỉ ra được một lỗi thật nó bắt, nếu không thì bỏ.

### Nâng (mỗi thứ gắn với một lỗi đã lọt)

| # | Việc | Lỗi nó nhắm | Trạng thái |
|---|---|---|---|
| N1 | **Sổ `Caught-by:`**: hook `commit-msg` bắt mọi commit sửa lỗi ghi tầng đã bắt (gate, test, bite, reviewer, advisor, owner, user, prod), fail-closed, in cách sửa | "Gặp nhiều ca nhưng không đo nên không rõ sai ở đâu" (Chủ nhà 09/11) | v0.5 |
| N2 | **Bộ chấm `sos score`**: một bảng cho cả app lẫn kit (lỗi theo tầng, tỉ lệ lọt tới người, `[ready]`, `bite`, advisor, nhịp commit), xuất `--json` để so hai lần chạy | Không biết nâng cấp là cải tiến hay cải lùi | v0.5 |
| N3 | Giác quan cho ba lỗi đã lọt: `[ready] cmd` phải có lệnh; gate dò code không ai gọi; kiểm dữ liệu giả lấy từ bản ghi thật | `[ready]` trống, `mergedItems`, `capabilities` | `[ready]` vào `sos score` ở v0.5; hai gate kia cần thử trên app nhà trước (BACKLOG) |

### Rút (chỉ bỏ khi số liệu nói nên bỏ)

| # | Việc | Số liệu | Trạng thái |
|---|---|---|---|
| R1 | **Gộp Kiến trúc sư vào Quản đốc** thành một bước phương pháp; đầu ra còn lại là bất biến viết thành test | <2,4% usage; Fable 5.2: "fold subagents" | v0.5 (giữ lệnh `sos role architect` và `[roles.architect]` cho app cũ) |
| R2 | **Advisor: đo trước, chưa tắt.** `sos score` đếm lượt gọi và lượt hỏng; `advisor` là một tầng trong `Caught-by:` để biết nó có bắt được lỗi không | Đo 20/10: 11 lần gọi, 7 hỏng. Đo lại 11/11 trên `calls.jsonl`: home-control 37/37 chạy, reel 5/5, lucilius 5/5, turnsigil 2/4, thirty 0/5 (tháng 10). Gọi được, nhưng chưa có số nào cho thấy nó làm Thợ đổi hướng. Luật 2: đo trước khi bỏ | v0.5: đo; quyết tắt hay giữ sau khi sổ N1 có vài tuần số liệu |
| R3 | Bỏ đường dây cũ `scripts/` + `adapters/` | Cần đếm app còn dùng | Sau v0.5, khi đã đếm |
| R4 | Tách các lệnh hẹp (`sessions`, `sim`, `filter`) khỏi lõi | Cần `sos usage` | Sau v0.5 |

### Bộ chấm (để so "kit mới" với "kit cũ" và "không kit")

Chạy `sos score --json` trên app sau khi code xong, lưu vào `evidence/score/`. So hai lần chạy cùng loại việc. Các chỉ số:
- **Lỗi theo tầng**: số commit sửa lỗi, chia theo `Caught-by`. Tầng máy (gate, test, bite) bắt càng nhiều, tầng người (owner, user, prod) càng ít thì càng tốt.
- **Tỉ lệ lọt**: lỗi do owner, user, prod bắt chia cho tổng lỗi đã phân loại. Đây là chỉ số chính.
- **Chưa phân loại**: commit sửa lỗi không có `Caught-by` (app chưa có hook, hoặc lịch sử cũ). Với app không dùng kit, cột này là tất cả.
- **`[ready]`**: có lệnh chưa, lần chạy cuối xanh hay đỏ, cách bao lâu.
- **Advisor**: số lần gọi, số lần hỏng.
- **Nhịp**: số commit, số ngày, tỉ lệ commit sửa lỗi trên tổng commit.
- Token và thời gian lấy từ `sos usage --project <app>` (đã có).

### Thứ tự ngày 11/11
1. N1 + N2 (cần cho buổi thử app mới ngày 12/11).
2. R1, R2, câu nguyên tắc.
3. Test, Người soát khác họ model soát, PR, phát hành **v0.5.0** (đổi mặc định advisor và gộp vai là thay đổi hành vi, nên tăng số giữa).
4. Ngày 12/11: Chủ nhà thử app mới với kit mới; chạy `sos score` khi xong.

---

## Chi tiết

### Nguồn
- Lời Chủ nhà 09/11: `~/.claude/projects/-Users-nguyenhuuanh-Downloads-Documents-BOOK/memory/sos-kit-vai-chuyen-gia-code.md`.
- Vai: `harness-lite/roles/{orchestrator,architect,worker,reviewer}.md`, `harness-lite/CONTRACT.md`, `harness-lite/VERIFY.md`.
- Usage: `~/VibeNotes/30_Resources/usage-token-2026-11-05.md` (bảng theo vai, top 5; Kiến trúc sư và advisor không có dòng riêng).
- Lỗi lọt: `docs/research/KIT_VS_NO_KIT_2026-11-06.md` (mục "Kit không chặn được").
- 7 hướng 06/11: thư mục ghi chú chung, `sos-kit-huong-phat-trien-2026-11-06.md`. Hướng 1 (gate docs), 2 (`[ready]`), 3 (`sos usage --by skill`), 5 (`sos gate links`) đã làm ở v0.4.1–0.4.3.
- Bài Fable 5.2 của Reel: `~/code/reel/docs/research/fable-5.2-2026-11-11.md` (12 link, đọc 11/11).

### Bảng người chấm (theo depth-check bước 3)

| Đầu ra | Nguồn thô nhất có | Nguồn đang dùng | Đồ nghề (trong code?) | Ai chấm | Độc lập với người làm? | Vòng khép? |
|---|---|---|---|---|---|---|
| Brief của Quản đốc | Hành trình người dùng thật, bug Sếp báo | Ý Sếp + code | Không | Không ai | — | Không: tiêu chí trong brief không được so với kết quả |
| Báo cáo "sẵn sàng" | Sản phẩm chạy trên thiết bị thật | Báo cáo của Thợ, test lát | `sos gate ready` (có, nhưng `cmd` trống) | Chủ nhà | Có | Có nhưng muộn: Sếp là người kiểm thử cuối |
| Quyết định Kiến trúc sư | Code, schema, dữ liệu thật | Code | Không ("trên kệ": FMEA, fitness function chỉ trong sách) | Không ai | — | Không |
| Code của Thợ | Code + dữ liệu thật của API/thiết bị | Code + fake viết tay (đã có luật "fake từ bản ghi thật" ở VERIFY.md) | Có: compiler, test, `bite`, gate | Gate, Người soát | Gate: có. Người soát: có khi khác họ | Có ở tầng lát |
| Kết luận Người soát | Sản phẩm chạy | Simulator / đọc tĩnh | Có: `bite`, sim | Không ai chấm lại | — | Không: chưa đối chiếu với lỗi Sếp bắt sau |
| Lời khuyên advisor | Log test đỏ | Log test đỏ | `calls.jsonl` | Không ai | Mặc định cùng họ | Không: chưa đếm xanh sau lời khuyên |

### Chỗ hở, theo từng vai

**Kiến trúc sư (hở lớn nhất).** `architect.md` 3 đoạn, toàn quy trình, không persona (tốt theo depth-check bước 2). Nhưng đầu ra là chữ: "quyết định, đánh đổi, cách kiểm". Không có gì buộc "cách kiểm" thành thứ chạy được, và không ai quay lại xem quyết định đúng hay sai. Đồ nghề thật của người thiết kế hệ thống, xếp theo độ rẻ:
1. Bất biến viết thành test (property test, kiểm schema, luật import/phụ thuộc). Đây là cách duy nhất biến lớp 1 thành lớp 2.
2. Danh sách lỗi hỏng của từng ranh giới (mạng mất, thiết bị không trả lời, dữ liệu thật thiếu trường). Lỗi app nhà (`capabilities`, lệnh mất, WoL) đều nằm ở đây.
3. Dự đoán rủi ro có ngày chấm, dùng chung bộ quét với `sos predict`.

**Quản đốc.** Vai đắt nhất (65%) và nhiều chữ nhất (`orchestrator.md` ~5,6 KB). Đã có `sos gate ready` và luật báo cáo tách "chạy thật / simulator / chưa kiểm". Hở: (a) `[ready] cmd` trống nên luật báo cáo không có dữ liệu để trích; việc này đã nằm trong BACKLOG; (b) không so tiêu chí trong brief với kết quả, nên không biết brief nào tốt.

**Thợ.** Có đủ đồ nghề tầng lát. Ba lỗi lọt đã đo (code không ai gọi, fake khác thật, lát xanh ghép đỏ) đều đã có mục BACKLOG riêng. Không cần thêm prompt; cần sổ lỗi để biết các mục đó có đỡ được không.

**Người soát.** Quy trình dày nhất, đúng lăng kính Felix (dùng sản phẩm từ ngoài, `bite`, pre-mortem). Hở: chưa hiệu chỉnh theo Chủ nhà. depth-check mục "What is fine" đã dặn: đếm bao nhiêu lỗi của Chủ nhà nó bắt được, bao nhiêu nó báo mà Chủ nhà bác. Sổ `Caught-by:` cho con số này tự động (owner/user = Người soát bỏ sót, nếu việc đó đã qua soát).

**Advisor.** Nhỏ. Một con số rẻ từ dữ liệu sẵn có: tỉ lệ lần chạy test kế tiếp xanh sau lời khuyên. Có thể gộp vào `sos usage`.

### Fable 5.2: điều đụng tới các vai

Nguồn: bài Reel ở trên. Số liệu là của Anthropic, chưa đo trên app của Sếp. Luật "không dùng `fable` khi chưa hỏi Sếp" vẫn giữ.

- **Đồ nghề trước, model sau.** Thang leo của Anthropic khi một vai hỏng: (1) thêm tự kiểm (gate, test), (2) nâng effort một nấc, (3) lên model lớn hơn ở effort thấp hơn. Bước 1 chính là đồ nghề trong bản này. Với Kiến trúc sư: cho chạy Fable high chỉ đáng thử *sau khi* có bất biến chạy được và dự đoán có chấm; nếu không thì không biết Fable có hơn Opus không. Anthropic cũng khuyên đo điểm so với chi phí trên chính tập việc của mình.
- **Advisor.** Advisor hiếm khi thắng executor chạy một mình (Opus high + advisor Fable 5.1: +1,7 điểm, 2,1× tiền). Nếu luồng chính chạy Fable thì advisor Opus còn yếu hơn chính nó. Củng cố việc đo "xanh sau lời khuyên" trước khi giữ advisor ở mọi vai.
- **Gộp hay tách vai.** Bài "fold subagents" (22/10): một agent kèm skill thường thắng nhiều subagent; chỉ tách khi ngữ cảnh phình, cần chạy song song, hoặc quá nhiều tool. Đây là lý lẽ cho phương án B ở câu 3 (gộp Kiến trúc sư vào Quản đốc, giữ phần đồ nghề). Mô hình Quản đốc + Thợ rẻ hơn vẫn được ủng hộ (rẻ hơn khoảng 1/3 cho việc lớn hơn một ngữ cảnh).
- **Chữ trong vai.** Fable làm theo luật rất sát, nên các dòng "không…" dồn lại làm nó rụt rè: mỗi luật nên kèm phạm vi, lý do, và ngoại lệ trong chính câu. Không ghi ngân sách token hay "nghĩ ít thôi" trong prompt; muốn nghĩ ít thì hạ effort. Liên quan việc rà `orchestrator.md` (5,6 KB) và CONTRACT.
- **Thợ chạy medium dễ dừng sớm.** Bài có đoạn mẫu "Finishing work" giảm nửa số lần dừng sớm, tốn thêm ~30% output. Chỉ thêm nếu sổ lỗi cho thấy Thợ bỏ dở lát.
- **Chạy rẻ, chạy lại phần hỏng.** Low cho tất cả rồi high cho phần hỏng: ~97% ở $0,17/việc, so với high cho tất cả: 95,3% ở $0,29. Mẫu này hợp với Người soát chạy lát lặp lại, nhưng cần sổ lỗi để biết phần nào "hỏng".

### Sổ `Caught-by:` (phác thảo, chưa làm)
- Hook `commit-msg` của app: commit có tiêu đề `fix` (hoặc đụng file test kèm code) phải có trailer `Caught-by: <tầng>`; thiếu thì chặn và in danh sách tầng. Lý do là gate chứ không phải lời dặn: lời dặn ghi sổ tay đã bị bác 06/11 vì dựa vào trí nhớ.
- Tuỳ chọn trailer thứ hai `Should-catch: <tầng>` khi tầng bắt được là owner/user.
- `sos` đọc `git log` để đếm theo tầng, theo app, theo tuần; in ở `sos check`.
- Rủi ro: agent điền bừa. Đối chiếu ngẫu nhiên vài commit mỗi tuần với transcript; nếu sai nhiều thì bỏ.
- Đụng `templates/app/hooks/` nên cần soát độc lập (rule 4).

### Chưa kiểm
- Usage riêng của Kiến trúc sư và advisor: ghi chú usage chỉ có top 5; cần `sos usage --by role` (≈141 giây) nếu Sếp muốn số chính xác.
- Số app thật đang có `[ready] cmd` điền: lấy theo KIT_VS_NO_KIT 06/11 ("trống ở mọi app đã xem"), chưa kiểm lại hôm nay.
- Số liệu Fable 5.2 (advisor +1,7, low→high 97%, 1/3 rẻ hơn) lấy qua tóm tắt của Reel, chưa đọc bản gốc.
