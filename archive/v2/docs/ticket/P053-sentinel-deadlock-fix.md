# PHIẾU P053: Sentinel-vs-silent merge deadlock fix

> **ID format:** `P053` (assigned per BACKLOG harvest batch P049–P054).
> **Filename:** `docs/ticket/P053-sentinel-deadlock-fix.md`
> **Branch:** `fix/P053-sentinel-deadlock-fix`

---

> **Loại:** Bugfix (gate-interaction / security-surface contract)
> **Ưu tiên:** P1
> **Tầng:** 1 (security-surface contract: reconciles two P042-era security gates + touches Giám sát silent-when-clean contract → AUTO Tầng 1 per CLAUDE.md "security boundary touch → AUTO Tầng 1", dù diff nhỏ)
> **Ảnh hưởng:** `.claude/commands/security-review.md` (primary), `agents/boundary-check.md` (contract-doc clarify), `README.md` (1-line contract-doc clarify), `scripts/block-unsafe-merge.sh` (verify-only, KHÔNG sửa)
> **Dependency:** None (P042 already shipped; this reconciles its two gates)

---

## Context

### Vấn đề hiện tại

Hai gate sinh ra ở P042 (cùng wave, 2026-05-25) đang **deadlock** nhau trên một security-surface PR có review SẠCH (live bug, ket dogfood surface 2026-06-03):

1. **`scripts/block-unsafe-merge.sh`** (PreToolUse hook, Giám sát backstop) — khi PR touch security surface, hook ĐÒI một comment chứa sentinel block `<!-- security-review-start -->` + dòng `Verdict: APPROVE` trên PR trước khi cho `gh pr merge <N>`. `[verified]` — block-unsafe-merge.sh:103-109: grep `<!-- security-review-start -->` rồi `VERDICT_LINE=... grep -E '^Verdict:'` rồi `grep -q 'APPROVE'`.

2. **`/security-review` → `boundary-check` (Giám sát)** — **silent-when-clean** theo thiết kế P042 chống approve-fatigue: review SẠCH (APPROVE + 0 FLAG) → KHÔNG post comment. `[verified]` — silent-when-clean rule sống ở `.claude/commands/security-review.md` Step 3 (line 62: "APPROVE AND 0 FLAG → Do NOT post comment") VÀ `agents/boundary-check.md` line 194 (Giám sát luôn return block, caller quyết post-or-skip).

**Deadlock:** security-surface PR + review SẠCH → `/security-review` KHÔNG post comment → `block-unsafe-merge.sh` không bao giờ thấy `Verdict: APPROVE` → merge bị chặn vĩnh viễn. Lối thoát DUY NHẤT hiện tại = override marker `[security-review-skip:<reason>]` (block-unsafe-merge.sh:56) — đúng pattern `--no-verify`-death mà workflow tránh: gate bắt buộc phải bị bypass thủ công ở case HỢP LỆ (review chạy + sạch) = gate tự huỷ uy tín.

**Root đã chẩn:** hai gate khai báo lệch contract — một bên ĐÒI sentinel, một bên CỐ TÌNH im khi sạch. Đây chính là bài học retro "single-source-the-truth / sentinel mismatch" thu nhỏ: input của gate A (sentinel comment) bị gate B chủ động triệt tiêu ở case sạch.

### Giải pháp — Option A (ket precedent + safer)

**Chọn Option A:** `/security-review`, **CHỈ trong PR mode** (mode duy nhất mà `block-unsafe-merge` cai quản), LUÔN post sentinel comment — kể cả clean `Verdict: APPROVE`. Branch/range mode GIỮ NGUYÊN silent-when-clean (advisory, không có PR để gate).

> **V2 confirm — scoping explicit (đáp Turn 1 "Confirm scoping"):** auto-post clean-APPROVE **CHỈ nổ ở PR mode** (mode duy nhất `block-unsafe-merge` cai quản). ADVISORY / branch / range run **GIỮ NGUYÊN silent-when-clean** (P042 anti-approve-fatigue còn nguyên). Mode-distinction detect ở đâu: **anchor #6** — `block-unsafe-merge.sh:47` chỉ match `gh pr merge[[:space:]]+[0-9]+` (PR mode duy nhất có gate), và `.claude/commands/security-review.md` Step 0 phân rõ PR / branch / range mode trước khi tới Step 3/4. Worker Turn 1 confirm anchors #1–#7 ✅ → mode-distinction cleanly detectable, fix scope chặt.

**Tại sao A, không B:**

