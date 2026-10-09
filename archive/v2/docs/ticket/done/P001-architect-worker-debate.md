# PHIẾU P001: Architect ↔ Worker multi-turn debate (pre-code consensus loop)

> **Loại:** Feature (meta-design — sửa `.claude/agents/*` + docs + template)
> **Ưu tiên:** P1 — quan trọng nhưng không block ship hiện tại; là bước tiến hoá v2 → v2.1
> **Ảnh hưởng:**
> - `.claude/agents/architect.md` + `agents/architect.md`
> - `.claude/agents/worker.md` + `agents/worker.md`
> - `phieu/TICKET_TEMPLATE.md` (thêm Debate Log section)
> - `INSTALL.md` (workflow steps + verify)
> - `docs/HANDOFF.md` (thêm Handoff 2.5 — debate loop)
> - `README.md` (đoạn "Two ways to run the 3-role envelope" → cập nhật v2 description)
> **Dependency:** None. Phiếu này standalone.

---

## Context

### Vấn đề hiện tại

Workflow v2 Subagent mode hiện chỉ là **single-pass linear dispatcher**:

```
Sếp → spawn architect → phiếu → Sếp duyệt → spawn worker → code → done
```

**Cụ thể chỗ thiếu:**
1. Worker không có cơ chế **challenge phiếu trước khi code**. Hiện chỉ Tầng 1 escalation qua `AskUserQuestion` với Sếp — bắt Sếp giữa cuộc làm courier.
2. Architect không được spawn lại để **respond** với Worker's challenge — phải Sếp paste qua paste lại (chính cái RELAY_PROTOCOL.md mà v2 muốn loại bỏ).
3. Discovery Report là **post-hoc, async** — chỉ có hiệu lực sang phiếu sau, không cứu được phiếu hiện tại.
4. Workflow trước (Web Project mode) anh phải làm courier paste qua-lại vài turn cho 2 bên đồng nhất ý kiến → mệt, slow, dễ mất context. Anh đã chuyển sang Claude Code subagents để loại bỏ chính việc này, nhưng v2 hiện tại vẫn chưa có cơ chế debate tự động → anh vẫn đang làm courier nhưng giữa 2 subagent thay vì 2 sessions.

**Hậu quả thực tế khi áp v2 vào Tarot:** mỗi phiếu có giả định sai về code → Worker phát hiện ở Task 0 → escalation về Sếp → Sếp phải tự đoán paste gì lại cho Architect → vòng lặp chậm, không scale.

### Giải pháp

**Multi-turn debate loop** giữa Architect và Worker, do **main session (orchestrator)** điều phối — Sếp chỉ vào lúc đầu (brief) và lúc cuối (nghiệm thu phiếu trước code, nghiệm thu code sau ship).

**Flow mới:**

```
Sếp brief → main session orchestrate:
  1. Spawn architect → write phiếu draft V1 → [phiếu đặt vào docs/ticket/]
  2. Spawn worker (CHALLENGE mode) → đọc phiếu V1 + verify Task 0 + đọc code thật
                                   → viết Challenge Report vào phiếu Section "Debate Log Turn 1"
                                   → KHÔNG code, return
  3. Nếu Challenge Report = empty (no objections) → goto 6
  4. Spawn architect (RESPOND mode) → đọc Debate Log Turn 1
                                    → refine phiếu (update tasks/anchors) HOẶC defend with new evidence
                                    → ghi Response vào "Debate Log Turn 1 — Architect Response"
                                    → bump phiếu version V2
  5. Spawn worker (CHALLENGE mode) → đọc V2 → goto 2 (max N turns)
  6. Khi Worker chấp nhận (no new challenge) → main session cho Sếp xem phiếu cuối + Debate Log
                                              → AskUserQuestion "Approve để Worker triển?"
  7. Sếp duyệt → Spawn worker (EXECUTE mode) → code → test → Discovery → commit
  8. Sếp nghiệm thu code
```

**Sếp's role trong flow này:**
- Đầu: 1 câu brief ("build feature X")
- Giữa (chỉ khi agents bế tắc sau N=3 turns): AskUserQuestion break deadlock
- Cuối phiếu (trước code): Approve/veto phiếu đã debate
- Cuối ship: Nghiệm thu

**Sếp KHÔNG phải:**
- Paste challenge từ Worker sang Architect (orchestrator làm)
- Lặp lại Architect's reasoning cho Worker (orchestrator làm)
- Đọc trace mid-debate (debate log auto-saved vào phiếu file)

### Scope

