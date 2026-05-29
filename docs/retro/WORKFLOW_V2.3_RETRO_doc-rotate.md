# WORKFLOW v2.3 RETRO — pilot vòng 2 (doc-rotate)

> **STATUS: ALL Q-D FORGED (Q-D1–Q-D7) — chờ Sếp CROSS-CHECK trước khi hạ ĐƠN THUỐC v2.3.** Q-D1 ✅ RESOLVED+SHIPPED (committed `03f0579`). Q-D2–Q-D7 = recommendation forged, **CHƯA ratify/hạ**. **KHÔNG ratify gì** cho tới khi cross-check xong → rồi mới consolidate ĐƠN THUỐC → hạ vào `WORKFLOW_V2.3.md` + golden. Per CLAUDE.md "Edit Workflow doctrine": đây là kênh hợp lệ DUY NHẤT; cấm sửa `WORKFLOW_V2.x.md` ad-hoc ngoài file này. ⚠️ Forge nội bộ bịa evidence 2/3 lần (R6) — external cross-check là lớp bắt cuối BẮT BUỘC. → **R7-R9: Sếp LOCK 3-rổ (§6); Q-D2 ĐO XONG = keep-two-axes (⚠️ ĐƠN THUỐC còn mầm bệnh auto-Guarded-keyword CHƯA xử); hạ theo WAVE, chỉ cái chạy-thật/đo-xong.**
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

## §1. Ba cột — bảng SẮP XẾP (chẩn đoán gốc; verdict đã chốt ở §3 + §6)

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
| ~~**ticket↔phiếu hai giọng**~~ ✅ **FIXED R4** (Q-D1) | (was: ticket_dir=phieu/active, golden mâu thuẫn 5 mặt, gate DEAD) → hook đọc ticket_dir + sos-kit migrate hết về docs/ticket | mold bug — doctrine, ĐÃ ĐÓNG |
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

## §3. CÂU DOCTRINE — ĐÃ FORGE (recommendation, CHƯA hạ — chờ cross-check, KHÔNG sửa lén)
> Đây là các nút làm khuôn cãi nhau. Mỗi câu chốt 1 giọng thì một "bug-thuần-trá-hình" mới được sửa.

- **Q-D1 — Một path hay hai path phiếu?** ✅ **[RESOLVED + DONE — committed `03f0579`]** *(R3 đề xuất per-repo-variant; R4 Sếp chốt MẠNH hơn: một giọng tuyệt đối — bản R3 superseded)*:
  - **Bug gốc:** hook hardcode `^docs/ticket/` trong khi sos-kit thật ở `phieu/` → Discovery gate CHẾT cho sos-kit.
  - **Fix cơ chế (single-source = hạt giống Q-D6):** hook **ĐỌC `ticket_dir`** từ `.docs-gate.toml`, không hardcode → DONE + canary (portable mọi repo). **Q-D6 small-scale test = PASS.**
  - **Quyết định cuối (Sếp R4):** fleet default = `docs/ticket/`; **sos-kit MIGRATED HẾT về docs/ticket — KHÔNG dogfood-exception, một giọng tuyệt đối.** 14 phiếu → `docs/ticket/done/`; ticket_dir→docs/ticket; swept 8 functional + 2 forward-looking (BOOTSTRAP default, ORCHESTRATION:212).
  - **GIỮ NGUYÊN:** historical phiếu + CHANGELOG (no rewrite history); phieu/ backbone (workflow component); legacy-fallback phieu.sh/banner. README/INSTALL dạy docs/ticket = ĐÚNG cho default → không cần sweep prose.
  - **Verified:** gate canary docs/ticket FIRE; bash -n OK; hook chạy thật lúc commit → pass 4/4.
