# Đổi toàn bộ code tool đi kèm agent sang Rust — kế hoạch

> **Sếp chốt 04/11/2026:** đổi hết phần code (Bash, Python, TS/JS) của tool đi kèm agent sang Rust, vì cần đồng bộ, dễ bảo trì, chạy nhanh. Làm dài hạn, có lỗi thì debug dần. Prompt Markdown (skill, subagent, lệnh slash, AGENTS.md) giữ nguyên.
> **Nguồn:** kiểm kê `~/VibeNotes/10_Projects/tong-dai/kiem-ke-tool-2026-11-04.md` (155 tool, đã dọn 31 mục ngày 04/11), các cổng định thêm trong `~/VibeNotes/10_Projects/tong-dai/harness-cong-cu.md`.
> **Người viết:** session con của Tổng đài, nhánh `claude/rust-all-tools-d1e85b`. Build và test trên máy Win (WSL), không build trên Mac 8 GB.

## Nguyên tắc chung

1. **Một binary chính.** Tool chung cho mọi dự án vào `sos` (lệnh con mới). Tool chỉ một repo dùng thì đổi ngay trong repo đó (nhánh riêng). Tool của Tổng đài: phần chung (máy luật, đếm session) vào `sos`, phần dính bảng vault vào crate riêng (xem Q1).
2. **Bản cũ là chuẩn so sánh.** Bản cũ có test thì cùng bộ test đó chạy cho cả hai bản, cùng đầu vào, so đầu ra (mã thoát, chữ chặn, JSON trả agent). Chưa có test thì viết test hành vi trên bản cũ trước, xanh trên bản cũ rồi mới đổi.
3. **Bản cũ để cạnh.** Chỉ thay chỗ gọi khi test so sánh xanh. Bản cũ không xoá: khi hết chỗ gọi thì chuyển vào `~/.local/share/tool-archive-2026-11-04/` và ghi `MANIFEST.md` như session dọn đã làm. File do kit phát (`harness-lite/`) thì bỏ ở bản phát hành sau, khi `sos check` không còn thấy app nào gọi.
4. **Chỗ gọi đổi theo bản phát hành.** App gọi hook qua `.claude/settings.json` và `.codex/hooks.json` (file của dự án, `sos update` không đụng). Lệnh mới chỉ có từ bản `sos` mới, nên thứ tự luôn là: code + test → phát hành (tag cần Sếp đồng ý) → cài `sos` mới trên Mac/Win → đổi chỗ gọi từng repo trên nhánh riêng.
5. **Không đổi thứ không phải của mình** (skill đồng bộ của Anthropic, skill hệ thống Codex) và **không đổi thứ nên bỏ**: bản trùng, bản chết thì chuyển vào lưu trữ thay vì viết lại.

## Bảng tool có code không phải Rust

Ký hiệu nơi đến: **sos** = lệnh con của `sos` · **repo** = đổi trong repo của nó · **riêng** = crate/dự án riêng · **giữ** = không đổi (lý do ở mục cuối) · **bỏ** = lưu trữ, không viết lại.

### Đợt 1 — hook và cổng chạy trước mỗi lệnh

