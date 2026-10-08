# Khảo sát harness — SOS Kit, Harness Lite và thế giới bên ngoài (2026-10-08)

> Đầu vào: 5 repo app của anh (Lucilius, Thirty, Payquill, Nếp nhà + tarot qua memory), phiên Codex 05–07/10 nơi anh thiết kế Harness Lite, ghi chú Obsidian "Triết lý làm việc với AI", prompt-audit sos-kit hôm nay, tài liệu chính thức Anthropic/OpenAI và các harness cộng đồng.
> Phân loại độ tin: **[đo]** = số lấy từ repo/git; **[docs]** = tài liệu chính thức đã mở đọc; **[2nd]** = nguồn thứ cấp chưa đọc bản gốc.

---

## 0. Kết luận ngắn

1. **Cảm nhận của anh đúng và có bằng chứng.** SOS Kit nặng, đắt, và vẫn có lỗ hổng. Lỗ hổng không nằm ở "thiếu luật" mà ở **chỗ nối giữa các mảnh** và ở **chữ đã chết** mà không ai phát hiện, vì có quá nhiều chữ.
2. **Hướng Lite đúng với thời đại.** Anthropic, OpenAI và nghiên cứu độc lập năm 2025–26 cùng hội tụ về một kiểu: một agent viết, chỉ dẫn ngắn dạng bản đồ, luật cứng giao cho máy, kiểm bằng thứ chạy được, người giữ hai đầu.
3. **Nhưng Lite hiện thiếu đúng thứ anh cảm thấy thiếu: vòng phản hồi có lực.** Bằng chứng ở Payquill: 52/52 lát `passes: true`, rồi một lượt "phá hoại" độc lập vẫn tìm ra crash, restore không qua validator và lưu cả sổ trên main thread. Lỗi được bắt bởi **một mắt khác, ngữ cảnh mới, có nhiệm vụ phá**, chứ không phải bởi quy trình từng lát.
4. **Đề xuất:** đừng thêm lại nghi lễ của SOS. Hãy đặt **hai điểm phản hồi cứng** (đầu: thợ đối chiếu brief với thực tế; cuối: một lượt phá hoại khác model trước khi giao anh), **ít cổng máy nhưng fail-closed**, và **báo cáo buộc dẫn bằng chứng**. Các phần còn lại để model tự lo.

---

## 1. Anh đã làm gì — dòng thời gian thực tế

| Dự án | Thời gian | Harness | Kết quả đo được |
|---|---|---|---|
| **Tarot / SoulSign** | 04–07/2026 | SOS Kit đầy đủ: phiếu, debate 3 vòng, approval gate, ~11 tool Rust, 8 pre-commit gate | Ship được; Apple reject 4.3(b). Memory ghi một lần sửa privacy media tốn **~160k token / 21 phút cho ~3 dòng code** |
| **Lucilius** | 01–11/09 | 01/09: commit "drop ticket system, keep 3 hard rules". 02/09: đảo lại, "establish SOS workflow" ("liều gọn") | **[đo]** 35 phiếu / ~10 ngày. `DISCOVERIES.md` 1.410 dòng, `CHANGELOG.md` 1.151 dòng; copy cả `LAYERS`/`ORCHESTRATION`/`WORKFLOW_V2.2` vào repo app. TestFlight + sandbox; Apple hỏi thông tin 2.1; còn bug "Today kept-state". Ngừng commit từ 11/09 |
| **Thirty** | 02–18/09 | **Lite v0.2**: AGENTS.md 35 dòng; 3 vai + Người soát; `FEATURES.json` có `passes` (features-guard chặn lật lại); BLUEPRINT; PROGRESS; Stop/SubagentStop chạy `make check`; docs-gate | **[đo]** 31 lát, 29 pass, ~16 ngày, TestFlight build 4. Vai đổi model 3 lần (fable → opus → fable) |
| **Payquill** (`~/code/tally`) | 22/09–01/10 | Lite v0.2 từ mẫu Thirty + **Codex chạy song song** + Hub (oracle fixtures, test pack) | **[đo]** 53/53 lát, ~10 ngày tới App Review. Nhưng: `BLUEPRINT.md` 433 dòng (luật ghi ≤150), `QA.md` 1.005 dòng, `PROGRESS.md` 596 dòng |
| **Nếp nhà** | 07/10 (một ngày) | **Lite mới** (bản trong kit, vendor kèm `UPSTREAM.json` hash), worker Sonnet 5.5 medium, architect Fable 5.1 high | **[đo]** 37 commit trong ~9 giờ; luồng A (cảnh báo iPad → iPhone) PASS hẹp trên máy thật; luồng B chưa đạt. Tạm hoãn 08/10 |