**CHỈ làm:**
- Sửa `agents/architect.md` + `.claude/agents/architect.md`: thêm 2 invocation modes (DRAFT, RESPOND)
- Sửa `agents/worker.md` + `.claude/agents/worker.md`: thêm 2 invocation modes (CHALLENGE, EXECUTE)
- Sửa `phieu/TICKET_TEMPLATE.md`: thêm Debate Log section schema
- Tạo mới `docs/ORCHESTRATION.md`: spec main-session orchestrator loop (state machine, max turns, escalation triggers)
- Sửa `docs/HANDOFF.md`: thêm Handoff 2.5 (Architect ↔ Worker debate)
- Sửa `INSTALL.md`: cập nhật workflow steps (1 brief command thay vì 2 spawn commands)
- Sửa `README.md` mục "Two ways to run the 3-role envelope" (v2 description)

**KHÔNG làm trong phiếu này:**
- Không tự động spawn theo cron / không daemon
- Không dùng external state store — debate trail SỐNG trong phiếu file (đơn giản, có audit trail)
- Không thay đổi `phieu/phieu.sh` (shell function đủ cho v2.1)
- Không thay đổi `scripts/architect-guard.sh` (vẫn block code reads cho Architect — kể cả ở RESPOND mode)
- Không thay đổi vps/ship/guard/docs-gate Rust binary
- Không tự động áp vào Tarot — phiếu này deliver vào sos-kit, anh tự copy `.claude/` vào Tarot khi sẵn sàng (theo INSTALL.md hiện có)

---

## Task 0 — Verification Anchors

> Worker chạy `/verify` trước khi code. Cập nhật cột Result với ✅ / ⚠️ / ❌.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `agents/architect.md` tồn tại + có frontmatter `tools: Read, Write, Glob, TaskCreate, TaskUpdate, TaskList, AskUserQuestion` | `head -10 agents/architect.md` | ✅ verified pre-implement |
| 2 | `.claude/agents/architect.md` tồn tại và **chỉ khác** `agents/architect.md` ở từ "Sếp" vs "Chủ nhà" | `diff agents/architect.md .claude/agents/architect.md` | ⚠️ pre-implement: 7 lines differed beyond name; ✅ post-implement: harmonized via sync script, only Sếp/Chủ nhà differs now |
| 3 | `agents/worker.md` tồn tại + tools: Read/Write/Edit/Glob/Grep/Bash/Task*/AskUserQuestion | `head -10 agents/worker.md` | ✅ verified pre-implement |
| 4 | `.claude/agents/worker.md` khác `agents/worker.md` chỉ ở Sếp/Chủ nhà | `diff agents/worker.md .claude/agents/worker.md` | ⚠️ → ✅ same as #2 |
| 5 | `phieu/TICKET_TEMPLATE.md` chưa có section "Debate Log" | `grep -i "debate" phieu/TICKET_TEMPLATE.md` | ✅ pre-implement empty; post-implement 1 hit (added) |
| 6 | `docs/HANDOFF.md` hiện có 5 handoffs (0–4), chưa có 2.5 | `grep -E "^## Handoff" docs/HANDOFF.md` | ✅ pre-implement 5 lines; post-implement 6 (added 2.5) |
| 7 | `scripts/architect-guard.sh` block path `src/` khi marker `.claude/.architect-active` tồn tại | smoke test exit code | ✅ smoke test confirmed exit=2 on `src/main.rs` |
| 8 | `INSTALL.md` Step 5 "Verify install" có dòng test architect-guard | `grep -n "architect-guard" INSTALL.md` | ✅ verified |
| 9 | `docs/ORCHESTRATION.md` chưa tồn tại | `ls docs/ORCHESTRATION.md 2>&1` | ✅ pre-implement: No such file; post-implement: 150 lines |
| 10 | `README.md` có section "Two ways to run the 3-role envelope" với table v1/v2 mode | `grep -n "Two ways to run" README.md` | ✅ verified |

---

## Nhiệm vụ

### Task 1: Mở rộng `phieu/TICKET_TEMPLATE.md` với schema Debate Log

**File:** `phieu/TICKET_TEMPLATE.md`

**Tìm:** dòng phân cách `---` ngay trước section `## Nhiệm vụ` (sau Task 0 table)

**Thay bằng / Thêm:** chèn section mới **giữa Task 0 và Nhiệm vụ**:

```markdown
---

## Debate Log

> Auto-populated by Worker (CHALLENGE mode) and Architect (RESPOND mode). Sếp chỉ đọc khi nghiệm thu phiếu — không cần can thiệp mid-debate trừ khi orchestrator triệu Sếp do bế tắc (max-turn cap reached).
> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Phiếu version bump V1 → V2 → ... mỗi turn Architect refine.

### Turn 1 — Worker Challenge (phiếu V1)
**Verified by Worker against real code (Task 0 grep results):**
- Anchor #N: ✅/⚠️/❌ + 1 dòng tóm tắt nếu ⚠️/❌

**Objections (Tầng 1 — phiếu cần sửa):**
- [Mã: O1.1] Phiếu giả định X tại file Y, code thật là Z (cite file:line). Tác động: …
- [Mã: O1.2] …

**Proposed alternatives (1-2 options, Worker khuyến nghị 1):**
- A. … (Worker lean — vì …)
- B. …

**Status:** ⏳ AWAITING ARCHITECT RESPONSE

### Turn 1 — Architect Response (phiếu → V2)
- O1.1 → ACCEPT / DEFEND / DEFER → cập nhật Nhiệm vụ Task M / giữ nguyên với evidence từ doc Z
- O1.2 → …

**Status:** ✅ RESPONDED — phiếu bumped to V2

[lặp Turn 2, Turn 3 nếu cần. Cap = 3 turns]

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Sếp: [date] — code có thể bắt đầu
```

**Lưu ý:** giữ nguyên section `## Nhiệm vụ` và sau, không xoá. Debate Log nằm GIỮA Task 0 và Nhiệm vụ vì chronologically là sau Task 0 verify, trước code.

---

### Task 2: Thêm CHALLENGE mode vào Worker subagent (cả 2 file)

**File:** `agents/worker.md` VÀ `.claude/agents/worker.md` (giữ 2 bản đồng bộ — xem Task 6)

**Tìm:** section `## On invocation, do this in order` (đầu file body)

**Thay bằng / Thêm:** đổi cấu trúc thành **2 modes** rõ ràng. Trước section "On invocation", thêm:

```markdown
## Invocation modes

Worker được spawn với 1 trong 2 mode (orchestrator chỉ định trong prompt):

| Mode | Trigger phrase trong prompt | Behavior |
|---|---|---|
| **CHALLENGE** | "Worker challenge phiếu P<NNN>" hoặc "review phiếu pre-code" | Đọc phiếu + verify Task 0 + đọc code thật → viết Debate Log Turn N → **KHÔNG code, KHÔNG commit, return** |
| **EXECUTE** | "Worker execute phiếu P<NNN>" hoặc "implement P<NNN>" (chỉ sau khi Sếp đã approve) | Workflow cũ: Task 0 → code → test → Discovery → commit. (Mode mặc định nếu prompt không nói rõ — backward compat) |

Default = EXECUTE nếu không có trigger phrase rõ ràng.
```

**Sau đó:** đổi heading `## On invocation, do this in order` thành `## CHALLENGE mode workflow` và viết riêng:

```markdown
## CHALLENGE mode workflow

1. Read phiếu file `docs/ticket/P<NNN>-<slug>.md`
2. **Run Task 0 verification** — đầy đủ như mode EXECUTE: grep từng anchor, cập nhật cột Result trong file
3. **Read code thật** ở các file phiếu reference — chú ý: phiếu nói gì, code thật như nào
4. **Identify objections (Tầng 1 only):**
   - File/function không tồn tại như phiếu giả định
   - Function signature khác phiếu
   - Schema/migration phiếu không lường tới
   - Phiếu's approach conflict với pattern hiện có (Context vs Zustand, …)
   - Side-effect mà phiếu không document
5. **NẾU không có objection** → ghi vào Debate Log: `**Worker accepted V<N>** — no challenges. Ready for Sếp approval.` → return.
6. **NẾU có ≥1 objection:**
   - Cho mỗi objection: cite `file:line` từ code thật
   - Đề xuất 1-2 alternatives, recommend 1 (Worker lean)
   - Append vào phiếu file section `## Debate Log` → `### Turn <N> — Worker Challenge`
   - Status: `⏳ AWAITING ARCHITECT RESPONSE`
   - **Hand back to orchestrator (do NOT spawn architect from here — orchestrator điều phối)**
7. **KHÔNG code, KHÔNG commit, KHÔNG sửa file ngoài phiếu file đó.**

## EXECUTE mode workflow