| # | Tool | Ngôn ngữ, cỡ | Đi đâu | Chứng minh đúng |
|---|---|---|---|---|
| 1.1 | `adapters/claude/hook.py` (11 repo) | Python 112 | **sos**: `sos hook claude <session-start\|pre-edit\|pre-bash\|post-bash>` | `tests/adapters/run.sh` (≈150 ca, payload thật bắt 09/10) chạy hai lần: bản Python và `SOS_HOOK_BIN=sos` |
| 1.2 | `adapters/codex/hook.py` | Python 158 | **sos**: `sos hook codex <…>` | như trên, phần codex (patch V4A, Unicode, khối Windows trả JSON deny) |
| 1.3 | `scripts/env-guard.sh` | Bash 26 | **sos**: `sos hook env-guard <path>…` | ca `.env` trong run.sh |
| 1.4 | `scripts/sim-guard` | Python 282 | **sos**: `sos hook sim-guard --cmd …` (bite gọi lệnh này) | 60 ca sim-guard trong run.sh + test đơn vị Rust cho bộ đọc lệnh shell |
| 1.5 | `scripts/status.sh` | Bash 70 (+ Python nhúng) | **sos**: `sos hook status` (session-start dùng chung) | ca status trong run.sh |
| 1.6 | Git hook `templates/app/hooks/pre-commit`, `pre-push`; `hooks/pre-commit`, `pre-push` của kit | Bash 20–60 | Logic vào **sos** (`sos gate pre-commit`/`pre-push`: chạy gate rồi gọi `*.local`); file hook còn lại 2 dòng gọi `sos` (**giữ** phần vỏ) | `tests/smoke.sh` + ca mới: thiếu `.local`, `.local` không chạy được, `.local` đỏ |
| 1.7 | Hook agent ngoài kit: `chan.py` (vault), `win_heavy_guard.py` (BOOK), `worker-guard.sh` (soulsign-marketing) | Python 58/55, Bash 78 | **sos**: máy luật khai báo `sos hook rules --file <luật.toml>` (mỗi dòng: công cụ + mẫu lệnh/đường dẫn → cho/chặn/hỏi, kèm câu cách sửa; giống execpolicy của Codex, đúng đề xuất trong harness-cong-cu). Luật nằm ở repo của nó | Viết test hành vi trên bản cũ trước (payload → mã thoát + chữ), chạy lại trên `sos hook rules` |
| 1.8 | `block-unsafe-merge.sh` (tarot) | Bash 152 | **repo** tarot, dùng máy luật 1.7 cho phần khớp lệnh, phần gọi `gh` giữ trong luật kiểu «chạy lệnh kiểm» | test hành vi trước, có `gh` giả |
| 1.9 | `session-start-banner.sh` (soulsign-marketing), `agent_hook.py` (turnsigil) | Bash 77, Python 63 | **repo**: banner thành `[status] extra` trong `.sos.toml` nếu chỉ là in file; phần riêng turnsigil đổi trong repo | test hành vi trên bản cũ |
| 1.10 | `~/.claude/hooks/hub-audit.sh` (mọi repo) | Bash 25, gọi `hub audit` (Python) | Vỏ Bash vào máy luật 1.7 («sửa .md trong BOOK thì chạy `hub audit`»); `hub` đổi cùng đợt 5 | test hành vi trước |

Đợt 1 tách bốn phần, mỗi phần một PR:
- **1a** (session này): 1.1–1.5 trong `sos`. Riêng `post-bash` tạm gọi `scripts/test-watch.py` (đổi ở đợt 2 cùng `advise`), nên Python còn cần cho hook sau lệnh, không còn cần cho hook trước lệnh.
- **1b**: đổi mẫu `templates/app/claude-settings.json`, `codex-hooks.json` sang `sos hook …`; `sos check` nhận cả hai dạng và nhắc app còn gọi `hook.py`; bỏ bản viết lại lệnh Codex cho Windows trong `kit.rs` (lệnh `sos hook codex pre-edit` chạy y nguyên trong cmd, PowerShell, bash vì `sos` tự tìm gốc git). Đổi `templates/app/` nên cần một người soát độc lập (luật 4 của kit). `.claude/settings.json` của chính sos-kit chỉ đổi sau khi `sos` mới đã cài trên Mac: bản 0.3.13 không có `sos hook`, mà Claude Code coi mã thoát 2 (lỗi tham số của clap) là chặn, nên đổi sớm sẽ chặn mọi lần sửa file.
- **1c**: 1.6 (git hook).
- **1d**: 1.7–1.10, máy luật trước, rồi từng repo trên nhánh riêng (không đụng lucilius khi session Lucilius đang làm dở).

### Đợt 2 — script repo