| | Option A (chọn) | Option B (loại) |
|---|---|---|
| Cơ chế | `/security-review` PR mode luôn post APPROVE sentinel | `block-unsafe-merge` chấp nhận "clean-review-ran" signal thay vì đòi comment |
| Tín hiệu | **Positive explicit** — comment APPROVE thật tồn tại trên PR | **Inferred absence** — hook đoán review đã chạy-và-sạch |
| An toàn | Cao — không có comment = chắc chắn chưa review (block đúng) | Yếu — "không có FLAG" lẫn với "chưa review bao giờ" → false-clean → merge lọt |
| Agent-agnostic | Comment là PR comment thật (Codex/opencode đọc được) | Signal phụ thuộc cơ chế Claude-specific để biết review đã chạy |
| Precedent | Đã PROVEN ở ket WORKFLOW §21 ("PR-gated → ALWAYS post sentinel, even clean APPROVE") | Chưa ai chạy |
| One-disease-one-mechanism | Sửa MỘT chỗ (slash command emit logic) | Sửa hook accept logic + vẫn phải có cách hook biết review ran |

Option A thắng cả 4 trục an-toàn/proven/agent-agnostic/đơn-cơ-chế. Lean của brief xác nhận; assess honest → giữ A.

**Sentinel marker tái dùng (NGUYÊN VĂN, không sáng chế chuỗi mới):** `[verified]`
- Mở: `<!-- security-review-start -->`
- Đóng: `<!-- security-review-end -->`
- Dòng verdict block-unsafe-merge grep: `Verdict: APPROVE` (block-unsafe-merge.sh:105 grep `^Verdict:`, line 106 grep `APPROVE`).

Giám sát ĐÃ emit đúng block này cho mọi verdict (boundary-check.md line 174-189 + line 194 "Em vẫn return sentinel block in final report luôn"). Nên Option A KHÔNG sửa Giám sát emit — chỉ sửa caller (slash command) để KHÔNG nuốt comment ở PR mode.

### Known limitation — stale-sentinel hole (V2, đáp [O1.1], KHÔNG mở scope)

> **Đây là known-limitation đã ghi nhận có chủ đích — KHÔNG nằm trong scope P053.** Tracked riêng = **[P055]** (Open backlog).

`scripts/block-unsafe-merge.sh:102` fetch TẤT CẢ comment body (`gh pr view "$PR" --json comments --jq '.comments[].body'`), rồi `:103-106` grep sentinel `<!-- security-review-start -->` đầu tiên + dòng `Verdict: APPROVE` — **KHÔNG có SHA/timestamp scoping** (oracle: `grep -n 'sha\|SHA\|commit\|head_sha\|created_at\|updated_at' scripts/block-unsafe-merge.sh` → 0 hit; SOUND). `[verified]` qua Worker Turn 1 oracle.

**Hệ quả:** một khi Option A bắt đầu auto-post clean APPROVE sentinel, một APPROVE cũ (review commit A) có thể green-light commit B+C chưa review trên CÙNG multi-commit PR. Lỗ này **đã tồn tại trước P053** (gate vốn grep bất kỳ APPROVE lịch sử nào), nhưng Option A làm nó dễ phát sinh hơn (clean review giờ cũng đẻ sentinel).

**Tại sao KHÔNG fix trong P053:** SHA-scoping = **bệnh khác** (sửa accept-side `block-unsafe-merge.sh`, đòi jq filter mới bind head-SHA + slash command post thêm `Head SHA:` line). Gộp vào đây = vi phạm one-disease-one-mechanism (P053 chỉ sửa emit-side). Mirror đúng cách `block-unsafe-merge.sh:15-16` đã document một known-bypass thay vì âm thầm.

**Mitigations hiện có (ghi để honest, không phải fix):**
- Chủ nhà đọc comment APPROVE có timestamp trước khi merge (comment hiển thị thời điểm review).
- Squash-merge collapse history → giảm cửa sổ multi-commit-chưa-review.
- SHA-scoping tracked separately = **[P055]** (xem "Discovery debt to file" + BACKLOG Open backlog).

### Scope

- **SỬA:** `.claude/commands/security-review.md` — Step 3 + Step 4 logic: scope silent-when-clean về branch/range mode; PR mode luôn post (kể cả clean APPROVE).
- **CLARIFY (doc, không đổi hành vi):** `agents/boundary-check.md` line 194 silent-when-clean — thêm 1 câu rằng caller áp dụng silent CHỈ cho non-PR-gated runs. (Giám sát hành vi không đổi — vẫn luôn return block.)
- **CLARIFY (doc, V2 — đáp [O1.1-minor]):** `README.md` line ~74 mô tả boundary-check "posts ADVISORY comment (silent when clean — KHÔNG block merge)" — cập nhật 1 dòng: PR mode LUÔN post sentinel (cả clean APPROVE); silent-when-clean giờ chỉ áp ADVISORY/branch/range mode.
- **CLARIFY (doc, V2):** thêm comment near gate grep (HOẶC trong boundary-check/security-review doc) note stale-sentinel limitation (xem Task 4).
- **KHÔNG SỬA:** `scripts/block-unsafe-merge.sh` accept logic — đã đúng cho scope này (one-disease-one-mechanism: KHÔNG patch cả 2 gate; sửa emit, không sửa accept). SHA-scoping = [P055].

