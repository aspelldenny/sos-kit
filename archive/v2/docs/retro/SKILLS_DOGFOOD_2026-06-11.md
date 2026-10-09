# SKILLS DOGFOOD REPORT — 13 generic skills (2026-06-11)

> Sếp yêu cầu: dogfood tất cả skill, xem cái nào hoạt động/hoạt động như nào/tác dụng gì —
> "từ lúc xỉa của gstack về anh không dùng, đúng là build cho có."
> Method: invoke sống khi được (idea), chạy theo ruột SKILL.md trên INPUT THẬT của ngày
> 2026-06-11 cho phần còn lại. Evidence usage: tarot (13/13 skill = 0 invocation thật).

## 3 finding HỆ THỐNG (đắt hơn từng skill)

1. **Kit không tự ăn được skill của mình** — `Skill(retro)` → `Unknown skill`: 12/13 nằm ở
   `skills/` (library source), chỉ `idea` được link vào `.claude/skills/`. Tarot có đủ 13
   trong `.claude/skills/` nhưng 0 lần gọi → chết cả 2 đầu (không register / register mà không gọi).
2. **Sống = có caller không-phải-LLM.** Mọi thứ đang sống ở tarot đều có hook/cron/gate/spawn-contract
   gọi hộ. Banner nhắc tận mặt "/idea" vẫn không ai invoke (model ghi BACKLOG thẳng). n=13.
3. **Skill-content có thể TỐT mà vẫn chết** (retro, idea) — bệnh ở tầng trigger, không phải tầng content.
   Ngược lại content có thể THỐI mà không ai biết vì không ai chạy (plan còn doctrine Web-Project đời v1).

## Bảng 13 skill

| Skill | Chạy được? | Hoạt động như nào (dogfood thật) | Tác dụng so với inline | Verdict |
|---|---|---|---|---|
| **idea** | ✅ invoke sống | 5 bước: load BACKLOG → dedup search → AskUserQuestion section/tag → append format chuẩn + date → confirm | **CÓ thật**: dedup (inline không bao giờ làm) + section do Sếp click (test thật: em đoán Open, Sếp muốn Active — đoán sai thật) | **GIỮ + hook định danh** (UserPromptSubmit idea-smell — đã vào Active) |
| **retro** | ✅ chạy data thật | git log tuần → velocity (52 commits, 7 feat/11 fix) + hotspot (CHANGELOG×29, sos.sh×9) + learnings review | **CÓ**: hotspot analysis inline không ai làm. Phát hiện phụ: `ship learn` 0 record từ thuở khai sinh | **GIỮ nếu gắn cron weekly** (advisory-cron pattern); không cron → park |
| **route** | ✅ chạy input thật | 3 câu phân loại → brief 6 trường | Brief format hơi có giá; NHƯNG 5 lane **thiếu ops/infra** — inbound thật đầu tiên (jarvis PAT) đã không fit | **PARK** (Quản đốc route inline ổn; lane taxonomy lỗi thời) |
| **decide** | ✅ đối chiếu live | options + user-impact + recommend + AskUserQuestion | Spec = ĐÚNG những gì Quản đốc đã làm inline (vụ private-repos hôm nay khớp từng bước) → ruột đã absorbed | **PARK** (absorbed vào hành vi orchestrator) |
| **insight** | 🟡 đọc + thử khô | distill raw → bullets cho PROJECT/SOUL/CHARACTER | Hợp repo PRODUCT có vision docs (tarot); kit repos không có SOUL/CHARACTER. 0 usage kể cả tarot | **PARK** (hồi sinh nếu quay lại product-vision work) |
| **plan** | ⛔ content THỐI | — | SKILL.md còn "Kiến trúc sư lives in Claude Web Project, no shell" = doctrine v1 ĐÃ BỎ. Ruột thật sống trong `agents/architect.md` (13 chỗ Task 0/TICKET) | **PARK** (stale + absorbed) |
| **verify** | 🟡 absorbed | Task 0 grep-first | `agents/worker.md` inline ×14 — Thợ làm native. ⚠️ Trùng tên skill `verify` built-in của Claude Code (name collision) | **PARK** (absorbed + collision) |
| **qa** | 🟡 đọc | test app → find bug → fix → prove | Tham chiếu "V8 pipeline" (gstack đời cũ). Trùng vai EXECUTE-test của Thợ + Giám sát | **PARK** |
| **review** | 🟡 đọc | staff-engineer review pre-merge | Vai này Giám sát `/security-review` đã chiếm (sống vì gate đòi); generic review 0 caller | **PARK** |
| **ship** | 🟡 đọc | wrapper quanh `ship` CLI | Binary `ship` sống (Tier 1, ship_canary tarot dùng); SKILL chỉ là prose mỏng quanh CLI | **PARK** (binary giữ, skill bỏ) |
| **init** | 🟡 chưa tới phiên | Phase-0 vision capture, 3 câu hỏi → PROJECT/SOUL/CHARACTER | Caller cơ học CÓ (`sos init` in "run skill /init"). n=0 vì chưa repo nào chạy genesis từ kit | **GIỮ — genesis trio** (label: unproven n=0) |
| **apply** | 🟡 chưa tới phiên | execute 1 recipe end-to-end | Caller: `sos apply`. n=0 cùng lý do | **GIỮ — genesis trio** |
| **forge** | 🟡 chưa tới phiên | research → viết recipe mới vào library | Caller: `sos recipe new`. Ứng viên dogfood thật đầu tiên: recipe `rust/golden-oracle-port` (inv-gate cần) | **GIỮ — genesis trio** (dogfood khi forge recipe đó) |

## Tổng: GIỮ 4 (idea + genesis trio) · GIỮ-CÓ-ĐIỀU-KIỆN 1 (retro nếu gắn cron) · PARK 8

**Luật rút ra (đề xuất vào LAYERS):** skill vào kit phải khai **caller cơ học** (hook/cron/CLI/gate/handbook-contract)
ngay frontmatter — `caller:` field. Không caller = không nhận. Content tốt không cứu được skill mồ côi.

## Việc làm sau khi Sếp duyệt báo cáo
1. `skills/attic/` + move 8-9 con park (giữ README lý do + điều kiện hồi sinh).
2. 4-5 con giữ: thêm `caller:` field + register vào `.claude/skills/` của chính sos-kit (fix finding #1).
3. UserPromptSubmit idea-smell hook (Active item, đã click).
4. retro: Sếp quyết có gắn cron weekly không → quyết sống/park.
5. README + LAYERS + CLAUDE.md ("13 skills" → mới) + `sos new/adopt/sync` exclude attic.