| # | Tool | Ngôn ngữ, cỡ | Đi đâu | Chứng minh đúng |
|---|---|---|---|---|
| 2.1 | `scripts/test-watch.py` + `scripts/advise` | Python 357 + 272 | **sos**: `post-bash` tự xử lý, `sos advise` (gọi Claude/Codex CLI, `.advise-state/`) | phần post-bash và advise của run.sh (CLI giả), `tests/adapters/test_watch_units.py` đổi thành bảng ca dùng chung |
| 2.2 | `scripts/bite` | Python 560 | **sos**: `sos bite` | `tests/bite.sh`, `tests/bite_parsers.py` (mẫu đầu ra Node/Rust/pytest/xcodebuild) |
| 2.3 | `scripts/trust-gate.sh`, `scripts/install-hooks.sh`, `tests/docs-check.py` (cổng pre-commit của kit) | Bash 332 + 68, Python | **riêng**: crate `crates/sos-dev` trong workspace (không phát cho app) | trust-gate: so baseline thật + ca unicode ẩn; docs-check: chạy cả hai bản trên cùng cây, so danh sách lỗi |
| 2.4 | tally `agent-usage.py` | Python 99 | **sos**: `sos usage` (đếm token theo vai từ transcript) nếu Sếp còn dùng; không thì **bỏ** | transcript mẫu → bảng số giống hệt |
| 2.5 | `wait-settled.sh`, `sim-boot.sh` (tally, turnsigil: hai bản y nhau), lucilius `resolve-simulator-destination.sh` | Bash 38/18/41 | **sos**: `sos sim boot\|settle\|destination`, gộp với sim-guard (sau này thành khoá simulator của hồ sơ `ios-app` trong BACKLOG) | test hành vi với `xcrun simctl` giả |
| 2.6 | turnsigil `agent_role.py` | Python 151 | **bỏ**: trùng `sos role`; đổi chỗ gọi sang `sos role` | so lệnh sinh ra bằng `sos role --dry-run` |
| 2.7 | tarot `security-gate.sh`, `check-hardcoded-secrets.py`, `refresh-docs-stats.sh` | Bash 217/73, Python 192 | **repo** tarot. Phần tìm khoá trùng `sos gate secrets` (gitleaks) → bỏ phần trùng; INV còn lại thành luật khai báo nếu máy luật 1.7 đủ, không thì Rust trong tarot | chạy INV-001→010 cả hai bản trên cùng cây và trên các commit cũ từng bị bắt |
| 2.8 | lucilius `quality-gate.sh`, `pre-commit.local`, `pre-push.local`; tarot, turnsigil `pre-commit.local` | Bash | **giữ** (đề nghị; Q3) | — |
| 2.9 | `~/.config/don-rac/don-rac.py` (launchd 15 phút, dọn tiến trình Mac + WSL) | Python 169 | **riêng**: cùng crate Tổng đài (đợt 4) | test hành vi với `ps` giả: danh sách giết phải trùng khớp từng PID |

### Đợt 3 — script kèm skill

| # | Tool | Ngôn ngữ, cỡ | Đi đâu | Chứng minh đúng |
|---|---|---|---|---|
| 3.1 | `skills/depth-check/scripts/predictions.py` | Python 219 | **sos**: `sos predict` (skill gọi `sos`) | viết test hành vi trước (ghi, chấm, file jsonl) |
| 3.2 | `product-description/references/check-links.py` | Python 102 | **sos**: `sos check-links <thư mục>` (kiểm link Markdown, dùng được cho mọi repo) | thư mục mẫu có link hỏng/đúng → cùng danh sách |
| 3.3 | thanhtra `thanhtra-pre-scan.py` + CLI `thanhtra` | Python 44 + ~1.500 | **repo** thanhtra (dự án công khai riêng, có test) | bộ test của thanhtra chạy cả hai bản |
| 3.4 | Plugin Codex `codex-claude-bridge` (`bridge.py`, `ghostty_bridge.py`) | Python | **bỏ** (nghi bỏ từ 07/2026, đã có start_session/SendMessage) | — |
| 3.5 | Skill đồng bộ của Anthropic (docx, pdf, pptx, xlsx, skill-creator, web-artifacts-builder, google-workspace) và skill hệ thống Codex | Python/JS/Bash | **giữ** | — |

### Đợt 4 — Tổng đài

| # | Tool | Đi đâu | Chứng minh đúng |
|---|---|---|---|
| 4.1 | `chan.py` | Luật trong `~/VibeNotes/.../chan.toml`, chạy bằng `sos hook rules` (1.7) | test hành vi viết trên `chan.py` trước |
| 4.2 | `kiem.py` (lỗi bảng, dự án cần nhắc, mốc) | crate Tổng đài (Q1) | bảng thật hôm nay + bảng mẫu có lỗi → cùng đầu ra |
| 4.3 | Canh session đứng im (đọc `claude-code-sessions/…/local_*.json`, so cột «Đầu mối») | `sos sessions idle` (phần đọc file session là chung) + cột bảng ở crate Tổng đài | thư mục session mẫu với `lastActivityAt` giả |
| 4.4 | Tối đa 3 session con (hook PreToolUse cho `start_session`) | `sos sessions count --max 3` + một dòng luật | test hành vi: 2 → cho, 3 → chặn với câu «xếp hàng chờ» |
| 4.5 | Sổ giao ca kèm id + đường dẫn; chặn `clear_session` khi bảng lỗi | crate Tổng đài | dòng sổ thiếu `local_…` → lỗi |
| 4.6 | Tin cho Sếp ≤100 chữ (hook Stop) | `sos hook word-limit --max 100` | thử vài ngày trước khi bật chặn (harness-cong-cu ghi «dễ phiền») |
| 4.7 | `brief.py` (sinh brief đủ câu bắt buộc) | crate Tổng đài | so brief sinh ra với mẫu |

