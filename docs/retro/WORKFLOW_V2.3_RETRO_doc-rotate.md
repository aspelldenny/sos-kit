# WORKFLOW v2.3 RETRO — pilot vòng 2 (doc-rotate)

> **STATUS: OPEN — forge in progress.** Khung này SẮP XẾP bằng chứng + LIỆT KÊ câu doctrine phải quyết. **KHÔNG ratify gì.** Nothing here is doctrine cho tới khi forge qua nhiều vòng (v2.2 mất 7 vòng — không pin trước số vòng, chạy tới khi hội tụ). Per CLAUDE.md "Edit Workflow doctrine": đây là kênh hợp lệ DUY NHẤT để đổi doctrine; cấm sửa `WORKFLOW_V2.x.md` ad-hoc ngoài file này.
>
> Opened: 2026-05-29. Pilot: doc-rotate (Python, oracle PARTIAL — test giả thuyết PARTIAL-oracle, non-Rust đầu tiên).
> Seed: 3 Claude Code workflow (spec-review v2.2 + gap-analysis doc-rotate + invariant/variant map) + 7 Discovery Report P001–P007 + maintainer review 2026-05-29.

---

## §0. Ba cú lật (framing — phải đọc trước khi forge)

1. **Agent KHÔNG quên xương sống — KHUÔN nói hai giọng.** Spine (5 vai, envelope, state machine, skeleton phiếu, Tầng) giống hệt qua 5 repo / 3 thế hệ; agent chưa từng sai. Nó "quên" ở chỗ khuôn golden tự mâu thuẫn (`ticket_dir = "phieu/active"` — field *ticket* giá trị *phieu*, trong MỘT biến). Agent trung thành chép một nguồn hai giọng. → Slogan "đừng bắt LLM nhớ, bắt cơ chế nói sự thật": vế-1 đúng (agent đọc từ khuôn), **vế-2 hỏng — cơ chế đang nói DỐI.** Sửa khuôn nói MỘT giọng → agent tự hết quên.

2. **v2.2 không thêm cơ chế CHẠY — nó thêm cơ chế NGỦ.** Tất cả thứ v2.2 thêm so với v2.1 (doctor, lane-check, AGENT_MAP validate, Giám sát-injection) đều present-but-dormant. "5 version hỗn loạn" thực ra = **MỘT workflow thật (spine) chạy khắp nơi + một lớp tham vọng v2.2 chưa cắm răng + vài chỗ khuôn cãi nhau.**

3. **doc-rotate KHÔNG phải pilot v2.2 thành công — nó là bằng chứng v2.2 đang ngủ. tarot là reference cho WIRING PATTERN của vai Giám sát — KHÔNG phải reference cho nội dung/cấu trúc repo** (**reference-WIRING, KHÔNG reference-QUALITY** — tarot chạy Giám sát nhưng CHƯA từng bị canary chất lượng, §1 A2; tarot là Next.js/product nên cấm copy bừa map/INVARIANTS — "thuốc độc map nói dối"). Cơ chế khó nhất của v2.2 — vai Giám sát (boundary-check inject INV-LOCAL) — chạy end-to-end ĐÚNG 1 repo: tarot. **Bằng chứng (grounded 2026-05-29): Giám sát xuất hiện 0 lần trong CẢ 7 phiếu doc-rotate** (P001–P007, grep `boundary-check|security-review|giám sát` = 0/0/0/0/0/0/0). doc-rotate auto-merge không qua gác.

**Hệ quả cho đọc lại pilot:** §4-Q1 "PASS" phải đọc là **"PASS cho mảng Architect–Worker, N/A cho mảng Giám sát vì Giám sát chưa từng chạy".** Pilot chứng minh 2/3 của v2.2; phần khó nhất (1/3) vắng mặt. Cảnh báo trung thực: 7 report do agent tự viết, tự chấm (308 ✅ / 72 "pass" / "discipline held"), trong khi vai chịu trách nhiệm GÁC discipline chưa chạy → lời tự khen không đáng tin khi người kiểm cũng là agent.

---

## §1. Ba cột — bảng SẮP XẾP (chưa quyết gì)