**Về bản "update Lite cho Nếp nhà" anh tưởng đã mất:** không mất gì.
- Phiên dọn ổ lúc 14:33 hôm nay đã push đủ 10 commit còn treo trên `main` và nhánh `wip/b-report-probe`, rồi mới xoá. `evidence/` (4.122 file, khớp checksum) và `Local.xcconfig` đã chuyển sang Windows `/home/sep/archive/nep-nha-ios`.
- Bản Lite trong Nếp nhà **khớp hash từng file** với `~/sos-kit/harness-lite/` hiện tại (chưa commit). Nghĩa là bản update vẫn nằm nguyên trong kit.
- Phần riêng của Nếp nhà (AGENTS.md, STATE, 5 bài học trong memory) đều có trên GitHub và trong `~/.claude/projects/...nep-nha-ios/memory`.

---

## 2. Bằng chứng: harness nào bắt được lỗi gì, và tốn gì

### 2.1 Lỗi thật được bắt bởi ai

| Lỗi | Dự án | Ai bắt | Quy trình từng lát có bắt không |
|---|---|---|---|
| File backup dựng cố ý làm tràn số ở 4 điểm, U15 crash mỗi lần mở sau restore | Payquill | Lượt "RRI destroyer" (Quản đốc + 2 Người soát, trần ~120 phút) | **Không**. Lúc đó 52/52 lát đã pass |
| Restore không chạy Validator → màn trống không lý do | Payquill | Cùng lượt trên | Không |
| Lưu toàn sổ mỗi phím gõ trên main thread (1.826 ca ~1,1 s ở Debug) | Payquill | Cùng lượt trên | Không |
| CSV injection, ngắt số ở AX5 | Payquill | Destroyer + reviewer GPT-5.6 Sol (khác họ model) | Không |
| Today/ChildHome không dừng khi mất quyền; bỏ ghép báo "đã dừng" trước khi iPad xác nhận | Nếp nhà | Reviewer ngữ cảnh mới | Không (lát báo xanh) |
| Lát song song xanh riêng lẻ nhưng bản ghép fail | Nếp nhà | Quản đốc khi build bản ghép | Không |
| CK12/CK22, `includesPastActivity` tính từ 00:00 | Nếp nhà | Chạy trên máy thật | Simulator không thể bắt |
| `/security-review` bỏ sót INV-LOCAL; `/advisory-scan` không khớp marker với binary; subagent bị bắt dùng `AskUserQuestion` (tool runtime không cấp) | sos-kit | Prompt-audit hôm nay | Chạy nhiều tháng không ai thấy |

**Đọc bảng này ra 3 điều:**
1. Lỗi đắt (crash, mất/sai dữ liệu, trạng thái nói dối) **không được bắt ở cấp lát**. Chúng được bắt bởi (a) một lượt có **nhiệm vụ phá**, (b) **ngữ cảnh mới**, thường là **model khác**, (c) **môi trường thật** (máy thật, bản ghép).
2. Đúng như anh nói hôm 05/10: *"dùng codex làm, xong dùng claude crosscheck… nó cũng bắt được lỗi khác"*.
3. SOS Kit có rất nhiều cổng, nhưng cổng nào cũng kiểm **từng mảnh**. Không cổng nào kiểm các mảnh có khớp nhau không, nên 3 bug ở sos-kit nằm đúng chỗ nối.

### 2.2 Chi phí

- **[đo/memory]** SOS: ~42k token cho một phiếu nhiều vòng; ~160k token / 21 phút cho một fix ~3 dòng.
- **[docs]** Anthropic, *Harness design for long-running apps* (24/03/2026): bản chạy đơn 20 phút / $9; bản harness đầy đủ (planner, generator, evaluator) 6 giờ / $200, tức hơn 20 lần.
- **[docs]** Anthropic, *multi-agent research system* (06/2025): agent đơn tốn khoảng 4 lần token so với chat; đa agent khoảng **15 lần**. Bài viết nói **coding là việc kém hợp với đa agent**.
- **[2nd]** ETH Zurich, *Evaluating AGENTS.md* (ICLR 2026): file context làm chi phí suy luận **tăng trên 20%** mà không tăng, thậm chí giảm, tỉ lệ thành công. Lý do: agent làm theo các yêu cầu thừa.
- **Chưa đo:** token thực tế của Lite trên Thirty, Payquill và Nếp nhà. Đây là lỗ hổng dữ liệu lớn nhất của báo cáo này (xem mục 6).