### Đợt 5 — dự án riêng, làm sau cùng

| # | Tool | Cỡ | Đề nghị |
|---|---|---|---|
| 5.1 | Server Reel `~/code/reel/server` | Python ≈4.300 dòng, 19 module, 407 test, đang chạy thật trên VPS | Tách dự án riêng, giao Quản đốc Reel. 407 test Python làm chuẩn: chuyển thành test mức HTTP/CLI (đầu vào → JSON, thẻ ra) chạy được với cả hai bản, chạy song song bản Rust cạnh bản Python trên VPS một thời gian, so đầu ra, rồi mới đổi. Xem Q2 |
| 5.2 | `hub` (BOOK `_hub`, Bash bọc 40 file Python, ≈9.000 dòng) | lớn | Như Reel: dự án riêng, giao Quản đốc BOOK, sau cùng. Xem Q2 |
| 5.3 | Plugin `mac` (`~/claude-mods/mac/hooks/register.tsx`) | TS 443 | **giữ** vỏ TS (API mod của Claude chỉ nhận module TS/JS); phần tính toán (RAM, agent im, chặn sim thứ hai) có thể gọi `sos` khi các lệnh tương ứng có |

## Không nên đổi (để Sếp quyết)

| Thứ | Lý do |
|---|---|
| Vỏ git hook (`pre-commit`, `pre-push`, 2–3 dòng) | Git cần một file chạy được ở `core.hooksPath`; vỏ chỉ gọi `sos`. Logic đã chuyển vào `sos` (1.6) |
| `install.sh`, `bin/sos-npm`, `scripts/npm-postinstall.sh` | Chạy *trước khi* có binary `sos` (tải binary về), nên không thể là Rust |
| Bộ test của kit (`tests/*.sh`, `tests/*.py`) | Chúng là chuẩn so sánh trong lúc đổi; viết lại cùng lúc thì mất cái để so. Đổi sau cùng nếu còn muốn, trừ `docs-check.py` là cổng nên đi cùng 2.3 |
| `pre-commit.local`/`pre-push.local` và `quality-gate.sh` của từng app (2.8) | Chỉ là keo gọi `xcodebuild`, `swift`, `npm` của chính app; bản Rust vẫn phải gọi y các lệnh đó, không nhanh hơn, khó sửa hơn cho Thợ của app |
| Skill đồng bộ của Anthropic, skill hệ thống Codex (3.5) | Không phải của mình; lần đồng bộ sau ghi đè |
| Plugin `mac` (5.3) | API mod chỉ nhận TS/JS |
| Server Reel, `hub` (5.1, 5.2) | Đề nghị cân nhắc: hai sản phẩm đang chạy, nút thắt là mạng và gọi model chứ không phải tốc độ Python; đổi tốn nhiều công, rủi ro hồi quy cao. Nếu đổi thì dự án riêng như trên |

## Sau kiến trúc và soát 05/11 (Sếp chốt 05/11: «Theo Fable»)

Kiến trúc: `RUST_ARCHITECTURE_2026-11-05.md` (Kiến trúc sư Fable 5.1). Soát: `RUST_REVIEW_2026-11-05.md` (Codex gpt-6-astra): sửa 1a MERGE (đã sửa, `eb13cfa`), kiến trúc và `sos usage` SOUND-WITH-CHANGES, Kiến trúc sư đã sửa theo cả 9 + 7 điểm (bản sửa chưa qua lượt soát thứ hai; lượt soát của 1b sẽ đi qua nó).

Thứ tự đề nghị: 1a (xong) → **1b+1c** một PR, một bản phát hành (launcher fail-closed; Codex Windows giữ `python -c` gọi `sos hook … --protocol 1` cho tới khi thử thật 4 ca trong Codex Windows; `sos update --wiring` opt-in) → **2.4 `sos usage`** (Sếp duyệt 05/11, mục 7 kiến trúc) → **2.1** test-watch + advise (hook hết Python) → **1d** máy luật trong crate mới `sos-rules` → 2.2 → 2.3 → 2.5 → đợt 3 → đợt 4 → đợt 5.