- **Q-D2 — gộp Tầng+Lane 1 field, hay giữ 2 trục?** ✅ **[MEASURED R8 → KEEP-TWO-AXES; R9 cross-check sửa ĐƠN THUỐC]**
  - **Trụ verdict (CỨNG — KHÔNG dựa con số mềm):** (1) loại **high-risk-small TỒN TẠI có tên cụ thể** — P298 crisis-hotline (1 file, life-safety), P296 crisis-bypass-auth (0 code), P291 auth-scheme — đủ bác merge **bất kể tỷ lệ**; (2) **đường-mòn-trên-cỏ:** doc-rotate P005 + advisory P006/P007 tác giả ĐÃ tự override size bằng risk → **người đã đi đường 2-trục dù khuôn chỉ cho 1.** Keep-two-axes = chính thức hóa đường mòn.
  - **Số đo (phụ, MỀM):** 26/49 (53%) high-risk-small / 5 low-risk-large / 18 on-diagonal. ⚠️ panel tự thú 53% soft (corpus tool-CLI 20/49 over-sample); **direction (≠0) robust nhưng magnitude KHÔNG phải trụ** — đừng dựa verdict vào 53%, dựa vào ca-có-tên + đường-mòn.
  - ~~**R5 HYPOTHESIS: merge → 1 field `Lane:` Fast/Normal/Guarded, kill Locked, budget observe**~~ — **REFUTED by R8.** Codex merge-safe bị bác bằng chính tiêu chí Codex (high-risk-small là ô đa số ≠ ~0). auto-Guarded-patch = risk-axis nhét cửa sau = relabeled-two-axis → keep-two-axes thật hơn.
  - **→ ĐƠN THUỐC (corrected R9 — keep-two-axes):** giữ **Tầng (risk)** field hiện có; size-budget → §10-N5 observe (không gate).
    - ⚠️ **MẦM BỆNH đã biết — PHẢI xử trước khi hạ:** "security/schema/auth/dep/migration → auto-Guarded" là **keyword-list cứng = grep một thứ PARTIAL** → TRƯỢT ca khó (P298 life-safety không thuộc 5 keyword). Phép đo tìm ra P298 rồi đơn thuốc keyword lại thả P298 = mỉa mai chết người.
    - **Sửa:** risk là **judgment NGƯỜI gán (partial), KHÔNG suy từ keyword**; doctor chỉ ENFORCE **Tầng-field tồn tại + có lý do** (sound/mechanical); auto-Guarded-keyword chỉ là **LƯỚI AN TOÀN phụ** (đụng auth rõ → chắc Guarded), KHÔNG phải cơ chế chính. Đúng oracle SOUND/PARTIAL: cơ chế chính cho phán đoán partial PHẢI là người.