---

## Task 0 — Verification Anchors

> Architect dùng Read/Glob; runtime grep behavior do Worker verify.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `block-unsafe-merge.sh` đòi sentinel `<!-- security-review-start -->` + `Verdict: APPROVE` để allow merge security-surface PR | `grep -n 'security-review-start\|^Verdict:\|APPROVE' scripts/block-unsafe-merge.sh` | ✅ `[verified]` lines 103-109 |
| 2 | Sentinel marker chuỗi NGUYÊN VĂN = `<!-- security-review-start -->` / `<!-- security-review-end -->` | `grep -n 'security-review-start\|security-review-end' agents/boundary-check.md scripts/block-unsafe-merge.sh` | ✅ `[verified]` boundary-check.md:174,189 + block-unsafe-merge.sh:103 |
| 3 | Verdict line format Giám sát emit = `Verdict: APPROVE` (khớp grep `^Verdict:` + `APPROVE` của hook) | `grep -n 'Verdict:' agents/boundary-check.md` | ✅ `[verified]` boundary-check.md:188 "Verdict: APPROVE \| NEEDS_REVIEW" |
| 4 | Silent-when-clean suppress comment sống ở slash command Step 3 (caller), KHÔNG ở Giám sát | đọc `.claude/commands/security-review.md` Step 3 + `agents/boundary-check.md` line 194 | ✅ `[verified]` slash Step 3 line 62 "Do NOT post comment"; boundary-check.md:194 "caller's slash command applies silent-when-clean rule before gh pr comment" |
| 5 | Giám sát LUÔN return sentinel block kể cả APPROVE (nên Option A không cần sửa Giám sát emit) | đọc `agents/boundary-check.md` line 194 + 244 | ✅ `[verified]` line 194 "Em vẫn return sentinel block in final report luôn" |
| 6 | PR mode = mode duy nhất `block-unsafe-merge` cai quản (branch/range không có PR để gate) → scoping explicit khả thi | đọc `.claude/commands/security-review.md` Step 0 + `block-unsafe-merge.sh` match `gh pr merge <N>` | ✅ `[verified]` block-unsafe-merge.sh:47 chỉ match `gh pr merge[[:space:]]+[0-9]+`; slash Step 0 phân PR/branch/range mode |
| 7 | Step 4 PR mode đã có `gh pr comment <N> --body` path | đọc `.claude/commands/security-review.md` Step 4 | ✅ `[verified]` line 68 `gh pr comment <N> --body "<sentinel-block-content>"` |
| 8 | Worker cần verify: khi APPROVE clean, comment posted có đúng grep được bởi hook end-to-end (post → block-unsafe-merge re-read) | `[needs Worker verify]` — runtime: tạo PR security-surface clean, chạy `/security-review`, rồi `gh pr merge` → hook allow | ⏳ TO VERIFY |
| 9 | (V2, [O1.1-minor]) `README.md` ~line 74 mô tả boundary-check "ADVISORY comment (silent when clean — KHÔNG block merge)" — đúng vị trí cần clarify 1 dòng | `grep -n 'boundary-check\|silent.*clean\|ADVISORY' README.md` | `[needs Worker verify]` — line số có thể trôi; Worker grep, sửa dòng mô tả contract, không sửa thêm |
| 10 | (V2, [O1.1]) `block-unsafe-merge.sh` KHÔNG SHA-scope APPROVE comment (stale-sentinel hole có thật, là known-limitation không phải phiếu-assumption) | `grep -n 'sha\|SHA\|commit\|head_sha\|created_at\|updated_at' scripts/block-unsafe-merge.sh` → 0 hit | ✅ `[verified]` Worker Turn 1 oracle (SOUND, 0 hit) |