---

## 3. Thế giới bên ngoài nói gì

### 3.1 Anthropic (chính chủ)

| Nguồn | Điểm then chốt |
|---|---|
| Docs Claude Code: *best practices*, *memory*, *features overview* | CLAUDE.md **dưới 200 dòng**; chỉ ghi điều Claude không tự suy ra được. "Over-specified CLAUDE.md" là một failure pattern có tên. IMPORTANT ở khắp nơi thì không dòng nào nổi bật. **Luật cứng đưa vào hook**, vì prose "là lời nhờ, không phải bảo đảm". Chỉ thêm một dòng CLAUDE.md khi Claude sai cùng chỗ **hai lần**. |
| Docs *subagents* | Các phase chia nhau nhiều ngữ cảnh (plan → code → test) nên **ở chung một ngữ cảnh**. Subagent dùng để cô lập output dài, giới hạn tool, hoặc review độc lập. **Subagent không bao giờ có `AskUserQuestion`**; subagent chạy nền còn mất cả task tools. |
| *Building effective agents* (12/2024) | Chọn giải pháp đơn giản nhất. Chỉ thêm phức tạp khi nó **chứng minh** được là cải thiện kết quả. |
| *Effective harnesses for long-running agents* (11/2025) | Danh sách feature dạng **JSON**, `passes` ban đầu đều false, cấm xoá test; progress log; một feature mỗi phiên; test end-to-end như người dùng thật. Đây chính là khung Thirty đã dùng. |
| *Harness design for long-running apps* (03/2026) | "Every component in a harness encodes an assumption about what the model can't do." Model mạnh lên thì **gỡ giàn giáo**: bỏ chia sprint (từ Opus 4.6), bỏ reset ngữ cảnh (từ Opus 4.5), gộp evaluator từng sprint thành **một lượt QA cuối**. Agent luôn chấm bài của chính mình quá cao; **một evaluator riêng, hoài nghi** dễ tinh chỉnh hơn nhiều. |
| claude.dev, *New rules of context engineering for Claude 5* (Thariq Shihipar, 24/07/2026) | Anthropic **gỡ hơn 80% system prompt của Claude Code** cho Opus 5 / Fable 5 mà eval coding không giảm. Rủi ro chính bây giờ là **bó buộc quá mức**. Cho phán đoán thay cho luật; tải dần theo cây file. |
| Platform docs: prompting Opus 5 / Fable 5 / Opus 5.5 | Opus 5 tự kiểm bài; lệnh kiểu "dùng subagent để verify" gây **verify quá đà**, và "the same applies to legacy harness scaffolding". Fable 5: skill viết cho model cũ thường quá cứng và làm giảm chất lượng. Chỉ dừng hỏi người khi có việc phá huỷ/không đảo ngược, đổi scope thật, hoặc cần thông tin chỉ người có. **Buộc mỗi claim tiến độ đối chiếu với kết quả tool** thì gần như hết báo cáo bịa. Opus 5.5: effort mặc định medium. |
| claude.dev, *Spending your effort* (09/2026) | Công sức của người dồn vào **spec** (để Claude phỏng vấn mình). Code ở effort thấp; **kiểm và test ở effort cao**. |

### 3.2 OpenAI / Codex

- **AGENTS.md:** đọc dồn từ git root xuống thư mục hiện tại; trần mặc định **32 KiB**, vượt thì cắt im lặng. *Harness engineering* (02/2026) **[2nd]**: file AGENTS.md khổng lồ ban đầu thất bại; cắt còn khoảng **100 dòng, "mục lục chứ không phải bách khoa"**, trỏ vào `docs/`.
- **Luật kiến trúc giao cho linter và structural test**, với **thông báo lỗi viết thành hướng dẫn sửa cho agent**. Có agent định kỳ quét drift của docs. Agent review PR của nhau. Người giữ môi trường, ý định và vòng phản hồi, **không duyệt từng task**.
- **Subagent:** dùng cho việc đọc nhiều và độc lập. Cảnh báo việc **ghi song song** (xung đột, chi phí phối hợp) và tốn token hơn chạy đơn.
- **Cổng người** đặt ở **biên năng lực** (sandbox `workspace-write` + `on-request`), không đặt trước mỗi task.