[giữ nguyên workflow cũ — Task 0 → code → Discovery → commit]
```

**Lưu ý:**
- CHALLENGE mode chỉ append vào phiếu file, không sửa file khác → low risk
- Tầng 2 issues (var name, internal helper) **không** challenge — log to Discovery sau khi EXECUTE như cũ
- Worker phải tự discipline: nếu không tìm được objection thật, accept ngay — đừng bịa challenge để "thấy mình hữu ích"

---

### Task 3: Thêm RESPOND mode vào Architect subagent (cả 2 file)

**File:** `agents/architect.md` VÀ `.claude/agents/architect.md`

**Tìm:** section `## On invocation, do this in order` (sau Hard envelope rules)

**Thay bằng / Thêm:** chèn section `## Invocation modes` (giống Worker, trước "On invocation"):

```markdown
## Invocation modes

| Mode | Trigger phrase trong prompt | Behavior |
|---|---|---|
| **DRAFT** | "Spawn architect viết phiếu cho X" hoặc "plan X" | Workflow gốc — đọc docs, viết phiếu mới ở `docs/ticket/P<NNN>-*.md`. Phiếu version = V1 |
| **RESPOND** | "Architect respond to Debate Log Turn <N> in P<NNN>" | Đọc Debate Log → respond từng objection → refine phiếu → bump version |

Default = DRAFT nếu không có trigger phrase.
```

**Đổi `## On invocation, do this in order` → `## DRAFT mode workflow`** (giữ nội dung cũ).

**Thêm section mới sau DRAFT:**