Khác kế hoạch ở trên: gộp 1c vào 1b; 2.1 lên trước 1d; 2.4 lên ngay sau 1b; thêm crate `sos-rules`; thêm `sos update --wiring`. **Sếp chốt 05/11 (qua Tổng đài): thứ tự trên; Q1 A (crate riêng tư `~/code/tong-dai`); Q3 A (keo `pre-commit.local` giữ Bash); Q2 A kèm hai đề xuất của Kiến trúc sư: `hub` tách đôi (audit/verify/search sang Rust, OCR/convert giữ Python qua máy Win), Reel chỉ bắt đầu khi có mục tiêu đo được, ghi vào BACKLOG của Reel.**

## Câu hỏi cho Sếp

- **Q1.** Code dính vault (đọc bảng `10_Projects/README.md`, sổ giao ca) để ở đâu? sos-kit đang **công khai** trên GitHub. Em đề nghị: phần chung (máy luật, đếm session, giới hạn chữ) vào `sos`; phần dính vault vào crate Rust riêng tư `~/code/tong-dai` (không push lên repo công khai). Lựa chọn: A crate riêng tư / B crate trong workspace sos-kit (công khai).
- **Q2.** Reel server và `hub`: A đổi sau cùng thành dự án riêng (như brief) / B giữ Python, chỉ đổi hook và cổng quanh chúng.
- **Q3.** Keo `pre-commit.local` của app: A giữ Bash / B đổi luôn.

## Cách làm từng tool (áp cho mọi đợt)

1. Tìm test của bản cũ. Không có: viết test hành vi (đầu vào thật → mã thoát, stdout, stderr), chạy xanh trên bản cũ, commit riêng.
2. Viết bản Rust, thêm test đơn vị cho phần khó (bộ đọc lệnh shell, phân tích patch).
3. Chạy cùng bộ test cho cả hai bản trên máy Win qua `may-win chay` (Linux/WSL); CI của PR chạy thêm macOS và Windows.
4. Xanh cả hai thì đổi chỗ gọi (theo nguyên tắc 4), bản cũ để cạnh đến bản phát hành sau, rồi chuyển vào lưu trữ.
5. Ghi CHANGELOG, BACKLOG; một dòng «Nhật ký bảng» khi xong đợt.

## Tiến độ

- 04–05/11: kế hoạch này.
- 05/11: **đợt 1a xong** (`crates/sos-cli/src/{hook,sim_guard,status}.rs`). Trên WSL: `tests/adapters/run.sh` bản Python 171/171, `SOS_HOOK_BIN=…/sos` 178/178 (thêm 3 ca so từng byte chữ status và 1 ca cắt 220 ký tự), `tests/bite.sh` 26/26, test đơn vị Rust xanh; một test `kit.rs` đỏ vì WSL thiếu gitleaks (không liên quan). Khác có chủ ý: `.env` guard tách cả `\` của đường dẫn Windows; status cắt dòng theo ký tự (GNU `cut` cắt byte, làm vỡ chữ Việt). Chưa đổi chỗ gọi: phải chờ bản phát hành có `sos hook` (bản 0.3.13 sẽ báo lỗi tham số, mã 2, Claude Code hiểu là chặn mọi lệnh sửa). Trên Mac (bản debug do pre-commit build): bản Rust 178/178. Windows native chưa chạy: CI của PR chạy (đã thêm bước `SOS_HOOK_BIN=target/debug/sos tests/adapters/run.sh`).
- 05/11: **đợt 1b+1c xong** (chưa push). Mẫu `templates/app/` gọi `sos hook <agent> <event> --protocol 1`; pre-edit/pre-bash qua launcher biến mọi lỗi chạy `sos` thành chặn, `sos hook` tự chặn sau 30 giây; Codex Windows giữ launcher `python -c` gọi `sos`; `sos update --wiring` (sửa JSON theo cấu trúc, giữ thứ tự khoá); `sos update` không xoá file kit bị bỏ khi wiring còn gọi; `sos gate pre-commit|pre-push` + vỏ hook ngắn. Soát Astra lượt 4: 9 điểm, đã sửa (`RUST_REVIEW_2026-11-05.md`). Test: WSL `cargo test --locked` xanh trừ 1 ca cần gitleaks, adapters Python 180/180, Rust 189/189, bite 26/26, docs-check, npm, clippy; **Windows native** `cargo test -p sos-cli` xanh (kit 20/20, gồm launcher Codex qua cmd và PowerShell). Còn mở: 4 ca trong Codex Windows thật (điều kiện phát hành); `run_limited` chưa dọn tiến trình cháu.
