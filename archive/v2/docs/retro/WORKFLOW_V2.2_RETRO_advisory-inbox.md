# Retrospective — advisory-inbox pilot (Workflow v2.1 → v2.2)

> **Status:** **CLOSED 2026-05-28** — materialized into `~/sos-kit/docs/WORKFLOW_V2.2.md`. KHÔNG edit sau khi close — viết retro mới cho vòng 2.
> **Pilot:** advisory-inbox — 13 phiếu, 14 PR, Rust, oracle SOUND
> **Mục đích:** v2.1 trả lời "có an toàn / có trigger / có durable chưa". v2.2 phải trả lời "có RẺ hơn chưa, và có chống mù khi oracle PARTIAL chưa".
> **Nguyên tắc retro:** tách CHẨN ĐOÁN (lỗi + nguyên nhân) khỏi ĐƠN THUỐC (cơ chế sửa). Mỗi fix phải gắn cờ `[gate]` / `[hook]` / `[guidance]` — KHÔNG fix nào được là prose để agent nhớ.

---

## §0. Một câu tổng kết

v2.1 chạy đúng như thiết kế: hook chặn unsafe merge đúng chỗ (Sub-mech A precedent reinforced), taxonomy objection bắt lỗi thật, debate cycle có giá trị (P003/P011/P013 đều catch lỗi thật pre-EXECUTE). Ship sạch: 69 test, zero clippy, zero unsafe.

NHƯNG v2.1 mới tối ưu "quy trình đúng", chưa tối ưu "mức tải thực tế". Lane phân loại đúng nhưng **không thực sự cắt tải** — mọi phiếu chạy gần như cùng một con đường nặng. Và pilot chạy trên Rust (oracle SOUND), nên mảng nguy hiểm nhất — chống mù khi oracle PARTIAL — CHƯA được test.

---

## §1. CHẨN ĐOÁN — 6 lỗi quan sát từ 13 phiếu

### D1. Lane phân loại đúng nhưng không cắt tải

**Triệu chứng:** P003 ghi Normal lane (pure parser, no I/O, no schema, no secret, no network), nhưng vẫn chạy full Architect DRAFT → Worker CHALLENGE → Architect RESPOND → Worker SURGICAL Turn 2 → EXECUTE. Phiếu dài 643 dòng cho một file 6 test. Bảng tổng kết: 8/13 phiếu Guarded — workflow thực tế biến gần như mọi thứ thành guarded.

**Nguyên nhân:** classifier output ra `risk_lane` nhưng lane KHÔNG thực sự thay đổi lượng công agent bỏ ra. Không có ràng buộc cứng nào nối "lane = X" với "architect được đọc bao nhiêu / phiếu dài bao nhiêu / có RESPOND cycle không".

### D2. Oracle-resolvable objection vẫn bị route qua debate LLM

**Triệu chứng:** ba objection đắt nhất sprint đều là thứ compiler/`--help` phán tức thì:
- P003 O1.1: `str::find` vs `regex` → `cargo check` phán.
- P011 O1.1: import path `Parameters`/`Json` sai → `cargo build` phán ngay.
- P013 O1.2: `--report -` có nhận stdin không → `--help` phán trong 30s (Worker tự ghi nhận điều này trong discovery).

Cả ba vẫn đi qua một vòng Architect RESPOND tốn 50-100k token.

**Nguyên nhân:** taxonomy v2.1 phân `mechanical/shape/design` nhưng routing đối xử cả ba gần như nhau. Thiếu trục thứ hai: "có oracle deterministic phán được không". Có oracle rẻ nhất thế giới (rustc) ngồi sẵn mà vẫn route qua LLM trước.

### D3. Discovery Report bắt điền full ma trận Sub-mech A-F kể cả khi N/A

**Triệu chứng:** mỗi Discovery Report có bảng "Sub-mech A-F Verification Trace", một nửa số dòng là "N/A, không áp dụng" (P003: A N/A, C N/A, F N/A).

**Nguyên nhân:** di sản của doctrine-heavy CLAUDE.md. Agent đang "chứng minh đã nhớ catalog" thay vì để hook tự nổ đúng cái liên quan.

### D4. Architect đọc quá rộng (token cháy)

**Triệu chứng:** P004 draft 113.9k token, P005 109.5k, P006 121.7k — mỗi draft 6-8 phút, cho một repo Rust nhỏ. P006 worker 213k token / 1h40m cho một subcmd append.