### 3.3 Harness cộng đồng (số sao kiểm qua GitHub API hôm nay)

| Harness | Sao | Nặng | Ý đáng học | Rủi ro |
|---|---|---|---|---|
| obra/superpowers | 297k | Vừa (skill tải khi cần) | Mỗi task một subagent ngữ cảnh mới + TDD + review hai tầng | Luồng cứng; tốn token subagent |
| garrytan/gstack | 136k | Vừa–nặng | `/qa` chạy trình duyệt thật; **chỉ đưa "quyết định gu" cho người duyệt** | Nhiều lệnh phải nhớ; năng suất tự báo |
| github/spec-kit | 141k | Nặng tài liệu | "Constitution" = các bất biến ổn định | "Biển markdown", khó review, spec trôi |
| BMAD-METHOD | 54k | Nặng: ~6 persona | Story tự đủ cho dev agent | **Gần SOS nhất**: nghi lễ persona, cạn token, không có benchmark |
| HumanLayer ACE / 12-factor | 12–27k | Vừa | Nén có chủ đích, mỗi phase một ngữ cảnh mới | Chính tác giả tự sửa: plan 1.000 dòng không review nổi, **phải review code** |
| Ralph loop | 22k | Tối thiểu | Vòng lặp ngữ cảnh mới + state trong file + test làm điều kiện dừng | Kém ở việc cần phán đoán; dễ đốt token |

