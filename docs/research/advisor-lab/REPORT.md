# Advisor lab — tự làm advisor cho thợ (2026-10-08)

> **09/10: đã đưa vào kit** thành `scripts/advise` + `scripts/test-watch.py` + `adapters/{claude,codex}` (xem `adapters/README.md`). `advise-on-fail.py` ở đây là bản thử cũ, chỉ để tham khảo.
>
> Bản thử, chưa đưa vào harness. Mục đích: xem một "advisor tự làm" (đọc được, đổi được model) có chạy được với thợ Sonnet không, tốn bao nhiêu, lời khuyên tốt đến đâu.

## Thành phần
- `advise`: CLI. Gói câu hỏi + lỗi gần nhất + `git status`/`git diff` (bỏ `evidence/`) + brief, gửi cho model mạnh, nhận lời khuyên theo khuôn **VERDICT / WHY / NEXT / RISKS / CHECK**, lưu vào `evidence/advice/`.
  - Backend `claude` (mặc định Opus) hoặc `codex` (model mặc định trong config Codex, đổi bằng `ADVISE_MODEL`).
- `advise-on-fail.py`: hook `PostToolUse` + `PostToolUseFailure` cho Bash.
  - Đếm số lần chạy test fail liên tiếp; tới `ADVISE_AFTER` (mặc định 2) thì gọi `advise` và đưa lời khuyên vào ngữ cảnh thợ qua `additionalContext`.
  - Test xanh thì đếm lại từ đầu; mỗi subagent có bộ đếm riêng; không bao giờ chặn thợ, lỗi nội bộ thì im lặng.
- `settings.example.json`: cách nối hook. `lab/`: bài thử (tính lương tuần có 2 bug: ca vắt qua 00:00 Chủ nhật, làm tròn xu) + `run-e2e.sh`.

## Kết quả (một bài, mỗi cấu hình chạy 1 lần)

| Lượt | Thời gian | Chi phí thợ | Advisor | Kết quả |
|---|---|---|---|---|
| Sonnet 5.5 một mình (hook chưa bắn do lỗi, xem phát hiện 1) | 23 s | $0,138 | — | Sửa đúng cả 2 bug |
| Sonnet 5.5 + Opus 5.5 (hook bắn ở lần fail đầu) | 46 s | $0,132 | 22 s; lúc đó còn nạp mặc định, khoảng $0,6–0,7 khi cache nguội | Thợ nhận lời khuyên ("continue"), làm theo, sửa đúng |
| Sonnet 5.5 + Codex `gpt-5.6-sol` | 33 s | $0,130 | 15 s; tính vào gói ChatGPT | Lời khuyên "change approach" đúng; thợ làm theo, sửa đúng |
| Giả lập thợ sửa sai (đổi sang `round()`) → Codex | — | — | 26 s | Chỉ đúng bug gốc **và** bắt thêm bug ẩn test không phát hiện: `round()` là banker's rounding, và đang làm tròn hai lần |
| `advise` Opus sau khi tối ưu | — | — | 15–19 s, **$0,04–0,06** | Lời khuyên đầy đủ: số dòng, công thức số nguyên, các trường hợp biên |

**Đọc kết quả:** Sonnet 5.5 tự giải được bài này. Với ngưỡng thật là 2 lần fail, advisor sẽ không bắn ở đây, nên ở việc dễ thì gần như không tốn thêm. Giá trị của advisor phải đo trên **bài khó hơn, ở app thật**. Chất lượng lời khuyên: Opus chi tiết hơn và nêu nhiều trường hợp biên hơn; Codex gọn, nhanh hơn, và là "mắt khác họ".

## Phát hiện (chỉ lộ ra khi chạy thật)
1. **Pipe nuốt exit code.** Thợ chạy test kiểu `… 2>&1 | tail -15`, nên lệnh trả exit 0 dù test đỏ, và Claude Code báo `PostToolUse` thay vì `PostToolUseFailure`. Hook phải nhận diện fail **từ nội dung output** (`FAILED (`, `FAIL:`, `** TEST FAILED **`…), không tin exit code.
2. **`claude -p` nạp khoảng 77k token mặc định** (MCP, skill, plugin của máy). Khi cache nguội, một lần hỏi Opus tốn khoảng $0,69. Thêm `--system-prompt … --strict-mcp-config --disable-slash-commands --setting-sources ""` thì chỉ còn khoảng 1,4k token: rẻ hơn khoảng 36 lần. (`--bare` đòi API key, không dùng được với gói Max.)
3. **Advisor bị nhiễm qua `git diff`.** Lời khuyên nhắc tới `evidence/e2e.json` của lượt trước. Phải loại `evidence/` và trạng thái hook khỏi gói gửi đi.
4. **Dữ liệu thử bị nhiễm (lỗi của Quản đốc).** `git add -A` commit nhầm bản đã sửa thành baseline, nên lượt sau "không có bug". Đã sửa: gắn tag `baseline-bug`, và script tự dừng nếu baseline không đỏ.
5. `gpt-6.1-sol` bị từ chối vì `codex` trong terminal (npm, 0.146) quá cũ; bản trong app ChatGPT (0.162) chạy được. Đã nâng `npm i -g @openai/codex@latest` lên 0.162. `advise` không còn gắn cứng model.

## Giới hạn
- Một bài dễ, mỗi cấu hình chạy 1 lần; chất lượng lời khuyên do Quản đốc tự chấm.
- Chi phí Codex không đo theo lượt (tính vào gói).
- Chưa thử trong subagent của một phiên Quản đốc; mới thử với thợ headless `claude -p`. Docs nói hook chạy cả trong subagent (có `agent_id`), nhưng chưa kiểm thật.

## Đề xuất bước tiếp
- Áp vào app mới (bước 3 của v3) ở dạng **tuỳ chọn**: ngưỡng 2, advisor mặc định Opus (lean), Codex khi muốn mắt khác họ.
- Đo số lần bắn, chi phí, và số lần lời khuyên đổi hướng thợ.
- Chọn model và effort sau khi có số đo, theo ý Chủ nhà.