### Cột A — CHỨNG MINH THẬT (proven)
> Tách đôi theo test "có CHẠY không" (không gộp present với running):

**A1 — proven-RUNNING khắp nơi** (đã cắn ở mọi repo gồm pilot Rust thành công):
- 5 vai + tool envelope (architect no Bash/Grep/Edit; worker no vision-docs; orchestrator `tools:[]`)
- State machine DRAFT→CHALLENGE→[RESPOND⇄CHALLENGE cap=3→FORCE_ESCALATION]→APPROVAL_GATE→EXECUTE + Tầng2-skip-CHALLENGE
- Skeleton phiếu: Task 0 Verification Anchors + humility markers + Debate Log cap=3 + Nghiệm thu
- Model Tầng 1|2; architect-guard.sh + block-env-edit.sh; type-check + docs-gate spine; DOCS GATE 2-Tầng; knowledge-durability convention; Sub-mech A–F rubric
- **Pilot win (Architect–Worker only):** PARTIAL-oracle HOLD trên Python — oracle self-stop 5/5, 0 routing violation /7 phiếu (Q1)
- **Pilot win:** cap-gate WIRE-và-BLOCK được — P007 A2, HEAD SHA unchanged, byte-exact restore (Q4) — bằng chứng đầu tiên §6 gate có răng thật

**A2 — proven-PRESENT khắp nơi nhưng SPAWNS đúng 1 repo (tarot); chất-lượng verdict CHƯA canary; bằng chứng-chạy = n=1:**
- Giám sát spawn scaffold (boundary-check.md + /security-review + sentinel + auto-invoke) + block-unsafe-merge.sh
- ⚠️ **KHÔNG promote A2 lên "proven" mà không có pilot nữa.** Invariant về hình thức ≠ proven về vận hành.
- ⚠️ **spawning ≠ gating-well** (forge R1, present-vs-running một tầng nữa): "tarot chạy Giám sát" mới chứng minh nó SPAWN + ra verdict, CHƯA chứng minh verdict có CHẤT LƯỢNG. Cần canary verdict-quality trên tarot TRƯỚC khi promote A2. ✅ **[GROUNDED 2026-05-29]:** N1 canary 1+2 CHẠY THẬT trên **advisory-inbox PR#14** (retro v2.2 L192/243/249) — kết luận: gác soi-thật KHI có rubric, MÙ khi không inject. tarot 86 "canary" = **deployment** (0 ở context Giám sát); chỉ có wiring smoke-test, KHÔNG quality canary. **Lật một bậc: quality canaried ở advisory-inbox, runs-live ở tarot — KHÔNG repo nào vừa-live-vừa-proven-quality.** Cái N1 chứng minh là INJECTION mechanism, không phải verdict-quality của Giám sát tarot đang chạy.

### Cột B — PHƠI RA HỎNG (dormant + mold bug, grounded)
| Item | Bằng chứng | Loại |
|---|---|---|
| **Giám sát dormant** | 0/7 phiếu doc-rotate; chết vì 4 đứt gãy đồng thời (xem dưới) | dormant |
| **Sentinel mismatch** | golden `block-unsafe-merge.sh:100,102` grep `SECURITY_REVIEW_START` (HOA+`_`) vs emit `security-review-start` (thường+`-`) ở boundary-check.md:39/174/246 + security-review.md:48/55/86 → verdict KHÔNG BAO GIỜ match → chuỗi merge-gate chết ở gốc | **mold bug — pure** |
| **ticket↔phiếu hai giọng** | `ticket_dir = "phieu/active"`; golden mâu thuẫn 5 mặt; Discovery gate grep `^docs/ticket/` trong khi phiếu sống ở `phieu/` → gate DEAD | **mold bug — doctrine** |
| **doctor BLOCKING NOWHERE** | serve-mode MCP ở cả 3 repo v2.2, không hook nào invoke | dormant |
| **lane-check chết** | no `**Lane:**` field (chỉ `**Tầng:**`) + sai path + vocab 3-lane (doctor) vs 4-lane (doctrine `Locked`) | dormant + doctrine |
| **AGENT_MAP không validate** | chỉ doc-rotate có map; validate-map serve-only | dormant |
| **markers không instantiate** | luật §0.1 bắt đánh cờ; 0/5 repo có `[mechanical]/[judgment]` inline | doctrine |

