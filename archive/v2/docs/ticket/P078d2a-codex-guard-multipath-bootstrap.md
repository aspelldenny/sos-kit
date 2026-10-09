# PHIẾU P078d2a: Codex adapter enforcement/security fixes — multi-path guard bypass + approval bootstrap + spawn caveat (#5/#6/#7)

---

> **Loại:** Bugfix (security)
> **Ưu tiên:** P1 (#6 = live security hole → vá SỚM)
> **Tầng:** 1 — chạm enforcement logic (guard path-parsing + approval gate) = security boundary. AUTO Tầng 1 dù diff nhỏ. Sai → LAN tới mọi Codex user + KHÔNG-đảo (bypass cho phép ghi `.env`/src/state ngoài phiếu).
> **Lane:** Guarded (no-cap). **SECURITY → CHALLENGE kỹ** (bắt buộc Worker CHALLENGE trước EXECUTE, dù sprint-delegate).
> **Ảnh hưởng:** `crates/sos-adapter-codex/src/templates.rs` — guard content-fn (path-extraction ở `:379/481/538/606` `[needs Worker verify]`), approval-gate content-fn (`:620` `[needs Worker verify]`), AGENTS.md content-fn (spawn caveat) + test module.
> **Dependency:** P078d1 (startup-schema fixes) SHIPPED. Spec = `docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md` #5/#6/#7. **Sibling d2b = #4 (marker fail-open) DEFER — cần SubagentStart probe.**

---

## Context

### Vấn đề hiện tại

P079 live-dogfood (Codex CLI 0.145.0) chạy trọn một phiếu thật trên adapter Codex đã cài. Kiến trúc **SOUND** (main-thread guard enforce đúng: orchestrator-guard chặn src probe, block-env-edit chặn `.env`). Nhưng dogfood phơi **4 bug enforcement/security** (findings #4–#7). Phiếu này xử **3 bug specifiable NGAY** — KHÔNG cần Codex probe. Bug #4 (SubagentStart marker fail-open) tách sang **d2b** vì root-cause = SubagentStart matcher semantics chưa xác định (likely cần probe như apply_patch b3).

**3 bug phiếu này (tất cả `crates/sos-adapter-codex/src/templates.rs`):**

1. **#6 multi-path bypass (SECURITY HOLE — ưu tiên cao nhất).** MỌI guard (`:379/481/538/606` `[needs Worker verify]` — architect/orchestrator/env/approval) chỉ trích path ĐẦU TIÊN của apply_patch (`head -n1`). apply_patch payload cho phép **nhiều file trong 1 patch** = nhiều dòng `*** (Add|Update|Delete|Move) File: <path>` (Sếp b3 sample `[verified]`). **Bypass:** đặt một path ticket-được-phép TRƯỚC + `.sos-state/ticket-state.env` (hoặc `.env`/`src/**`) SAU → guard match first-path exemption → exit ALLOW → cho qua CẢ HAI file. **Fix:** parse MỌI path trong patch, check TẤT CẢ, **BLOCK nếu BẤT KỲ path vi phạm** (block-all-violating; exemption chỉ khi MỌI path đều exempt).

2. **#5 approval bootstrap deadlock.** approval-gate (`:620` `[needs Worker verify]`) BLOCK mọi non-ticket patch khi state-file (`.sos-state/ticket-state.env`) thiếu — **kể cả patch TẠO chính `.sos-state/ticket-state.env`** → chicken-egg: không cách nào khởi tạo state, workflow không boot được ở fresh install. **Fix (an toàn, KHÔNG mở lỗ):** hai lớp — (a) render **skeleton state-file lúc install** (non-clobber, không đè state thật) để nhánh missing-file gần như không bao giờ chạm; (b) approval-gate **self-bootstrap exemption**: cho phép patch tạo/sửa **CHỈ** `.sos-state/ticket-state.env` khi state thiếu. **Exemption này CHỈ an toàn VÌ #6 land cùng phiếu** — all-path check chặn tổ hợp "ticket-state.env + path độc". Đây là lý do #5 và #6 phải cùng một phiếu (coupling cứng).

3. **#7 spawn caveat (doc).** First custom-agent spawn fail: "full-history forked agents inherit parent agent type; omit agent_type or spawn without full-history fork." AGENTS.md render thiếu caveat này (orchestrator guidance). **Fix:** thêm một dòng caveat vào AGENTS.md content-fn.

### Giải pháp

Sửa enforcement logic trong `templates.rs` guard/approval content-fn + AGENTS render + tests. **Additive:** chỉ đổi guard shell-script content (string emit trong content-fn) + approval-gate + AGENTS + test module. KHÔNG đụng engine/install-engine/core/adapter-claude, KHÔNG đổi render 3 startup-file (d1), KHÔNG đụng marker lifecycle (#4 = d2b).

**Oracle mấu chốt** `[oracle: multi-path guard block-all-violating + bootstrap-safe + real-payload]`:
- **#6** → **mock-payload test chạy REAL guard script** (render guard → feed apply_patch fixture qua tool_input.command → assert exit code). Hai fixture bắt buộc: (a) **REAL single-path apply_patch (Sếp b3)** allowed-ticket → guard ALLOW (không regress); (b) **multi-path bypass fixture** — ticket-allowed path TRƯỚC + `.env`/`src/**`/`.sos-state/ticket-state.env` SAU → guard **BLOCK** (chứng minh all-path check). Áp cho MỌI guard có exemption.
- **#5** → **bootstrap-init test**: state-file thiếu + patch tạo CHỈ `.sos-state/ticket-state.env` → ALLOW; state-file thiếu + patch tạo ticket-state.env **kèm** một path độc → BLOCK (coupling với #6).
- **Negative-test (răng):** revert #6 (về head-n1) → multi-path bypass fixture chuyển từ BLOCK→ALLOW = test FAIL. revert #5 exemption → bootstrap patch chuyển ALLOW→BLOCK = test FAIL.
- **Behavioral #4** (marker) = **P079 round-2** (d2b), ngoài phiếu này.

### Scope

- CHỈ sửa: `crates/sos-adapter-codex/src/templates.rs` — guard content-fn (path-parse), approval-gate content-fn (bootstrap), AGENTS.md content-fn (caveat), install skeleton-state emit, test module cùng crate.
- KHÔNG sửa: SubagentStart/Stop marker lifecycle (`:302` = **#4 d2b**), 3 startup render-fn (config.toml/rules/hooks.json = d1 SHIPPED), engine/install-engine core logic/core/adapter-claude. **Nếu muốn đổi marker → DỪNG, đó là d2b.**

---

## Task 0 — Verification Anchors

> Architect docs-only (no Bash/Grep/src read). Line số + guard-site từ P079 findings + Codex discovery report; apply_patch format từ Sếp b3 sample. Code-site = `[needs Worker verify]`; format-fact = `[verified]`. Worker grep-verify TRƯỚC khi sửa.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | MỌI guard (architect/orchestrator/env/approval) trích path apply_patch bằng `head -n1` (first-path only) ở `templates.rs:379/481/538/606` `[needs Worker verify]` | `rg -n "head -n1\|head -1\|\\*\\*\\* .*File" crates/sos-adapter-codex/src/templates.rs` → xác nhận mỗi guard chỉ lấy path đầu | ✅ Confirmed 4/4 at `:396-399, :494-497, :551-554, :619-622` (drift ~15-20 lines from d1 comment additions, same code) |
| 2 | apply_patch multi-file payload = nhiều dòng `*** (Add\|Update\|Delete\|Move) File: <path>` giữa `*** Begin Patch` / `*** End Patch` (Move kèm `*** Move to:` dest) `[verified: Sếp b3 sample]` | So khớp REAL b3 fixture; xác nhận không có delimiter khác cho path | ✅ Confirmed via `tests/fixtures/codex-apply-patch-payloads.jsonl` + header `templates.rs:286-297`; Move-dest line still unconfirmed (not in sample) — stays `[needs Worker verify]` |
| 3 | approval-gate content-fn (`:620` `[needs Worker verify]`) BLOCK mọi non-ticket patch khi `.sos-state/ticket-state.env` missing, KHÔNG có exemption tạo state-file | `rg -n "ticket-state\|sos-state\|approved_version\|BLOCK\|deny" crates/sos-adapter-codex/src/templates.rs` (approval content-fn) | ✅ Confirmed `:634-637` unconditional `exit 2` on missing file, zero exemption |
| 4 | `.sos-state/ticket-state.env` = state-file path; format = env-style key/value (approved ticket/version/state) `[needs Worker verify]` | `rg -n "sos-state\|ticket-state\|\\.env\b" crates/sos-adapter-codex/src/templates.rs` + so `phieu/TICKET_TEMPLATE.md` snapshot block (`.sos-state`) | ✅ Confirmed `templates.rs:592-595` comment + `lib.rs:775-778` test literal `version=V2\napproved_version=V2\n` |
| 5 | AGENTS.md content-fn tồn tại + có chỗ đặt orchestrator spawn-guidance để thêm caveat `[needs Worker verify]` | `rg -n "AGENTS\|agents.md\|spawn\|subagent\|fn .*agents" crates/sos-adapter-codex/src/templates.rs` | ✅ Confirmed `agents_md()` `templates.rs:145-162`, insertion point = line 155-157 bullet list |
| 6 | Guard nào có exemption-list (ticket-path allowed) cần all-path check: architect-guard, orchestrator-guard, block-env(-edit), approval-gate `[needs Worker verify: guard nào có allow-list vs deny-only]` | `rg -n "allow\|exempt\|ticket\|docs/ticket\|\\.env" crates/sos-adapter-codex/src/templates.rs` → phân loại guard allow-list vs pure-deny | ✅ Classified: architect-guard=allow-list(ticket .md), orchestrator-guard=marker-gated, block-env=deny-only+.env.example inverse-exemption, approval-gate=ticket-exemption+version-match. All 4 need the fix — none exempt |
| 7 | Guard = Bash script emit dạng string từ content-fn, chạy standalone được với mock `tool_input.command` (env/stdin) → test feed fixture + assert exit code khả thi `[needs Worker verify]` | Render guard → chạy `bash <script>` với payload fixture; xác nhận contract đọc payload từ đâu (env var / stdin / arg) | ✅ CONFIRMED with existing proof — `lib.rs:582-616` `run_guard()` harness already does exactly this, exercised by 15+ passing tests. Oracle not speculative |

**Nếu Result có ❌ (guard đã all-path sẵn / approval đã có exemption / line lệch) → Worker DISCOVERY_REPORT + điều chỉnh; nếu bug KHÔNG tồn tại như spec → dừng + báo (có thể partially-fixed).**
**Nếu anchor #7 sai (guard KHÔNG chạy standalone được — payload đến qua cơ chế Codex-runtime không mock được) → ESCALATE: oracle #6 phải hạ xuống structural-string assert trên guard content + behavioral defer P079 round-2. Báo orchestrator trước khi tự đổi oracle.**

---

## Debate Log

> Cap = 3 turns. Security phiếu → Worker CHALLENGE bắt buộc dù sprint-delegate.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Worker accepted V1 — no blocking Tầng-1 objections.** Real-code read confirms the spec is architecturally sound and unusually low-risk to implement. One advisory note (self-closing via oracle, not blocking).

**Anchor verification (recap Task 0):**
- Anchor #1 ✅ — all 4 guards extract path via identical pattern `grep -oE '\*\*\* (Add|Update|Delete|Move) File: [^\\"]+' | head -n1 | sed ...` at `crates/sos-adapter-codex/src/templates.rs:396-399` (architect-guard), `:494-497` (orchestrator-guard), `:551-554` (block-env-edit), `:619-622` (approval-gate). Line numbers drifted ~15-20 from phiếu's `379/481/538/606` (P078d1 comment additions shifted offsets) — same code, confirmed by content match, not a mismatch.
- Anchor #2 ✅ — real fixture `crates/sos-adapter-codex/tests/fixtures/codex-apply-patch-payloads.jsonl` + header comment `templates.rs:286-297` confirm multi-file-capable V4A shape (`*** Begin Patch\n*** Add/Update/Delete File: <path>\n...\n*** End Patch`); Move destination line (`*** Move to:`) not in sample, correctly marked `[needs Worker verify]` in phiếu.
- Anchor #3 ✅ — `templates.rs:634-637`: `[ ! -f "$STATE_FILE" ]` → unconditional `exit 2`, zero exemption. Matches spec exactly.
- Anchor #4 ✅ — format confirmed both in guard comment (`templates.rs:592-595`) and test setup `lib.rs:775-778` (`version=V2\napproved_version=V2\n`), plain key=value.
- Anchor #5 ✅ — `agents_md()` at `templates.rs:145-162`, orchestrator spawn-guidance bullet at line 155-157 is the natural insertion point for the #7 caveat.
- Anchor #6 ✅ — classified: architect-guard = allow-list (ticket `.md` only), orchestrator-guard = marker-gated allow, block-env-edit = deny-only (with `.env.example` inverse-exemption), approval-gate = ticket-file exemption + version-match. All 4 need all-path parsing per Task 1 — phiếu's blanket "áp cho MỌI guard" is correct, no guard is exempt from the fix.
- Anchor #7 ✅ — CONFIRMED, not just theoretically: `lib.rs:582-616` `run_guard()` already renders a guard to a temp dir, feeds JSON on stdin via `bash <script>`, and asserts exit code — this exact harness is exercised by 15+ existing tests (`lib.rs:642-838` range). The proposed Task 4 oracle is not speculative, it's an established, already-proven pattern in this crate. Low implementation risk.

**Key structural insight (de-risks Task 1 significantly):** `grep -oE` already emits **one output line per match**, even when all matches sit on a single JSON line (which they do — Codex's `\n` inside `tool_input.command` is a literal escaped `\n`, not a real newline, so the whole patch is one grep-input-line). This means removing `| head -n1` from all 4 sites is *already* "extract every path" — Task 1 does not need a new regex, only deleting a pipe stage and wrapping the existing per-path check in a loop with block-if-any semantics. Minimal, well-bounded diff — consistent with the phiếu's "Additive" constraint.

**Advisory note (non-blocking, self-closing via oracle):**
- Claim: converting the single-path `if`-check into a multi-path loop is at risk of a classic bash pitfall — `printf '%s\n' "$PATHS" | while read -r p; do ...; exit 2; done` runs the loop body in a **subshell** (piped `while`), so `exit 2` only exits the subshell; the parent script falls through and still exits 0 — a silent no-op "fix" that keeps printing "BLOCKED" to stderr while returning ALLOW.
- Oracle: the phiếu's own Task 4 bypass-fixture test (`assert_eq!(code, 2)` via `run_guard`) is a SOUND oracle for this exact claim — if Worker falls into the pipe-subshell trap, the bypass test fails immediately (wrong exit code), not silently.
- Soundness: SOUND for this claim.
- Verdict: self-closed via oracle — no Architect respond needed. Flagging only so Worker avoids the trap directly (use `while read; do ...; done <<< "$PATHS"` herestring or a plain `for`/`readarray` loop — not a piped `while` — so `exit` propagates to the script, not a subshell).

**Secondary pointer (Tầng 2, informational, not an objection):** `lib.rs:401-409` `plan_renders_seventeen_artifacts` hard-asserts `plan.operations.len() == 17`. If Task 2's install-skeleton-state emit becomes a NEW `all_assets()` entry (18th asset) rather than folding into an existing render path, this test needs updating to 18 — already anticipated by the phiếu's own `[needs Worker verify]` note on skeleton placement (Task 2 Lưu ý). Citing the exact test site so Worker doesn't miss it if that branch is taken.

**Status:** ✅ Worker CHALLENGE complete — no Tầng-1 objections. Ready for Chủ nhà approval gate.

### Turn 2 — Chủ nhà Approval

**CHALLENGE APPROVE V1 → EXECUTE mode.** Sprint delegated by Chủ nhà via Quản đốc relay. Subshell-pitfall flag from Turn 1 acknowledged and carried into EXECUTE (loop implementation MUST NOT use `pipe | while read; ... exit` — use flag-var+check-after, `for p in $(...)`, or process-substitution `< <(...)`).

**Status:** ✅ APPROVED — phiếu stays V1 (no content changes, only execution proceeds).

### Final consensus
- Phiếu version: V1
- Approved by Chủ nhà: 2026-07-22 (via Quản đốc relay, EXECUTE dispatch)

---

## Nhiệm vụ

### Task 1: guards — parse+check MỌI apply_patch path, block-all-violating (#6, SECURITY)

**File:** `crates/sos-adapter-codex/src/templates.rs` — mỗi guard content-fn có path-extraction (`:379/481/538/606` `[needs Worker verify]`).

**Tìm:** trong mỗi guard shell-script string, đoạn trích path từ patch, hiện lấy **path đầu tiên** (kiểu `... | grep '*** ' | head -n1 | sed ...` `[needs Worker verify]` cú pháp chính xác).

**Thay bằng / Thêm:** trích **MỌI** path:
- Grep tất cả dòng `^\*\*\* (Add|Update|Delete|Move) File: ` → strip prefix → danh sách paths (Move: thêm dest `*** Move to:` nếu có `[needs Worker verify]` format Move của Codex 0.145.0).
- Loop qua TỪNG path, áp đúng logic check cũ của guard đó.
- **Quyết định BLOCK nếu BẤT KỲ path vi phạm.** Exemption/ALLOW chỉ khi **MỌI** path đều thuộc allow-set. (Đảo ngược semantic hiện tại "first-path-exempt → allow-all".)
- Empty patch / không path nào → giữ hành vi mặc định hiện tại (`[needs Worker verify]` default; nếu default là ALLOW thì giữ, không siết ngoài scope).

**Lưu ý:** đây là gốc security hole. Cẩn thận regex: `*` phải escape trong grep; path có space → không split path theo whitespace (chỉ split theo dòng `*** ... File:`). Áp cho MỌI guard có allow-list (anchor #6); guard pure-deny (không exemption) cũng phải check mọi path (kẻo path độc thứ 2 lọt). KHÔNG đổi tiêu chí check của từng path — chỉ đổi "1 path" → "mọi path" + "block-if-any".

### Task 2: approval-gate — self-bootstrap exemption + install skeleton-state (#5)

**File:** `crates/sos-adapter-codex/src/templates.rs` — approval-gate content-fn (`:620` `[needs Worker verify]`) + install artifact emit (skeleton state-file).

**Tìm:** nhánh approval-gate khi `.sos-state/ticket-state.env` missing → BLOCK mọi non-ticket patch.

**Thay bằng / Thêm (hai lớp, an toàn):**
1. **self-bootstrap exemption:** nếu state-file missing VÀ patch (đã qua Task-1 all-path parse) touch **CHỈ** `.sos-state/ticket-state.env` (không path nào khác) → ALLOW (cho khởi tạo state). Nếu patch touch ticket-state.env **kèm** path khác → BLOCK (Task-1 all-path đã lo; giữ nguyên).
2. **install skeleton:** render `.sos-state/ticket-state.env` skeleton (empty/no-approval defaults) lúc install artifact, **non-clobber** (chỉ tạo nếu absent — không đè state thật). Nhánh missing-file gần như không chạm ở fresh install.

**Lưu ý:** exemption self-bootstrap **CHỈ an toàn vì Task 1 (#6) all-path check land cùng phiếu** — nếu không, kẻ tấn công đặt ticket-state.env + approved_version giả + path độc trong 1 patch sẽ lọt. Ghi rõ coupling này vào Discovery. KHÔNG cho exemption với path nào khác ngoài đúng `.sos-state/ticket-state.env`. Skeleton non-clobber: nếu ticket-state.env đã tồn tại → KHÔNG ghi đè (bảo toàn approval thật). `[needs Worker verify]` install artifact emit nằm ở đâu (cùng templates.rs hay install-engine — nếu install-engine thì skeleton emit là D2b-adjacent, ghi Discovery + confirm với orchestrator; ưu tiên emit trong codex adapter render nếu có surface).

### Task 3: AGENTS.md — thêm spawn caveat (#7)

**File:** `crates/sos-adapter-codex/src/templates.rs` — AGENTS.md content-fn (`[needs Worker verify]` vị trí orchestrator/spawn guidance).

**Tìm:** phần AGENTS.md nói về spawn subagent / orchestrator delegation.

**Thay bằng / Thêm:** một dòng caveat: khi spawn custom-agent, **omit `agent_type`** HOẶC spawn **không** full-history fork — vì full-history forked agents kế thừa agent type của parent (first-spawn fail observed P079). Diễn đạt ngắn, orchestrator-facing.

**Lưu ý:** chỉ doc-string trong render, không đụng hook/guard. Giữ dưới 32KiB AGENTS.md limit (discovery §1). Wording khớp lỗi thật P079 #7.

### Task 4: tests — real-payload guard fixtures + bootstrap + negative-test

**File:** `crates/sos-adapter-codex/src/templates.rs` test module (`[needs Worker verify]` vị trí).

**Thêm test** (render guard/approval → chạy REAL script với fixture → assert exit code):

1. **#6 no-regress:** REAL single-path apply_patch (Sếp b3), path ticket-allowed → guard ALLOW.
2. **#6 bypass BLOCK (core):** multi-path fixture — ticket-allowed path TRƯỚC + `.env` (hoặc `src/**`, hoặc `.sos-state/ticket-state.env`) SAU → guard **BLOCK**. Lặp cho mỗi guard có allow-list (anchor #6).
3. **#5 bootstrap ALLOW:** state-file missing + patch tạo CHỈ `.sos-state/ticket-state.env` → approval-gate ALLOW.
4. **#5 bootstrap+độc BLOCK:** state-file missing + patch tạo ticket-state.env kèm path độc → BLOCK (coupling #6).
5. **Negative-test (răng, bắt buộc — ghi Discovery):** revert Task-1 (về head-n1) → test #2 chuyển BLOCK→ALLOW = FAIL; revert Task-2 exemption → test #3 chuyển ALLOW→BLOCK = FAIL.

**Lưu ý:** nếu anchor #7 sai (guard không chạy standalone được) → ESCALATE trước khi đổi oracle (xem Task 0 note). Fixture apply_patch phải là REAL b3 shape, không phải string bịa. Ground-truth cuối cho #4-class behavioral = live Codex P079 round-2 (d2b) — phiếu này KHÔNG test #4.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-adapter-codex/src/templates.rs` | Task 1: mọi guard parse all-path + block-if-any (#6); Task 2: approval-gate self-bootstrap exemption + install skeleton-state non-clobber (#5); Task 3: AGENTS.md spawn caveat (#7); Task 4: real-payload guard fixtures + bootstrap + negative-test |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-adapter-codex/src/templates.rs` — SubagentStart/Stop marker (`:302`) | KHÔNG đụng — #4 = **d2b** (cần SubagentStart probe). Render marker hook KHÔNG đổi |
| `crates/sos-adapter-codex/src/templates.rs` — 3 startup render-fn (config.toml/rules/hooks.json) | d1 SHIPPED — KHÔNG regress output |
| `crates/sos-install/**`, `crates/sos-core/**`, `crates/sos-adapter-claude/**` | Untouched (nếu skeleton-state emit hoá ra thuộc install-engine → confirm orchestrator, ghi Discovery) |

---

## Luật chơi (Constraints)

1. **#6 block-all-violating là bất biến security:** guard BLOCK nếu BẤT KỲ path vi phạm; ALLOW chỉ khi MỌI path exempt. KHÔNG giữ bất kỳ "first-path-exempt → allow-all" nào.
2. **#5 exemption tối thiểu + coupled:** self-bootstrap chỉ cho `.sos-state/ticket-state.env` đơn độc; an toàn CHỈ vì #6 cùng land. Skeleton install non-clobber (không đè state thật). KHÔNG mở exemption cho path khác.
3. **KHÔNG đụng #4 marker lifecycle** (d2b) — nếu thấy muốn sửa marker → DỪNG, escalate.
4. **Oracle real-payload:** #6/#5 test chạy REAL guard script với REAL apply_patch fixture (b3 shape) + assert exit code — KHÔNG string-contains thô trên guard content (trừ khi anchor #7 chứng minh guard không mock được → ESCALATE trước, không tự hạ oracle). Negative-test bắt buộc cả #6 và #5.
5. **Additive, dep-direction giữ** — adapter→core, không tạo core→adapter import. Không đổi signature public.
6. **Dựa trên P079 errors thật + Sếp b3 apply_patch sample**, KHÔNG đoán format. Bug không khớp spec → DISCOVERY + báo, không "sửa mò".

---

## Nghiệm thu

### Automated
- [ ] `cargo check` clean
- [ ] `cargo test -p sos-adapter-codex` pass (gồm test mới #6 no-regress/bypass, #5 bootstrap ALLOW/BLOCK)
- [ ] Oracle: multi-path bypass fixture (ticket-first + `.env`/src/state-second) → guard **BLOCK** cho MỌI allow-list guard; single-path b3 ticket → **ALLOW**; bootstrap-only ticket-state.env → ALLOW; bootstrap+độc → BLOCK
- [ ] **Negative-test:** revert #6 → bypass fixture ALLOW (FAIL); revert #5 → bootstrap BLOCK (FAIL). Ghi Discovery
- [ ] Flake gate: `cargo test -p sos-adapter-codex` ×20 → 0-flaky
- [ ] Dep-direction guard xanh (adapter→core)

### Manual Testing
- [ ] (Nếu có Codex 0.145.0 tay) fresh install → skeleton `.sos-state/ticket-state.env` tồn tại; approval-gate không deadlock khi khởi tạo phiếu đầu
- [ ] AGENTS.md render chứa spawn caveat, dưới 32KiB

### Regression
- [ ] 3 startup-file (config.toml/rules/hooks.json — d1) render output KHÔNG đổi
- [ ] SubagentStart/Stop marker render (#4 = d2b) KHÔNG đổi
- [ ] Guard hành vi hợp lệ cũ (single valid ticket patch) vẫn ALLOW — không over-block

### Docs Gate
- [ ] `CHANGELOG.md` — entry P078d2a (multi-path guard bypass fix + approval bootstrap + spawn caveat)
- [ ] **`SECURITY.md`** — ghi lỗ multi-path `head -n1` bypass ĐÃ ĐÓNG (guard nay check mọi apply_patch path, block-if-any); note approval self-bootstrap exemption coupled-with-all-path-check
- [ ] Codex adapter format/enforcement note — thêm vào `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md` (hoặc `docs/adapters/codex/MAPPING.md`/`CAPABILITY.md` **nếu tồn tại** `[needs Worker verify]`; nếu chưa có surface → ghi "N/A, note trong Discovery"): multi-path fix + bootstrap mechanism
- [ ] `docs/discoveries/P078d2a.md`

### Discovery Report
- [ ] Write to `docs/discoveries/P078d2a.md`
  - Anchor #1–7 — CORRECT / WRONG (file:line thật cho guard-sites + approval + AGENTS; guard nào có allow-list; guard chạy standalone được không)
  - **Coupling note (bắt buộc):** #5 self-bootstrap exemption an toàn CHỈ vì #6 all-path land cùng phiếu
  - Negative-test kết quả (revert #6 → bypass ALLOW; revert #5 → bootstrap BLOCK)
  - Skeleton-state emit ở đâu (codex render vs install-engine) — nếu install-engine, note scope-boundary
  - **#4 handoff to d2b:** SubagentStart matcher semantics vẫn open (cần probe); ghi rõ #4 KHÔNG fix trong phiếu này
  - Docs updated (SECURITY.md + adapter note; hoặc "N/A" explicit)
  - Tier escalations (None expected — nếu chạm marker/engine → escalate)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
