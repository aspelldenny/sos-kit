# Soát độc lập — đợt 1a và kiến trúc Rust (05/11/2026)

> Người soát: Codex gpt-6-astra, effort high, chỉ đọc, chạy qua `sos role reviewer --from claude` (Sếp duyệt «Gọi cả hai» qua Tổng đài). Hai lượt: (1) diff đợt 1a `2a9e314` + `c93d48d`; (2) bản kiến trúc `RUST_ARCHITECTURE_2026-11-05.md` kèm các sửa của lượt 1. Bản đầy đủ của Người soát ở thư mục tạm của phiên làm (không commit, có đường dẫn máy); dưới đây là phần cần giữ.

## Lượt 1 — diff đợt 1a: FIX-THEN-MERGE

Người soát dò bằng CLI trên macOS và đọc so với bản Python. `run.sh` không chạy được trong sandbox chỉ đọc (không tạo được thư mục tạm).

| # | Mức | Phát hiện | Sửa |
|---|---|---|---|
| 1 | chặn (lọt cổng) | `env_guard` tách cả `\`, nên trên POSIX file `.env.local\notes.txt` (một tên file) chỉ bị xét phần `notes.txt`: Python chặn, Rust cho qua (Claude `file_path`/`notebook_path` và patch Codex) | Xét cả tên theo POSIX (chỉ `/`) và theo Windows (`/` và `\`), khớp một trong hai là chặn; ca hồi quy cho Claude và Codex |
| 2 | chặn | `CLAUDE_PROJECT_DIR` tương đối: tìm thấy `../../scripts/test-watch.py`, rồi đổi cwd về gốc repo nên đường dẫn trỏ sai, advisor im | Gốc dự án và thư mục scripts luôn tuyệt đối; ca chạy từ thư mục con |
| 3 | chặn | `ps` của sim-guard mất giới hạn 10 giây (Python có `timeout=10`) | `run_limited`: hạn cho cả tiến trình lẫn việc đọc stdout; ca `ps` treo 30 giây, quyết trong ≈10 giây và vẫn chặn lệnh thiếu cờ |
| 4 | chặn | Bước CI mới `SOS_HOOK_BIN=target/debug/sos` là đường dẫn tương đối, `run.sh` đã `cd` vào repo tạm nên trả 127 | `run.sh` đổi `SOS_HOOK_BIN` thành tuyệt đối trước khi `cd`; chạy đúng lệnh CI trên WSL |
| 5 | nên sửa | Codex: `command` không phải chuỗi, hoặc đường dẫn chứa NUL: Python crash nên chặn, Rust cho qua | Patch không phải chuỗi → chặn; NUL trong đường dẫn → chặn (mọi agent); lệnh shell không phải chuỗi → chặn trên macOS như Python |
| 6 | nên sửa | Hạn 300 giây của post-bash không bao việc chờ EOF stdout (tiến trình cháu giữ pipe) | Cùng `run_limited`: hết hạn thì bỏ, không chờ |
| 7 | nhỏ | `٢>/dev/null xcodebuild test`: Python (`isdigit`) chặn, Rust (chữ số ASCII) cho qua | Nhận mọi chữ số Unicode (`char::is_numeric`, rộng hơn một chút nên chỉ chặn nhiều hơn); có ca test |
| 8 | nhỏ | «178 ca qua Rust» chưa đúng hết: hai ca mô phỏng `python3` hỏng vẫn gọi adapter Python; `check()` không kiểm JSON | Hai ca đi qua `sos hook`; `check()` đòi stdout dạng JSON phải parse được. Bản Rust thêm bước dò trình thông dịch (python3 rồi python, Windows ngược lại) vì không còn «trình thông dịch của chính mình» như adapter Python |

Sau khi sửa, trên WSL: bản Python 180/180, `SOS_HOOK_BIN=target/debug/sos` (đúng lệnh CI) 188/188, clippy sạch. Khác có chủ ý mới: đường dẫn Windows tới `.env` chỉ bản Rust chặn (ca chỉ chạy cho bản Rust).

Sau lượt 2, đóng nốt mục 8: `check()` tách stdout khỏi stderr và đòi toàn bộ stdout (khi là JSON) là đúng một object có `hookSpecificOutput`; JSON deny của Windows phải nằm trên stdout; thêm ca lệnh Bash không phải chuỗi; `SOS_HOOK_BIN=sos` (tên trần) được tìm trên PATH. Chú thích chữ số Unicode nói đúng là rộng hơn `isdigit` một chút. WSL: Python 180/180, Rust 189/189; Mac: Rust 189/189. Commit `eb13cfa`. Còn ghi nhận, chưa sửa: `run_limited` chỉ giết tiến trình con trực tiếp, chưa dọn cháu; chưa có ca riêng cho «cha thoát, cháu giữ stdout».

## Lượt 2 — kiến trúc + các sửa 1a

**Sửa 1a: MERGE** (soát tĩnh; không thay cho chạy native Windows). **Kiến trúc: SOUND-WITH-CHANGES**, 9 điểm chặn, Kiến trúc sư đã sửa trong `RUST_ARCHITECTURE_2026-11-05.md` (mục «Sửa theo soát»):

| # | Điểm | Hướng sửa trong bản kiến trúc |
|---|---|---|
| 1 | 1b bỏ «thiếu `sos` thì chặn» trên Codex Windows; binary cũ trả lỗi clap không thành deny | Giữ launcher `python -c` cho Codex Windows, gọi `sos hook codex … --protocol 1`, mọi lỗi → deny; chỉ bỏ sau khi thử thật 4 ca (mới/cũ/thiếu/không chạy) trong Codex Windows — điều kiện phát hành 1b |
| 2 | Vỏ POSIX `command -v sos && exec sos …` không bao lỗi của npm shim (tải binary thất bại, exit 1) | `hook` vào nhánh không tự tải của `bin/sos-npm`; launcher không `exec`, mọi mã ≠ 0 → exit 2; test bằng npm shim thật |
| 3 | «Luật khớp đầu tiên quyết định» cả payload: `git status; ssh win true` lọt nếu có luật allow `git status` | Xét từng lệnh đơn / từng path; `allow` có `scope`, không dừng duyệt |
| 4 | `check.run` + `sh -c` + `{path}` = chèn lệnh shell qua tên file | `program` + `args` có cấu trúc, thay biến từng đối số sau parse, cấm `{…}` trong `sh -c` |
| 5 | Bộ đọc của sim-guard bỏ wrapper nên `sudo cat` không còn bị chặn; heredoc đưa vào shell bị bỏ qua | Bộ đọc riêng cho luật: giữ wrapper, đọc lại heredoc của `sh <<EOF`, không hiểu → chặn |
| 6 | `realpath` chỉ khi file tồn tại: symlink cha trỏ ra ngoài vault + file mới lọt | Giải ancestor tồn tại dài nhất, so theo thành phần, hoa/thường tường minh, Codex giữ đủ Move to |
| 7 | JSON đúng nhưng sai kiểu / mất file luật → cho qua | Schema theo tool, `--file` bắt buộc khi đã cài, lỗi I/O là lỗi |
| 8 | Luật ví dụ làm mất các ca chặn của 5 script cũ (sudo, build của chan.py; `hub ocr`; marker cũ worker-guard; `.markdown`/multiline của hub-audit…) | Bảng bảo toàn hành vi từng script; mỗi ca chạy trên bản cũ trước khi dịch |
| 9 | Bỏ adapter ở N+1 dựa vào trạng thái fleet: người ngoài fleet nâng thẳng bị mất `hook.py` mà settings còn gọi | Kiểm tại repo đang `update`: còn tham chiếu → giữ, thoát ≠ 0 |

Nên sửa (10): giữ luật «không build trên Mac»; Q2 (`hub` tách đôi, Reel cần mục tiêu đo được) ghi là đề xuất chờ Sếp. Đồng ý: gộp 1c vào 1b, 2.1 trước 1d, crate `sos-rules`, `sos update --wiring` (opt-in, sửa theo cấu trúc JSON).

## Lượt 3 — `sos usage` (mục 7 bản kiến trúc)

**SOUND-WITH-CHANGES**, đối chiếu trường trên transcript thật (đã xác nhận `message.usage` 4 cột, `output_tokens_details.thinking_tokens`, `cache_creation.ephemeral_5m/1h`, `api-request.querySource` nối `requestRef`, `agentType` + `toolUseId` trong meta, khử trùng theo `message.id`). Năm điểm chặn: (1) Codex: cộng `last_token_usage` mọi dòng dư 78.217 token ở một rollout thật, và compaction (246.204 token) nằm ở `token_usage_record` ngoài `token_count`; (2) «kết quả subagent làm phình cha» đặt tên quá đại lượng đo được (chỉ đo ký tự văn bản trả về; có kết quả async); (3) ngưỡng 80% không xác định được token do fork; (4) `originator` không phải vai Codex, dùng `source.subagent.thread_spawn`; (5) `agent-usage.py` chỉ là chuẩn so cho phần giao nhau (nó gán vai khác, bỏ sidechain và `<synthetic>`). Nên sửa: chính sách trùng giữa file/resume; ngân sách bộ nhớ đo được (file lớn nhất ≈1 GB, dòng lớn nhất ≈4 MB); test riêng tư bằng chuỗi bí mật giả. Kiến trúc sư sửa mục 7 theo các điểm này.