**Không có ❌.** Code-claim `[verified]` từ Read + Worker Turn 1 oracle; anchor #8 (end-to-end runtime allow) + #9 (README line drift) là `[needs Worker verify]`.

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V2 (Architect responded to Turn 1)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**
- Anchor #1: ✅ block-unsafe-merge.sh:103-109 — sentinel grep confirmed exact strings.
- Anchor #2: ✅ `<!-- security-review-start -->` / `<!-- security-review-end -->` verified in both files (boundary-check.md:174,189; block-unsafe-merge.sh:103).
- Anchor #3: ✅ boundary-check.md:188 emits `Verdict: APPROVE | NEEDS_REVIEW`; hook grep-E `^Verdict:` at :105 + grep `APPROVE` at :106 — strings match.
- Anchor #4: ✅ security-review.md:62 "Do NOT post comment" (silent-when-clean); boundary-check.md:194 "caller's slash command applies silent-when-clean rule before `gh pr comment`".
- Anchor #5: ✅ boundary-check.md:194 "Em vẫn return sentinel block in final report luôn".
- Anchor #6: ✅ block-unsafe-merge.sh:47 matches only `gh pr merge[[:space:]]+[0-9]+`; security-review.md Step 0 enumerates PR/branch/range modes.
- Anchor #7: ✅ security-review.md:68 `gh pr comment <N> --body "<sentinel-block-content>"`.
- Anchor #8: ⏳ `[needs Worker verify]` — cannot confirm end-to-end runtime allow statically.

**Objections:**

- [O1.1] **CRITICAL — Stale-sentinel hole (no SHA scoping in block-unsafe-merge.sh).**
  Claim: does the hook re-check the sentinel against the diff that was *actually reviewed*, or does it accept any historical APPROVE comment regardless of subsequent commits?
  Oracle: `grep -n 'sha\|SHA\|commit\|latest\|head_sha\|updated_at\|created_at' scripts/block-unsafe-merge.sh` → 0 hits on any SHA/timestamp scoping.
  Soundness: SOUND (compiler-equivalent: grep of an exact pattern in a bash file that would have to be there if scoping existed).
  Verdict: **self-closed via oracle — hole confirmed, not phiếu assumption.**

  **Concretely:** block-unsafe-merge.sh:102 does `gh pr view "$PR" --json comments --jq '.comments[].body'` — this fetches ALL comment bodies concatenated, then at :103-106 greps for the first `<!-- security-review-start -->` sentinel + first `Verdict:` line within 50 lines of it. There is zero SHA/timestamp scoping. Result: if Commit A is reviewed (APPROVE comment posted), then Commits B+C are pushed to the same PR (touching security surface), the stale APPROVE comment still satisfies the hook. `gh pr merge <N>` will be ALLOWED even though B+C were never reviewed.

  **This is a Tầng 1 concern because:** it's a security contract question — Option A as written potentially creates a weaker gate than the deadlock it fixes. The deadlock is bad, but the fix must not introduce a false-security hole. The phiếu does not acknowledge or mitigate this gap.

  **Proposed alternatives:**
  - A. (Worker lean) — Document the stale-sentinel limitation explicitly in Nghiệm thu + boundary-check.md as a known constraint (same pattern as the "branch-only form bypass" known limitation at block-unsafe-merge.sh:15-16). Rationale: SHA-scoping requires gh API to return comment `created_at` + PR `head_sha` at time of comment, which needs a more involved jq filter and a new phiếu. Acknowledging the gap honestly is better than silently shipping it as "fixed." The workflow's defense-in-depth (Sếp reads the APPROVE comment; squash-merge timestamps are visible) partially mitigates. P053 scope = fix deadlock; SHA-scoping = separate P054/P055.
  - B. Add minimal SHA-scoping to block-unsafe-merge.sh in this phiếu: capture `gh pr view <N> --json headRefOid --jq .headRefOid` and require the APPROVE comment body contain a `Head SHA: <sha>` line that the slash command posts alongside the sentinel. Requires coordinated change to both files (expanding scope beyond current phiếu "KHÔNG SỬA block-unsafe-merge.sh" constraint).

  **[O1.1-minor]** `README.md:74` currently says boundary-check "posts an ADVISORY comment to the PR (silent when clean — KHÔNG block merge)" — describes the contract Option A changes. Needs a 1-line doc update.

**Status:** ✅ RESPONDED (Architect Turn 2 below)

### Turn 2 — Architect Response (phiếu V2)