**Vì sao Giám sát chết (4 đứt gãy, mọi cái STATIC):** (1) sentinel mismatch; (2) /security-review thiếu bước đọc+inject INVARIANTS (bước đó chỉ sống trong orchestrator handbook); (3) không có gì auto-spawn; (4) không có docs/security/INVARIANTS.md để inject.

### Reference — tarot (chép "Giám sát chạy đúng trông thế nào")
- INV-101→107 (audit sửa: **107 không phải 108**) + /security-review wired + orchestrator auto-invoke + block-unsafe-merge sentinel **khớp UPPERCASE cả 2 đầu** → advisory verdict → hard merge gate.
- Lưu ý: tarot **hardcode** dải INV trong command, agent tự đọc `docs/security/INVARIANTS.md:135-192` — KHÔNG phải dynamic-inject. (Khi forge: quyết tarot-pattern hardcode hay dynamic-inject là chuẩn.)
- **Phạm vi reference (forge R1 narrow):** chỉ chép WIRING PATTERN của Giám sát (sentinel khớp, command có bước inject, auto-spawn trigger, merge-gate wired). KHÔNG chép nội dung/cấu trúc repo. Cái transferable từ tarot là wiring-pattern, KHÔNG phải invariant cụ thể của nó.

---

## §2. NÚT TRUNG TÂM — verify-vai-CHẠY (không phải verify-vai-CÓ-FILE)

**Bài học sâu nhất:** cả maintainer lẫn agent verify setup bằng MẮT (đếm file vai), mà mắt quên y như agent. doc-rotate đủ file boundary-check → tick "đủ vai" → nhưng spawn 0 lần. **Verify sự TỒN TẠI ≠ verify sự HOẠT ĐỘNG.** Slogan phải áp lên chính việc kiểm setup: đừng bắt MAINTAINER nhớ check vai có chạy — phải có cơ chế tự báo.

**Đề xuất MVP (forge để xác nhận):** lỗi doc-rotate 100% phát hiện được TĨNH → `doctor verify-setup` kiểm **toàn vẹn chuỗi wiring** mỗi vai, không đếm spawn runtime:
- mỗi vai: trigger có wired? input tồn tại? emit/parse contract khớp (sentinel case+separator)? enforcement wired vào merge/commit path?
- output: `boundary-check: DORMANT — sentinel HOA vs thường; no INVARIANTS.md; no auto-spawn` (cái báo động đáng-lẽ-cứu-anh lúc bootstrap)
- **Đây là cơ chế spec ĐỂ DEFERRED — retro un-defer thành yêu cầu v2.3.** Nếu chốt doctrine mà thiếu nó → repo sau lại đẻ ra một doc-rotate khác (đủ file, auto-merge không gác).

**⚠️ TENSION (forge R1) — verify-setup có nguy cơ thành con bệnh tiếp:** ai viết checklist "wiring đúng trông thế nào"? Cùng cái đầu vừa viết khuôn hai giọng (`ticket_dir=phieu/active`). Checklist tự-nghĩ sẽ PASS đúng cái lỗ mình mù. → **Xây từ DIFF-với-tarot (reference-based), không từ checklist-tự-nghĩ (spec-based)** — tarot là oracle (nơi DUY NHẤT Giám sát cắn), không phải cái đầu mình. **Reconcile (đề xuất em):** không phải chọn một — **derive checklist TỪ wiring-chain đã-chạy-thật của tarot**; các mục v0 (sentinel khớp / command tồn tại / auto-invoke / INVARIANTS tồn tại / merge-hook wired) trở thành GIẢ THUYẾT được tarot xác nhận đủ, không phải bịa trong chân không. ⚠️ KHÔNG diff tarot WHOLESALE (tarot n=1, Next.js — sẽ ra false-positive cho tool Rust); chỉ diff đúng **wiring-pattern của Giám sát** (khớp §1 narrow). → vào **Q-D5**.

