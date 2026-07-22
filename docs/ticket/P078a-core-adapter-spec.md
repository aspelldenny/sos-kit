# PHIẾU P078a: core adapter-shared serialization contract (`core/STATE.md`)

> **Loại:** Feature (docs-spec thuần, no code)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — core contract surface; sai thì LAN sang CẢ Claude adapter (P076) LẪN Codex adapter (P078b). Core semantics = AUTO Tầng 1.)
> **Lane:** Guarded — docs-spec đa-mục (6 format canonical), khai token trần theo brief. Không cap dòng.
> **Ảnh hưởng:** `core/STATE.md` (NEW), `SOS.md`, `core/README.md`, `core/ASSETS.md`, `CLAUDE.md`, `CHANGELOG.md`
> **Dependency:** P075 (portable core) DONE, P076 (Claude adapter parity) DONE. Blocks P078b (Codex adapter).

---

## Context

### Vấn đề hiện tại

Codex Adapter Discovery Report (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md`, dòng 26-28) liệt kê **Core UNDER-SPECIFICATION**: một loạt cơ chế mà CẢ Claude LẪN Codex adapter đều cần để render nhất quán, nhưng `core/**` hiện KHÔNG specify. Chúng đang **implicit** trong wiring `.claude/` của Claude (markers `.sos-state/`, Debate Log prose, TICKET_TEMPLATE convention, orchestrator hard rules). Codex adapter (P078b) không có `.claude/` để đọc → nếu core không nâng các format này lên thành canonical runtime-neutral, hai adapter sẽ render lệch nhau (asymmetry), phá cam kết P075 "portable core = 2 adapter cùng semantics".

Report cũng xác nhận (dòng 24): mọi vùng semantic của core "Clear"; các under-spec này là **adapter-rendering artifacts** (format/path/marker), KHÔNG phải core contradiction. Nhiệm vụ = codify format, KHÔNG đổi semantics.

### Giải pháp

Thêm **1 file core mới `core/STATE.md`** = "Portable Serialization Contract" — nơi định nghĩa canonical *machine-readable format + path/schema* cho các artifact mà mọi adapter phải render giống nhau. Tách bạch rõ:
- `core/WORKFLOW.md` = **semantics** của state/transition (đã có, KHÔNG đụng nội dung).
- `core/POLICY.md` = **authority/safety** semantics (đã có, KHÔNG đụng nội dung).
- `core/STATE.md` (MỚI) = **serialization** của các thứ trên: format bytes + path canonical mà adapter đọc/ghi.

**Nguyên tắc CHỐT (in nguyên vào đầu STATE.md):**
1. **Runtime-NEUTRAL** — mô tả CÁI GÌ (canonical path/fields/format), adapter nói LÀM SAO. Mỗi mục có 1 note "an integration renders this via its own mechanism" (neutral). TUYỆT ĐỐI không chứa token host: không `Claude`/`Codex`/`.claude`/`.codex`/`.sos-state`/`AskUserQuestion`/`PreToolUse`/`launchd`/`gh`.
2. **BACKWARD-COMPATIBLE** — spec phải KHỚP hành vi Claude hiện tại (orchestrator.md + TICKET_TEMPLATE + WORKFLOW.md). Codify cái đang có, KHÔNG đổi behavior. Nếu phát hiện mâu thuẫn → FLAG trong Discovery, KHÔNG tự đổi Claude.
3. **Chỉ lift cái THẬT SỰ shared** — cái adapter-specific (marker filename, hook event name) để adapter. STATE.md chỉ giữ canonical.

**Quyết decomposition (Architect chốt):** 1 phiếu, KHÔNG tách. Đây là 1 file cohesive; tách 6 format qua 2 phiếu tạo section-ordering/merge pain vô ích. Guarded lane = size không phải gate. 4 mục lớn hơn (tier-classifier-owner, concurrent-ownership lock/worktree, publish-actor division, backlog serialization) **DEFER** sang follow-up P078a2 (xem "Files KHÔNG sửa" + note cuối) — không phải split của phiếu này, mà là scope-out có chủ đích (đúng cái Sếp cho phép defer).

**Quyết layout (Architect chốt):** file MỚI `core/STATE.md` thay vì nhồi WORKFLOW.md/POLICY.md — vì (a) 6 format là 1 concern riêng (serialization) đáng 1 home, (b) nhồi vào 2 file có sẵn làm phình vô tổ chức + trộn semantics với format, (c) SOS.md canonical map thiết kế để mỗi concern 1 file.

### Scope
- CHỈ tạo `core/STATE.md` + cập nhật 5 doc tham chiếu (SOS.md, core/README.md, core/ASSETS.md, CLAUDE.md, CHANGELOG.md).
- KHÔNG đụng nội dung semantic của WORKFLOW.md / POLICY.md / ROLES.md (chỉ được thêm 1 dòng "see also STATE.md" nếu cần, KHÔNG sửa rule).
- KHÔNG đụng bất kỳ `.claude/`, `agents/*.md`, `adapters/`, `crates/`, `scripts/`.
- KHÔNG đổi hành vi Claude. Mismatch phát hiện = FLAG, không fix.

---

## Task 0 — Verification Anchors

> Architect docs-only: các anchor `[verified]` = đã Read; `[needs Worker verify]` = Worker phải grep/mở file xác nhận trước khi port. Cite RANGE, không count.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Codex under-spec list = 11 mục tại report dòng 26-28 (ticket path/schema · state-file 6 field · backlog serial · approval-record+authority · tier-classifier owner · edit-allowlist glob/normalize/symlink/amend · concurrent lock/worktree · publish actor · review-trigger map · BLOCKED storage+resume) | Read `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:26-28` | ✅ `[verified]` — 6 lift-now + defer rest |
| 2 | State-file 6 field canonical = `ticket/version/state/approved_version/previous_state/blocked_reason` | Report dòng 27 | ✅ `[verified]` |
| 3 | Ticket path canonical = `docs/ticket/P<NNN>-<slug>.md` (active) → `docs/ticket/done/`; ID = `P`+3 digit | Read `phieu/README.md:100-118` + `phieu/TICKET_TEMPLATE.md:1-6` | ✅ `[verified]` |
| 4 | `.docs-gate.toml` `ticket_dir = "docs/ticket"` (canonical value, khớp path anchor #3) | Read `.docs-gate.toml:36-43` | ✅ `[verified]` line 39 |
| 5 | Ticket schema (required sections) = header{Loại,Ưu tiên,Tầng,Lane,Ảnh hưởng,Dependency} · Context · Task 0 · Debate Log · Nhiệm vụ · Files cần sửa/KHÔNG sửa · Luật chơi · Nghiệm thu | Read `phieu/TICKET_TEMPLATE.md` full | ✅ `[verified]` toàn file |
| 6 | Approval binds EXACT ticket version; owner OR bounded-delegate mutation authority; silence/prior-approval/passing-tests KHÔNG implicit approve | Read `core/WORKFLOW.md:52-56` + `core/POLICY.md:34-45` | ✅ `[verified]` |
| 7 | Claude approval-record render = "Approved by Chủ nhà: [date]" tại Debate Log Final consensus + gate | Read `phieu/TICKET_TEMPLATE.md:110-113` | ✅ `[verified]` (Claude render — STATE.md phải neutral hóa) |
| 8 | BLOCKED side-state đã ghi 4 field trong WORKFLOW: previous state · reason · unblock-owner · safe resume state | Read `core/WORKFLOW.md:17-19` | ✅ `[verified]` — STATE.md formalize storage/format của 4 field này |
| 9 | edit-allowlist semantics: `edit_allow` narrow+explicit / `verify_read` broader / edit ngoài allowlist STOP tới khi amend contract | Read `core/POLICY.md:68-77` + `core/WORKFLOW.md:62-64` | ✅ `[verified]` — glob/normalize/symlink/amend syntax = phần STATE.md thêm |
| 10 | review-trigger surfaces (diff→gate): auth/session/permission/privacy · `src/`\|`app/`\|`lib/` · schema/migrations · `.env*` (trừ `.env.example`) · middleware · webhook · `INV-LOCAL-*` | Read `agents/orchestrator.md:81-83` (Claude render — STATE.md neutral hóa thành surface→required-gate map) | ✅ `[verified]` |
| 11 | Claude state hiện KHÔNG có per-ticket machine-readable state-file; state truth = Debate Log (prose) + markers; `.sos/state.toml` = adoption-level (`phase="INIT"`), KHÔNG per-ticket | Read `.sos/state.toml` + `agents/orchestrator.md:21` ("State truth = phiếu Debate Log") | ✅ `[verified]` — state-file = machine PROJECTION mới; Debate Log vẫn authoritative (backward-compat) |
| 12 | core hiện = 5 file (README/ROLES/WORKFLOW/POLICY/ASSETS); SOS.md canonical map liệt 5 dòng | Read `core/` glob + `SOS.md:5-15` | ✅ `[verified]` — thêm dòng thứ 6 |
| 13 | `core/**` = PORTABLE class trong ASSETS portable table | Read `core/ASSETS.md:16-18` | ✅ `[verified]` — STATE.md nằm dưới `core/**` đã covered; xác nhận không cần dòng mới HAY cần dòng riêng | ✅ `[verified]` (`SOS.md, core/** ` dòng 18 đã bao — nhưng thêm explicit note nếu muốn) |
| 14 | guard scripts xử lý symlink thế nào khi resolve edit-allowlist path (normalization/symlink-follow) | `grep -n "realpath\|readlink\|symlink" scripts/architect-guard.sh scripts/orchestrator-guard.sh` | ⏳ `[needs Worker verify]` — quyết symlink-normalization canonical trong STATE.md phải khớp Claude thực tế; nếu Claude không normalize → spec ghi "adapter-defined, canonical = repo-relative logical path" |
| 15 | CLAUDE.md repo-structure block liệt `core/{README,ROLES,WORKFLOW,POLICY,ASSETS}.md` ở đâu để thêm STATE | `grep -n "core/{README" CLAUDE.md` HOẶC grep `ASSETS.md` trong CLAUDE.md repo-structure | ⏳ `[needs Worker verify]` — CLAUDE.md dài; Worker định vị dòng chính xác trước khi Edit |

**Anchor ❌:** không có. #14, #15 = `[needs Worker verify]` (Architect docs-only, không grep được scripts/CLAUDE.md-body chính xác).

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

Docs-only phiếu → rollback = `git checkout core/STATE.md SOS.md core/README.md core/ASSETS.md CLAUDE.md CHANGELOG.md`.

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Worker accepted V1 — no Tầng 1 challenges.**

**Anchor verification (recap Task 0):**
- #1-#13: ✅ confirmed as Architect verified (re-grepped: `phieu/README.md:100-118`, `phieu/TICKET_TEMPLATE.md` full, `core/WORKFLOW.md:49,52-56,17-19`, `core/POLICY.md:68-77`, `agents/orchestrator.md:81-83`, `core/ASSETS.md:16-18`).
- #14 (symlink): ✅ `[verified]` — `grep -n "realpath\|readlink\|symlink" scripts/architect-guard.sh scripts/orchestrator-guard.sh` finds ZERO normalization calls; only a prose residual comment (`scripts/orchestrator-guard.sh:57`). Guards match on string-stripped `NORMALIZED_PATH` (`./` + `$PWD` prefix strip), no `realpath`/`readlink`. Confirms Claude does NOT resolve symlinks — the phiếu's proposed fail-safe wording ("matching is on the repo-relative logical path as written… symlink resolution is integration-defined and MUST be reported") is accurate, no change needed.
- #15 (CLAUDE.md core list location): ⚠️ `[partially wrong]` — `grep -n "core/{README\|ASSETS.md\|core/README\|core/ROLES\|core/WORKFLOW\|core/POLICY" CLAUDE.md` → **zero hits**. CLAUDE.md's repo-structure tree block does NOT currently list `core/` as a directory at all (checked full tree block, only `crates/` and `adapters/claude/` mention "core" as a word inside comments). This is pre-existing drift, not caused by this phiếu. Worker EXECUTE-time impact: Task 9's CLAUDE.md edit cannot "append STATE.md to an existing core file list" — Worker must add a **new `core/` tree entry** (5 existing files + STATE.md) to the repo-structure block. Still within declared "Files cần sửa" (CLAUDE.md already listed) — not a scope break, just a note for Worker EXECUTE + Discovery.

**Coverage check (oracle 1):** Codex report (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:26-28`) lists 11 under-spec items (ticket storage-path, ticket schema, state-file format, backlog serialization, approval-record, tier-classifier owner, edit-allowlist glob/norm/symlink/amend, concurrent lock/worktree, publish-actor, review-trigger map, BLOCKED format). Phiếu's 6 lift-now sections (Task 2-7) cover 7 of these (Task 2 combines ticket-storage + ticket-schema); the 4 remainder (backlog, tier-classifier, concurrent, publish-actor) are explicitly named in Task 8 Deferred-scope. 7+4=11 — full coverage, nothing silently dropped.

**Neutrality check (oracle 2 — CRITICAL):** read all Task 1-8 "Thêm:" (file-content) blocks — none embed `Claude/Codex/.claude/.sos-state/AskUserQuestion/PreToolUse` etc. The phiếu is careful to keep those tokens confined to "Lưu ý:" (Worker-instruction) prose, e.g. Task 4/5/6 explicitly say "KHÔNG name AskUserQuestion/…", "STATE.md giữ neutral shape". Design is sound; residual risk is execution-time discipline (Worker must not let Lưu ý tokens leak into the actual written section) — not a phiếu defect.

**Backward-compat check (oracle 3 — CRITICAL):**
- State-file OPTIONAL projection (Task 3): text explicitly says "ticket debate log remains the authoritative human record… machine state artifact is a derived projection… an integration KHÔNG bắt buộc phải materialize file riêng" — matches `core/WORKFLOW.md:49` ("ticket debate log is the authoritative record") exactly. Debate-Log authority preserved.
- Ticket schema (Task 2) vs `phieu/TICKET_TEMPLATE.md`: header block / Context / Task 0 / Debate Log / Nhiệm vụ / Files cần sửa-không sửa / Luật chơi / Nghiệm thu — matches 1:1.
- Approval record (Task 4) vs `core/WORKFLOW.md:52-56` + `core/POLICY.md:33-44`: exact-version bind, owner/bounded-delegate only, silence≠approve — matches.
- Edit-allowlist (Task 5) vs `core/POLICY.md:67-76`: `edit_allow` narrow / `verify_read` broader / edit outside allowlist stops until amended — matches.
- Review-trigger map (Task 6) vs `agents/orchestrator.md:83`: surface list (auth/session/permission/privacy, `src/|app/|lib/`, schema/migrations, `.env*` except `.env.example`, middleware, webhook, `INV-LOCAL-*`) — matches verbatim, correctly abstracted to neutral surface-classes in the spec text.
- BLOCKED format (Task 7) vs `core/WORKFLOW.md:17-19,19`: previous_state/reason/unblock_owner/resume_state — matches the 4-field prose.
- ASSETS.md (Task 9 CLAUDE.md/ASSETS.md note): confirmed `core/**` already listed under `PORTABLE` (`core/ASSETS.md:16-18` — actually rendered as row `SOS.md, core/**`) — no new ASSETS.md line strictly required, matches anchor #13.

No mismatch found that would require changing Claude behavior. No objection.

**Objections (Tầng 1 only):** none.

**Status:** ✅ ACCEPTED — ready for Chủ nhà approval gate.

### Final consensus
- Phiếu version: V<N>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1: Tạo `core/STATE.md` — header + nguyên tắc

**File:** `core/STATE.md` (NEW)

**Thêm:** Mở đầu file:
- Title: `# Portable Serialization Contract`
- 1 đoạn: file này sở hữu *canonical machine-readable format + path/schema* của các artifact mà mọi integration phải render giống nhau. Nó KHÔNG định nghĩa lại semantics — state/transition thuộc `core/WORKFLOW.md`, authority/safety thuộc `core/POLICY.md`, role thuộc `core/ROLES.md`. STATE.md serialize các contract đó.
- Section `## Neutrality and compatibility rules`:
  1. Every format below is **host-neutral**: it names fields, paths and shapes, not host tools or file layouts. An integration renders each artifact through its own mechanism and must not require a specific host to remain meaningful.
  2. These formats **describe existing behavior**; an integration must render them without changing established lifecycle behavior. A host limitation that cannot represent a format must be reported per `core/README.md` integration obligations, not silently weakened.
  3. Only genuinely shared artifacts live here. Host-specific rendering detail (concrete marker filenames, event names, prompt wording) stays in the integration.

**Lưu ý:** TUYỆT ĐỐI không token host (xem Constraint 1). Đây là điểm oracle neutrality-grep sẽ soi.

### Task 2: `## Ticket storage and schema`

**File:** `core/STATE.md`

**Thêm section định nghĩa canonical (per anchor #3,#4,#5):**
- **Path:** active ticket = `docs/ticket/P<NNN>-<slug>.md`; completed = `docs/ticket/done/P<NNN>-<slug>.md`. `<NNN>` = zero-padded 3-digit monotonic ID; `<slug>` = kebab-case. Canonical directory value is `docs/ticket` (một integration MAY expose nó qua config nhưng default là giá trị này).
- **Active selection:** at most one ticket is the active delivery unit at a time (khớp WORKFLOW.md one-delivery-unit rule). Core specifies the *semantic* (one active unit); an integration renders the pointer through its own mechanism (working-tree/branch identity, a state artifact, hoặc explicit selection) — note neutral, KHÔNG name mechanism cụ thể.
- **Schema (required sections, thứ tự canonical):** header block {type, priority, tier, lane, affected-files, dependency} · context · verification anchors (Task 0) · debate log · task list · edit-and-verify file lists · constraints · acceptance. Mô tả mỗi section 1 dòng ý nghĩa. Ghi rõ: tier axis (consequence) và lane axis (budget) độc lập.

**Lưu ý:** Đây codify TICKET_TEMPLATE.md hiện tại → backward-compat. Nếu Worker thấy TICKET_TEMPLATE có section KHÔNG map được (vd "Skills consulted" optional) → note optional, KHÔNG bỏ.

### Task 3: `## Lifecycle state artifact`

**File:** `core/STATE.md`

**Thêm (per anchor #2, #11):**
- Định nghĩa canonical machine-readable fields cho state của 1 delivery unit: `ticket` (id/path), `version` (approved-against contract version), `state` (một trong các state của WORKFLOW.md), `approved_version` (version đã được bind approval, null nếu chưa), `previous_state` (cho BLOCKED resume), `blocked_reason` (null nếu không BLOCKED).
- **Authoritative-vs-projection rule (BACKWARD-COMPAT — CRITICAL):** the **ticket debate log remains the authoritative human record** of the draft/challenge/approval history (khớp WORKFLOW.md:49 + anchor #11). Machine state artifact này là a **derived projection** an integration MAY materialize để enforce gates deterministically. Where a projection and the debate log disagree, the debate log governs history; the projection must be corrected. Một integration KHÔNG bắt buộc phải materialize file riêng nếu nó derive state từ ticket trực tiếp.
- Value vocabulary của `state` = liệt kê tên state neutral khớp WORKFLOW.md States table (INTAKE/DRAFT/CHALLENGE/APPROVAL/EXECUTE/DISCOVERY/REVIEW/DELIVERED/BLOCKED) — reference WORKFLOW.md, KHÔNG tự định nghĩa lại.

**Lưu ý:** Đây là mục backward-compat rủi ro nhất. Claude KHÔNG có per-ticket state-file (anchor #11) — spec phải nói state-file là OPTIONAL projection, nếu không sẽ mâu thuẫn orchestrator hard-rule "state truth = Debate Log". Nếu Worker thấy wording nào ép buộc materialize → FLAG.

### Task 4: `## Approval record`

**File:** `core/STATE.md`

**Thêm (per anchor #6, #7):**
- Canonical approval record binds **exactly one ticket version** và ghi: approving actor (owner hoặc named bounded delegate), the bound version, timestamp, và scope-of-delegation nếu delegate. 
- **Mutation authority:** chỉ owner (hoặc bounded delegate theo `core/POLICY.md`) may create an approval record. Silence, prior roadmap approval, hoặc passing tests KHÔNG constitute approval (khớp WORKFLOW.md:55). Any material scope/architecture change invalidates the record → return to draft/challenge.
- Note neutral: an integration renders the approval interaction + record location through its own mechanism (an interactive gate + a recorded marker in the ticket/state artifact). KHÔNG name `AskUserQuestion`/Debate-Log-line.

**Lưu ý:** Claude render = AskUserQuestion gate + "Approved by Chủ nhà: [date]" line (anchor #7) — đó là RENDER, STATE.md giữ neutral shape.

### Task 5: `## Edit allowlist and verify scope`

**File:** `core/STATE.md`

**Thêm (per anchor #9, #14):**
- Canonical: một ticket khai 2 tập path — `edit_allow` (narrow, explicit) và `verify_read` (broader). Asymmetry khớp POLICY.md:68-77.
- **Glob semantics:** allowlist entries là repo-relative path patterns; định nghĩa canonical matching (literal path hoặc glob `**`/`*`), **path normalization** = repo-relative logical path (leading `./` stripped, no trailing slash). 
- **Symlink handling:** ⚠️ per anchor #14 `[needs Worker verify]` — nếu Claude guard KHÔNG normalize/resolve symlink → spec canonical = "matching is on the repo-relative logical path as written in the ticket; symlink resolution is integration-defined and MUST be reported if it can bypass the allowlist." (Fail-safe wording — không ép Claude đổi behavior.)
- **Amendment syntax:** một edit ngoài allowlist STOP execution tới khi ticket contract được amend (khớp POLICY.md:74 + WORKFLOW.md:63). Canonical: amendment = add path vào `edit_allow` của ticket + bump version; KHÔNG có "verbal" widening.

**Lưu ý:** symlink là mục duy nhất Architect không verify được — Worker grep guard scripts (anchor #14), nếu Claude behavior khác spec đề xuất → dùng fail-safe wording ở trên (integration-defined + must-report), KHÔNG đổi Claude.

### Task 6: `## Review trigger map`

**File:** `core/STATE.md`

**Thêm (per anchor #10):**
- Canonical map: **diff surface → required review gate**. Liệt kê neutral surfaces: authentication/session/permission/privacy paths · implementation source directories · schema/migration files · secret/environment files (trừ example templates) · request middleware · external webhook handlers · files enforcing a project-local invariant.
- Rule: một diff chạm surface trong map → the configured invariant/security review gate BẮT BUỘC pass trước merge/publish (khớp WORKFLOW.md:73 + orchestrator hard-rule anchor #10). Pattern-match là pattern-match — KHÔNG tự judge "scope nhỏ/docs-only".
- Note neutral: concrete path globs + gate command là integration/project config; core specifies WHICH surface classes trigger review, không phải glob cụ thể của 1 host.

**Lưu ý:** orchestrator.md dùng `src/|app/|lib/`, `.env*`, `gh pr` — đó là Claude render. STATE.md giữ surface CLASS neutral ("implementation source directories", "secret/environment files").

### Task 7: `## Blocked state format`

**File:** `core/STATE.md`

**Thêm (per anchor #8):**
- Canonical BLOCKED record (formalize WORKFLOW.md:17-19 four fields): `previous_state` (state để resume), `reason` (blocking decision/capability), `unblock_owner` (role/actor giải block — thường owner), `resume_state` (safe state để tiếp tục, thường = previous_state trừ khi cần re-verify).
- **Storage + resume:** BLOCKED record lives with the ticket (in the ticket hoặc its state projection). On unblock: verify the blocking condition resolved, then transition to `resume_state`. A blocked delivery unit MUST NOT be silently dropped — nó persist tới khi resolved hoặc explicitly abandoned by owner.
- Note neutral về persistence mechanism (ticket section hoặc state artifact) — KHÔNG name host file.

**Lưu ý:** WORKFLOW.md đã có 4 field ở dạng prose; STATE.md formalize thành named-field record. KHÔNG mở rộng semantics, chỉ serialize.

### Task 8: `## Deferred scope` (in-file scope note)

**File:** `core/STATE.md`

**Thêm section cuối** liệt kê explicit 4 mục under-spec **CHƯA** codify ở đây, để P078b biết KHÔNG dựa vào chúng và để có audit trail:
- tier-classifier ownership (role nào phân Tầng) — semantics đã ở POLICY authority tiers; formalize sau nếu Codex cần.
- concurrent-ownership lock / worktree isolation format — lớn (worktree semantics khác host); follow-up.
- publish-actor division (ai commit/push/merge) — semi-covered WORKFLOW.md delivery; follow-up.
- backlog serialization format — chưa cần cho P078b render.

Ghi: "These remain integration-defined until a follow-up ticket lifts them; an integration must not assume a core-canonical format for them yet."

### Task 9: Cập nhật 5 doc tham chiếu (DOCS GATE)

**File:** `SOS.md` — Canonical map table (dòng 7-14): thêm 1 dòng `| Machine-readable serialization of tickets, state, approval and review | core/STATE.md |`. Conflict-resolution list nếu cần: STATE.md governs serialization only (dưới WORKFLOW/POLICY/ROLES trong precedence).

**File:** `core/README.md` — "What belongs here" (dòng 19-24): thêm bullet "canonical serialization formats for shared tickets, state, approval, edit-scope, review triggers and blocked records (`STATE.md`)". Xác nhận không vi phạm "What does not belong here" (host file layout) — STATE.md là neutral format, KHÔNG host layout.

**File:** `core/ASSETS.md` — portable assets table (dòng 16-18): `core/**` đã bao `STATE.md` (anchor #13). Cân nhắc thêm explicit note dòng `SOS.md, core/**` đã cover — Worker quyết: nếu table liệt từng-file thì thêm; nếu dùng `core/**` glob thì KHÔNG cần dòng mới (chỉ verify STATE.md nằm dưới PORTABLE, không phải TRANSITIONAL_MIXED).

**File:** `CLAUDE.md` — repo-structure block: thêm `STATE.md` vào list core file. `[needs Worker verify]` anchor #15 — grep vị trí `core/{README,ROLES,WORKFLOW,POLICY,ASSETS}` HOẶC dòng liệt ASSETS.md, thêm STATE. Nếu CLAUDE.md có prose "core = 5 file" → cập nhật thành 6.

**File:** `CHANGELOG.md` — entry mới trên cùng: "P078a — add `core/STATE.md` portable serialization contract (ticket storage/schema, lifecycle state artifact, approval record, edit allowlist, review trigger map, blocked format); runtime-neutral, backward-compatible with Claude adapter; unblocks P078b Codex adapter."

**Lưu ý:** KHÔNG đụng WORKFLOW.md/POLICY.md/ROLES.md content. Nếu muốn thêm "see also core/STATE.md" pointer vào WORKFLOW/POLICY → tối đa 1 dòng cross-ref, KHÔNG sửa rule. Cân nhắc (guidance, không bắt buộc): nếu Worker thấy `docs/HANDOFF.md`/`docs/LAYERS.md` formalize marker/approval handoff theo cách STATE.md giờ chuẩn hóa → FLAG trong Discovery cho follow-up, KHÔNG tự sửa phiếu này (giữ scope).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `core/STATE.md` | NEW — Task 1-8: 7 canonical format sections + neutrality rules + deferred-scope note |
| `SOS.md` | Task 9: +1 dòng canonical map |
| `core/README.md` | Task 9: +1 bullet "what belongs" |
| `core/ASSETS.md` | Task 9: verify/note STATE.md PORTABLE |
| `CLAUDE.md` | Task 9: +STATE.md vào core file list (Worker định vị) |
| `CHANGELOG.md` | Task 9: entry P078a |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `core/WORKFLOW.md` | STATE.md serialize đúng states/BLOCKED/approval semantics, KHÔNG mâu thuẫn. KHÔNG sửa content (tối đa 1 dòng cross-ref) |
| `core/POLICY.md` | STATE.md serialize đúng edit-scope/authority/delegation, KHÔNG sửa rule |
| `core/ROLES.md` | Capability vocab (edit_files/publish_changes...) referenced đúng, KHÔNG sửa |
| `phieu/TICKET_TEMPLATE.md` | Schema canonical (Task 2) khớp template hiện tại — mismatch = FLAG Discovery |
| `agents/orchestrator.md` | review-trigger/marker/approval Claude render khớp neutral spec — mismatch = FLAG, KHÔNG đổi |
| `scripts/*guard.sh` | Anchor #14 symlink behavior — grep only, quyết fail-safe wording |
| `adapters/`, `.claude/`, `crates/` | KHÔNG chạm |

---

## Luật chơi (Constraints)

1. **Neutrality tuyệt đối:** `core/STATE.md` KHÔNG chứa token host: `Claude`, `Codex`, `.claude`, `.codex`, `.sos-state`, `AskUserQuestion`, `PreToolUse`, `launchd`, `gh`, `Quản đốc`/`Thợ`/`Kiến trúc sư` (role display-name — dùng role ID neutral `owner/orchestrator/architect/worker` như ROLES.md). Oracle grep sẽ enforce.
2. **Backward-compat:** mỗi format PHẢI khớp hành vi Claude hiện tại (anchor #3,5,6,7,8,9,10). Phát hiện Claude làm khác spec → FLAG Discovery, KHÔNG đổi Claude, KHÔNG đổi orchestrator/TICKET_TEMPLATE.
3. **Semantics KHÔNG đổi:** STATE.md chỉ serialize. Reference WORKFLOW/POLICY/ROLES cho semantics, KHÔNG định nghĩa lại state/authority/role. Không sửa content 3 file đó.
4. **Chỉ lift 6 mục:** ticket storage/schema · lifecycle state artifact · approval record · edit allowlist · review trigger · blocked format. 4 mục còn lại (tier-classifier/concurrent/publish/backlog) = deferred-scope note, KHÔNG spec.
5. **State-file = OPTIONAL projection:** wording KHÔNG được ép integration materialize file riêng (phá backward-compat với Claude Debate-Log-as-truth). Debate log authoritative; state artifact derived.
6. **Cite RANGE khi reference doc khác** (vd `core/WORKFLOW.md:52-56`), KHÔNG count.

---

## Nghiệm thu

### Automated (oracle: docs-spec, KHÔNG cargo)
- [ ] **Coverage checklist:** 6 mục lift-now đều có section trong `core/STATE.md` (ticket storage/schema · lifecycle state artifact · approval record · edit allowlist · review trigger map · blocked format). 4 mục defer có trong Deferred-scope note. `[oracle: coverage checklist vs Codex-report dòng 26-28]` SOUND.
- [ ] **Neutrality grep:** `grep -niE 'claude|codex|\.sos-state|asktuserquestion|pretooluse|launchd|\bgh\b|quản đốc|kiến trúc|thợ' core/STATE.md` → **zero match** (case-insensitive). `[oracle: neutrality grep]` SOUND.
- [ ] **Claude-compat diff:** mỗi canonical format cross-check với source Claude (ticket path↔phieu/README:100-118 · schema↔TICKET_TEMPLATE · approval↔WORKFLOW:52-56 · edit-scope↔POLICY:68-77 · review-trigger↔orchestrator:81-83 · blocked↔WORKFLOW:17-19). Mismatch nào = ghi Discovery FLAG, KHÔNG auto-fix. `[oracle: Claude-compat diff, PARTIAL]` — SÀNG được đa số; symlink (anchor #14) residual cần Worker grep.

### Manual Testing
- [ ] Đọc STATE.md end-to-end: một người CHỈ có `core/**` (không có `.claude/`) có render được 6 format không? (proxy cho Codex adapter author.)
- [ ] SOS.md canonical map + core/README.md "what belongs" phản ánh STATE.md; không dòng nào `core/** → adapters/**` (dependency 1 chiều).

### Regression
- [ ] `grep -rn 'adapters/' core/` → zero (core không name adapter — README.md:12-16 invariant).
- [ ] WORKFLOW.md/POLICY.md/ROLES.md content KHÔNG đổi (`git diff` chỉ được có cross-ref 1 dòng nếu thêm, hoặc no-change).

### Docs Gate
- [ ] `SOS.md` — canonical map +1 dòng STATE.md
- [ ] `core/README.md` — "what belongs" +1 bullet
- [ ] `core/ASSETS.md` — STATE.md PORTABLE verified/noted
- [ ] `CLAUDE.md` — core file list +STATE.md (repo-structure block)
- [ ] `CHANGELOG.md` — entry P078a

### Discovery Report
- [ ] Write `docs/discoveries/P078a.md`:
  - Anchors CORRECT/WRONG (file:line)
  - **Backward-compat FLAGS:** mọi chỗ Claude làm khác neutral spec (đặc biệt state-file materialize + symlink anchor #14) — liệt kê cho P078b + future Claude-render reconcile
  - Symlink resolution (anchor #14) — Claude guard behavior thực tế + wording chốt
  - Deferred 4 mục — confirm note có trong STATE.md
  - Docs updated (Tầng 1): list; hay "N/A"
  - Tier escalations: "None" nếu không
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