- **[O1.1] → ACCEPT (as documented known-limitation, NO scope expansion).**
  Worker oracle is SOUND — `block-unsafe-merge.sh:102-106` fetches all comment bodies and greps any historical `Verdict: APPROVE` with zero SHA/timestamp scoping (anchor #10 ✅ verified, 0 hit). Option A's auto-post of clean APPROVE makes a pre-existing stale-sentinel hole easier to hit on multi-commit PRs. **Resolution = Worker lean (alt A) + one-disease doctrine:**
  - P053 stays scoped to the deadlock fix, emit-side only. `scripts/block-unsafe-merge.sh` accept logic stays UNTOUCHED — adding SHA-scoping is a *different disease* (would patch accept-side + need a new jq filter binding head-SHA + a `Head SHA:` post line), violating one-disease-one-mechanism. Rejected alt B for that reason.
  - Folded the limitation into Context "Known limitation — stale-sentinel hole" section (above), into **Task 4** (ship a comment near the gate grep / in security-review doc, mirroring the documented bypass at block-unsafe-merge.sh:15-16), into Constraints #6, and into Nghiệm thu Regression.
  - Follow-up filed as **[P055]** in `docs/BACKLOG.md` Open backlog (Discovery→backlog flow, per BACKLOG maintenance rule #4). I (Architect) appended it directly — handbook RESPOND flow + BACKLOG rule #4 both sanction appending a surfaced-debt item; the phiếu's "Discovery debt to file" section also records it as backstop.
  - **Action taken:** Context section added; Task 4 added; anchor #10 added; Constraint #6 added; Regression check added; Discovery debt section added; BACKLOG [P055] appended.

- **[O1.1-minor] → ACCEPT.**
  `README.md` line ~74 describes the silent-when-clean contract Option A narrows. **Action taken:** added **Task 5** (1-line README update — PR mode ALWAYS posts sentinel incl. clean APPROVE; silent-when-clean now applies to ADVISORY/branch/range mode only), added anchor #9 (`[needs Worker verify]` — line number may drift, Worker greps), updated header Ảnh hưởng + Scope + Files cần sửa + Docs Gate to include README.md.

- **Scoping confirm (Turn 1 "Confirm scoping") → DEFEND + restate.**
  Clean-APPROVE auto-post fires **only in PR mode** (block-unsafe-merge-governed); ADVISORY/branch/range runs stay silent-when-clean. Detectable via **anchor #6** (`block-unsafe-merge.sh:47` matches only `gh pr merge[[:space:]]+[0-9]+`; slash Step 0 partitions PR/branch/range before Step 3/4). Worker confirmed anchors #1–#7 ✅ → mode distinction is cleanly detectable. **Action taken:** added explicit V2 confirm callout in Giải pháp section; reinforced in Task 1/Task 2 Lưu ý + Constraint #3.

**Status:** ✅ RESPONDED — phiếu bumped to V2. No DEFER TO CHỦ NHÀ. No blocking APPROVAL_GATE design fork — Option A stands, stale-sentinel documented + filed as [P055]. Next: Worker (CHALLENGE) re-verify consensus or proceed to approval gate.

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1: Scope silent-when-clean về non-PR-gated runs ở Step 3

**File:** `.claude/commands/security-review.md`

**Tìm:** (Step 3 — Extract sentinel block from subagent output, đoạn quyết post-or-skip)

```
- If verdict line inside block = `APPROVE` AND 0 FLAG → **silent-when-clean rule fires.** Do NOT post comment. Tell user: "Security review complete. APPROVE (0 flags). No comment posted."
- If verdict = `NEEDS_REVIEW` OR ≥1 FLAG → continue to Step 4.
```

**Thay bằng:**

```
- **PR mode (block-unsafe-merge-governed) — ALWAYS post, including clean APPROVE.** Vì `scripts/block-unsafe-merge.sh` ĐÒI sentinel `Verdict: APPROVE` comment trên PR để cho merge security-surface PR. Nếu silent-when-clean nuốt comment ở PR mode → merge deadlock (chỉ thoát bằng override marker = `--no-verify`-death). Trong PR mode, BỎ QUA silent-when-clean → luôn continue to Step 4 và post block (cả APPROVE lẫn NEEDS_REVIEW). Đây là Option A (ket WORKFLOW §21 precedent), scoped chặt CHỈ PR mode.
- **Branch / range mode (advisory, no PR to gate) — silent-when-clean GIỮ NGUYÊN.** Nếu verdict = `APPROVE` AND 0 FLAG → Do NOT post/write. Tell user: "Security review complete. APPROVE (0 flags). No comment posted." Nếu `NEEDS_REVIEW` OR ≥1 FLAG → continue to Step 4 (write to local file per Step 4 branch/range path).
```

**Lưu ý:** Đây là HÀNH VI cốt lõi của fix. Phân nhánh theo mode (PR vs branch/range) đã có sẵn ở Step 0 + Step 4 — Worker chỉ cần đảm bảo Step 3 quyết post-or-skip THEO mode đã resolve ở Step 0 (anchor #6), không phải theo verdict đơn thuần. KHÔNG đổi cách Giám sát emit (Giám sát luôn return block — anchor #5). **Scoping (V2):** clean-APPROVE auto-post CHỈ ở PR mode; ADVISORY/branch/range im như cũ.

### Task 2: Step 4 PR-mode post path cover clean APPROVE

**File:** `.claude/commands/security-review.md`

**Tìm:** (Step 4 — Post advisory comment, đầu mục PR mode)

```
**PR mode (preferred):**
- `gh pr comment <N> --body "<sentinel-block-content>"` — post the full sentinel-wrapped block as a PR comment.
- Verify post: `gh pr view <N> --json comments` should show the new comment.
```

**Thay bằng / Thêm:**

```
**PR mode (preferred) — post for BOTH clean APPROVE and NEEDS_REVIEW:**
- `gh pr comment <N> --body "<sentinel-block-content>"` — post the full sentinel-wrapped block (chứa `<!-- security-review-start -->` … `Verdict: APPROVE|NEEDS_REVIEW` … `<!-- security-review-end -->`) as a PR comment.
- **Lý do post cả clean APPROVE:** `scripts/block-unsafe-merge.sh:103-109` grep comment cho `<!-- security-review-start -->` + `^Verdict:` chứa `APPROVE` để cho merge. Không có comment APPROVE = hook chặn merge (deadlock). PR mode KHÔNG áp silent-when-clean (xem Step 3).
- Verify post: `gh pr view <N> --json comments` should show the new comment with the sentinel block.
```

**Lưu ý:** Sentinel chuỗi tái dùng nguyên văn — KHÔNG sáng chế marker mới (anchor #2, #3). Branch/range mode silent-when-clean ở Step 4 (write to `docs/security/last-review.md`) giữ nguyên — chỉ post khi NEEDS_REVIEW/FLAG. PR-comment-fail fallback (line 75-77) giữ nguyên.

### Task 3: Clarify silent-when-clean scope trong Giám sát handbook (doc-only, hành vi KHÔNG đổi)

**File:** `agents/boundary-check.md`

**Tìm:** (line 194, Silent-when-clean rule)

```
**Silent-when-clean rule (generic anti-approve-fatigue principle):** Verdict `APPROVE` + 0 FLAG → exit silently, KHÔNG post comment. Verdict `NEEDS_REVIEW` HOẶC ≥1 FLAG → emit sentinel block như spec. Em vẫn return sentinel block in final report luôn (caller decides post-or-skip based on verdict); but caller's slash command applies silent-when-clean rule before `gh pr comment`.
```

**Thay bằng:**

```
**Silent-when-clean rule (generic anti-approve-fatigue principle):** Verdict `APPROVE` + 0 FLAG → caller MAY exit silently (KHÔNG post comment) — NHƯNG chỉ cho **advisory / non-PR-gated runs** (branch/range mode). **Cho PR mode mà `scripts/block-unsafe-merge.sh` cai quản, caller LUÔN post sentinel comment kể cả clean APPROVE** (P053): hook đòi `Verdict: APPROVE` comment để cho merge; silent ở đây = merge deadlock. Em (Giám sát) hành vi KHÔNG đổi — em **vẫn luôn return sentinel block in final report** cho mọi verdict; quyết post-or-skip là của caller's slash command (PR mode → luôn post; branch/range mode → silent-when-clean).
```

**Lưu ý:** Đây là CLARIFY contract, KHÔNG đổi hành vi Giám sát (Giám sát luôn return block — đã đúng). Tránh đụng anti-pattern list (line 254 "KHÔNG auto-bóp NEEDS_REVIEW về APPROVE") — không liên quan, giữ nguyên. P042 implementation status section (line 270+) không đổi (verdict logic không đổi, chỉ caller post-decision đổi).

### Task 4: Document stale-sentinel limitation (V2, đáp [O1.1] — known-limitation, KHÔNG fix accept logic)

**File:** `.claude/commands/security-review.md` (preferred — đặt near Step 4 PR-mode post path, nơi sentinel APPROVE được tạo ra). HOẶC `agents/boundary-check.md` near line 194 nếu Worker thấy hợp ngữ cảnh hơn. `[needs Worker verify]` — Worker chọn 1 trong 2 chỗ, đặt note ở chỗ reviewer tương lai chắc chắn đọc khi đụng sentinel.

**Thêm (block note, NGUYÊN VĂN gợi ý — Worker có thể chỉnh wording cho khớp doc voice):**

```
> **Known limitation — APPROVE sentinel is NOT SHA-scoped.** `scripts/block-unsafe-merge.sh:102-106` greps for ANY historical `Verdict: APPROVE` sentinel comment on the PR, with no binding to the reviewed commit's head SHA. Hệ quả: trên một multi-commit PR, một clean APPROVE trên commit A có thể satisfy gate cho commit B+C chưa review. **Mitigations:** (1) Chủ nhà đọc comment APPROVE có timestamp trước khi merge; (2) squash-merge collapse history. SHA-scoping tracked separately = **[P055]** (docs/BACKLOG.md Open backlog). Mirror pattern: documented bypass at block-unsafe-merge.sh:15-16.
```

**Lưu ý:** Đây là DOC note, KHÔNG sửa logic `block-unsafe-merge.sh`. One-disease-one-mechanism: P053 chỉ sửa emit-side; SHA-scoping = bệnh khác = [P055]. KHÔNG thêm `Head SHA:` line vào sentinel ở phiếu này (đó là Option B đã loại).

### Task 5: Clarify README contract line (V2, đáp [O1.1-minor], doc-only)

**File:** `README.md`

**Tìm:** `[needs Worker verify]` — Worker grep `grep -n 'boundary-check\|silent.*clean\|ADVISORY' README.md` (V1 anchor cited line 74; line số có thể trôi). Dòng mô tả boundary-check kiểu "posts an ADVISORY comment to the PR (silent when clean — KHÔNG block merge)".

**Thay bằng (1-line clarify, giữ tight):**

```
... posts a sentinel comment to the PR. In **PR mode** (block-unsafe-merge-governed) the sentinel is ALWAYS posted incl. clean APPROVE (P053 — needed for the merge gate); silent-when-clean now applies to ADVISORY / branch / range mode only.
```

**Lưu ý:** CHỈ sửa dòng mô tả contract, KHÔNG đụng phần khác của README. Nếu README chỉ liệt kê tên agent không mô tả contract ở dòng đó → Worker tìm đúng dòng mô tả silent-when-clean contract; nếu thật sự không có mô tả contract ở đâu → ghi Discovery "N/A — README không mô tả silent-when-clean contract" và skip (anchor #9).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `.claude/commands/security-review.md` | Task 1: Step 3 post-or-skip theo mode (PR mode luôn post); Task 2: Step 4 PR-mode path cover clean APPROVE; Task 4 (option): stale-sentinel limitation note |
| `agents/boundary-check.md` | Task 3: line 194 clarify silent-when-clean scope = advisory/branch only, PR mode luôn post (doc-only); Task 4 (alt location): stale-sentinel note |
| `README.md` | Task 5: 1-line clarify — PR mode always posts sentinel; silent-when-clean = ADVISORY/branch/range only (doc-only) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/block-unsafe-merge.sh` | One-disease-one-mechanism: KHÔNG sửa accept logic (incl. KHÔNG thêm SHA-scoping — đó là [P055]). Verify grep `<!-- security-review-start -->` + `^Verdict:` + `APPROVE` (lines 103-109) khớp NGUYÊN VĂN chuỗi mà slash command post ra. Nếu lệch 1 ký tự → end-to-end fail (anchor #8). |
| `.claude/commands/security-review.md` Step 0 mode resolution | PR vs branch/range mode đã phân đúng — Task 1/2 dựa vào mode này; không đổi Step 0. |
| `agents/boundary-check.md` anti-pattern list (line 248-260) | "KHÔNG auto-bóp NEEDS_REVIEW về APPROVE" giữ nguyên — fix này KHÔNG đụng verdict logic, chỉ caller post-decision. |

---

## Luật chơi (Constraints)

1. **One disease, one mechanism.** Sửa CHỈ emit-side (slash command) + doc-clarify. KHÔNG sửa `block-unsafe-merge.sh` accept-side. (Loại Option B.)
2. **Sentinel marker tái dùng nguyên văn** — `<!-- security-review-start -->` / `<!-- security-review-end -->` / `Verdict: APPROVE`. KHÔNG sáng chế chuỗi mới. Hook grep exact (block-unsafe-merge.sh:103-106) — lệch 1 ký tự = deadlock vẫn còn.
3. **Scope chặt theo mode.** PR mode (block-unsafe-merge-governed) → luôn post (cả clean APPROVE). Branch/range/ADVISORY mode → silent-when-clean GIỮ NGUYÊN. P042 chống approve-fatigue PHẢI còn nguyên cho advisory runs — chỉ đổi hành vi cho PR-gated. (V2 confirm — scoping detectable qua anchor #6.)
4. **Giám sát hành vi bất biến.** Giám sát vẫn luôn return sentinel block (đã đúng). KHÔNG đổi verdict logic, KHÔNG đổi 5-INV rubric.
5. **ADVISORY mode bất biến.** Slash command vẫn KHÔNG `gh pr merge --block`. Fix này chỉ đảm bảo INPUT cho block-unsafe-merge tồn tại; không biến /security-review thành blocking gate.
6. **(V2) Stale-sentinel = known-limitation, KHÔNG fix ở đây.** P053 KHÔNG thêm SHA-scoping. Limitation phải được DOCUMENT (Task 4) + filed = [P055]. KHÔNG âm thầm ship như "đã fix". Mirror block-unsafe-merge.sh:15-16 documented-bypass pattern.

---

## Nghiệm thu

### Automated
- [ ] Không có code runtime đổi (markdown handbook + slash command + README). Không cần type-check/test compile.
- [ ] Verify chuỗi sentinel khớp 2 đầu: `grep -F '<!-- security-review-start -->' .claude/commands/security-review.md agents/boundary-check.md scripts/block-unsafe-merge.sh` → cả 3 file có cùng chuỗi.

### Manual Testing
- [ ] **End-to-end clean-APPROVE (anchor #8):** Tạo nhánh touch security surface (vd thêm dòng vào `src/` hoặc `hooks/pre-commit`), PR, chạy `/security-review <PR>` với review SẠCH → confirm comment APPROVE posted (`gh pr view <PR> --json comments`). Rồi `gh pr merge <PR>` → confirm `block-unsafe-merge.sh` ALLOW (không còn deadlock, không cần override marker).
- [ ] **NEEDS_REVIEW path:** PR có FLAG → comment posted (như cũ) → `gh pr merge` block (như cũ, đúng).
- [ ] **Branch mode silent giữ nguyên:** `/security-review <branch>` clean → KHÔNG post/write (silent-when-clean còn sống cho advisory).

### Regression
- [ ] Override marker `[security-review-skip:<reason>]` vẫn hoạt động (block-unsafe-merge.sh:56) — không bị fix này phá.
- [ ] Non-security-surface PR vẫn merge tự do (block-unsafe-merge.sh:92-98 không đổi).
- [ ] Approve-fatigue: branch/range advisory runs vẫn im khi sạch (P042 intent giữ nguyên).
- [ ] **(V2) Stale-sentinel limitation DOCUMENTED** (Task 4 note shipped) + `block-unsafe-merge.sh` accept logic KHÔNG bị đụng (verify: `grep -n 'sha\|SHA\|head_sha\|created_at' scripts/block-unsafe-merge.sh` vẫn 0 hit — P053 không thêm scoping; đó là [P055]).

### Docs Gate (Tầng 1 — security-surface, BẮT BUỘC)
- [ ] `CHANGELOG.md` — entry P053 (sentinel deadlock fix, Option A; stale-sentinel known-limitation documented + filed [P055]).
- [ ] `agents/boundary-check.md` — đã update (Task 3) — đây là chính surface đổi.
- [ ] **Per CLAUDE.md DOCS GATE mapping (`agents/boundary-check.md` change → `docs/LAYERS.md` specialist subsection + `README.md` row):**
  - [ ] `docs/LAYERS.md` — specialist subagents subsection: nếu mô tả Giám sát silent-when-clean contract, update để phản ánh PR-mode-always-post scoping. `[needs Worker verify]` Worker grep `boundary-check\|Giám sát\|silent-when-clean` in `docs/LAYERS.md`; nếu có mô tả contract → update; nếu chỉ liệt kê tên agent → ghi "N/A — LAYERS.md chỉ liệt kê, không mô tả contract".
  - [ ] `README.md` — Giám sát / boundary-check contract line: **đã update (Task 5)** — PR mode always-post + silent-when-clean scope clarify. (V2: [O1.1-minor] ACCEPT.)
- [ ] `.claude/commands/security-review.md` — đã update (Task 1, 2, optionally 4) — slash command là chính surface đổi.
- [ ] Discovery Report ghi rõ "Tầng 1 docs updated: <list>" hoặc "Tầng 1 N/A cho file X (lý do)".

### Discovery Report
- [ ] Write to `docs/discoveries/P053.md`
  - Anchor #8 end-to-end result (deadlock thật giải chưa — hook allow merge sau clean APPROVE?)
  - Sentinel chuỗi 3-file khớp nguyên văn (CORRECT/WRONG với file:line)
  - Anchor #9 README line — đúng dòng mô tả contract? (line số thực tế vs V1-cited :74)
  - Task 4 stale-sentinel note — đặt ở file nào (security-review.md hay boundary-check.md)?
  - Có phát sinh edge nào: PR có cả APPROVE comment cũ + review mới? gh pr comment idempotency? (ket precedent có note không?)
  - Docs updated to match reality (list, hoặc "None — explicit")
  - Tier escalations (write "None" nếu không có)
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`

### Discovery debt to file (V2)
- [P055] đã được Architect append vào `docs/BACKLOG.md` Open backlog trong Turn 2 (Discovery→backlog flow). Nếu vì lý do nào đó dòng đó không có khi Worker EXECUTE, Worker append:
  `[P055] SHA-scope the block-unsafe-merge APPROVE sentinel — gate currently greps any historical APPROVE comment (no head-SHA binding); a stale clean APPROVE can green-light later unreviewed commits on a multi-commit PR. Surfaced by P053 CHALLENGE 2026-06-06. block-unsafe-merge.sh:102-106.`