**Bằng chứng tổng hợp:**
- Cognition (*Don't build multi-agents*): các agent ngữ cảnh rời ra quyết định ngầm mâu thuẫn nhau.
- Nghiên cứu multi-agent debate (arXiv 2311.17371, 2502.08788): tốn nhiều token hơn, lợi ích nhỏ hoặc không có so với một agent đơn mạnh. **Điều này nói thẳng về vòng debate 3 lượt Architect ↔ Worker của SOS.**

---

## 4. Đối chiếu: SOS Kit và Lite trước chuẩn hiện tại

| Nguyên tắc (nguồn) | SOS Kit | Lite v0.2 (Thirty/Payquill) | Lite mới (kit/Nếp nhà) |
|---|---|---|---|
| Chỉ dẫn ngắn dạng bản đồ, <200 dòng (Anthropic, OpenAI, ETH) | ✘ CLAUDE.md 288 dòng + handbook 200–300 dòng mỗi vai; Lucilius copy cả doctrine vào app | ✔ AGENTS.md 35–48 dòng, vai 14–33 dòng | ✔ contract 45 dòng + vai 7–15 dòng |
| Không bó buộc quá mức, cho phán đoán (Claude 5 rules) | ✘ luật kèm lịch sử sự cố, IG-xx, sensor table | ~ còn "≤10 dòng", "≤150 dòng", luật code chi tiết | ✔ |
| Luật cứng → hook/linter, lỗi viết thành hướng dẫn sửa (Anthropic, OpenAI) | ✔ nhiều gate fail-closed, nhưng **quá nhiều**; prose và hook chồng nhau | ✔ quality-gate, features-guard, block-env, runtime-scan | ~ **chỉ là guidance** (README tự nói); bỏ Stop hook |
| Một agent viết; subagent cho đọc nhiều / review (Anthropic, Codex, Cognition) | ✘ Architect cấm đọc code + debate 3 vòng | ✔ một lát một Thợ | ✔ |
| Người giữ hai đầu, gate theo rủi ro (Fable/Opus docs, Codex) | ✘ approval gate trước **mọi** EXECUTE | ~ duyệt BLUEPRINT + cuối phase | ✔ ý định + nghiệm thu |
| State trong file/git, feature JSON (long-running harness) | ~ phiếu + BACKLOG + DISCOVERIES (phình) | ✔ FEATURES.json + PROGRESS | ✔ một STATE |
| **Evaluator riêng, hoài nghi; tốt nhất khác model** (harness design; kinh nghiệm của anh) | ~ Worker CHALLENGE (cùng model, trước code) | ~ Người soát cuối phase (cùng họ model) | ~ reviewer "khi cần", **không có điểm bắt buộc** |
| Kiểm end-to-end trên bản ghép + môi trường thật | ✘ kiểm từng phiếu | ✘ E2E mỗi lát → tốn, nhưng vẫn sót | ✔ theo hành trình, trên bản ghép (README) |
| Báo cáo đối chiếu bằng chứng (Fable docs) | ~ Discovery Report | ~ ✔/✘ từng dòng verify | ~ có trong contract, nhưng Nếp nhà vẫn báo nhầm "bản dùng thử" |
| Gỡ giàn giáo khi model lên đời (harness design) | ✘ chỉ thêm, chưa từng gỡ | — | ✔ "retire a costly rule only after checking what protection is lost" |

---

## 5. Chẩn đoán — vì sao "nhanh nhưng chưa đúng"

Kết hợp nhận xét của anh (05/10 và hôm nay) với dữ liệu:

1. **SOS đặt phản hồi sai chỗ.** Tranh luận trên giấy *trước* khi có code (Architect không được đọc code, Worker challenge, tối đa 3 vòng) thì tốn token mà bắt được ít. Lỗi đắt chỉ lộ ra khi có bản chạy, có dữ liệu xấu, có máy thật. Đúng như anh nói: "quy trình vẫn vỡ vì ko bắt được lỗi ngay trong quy trình".
2. **Lite bỏ nghi lễ nhưng bỏ luôn lực của vòng phản hồi.** Reviewer chỉ chạy "khi cần", cùng họ model, và không có nhiệm vụ phá. Payquill cho thấy lượt có lực thật là lượt **destroyer** chạy *sau* khi mọi lát đã xanh. Thứ anh thấy thiếu không phải là "challenge trước code" mà là **"phá sau khi ghép"**.
3. **Nhiều chữ thì chữ chết.** Ở sos-kit, prompt bắt subagent dùng tool mà runtime không cấp, nhiều tháng không ai biết. Ở app, BLUEPRINT vượt trần 3 lần, QA.md hơn 1.000 dòng, DISCOVERIES 1.400 dòng. Tất cả là thuế token mỗi phiên và là chỗ chỉ dẫn mâu thuẫn trú ngụ.
4. **Báo cáo là một lỗ hổng riêng.** Nếp nhà: Quản đốc gọi bản simulator có dữ liệu minh hoạ là "bản dùng thử", anh tưởng đã xong và cài lên máy. Anh còn phải tự kiểm thủ công từng bước. Model mới có cách sửa đã được ghi trong docs: mỗi claim phải trỏ tới kết quả tool, và có một dòng trạng thái tổng.
5. **Môi trường là ràng buộc của harness, không phải chi tiết.** Máy 8 GB RAM, một simulator, máy thật bắt buộc cho DeviceActivity/CloudKit, worktree tự động của runtime hỏng. Harness phải mã hoá chuyện này (lease simulator, build song song nhưng chạy sim tuần tự, luôn build bản ghép). Lite đã có phần lớn.
6. **Model là một phần, nhưng không phải phần chính.** Anh tin "do model một phần" và đúng một phần. Docs nói rõ: model mới tự plan, tự verify, nên giàn giáo cũ còn gây hại. Nhưng lỗi Payquill xảy ra với cả Fable lẫn Opus; thứ bắt được nó là **cấu trúc phản hồi**, không phải model mạnh hơn.

---

## 6. Đề xuất cho Lite (để bàn, chưa làm)

Nguyên tắc của anh: *"vừa nhanh vừa đúng thì phải đặt đúng mọi thứ vào đúng chỗ"*. Cụ thể:

### 6.1 Phản hồi đặt đúng chỗ (sửa theo ý anh 08/10: lượt phá là gợi ý lúc nghiệm thu, không bắt buộc)
- **Đầu lát, đã có:** Thợ đối chiếu brief với code thật trước khi xây. Giữ nguyên, không tách thành phase riêng.
- **Trong lát:** advisor (model mạnh hơn) tư vấn ở các điểm quyết định, thay cho review mỗi lát. Xem mục 8.
- **Ở mốc hành trình hoặc vùng rủi ro:** người soát nhìn từ ngoài vào. Xem mục 8.
- **Lúc nghiệm thu, gợi ý không bắt buộc:** lượt phá có giới hạn thời gian, nên dùng khác họ model. Payquill cho thấy lượt này đáng tiền với app có dữ liệu hoặc tiền.
- **Bỏ:** reviewer giữa chừng theo nhịp cố định; debate nhiều vòng; approval từng lát.

### 6.2 Ít cổng máy nhưng fail-closed, lỗi viết thành cách sửa
- **Giữ:** secrets/env, `features-guard` (cho phép lật về false có ghi lý do), quality-gate chữ hiển thị cho người dùng, chặn code trên default branch.
- **Thêm một lệnh `make ready`**, tức "sẵn sàng giao": build bản ghép + test + kiểm tra một danh sách bằng chứng bắt buộc. Quản đốc **không được nói "xong"** khi lệnh này đỏ. Đây là chỗ thay Stop hook full-suite: chạy một lần ở ranh giới, không chạy mỗi lần dừng.
- **Bỏ:** docs-gate CHANGELOG mỗi commit, lane budget, AGENT_MAP, sensor table, trust-gate cho repo app.

### 6.3 Báo cáo buộc bằng chứng
- Mỗi báo cáo của Quản đốc kết thúc bằng: **"Cài máy thật được: CÓ/CHƯA"**, rồi các bảng *đã chạy thật / chỉ simulator / chưa kiểm*, mỗi dòng trỏ tới lệnh hoặc ảnh. (Đây là bài học Nếp nhà, nâng từ memory dự án lên contract.)
- Quản đốc tự chạy kiểm trên máy thật (XCUITest + kéo log); chỉ nhờ anh việc máy không làm được.

### 6.4 Giữ tài liệu nhỏ bằng máy, không bằng lời
- Đặt trần dòng cho `STATE`/`PROGRESS`/`QA`/`BLUEPRINT` và chặn bằng `doc-rotate` (tool anh đã có). Bằng chứng chi tiết để trong `evidence/`, STATE chỉ trỏ tới.

### 6.5 Model và effort theo vai, có bằng chứng
- Mặc định: builder chạy model đang cấu hình ở effort medium; lượt phá chạy model khác họ ở effort cao ("code ở effort thấp, kiểm ở effort cao", theo claude.dev). Architect chỉ gọi khi có bất định cấu trúc.
- **Cần xác minh:** Thirty khai `effort: high` trong frontmatter subagent, nhưng memory Nếp nhà ghi "effort subagent không đặt được qua frontmatter". Phải kiểm docs Claude Code hiện tại trước khi viết vào contract.

### 6.6 Đo trước khi tin
Pilot kế tiếp ghi 4 con số mỗi mốc:
1. Token/usage theo vai.
2. Thời gian thực.
3. Lỗi bắt được, chia theo ai bắt: builder / reviewer cùng model / reviewer khác model / anh / Apple.
4. Số lần anh bị làm phiền không cần thiết.

Không có số này thì "Lite tốt hơn" vẫn chỉ là cảm giác, đúng như README Lite tự nhận.

### 6.7 Còn sos-kit thì sao
Đề xuất chuyển vai trò sos-kit từ "workflow" sang **"thư viện gate + Lite làm mặc định"**:
- Giữ các binary và hook còn đáng giá: guard env/secrets, features-guard, quality-gate, doc-rotate, runtime-scan.
- Đưa workflow nặng (phiếu, debate, handbook) vào archive, có ghi ngày và lý do.
- Việc này là quyết định của anh. Nên làm sau pilot có số đo, không phải trước.

---

## 7. Câu hỏi để anh quyết

1. Lượt phá cuối mốc: bắt buộc khác họ model (Codex ↔ Claude), hay chỉ cần ngữ cảnh mới?
2. "Mốc" là gì với app nhỏ: mỗi phase của BLUEPRINT, hay chỉ trước khi giao anh dùng thử và trước khi nộp Apple?
3. Pilot đo số ở app nào: quay lại Nếp nhà (đã có nền, còn luồng B), hay app 4 (5E Session Sheet đang chốt bên Hub)?
4. Có đưa sos-kit về "thư viện gate" sau pilot không, hay giữ song song?

---

## Nguồn

- Repo: `~/code/{lucilius,thirty,tally}`, GitHub `aspelldenny/nep-nha-ios` (clone tạm), `~/sos-kit/harness-lite`, transcript Codex `2026-10-05T14-28-08`, Claude `nep-nha-ios/326e7396…`, ghi chú `VibeNotes/30_Resources/ai-thinking/triet-ly-lam-viec-voi-ai.md`.
- Anthropic: code.claude.com/docs/en/{best-practices, memory, features-overview, sub-agents, agent-teams}; anthropic.com/engineering/{building-effective-agents, multi-agent-research-system, effective-context-engineering-for-ai-agents, effective-harnesses-for-long-running-agents, harness-design-long-running-apps, building-c-compiler}; claude.com/blog/steering-claude-code-…; claude.dev/blog/{the-new-rules-of-context-engineering-for-claude-5-generation-models, spending-your-effort}; platform.claude.com prompting pages (Opus 5, Fable 5, Opus 5.5).
- OpenAI: learn.chatgpt.com Codex docs (AGENTS.md, subagents), developers.openai.com/codex/{hooks, sandboxing}; *Harness engineering* và *Unrolling the Codex agent loop* **[2nd]** (openai.com trả 403).
- Khác: arXiv 2602.11988 (ETH AGENTS.md) **[2nd]**, 2311.17371 / 2502.08788 (debate); Cognition *Don't build multi-agents*; repo superpowers, spec-kit, gstack, BMAD, 12-factor-agents, ralph.

---

## 8. Bổ sung 08/10 — phối hợp model và điểm mù của người soát

### 8.1 Advisor (bài anh nhớ là "The advisor strategy", claude.com/blog, 09/04/2026) [docs]
- **Cơ chế:** model chính (thợ, ví dụ Sonnet 5.5) chạy việc từ đầu đến cuối. Tại điểm khó, nó tự gọi advisor (Opus 5.5 hoặc Fable 5.1). Advisor đọc toàn bộ transcript và trả về kế hoạch, lời sửa, hoặc lệnh dừng (thường 400–700 token). Advisor **không gọi tool, không viết code**: "người hướng dẫn chọn hướng, thợ code".
- **Số chính chủ:** Sonnet + Opus advisor trên SWE-bench Multilingual cao hơn Sonnet đơn 2,7 điểm và rẻ hơn 11,9% mỗi task.
- **Trong Claude Code** (docs `code.claude.com/docs/en/advisor`, experimental):
  - Bật bằng `/advisor opus|fable`, `advisorModel` trong settings, hoặc `--advisor`.
  - **Subagent kế thừa advisor**, kiểm cặp theo model của chính nó. Sonnet 5.5 nhận advisor Fable, Opus 5 trở lên, hoặc Sonnet 5.5.
  - Claude tự quyết khi nào gọi, thường là trước khi chốt hướng, khi lỗi lặp lại, và trước khi tuyên bố xong. Không có setting ép hoặc giới hạn số lần gọi; chỉ điều chỉnh được bằng lời dặn.
  - Bật/tắt giữa phiên không phá cache. Nhưng mỗi lần gọi, advisor đọc lại toàn transcript, không cache.
  - Chỉ chạy trên Anthropic API (gói subscription vẫn dùng được). Advisor Fable tính vào usage credits.
  - Máy anh: Claude Code 2.1.287, đủ phiên bản.
- **Giới hạn quan trọng:** advisor nhìn **đúng transcript của thợ**, nên mạnh về chọn hướng nhưng **không độc lập**. Nó chung điểm mù với những gì thợ đã thấy. Advisor thay được *review mỗi lát* và phần lớn *vai architect trong lát*, nhưng không thay được người soát.

### 8.2 Vì sao người soát vẫn mù
- Người soát được giao BLUEPRINT + diff, nên **thế giới của nó là bản vẽ và code**. Nó kiểm "code có khớp bản vẽ không" (từ trong ra), như unit test viết cùng code: chứng minh điều đã nghĩ tới, không tìm điều chưa nghĩ tới.
- Lỗi đắt ở Payquill và Nếp nhà đều **nằm ngoài bản vẽ**: file restore phá hoại, 1.826 ca, AX5, 00:00, bản ghép, máy thật.
- **Đổi model thôi chưa đủ.** ICML 2025 (arXiv 2506.07962): các model mạnh, kể cả khác nhà cung cấp, khi cùng sai thì trùng nhau khoảng 60%. Cùng nhà cung cấp còn trùng hơn, và judge cùng họ chấm phồng điểm cho nhau. NeurIPS 2024 (arXiv 2404.13076): model chấm bài của chính mình cao hơn. → Khác model giúp, nhưng **khác đầu vào và khác câu hỏi** mới là thứ mở góc nhìn.

### 8.3 Người soát "từ ngoài vào"
- **Đầu vào:**
  - Ý định và lời hứa với người dùng (PRODUCT/SOUL/acceptance).
  - **Bản chạy trên bản ghép**.
  - Dữ liệu thật, lớn, hỏng.
  - Ma trận môi trường (máy thật/sim, OS, cỡ chữ, múi giờ).
  - Code chỉ dùng để truy nguyên nhân. Báo cáo của thợ đọc sau đánh giá đầu tiên.
- **Câu hỏi thay cho checklist:**
  1. **Hành trình:** đi lại từng hành trình người dùng từ đầu đến cuối, kể cả lúc bị gián đoạn (kill app, mất mạng, mất quyền, đổi ngày giờ).
  2. **Vòng đời dữ liệu:** tạo → sửa → lưu → backup → restore → export → xoá; với dữ liệu lớn và dữ liệu hỏng.
  3. **Lời hứa:** mỗi màn hình, mỗi câu chữ hứa gì với người dùng? App có giữ được lời hứa đó không, hay đang hiện trạng thái nói dối?
  4. **Biên:** thời gian (DST, 00:00), cỡ chữ, máy nhỏ, OS cũ, máy thật khác sim.
  5. **Pre-mortem:** "3 tháng sau app bị 1 sao / bị Apple từ chối / làm mất dữ liệu người dùng, vì sao?" Lấy 3–5 giả thuyết rồi kiểm từng cái.
  6. **Cái không được vẽ:** BLUEPRINT im lặng về điều gì mà người dùng chắc chắn sẽ gặp?
- **Đầu ra:** mỗi finding phải tái hiện được trên bản chạy (bước, dữ liệu, môi trường). Không nhận ý kiến về style code.
- **Hội tụ với nguồn ngoài:** evaluator của Anthropic click trên app đang chạy theo tiêu chí; long-running harness đòi test như người dùng; gstack `/qa` chạy trình duyệt thật; RRI-T anh đã đánh giá (25/09) cũng khuyên khám phá rủi ro theo câu hỏi.

### 8.4 Gọi người soát khi nào (không mỗi lát)
- **Gọi khi có sự kiện:**
  - Một hành trình người dùng **vừa nối liền lần đầu**.
  - Lát chạm vùng rủi ro: lưu/restore/migration dữ liệu, tiền/tính toán, quyền riêng tư/bảo mật, đồng bộ/đa thiết bị, thanh toán.
  - Trước khi giao anh dùng thử hoặc nộp Apple.
  - Khi có tín hiệu xấu: thợ sửa 2 lần không xong, advisor cảnh báo, báo cáo mâu thuẫn với bằng chứng.
- **Không gọi:** lát UI nhỏ, đổi chữ, refactor đã có test bao, lát đã có oracle độc lập.
- **Ước lượng:** một app cỡ Nếp nhà (khoảng 10 lát) cần khoảng 2–3 lượt soát thay vì 10, cộng advisor ở các điểm quyết định.

### 8.5 Ba tầng phối hợp model
| Tầng | Ai | Việc | Khi nào |
|---|---|---|---|
| Trong lát | Thợ Sonnet 5.5 (medium) + advisor Opus 5.5/Fable 5.1 | Code; advisor chọn hướng, gỡ kẹt, xác nhận trước khi báo xong | Model tự quyết tại điểm quyết định |
| Mốc hành trình / rủi ro | Người soát ngữ cảnh mới, nhìn từ ngoài vào | Hành trình, vòng đời dữ liệu, lời hứa, biên, pre-mortem trên bản chạy | Theo sự kiện ở 8.4 |
| Nghiệm thu (gợi ý) | Model khác họ (`codex exec` read-only, máy anh đã có Codex 0.146) hoặc Hub | Lượt phá có giới hạn thời gian | Trước khi giao anh / nộp Apple, khi app có dữ liệu hoặc tiền |

**Cần đo ở pilot:** lỗi bắt được theo từng tầng, token theo tầng, và số lần advisor được gọi.