**Nguyên nhân:** architect không có ràng buộc cứng về scope đọc. Sequential-thinking dump P004 cho thấy nó deliberate từng enum variant, đọc source, plan mọi nhánh lỗi — cho một subcmd parse. (Lưu ý: trên repo NHỎ như advisory-inbox đây là full-read tự nhiên; trên repo docs-giàu như tarot nó thành thảm họa — đây là lý do AGENT_MAP cần cho repo lớn.)

### D5. CHƯA test mảng partial-oracle (gap lớn nhất, không phải lỗi đã xảy ra)

**Triệu chứng:** không có triệu chứng — vì pilot là Rust, rustc luôn vớt. Mọi lúc architect đọc thiếu mà spec sai shape, compiler bắt ở worker. Oracle SOUND làm tấm lưới dưới đáy.

**Nguyên nhân + rủi ro:** sang Python/Next.js, oracle PARTIAL — `tsc`/`pytest`/`next build` xanh KHÔNG có nghĩa đúng (tarot Sub-mech B #7: test PASS 803/803 nhưng next build FAIL; Sub-mech E: local tsc pass, VPS fresh install fail). Tấm lưới thủng. Toàn bộ v2.1 + "oracle-first" của v2.2 CHƯA được chứng minh chống mù khi không có rustc. Đây là mảnh thiếu trong bằng chứng.

### D6. Operational state cầm tay (rò "bắt LLM nhớ" dạng vận hành)

**Triệu chứng:** `echo "5" > .phieu-counter` gõ tay số kế tiếp; `touch/rm .sos-state/architect-active` orchestrator tự cầm vòng đời bằng tay. Quên `rm` một lần = kẹt state. Bonus finding: DISCOVERIES.md tarot 265KB, over soft-cap 1000 dòng ~5x — rule rotate là prose trong CLAUDE.md, KHÔNG enforce, đã quên đúng như lý thuyết.

**Nguyên nhân:** rule rotate + lifecycle state là prose để agent nhớ, không phải cơ chế tự chạy.

---

## §2. ĐƠN THUỐC — v2.2, mỗi mục gắn cờ cơ chế

> **Luật vàng 1:** **không fix nào được là prose để agent nhớ.** Mỗi mục là `[gate]` (chặn trên exit≠0), `[hook]` (one-liner nổ đúng lúc), hoặc `[guidance]` (judgment thật, không cơ chế hóa được — giữ tối giản).
>
> **Luật vàng 2 (round 4b — canary 2 reinforce):** **Một bệnh, một cơ chế rẻ nhất bắt 80% ca.** Cấm 3 tầng gate+hook+guidance cho một bệnh. Pattern thấy-phát-hiện → mọc-cơ-chế là phản xạ over-engineer (Orchestrator vi phạm 3 lần trong forge này — xem §7g).

### Fix cho D1 — Lane budgets `[gate]`

Nối lane với ràng buộc CỨNG, kiểm được:

```
Normal lane:
- phiếu ≤ 250 dòng (orchestrator approve mới vượt)
- ≤ 5 anchors, ≤ 5 hard constraints
- KHÔNG full Sub-mech matrix, KHÔNG full provenance
- chỉ surface docs (qua AGENT_MAP), KHÔNG full doctrine reread
- KHÔNG full RESPOND cycle nếu objection oracle-resolvable (xem D2)

Guarded lane: mới được dài + full RESPOND.
```

Cơ chế: `[gate]` — một check đếm dòng phiếu + đếm anchor, chạy pre-CHALLENGE. Phiếu Normal vượt budget → block. (P003 đáng lẽ chạy ~150 dòng + skip RESPOND, không phải 643 dòng + full cycle.)

**Override phải ĐAU (round 3 reviewer chỉnh — ChatGPT đề xuất structured override, reviewer kéo lại):** override với `reason` tự khai do orchestrator tự duyệt = TÁI TẠO đúng bệnh D1. Lane v2.1 cũng là "phân loại rồi cho đi đường tương ứng" và đã trượt về guarded vì không gì cản sự "thôi cho kỹ hơn tí". Override lý-do-tự-do sẽ trượt y hệt: gặp phiếu hơi dài, gõ một dòng reason, qua → 6 tháng sau 60% phiếu có override, budget thành nhãn dán.
Cứu bằng cách làm override tốn phí thật, chọn 1:
- (a) override do **Chủ nhà duyệt**, KHÔNG phải orchestrator tự duyệt → đắt về phải-hỏi-người → agent chỉ xài khi thật cần; HOẶC
- (b) override-rate có **ngưỡng CỨNG**: vượt 20% trong 50 PR gần nhất → **HARD-FAIL cả sprint, buộc dừng** (KHÔNG phải "tune classifier" nhẹ nhàng — phải có phanh cứng, nếu không drift từ từ).

### Fix cho D2 — Oracle-first routing `[guidance]` + `[gate]`

Thêm trục oracle vào taxonomy. Worker CHALLENGE hỏi 2 câu, KHÔNG phải 1:
1. Loại objection? (mechanical / shape / design)
2. **Oracle nào phán được, và SOUND hay PARTIAL?**

Routing mới:

```
[mechanical + oracle SOUND]   → Worker verify/fix trực tiếp, log patch note. KHÔNG gọi Architect.
[shape + oracle SOUND]        → Worker chạy compile/probe, fix tại chỗ nếu local. KHÔNG gọi Architect.
[shape + oracle PARTIAL]      → Worker chạy oracle như SÀNG, phần chạm contract cần contract-test HOẶC Architect short.
[shape + architecture-impact] → Architect short respond.
[design / security]           → Architect full respond.
```

Oracle SOUND (phán pass = chắc chắn đúng): rustc/cargo check, JSON schema validator, grep exact line, `--help` về sự tồn tại của flag.
Oracle PARTIAL (pass ≠ đúng): pytest, `tsc` (any nuốt lỗi), `next build` local, mypy non-strict.

**Oracle đóng CLAIM, KHÔNG đóng code (round 3 — ChatGPT patch, reviewer chấp nhận + tự sửa ví dụ ẩu của round 1):** điều kiện để bỏ debate KHÔNG phải "có oracle nào chạy được", mà "oracle có phán đúng CÁI CLAIM của objection không". Reviewer round 1 dùng P003 `str::find` vs `regex` làm ví dụ "compiler phán được" là SAI. `cargo check` chỉ phán "str::find code compile không / regex code compile không". Nó KHÔNG phán "BACKLOG viết chữ regex thì có buộc dùng regex crate không" — đó là docs/contract ambiguity, compiler CÂM. Đây cùng họ với bẫy SOUND/PARTIAL nhưng tinh hơn: oracle trả lời câu A trong khi objection hỏi câu B.

Phân loại lại 3 ví dụ sprint cho đúng:
```
P011 import path Parameters sai  → CLAIM = "path này tồn tại không" → cargo check ĐÓNG ĐƯỢC (SOUND).
P013 --report - có nhận stdin    → CLAIM = "flag behavior thế nào" → --help/smoke ĐÓNG ĐƯỢC (SOUND).
P003 str::find vs regex          → CLAIM = "docs wording có buộc regex crate" → compiler KHÔNG đóng → cần docs-precedence rule HOẶC Architect short.
```

**Worker BẮT BUỘC ghi 3 thứ trước khi bỏ debate (ChatGPT đề xuất, lấy nguyên văn):** *"Không phải oracle nào cũng trả lời cùng một câu hỏi. Trước khi bỏ debate, Worker phải ghi rõ: claim là gì, oracle nào phán claim đó, và oracle đó SOUND hay PARTIAL cho claim này."* Thiếu 3 thứ này = không được tự đóng objection.

Cơ chế: phần "worker tự xử khi oracle-đóng-được-claim + SOUND" là `[guidance]` trong worker.md. Phần "Normal lane skip RESPOND nếu oracle-resolvable" là `[gate]` (nối D1 budget).

**Câu chốt doctrine:** *Compiler/--help/schema phán được thì đừng cho LLM debate. Nhưng phải biết oracle nào SOUND oracle nào PARTIAL — PARTIAL chỉ là vòng sàng, không phải vòng quyết.*

### Fix cho D3 — Sparse Discovery Report `[guidance]`

Discovery chỉ ghi sub-mech ĐÃ FIRED hoặc có finding. Không ghi full A-F matrix với N/A.

```
Sub-mech fired:
- B: cargo check/test/clippy/fmt pass
- D: CLAUDE.md doctrine sync regex→str::find
Not fired: none required by classifier
```

Cơ chế: `[guidance]` ngắn trong worker.md. Khi 6 sub-mech thành hook (xem dưới), bảng N/A tự chết — hook liên quan thì nổ, không thì im, agent không cần điền "N/A" để chứng minh đã nhớ.

### Fix cho D4 — AGENT_MAP cho repo docs-giàu `[gate]` + `[guidance]`

KHÔNG cần cho repo nhỏ (advisory-inbox < 10 docs → grep convention đủ). CẦN cho repo > 10 docs (tarot 1.8MB).

Map shape (vào sos-kit golden template, không ruột tarot):
- `surface → {edit, read_shallow / read_deep, load_bearing, blast, contract_test}`
- tách `read_shallow` vs `read_deep` trên cùng surface (sửa case "đổi 1 dòng copy mà đọc 327KB")
- `never_default_read:` cho CHANGELOG / DISCOVERIES / BACKLOG / Archive (log + idea, KHÔNG nạp mặc định)
- dòng `blast` = "đổi cái này gãy đâu" → cho architect quyền DỪNG đọc khi đã thấy hết vùng nổ

Cơ chế: map là `[guidance]` cho architect (đọc gì tới đâu). NHƯNG `validate-map` là `[gate]` BẮT BUỘC — mọi path tồn tại (`test -e`), mọi anchor `#section` còn — chạy pre-commit. **Map không validator = thuốc độc chậm** (xem DISCOVERIES 265KB: rule không enforce = drift). Map drift còn tệ hơn không map vì nó chủ động nói architect "leaf, đọc nông" trong khi load-bearing, và partial-oracle không vớt.

**Mini-map trong pilot — ranh giới (round 3, ChatGPT patch 4 + reviewer ghim ranh):** repo Python pilot dù nhỏ vẫn nên có mini-map 3 surface (vd `api_boundary` / `state_file` / `runtime_env`) ĐỂ test cơ chế. NHƯNG ghim rõ nó test được gì: mini-map chỉ validate được **VALIDATOR MECHANISM** (path tồn tại, anchor còn). Nó KHÔNG validate được **giá trị blast-radius / load-bearing flag** — vì repo greenfield thì surface là BỊA (giống fixture tự chế ở M1). Validator kiểm map có nói dối về SỰ TỒN TẠI, KHÔNG kiểm map có nói dối về TẦM QUAN TRỌNG. Cái sau chỉ kiểm được trên repo có lịch sử thật (tarot). Đừng tưởng pilot đã validate cả map — mới validate cái vỏ.

### Fix cho D5 — Repo test tiếp PHẢI partial-oracle `[process]`

v2.2 chỉ được coi là "đã chứng minh" khi chạy end-to-end trên một repo PARTIAL-oracle (Python hoặc TS nhỏ, có thật, mày thật sự cần — vd metrics-sync bằng Python). Rust thêm chỉ lặp cái đã biết.

Bổ sung oracle theo stack vào sos-kit:
- Rust: rustc/clippy = SOUND cho shape.
- Next.js: oracle THẬT = `pnpm install --frozen-lockfile && next build` trong env sạch (đóng hộp Sub-mech E). `pnpm test` một mình KHÔNG đủ.
- TypeScript: `tsc --noEmit` strict; coi `any` là vùng oracle MÙ → objection chạm `any` = không oracle-resolvable.
- Python: oracle yếu nhất; `mypy --strict` (nếu có type) + pytest; mặc định PARTIAL, nâng ngưỡng "cần con mắt rộng".

Đi kèm: **edit-scope ≠ verify-scope** `[gate]` trong ticket template:
```
Files allowed to EDIT: [hẹp — chống ngứa nghề]
Files required to READ for verify: [rộng theo boundary — chống mù]
Contract tests/checks: [chạm thật qua boundary, không mock]
```
Worker edit hẹp, verify theo boundary. KHÔNG "chỉ nhìn file trong scope" (= sờ chân voi).

### Fix cho D6 — State tự chạy `[hook]`

- `.phieu-counter`: dùng hàm `phieu` tăng nguyên tử, KHÔNG gõ tay. (CLAUDE.md nói hàm tồn tại nhưng pilot vẫn gõ tay — `[hook]`/wrapper ép dùng.)
- `.sos-state/architect-active`: chính agent architect tự touch lúc start, tự rm lúc end. **NHƯNG (round 3, ChatGPT patch 5) "tự rm lúc end" KHÔNG đủ — architect crash thì không bao giờ chạy tới "lúc end", lock kẹt vĩnh viễn.** Lock phải chứa `pid + started_at + ticket_id + ttl_minutes`. Session-start/orchestrator check: lock tồn tại + (pid chết HOẶC ttl hết) → stale, dọn CÓ LOG; lock tồn tại + pid sống → block architect mới. Đừng dựa hoàn toàn vào "agent tự cleanup".
- Rotate cap: pre-commit `[hook]` đếm dòng DISCOVERIES/CHANGELOG, vượt soft-cap → warn/block, KHÔNG dựa prose "nhớ rotate".

### Fix nền — 6 Sub-mech: prose → hook `[hook]`

Toàn bộ "Layer 2 capability check" trong CLAUDE.md (hiện là prose để agent đọc rồi nhớ chạy) → cơ chế chạy:
- A (trigger gap): `grep "if: *false"`, `[[ -x hook ]]`, block merge chưa approve → đã có `block-unsafe-merge.sh`. HOOK.
- B (capability): grep export route.ts → pre-commit hook; `next build` sau dep bump → CI gate.
- C (migration): `jq length` vs `wc -l` → migration script tự check.
- D (persistence): commit chạm doctrine file mà message thiếu `home:` → pre-commit block.
- E (env drift): `install --frozen-lockfile && build` env sạch → CI gate.
- F (runtime state): scan `.git/config` token → SessionStart + pre-commit hook.

Số sub-mech cần "tool mới đúng nghĩa" ≈ 0. Phần lớn là one-liner trong hook, hoặc "chạy build/test thật" làm gate. Logic cần parse/count/classify (test count, LOC, touched-surface, pr-diff capture) → dồn vào MỘT `doctor` binary nhiều subcmd, KHÔNG tám binary. Tất cả sống trong sos-kit golden template.

**Ranh giới tuyệt đối:** chỉ MECHANICAL mới gate được. Judgment (INV-101..108, câu hỏi vàng, "có over-engineer không") KHÔNG grep được → ở lại `[guidance]` (boundary-check advisory mode). Đừng ép judgment thành hook giả → phán bừa.

### Fix N1 — Boundary-check rubric injection `[hook]` (canary 1+2 graduated, round 4b)

**Bệnh:** 5-INV generic rubric mù với project-specific INV-LOCAL-*. Canary 1 (no inject): subagent miss INV-LOCAL-002 atomic write degrade — chính INV subagent vừa verify clean ở P006 1 sprint trước. Canary 2 (inject INV-LOCAL-* vào prompt): catch chính xác với reasoning sâu (userspace buffer vs fsync syscall, kernel reorder across crash).

**Cơ chế — MỘT hook, hết:**

```
Skill /security-review (hoặc orchestrator pre-spawn) BẮT BUỘC:
  1. Read docs/security/INVARIANTS.md
  2. Extract block INV-LOCAL-*
  3. Paste verbatim vào prompt cho boundary-check
```

Đếm dòng code fix < 20.

**KHÔNG làm:**
- ❌ `[guidance]` "boundary-check tự grep INVARIANTS.md" = prose để nhớ.
- ❌ `[gate]` "verdict validation grep đủ INV expected" = sân khấu.

**Doctrine (canary 2 refined):** *Subagent đọc semantic được nếu được CHỈ phải canh. Một bệnh, một cơ chế.*

---

## §3. Thứ tự thực thi (4 nhịp + Nhịp 2.5 verify safety)

```
Nhịp 1 — Retro (CHẨN ĐOÁN)              ← FILE NÀY, CLOSED 2026-05-28
Nhịp 2 — v2.2 doctrine (ĐƠN THUỐC)       → ~/sos-kit/docs/WORKFLOW_V2.2.md (SHIPPED)
Nhịp 2.5 — Verify safety nền (parallel với 2):
           • N1 canary 2-PR advisory-inbox  ✅ DONE 2026-05-28 (rubric injection finding)
           • N3 fix block-unsafe-merge cross-repo flag (bug đã lộ, ship trong nhịp 3)
Nhịp 3 — Hạ v2.2 vào sos-kit golden template
         (agents/*, recipes, hooks, doctor binary, AGENT_MAP shape + validate-map,
          arm M1-M6 sensor, N2 token cap)
Nhịp 4 — Mở repo Python/TS THẬT từ template → end-to-end
         (thí nghiệm có giả thuyết: "v2.2 chống mù khi oracle PARTIAL?")
         → phơi lỗi mới → retro mới → v2.3 → lại hạ sos-kit (vòng lặp)
```

**Đừng nuốt nhịp 3.** Copy v2.2 thẳng sang repo mới = bỏ qua sos-kit = per-repo cost quay lại. Template là nơi doctrine sống một lần; repo mới chỉ bootstrap từ nó.

advisory-inbox = vòng 1 (test trên SOUND-oracle, đã xong). Repo Python = vòng 2 (test trên PARTIAL-oracle, mảnh thiếu). sos-kit tích lũy qua mỗi vòng — đó mới là "áp ngược sos-kit" đúng nghĩa: không một lần áp, mà mỗi vòng bồi một lớp.

---

## §6. Watchlist — vùng pilot KHÔNG dạy được (round 2 + reviewer chỉnh)

> **Round 2 (orchestrator advisory-inbox)** đóng góp lớn nhất: biến D5 (một dòng "chưa test partial-oracle") thành bản đồ các vùng v2.1 chưa test vì pilot quá thuận lợi (solo, greenfield, no legacy data, no parallel, no production, linear backlog). Đây là loại chẩn đoán thứ hai — KHÔNG phải "lỗi đã thấy" mà "unknown ta biết mình không biết".
>
> **Reviewer chỉnh 3 điểm trước khi đóng băng** (kẻo v2.2 phình hơn v2.1 đang cố làm nhẹ):
>
> 1. **Mỗi mảng chỉ MỘT cơ chế rẻ nhất bắt 80% ca.** Round 2 có xu hướng kê đủ gate+hook+guidance mỗi mảng — đó là phản xạ "thấy lỗi thì thêm cơ chế", chính bệnh làm v2.1 phình rồi quên. Cấm ba tầng cho một bệnh.
> 2. **M1-M6 KHÔNG fix-trước. Vào v2.2 dạng CẢM BIẾN (sensor), không phải hệ phòng thủ.** Fix trước = over-engineer dựa giả định (vi phạm "câu hỏi vàng": giải vấn đề ĐANG có hay GIẢ ĐỊNH có). Arm sensor rẻ → Python pilot phơi M nào NỔ thật → M đó mới lên doctrine v2.3. M nào cả pilot không nổ = giả định, vứt miễn phí.
> 3. **N1 nâng từ "thấp" lên CAO + làm canary NGAY.** N1 nghi chính nền safety là sân khấu (boundary-check 13/13 APPROVE, có thể pass-through). Con gác chưa bắt ai thì không biết giỏi hay ngủ.

### Watchlist — mỗi mảng MỘT sensor rẻ nhất (không fix đắt)

| ID | Vùng chưa test | Sensor (1 cơ chế rẻ nhất) | Cờ | Khi nào lên doctrine |
|----|----------------|---------------------------|-----|----------------------|
| **N1** ✅ | ~~Subagent verdict = sân khấu?~~ **GRADUATED 2026-05-28** | Canary 1+2 chạy trên advisory-inbox PR #14. Finding: subagent đọc semantic OK, rubric mù INV-LOCAL-*. Fix [hook] inject INV-LOCAL-* vào prompt — xem §2 cuối. | `[hook]` shipped | Graduated khỏi watchlist → §2 fix |
| **M1** | Real legacy data ≠ fixture tự chế (P013 hit format thứ 4) | `[hook]` migration phiếu thiếu file snapshot trong `fixtures/` từ real export → block | `[hook]` | Khi Python repo hit format ngoài spec |
| **M2** | Branch stale / rebase mid-flight | `[hook]` 1 dòng `git merge-base --is-ancestor origin/main HEAD` pre-EXECUTE → not ancestor = block | `[hook]` | Khi conflict thật xảy ra |
| **M3** | NEEDS_REVIEW path chưa chạy lần nào | `[hook]` verdict NEEDS_REVIEW → orchestrator AskUserQuestion, KHÔNG tự skip dù autonomous | `[hook]` | Khi verdict thật xuất hiện |
| **M6** | Counter/marker race (team) | `[hook]` counter dùng `mkdir` atomic thay `echo N >`; architect-active PID-tagged | `[hook]` | Khi 2 agent push parallel |
| **M4** | Hotfix lane / interrupt mid-sprint | `[guidance]` thêm Hotfix lane scope cứng (prod-down/security/user-blocking), security-review POST-merge | `[guidance]` | Khi interrupt thật |
| **M5** | CI flake / retry policy | `[guidance]` max 2 retry + 1-line flake reason, >2 = bug thật return Worker | `[guidance]` | Khi CI flake thật |
| **N2** | Token cháy (P006 213k) | `[gate]` token cap/subagent theo lane (Fast 30k / Normal 80k / Guarded 150k — số gợi ý, tune ở pilot) → vượt = AskUserQuestion "scope creep?" | `[gate]` | Cap ngay (nối D1 lane budget) |
| **N3** | Hook cross-repo fail (P013 hit thật) | `[hook]` block-unsafe-merge detect `-R <owner>/<repo>` flag → invoke gh đúng repo | `[hook]` | **Bug đã lộ — fix luôn**, không phải watchlist |

### Phân biệt cốt lõi (mang đi round 3)

**Fix ≠ Arm a sensor.** Fix M2 = viết cả policy rebase conflict khi chưa gặp conflict nào (over-engineer giả định). Arm M2 = cài đúng 1 dòng is-ancestor để khi stale THẬT thì nổ + mày THẤY, rồi fix dựa ca thật.

Việc của v2.2 KHÔNG phải fix hết watchlist. Là cài cảm biến để Python pilot DẠY cái nào là thật. M nổ → doctrine. M im → giả định, vứt.

### Trước khi mở Python repo

1. ~~N1 canary NGAY~~ ✅ **DONE 2026-05-28** — graduated, [hook] inject ship trong v2.2 §8.
2. **N3 fix** — bug đã lộ thật, ship nhịp 3.
3. **N2 token cap** — nối D1 lane budget, ship nhịp 3.
4. M1-M6 còn lại: chỉ ARM sensor, KHÔNG fix. Để pilot phơi.

**Câu chốt round 2:** Pilot chứng minh "v2.1 đúng khi mọi thứ tuyến tính + oracle SOUND". CHƯA chứng minh "sống được khi data lằng nhằng, branch stale, verdict thật needs-review, team push parallel". Bốn cái sau mới là môi trường thật — và là lý do Python pilot phải có data thật + (lý tưởng) không phải greenfield.

---

## §4. Provenance (forge log)

| Round | Tác giả | Output |
|-------|---------|--------|
| 1 | Claude Web (reviewer) | 6 chẩn đoán (§1) + đơn thuốc gắn cờ cơ chế (§2) |
| 2 | Orchestrator advisory-inbox | §6 watchlist — bản đồ vùng chưa test (M1-M6 + N1-N3) |
| 2b | Claude Web (reviewer) | Chỉnh §6: 1-sensor-rẻ-nhất, arm-not-fix, N1 nâng cao + canary ngay |
| 3 | ChatGPT | 5 patch: oracle-đóng-claim, structured override, blind canary, mini-map pilot, lock PID/TTL. Chấm 9/8.5/9/8. |
| 3b | Claude Web (reviewer) | Nhận 4.5/5 patch. Chỉnh: override phải ĐAU (Chủ-nhà-duyệt HOẶC hard-fail ngưỡng, không tự-khai); canary 2-PR (FLAG + APPROVE); mini-map chỉ test validator-mechanism KHÔNG test blast-value. |
| 4 | Orchestrator tarot | Thứ tự nhịp unified (2.5 verify safety), doctor=Rust, edit/verify asymmetric, câu §5.3/5/6 closed. |
| 4b | Sếp + Orchestrator tarot | Canary 1+2 chạy 2026-05-28. N1-Fix cắt 3 tầng → 1 hook. Luật vàng 2 "một bệnh một cơ chế" thêm vào §2. |

---

## §5. Câu hỏi mở — closed status

1. ~~Lane budget 250 dòng — cứng hay flex?~~ **Defer Python pilot tune.**
2. ~~Oracle SOUND/PARTIAL table — stack ở giữa?~~ **Defer Python pilot — Go/Java khi gặp.**
3. ~~`doctor` binary — Rust hay Python?~~ **CLOSED round 4 — Rust** (pattern "1 binary 1 repo" precedent + portable).
4. ~~AGENT_MAP validate-map — load_bearing khớp caller?~~ **Defer round 3b — chỉ check path/anchor; pilot validate validator-mechanism, không blast-value.**
5. ~~edit-scope vs verify-scope enforce?~~ **CLOSED round 4 — asymmetric: edit [gate], verify [guidance].**
6. ~~Mảng pilot không test?~~ **CLOSED — §6 watchlist cover.**

---

## §7. Round 4 — Orchestrator tarot nuance + canary 1+2 finding

### a) §3 thứ tự nhịp unified
Thêm Nhịp 2.5 (verify safety parallel) vào §3. Canary N1 không sequential trước v2.2 spec — parallel.

### b) Câu §5.3 closed — doctor binary RUST
Pattern "1 binary 1 repo" precedent (6 binary đã ship: vps/ship/guard/quality-gate/advisory-cron/advisory-inbox) + portable thắng Python native-stack. Đặt `~/doctor/`.

### c) Câu §5.5 closed — edit/verify asymmetric
`edit_allow` [gate] (mechanical, grep được từ git diff), `verify_read` [guidance] (judgment, không grep được "agent đã đọc đủ"). Application của Luật vàng 3 "mechanical mới gate".

### d) Câu §5.6 closed — §6 watchlist đã cover

### e) **N1 canary 1+2 — finding decisive (2026-05-28)**

**Canary 1 (blind, no inject INV-LOCAL):**
- ✅ INV-1 env var no template — CAUGHT (subagent đọc diff thật, không pass-through)
- ✅ Bonus eprintln DoD — CAUGHT (subagent đọc CLAUDE.md ngoài 5 INV rubric)
- ❌ INV-LOCAL-002 atomic write degrade (`sync_all()` → `flush()`) — MISSED, chính INV mà cùng subagent vừa verify clean ở P006 1 sprint trước.

**Em giả thuyết sai:** "subagent ngủ pass-through default". Subagent THẬT SỰ đọc diff.

**Vấn đề thật:** rubric design gap — 5 generic INV không có slot cho INV-LOCAL-*.

**Canary 2 (inject INV-LOCAL-* vào prompt, cùng diff):**
- ✅ INV-LOCAL-002 — CAUGHT với reasoning sâu (userspace buffer vs fsync syscall, kernel reorder across crash boundary).

**Phân biệt chẩn đoán:**

| Chẩn đoán | Test | Result |
|-----------|------|--------|
| A — rubric thiếu slot (auto-inject CỨU) | Canary 2 inject + same diff | ✅ ĐÚNG |
| B — gác không hiểu sâu (auto-inject VÔ DỤNG) | Canary 2 | ❌ REFUTED |

→ Subagent đọc semantic được nếu được CHỈ phải canh; không đọc nếu không biết phải canh.

### f) **N1-Fix v2.2 — final, một tầng**

Em đề xuất sai 3 tầng [gate+hook+guidance] sau canary 1. Sếp cắt về 1 tầng [hook] sau canary 2. Reasoning:
- [guidance] (bắt subagent tự grep INVARIANTS.md) = prose để nhớ — chính bệnh v2.2 chữa.
- [gate] (verdict validation grep đủ INV) = sân khấu — bắt subagent "đề cập đủ vai".
- [hook] (orchestrator pre-spawn inject) = cơ chế rẻ nhất, < 20 dòng code.

**Một bệnh, một cơ chế.** Doctrine v2.2 reflect (xem §2 cuối + WORKFLOW_V2.2.md §8).

### g) **Em sai 3 lần — kỷ luật meta**

Sếp catch em 3 lần cùng 1 reflex:
1. §6 đầu (round 2): chia 6 mảng + 3 phụ → quá rộng (Claude Web 2b kéo gọn → watchlist + arm sensor)
2. N1-Fix sau canary 1: đề xuất [guidance] + [hook] + [gate] → 3 tầng cho 1 bệnh
3. (Suýt) đóng băng kết luận "subagent ổn, rubric mù" làm fact từ 1 điểm dữ liệu

**Pattern:** thấy phát hiện → mọc cơ chế phòng thủ. Vi phạm "một bệnh một cơ chế" + "câu hỏi vàng" (giải vấn đề ĐANG có hay GIẢ ĐỊNH có).

**Lesson durable cho v2.2:** Luật vàng 2 thêm vào §2 doctrine — *"Một bệnh, một cơ chế rẻ nhất bắt 80%."* Cấm 3 tầng cho 1 bệnh.

### h) Chưa chứng minh — giữ watchlist

- Subagent behavior trên diff lớn (500+ dòng) — canary 1, 2 nhỏ (~31 dòng).
- Subagent behavior trên vi phạm tinh vi hơn `sync_all → flush` (race condition async, off-by-one validation logic).
- Subagent consistency — 2 spawn cùng diff verdict identical không?

Arm-sensor trên Python pilot, KHÔNG fix preemptive.

---

## §8. Status: **CLOSED 2026-05-28**

Retro materialized into `~/sos-kit/docs/WORKFLOW_V2.2.md`. KHÔNG edit retro sau khi close — viết retro mới cho vòng 2 (Python pilot): `WORKFLOW_V2.3_RETRO_<pilot-name>.md`.
