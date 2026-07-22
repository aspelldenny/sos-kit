# PHIẾU P083: doc-drift cleanup (README install + LAYERS v1-residue + tagline overclaim)

> **ID format:** `P083` — assigned manually (out-of-sprint interleave, not via `phieu` counter this session).
> **Filename:** `docs/ticket/P083-doc-drift-cleanup.md`
> **Branch:** `docs/P083-doc-drift-cleanup`

---

> **Loại:** Bugfix (docs)
> **Ưu tiên:** P1
> **Tầng:** 1 — README = "single source of truth" contract surface (CLAUDE.md Rule #4) + `docs/LAYERS.md` = Foundation doc (role-framing contract). Sai/stale ở đây LAN sang mọi Architect phiếu đọc doc → AUTO Tầng 1 dù diff nhỏ. LOC không quyết Tầng.
> **Lane:** Guarded
> **Approval:** Chủ nhà direct approval — Sếp-ratified out-of-sprint 2026-07-22 (KHÔNG trong Active sprint BACKLOG; interleave sau P077b, trước P077c). Nguồn: external kit review, orchestrator verified 3 drift thật với file:line.
> **Ảnh hưởng:** `README.md`, `docs/LAYERS.md`, `docs/PHILOSOPHY.md` (F1 chỉ sau Sếp chốt câu chữ), `phieu/RELAY_PROTOCOL.md` (mode-note), `docs/SETUP.md` (docs-gate mirror)
> **Dependency:** None

---

## Context

### Vấn đề hiện tại

External kit review phát hiện 3 drift thật, orchestrator đã verify với file:line (dưới đây là Task 0 anchors, Worker re-verify):

- **F3 (BREAKING — ưu tiên cao nhất):** `README.md:240-254` "Skills (global)" install block chạy `cp -r skills/{init,insight,route,decide,plan,forge,verify,apply,review,qa,ship,retro}` = **12 skill**. Chỉ **5 LIVING** tồn tại ở `skills/` (`apply` `forge` `idea` `init` `retro`); 8 cái (`insight` `route` `decide` `plan` `verify` `review` `qa` `ship`) đã dời `skills/attic/`. → `cp -r skills/route ...` v.v. **FAIL trên fresh clone** (no such file). Top README (`README.md:127` "5 living") + skill table `README.md:134-142` đã đúng — chỉ install block stale.

- **F2 (doctrine — nặng nhất về ảnh hưởng):** `docs/LAYERS.md:48` "**Critical**: Kiến trúc sư lives in Claude Web Project" + residue rải `LAYERS.md:77,84-85,92,99,115,166,195-198` (human courier giữa Claude Web ↔ Claude Code, RELAY qua Chủ nhà, "separate sessions") = khung **v1 đã chết**. v2 default (CLAUDE.md "v2.1+ Subagent mode"; `agents/architect.md` tools Read/Write/Glob) = Architect là **subagent chạy TRONG Claude Code**, Quản đốc spawn cả Architect+Worker in-session → KHÔNG có human-courier-giữa-web-và-code. LAYERS = Foundation doc → residue này ảnh hưởng lớn.

- **F1 (nhẹ — nhưng USER-FACING TEXT → Chủ nhà chốt câu chữ):** `README.md:3` "Full operating system **from inbound request** to production health" overclaim đầu pipeline; mâu thuẫn `docs/PHILOSOPHY.md:26` "just the **tail of the pipeline** — from code-ready to production-verified" + CLAUDE.md "picks up after code is ready". Đây là câu chữ user nhìn thấy = Tầng 1 **Chủ nhà chốt** (HANDOFF mismatch table). Phiếu đề xuất 2-3 tagline, orchestrator hỏi Sếp ở APPROVAL. KHÔNG tự quyết.

### Giải pháp

- **F3:** Sửa install block `README.md:240-254` → chỉ copy 5 skill tồn tại đúng caller. Vì skills giờ được gọi cơ học (hook/cron/CLI — CLAUDE.md caller law), copy-thủ-công phần lớn thừa; ưu tiên trỏ về caller cơ học + giữ note `/idea` project-local (đã có `README.md:257`). Worker chọn diễn đạt Tầng 2 miễn KHÔNG list skill không tồn tại.
- **F2:** Re-frame LAYERS sang v2 subagent. Điểm cứng phải GIỮ (đừng xoá mù): Architect **vẫn docs-only** (no Bash/Grep/Edit) — đó là **envelope cơ học** (`agents/architect.md`), KHÔNG phải "vì ở Web". Cross-session relay (Thợ ở worktree/session khác) nếu còn hợp lệ → diễn đạt lại là **chế độ v1/cross-session tuỳ chọn**, default v2 = in-session, KHÔNG present như current-mode.
- **F1:** Đợi Sếp chốt tagline (options ở dưới) → sửa `README.md:3` khớp scope PHILOSOPHY.

### Scope

- CHỈ sửa: `README.md` (install block + tagline sau Sếp + relay-section framing), `docs/LAYERS.md` (v1→v2 re-frame), `docs/PHILOSOPHY.md` (verify-only, F1 không sửa PHILOSOPHY — nó đã đúng), `phieu/RELAY_PROTOCOL.md` (mode-clarification note top), `docs/SETUP.md` (docs-gate mirror nếu skill list drift).
- KHÔNG sửa (đã verify KHÔNG phải lỗi):
  - **F4** `/init` name-collision với built-in `/init`: đã có ⚠ caveat chủ đích (CLAUDE.md skills line) — không đụng.
  - **F5** file "missing": `bin/sos.sh` / `core/ROLES.md` / `adapters/claude/MAPPING.md` ĐỀU tồn tại; `README:70` nhãn `sos` "(planned)" trung thực. Không drift — không đụng.
  - KHÔNG đụng runtime/code (repo là meta-kit, docs-only phiếu).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `README.md:240-254` install block copies 12 skills incl. `insight route decide plan verify review qa ship` `[verified]` | `sed -n '238,255p' README.md` | ✅ Architect Read confirmed lines 240-254 |
| 2 | Only 5 skill dirs live at `skills/` (`apply forge idea init retro`); 8 others under `skills/attic/` `[needs Worker verify]` | `ls -d skills/*/ ; ls -d skills/attic/*/` | ⏳ TO VERIFY (docs claim 5 living @ README:127,134-142) |
| 3 | `docs/LAYERS.md:48` = "Kiến trúc sư lives in Claude Web Project"; v1 residue also at `77,84-85,92,99,115,166,195-198` `[verified]` | `grep -n "Claude Web\|human courier\|separate sessions\|Ping Architect" docs/LAYERS.md` | ✅ Architect Read confirmed |
| 4 | `agents/architect.md` grants Read/Write/Glob (envelope = docs-only in-session, NOT "because Web") `[needs Worker verify]` | `grep -n "tools:\|Read\|Write\|Glob\|Bash" agents/architect.md` | ⏳ TO VERIFY (basis for re-frame) |
| 5 | `README.md:3` tagline "from inbound request to production health" vs `docs/PHILOSOPHY.md:26` "tail of the pipeline — from code-ready" `[verified]` | `sed -n '3p' README.md ; sed -n '26p' docs/PHILOSOPHY.md` | ✅ Architect Read confirmed conflict |
| 6 | `INSTALL.md:100-102` only copies `idea` skill (no 12-skill drift) — likely correct, verify no stale copy elsewhere `[verified]` | `grep -n "cp .*skills" INSTALL.md` | ✅ Architect Read confirmed only `idea` @ 100-102 |
| 7 | LAYERS layer-boxes list attic skills as if living (`LAYERS.md:76` `/insight /route /decide`; `:91` `/plan /verify`; `:104` `/verify /review /qa /ship`) `[verified]` | `grep -n "Skills:" docs/LAYERS.md` | ✅ Architect Read confirmed — F3-adjacent skill drift |
| 8 | `docs/LAYERS.md` has a dedicated skill map/table (per CLAUDE.md "LAYERS skill table") — reconcile to 5 living `[needs Worker verify]` | `grep -n "^| \`/\|SKILL\|skill map\|/idea\|/retro" docs/LAYERS.md` | ⏳ TO VERIFY — Worker locate + check 5-living correctness |
| 9 | `phieu/RELAY_PROTOCOL.md:3` frames Architect as "Claude Web Project" v1 `[verified]` | `sed -n '1,10p' phieu/RELAY_PROTOCOL.md` | ✅ Architect Read confirmed |

**⚠️ Anchors #2,#4,#8 = `[needs Worker verify]`** — Architect (docs-only, no Grep) căn từ doc claim; Worker grep xác nhận trước khi sửa. Nếu #2 lệch (skill nào đó không như phân loại) → DISCOVERY_REPORT, đừng đoán.

---

## Debate Log

> Auto-populated by Worker (CHALLENGE) + Architect (RESPOND). Chủ nhà đọc khi nghiệm thu.
> Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge
**Worker accepted V1 — no challenges.** Anchor verification: #2 ✅ (5-living/8-attic split confirmed exact), #4 ✅ (`agents/architect.md:4,19-31` — Read/Write/Glob grant, no Bash/Grep/Edit, no "Web" mention), #8 ✅ (LAYERS has a dedicated skill table `:216-222` already correct; drift is only in the ASCII layer-boxes 130+ lines above). All other anchors (#1,#3,#5,#6,#7,#9) re-confirmed ✅. HANDOFF.md partial v1 residue noted as non-blocking (already gated by phiếu's own Nghiệm thu `:210`).

**Status:** ✅ READY FOR CHỦ NHÀ APPROVAL GATE

### Final consensus
- Phiếu version: V1 (no revisions needed)
- Approved by Chủ nhà: 2026-07-22 — F1 tagline = Option A. Execution completed same session.

---

## Nhiệm vụ

### Task 1: F3 — Sửa README install block (BREAKING fresh-clone)

**File:** `README.md`

**Tìm:** khối "### Skills (global)" — dòng `cp -r skills/init ~/.claude/skills/init` xuống hết `cp -r skills/retro ~/.claude/skills/retro` (Architect Read: lines 240-254; Worker verify exact range).

**Thay bằng / Thêm:** khối chỉ copy **skill tồn tại** đúng caller. Sau khi Task 0 anchor #2 xác nhận 5 living = `apply forge idea init retro`, chọn 1 trong 2 hướng (Worker Tầng-2 self-decide, miễn không list skill vắng mặt):

- Hướng A (khuyến nghị): thay block bằng note ngắn "Skills ship với caller cơ học (hook/cron/CLI — xem CLAUDE.md caller law); global copy chỉ cần cho skill Chủ-nhà/Kiến-trúc-sư dùng thủ công" + copy **đúng các skill living có caller CLI/manual** (`init` `forge` `apply` `retro`), và giữ note `/idea` project-local (đã có ở `README.md:257`).
- Hướng B: giữ format `cp -r`, nhưng CHỈ 5 dòng cho 5 living.

**Lưu ý:**
- KHÔNG để dòng nào trỏ `skills/insight|route|decide|plan|verify|review|qa|ship` (đã ở attic → fresh-clone fail).
- Nghiệm thu: mỗi `cp -r skills/<name>` còn lại phải trỏ path tồn tại (chạy thử từng cái không lỗi).
- Đối chiếu `README.md:134-142` (skill table) + `README.md:142` "Parked (attic)" — hai chỗ này đã đúng, giữ nguyên; install block phải khớp chúng.

### Task 2: F2 — Re-frame LAYERS.md v1 (Claude Web) → v2 (in-session subagent)

**File:** `docs/LAYERS.md`

**Tìm + sửa từng residue (Worker grep anchor #3 để lấy line chính xác — line có thể drift):**

1. `LAYERS.md:48` "**Critical**: Kiến trúc sư lives in Claude Web Project. No Bash, no Grep on source, no filesystem access beyond project's attached docs." → re-frame: envelope docs-only là **cơ học** (agents/architect.md: Read/Write/Glob, no Bash/Grep/Edit-src), Architect chạy **in-session như subagent** do Quản đốc spawn — Task 0 grep-first + Discovery vẫn là cầu nối tới code reality. GIỮ semantics "docs-only" (đúng), BỎ "lives in Claude Web Project".
2. `LAYERS.md:92` "Tools: Claude Web Project — docs access ONLY" → "Tools: Read / Write / Glob (docs-only envelope; no Bash/Grep/Edit-src) — spawned in-session by Quản đốc".
3. `LAYERS.md:84-85` + `:115` + `:166` (responsibility #7) + `:195-198` (Anti-pattern #3 "Worker pings Architect directly / Claude Code and Claude Web are separate sessions"): re-frame human-courier là **chế độ v1/cross-session tuỳ chọn**, KHÔNG default. v2 default = Quản đốc relay Architect↔Worker in-session (không copy-paste). Anti-pattern #3 giữ được ý "Worker không tự ping Architect" nhưng lý do đúng v2 = orchestrator routes, KHÔNG "vì Web ≠ Code session".
4. `LAYERS.md:77` "Tools: Claude Code OR Claude Web (usually wherever the human is)" — đây là **Chủ nhà** (human role) → GIỮ (human ngồi đâu cũng được, không phải drift). Chỉ đụng nếu câu chữ gây nhầm với Architect framing.

**Lưu ý:**
- Đây là re-frame CẨN THẬN, KHÔNG xoá mù. Giữ đúng: (a) Architect docs-only envelope, (b) Worker không tự ý ping/re-architect, (c) cross-session relay vẫn tồn tại cho worktree khác session — chỉ hạ từ "default" xuống "v1/optional".
- Căn v2 đúng bằng cách đối chiếu `agents/architect.md` + `docs/ORCHESTRATION.md` (state machine: DRAFT→CHALLENGE→RESPOND→APPROVAL→EXECUTE, orchestrator spawn cả 2 subagent).
- LAYERS = "single source" cho Tầng def (`:145-154`) — KHÔNG đụng phần đó.

### Task 3: F2-adjacent — Reconcile LAYERS skill references to 5 living

**File:** `docs/LAYERS.md`

**Tìm:** skill lists trong layer-boxes: `:76` "Skills: /init /idea /insight /route /decide", `:91` "Skills: /plan /forge /verify", `:104` "Skills: /verify /apply /review /qa /ship /retro"; + skill map/table (anchor #8 — Worker locate).

**Thay bằng:** chỉ reference 5 living (`/idea /retro /init /apply /forge`) HOẶC đánh dấu attic ones rõ ràng (vd "(attic — absorbed by orchestrator/Giám sát)"). Khớp `README.md:142` "Parked (attic)" wording để 2 doc đồng bộ.

**Lưu ý:** `[needs Worker verify]` — Worker confirm phân loại 5-living/8-attic (anchor #2) TRƯỚC khi sửa list. Nếu LAYERS có skill table riêng, sửa cả table + inline boxes cho nhất quán.

### Task 4: F2 — RELAY_PROTOCOL mode-clarification note

**File:** `phieu/RELAY_PROTOCOL.md`

**Tìm:** header blockquote `:1-16` ("Kiến trúc sư (Claude Web Project) and Thợ (Claude Code) are separate sessions...").

**Thay bằng / Thêm:** note ngắn đầu file phân biệt rõ: **v2 default** = Quản đốc spawn Architect+Worker in-session, relay tự động (doc này KHÔNG áp dụng); **RELAY_PROTOCOL này dùng cho v1/cross-session mode** (Thợ ở worktree/session tách, Architect session khác) — vẫn hợp lệ nhưng không phải happy-path mặc định.

**Lưu ý:** KHÔNG xoá protocol (còn giá trị cho cross-session). Chỉ thêm mode-banner để không đọc nhầm là current-default.

### Task 5: F1 — README tagline (CHỜ SẾP CHỐT — không sửa trước APPROVAL)

**File:** `README.md`

**Tìm:** `README.md:3` "One person. No team. Full operating system from inbound request to production health."

**Thay bằng:** 1 trong các option Sếp chốt ở APPROVAL (xem "F1 tagline options" cuối phiếu). KHÔNG Worker tự quyết câu chữ — user-facing → Chủ nhà final cut.

**Lưu ý:**
- Cũng rà `README.md:196-200` "Relay Protocol — Chủ nhà as the courier (Web Project mode)": đã honestly labeled "(Web Project mode)" + "v2 Subagent mode bypasses this" → chấp nhận được (historical). Chỉ chỉnh nếu Sếp muốn đồng bộ giọng v2; mặc định GIỮ.
- `docs/PHILOSOPHY.md:26` = đã đúng ("tail of the pipeline") → KHÔNG sửa; nó là reference để căn tagline.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `README.md` | Task 1 (install block 5-living), Task 5 (tagline — sau Sếp) |
| `docs/LAYERS.md` | Task 2 (v1→v2 re-frame), Task 3 (skill list reconcile) |
| `phieu/RELAY_PROTOCOL.md` | Task 4 (mode-clarification note) |
| `docs/SETUP.md` | Docs-gate mirror — nếu chứa skill install list drift tương tự F3 (Worker verify) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `docs/PHILOSOPHY.md` | `:26` "tail of the pipeline" đã đúng — là reference cho tagline, KHÔNG sửa |
| `INSTALL.md` | `:100-102` chỉ copy `idea` — verify không có 12-skill drift ẩn chỗ khác |
| `agents/architect.md` | Envelope Read/Write/Glob no-Bash/Grep — basis cho re-frame; KHÔNG sửa |
| `README.md:134-142` | Skill table + "Parked (attic)" đã đúng — install block phải khớp nó |
| `CLAUDE.md` | Role framing ("v2.1+ Subagent mode", caller law) — nguồn chuẩn, KHÔNG sửa |

---

## Luật chơi (Constraints)

1. **Docs-only phiếu** — KHÔNG đụng bất kỳ runtime/code (repo là meta-kit). No Bash beyond git/verify.
2. **F2 re-frame CẨN THẬN, không xoá mù** — giữ 3 semantics đúng: Architect docs-only envelope (cơ học, không "vì Web"), Worker không tự ping/re-architect, cross-session relay hạ xuống v1/optional (không xoá).
3. **F1 tagline KHÔNG tự quyết** — chờ Sếp chốt ở APPROVAL trước khi sửa `README.md:3`.
4. **F3 nghiệm thu cứng** — mọi `cp -r skills/<name>` còn lại phải trỏ path tồn tại; fresh-clone không fail.
5. **KHÔNG đụng F4/F5** — đã verify không phải lỗi (`/init` caveat chủ đích; `bin/sos.sh`/`core/ROLES.md`/`adapters/claude/MAPPING.md` tồn tại + `sos` nhãn "(planned)" trung thực).
6. **KHÔNG đụng LAYERS Tầng-def single-source** (`:145-154`) và skill-table Tầng split.
7. **DOCS GATE Tầng 1** (xem Nghiệm thu → Docs Gate).

---

## Nghiệm thu

### Automated
- [ ] `docs-gate --all` clean (changelog fresh + staged)
- [ ] Fresh-clone skill install dry-run: từng `cp -r skills/<name>` trong README install block trỏ path tồn tại (không "No such file")

### Manual Testing
- [ ] `README.md` install block chỉ list 5 living skill (`apply forge idea init retro`), 0 reference tới 8 attic
- [ ] `docs/LAYERS.md` không còn "Claude Web Project" như **current mode** cho Architect (chỉ được nhắc historical/v1 nếu cần); Architect framed as in-session subagent, envelope docs-only cơ học giữ nguyên
- [ ] LAYERS skill references + skill table khớp 5 living (attic ones đánh dấu rõ)
- [ ] `phieu/RELAY_PROTOCOL.md` có mode-banner phân biệt v2-default (in-session) vs v1/cross-session
- [ ] `README.md:3` tagline khớp PHILOSOPHY scope (code-ready → production) — sau Sếp chốt F1

### Regression
- [ ] README skill table (`:134-142`) + "Parked (attic)" (`:142`) vẫn đúng, đồng bộ install block
- [ ] LAYERS Tầng-def single-source (`:145-154`) không đổi
- [ ] Semantics Architect docs-only + Worker no-self-ping vẫn còn (chỉ đổi framing, không đổi luật)

### Docs Gate (Tầng 1 — BẮT BUỘC)
- [ ] `README.md` sửa → per CLAUDE.md Rule #4 "README single source of truth": khớp `docs/SETUP.md` (skill install section) + `docs/LAYERS.md` skill table. Verify cả 3 đồng bộ.
- [ ] `docs/LAYERS.md` role-framing sửa → kiểm chạm `docs/HANDOFF.md` (relay format Handoff 3) — nếu HANDOFF cũng frame "Claude Web courier default", cần đồng bộ note v2 (Worker verify; nếu có → thêm task hoặc DISCOVERY).
- [ ] `CHANGELOG.md` — entry P083
- [ ] Worker ghi Discovery: "Tầng 1 docs updated: <list>" (README/LAYERS/SETUP/HANDOFF nếu chạm)

### Discovery Report
- [ ] Write `docs/discoveries/P083.md`:
  - Anchor #2/#4/#8 verify kết quả (5-living/8-attic phân loại đúng? architect.md tools? LAYERS skill table location?)
  - HANDOFF.md có cùng v1 residue không (chạm → note)
  - Tagline option Sếp chốt
  - Docs updated to match reality (list) / "None" nếu không
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`

---

## F1 tagline options (orchestrator hỏi Sếp ở APPROVAL — Chủ nhà final cut)

Giữ punch "One person. No team." nhưng bỏ overclaim đầu pipeline (khớp PHILOSOPHY "tail of the pipeline — code-ready → production"):

- **Option A (Recommended — sát PHILOSOPHY nhất):** "One person. No team. From code-ready to production health."
- **Option B (giữ 'operating system' punch):** "One person. No team. A full operating system for the back half of the pipeline — code-ready to production health."
- **Option C (nhấn 'without dropping context'):** "One person. No team. Ship and run production without dropping context — code-ready to health checks."