- **Q-D3 — Đánh cờ markers mọi hard rule?** ⚠️ **[FORGED R6 — verdict: DEFER, KHÔNG instantiate now]:**
  - Forge đề xuất "tag 8 rule now + reject doc-lint". **Stress WOUNDED:** evidence "P006 = Rule #4 không teeth" là **BỊA** (P006 thực ra = docs-gate bootstrap friction, không dính Rule #4) → strip xong disease drop n=1→**n=0**. "Tag rules now" = chính cú over-build session bắt **lần 4** (sau cargo/skeleton/verify-setup). Thêm: Rule #8 đã có Mechanical/Judgment split + Tầng-1 mapping table đã bind surface→enforcer → redundant.
  - **Refined: KHÔNG sửa CLAUDE.md Rules giờ. ARM 1 §10 sensor** (N: rule ships no-enforcer → grep numbered-rule thiếu enforcement-ref; [guidance]→[hook] chỉ sau lần nổ đầu). Precedent thật = DISCOVERIES tarot 265KB (rotate rule unenforced) → argues sensor, không mandate. **Giữ:** reject-doc-lint. **Cắt:** tag-8-rules-now.
  - ĐƠN THUỐC-level (chạm §10) → STOP.
  - **⚠️ CROSS-CHECK R7 (Codex):** defer-tất có thể defer-vì-ngại. Marker tốt **tự thân** (như type annotation — làm rõ rule enforce kiểu gì, hợp executable-contract), KHÔNG cần bệnh biện minh. → **TÁCH: làm annotation 8 rule (rẻ, doc-convention) + defer doc-lint (over-build).** Verdict sửa: KHÔNG defer-tất.
- **Q-D4 — Inject sống ở command hay handbook?** ⚠️ **[FORGED R6 — WOUNDED→refined]:**
  - Forge: "move inject vào command, [hook]". **Stress WOUNDED:** sos-kit /security-review là **Task-tool-driven (LLM prompt), KHÔNG Bash** như tarot → "Step 1.5 grep fires deterministic" SAI cho sos-kit (chỉ relocate prose-to-remember); [hook] label sai → thực ra **[guidance]**. Demote orchestrator.md → bỏ trống path direct-spawn.
  - **Refined (cheapest, L2): thêm SLOT `INV-LOCAL` có nhãn vào Step-2 spawn-prompt TEMPLATE** (đổi "nhớ inject" → "điền chỗ này"), **GIỮ** orchestrator.md:65-76 làm procedure cho direct-spawn (KHÔNG demote — path đó không coverage khác). **[guidance]**. Dynamic-read INVARIANTS (KHÔNG hardcode như tarot — sos-kit command là template N-repo). Disease thật (canary n=2). **DEFER §8 reword → retro.** ĐƠN THUỐC chỉ nếu chạm §8; slot-edit thì không.
- **Q-D5 — verify-WIRING cơ chế nào?** ✅ **[RESOLVED R2 framing — 3 nguồn hội tụ]:** `doctor verify-setup` = subcmd thứ 5, **TĨNH/SOUND** (đọc file/grep/check tồn tại — KHÔNG spawn; file-checked: 4 subcmd doctor hiện tại đều static 0-spawn → canary KHÔNG nhét vào đây kẻo bẩn binary sound). Kiểm chuỗi nối (sentinel khớp / command tồn tại / INVARIANTS tồn tại nếu enabled / auto-spawn / merge-hook / inject-path). Reference = **tarot-WIRING** (wiring tĩnh kiểm được, tarot là mẫu đã-chạy). Nhãn cứng: **kiểm NỐI, KHÔNG kiểm TỐT.** Chạy bootstrap + pre-commit. Un-defer từ spec. → quality KHÔNG ở đây, xem **Q-D7**.
- **Q-D6 (câu GỐC) — vocab-consistency cơ chế nào?** ⚠️ **[FORGED R6 — verdict: KHÔNG build vocab-grep; single-source thay thế]:**
  - Forge: "build glossary + forbidden-variant grep pre-commit, n≥4 bites". **Stress NEAR-FATAL — demolish evidence:** INV_LOCAL underscore = **bịa (n=0**, grep=0); **Tầng-as-lane = MISDIAGNOSIS** (TICKET_TEMPLATE:12 dùng `Tầng:` là CANONICAL — forbid nó = block golden template + **pre-empt Q-D2 đang MỞ**); sentinel cross-repo = **cosmetic** (mỗi repo internally consistent; emit/grep KHÔNG cross repo → casing skew không break gì); phieu/active refs = **legacy CỐ Ý GIỮ** (grep false-flag → --no-verify death). Net live disease cho blocklist ≈ **0**.
  - **Refined: KHÔNG build vocab-blocklist.** Cơ chế đúng = **single-source-the-value** (generalize Q-D1: hook ĐỌC canonical từ 1 declaration, mọi consumer derive — một giọng ở GỐC, 0 grep, 0 false-positive). Drift không single-source được → **fold vào doctor verify-setup STATIC check** (assert emit==grep sentinel WITHIN 1 repo = rủi ro thật doc-rotate, thuộc **Q-D5**), KHÔNG standalone grep. Flags: (1) [gate] reuse single-source; (2) [hook] fold Q-D5.
  - **→ câu GỐC tự hòa tan:** "ép một giọng" KHÔNG cần tool vocab mới — = single-source mỗi value + verify-setup wiring-check. ĐƠN THUỐC-level + tương tác Q-D2/Q-D5 → STOP.
  - **⚠️ CROSS-CHECK R7 (Codex): "tự hòa tan" = OVERCLAIM.** Single-source diệt **value-drift** (ticket_dir, sentinel), KHÔNG diệt **concept-confusion-trong-prose** (ai viết "Tầng" chỗ đáng "Lane" trong văn xuôi). Mới giải **NỬA** — value-half hòa tan; concept-in-prose-half CHƯA giải → **track tiếp** (không build tool, nhưng đừng đóng câu gốc bằng tuyên bố đẹp).
- **Q-D7 (forge R2, TÁCH khỏi Q-D5) — verify-QUALITY bằng canary.** Vai giám sát phải qua planted-flaw canary (đút vi phạm INV-LOCAL giả → BẮT BUỘC FLAG; đút bản sạch → APPROVE), chứng minh GÁC chứ không pass-through. **RUNTIME/PARTIAL** — spawn boundary-check thật, tạo side-effect (PR/branch phải dọn). **KHÔNG phải subcmd doctor** (giữ doctor sound). N1 (advisory-inbox PR#14) đã chứng minh phương pháp. **3 chỗ file-grounded cả 2 reviewer chưa thấy:**
  - **(i) Cadence = mỗi PHIÊN BẢN prompt giám sát, KHÔNG "1 lần/repo".** File-checked: **3 version boundary-check.md live** — 255 dòng (advisory-cron/inbox, md5 giống) · 278 (doc-rotate/sos-kit) · **166 (tarot)**. Behavior đổi theo prompt → canary v255 không nói gì về v278/v166. **tarot chạy v166 = version DUY NHẤT production, không khớp golden v278, chưa canary.** (Sub-mech A tầng behavior: "test 1 lần" thành prose-không-răng nếu prompt đổi mà không re-test.)
  - **(ii) Canary phải đi ĐÚNG production injection-path, KHÔNG force-inject riêng.** File-grounded failure-mode #3: boundary-check.md:245 (cron/inbox) ghi *"project-local INV not Giám sát's responsibility unless user updates this agent file"* → normal op vai BỎ QUA local INV. Force-inject để test = certify cấu hình KHÁC production (đúng gap §8). → nối Q-D7 vào **Q-D4** (inject sống ở đâu thì canary test đúng chỗ đó).
  - **(iii) Giới hạn partial của chính canary:** planted-flaw do cùng cái đầu có lỗ nghĩ ra → canary PASS = "không pass-through trên lỗi ĐÃ BIẾT", KHÔNG phải "bắt vi phạm MỚI". Cấm overclaim canary-PASS = "quality proven". (Recursion-limit Claude chỉ cho verify-setup, áp luôn cho canary.)
  - **Nhánh production-safe:** canary trên sandbox (doc-rotate/advisory-inbox) vô hại; trên **tarot (production)** planted-flaw PR lỡ merge = chạm prod → cần nhánh riêng (no-merge, auto-dọn, isolated).
  - **(iv) Precondition (reviewer "con mắt soi lại" R3):** hai-tool **sống sót cú phá** (phẳng-vì-đúng, không cùng-mù) NHƯNG lòi mối nối thứ tự: canary có **tiền đề verify-setup PASS** — không spawn được vai để test nếu sentinel còn lệch. **canary-FAIL khi wiring chưa xanh phải đọc "CHƯA NỐI" (Q-D5), KHÔNG "GÁC DỞ" (Q-D7)** — chặn lỗi đọc-nhầm-tín-hiệu.
  - 🟡 **PRODUCTION RISK ĐO RỒI (R10 canary):** tarot v166 = **REAL-GATE** — bắt 4/4 lỗi cấy (INV-104 IDOR / INV-105 non-atomic credit / INV-106 unsigned payment-webhook / INV-101 env-map-drift) + approve control sạch (0 false-positive), áp hard-verdict đúng. **KHÔNG pass-through.** Hạ cấp 🔴→🟡. **Residual:** (a) gate **ADVISORY** (detect không block) → phụ thuộc Sếp hành động comment NEEDS_REVIEW; (b) canary textbook single-INV known-class — 4/4 KHÔNG chứng minh bắt subtle/adversarial; (c) proxy agent-follow-prompt, chưa end-to-end /security-review harness. Method Q-D7 validated một phần.

---

## §4. THỨ TỰ (maintainer-corrected 2026-05-29)

1. **Step 1 (NGAY, không cần retro):** bug thật-sự-thuần — một cách sửa, không quyết gì:
   - ✅ **sentinel case+separator (DONE, canary 3/3):** `block-unsafe-merge.sh` grep `security-review-start` khớp emit.
   - ❌→📋 **bootstrap rename (R4 SOI):** KHÔNG phải bug code sos-kit — advisory-inbox header sai do tạo bằng copy sibling, không từ golden. → **procedure note cho bootstrap doctrine** ("bootstrap repo mới TỪ golden, không copy sibling"), KHÔNG one-script-sed. Bỏ khỏi Step-1.
   - ⛔ KHÔNG đụng phieu-path, inject-step ở bước này (= Q-D1, Q-D4) — *(historical: lệnh xếp thứ tự R1; Q-D1 sau đó đã đóng ở R3-R4)*.
2. **Step 2 (forge này):** quyết Q-D1…Q-D7. Dùng tarot làm reference cho A2/Giám sát.
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
- **Vòng 5 (2026-05-29):** Q-D2 forge panel (3 vị trí merge/coexist/kill → adversarial stress → synth). Verdict: KILL survives, MERGE+COEXIST wounded → **recommendation HYBRID** (merge field `**Lane:** Fast/Normal/Guarded`, kill Locked; budget→§10-N5 observe, missing-field→exit-2 gate). Panel tự soi bias (closure/sunk-cost/under-weight-COEXIST) + 5 seam cho reviewer. **Migration = ĐƠN THUỐC, KHÔNG hạ giờ.** Q-D2 ✅ forged. → forge tiếp Q-D3/Q-D4/Q-D6.
- **Vòng 6 (2026-05-29):** forge Q-D3/Q-D4/Q-D6 (grounded forge + adversarial stress mỗi câu).
  - **Stress WOUNDED cả 3, bắt 2 FABRICATION** (Q-D3 "P006=Rule#4-no-teeth" bịa → P006 thực ra docs-gate-bootstrap; Q-D6 "INV_LOCAL n=0" + Tầng-as-lane misdiagnosis). Mọi refined đẩy về **ÍT build hơn** (defer / single-source) — đúng anti-over-build.
  - **Q-D3:** DEFER — arm §10 sensor, KHÔNG tag rules now (disease n=0 sau strip fabrication). Giữ reject-doc-lint.
  - **Q-D4:** thêm `INV-LOCAL` slot vào command spawn-template **[guidance]** (không phải [hook] — sos-kit command là LLM-prompt không Bash), GIỮ handbook cho direct-spawn, defer §8 reword.
  - **Q-D6:** KHÔNG build vocab-grep (evidence demolished, sẽ false-flag + pre-empt Q-D2 mở); answer = **single-source-the-value** (generalize Q-D1) + fold within-repo sentinel-consistency vào **Q-D5 verify-setup**. Câu GỐC tự hòa tan — không cần tool vocab mới.
  - 🔴 **CẢNH BÁO BIAS QUAN TRỌNG:** forge agents (panel NỘI BỘ) **bịa evidence 2/3 lần**; stress nội bộ bắt được — nhưng nghĩa là **internal panel KHÔNG đủ tin**. **External cross-check (Sếp + ChatGPT/Claude) là lớp bắt cuối BẮT BUỘC** trước ĐƠN THUỐC. Đừng tin refined-verdict tuyệt đối tới khi cross-check.
  - **TẤT CẢ Q-D1–Q-D7 ĐÃ FORGED.** Retro COMPLETE. **STOP — chờ cross-check, KHÔNG hạ ĐƠN THUỐC.**
- **Vòng 7 (2026-05-29):** external cross-check (Codex đọc-file + Claude soi-lý-luận) + Sếp LOCK.
  - **Hội tụ 2 reviewer:** Q-D2 là chỗ nguy nhất ("gọn trên giấy, mất răng vận hành") — 2 bias khác (file vs logic) cùng chỉ một chỗ.
  - **4 chỗ bắt trong orchestration của em (nhận hết):** (1) 🔴 META — fabrication R6 phải làm NGHI LẠI mọi verdict pre-R6 (Q-D2@R5, Q-D5/Q-D7@R2 chưa bị stress-bắt-bịa), em chỉ sửa 2 cái lộ; (2) Q-D2 lý-luận-vòng ('doctor đọc 3' = sự-thật-file, '3 đủ' = kết-luận-kill-Locked, đừng trộn); (3) Q-D3 defer-có-thể-vì-ngại (marker tốt tự thân, không cần bệnh); (4) Q-D6 'tự hòa tan' overclaim (single-source diệt value-drift KHÔNG diệt concept-in-prose).
  - **Codex⟷Claude VÊNH ở Q-D2:** Codex 'thêm mapping+auto-Guarded rồi hạ'; Claude 'auto-Guarded = risk-trục nhét cửa sau = dấu hiệu đừng merge → ĐO tương quan trước'. Cú lật chung: **Tầng=risk-axis, Lane=process-axis — tương quan nhưng có thể KHÔNG đồng nhất** (việc nhỏ-đụng-móng).
  - **Sếp LOCK (theo em recommend):** disposition 3-rổ (xem §6); **Q-D2 = ĐO-trước** (Claude), phép đo phân xử Codex-vs-Claude empirically. Measurement launched (~32 phiếu).
  - **Nguyên tắc lock:** chỉ hạ cái proven-running/don't-do; build+chạy cái framing; đo cái Q-D2. Đừng hạ 6 paper một lượt = lặp dormant v2.2 mà retro vừa chẩn.
- **Vòng 8 (2026-05-29):** ✅ phép đo tương quan Tầng/Lane (49 phiếu unique, 3 repo) — **Q-D2 RESOLVED bằng DATA.**
  - **26/49 (53%) high-risk-small** + 5 low-risk-large → 18/49 (37%) on-diagonal. Axes **ANTI-CORRELATED ở ô nguy hiểm**: móng/security tới trong gói NHỎ một cách hệ thống.
  - **Smoking gun:** doc-rotate P005 + advisory P006/P007 — tác giả ĐÃ tự override size bằng risk (1 field không đủ, đã vá tay trong corpus).
  - **Phán xử Codex⟷Claude: CLAUDE đúng.** plain-merge BÁC (Codex merge-safe refuted: 53% ≠ 0). auto-Guarded-patch = risk-axis cửa sau = relabeled-two-axis → keep-two-axes thật hơn.
  - **Caveat panel tự soi:** 53% soft (sample tool-CLI skew) nhưng DIRECTION robust → decision vững.
  - **Q-D2 disposition: 'đo→quyết' → KEEP-TWO-AXES** (giữ Tầng risk gate-driver + doctor đọc Tầng + security floor auto-Guarded; size→observe). Chi tiết = ĐƠN THUỐC. **TẤT CẢ Q-D giờ forged HOẶC measured. STOP — chờ Sếp final-approve hạ ĐƠN THUỐC.**
- **Vòng 9 (2026-05-29):** cross-check phép đo R8 (Codex+Claude) + Sếp dọn-stale + wave-plan.
  - **3 chỉnh trước khi verdict thành trụ ĐƠN THUỐC:** (1) đừng dựa **53%** (mềm, corpus thiên lệch — panel tự thú) → dựa **ca-có-tên** (P298/P296/P291); (2) trụ MẠNH nhất = **đường-mòn-trên-cỏ** (người đã tự vá P005/P006/P007), không phải số; (3) 🔴 **MẦM BỆNH ĐƠN THUỐC Q-D2:** "auto-Guarded keyword-list" = grep một thứ PARTIAL → trượt P298 life-safety (phép đo tìm ra P298 rồi đơn thuốc thả P298 = mỉa mai). **Sửa:** Tầng = judgment người (partial), doctor enforce field-có-lý-do (sound), keyword = lưới phụ. → folded vào Q-D2.
  - **Dọn stale (Sếp):** header "đang ĐO"→"đo xong"; §1 "(chưa quyết gì)"→"(verdict ở §3)"; Q-D2 R5-merge bullets → gói ~~REFUTED by R8~~.
  - **Wave plan §6:** A (file-edit) → B (build verify-setup + chạy) → C (canary) → D (viết doctrine CHỈ sau B/C có evidence + Q-D2 mầm bệnh xử). KHÔNG hạ B/C như A.
  - **→ Commit checkpoint doc-only** (message ghi đúng trạng thái: forged+measured, PENDING-ratification, Q-D2 mầm bệnh auto-Guarded CHƯA xử). KHÔNG ratify gì.
- **Vòng 10 (2026-05-29):** ✅ **canary v166 tarot — BƯỚC SANG VÙNG LÀM** (Claude đúng: làm cái khó-chưa-biết-kết-quả trước; Codex Wave-A-first hoãn).
  - **Kết quả: REAL-GATE.** 4/4 lỗi cấy caught (INV-104/105/106/101) + control approve sạch, 0 false-positive, hard-verdict đúng, silent-when-clean. **tarot KHÔNG gác ngủ** → production-risk 🔴→🟡.
  - **Residual honest:** (a) gate ADVISORY không block → human-follow-through; (b) textbook-không-adversarial (4/4 known-class ≠ bắt subtle/novel); (c) proxy chưa end-to-end harness.
  - **Ý nghĩa:** câu khẩn nhất (production gác ngủ?) ĐÓNG bằng RUN thật, rẻ (~95s, 0 mutation). Q-D7 method validated một phần. **Thứ chạy-thật #3 của phiên** (sau Q-D1 + sentinel) — ra khỏi forge.
  - **Next options:** build verify-setup Q-D5 chạy doc-rotate (test design còn lại — có bắt dormant 0/7 không?) / Wave A (giờ an toàn vì khẩn đã đóng) / cân nhắc advisory→block cho class nguy (design Q mới, không gấp).
- **Vòng 11 (2026-05-29) — CLOSE phiên (Sếp clear context, resume tươi sau):**
  - **Cross-check canary (Codex+Claude):** report KHÔNG overclaim (REAL-GATE known-class, ghi rõ chưa adversarial/end-to-end) — đúng mức.
  - **⚠️ REWEIGHT residual (a) — Claude push, em nhận sai trọng lượng:** advisory-không-block KHÔNG "không gấp" ngang Wave A. Mắt xích cuối lá chắn = **người nhớ đọc comment NEEDS_REVIEW mỗi lần** = ĐÚNG bệnh gốc cả phiên (con-người-nhớ-làm: DISCOVERIES phình, giám sát dormant, "tao cũng quên"). 2 class chí tử (payment-webhook spoof + cross-user leak = tiền + dữ liệu phụ nữ yếu lòng) → 1-lần-lọt quá đắt → **advisory→block cho 2 class đó = residual THẬT NHẤT sau canary** (detect=advisory mềm; hậu-quả-không-đảo-ngược phải gate cứng). Design Q riêng, KHÔNG đêm nay, KHÔNG xếp ngang Wave A.
  - **Quyết: commit Vòng 10-11 checkpoint → NGHỈ/CLEAR.** verify-setup (Q-D5) là việc THẬT (không 95s) xứng đầu-óc-tươi — làm cuối phiên marathon dễ đẻ verify-setup dormant (đúng bệnh retro chẩn).
  - **RESUME (session sau): §6 wave plan** — (B) build doctor verify-setup MVP hẹp → chạy doc-rotate (bắt dormant 0/7?) → ghi result → Wave A. Q-D2 ĐƠN THUỐC còn mầm-bệnh auto-Guarded phải xử (Tầng=judgment người). advisory→block 2-class = Q design mở. Forge arc TRỌN; còn build+chạy + hạ ĐƠN THUỐC.
  - **TAKEAWAY phiên (giữ):** 95s RUN đóng nỗi lo mà 9 vòng forge không đóng. Phần khó KHÔNG phải nghĩ-đúng, là **dừng-nghĩ-cho-chạy**. Lần sau forge tới vòng 5 trên thứ-chưa-chạy → nhớ 95 giây này.

---

## §6. DISPOSITION HẠ ĐƠN THUỐC (locked R7 — Sếp; thước proven-running vs paper)

> 7 verdict, chỉ Q-D1 chạy thật, 6 cái paper. **Hạ cả 6 = lặp đúng bệnh dormant v2.2.** Phân 3 rổ:

- ✅ **HẠ được** (chạy thật / don't-do / generalize Q-D1):
  - **Q-D1** — đã ship (committed 03f0579).
  - **Q-D4** — ĐƠN THUỐC = sửa **CỤ THỂ** slot INV-LOCAL trong `.claude/commands/security-review.md` Step-2 prompt (không chỉ doctrine prose). [guidance].
  - **Q-D6** — don't-build vocab-tool (+ track concept-confusion-in-prose half — CHƯA giải).
  - **Q-D3** — *làm* phần annotation (đánh cờ 8 rule [mechanical]/[judgment] — doc-convention rẻ, tốt tự thân); *defer* doc-lint (over-build).
- 🔨 **BUILD → CHẠY → rồi mới hạ** (framing chốt, paper-chưa-chạy):
  - **Q-D5** verify-setup (static wiring check) · **Q-D7** canary (quality, runtime). Build + chạy thật → rồi mới ghi doctrine.
- 📏→✅ **ĐO XONG → KEEP-TWO-AXES** (Q-D2):
  - **Q-D2** — đo 49 phiếu: **26 (53%) high-risk-small** → plain-merge BÁC, **keep-two-axes**. ĐƠN THUỐC: doctor đọc **Tầng (risk)** thay Lane (sửa dead-lane-check) + **security/schema/auth/dep floor → auto-Guarded** + size → §10-N5 observe. KHÔNG merge 1 field.
- **META (Codex):** fabrication R6 → re-doubt mọi verdict pre-R6. Q-D2 re-doubt = phép đo này; Q-D5/Q-D7 re-doubt = build-then-run (chạy thật là cách chứng).
- **WAVE PLAN hạ (Sếp R9 — chia sóng, KHÔNG hạ một lượt):**
  - **Wave A** (hạ được ngay, file-edit không doctrine): Q-D4 slot INV-LOCAL trong command + Q-D3 annotation + Q-D6 don't-build note.
  - **Wave B:** BUILD `doctor verify-setup` (Q-D5) → **chạy thật.**
  - **Wave C:** chạy quality canary (Q-D7).
  - **Wave D:** chỉ SAU khi Q-D5/Q-D7 có evidence thật **+ Q-D2 mầm-bệnh-auto-Guarded đã xử** → mới viết `WORKFLOW_V2.3.md`/golden.
  - ⚠️ **KHÔNG hạ Wave B/C như Wave A** — verify-setup/canary phải build+chạy TRƯỚC khi vào doctrine, kẻo lặp dormant v2.2 (cơ chế trên giấy, răng chưa cắm) mà retro này sinh ra để chẩn.
- **⛔ Vẫn DỪNG trước ĐƠN THUỐC** — disposition này là KẾ HOẠCH hạ, chưa hạ. Hạ khi: Q-D2 mầm-bệnh xử xong + Q-D5/Q-D7 build+chạy + Sếp final-approve.