**✅ Doctrine line (R2, 3 nguồn hội tụ — ghi khi chốt v2.3):** `verify-setup PASS ≠ security-review quality PASS`. *verify-setup kiểm DÂY (tĩnh/sound, Q-D5); canary THỬ CHÁY (runtime/partial, Q-D7). Đừng dùng đồng hồ đo dây khẳng định báo cháy hoạt động. Hai con dao — một mài lưỡi, một thử chém — đừng rèn thành một tool.*

---

## §3. CÂU DOCTRINE PHẢI QUYẾT (UNRESOLVED — forge, KHÔNG sửa lén)
> Đây là các nút làm khuôn cãi nhau. Mỗi câu chốt 1 giọng thì một "bug-thuần-trá-hình" mới được sửa.

- **Q-D1 — Một path hay hai path phiếu?** ✅ **[RESOLVED R3]:** KHÔNG phải "chọn phieu hay ticket" — path là **per-repo VARIANT** (sos-kit=`phieu/active|done`, downstream=`docs/ticket/`; dual cố ý qua phieu.sh location-detect). Bug thật hẹp: hook **hardcode** `^docs/ticket/` trong khi `.docs-gate.toml ticket_dir="phieu/active"` → Discovery gate CHẾT cho sos-kit. **Fix theo single-source (hạt giống Q-D6):** hook **ĐỌC** `ticket_dir`, không hardcode → ✅ **DONE + canary** (phieu/active FIRE; portable downstream; `bash -n` OK). **Ca thử Q-D6 quy mô 1-từ-vựng = PASS** → có bằng chứng để sau build Q-D6 đầy đủ. **Prose half CHƯA làm:** README/INSTALL/phieu-README/architect.md reference ticket_dir thay vì restate `docs/ticket/`. **Fleet default ✅ [Sếp]: repo MỚI = `docs/ticket/`. sos-kit MIGRATED HẾT về docs/ticket (R4) — KHÔNG còn dogfood-exception, một giọng tuyệt đối.** DONE: 14 phiếu → `docs/ticket/done/`; `ticket_dir`→docs/ticket; 8 functional surface swept (hook đọc ticket_dir / banner / phieu.sh detect / TICKET_TEMPLATE / worker.md×2 / CLAUDE.md×2 / .docs-gate); + 2 forward-looking (BOOTSTRAP default, ORCHESTRATION:212). **GIỮ NGUYÊN:** historical phiếu content + CHANGELOG (no rewrite history), phieu/ backbone (workflow component), legacy-fallback trong phieu.sh/banner (robust repo cũ). **Verified:** gate canary docs/ticket FIRE + bash -n OK.
- **Q-D2 — Lane hay Tầng? 3 lane hay 4?** Reconcile doctrine §1 + doctor binary (`Normal|Guarded|Fast`) + template (`Tầng`) + advisory-inbox (`Locked`). Hoặc thêm `**Lane:**`, hoặc bỏ lane-check khỏi orchestrator/§1.
- **Q-D3 — Đánh cờ `[mechanical]/[judgment]` mọi hard rule, hay nới luật?** (0/5 instantiate hôm nay, kể cả golden.)
- **Q-D4 — Bước inject INVARIANTS sống ở /security-review command hay orchestrator handbook?** (gap path-chạy vs handbook.)
- **Q-D5 — verify-WIRING cơ chế nào?** ✅ **[RESOLVED R2 framing — 3 nguồn hội tụ]:** `doctor verify-setup` = subcmd thứ 5, **TĨNH/SOUND** (đọc file/grep/check tồn tại — KHÔNG spawn; file-checked: 4 subcmd doctor hiện tại đều static 0-spawn → canary KHÔNG nhét vào đây kẻo bẩn binary sound). Kiểm chuỗi nối (sentinel khớp / command tồn tại / INVARIANTS tồn tại nếu enabled / auto-spawn / merge-hook / inject-path). Reference = **tarot-WIRING** (wiring tĩnh kiểm được, tarot là mẫu đã-chạy). Nhãn cứng: **kiểm NỐI, KHÔNG kiểm TỐT.** Chạy bootstrap + pre-commit. Un-defer từ spec. → quality KHÔNG ở đây, xem **Q-D7**.
- **Q-D6 (forge R1, câu GỐC) — cơ chế nào ép khuôn nói MỘT giọng xuyên các file?** Ba cú lật đều quy về một bệnh: khuôn nói hai giọng (ticket/phieu, sentinel HOA/thường, lane/Tầng). Q-D1…Q-D5 chốt từng chỗ = nhổ từng cây cỏ; Q-D6 diệt gốc. `ticket_dir=phieu/active` qua được docs-gate vì docs-gate kiểm ĐỘ-MỚI, không kiểm NGHĨA. Cần **"vocabulary-consistency = docs-gate cho NGHĨA"**. ⚠️ **MVP phải RẺ (Luật 2), KHÔNG phải AI semantic synonym-detector:** đề xuất = một glossary khai báo từ-vựng-chuẩn (term + sentinel canonical) + grep cấm biến thể xuyên file. Bằng chứng đây là gốc: sentinel vừa fix (sos-kit→thường) lệch với tarot (HOA) — **cross-repo vocab drift VẪN sống**, đúng cái Q-D6 phải bắt. *(Q-D5 verify-setup = wiring 1 vai; Q-D6 = vocab xuyên file — cùng họ static-consistency, có thể cùng tool `doctor`, đừng vội gộp.)*
- **Q-D7 (forge R2, TÁCH khỏi Q-D5) — verify-QUALITY bằng canary.** Vai giám sát phải qua planted-flaw canary (đút vi phạm INV-LOCAL giả → BẮT BUỘC FLAG; đút bản sạch → APPROVE), chứng minh GÁC chứ không pass-through. **RUNTIME/PARTIAL** — spawn boundary-check thật, tạo side-effect (PR/branch phải dọn). **KHÔNG phải subcmd doctor** (giữ doctor sound). N1 (advisory-inbox PR#14) đã chứng minh phương pháp. **3 chỗ file-grounded cả 2 reviewer chưa thấy:**
  - **(i) Cadence = mỗi PHIÊN BẢN prompt giám sát, KHÔNG "1 lần/repo".** File-checked: **3 version boundary-check.md live** — 255 dòng (advisory-cron/inbox, md5 giống) · 278 (doc-rotate/sos-kit) · **166 (tarot)**. Behavior đổi theo prompt → canary v255 không nói gì về v278/v166. **tarot chạy v166 = version DUY NHẤT production, không khớp golden v278, chưa canary.** (Sub-mech A tầng behavior: "test 1 lần" thành prose-không-răng nếu prompt đổi mà không re-test.)
  - **(ii) Canary phải đi ĐÚNG production injection-path, KHÔNG force-inject riêng.** File-grounded failure-mode #3: boundary-check.md:245 (cron/inbox) ghi *"project-local INV not Giám sát's responsibility unless user updates this agent file"* → normal op vai BỎ QUA local INV. Force-inject để test = certify cấu hình KHÁC production (đúng gap §8). → nối Q-D7 vào **Q-D4** (inject sống ở đâu thì canary test đúng chỗ đó).
  - **(iii) Giới hạn partial của chính canary:** planted-flaw do cùng cái đầu có lỗ nghĩ ra → canary PASS = "không pass-through trên lỗi ĐÃ BIẾT", KHÔNG phải "bắt vi phạm MỚI". Cấm overclaim canary-PASS = "quality proven". (Recursion-limit Claude chỉ cho verify-setup, áp luôn cho canary.)
  - **Nhánh production-safe:** canary trên sandbox (doc-rotate/advisory-inbox) vô hại; trên **tarot (production)** planted-flaw PR lỡ merge = chạm prod → cần nhánh riêng (no-merge, auto-dọn, isolated).
  - **(iv) Precondition (reviewer "con mắt soi lại" R3):** hai-tool **sống sót cú phá** (phẳng-vì-đúng, không cùng-mù) NHƯNG lòi mối nối thứ tự: canary có **tiền đề verify-setup PASS** — không spawn được vai để test nếu sentinel còn lệch. **canary-FAIL khi wiring chưa xanh phải đọc "CHƯA NỐI" (Q-D5), KHÔNG "GÁC DỞ" (Q-D7)** — chặn lỗi đọc-nhầm-tín-hiệu.
  - 🔴 **PRODUCTION RISK CHƯA ĐO (track riêng, KHÔNG xuống BACKLOG chung):** tarot đang gác production THẬT bằng boundary-check **v166 unique, chưa từng canary quality**. Nếu v166 gác dở (pass-through) → tarot merge code qua gác ngủ mà không ai biết. Đây là rủi-ro-đang-mở, không phải nice-to-have.

---

## §4. THỨ TỰ (maintainer-corrected 2026-05-29)

1. **Step 1 (NGAY, không cần retro):** chỉ 2 bug THẬT SỰ thuần — một cách sửa, không quyết gì:
   - **sentinel case+separator:** sửa `block-unsafe-merge.sh` grep `security-review-start`/`-end` (thường+`-`, khớp emit). 1 file outlier, 2 file emit đã đồng thuận.
   - **bootstrap rename:** copy procedure `sed` tên repo vào header (advisory-inbox còn ghi "advisory-cron").
   - ⛔ KHÔNG đụng phieu-path, inject-step ở bước này (= Q-D1, Q-D4).
2. **Step 2 (forge này):** quyết Q-D1…Q-D5. Dùng tarot làm reference cho A2/Giám sát.
3. **Step 3 (SAU forge):** sửa bug dính-doctrine theo quyết định + build verify-setup → RỒI mới skeleton (skeleton = A1-spine đóng băng + checklist per-project; **KHÔNG** đóng băng A2/dormant).

---

## §5. Forge log
> (điền theo vòng — pilot → CHẨN ĐOÁN → reviewer rounds → ĐƠN THUỐC v2.3 → hạ vào golden)

- **Vòng 0 (2026-05-29):** khung mở, bằng chứng grounded, 5 câu doctrine xác định. Chưa quyết gì.
- **Vòng 1 (2026-05-29):** 2 reviewer ngoài (ChatGPT = approve+bounded verify-setup v0; Claude = 3 vặn + Q-D6) + cross-check.
  - **HỘI TỤ (hành động, nhất trí 3 nguồn):** sửa sentinel NGAY → **ĐÃ LÀM + canary 3/3 PASS**; giữ bootstrap-rename chờ soi; thứ tự Step1/2/3; phieu-path+inject = doctrine không phải bug thuần; un-defer verify-setup; verify TỒN-TẠI≠HOẠT-ĐỘNG.
  - **CHƯA hội tụ (doctrine):** ① **verify-setup design = 2 reviewer MÂU THUẪN** (ChatGPT spec-checklist vs Claude diff-với-tarot) → reconcile = "tarot-derived checklist" (§2 TENSION), vào Q-D5. ② tarot-reference → narrow về wiring-pattern (folded §0/§1). ③ A2 "runs"→"spawns, verdict-quality chưa canary" (folded §1, kèm cờ ground). ④ **+Q-D6 vocab-consistency** (folded §3).
  - **Chưa quyết:** Q-D1…Q-D6. Next: ground claim canary-tarot + chốt fork Q-D5 (Sếp call).
- **Vòng 2 (2026-05-29):** ground canary (✅ N1 = advisory-inbox PR#14 thật; tarot canary = deployment, KHÔNG quality) → đặt câu hỏi wiring-vs-quality cho 2 reviewer.
  - **HỘI TỤ 3 nguồn (Sếp + ChatGPT + Claude) — RESOLVED framing:** verify-setup ≠ canary là **HAI TOOL, không hai tầng một tool.** Q-D5 = verify-WIRING (doctor subcmd, tĩnh/sound, ref tarot-wiring, nhãn "kiểm NỐI không TỐT"). **Q-D7 mới** = verify-QUALITY (canary runtime/partial, KHÔNG phải doctor). Doctrine line: `verify-setup PASS ≠ quality PASS`. §0 hạ tarot xuống reference-WIRING.
  - **Delta ChatGPT vs Claude:** Claude thêm 3, em file-VALIDATE cả 3: canary ≠ doctor-subcmd (doctor 0-spawn ✓); per-VERSION không per-repo (3 version live ✓); production-safe branch. Cấu trúc Claude (Q-D7 riêng) > ChatGPT (Q-D5b lồng) → dùng Q-D7.
  - **Em (file) thêm 3 cái cả 2 reviewer chưa thấy → folded Q-D7:** (i) tarot chạy v166 unique-chưa-canary; (ii) canary phải dùng production injection-path (failure-mode #3, nối Q-D4); (iii) canary-PASS có giới hạn partial (planted-flaw author-blind).
  - **Em tự soi bias:** lần này KHÔNG rubber-stamp hội tụ — em thêm friction (3 cái trên) trước khi chốt, nên không phải premature-closure. Nhưng cảnh báo: em vẫn thiên đóng-gọn; nếu 3 nguồn + em đều gật thì cần một con mắt CHƯA tham gia soi lại.
  - **Chưa quyết:** Q-D1…Q-D4, Q-D6 còn nguyên mở. Q-D5/Q-D7 mới chốt FRAMING (hai tool), chưa build. Next: production-safe canary design cho tarot (rủi ro thật) HOẶC forge Q-D1 (path — gốc bệnh quên).
- **Vòng 3 (2026-05-29):** Sếp (consequence-bearer) chốt fork = Q-D1. Reviewer làm "con mắt soi lại" trước.
  - **hai-tool sống sót cú phá** (Q-D5/Q-D7 phẳng-vì-đúng) + thêm precondition: canary cần verify-setup PASS; canary-fail-khi-wiring-hỏng = "chưa nối" không "gác dở" (folded Q-D7 (iv)).
  - **Q-D1 ✅ RESOLVED + DONE:** ground → sự thật = path per-repo VARIANT, single-source `ticket_dir` đã tồn tại, bug = hook hardcode `docs/ticket` → gate chết. **Fix: hook đọc ticket_dir** (DONE + canary: phieu/active FIRE, portable, bash -n OK). Discovery gate sos-kit sống lại. **Q-D6 small-scale test PASS.**
  - **CÒN MỞ sau R3:** (a) prose-half Q-D1 (README/handbook reference ticket_dir); (b) **fleet-question** (golden default new-repo = phieu/ticket — Sếp call); (c) 🔴 production-risk tarot v166 chưa-canary-quality (track riêng); (d) Step-1 bootstrap-rename vẫn lửng lơ (reviewer nhắc đóng nốt).
  - **Bias check:** lần này KHÔNG paralysis (Sếp: "bias thật nhưng không thể vì bias mà không làm") — đã LÀM (gate-fix có răng + canary), không chỉ bàn.
- **Vòng 4 (2026-05-29):** Sếp chốt "sos-kit cũng đổi cho hết mâu thuẫn" — full migration sang docs/ticket (không exception).
  - **DONE + verified:** `git mv` 14 phiếu → docs/ticket/done/; ticket_dir→docs/ticket; swept 8 functional surface + 2 forward-looking (BOOTSTRAP draft default, ORCHESTRATION:212); gate canary docs/ticket FIRE; bash -n 3 script OK.
  - **Giữ:** historical phiếu + CHANGELOG (no rewrite history), phieu/ backbone, legacy-fallback (phieu.sh/banner).
  - **bootstrap-rename SOI:** KHÔNG phải bug code sos-kit (advisory-inbox header sai vì tạo bằng copy sibling advisory-cron, không từ golden) → **procedure note cho bootstrap doctrine** ("bootstrap repo mới TỪ golden, không copy sibling"), không sửa lén. Reviewer đoán đúng: dính naming-doctrine → forge, không one-script-sed.
  - **ĐÓNG:** Q-D1 + Step-1 (sentinel + path). **CÒN MỞ:** Q-D2 (Lane/Tầng), Q-D3 (markers), Q-D4 (inject), Q-D6 (vocab tool), Q-D5/Q-D7 (build verify-setup + canary), 🔴 tarot production-risk (track riêng).