```markdown
## RESPOND mode workflow

Bị spawn lại sau khi Worker viết Challenge. Constraint envelope vẫn áp dụng (KHÔNG đọc src/, không Bash, không Grep). Lý do giữ envelope: Worker đã verify code rồi, nhiệm vụ Architect là phán quyết dựa vào docs + Worker's evidence — không cần peek code.

1. **Read phiếu file** `docs/ticket/P<NNN>-*.md` — đặc biệt section `## Debate Log` (turn mới nhất chưa response)
2. **Read lại docs liên quan** — Worker challenge có thể fix một giả định mà doc cũ đã sai → kiểm tra DISCOVERIES.md xem có precedent không
3. **For each objection (O<N>.<M>):** quyết định 1 trong 4:
   - **ACCEPT** — Worker đúng, sửa Nhiệm vụ tương ứng. Update `Files cần sửa` table nếu cần.
   - **DEFEND** — Doc evidence còn vững (ghi rõ doc:section). Worker hiểu nhầm — clarify trong response.
   - **REFRAME** — cả 2 đều có lý nhưng vấn đề thực ra là Tầng 2 (Worker's call) → ghi vào response: "đây là Tầng 2, Worker tự quyết khi EXECUTE, log Discovery"
   - **DEFER TO SẾP** — đây là vision/scope decision, không phải tech → request orchestrator triệu Sếp qua AskUserQuestion
4. **Append vào phiếu** section `### Turn <N> — Architect Response`:
   - Liệt kê quyết định cho từng objection
   - Cập nhật phiếu metadata: `Phiếu version: V<N+1>`
   - Status: `✅ RESPONDED — phiếu bumped to V<N+1>`
5. **NẾU mọi objection đều ACCEPT/REFRAME** → có khả năng phiếu đã đồng nhất ý → **return to orchestrator** (orchestrator sẽ spawn Worker lần nữa CHALLENGE để xác nhận hoặc accept)
6. **NẾU có DEFEND** → có thể tạo Turn N+1 challenge từ Worker → orchestrator quyết
7. **NẾU bất kỳ DEFER TO SẾP** → status `⚠️ AWAITING SẾP` — orchestrator BẮT BUỘC triệu Sếp qua AskUserQuestion
```

**Lưu ý:**
- Architect KHÔNG được phép Read file `.rs/.ts/.py/...` ngay cả ở RESPOND mode — `architect-guard.sh` enforce
- Architect dựa vào Worker's `file:line` citation trong Debate Log như single source of truth về code reality
- Maximum cap = 3 turns. Sau Turn 3 nếu chưa consensus → orchestrator force-escalate Sếp

---

### Task 4: Tạo `docs/ORCHESTRATION.md` — spec orchestrator loop

**File:** `docs/ORCHESTRATION.md` (mới)

**Nội dung yêu cầu:**

1. **Mục đích:** spec cách main session (Claude Code default agent, không phải subagent) điều phối loop debate. Đây là role thứ tư bổ sung 3 role: **Orchestrator** = main session, không phải human, là Claude điều phối subagents.
2. **State machine:**
   ```
   IDLE → DRAFT_PHASE → CHALLENGE_PHASE → (consensus?) → APPROVAL_GATE → EXECUTE_PHASE → DONE
                              ↓ (objections)
                         RESPOND_PHASE → CHALLENGE_PHASE (Turn N+1)
                              ↓ (max turns OR DEFER TO SẾP)
                         FORCE_ESCALATION → AskUserQuestion → DRAFT_PHASE (Sếp adjusted brief) | EXECUTE | ABANDON
   ```
3. **Trigger phrases** mà main session detect để spawn đúng mode (đối ứng với Task 2 + Task 3 mode tables)
4. **Max turns cap = 3** — sau đó force-escalate. Lý do: tránh agents loop vô hạn.
5. **Approval gate:** trước EXECUTE_PHASE, main session BẮT BUỘC dùng AskUserQuestion với options "Approve để Worker code / Sếp tự sửa phiếu / Abandon".
6. **Failure modes + recovery:**
   - Architect RESPOND không update phiếu version → orchestrator force re-spawn 1 lần, lần 2 escalate
   - Worker CHALLENGE viết objection nhưng không cite file:line → orchestrator reject, ask re-do
   - Marker `.claude/.architect-active` không clean up → orchestrator clean trước mỗi spawn
7. **One concrete example session** end-to-end (3 turn debate → Sếp approve → execute), full transcript dạng pseudo-code.

**Voice:** English (đây là public-facing doc, theo CLAUDE.md rule). Body ≤ 250 lines.

---

### Task 5: Thêm Handoff 2.5 vào `docs/HANDOFF.md`

**File:** `docs/HANDOFF.md`

**Tìm:** section `## Handoff 3 — Thợ → Chủ nhà → Kiến trúc sư (architectural blocker)` (line ~87)

**Thay bằng / Thêm:** chèn section mới **trước** Handoff 3:

```markdown
## Handoff 2.5 — Architect ↔ Worker debate (v2 Subagent mode only)

**Trigger:** Architect đã viết phiếu V1. Trước khi Worker EXECUTE, Worker CHALLENGE để verify giả định phiếu chống lại code thật.

**Transport:** Section `## Debate Log` trong phiếu file. Append-only, không bao giờ xoá lịch sử turn cũ.

**Format (Worker → Architect, Turn N):**
```
### Turn <N> — Worker Challenge (phiếu V<N>)
- [Anchor verifications: ✅/⚠️/❌ list]
- [Objections O<N>.<M>: cite file:line]
- [Alternatives + Worker lean]
- Status: ⏳ AWAITING ARCHITECT RESPONSE
```

**Format (Architect → Worker, Turn N response):**
```
### Turn <N> — Architect Response (phiếu V<N+1>)
- O<N>.<M> → ACCEPT/DEFEND/REFRAME/DEFER → action
- Status: ✅ RESPONDED
```

**Termination:**
- Worker accept (no objection) → consensus → Sếp approval gate
- Max 3 turns reached → force-escalate Sếp via orchestrator
- Architect DEFER TO SẾP → escalate via AskUserQuestion

**Anti-pattern:** Worker dump objection không cite file:line → Architect không có evidence để judge → loop vô nghĩa.
**Fix:** mỗi objection BẮT BUỘC `file:line` reference. Orchestrator reject Challenge thiếu evidence.

**Anti-pattern:** Sếp bị triệu mid-debate khi 2 agents tự giải quyết được → vi phạm "Sếp không làm courier."
**Fix:** chỉ triệu Sếp ở approval gate cuối, hoặc khi Architect DEFER TO SẾP, hoặc max-turn cap.

**Replaces:** RELAY_PROTOCOL.md (Web Project mode legacy). Khi v2 Subagent mode active, Handoff 2.5 thay thế hoàn toàn vai trò Sếp-courier.
```

**Lưu ý:**
- Cập nhật bảng `## Handoff triggers summary` cuối file thêm 1 dòng:
  ```
  | Architect ↔ Worker (auto-debate) | Phiếu V1 vừa viết, pre-execute | Debate Log section in phiếu | (orchestrator) |
  ```
- KHÔNG xoá Handoff 3 (Tầng 1 escalation vẫn dùng được trong EXECUTE phase nếu mid-code phát hiện vấn đề mới)

---

### Task 6: Thống nhất source-of-truth cho `agents/` vs `.claude/agents/`

**File:** new `agents/README.md` + sửa `.claude/agents/architect.md` + `.claude/agents/worker.md`

**Vấn đề:** hiện `agents/architect.md` (root, "Chủ nhà" voice, public-facing) và `.claude/agents/architect.md` (project-local, "Sếp" voice) lệch nhau bằng tay → drift hazard.

**Quyết định kiến trúc:**
- `agents/` = **source of truth** (canonical, English-neutral, "Chủ nhà"). Đây là bản distribute cho external user.
- `.claude/agents/` = **local override** (chỉ trong sos-kit repo cho Denny work với "Sếp" voice). Có thể tự generate từ `agents/` qua sed.

**Hành động:**
1. Tạo `agents/README.md` ngắn (≤ 30 dòng) giải thích:
   - `agents/*.md` là source of truth
   - Khi install vào project khác (qua INSTALL.md): copy từ `~/sos-kit/agents/*.md` (KHÔNG phải `.claude/agents/`)
   - `.claude/agents/*.md` chỉ là local override của maintainer
2. Sửa `INSTALL.md` Step 1: đổi `cp ~/sos-kit/.claude/agents/architect.md` → `cp ~/sos-kit/agents/architect.md` (xem Task 7).
3. Tạo `scripts/sync-personal-agents.sh` để Denny chạy 1 lần khi update agents — sed thay "Chủ nhà" → "Sếp" và copy `agents/*.md` → `.claude/agents/*.md`. Lý do: tránh drift bằng tay; mọi update logic vào `agents/` rồi 1 command sync.

**Lưu ý:**
- Đây là small re-arch, không phải feature core, nhưng nếu skip thì Task 2 + Task 3 update sẽ phải làm 2 lần (4 file thay vì 2 file canonical)
- Worker nếu cảm thấy task 6 quá scope, escalate qua Debate Log (chính là dogfood phiếu này)

---

### Task 7: Cập nhật `INSTALL.md` workflow + verify steps

**File:** `INSTALL.md`

**Tìm:** Step 1 dòng `cp ~/sos-kit/.claude/agents/architect.md .claude/agents/`

**Thay bằng:**
```bash
cp ~/sos-kit/agents/architect.md .claude/agents/
cp ~/sos-kit/agents/worker.md .claude/agents/
```

**Tìm:** Step 4 (Update CLAUDE.md) phần "Workflow:" liệt kê 5 bước

**Thay bằng / Thêm:** đổi 5 bước cũ thành flow mới với debate:

```markdown
**Workflow (v2.1 với debate):**
1. Mở Claude Code → SessionStart hook show BACKLOG → Sếp pick item
2. **Sếp 1 câu brief** (e.g., "build feature X cho item ABC ở Active sprint")
3. Main session orchestrate:
   a. Spawn architect (DRAFT) → phiếu V1
   b. Spawn worker (CHALLENGE) → debate log Turn 1
   c. (nếu có objection) Spawn architect (RESPOND) → phiếu V2
   d. Loop tới consensus hoặc max 3 turns
4. **Sếp approval gate** — main session AskUserQuestion show phiếu cuối + debate log → Sếp duyệt
5. Spawn worker (EXECUTE) → Task 0 → code → test → Discovery → commit local
6. Sếp nghiệm thu, deploy
```

**Tìm:** Step 5 "Verify install" code block

**Thêm vào sau lệnh `/agents — should list 'architect' and 'worker'`:**

```bash
# Test debate mode
echo "1 câu brief test" | tee /tmp/brief.txt
# Trong Claude Code, chạy:
#   "build a phiếu cho a chore nhỏ test debate flow"
# → kỳ vọng: spawn architect → spawn worker (CHALLENGE) → (nếu phiếu OK) Sếp gate
```

**Tìm:** "First phiếu (smoke test)" section

**Thay bằng:** smoke test mới có 1 round debate (mock 1 objection để verify loop chạy).

---

### Task 8: Cập nhật `README.md` mục v2 description

**File:** `README.md`

**Tìm:** dòng `Subagent mode adds a `BACKLOG.md` forcing function...` (line ~27)

**Thay bằng:**

```markdown
Subagent mode adds two forcing functions:
1. **BACKLOG.md gate** — Architect chỉ viết phiếu cho item ở "Active sprint"
2. **Pre-code debate loop (v2.1)** — Worker challenge phiếu chống code thật trước khi code; Architect respond multi-turn cho tới consensus. Sếp chỉ vào ở 2 điểm: brief đầu + nghiệm thu cuối. Xem [`docs/ORCHESTRATION.md`](./docs/ORCHESTRATION.md) và [`docs/HANDOFF.md`](./docs/HANDOFF.md#handoff-25) chi tiết.

A SessionStart hook surfaces the backlog every time you open Claude Code. See [`INSTALL.md`](./INSTALL.md) for setup.
```

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `phieu/TICKET_TEMPLATE.md` | Task 1 — thêm Debate Log section schema |
| `agents/worker.md` | Task 2 — thêm CHALLENGE + EXECUTE invocation modes |
| `.claude/agents/worker.md` | Task 2 — đồng bộ với agents/worker.md (qua sync script Task 6) |
| `agents/architect.md` | Task 3 — thêm DRAFT + RESPOND invocation modes |
| `.claude/agents/architect.md` | Task 3 — đồng bộ |
| `docs/ORCHESTRATION.md` | Task 4 — TẠO MỚI |
| `docs/HANDOFF.md` | Task 5 — thêm Handoff 2.5, cập nhật summary table |
| `agents/README.md` | Task 6 — TẠO MỚI |
| `scripts/sync-personal-agents.sh` | Task 6 — TẠO MỚI |
| `INSTALL.md` | Task 7 — cập nhật install path + workflow + verify |
| `README.md` | Task 8 — cập nhật v2 description |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/architect-guard.sh` | Hook vẫn block source code reads ngay cả ở RESPOND mode (Architect Read .md vẫn được, .rs/.ts/.py vẫn block) |
| `scripts/session-start-banner.sh` | Banner không thay đổi — vẫn hiện BACKLOG ở SessionStart |
| `.claude/settings.json` | Hooks config không thay đổi |
| `phieu/phieu.sh` | Shell function tạo phiếu không thay đổi |
| `phieu/RELAY_PROTOCOL.md` | KHÔNG xoá (vẫn dùng cho v1 Web Project mode legacy) — chỉ thêm note "deprecated for v2.1+, see Handoff 2.5" ở đầu file (verify Worker khi đụng vào tự thêm note này, Tầng 2 — không cần escalate) |
| `hooks/pre-commit` | Pre-commit hook không thay đổi (debate diễn ra TRƯỚC commit, không ảnh hưởng gate) |
| Tất cả file Rust binary repos (`~/ship`, `~/vps`, …) | Phiếu này KHÔNG động đến binary, MCP server, v.v. |

---

## Luật chơi (Constraints)

1. **Architect envelope giữ nguyên** — RESPOND mode vẫn không được Read code. Hook `architect-guard.sh` vẫn enforce. Architect dựa vào Worker's `file:line` citation, không peek code "just to check."
2. **Debate trail = phiếu file** — không tách ra file riêng, không database. Audit trail nằm cùng phiếu, lưu git lịch sử.
3. **Max 3 turns** — sau đó force escalate Sếp. Tránh loop vô hạn agents tự thuyết phục nhau.
4. **Sếp's role:** brief đầu + approval gate + final nghiệm thu. KHÔNG mid-debate trừ khi DEFER TO SẾP / max-turn / Sếp tự xen vào (luôn được).
5. **Backward compatible** — Worker mode mặc định vẫn EXECUTE (giữ workflow cũ chạy được). Để dùng debate mới, orchestrator/người gọi phải explicit "CHALLENGE" hoặc trigger phrase mới.
6. **Voice phiếu** — Vietnamese (project sos-kit's docs theo bilingual rule: public docs English, internal phiếu Vietnamese OK). Debate Log content trong phiếu cũng Vietnamese.
7. **Voice public docs** (README, ORCHESTRATION.md, HANDOFF.md, INSTALL.md, agents/*.md) — English. `.claude/agents/*.md` Vietnamese cho Denny.
8. **No new dependency** — không thêm gem/crate/package mới. Tất cả là markdown + bash sed (cho sync script).
9. **No daemon, no cron** — orchestrator là main session in-line, không background process.
10. **Phiếu này không được tự dogfood** — tức Worker khi EXECUTE phiếu này, KHÔNG dùng debate mode (vì debate mode chưa tồn tại lúc execute). Sau khi merge, phiếu kế tiếp mới hưởng feature.

---

## Nghiệm thu

### Automated
- [ ] Bash script `scripts/sync-personal-agents.sh` chạy không lỗi: `bash scripts/sync-personal-agents.sh` exit 0
- [ ] Sau sync: `diff <(grep -v "Sếp\|Chủ nhà" agents/architect.md) <(grep -v "Sếp\|Chủ nhà" .claude/agents/architect.md)` empty (chỉ khác từ Sếp/Chủ nhà)
- [ ] `head -20 phieu/TICKET_TEMPLATE.md` không break — frontmatter còn nguyên
- [ ] `bash scripts/architect-guard.sh` test (như INSTALL.md Step 5) vẫn exit 2 khi block — không regress
- [ ] `grep "Handoff 2.5" docs/HANDOFF.md` có ≥1 hit
- [ ] `ls docs/ORCHESTRATION.md` tồn tại + ≥ 100 dòng

### Manual Testing (smoke test trên Tarot OR sos-kit dogfood mới)
- [ ] Trong project mới (sos-kit/temp-test/ hoặc Tarot fresh branch):
  1. Copy `.claude/agents/*` từ sos-kit
  2. Tạo phiếu fake với 1 anchor sai cố ý ("function `nonExistentFn` exists in `src/foo.ts`")
  3. Spawn Worker CHALLENGE → kỳ vọng: Worker grep, không thấy, viết Debate Log Turn 1 với O1.1 cite "file `src/foo.ts` exists nhưng không có `nonExistentFn`"
  4. Spawn Architect RESPOND → kỳ vọng: ACCEPT, refine phiếu V2
  5. Spawn Worker CHALLENGE V2 → kỳ vọng: accept (no objection)
  6. AskUserQuestion approval gate hiện ra
- [ ] Test max-turn cap: tạo phiếu với 3 anchor sai → kỳ vọng sau Turn 3 force-escalate Sếp
- [ ] Test backward compat: spawn Worker không có CHALLENGE keyword → kỳ vọng: chạy EXECUTE mode (workflow cũ)

### Regression
- [ ] v2 hiện có vẫn hoạt động: BACKLOG gate, /idea skill, SessionStart banner, architect-guard hook, pre-commit hook đều như cũ
- [ ] Phiếu cũ (nếu có ai đã tạo theo template cũ chưa có Debate Log section) vẫn parse OK — Worker EXECUTE mode bỏ qua section thiếu
- [ ] `phieu` shell function `phieu-init`, `phieu <slug>`, `phieu-list`, `phieu-done` không bị ảnh hưởng

### Docs Gate
- [ ] `CHANGELOG.md` (sos-kit chưa có — TẠO MỚI nếu thiếu) — entry: `- [v2.1] feat: Architect ↔ Worker pre-code debate loop (P001)`
- [ ] `docs/HANDOFF.md` cập nhật ✅ (Task 5)
- [ ] `docs/ORCHESTRATION.md` mới ✅ (Task 4)
- [ ] `README.md` v2 section cập nhật ✅ (Task 8)
- [ ] `INSTALL.md` workflow update ✅ (Task 7)

### Discovery Report
- [ ] Append entry vào `docs/DISCOVERIES.md` (TẠO MỚI nếu chưa có ở sos-kit):
  - Assumption phiếu nào CORRECT (đặc biệt: hooks behavior, agent envelope vẫn enforce ở RESPOND)
  - Assumption phiếu nào WRONG (Tầng 2 self-adapted hay Tầng 1 đã debate)
  - Edge cases tìm thấy lúc implement (e.g., phiếu version bump conflict nếu Architect crash giữa chừng — recovery như nào?)
  - Docs nào cập nhật để khớp reality

---

## Effort estimate

- Task 1 (template): 30 phút
- Task 2 (worker.md, 2 files): 1.5h
- Task 3 (architect.md, 2 files): 1.5h
- Task 4 (ORCHESTRATION.md mới): 2h (cẩn thận, có example session)
- Task 5 (HANDOFF.md): 45 phút
- Task 6 (sync script + agents/README): 1h
- Task 7 (INSTALL.md): 45 phút
- Task 8 (README.md): 15 phút
- Manual smoke test (debate flow on temp project): 1.5h

**Tổng: ~9.5h** — half-day + half-day. Recommend chia 2 PR:
- **PR 1 (foundation):** Task 1 + Task 2 + Task 3 + Task 6 (template + 2 agent files + agents/ source-of-truth) — phần code-shape, không user-facing
- **PR 2 (docs + integration):** Task 4 + Task 5 + Task 7 + Task 8 + smoke test — public-facing wiring

---

## Notes for Worker

- Phiếu này **dogfood-aware**: bản thân phiếu là design của debate flow, anh sẽ dùng debate flow ngay từ phiếu kế tiếp. Cẩn thận với "boostrap moment."
- Voice quan trọng: agents/*.md công khai (English-neutral, "Chủ nhà"), .claude/agents/*.md riêng Denny ("Sếp"). Đừng vô tình homogenize.
- Nếu phát hiện anchor sai HOẶC architect-guard.sh logic không đủ chặt cho RESPOND mode (e.g., cho phép Read .md trong src/ — có thể rò rỉ vision-related markdown trong source folders) → escalate Tầng 1.
- Effort estimate là rough — Discovery Report sẽ note chỗ nào off.
