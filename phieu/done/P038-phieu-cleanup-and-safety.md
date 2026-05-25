# PHIẾU P038: Phiếu lifecycle cleanup, doc hygiene auto-warn, Worker safety rails

> **Loại:** Feature (workflow infrastructure)
> **Ưu tiên:** P1 (cost optimization — Tarot dogfood chạm 80% week usage 2 tuần liên tiếp; non-blocking nhưng ROI cao)
> **Tầng:** 1 (móng nhà — đụng `agents/orchestrator.md`, `agents/worker.md`, `docs/ORCHESTRATION.md`, `phieu/phieu.sh`, `hooks/pre-commit`. State machine + safety contract)
> **Ảnh hưởng:** `agents/worker.md`, `phieu/phieu.sh`, `hooks/pre-commit`, `docs/ORCHESTRATION.md`, `agents/orchestrator.md` (session-start nudge), `scripts/session-start-banner.sh` (size warn check), `phieu/TICKET_TEMPLATE.md` (snapshot Task 0 reference), `docs/BACKLOG.md` (promote v2.2 wave → close), `CHANGELOG.md`
> **Dependency:** None. P036 (tier routing) đã ship — phiếu này NOT re-spec tier; chỉ cleanup + safety + doc hygiene.

---

## Context

### Vấn đề hiện tại

**Trigger:** 2 tuần dogfood sos-kit ở Tarot project chạm 80% week usage Max plan mỗi tuần. Trước sos-kit: < 80% never. P109 ($4.82/phiếu Tầng 2) baseline đo 2026-05-02 — `/cost` meter ghi "80% usage came from subagent-heavy sessions".

**Phân tích root cause** (data session 2026-05-02 + memory `project_p109_cost_baseline.md`):

1. **Auto-load context phình** mỗi Architect spawn:
   - Tarot CHANGELOG.md = 67k bytes (~17k tokens)
   - Tarot DISCOVERIES.md = **110k bytes (~28k tokens)** — sau 5 ngày đã phình to vì Worker write per-phiếu vào file dùng chung
   - Architect tự Read theo CLAUDE.md rule → load 45k tokens auto + reload mỗi spawn (cache giảm cost nhưng cache write 230k Opus = $0.86/phiếu)
   - **Không có warn khi vượt threshold** → Sếp không biết khi nào "gọi thợ cắt"

2. **Phiếu lifecycle thiếu cleanup**:
   - `phieu-done` (phieu.sh:160-174) chỉ remove worktree. KHÔNG strip Debate Log, KHÔNG delete branch local, KHÔNG cleanup backup.
   - Phiếu archive (`phieu/done/`) giữ full V2/V3 Debate Log → Architect grep tham chiếu phiếu cũ load nguyên Debate text → tốn token vô ích.
   - Branch local `feat/P<NNN>-*` accumulate sau merge (Sếp xác nhận: "làm xong phiếu không chuyển sang archive, nhiều khi đọc hết tốn công lắm")

3. **Worker safety chưa hard-rule**:
   - Worker EXECUTE có thể accidentally edit `~/.claude/projects/*/memory/*` (memory files, ngoài git project) → mất.
   - Có thể edit `.claude/settings.local.json` (permission allowlist Sếp build dần) → mất permissions.
   - Có thể `git push --force` hoặc `git reset --hard` ngoài branch phiếu → destroy work.
   - Memory `feedback_kill_process_specific_pid.md` đã có 1 incident similar (pkill orphan tóm cả active task) → safety rail là pattern đã verified.

4. **Pre-phiếu snapshot không có**:
   - Sếp work local; nếu phiếu fail mid-execute, không có rollback point cho settings.local.json + .sos-state markers.
   - GitHub backup chỉ cover code; settings + memory + state markers ngoài git.

5. **DISCOVERIES.md monolithic file**:
   - Mỗi phiếu Worker append → file phình. Architect Read full file (CLAUDE.md rule) cho mỗi phiếu mới → 95% nội dung không liên quan phiếu hiện tại.
   - Pattern decouple: per-phiếu Discovery file → Architect Read selective (chỉ khi phiếu mới reference component cũ).

6. **Session-start nudge không có**:
   - Banner hiện chỉ surface Active sprint. Không scan `phieu/active/` xem có phiếu approved+merged đang chờ `phieu-done`. Sếp dễ quên cleanup.

### Giải pháp

**6 sub-scopes** (1 phiếu Tầng 1 vì cùng đụng workflow contract). Tier routing (P036) đã ship — phiếu này KHÔNG re-spec, chỉ note cross-reference.

**1. Auto size warn (CHANGELOG/DISCOVERIES > 40k bytes)**
- Add check vào `scripts/session-start-banner.sh` (banner surface, không block commit). Threshold: 40k bytes.
- Format banner line: `⚠️  docs/CHANGELOG.md (52k > 40k threshold) — gọi thợ trim, archive cũ ra docs/archive/`
- KHÔNG block — chỉ warn. Sếp quyết khi nào trim.

**2. Worker safety hard rules** — extend `agents/worker.md` "Hard envelope rules" + "Anti-patterns".

**3. Pre-phiếu snapshot (Task 0 standard)**
- Update `phieu/TICKET_TEMPLATE.md` Task 0 — Worker auto first-step (mkdir/cp/git rev-parse).
- Add `.backup/` to `.gitignore`. Cleanup integrated với `phieu-done` (scope #5).

**4. DISCOVERIES decoupling — per-phiếu file pattern**
- New: `docs/discoveries/P<NNN>.md` (per-phiếu). Old monolithic → archive. Index file points to per-file.

**5. Cleanup `phieu-done` extension** — strip Debate Log, move active→done, delete branch (`-d` safe), cleanup `.backup/`. Location detect (`phieu/active/` vs `docs/ticket/`).

**6. Session-start cleanup nudge** — scan `phieu/active/` for approved+merged phiếu, echo `phieu-done <P>` hint. Use `git branch --merged main` (không cần `gh` CLI).

### Scope

- **CHỈ sửa**:
  - `agents/worker.md` (scope #2 — safety rails)
  - `agents/orchestrator.md` (scope #6 — cross-ref session-start nudge, ≤2 lines per CLAUDE.md ≤90 cap, V3 ACCEPT [O1.1])
  - `phieu/phieu.sh` (scope #5 — phieu-done extension)
  - `phieu/TICKET_TEMPLATE.md` (scope #3 + #4 + Anchor #9 drift fix — snapshot Task 0 + Discovery path + line 4 dual-path note, V3 ACCEPT [O1.2])
  - `scripts/session-start-banner.sh` (scope #1 + #6 — size warn + cleanup nudge)
  - `docs/ORCHESTRATION.md` (scope #5 + #6 — document phiếu-done lifecycle + cleanup nudge contract)
  - `docs/BACKLOG.md` (close v2.2 wave entry — see Note below; move Recently shipped)
  - `CHANGELOG.md` (entry P038)
  - `.gitignore` (add `.backup/`)
  - **NEW**: `docs/discoveries/P038.md` (eat own dogfood — per-phiếu file from day 1)

- **KHÔNG sửa**:
  - `agents/architect.md` (no architect change)
  - `phieu/RELAY_PROTOCOL.md`, `phieu/DISCOVERY_PROTOCOL.md` (orthogonal — DISCOVERY_PROTOCOL.md may need 1-line cross-ref to per-file pattern; defer to Worker EXECUTE judgment)
  - Tier routing logic in `agents/orchestrator.md:40-46` (P036 frozen)
  - `phieu/VISION_TEMPLATES/*` (no day-1 skeleton change)
  - `skills/*` (no skill change)
  - `hooks/pre-commit` (size warn lives in banner, not commit-blocker — keep pre-commit lean)
  - `phieu/phieu.sh:89` (phieu-create downstream-first; sos-kit places phiếu manually — V3 [O1.2] DEFEND on phieu.sh side)
  - Tarot directly — Tarot adapts after sos-kit ships

**BACKLOG note**: P038 không có trong Active sprint hiện tại (sprint = "Worker capability + install UX gaps" P005 + P006). Sếp invoke trực tiếp với cost-trigger evidence (80% week usage 2 tuần). Treat as **P0 cost-driven hotfix** per Architect Rule 0 exception. Worker EXECUTE phải update BACKLOG.md sau ship: add P038 entry vào "Recently shipped" + close v2.2 wave entry (line 49 — "v2.2 — Debate token optimization" — P038 đáp ứng phần lớn ý đó).

---

## Task 0 — Verification Anchors

> Architect verified via Read (no Bash/Grep tools). All anchors marked với confidence level.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `phieu/phieu.sh:_phieu_done_impl` (line 160-174) hiện chỉ remove worktree, không strip phiếu hay delete branch | Read phieu/phieu.sh:160-174 | ✅ **VERIFIED** [verified] — function tại line 160-174 chỉ chạy `git worktree remove "${wt_parent}/$name"` rồi echo. Không strip, không branch delete, không backup cleanup. Comment line 173: "(branch still exists)". Confirmed. |
| 2 | `agents/orchestrator.md` line 40-46 đã spec tier routing P036 (Tầng 2 skip CHALLENGE) — phiếu này KHÔNG đụng tier logic | Read agents/orchestrator.md:40-46 | ✅ **VERIFIED** [verified] — line 40-46 = "## Tier routing (P036)" section. Logic intact: Tầng 2 → DRAFT→APPROVAL→EXECUTE skip CHALLENGE; Tầng 1 → full debate; Worker may escalate 2→1 NEVER 1→2 demote. **DO NOT TOUCH** trong phiếu này. |
| 3 | `agents/worker.md` có "Hard envelope rules" section để extend safety rails | Read agents/worker.md | ✅ **VERIFIED** [verified] — line 12-26 = "## Hard envelope rules" section. Hiện có 3 CANNOT bullets (line 16-19) + 4 MUST NOT bullets (line 21-25). Anti-patterns không phải section riêng — embed trong "Tầng 1 vs Tầng 2" + "Hand-back format". **Note Worker**: phiếu spec "Anti-patterns section" — actually need add **NEW section** giữa "Hard envelope rules" và "Why this envelope" hoặc append cuối. Worker decide placement Tầng 2. |
| 4 | `scripts/session-start-banner.sh` tồn tại + hiện đang surface Active sprint | Read scripts/session-start-banner.sh | ✅ **VERIFIED** [verified] — file exists, 85 lines. Structure: parse BACKLOG.md Active sprint (line 22-48) → echo banner block (line 56-83). Có sẵn `set -uo pipefail` (line 13) + cross-platform sed/grep/awk (line 11 comment). Add size-warn + cleanup-nudge sau line 68 (sau Orchestrator contract block, trước Architect Rule 0). |
| 5 | `phieu/TICKET_TEMPLATE.md` Task 0 section structure + Nghiệm thu → Discovery Report | Read phieu/TICKET_TEMPLATE.md | ✅ **VERIFIED** [verified] — Task 0 section line 32-44 (header + 3-row example table). Nghiệm thu Discovery Report line 145-150 (path = `docs/DISCOVERIES.md`). Add **Pre-phiếu snapshot** subsection sau line 44 (after the "If Result column has ❌" note). Update Discovery Report path: `docs/DISCOVERIES.md` → `docs/discoveries/P<NNN>.md` (per-phiếu pattern). |
| 6 | `hooks/pre-commit` exists + structure để hiểu add size-warn vào hook hay banner | Read hooks/pre-commit | ✅ **VERIFIED** [verified] — file exists, 169 lines. 3 phases: type-check, docs-gate, sos-kit v2 checks. **Decision**: KHÔNG add size warn vào pre-commit — pre-commit đã đủ tải (3 phases, có thể block commit). Banner surface tốt hơn vì non-blocking + show mỗi session start. |
| 7 | `docs/ORCHESTRATION.md` phiếu-done lifecycle chưa có spec hiện tại | Read docs/ORCHESTRATION.md | ✅ **VERIFIED** [verified] — file 200 lines. Sections: Why 4th role / Session opening / State machine / Tier routing P036 / Trigger phrases / Hard rules (8 rules) / Failure modes / Concrete example / Replaces. **NO** section về phiếu-done lifecycle hay cleanup hygiene. Add NEW section "Phiếu lifecycle (post-ship)" between "Failure modes + recovery" (line 119-129) và "Concrete example session" (line 131-195). |
| 8 | `.gitignore` chưa có `.backup/` | Read .gitignore | ✅ **VERIFIED** [verified] — file 14 lines. Có `.sos/` + `.sos-state/` (line 13-14) nhưng KHÔNG có `.backup/`. Append `.backup/` vào cuối với comment. |
| 9 | sos-kit dogfood phiếu lưu ở `phieu/active/` và `phieu/done/`, NOT `docs/ticket/` (template line 4 đề cập docs/ticket/ là DRIFT) | Glob + Read template:4 | ⚠️ **DRIFT CONFIRMED** [verified] — `phieu/active/P038-phieu-cleanup-and-safety.md` exists (current file Architect đang đọc). `phieu/TICKET_TEMPLATE.md:4` literal: "Filename: `docs/ticket/P<NNN>-<slug>.md`". Drift cũng có ở `phieu.sh:89` `local ticket_file="docs/ticket/${id}-${slug}.md"` (hardcoded). **V3 resolution**: Task 3 expanded with micro-edit để update template line 4 sang dual-path note (sos-kit + downstream). `phieu.sh:89` left unchanged — phieu-create là downstream-first command, sos-kit không dùng để generate phiếu cho chính nó. |

**Summary**: 8/9 ✅ verified, 1/9 ⚠️ (DRIFT confirmed — handled in Task 3 micro-edit + Task 5 location detect). Zero ❌. Architect confidence high — Worker re-verify khi CHALLENGE.

---

## Debate Log

**Phiếu version:** V3 (Architect responded to Worker Turn 1 — both objections ACCEPTED, surgical edits applied)

### Turn 1 — Worker Challenge (phiếu V2)

**Anchor verification:**
- Anchor #1: ✅ `phieu/phieu.sh:160-174` — `_phieu_done_impl` confirmed: only `git worktree remove` + echo. Line 173 literal: `"✅ Worktree removed: ${wt_parent}/$name (branch still exists)"`. No strip, no branch delete, no backup.
- Anchor #2: ✅ `agents/orchestrator.md:40-46` — Tier routing section intact. Lines 42-46 confirmed unchanged. **DO NOT TOUCH**.
- Anchor #3: ✅ `agents/worker.md:12-26` — Hard envelope rules confirmed. Line 26 = "Skip Discovery Report — every phiếu ends there". Line 27 = "## Why this envelope". Insert point for new subsection: after line 26, before line 27. No existing Anti-patterns top-level section (`grep ^## Anti-patterns` = empty). Line 175 = "Save observations for Discovery Report." Line 177 = "## MANDATORY: track work + ask via tools". Insert new section between 175 and 177.
- Anchor #4: ✅ `scripts/session-start-banner.sh` — 85 lines. `FALLBACK_USED` block ends at line 68 (closing `fi`). Line 69 = blank. Line 70 = `echo "🤖 Orchestrator contract..."`. Insert point for Tasks 1+6: after line 68.
- Anchor #5: ✅ `phieu/TICKET_TEMPLATE.md:32-44` — Task 0 section confirmed. Line 43 = "If Result column has ❌..." note. Line 44 = blank. Line 45 = `---`. Line 145 = "### Discovery Report". Line 146 = "- [ ] Append entry to `docs/DISCOVERIES.md` (newest on top, like CHANGELOG)". Matches Architect spec.
- Anchor #6: ✅ Decision confirmed: size warn in banner, NOT pre-commit. `hooks/pre-commit` = 169 lines, 3 phases — already loaded. Pre-commit phiéu detection (line 114) already only matches `docs/ticket/P[0-9]+-` → sos-kit phiéu at `phieu/active/` are already outside its scope (pre-existing no-op for sos-kit). Keeping pre-commit untouched is correct.
- Anchor #7: ✅ `docs/ORCHESTRATION.md` — "## Failure modes + recovery" at line 119, "## Concrete example session" at line 131. Last row of Failure modes table = line 129 "Worker silently demoted Tầng 1 → Tầng 2". Insert new section between line 130 (blank) and line 131.
- Anchor #8: ✅ `.gitignore` — 12 lines (not 14 as Architect estimated — `.sos/` is line 12, `.sos-state/` is line 13 — actually 13 lines total). Append `.backup/` after last line.
- Anchor #9: ⚠️ DRIFT CONFIRMED — `phieu/TICKET_TEMPLATE.md:4`: "Filename: `docs/ticket/P<NNN>-<slug>.md`". `phieu.sh:89`: `local ticket_file="docs/ticket/${id}-${slug}.md"` hardcoded. See **[O1.2]** below.

**Objections (Tầng 1 only):**

- **[O1.1] `agents/orchestrator.md` exceeds ≤90-line cap (CLAUDE.md rule).** Current line count: `wc -l` = **88 lines**. Task 7 inserts ~4 lines for the "Phiérm cleanup nudge" section (`## Phiếu cleanup nudge (P038)` + blank + content line + trailing blank). New total: **92 lines**. CLAUDE.md line 106 says "~85 lines, ≤90 cap". The new section itself is correct behavior; it's the documented cap that it violates. Evidence: `agents/orchestrator.md:88` (last line), `CLAUDE.md:106`. Two alternatives:
  - **A. Compress the new section to 2 lines** (drop the blank line before it; merge the "You MUST surface... Full lifecycle spec..." sentence into a terse one-liner ≤80 chars). Result: 88 + 2 = 90 lines — exactly at cap. Worker lean. Rationale: the section's content (nudge + MUST NOT auto-run + cross-ref) fits in 1 imperative sentence given orchestrator.md's dense style.
  - **B. Raise the cap in CLAUDE.md to 95** (adjust the comment to reflect new stable size). Less elegant — cap exists precisely because orchestrator.md is the runtime system-prompt contract, and verbosity has token cost. Against cap's intent.

- **[O1.2] Anchor #9 DRIFT unaddressed — no task to fix `phieu/TICKET_TEMPLATE.md:4` or `phieu.sh:89`.** Architect noted in Anchor #9: "Template line 4 update... cũng nên trong scope." Files cần sửa lists TICKET_TEMPLATE.md for Task 3 and Task 4 — neither updates line 4. The template still says `Filename: docs/ticket/P<NNN>-<slug>.md` (misleading for sos-kit users). `phieu.sh:89` still hardcodes `docs/ticket/` for phieu-CREATE (Task 5 only updates phieu-DONE). This is an architectural gap: after P038, phieu-done handles both paths but phieu-create still creates at `docs/ticket/`. Evidence: `phieu/TICKET_TEMPLATE.md:4`, `phieu/phieu.sh:89`. Two alternatives:
  - **A. Add explicit micro-task: update TICKET_TEMPLATE.md:4 to note dual-path convention** (e.g., `Filename: phieu/active/P<NNN>-<slug>.md` for sos-kit, `docs/ticket/P<NNN>-<slug>.md` for downstream). Leave `phieu.sh:89` unchanged (phieu-create is downstream-first; sos-kit manually places phiéu). This is surgical, ~1 line, zero new files. Worker lean. Rationale: fixes template as documentation; phieu-create already mirrors template for downstream projects correctly; sos-kit workaround is manual and documented.
  - **B. Defer to next phiéu** — treat it as a separate `docs/` hygiene task. TICKET_TEMPLATE.md line 4 drift stays but is harmless until phieu-create is run on sos-kit itself (which it currently is not). P038 ships without it, next phiéu addresses it.

**Status:** ✅ RESPONDED (see Turn 1 — Architect Response below)

### Turn 1 — Architect Response (phiếu V3)

- **[O1.1] → ACCEPT (Option A)**. Worker right. CLAUDE.md:106 cap (≤90) is explicit, no exception clause; cap intent = orchestrator.md is runtime system-prompt contract, every line has token cost per session. **Action taken**: Task 7 File 2 (`agents/orchestrator.md`) rewrite — new section condensed from ~5 lines (`## H2 + blank + 2-sentence body + trailing blank`) to **exactly 2 lines** (`## H2 header` + `1-sentence body with cross-ref`). New total: 88 + 2 = **90 lines** (at cap). Cross-ref to `docs/ORCHESTRATION.md` "Phiếu lifecycle" preserves full spec; handbook stays terse per CLAUDE.md intent.
- **[O1.2] → ACCEPT (Option A, partial)**. Worker right on TICKET_TEMPLATE.md:4 — Architect noted intent in Anchor #9 ("Template line 4 update... cũng nên trong scope") but failed to spec a task. That is a Tầng 1 gap (template = doc contract for downstream consumers). **Action taken**: Task 3 expanded — new "File 1B" sub-edit updates `phieu/TICKET_TEMPLATE.md:4` with dual-path note. **Defend on `phieu.sh:89`**: phieu-create is the downstream-first command (users of sos-kit run it in their own repos to create phiếu); sos-kit itself doesn't use phieu-create to generate its own phiếu (Sếp drafts manually in `phieu/active/`). Changing line 89 = false friction (would break downstream pattern users invoke). Worker confirmed this stance in Option A reasoning. Scope section updated to make explicit: phieu.sh:89 in "KHÔNG sửa".

**Status:** ✅ RESPONDED — phiếu bumped to V3, both objections resolved, no DEFER. Ready for next CHALLENGE round or APPROVAL_GATE.

### Final consensus
- Phiếu version: V3
- Total turns: 1 (Worker Challenge → Architect Response, both ACCEPT)
- Approved by Chủ nhà: 2026-05-02 — code execution may begin

---

## Nhiệm vụ

### Task 1: Auto size warn for CHANGELOG/DISCOVERIES > 40k

**File:** `scripts/session-start-banner.sh`

**Decision:** Surface = banner (NOT pre-commit hook). Rationale: pre-commit có thể block commit và đã đủ tải; banner non-blocking, show mỗi session start, đúng "nudge không block" pattern (Constraint #2).

**Tìm:** sau line 68 (`if [ "$FALLBACK_USED" = "1" ]; then ... fi`), trước line 70 (`echo "🤖 Orchestrator contract..."`)

**Thay bằng / Thêm:** insert block sau line 68 closing `fi`:

```bash

# ─────────────────────────────────────────────────────────────────────
# Doc size warn (P038) — non-blocking nudge, threshold 40k bytes
# ─────────────────────────────────────────────────────────────────────
SIZE_THRESHOLD=40960  # 40k bytes
SIZE_WARNS=""
for doc in "docs/CHANGELOG.md" "docs/DISCOVERIES.md" "CHANGELOG.md"; do
    [ ! -f "$doc" ] && continue
    # Cross-platform byte count: prefer wc -c (POSIX), fall back to stat
    bytes=$(wc -c < "$doc" 2>/dev/null | tr -d ' ')
    [ -z "$bytes" ] && continue
    if [ "$bytes" -gt "$SIZE_THRESHOLD" ]; then
        kb=$((bytes / 1024))
        SIZE_WARNS="${SIZE_WARNS}⚠️  ${doc} (${kb}k > 40k threshold) — gọi thợ trim, archive cũ ra docs/archive/\n"
    fi
done
if [ -n "$SIZE_WARNS" ]; then
    echo ""
    echo "📏 Doc size warning:"
    printf "    %b" "$SIZE_WARNS"
fi
```

**Lưu ý:**
- Cross-platform: `wc -c < file` works on bash 4+, zsh 5+, macOS, Linux. KHÔNG dùng `stat -c` (GNU-only) hoặc `stat -f` (BSD-only).
- Loop cover cả `docs/CHANGELOG.md` (Tarot pattern) và root `CHANGELOG.md` (sos-kit pattern). Skip silently nếu file không tồn tại.
- `printf "%b"` để render `\n` literal trong `$SIZE_WARNS`.
- KHÔNG `exit 1` nếu warn — banner contract = non-blocking.
- Test by: `dd if=/dev/zero bs=1024 count=50 >> docs/CHANGELOG.md` (pad to 50k) → run banner → see warn → revert.

---

### Task 2: Worker safety hard rules

**File:** `agents/worker.md`

**Tìm:** line 12-26 = "## Hard envelope rules" section. Hiện kết thúc tại line 26 (`- Skip Discovery Report — every phiếu ends there`).

**Thay bằng / Thêm:** sau line 26, BEFORE line 27 (`## Why this envelope`), insert NEW subsection:

```markdown

### Destructive op safety rails (P038)

You MUST NOT (these are hard-stops — escalate via AskUserQuestion if phiếu seems to require them):

- `git push --force` / `git push -f` on ANY branch (including phiếu branch). Rationale: rebase conflicts on phiếu branch should escalate to Sếp, not be force-resolved silently.
- `git reset --hard` outside the current phiếu's worktree. Rationale: only the phiếu branch's working tree is your sandbox; main / other branches are untouchable.
- Edit any path under `~/.claude/projects/*/memory/*`. Rationale: Sếp's auto-memory is cross-session state; Worker overwriting it = silent context loss.
- Edit `.claude/settings.local.json` UNLESS the phiếu explicitly lists it in "Files cần sửa". Rationale: permission allowlist accumulates over time (P037 pattern); Worker mass-edit = lost permissions.
- Delete files under `.sos-state/`. Rationale: Orchestrator owns marker hygiene (architect-active marker); Worker delete = state-machine corruption.
- `rm -rf` on absolute paths or `~/`. Rationale: blast radius beyond phiếu scope. Use relative paths within worktree only.

When the phiếu seems to need any of the above → STOP, escalate via `AskUserQuestion` with options: A. abandon op, B. Sếp executes manually, C. update phiếu scope (return to Architect).
```

**Tìm (second edit):** any "Anti-patterns" section in worker.md. **Note**: re-Read confirmed worker.md hiện KHÔNG có top-level "## Anti-patterns" section (chỉ có "Hand-back format" + "Voice"). 

**Thay bằng:** Append NEW section sau line 175 (`Save observations for Discovery Report.`), BEFORE `## MANDATORY: track work + ask via tools` (line 177):

```markdown

## Anti-patterns (P038 safety addition)

1. **Editing memory/settings outside phiếu scope.** "While I'm here, let me also..." → NO. Memory + settings = Sếp's cross-session state, not Worker's surface.
2. **Force-pushing to recover from rebase conflict.** Escalate to Sếp; conflict resolution = Tầng 1 by definition (touches main branch history).
3. **`pkill -f <pattern>` to clean up orphans.** Use `kill <PID>` after `ps aux | grep <pattern>` confirms which PID. Memory: `feedback_kill_process_specific_pid.md` (2026-04-28 pkill vitest tóm cả task active).
4. **Mass `rm` to clean test artifacts.** Targeted `rm <specific-file>` only; if uncertain, leave it (banner size-warn will nudge).

```

**Lưu ý:**
- Total addition: 1 new subsection trong Hard envelope (~10 lines) + 1 new top-level Anti-patterns section (~8 lines).
- Backwards compat: existing rules (line 16-25) untouched. Just extending.
- Memory cross-ref: `feedback_kill_process_specific_pid.md` cho anti-pattern #3 (Sếp đã verify pattern này).
- Worker EXECUTE: re-read worker.md before edit để confirm latest line numbers — file có thể đã modified bởi P037/P035.

---

### Task 3: Pre-phiếu snapshot Task 0 standard + template line 4 drift fix (V3 ACCEPT [O1.2])

**File 1:** `phieu/TICKET_TEMPLATE.md`

**Decision:** Worker auto first-step (NOT Orchestrator spawn-prep). Rationale: Worker đã chạy Task 0 first action mỗi EXECUTE; thêm 4 lines bash đầu file = zero overhead, zero coordination cost. Orchestrator-side = thêm marker file logic + cross-spawn state, đắt hơn.

**Sub-edit 1A — Pre-phiếu snapshot subsection (existing scope):**

**Tìm:** line 32-44 = current Task 0 section ("## Task 0 — Verification Anchors" + 3-row example table + "If Result column has ❌" note at line 43-44).

**Thay bằng / Thêm:** sau line 44 (existing "If Result column has ❌..." note), BEFORE line 45 (`---` separator), insert:

```markdown

### Pre-phiếu snapshot (Worker auto first-step)

> **Worker EXECUTE FIRST ACTION** (before any code edit, before Task 0 grep verification): take a rollback point so failed mid-execute can revert.

```bash
# Run from project root (worktree root for phiếu workflow):
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/ — auto-cleaned on phieu-done"
```

If the phiếu hits ❌ mid-execute and you need to roll back: `cp .backup/${PHIEU_ID}/settings.local.json .claude/` and `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (within phiếu worktree only — NEVER on main per safety rails).

`.backup/` is gitignored. `phieu-done` cleans up automatically.

```

**Sub-edit 1B — Line 4 dual-path note (V3 ACCEPT [O1.2], Anchor #9 drift fix):**

**Tìm:** line 4 in `phieu/TICKET_TEMPLATE.md` literal:

```markdown
> Filename: `docs/ticket/P<NNN>-<slug>.md`
```

**Thay bằng:**

```markdown
> Filename: `phieu/active/P<NNN>-<slug>.md` (sos-kit dogfood layout) **OR** `docs/ticket/P<NNN>-<slug>.md` (downstream projects using `phieu-create` from `phieu/phieu.sh`). Both paths are recognized by `phieu-done` (P038 location detect — see Task 5).
```

**Lưu ý sub-edit 1B:**
- This is the ONLY edit to template line 4. Single-line replace, zero impact on template structure.
- `phieu/phieu.sh:89` (`local ticket_file="docs/ticket/${id}-${slug}.md"`) deliberately UNCHANGED. `phieu-create` is downstream-first (users invoke in their own repos); sos-kit itself drafts phiếu manually in `phieu/active/` (no `phieu-create` for own repo). Changing line 89 = false friction for downstream users. Architect [O1.2] DEFEND on phieu.sh side, ACCEPT on template side.
- Verify: re-Read template line 4 BEFORE edit — if file has been modified between Architect Read and Worker EXECUTE (e.g., P037 touched it), update line number accordingly. Pattern is the literal "Filename: `docs/ticket/`" — search by content not line number if uncertain.

**File 2:** `.gitignore`

**Tìm:** line 14 (last line) `.sos-state/`

**Thay bằng:** append after line 14:

```

# Pre-phiếu snapshot rollback points (P038)
.backup/
```

**Lưu ý:**
- `PHIEU_ID` extraction từ worktree directory name (e.g., `P038-phieu-cleanup-and-safety` → `P038`). Works because `phieu.sh:88` creates `wt_dir="${wt_parent}/${wt_name}"` where `wt_name="${id}-${slug}"`.
- `2>/dev/null || true` — fresh project chưa có settings.local.json hoặc .sos-state, snapshot vẫn proceed.
- Cleanup integrated trong Task 5 (`phieu-done` extension) — không cần Worker manual cleanup.
- `git rev-parse HEAD` capture hash hiện tại (phiếu branch HEAD), KHÔNG main HEAD — đúng intent (rollback within phiếu, không cross-branch).
- KHÔNG add `.backup/` vào downstream `bin/sos.sh` install (Sếp không request; defer to user-side gitignore — sos-kit's `.gitignore` covers this repo only). `bin/sos.sh` decision = Tầng 2, Worker self-decide if scope expands.

---

### Task 4: DISCOVERIES decoupling — per-phiếu file pattern

**File 1 (NEW):** `docs/discoveries/.gitkeep` (or first per-phiếu file as P038 dogfood)

**File 2:** `docs/DISCOVERIES.md` (existing — convert to index)

**File 3:** `docs/archive/DISCOVERIES_pre-2026-05.md` (NEW — old monolithic archive)

**Migration plan:**

1. Create `docs/archive/` if not exists.
2. Move current full `docs/DISCOVERIES.md` content → `docs/archive/DISCOVERIES_pre-2026-05.md` verbatim.
3. Replace `docs/DISCOVERIES.md` content with index-only:

```markdown
# Discoveries — sos-kit

> **Index file.** Per-phiếu Discovery Reports live at `docs/discoveries/P<NNN>.md` (one file per phiếu). This file = chronological index, newest on top.
>
> **Why per-file:** monolithic file phình to 100k+ bytes after ~30 phiếu. Architect's auto-Read of full file wastes tokens on irrelevant content. Per-phiếu = Architect Read selective (only when current phiếu references the same component).
>
> **Pre-2026-05 entries:** archived to `docs/archive/DISCOVERIES_pre-2026-05.md`. Migration date: 2026-05-02 (P038).

## Index

| Phiếu | Date | 1-line summary |
|---|---|---|
| [P038](discoveries/P038.md) | 2026-05-02 | Phiếu lifecycle cleanup + safety rails (THIS phiếu — Worker fill on EXECUTE) |
| [P037](archive/DISCOVERIES_pre-2026-05.md#p037) | 2026-04-27 | Marker file pre-approve template |
| [P036](archive/DISCOVERIES_pre-2026-05.md#p036) | 2026-04-27 | Tier routing + humility markers |
| [P035](archive/DISCOVERIES_pre-2026-05.md#p035) | 2026-04-27 | Orchestrator handbook + bulk-input |

<!-- Future entries: 1 line per phiếu, link to per-file. Worker on EXECUTE adds entry as last step before commit. -->
```

4. Each per-phiếu file structure (`docs/discoveries/P<NNN>.md`):

```markdown
# Discovery Report — P<NNN>: <slug>

> Worker fills on EXECUTE final step. Architect Reads selectively (only when next phiếu references same component as this one).

## Assumptions in phiếu — CORRECT
- Anchor #N: ✅ verified at `file:line`

## Assumptions in phiếu — WRONG
- (None / list mismatches with file:line)

## Scope expansions (if any)
- Original plan: ...
- Shipped: ...
- Reason: ...

## Edge cases / limitations found
- ...

## Docs updated to match reality
- (None / list)

## Tier escalations
- (None / "escalated 2→1, reason: ...")

## Cross-platform issues
- (None / list bash vs zsh vs Linux vs macOS)
```

**File 4:** `phieu/TICKET_TEMPLATE.md` Discovery Report section update.

**Tìm:** line 145-150 in TICKET_TEMPLATE.md:

```markdown
### Discovery Report
- [ ] Append entry to `docs/DISCOVERIES.md` (newest on top, like CHANGELOG)
  - Assumptions in phiếu — CORRECT / WRONG
  - Edge cases / limitations found
  - Docs updated to match reality
```

**Thay bằng:**

```markdown
### Discovery Report
- [ ] Write to `docs/discoveries/P<NNN>.md` (per-phiếu file, P038 pattern)
  - Assumptions in phiếu — CORRECT / WRONG (with file:line citations)
  - Scope expansions (if any — note original vs shipped, with reason)
  - Edge cases / limitations found
  - Docs updated to match reality (write "None" if nothing — explicit)
  - Tier escalations (write "None" if no 2→1 escalation)
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md` (link to per-phiếu file)
```

**File 5:** `CLAUDE.md` rule about Discovery path.

**Tìm:** Sếp's instruction to do this is in BACKLOG.md context but CLAUDE.md current does NOT have a Discovery path rule explicitly (Architect re-Read CLAUDE.md confirms — file is sos-kit-specific repo conventions, no per-project Discovery path rule). 

**Action:** SKIP CLAUDE.md edit for sos-kit dogfood — sos-kit's own discoveries already covered by P038 dogfood + index. Downstream projects (Tarot) will get the rule update via separate phiếu after sos-kit ships (downstream pull pattern, per Scope section "Tarot adapts after sos-kit ships").

**File 6:** `agents/worker.md` Discovery Report path reference.

**Tìm:** line 110-115 in worker.md:

```markdown
7. **Append Discovery Report** to `docs/DISCOVERIES.md` (newest on top):
```

**Thay bằng:**

```markdown
7. **Write Discovery Report** to `docs/discoveries/P<NNN>.md` (per-phiếu file, P038 pattern). Append 1-line index entry to `docs/DISCOVERIES.md`:
```

(rest of bullet points unchanged — line 111-115 stay as-is, just rephrase top line.)

**Lưu ý:**
- Migration script (Worker EXECUTE): `mkdir -p docs/archive docs/discoveries && cp docs/DISCOVERIES.md docs/archive/DISCOVERIES_pre-2026-05.md && cat > docs/DISCOVERIES.md <<EOF\n[index content]\nEOF`. Note: `docs/discoveries/` empty until Worker writes P038.md as dogfood (Nghiệm thu Discovery Report task).
- Backwards compat: old monolithic file preserved at archive path; existing index links remain navigable. No git history loss.
- Tarot pull: separate phiếu cho Tarot side (sau khi sos-kit P038 ship). Tarot's CLAUDE.md có Discovery rule explicit — đó là phiếu khác.
- **Worker note**: nếu `docs/DISCOVERIES.md` ở sos-kit hiện CHƯA tồn tại (sos-kit là meta-kit, không có Discovery file riêng) → skip migration step, just create new index file from scratch + create empty `docs/discoveries/` directory + write `docs/discoveries/P038.md` per Nghiệm thu. Verify before migrating: `ls docs/DISCOVERIES.md`. **[needs Worker verify]**: sos-kit có `docs/DISCOVERIES.md` không? Architect không thấy reference trong CLAUDE.md hay BACKLOG.md, nhưng `hooks/pre-commit:117` reference `docs/DISCOVERIES.md` → có thể tồn tại. Worker grep verify trước khi viết.

---

### Task 5: phieu-done cleanup extension

**File:** `phieu/phieu.sh`

**Tìm:** line 160-174 = current `_phieu_done_impl`:

```bash
_phieu_done_impl() {
  local cmd="$1"
  local root="$2"
  local wt_parent="$3"
  shift 3
  if [ -z "$1" ]; then
    echo "Usage: ${cmd} <P042-slug>"
    echo "Example: ${cmd} P042-user-export"
    return 1
  fi
  local name="$1"
  cd "$root" || return 1
  git worktree remove "${wt_parent}/$name" && \
    echo "✅ Worktree removed: ${wt_parent}/$name (branch still exists)"
}
```

**Thay bằng:**

```bash
_phieu_done_impl() {
  local cmd="$1"
  local root="$2"
  local wt_parent="$3"
  shift 3
  if [ -z "$1" ]; then
    echo "Usage: ${cmd} <P042-slug>"
    echo "Example: ${cmd} P042-user-export"
    return 1
  fi
  local name="$1"
  cd "$root" || return 1

  # Extract phiếu ID (e.g., P038-foo → P038)
  local phieu_id
  phieu_id=$(echo "$name" | grep -oE '^P[0-9]+')
  if [ -z "$phieu_id" ]; then
    echo "❌ Could not extract phiếu ID from '$name' (expected P<NNN>-<slug>)"
    return 1
  fi

  # 1. Detect phiếu file location (sos-kit: phieu/active/, downstream: docs/ticket/)
  local active_path="" done_path=""
  if [ -f "phieu/active/${name}.md" ]; then
    active_path="phieu/active/${name}.md"
    done_path="phieu/done/${name}.md"
    mkdir -p "phieu/done"
  elif [ -f "docs/ticket/${name}.md" ]; then
    active_path="docs/ticket/${name}.md"
    done_path="docs/ticket/done/${name}.md"
    mkdir -p "docs/ticket/done"
  else
    echo "⚠️  Phiếu file not found at phieu/active/${name}.md or docs/ticket/${name}.md"
    echo "    Skipping strip + move; will still remove worktree + branch."
  fi

  # 2. Strip Debate Log + move active → done
  if [ -n "$active_path" ]; then
    # awk: keep all lines EXCEPT "### Turn N — Worker Challenge" / "### Turn N — Architect Response" subsections.
    # Preserve: header, Context, Task 0, "## Debate Log" header itself, "**Phiếu version:**" line,
    #          "### Final consensus" subsection, Nhiệm vụ, Files, Constraints, Nghiệm thu.
    awk '
      BEGIN { skip = 0 }
      /^### Turn [0-9]+ — Worker Challenge/ { skip = 1; next }
      /^### Turn [0-9]+ — Architect Response/ { skip = 1; next }
      /^### Final consensus/ { skip = 0 }
      /^---$/ { skip = 0 }
      /^## / { skip = 0 }
      skip == 0 { print }
    ' "$active_path" > "${active_path}.stripped"

    if [ -s "${active_path}.stripped" ]; then
      mv "${active_path}.stripped" "$done_path"
      rm -f "$active_path"
      echo "✅ Phiếu moved + Debate Log stripped: $active_path → $done_path"
    else
      rm -f "${active_path}.stripped"
      echo "⚠️  Strip produced empty file — leaving original at $active_path untouched"
    fi
  fi

  # 3. Remove worktree
  if git worktree remove "${wt_parent}/$name" 2>/dev/null; then
    echo "✅ Worktree removed: ${wt_parent}/$name"
  else
    echo "⚠️  Worktree remove failed (already gone? uncommitted changes?) — continuing"
  fi

  # 4. Delete local branch (safe -d only, NOT -D force)
  # Detect branch by listing branches matching the phiếu ID
  local branch
  branch=$(git branch --list "*/${name}" "*/${phieu_id}-*" 2>/dev/null | head -1 | sed 's/^[* ] //' | tr -d ' ')
  if [ -n "$branch" ]; then
    if git branch -d "$branch" 2>/dev/null; then
      echo "✅ Branch deleted (safe): $branch"
    else
      echo "⚠️  Branch '$branch' not fully merged — keeping it (use 'git branch -D $branch' manually if intentional)"
    fi
  else
    echo "ℹ️  No matching local branch found for $phieu_id — skipping branch delete"
  fi

  # 5. Cleanup .backup/<phiếu-id>/
  if [ -d ".backup/${phieu_id}" ]; then
    rm -rf ".backup/${phieu_id}"
    echo "✅ Backup cleaned: .backup/${phieu_id}/"
  fi

  echo ""
  echo "🎉 phieu-done complete for $phieu_id"
}
```

**Lưu ý:**
- Cross-platform: `awk`, `grep -oE`, `sed`, `git branch --list` all work bash 4+ AND zsh 5+. KHÔNG dùng `[[ =~ ]]` extended regex (zsh handles differently from bash in some versions).
- Awk strip logic: state machine `skip=1` khi vào "### Turn N — Worker Challenge" hoặc "### Turn N — Architect Response", `skip=0` khi gặp `### Final consensus`, `^## ` (next H2), hoặc `^---$` (section separator). Preserves header + Tasks + Final consensus.
- Branch detect: `git branch --list "*/${name}" "*/${phieu_id}-*"` matches `feat/P038-phieu-cleanup-and-safety` etc. Strip leading `* ` (current branch marker) + space.
- `git branch -d` (safe-delete) only succeeds if branch fully merged into upstream. If unmerged → echo warn + keep branch (Constraint #3).
- Backwards compat Tầng 2 (no Debate Log): awk script no-op gracefully — không có `### Turn N` line nào → skip stays 0 throughout → output identical to input. Verify by: phiếu không có Debate Log section → run script → file unchanged.
- Edge case: phiếu file đã có "### Turn 1 — Worker Challenge" trong header (rare) → awk could over-strip. Mitigation: pattern requires line START (`^###`) — comment-style "(Turn 1)" prose không match.
- Test sequence (Worker EXECUTE): create test phiếu P-test-cleanup → add fake Debate Log với 2 turns → run `phieu-done P-test-cleanup` → verify (a) file moved active→done, (b) Debate Log gone, (c) Tasks intact, (d) branch deleted, (e) backup gone (if existed), (f) worktree removed.

---

### Task 6: Session-start cleanup nudge

**File:** `scripts/session-start-banner.sh`

**Tìm:** sau Task 1's size warn block (will be added between line 68 và line 70), còn trước line 70 (`echo "🤖 Orchestrator contract..."`).

**Thay bằng / Thêm:** insert AFTER size warn block (Task 1), BEFORE Orchestrator contract block:

```bash

# ─────────────────────────────────────────────────────────────────────
# Phiếu cleanup nudge (P038) — scan active phiếu for approved+merged
# ─────────────────────────────────────────────────────────────────────
NUDGES=""
# Detect phiếu directory: prefer phieu/active/, fall back to docs/ticket/
PHIEU_DIR=""
if [ -d "phieu/active" ]; then
    PHIEU_DIR="phieu/active"
elif [ -d "docs/ticket" ]; then
    PHIEU_DIR="docs/ticket"
fi

if [ -n "$PHIEU_DIR" ]; then
    # Get list of branches merged into main (no gh CLI required)
    MERGED_BRANCHES=$(git branch --merged main 2>/dev/null | sed 's/^[* ] //' | tr -d ' ')

    for phieu_file in "$PHIEU_DIR"/P*.md; do
        [ ! -f "$phieu_file" ] && continue
        # Skip TICKET_TEMPLATE.md if it lives here
        case "$(basename "$phieu_file")" in TICKET_TEMPLATE.md|TEMPLATE.md) continue;; esac

        # Check if phiếu has "Approved by Chủ nhà:" line with non-empty value
        approved_line=$(grep "Approved by Chủ nhà:" "$phieu_file" 2>/dev/null | head -1)
        # Skip if no approval line OR approval value is placeholder (e.g., "[date]" or empty)
        case "$approved_line" in
            *"[date]"*|*"Approved by Chủ nhà: $"*|"") continue;;
        esac

        # Extract phiếu ID
        phieu_id=$(basename "$phieu_file" .md | grep -oE '^P[0-9]+')
        [ -z "$phieu_id" ] && continue

        # Check if a branch matching this phiếu is merged
        if echo "$MERGED_BRANCHES" | grep -qE "/${phieu_id}-"; then
            slug=$(basename "$phieu_file" .md)
            NUDGES="${NUDGES}🧹 Phiếu ${phieu_id} approved + merged. Run: phieu-done ${slug}\n"
        fi
    done
fi

if [ -n "$NUDGES" ]; then
    echo ""
    echo "🧹 Cleanup nudge:"
    printf "    %b" "$NUDGES"
fi
```

**Lưu ý:**
- Cross-platform: `git branch --merged main` works on bash + zsh, all OS. KHÔNG require `gh` CLI (Sếp's machine 8GB RAM constraint — Memory `user_machine_8gb_ram_constraint.md`; gh CLI extra dep; Constraint: graceful degrade).
- Detect both `phieu/active/` (sos-kit) và `docs/ticket/` (downstream) — first-match wins.
- Skip template files (TICKET_TEMPLATE.md, TEMPLATE.md) — không phải actual phiếu.
- Approval detection: grep "Approved by Chủ nhà:" — placeholder `[date]` skipped; only matches when Sếp filled real date. Robust against unfilled placeholder in template.
- Branch match: `MERGED_BRANCHES` line containing `/P038-` (matches `feat/P038-foo`, `fix/P038-bar`, etc.) — flexible across types.
- KHÔNG auto-run phieu-done (Sếp quyết per Constraint #3 + Memory `feedback_skip_approval_gate.md`-style: low-risk surgical OK, but cleanup khác — chứa branch delete, prompt cleanup).
- If no `git` (bare directory) → `git branch --merged` returns empty → `MERGED_BRANCHES=""` → no nudges. Silent fail.
- Test by: approve P038 trong file (fill date) + merge branch + run banner → see nudge → revert.

---

### Task 7: Documentation update

**File 1:** `docs/ORCHESTRATION.md`

**Tìm:** sau line 129 (end of "Failure modes + recovery" table — last row "Worker silently demoted Tầng 1 → Tầng 2"), BEFORE line 131 (`## Concrete example session`).

**Thay bằng / Thêm:** insert NEW section:

```markdown

## Phiếu lifecycle (post-ship cleanup, P038)

After Worker EXECUTE ships and PR merges into main, Sếp runs `phieu-done <P-slug>` to close out the phiếu. This is NOT auto — Sếp's call. Banner script (`scripts/session-start-banner.sh`) nudges via `🧹 Phiếu P<NNN> approved + merged. Run: phieu-done P<NNN>` when both conditions met:
- Phiếu file has `Approved by Chủ nhà: <date>` (non-placeholder)
- Branch `feat|fix|chore|docs|infra/P<NNN>-<slug>` is in `git branch --merged main`

`phieu-done` does (in order):
1. Strip Debate Log "Turn N — Worker Challenge" / "Turn N — Architect Response" subsections from phiếu file (preserves header, Tasks, Final consensus).
2. Move stripped phiếu: `phieu/active/P<NNN>-*.md` → `phieu/done/P<NNN>-*.md` (or `docs/ticket/` ↔ `docs/ticket/done/` for downstream layouts).
3. Remove worktree: `git worktree remove <path>`.
4. Delete local branch: `git branch -d <branch>` (safe-mode only — refuses if unmerged).
5. Cleanup snapshot: `rm -rf .backup/P<NNN>/` (created by Worker Task 0 first-step).

**Why strip Debate Log post-ship:** archived phiếu (in `phieu/done/`) get Read-loaded by Architect when next phiếu references same component. Full Turn-N debate text = pure overhead at that point — decisions already merged, only the consensus + Tasks matter. Strip = ~30-50% file size reduction per multi-turn phiếu.

**Why `git branch -d` not `-D`:** safe-delete refuses unmerged branches → catches "merged via squash but local branch tracking lost" edge case. Worker may surface this as "branch unmerged" warn → Sếp investigates manually.

**Pre-phiếu snapshot (Worker Task 0 first-step):** Worker EXECUTE creates `.backup/P<NNN>/{settings.local.json, .sos-state/, main-head.txt}` BEFORE any code edit. Rollback path if mid-execute hits ❌. `.backup/` is `.gitignore`'d. Auto-cleaned on `phieu-done`.
```

**File 2:** `agents/orchestrator.md` (V3 ACCEPT [O1.1] — condensed to fit ≤90 cap)

**Tìm:** line 56-61 = "## Marker file hygiene" section. Last line of that section = line 61 (`Never leave a stale marker. Marker lives outside .claude/ so YOLO mode does not prompt.`). Line 62 = blank. Line 63 = `## Bulk input handling (P035)`.

**Thay bằng / Thêm:** AFTER line 61, insert exactly **2 lines** (1 H2 header + 1 single-line body) — NO trailing blank, NO leading blank. Existing line 62 (blank) becomes the separator before the new H2; new lines push subsequent content down by exactly 2. Result: 88 + 2 = 90 lines (exactly at CLAUDE.md:106 cap).

**Exact insertion (literal — Worker copy verbatim, NO extra blanks):**

```markdown
## Phiếu cleanup nudge (P038)
Banner shows `🧹 Phiếu P<NNN> approved + merged. Run: phieu-done P<NNN>` per matching phiếu — surface to Sếp, MUST NOT auto-run. Spec: `docs/ORCHESTRATION.md` "Phiếu lifecycle".
```

**Verification (Worker MUST do post-edit):**
- Run `wc -l agents/orchestrator.md` → MUST equal `90`. If 91+ → blank-line crept in (delete it). If 89 → existing blank line was consumed (acceptable, banner spec preserved).
- The single body line is 1 sentence covering 3 contracts: (a) what banner shows, (b) Orchestrator MUST surface (don't suppress), (c) MUST NOT auto-run. Cross-ref to ORCHESTRATION.md gives full spec for anyone needing depth.

**Lưu ý File 2:**
- V2 originally specified ~5 lines (H2 + blank + 2-sentence body + trailing blank). Worker [O1.1] flagged this would push line count 88 → 92, violating CLAUDE.md:106 ≤90 cap. Architect ACCEPT — handbook is runtime system-prompt contract, every line costs tokens per session.
- The condensed 2-line form preserves all 3 behavioral contracts (banner surface, surface-to-Sếp, no auto-run) at ~50% line count. Cross-ref takes the verbose explanation.
- Do NOT add trailing blank line after the body line — line 63 (existing `## Bulk input handling (P035)`) provides the visual separator. Standard markdown renders consecutive `## H2` sections fine without blank line between.
- If existing line 62 is NOT blank (file modified since Architect Read) → Worker insert WITH leading blank to maintain readability, accept 91 lines, document as "exceeded by 1 due to file drift" in Discovery Report.

**File 3:** `docs/BACKLOG.md`

**Tìm:** line 49 (Future waves section) — entry "v2.2 — Debate token optimization":

```markdown
- [ ] **v2.2 — Debate token optimization.** Park until ≥5 multi-turn phiếu deliver real cost-distribution data. Candidates: skip-CHALLENGE for trivial phiếu (needs criteria), Haiku for Architect DRAFT, inline doc snippets in spawn prompt to skip subagent's Read step. Baseline target: 42k → 25k tokens per multi-turn phiếu.
```

**Thay bằng:** mark closed (preserve history), move concept partially to Recently shipped via P038:

```markdown
- [x] **v2.2 — Debate token optimization.** Closed 2026-05-02 — partially shipped via **P038** (per-phiếu Discovery file pattern, Debate Log strip on phieu-done, banner size-warn at 40k threshold). Remaining candidates (skip-CHALLENGE for trivial phiếu — already done via P036 tier routing; Haiku for Architect DRAFT — still parked, awaiting evidence; inline doc snippets — parked).
```

**File 3 — Recently shipped section** (line 84-93): add P038 entry at top:

**Tìm:** line 88 (current top entry "Foundation v2.2 sprint COMPLETE")

**Thay bằng:** insert NEW entry above line 88:

```markdown
- ✅ **P038 / v2.1.7** — (2026-05-02) — Phiếu lifecycle cleanup + safety rails. `phieu-done` extended (strip Debate Log, move active→done, delete branch -d, cleanup .backup/). Banner doc size-warn at 40k + cleanup-nudge for approved+merged phiếu. Worker safety rules (no force-push, no edit memory/settings outside scope, no rm-rf on absolute paths). Pre-phiếu snapshot Task 0 standard. DISCOVERIES.md per-phiếu file pattern (decouple from monolithic). Trigger: Tarot dogfood 2 weeks → 80% week usage Max plan; cost optimization.
```

**File 4:** `CHANGELOG.md`

**Tìm:** line 5 (current top entry "## [v2.1.6] — 2026-04-27")

**Thay bằng:** insert NEW entry above line 5:

```markdown
## [v2.1.7] — 2026-05-02

### Added
- **P038: Phiếu lifecycle cleanup + safety rails + DISCOVERIES decoupling.** Trigger: 2-week Tarot dogfood pushed Max plan to 80% week usage; root cause analysis (`docs/discoveries/P038.md`) identified 6 sub-scopes — token bloat from monolithic DISCOVERIES.md (110k bytes / 28k tokens auto-loaded per Architect spawn), missing phiếu-done cleanup (Debate Log retained, local branches accumulate, no backup cleanup), missing Worker safety rails (force-push / memory edit / settings overwrite all possible), no pre-phiếu rollback point, no doc size warning, no cleanup nudge for approved+merged phiếu.
- **`phieu/phieu.sh`** — `_phieu_done_impl` extended: strips Debate Log Turn N subsections (awk preserve-Final-consensus), moves phiếu file `active/` → `done/` (location-detect: `phieu/active/` for sos-kit, `docs/ticket/` for downstream), `git branch -d` safe-delete (refuses unmerged), removes `.backup/<phiếu-id>/` snapshot. Backwards-compat: phiếu without Debate Log = no-op strip.
- **`scripts/session-start-banner.sh`** — doc size warn (40k byte threshold for CHANGELOG/DISCOVERIES) + phiếu cleanup nudge (scan `phieu/active/` for "Approved by Chủ nhà: <date>" + `git branch --merged main` match → echo `🧹 Phiếu P<NNN> approved + merged. Run: phieu-done P<NNN>`). No `gh` CLI dependency.
- **`agents/worker.md`** — new "Destructive op safety rails" subsection in Hard envelope rules (no force-push, no reset-hard outside phiếu, no edit memory/settings outside scope, no `.sos-state/` deletion, no `rm -rf` on absolute paths) + new top-level "Anti-patterns" section (memory edits, force-push for rebase, pkill -f, mass rm).
- **`phieu/TICKET_TEMPLATE.md`** — new "Pre-phiếu snapshot" subsection in Task 0 (Worker auto first-step: `mkdir .backup/<P>` + cp settings.local.json + cp .sos-state + git rev-parse HEAD). Discovery Report path updated: `docs/DISCOVERIES.md` → `docs/discoveries/P<NNN>.md` per-phiếu + 1-line index entry. **Line 4 dual-path note** (V3 [O1.2] fix Anchor #9 drift): filename now documents both `phieu/active/` (sos-kit) and `docs/ticket/` (downstream).
- **`docs/DISCOVERIES.md`** — converted to index-only (table linking to per-phiếu files). Old monolithic content archived at `docs/archive/DISCOVERIES_pre-2026-05.md`.
- **`docs/ORCHESTRATION.md`** — new "Phiếu lifecycle (post-ship cleanup, P038)" section between "Failure modes" and "Concrete example session".
- **`agents/orchestrator.md`** — new "Phiếu cleanup nudge (P038)" section after "Marker file hygiene" — condensed to 2 lines (V3 [O1.1] fix CLAUDE.md ≤90 cap), file goes 88 → 90 lines exactly at cap.
- **`.gitignore`** — added `.backup/`.

### Files changed
- New: `docs/discoveries/P038.md`, `docs/archive/DISCOVERIES_pre-2026-05.md`
- Modified: `phieu/phieu.sh`, `scripts/session-start-banner.sh`, `agents/worker.md`, `phieu/TICKET_TEMPLATE.md`, `docs/DISCOVERIES.md`, `docs/ORCHESTRATION.md`, `agents/orchestrator.md`, `docs/BACKLOG.md`, `.gitignore`

### Cost baseline shift
- Pre-P038: $4.82 / Tầng 2 phiếu (P109 baseline 2026-05-02). Driver: ~28k token DISCOVERIES.md auto-load + cache write 230k Opus.
- Post-P038 expected: per-phiếu Discovery selective-load → 5-10k token avg (vs 28k flat). Architect cache write reduced proportionally. Real measurement after 5+ phiếu post-ship.
```

**Lưu ý:**
- ORCHESTRATION.md addition = ~25 lines insertion sau Failure modes table. Doesn't touch Tier routing P036 section (line 80-93).
- orchestrator.md addition = **2 lines** (V3 condensed per [O1.1]). Cross-references full spec in ORCHESTRATION.md (handbook stays terse, satisfies CLAUDE.md ≤90 cap).
- BACKLOG.md: 2 edits — close v2.2 wave + add Recently shipped entry. KHÔNG add P038 vào Active sprint (already shipped, post-hoc).
- CHANGELOG.md: full entry with 6 sub-scopes summarized + Files changed list + Cost baseline note + V3 fix notes per O1.1/O1.2.
- Date in entries: 2026-05-02 (from system context).
- Version bump: v2.1.6 → v2.1.7 (matches Recently shipped pattern).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `agents/worker.md` | Task 2: extend Hard envelope rules ("Destructive op safety rails" subsection) + new "Anti-patterns" top-level section |
| `phieu/phieu.sh` | Task 5: extend `_phieu_done_impl` (strip Debate Log via awk, location detect, branch -d, backup delete). **NOTE**: line 89 (`phieu-create` ticket_file path) deliberately UNCHANGED — V3 [O1.2] DEFEND on phieu.sh side. |
| `phieu/TICKET_TEMPLATE.md` | Task 3: add Pre-phiếu snapshot Task 0 subsection + **line 4 dual-path note** (V3 [O1.2] ACCEPT, Anchor #9 drift fix). Task 4: update Discovery Report path |
| `scripts/session-start-banner.sh` | Task 1: doc size warn (40k threshold). Task 6: phiếu cleanup nudge |
| `agents/orchestrator.md` | Task 7: new "Phiếu cleanup nudge (P038)" section after Marker hygiene — **condensed to 2 lines** (V3 [O1.1] ACCEPT, fits ≤90 cap) |
| `docs/ORCHESTRATION.md` | Task 7: new "Phiếu lifecycle (post-ship cleanup, P038)" section after Failure modes |
| `docs/BACKLOG.md` | Task 7: close v2.2 wave entry (mark `[x]` + note partial-shipped via P038); add P038 to Recently shipped |
| `CHANGELOG.md` | Task 7: new v2.1.7 entry |
| `.gitignore` | Task 3: add `.backup/` |
| `docs/DISCOVERIES.md` | Task 4: convert to index file (preserve old → archive) |
| `docs/discoveries/P038.md` | NEW: Worker writes Discovery Report (eat dogfood — P038 itself uses new pattern) |
| `docs/archive/DISCOVERIES_pre-2026-05.md` | NEW: old monolithic content moved here |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `agents/architect.md` | No architect change — confirm worker.md edits don't conflict with architect humility marker rules (P036 frozen) |
| `agents/orchestrator.md` line 40-46 (tier routing) | DO NOT touch — P036 frozen logic |
| `phieu/phieu.sh:89` | DO NOT touch — V3 [O1.2] DEFEND. `phieu-create` is downstream-first command (users invoke in their own repos); sos-kit drafts manually in `phieu/active/`. Changing line 89 = false friction for downstream. Template line 4 dual-path note (Task 3) covers documentation drift. |
| `phieu/VISION_TEMPLATES/*` | No day-1 skeleton impact |
| `phieu/RELAY_PROTOCOL.md` | Orthogonal — covers v1 Web Project mode, not v2.1 lifecycle |
| `phieu/DISCOVERY_PROTOCOL.md` | May need 1-line cross-ref to per-file pattern; Worker Tầng 2 self-decide whether to add |
| `hooks/pre-commit` | Size warn lives in banner, not pre-commit. Verify pre-commit's existing `docs/DISCOVERIES.md` references (line 117, 144) still work after index-conversion (they check existence, not content) |
| `skills/*` | No skill change in this phiếu |
| `bin/sos.sh` | No `.backup/` install rule add (out of scope; user-side gitignore separate) |
| `templates/claude-settings.local.json` | No change (P037 covered marker permission allowlist) |

---

## Luật chơi (Constraints)

1. **NOT re-spec tier routing** — P036 đã ship logic ở `agents/orchestrator.md:40-46`. Phiếu này phụ trợ, không đụng tier.
2. **Banner warnings KHÔNG block** — banner echo only, exit 0. Sếp quyết khi nào trim hoặc cleanup.
3. **`phieu-done` cleanup an toàn** — `git branch -d` (safe-delete, refuses unmerged), KHÔNG `-D` (force). Backup delete only after worktree + branch confirmed clean. Awk strip preserves Final consensus + Tasks.
4. **Per-phiếu Discovery migration** không break Tarot dogfood — old `docs/DISCOVERIES.md` archive intact at `docs/archive/`, index file links back.
5. **Worker safety rules** = extend, không replace existing rules. Backwards compat với phiếu V1-V2 in flight (rules apply prospective, không retro-validate completed phiếu).
6. **Cross-platform**: phieu.sh + banner.sh changes work on bash 4+ AND zsh 5+ (current `phieu.sh:5` contract). KHÔNG GNU-specific stat / sed -i flags. Test cả 2 shells if possible.
7. **Sếp's machine 8GB RAM** — KHÔNG add heavy ops vào banner (banner runs every session start). Doc size check uses `wc -c` (lightweight). Cleanup nudge uses `git branch --merged` (single git invocation, cached). Memory: `user_machine_8gb_ram_constraint.md`.
8. **`gh` CLI optional** — cleanup nudge MUST work without gh (use `git branch --merged main`). Graceful degrade if no `git` repo.
9. **Eat dogfood from day 1** — P038 itself ships using new patterns: Worker writes `docs/discoveries/P038.md` (not append to monolithic), Worker uses `.backup/P038/` snapshot in Task 0, `phieu-done P038-phieu-cleanup-and-safety` after merge cleans everything per new lifecycle.
10. **`agents/orchestrator.md` ≤90 lines (CLAUDE.md:106 cap)** — V3 ACCEPT [O1.1]. New section MUST be exactly 2 lines (H2 header + 1 body line). Worker post-edit verify with `wc -l agents/orchestrator.md` → MUST equal 90 (or 89 if pre-existing blank consumed).

---

## Nghiệm thu

### Automated
- [ ] `bash -n phieu/phieu.sh` syntax clean (no parse error)
- [ ] `bash -n scripts/session-start-banner.sh` syntax clean
- [ ] `shellcheck phieu/phieu.sh scripts/session-start-banner.sh hooks/pre-commit` no critical (SC2086+ level) warns (if shellcheck installed; warn only — don't block)
- [ ] Markdown links valid in updated `docs/ORCHESTRATION.md`, `docs/DISCOVERIES.md`, `agents/orchestrator.md`, `agents/worker.md`, `CHANGELOG.md`, `phieu/TICKET_TEMPLATE.md`
- [ ] **`wc -l agents/orchestrator.md` ≤ 90** (V3 [O1.1] cap verification — Constraint #10)

### Manual Testing
- [ ] **Test phieu-done end-to-end (Tầng 1 phiếu with Debate Log)**:
  1. Create test phiếu file `phieu/active/P-test-cleanup.md` with header + Task 0 + fake Debate Log (Turn 1 Worker Challenge + Turn 1 Architect Response with content) + Final consensus + Tasks
  2. Create matching branch `chore/P-test-cleanup` (worktree at `~/sos-kit-wt/P-test-cleanup`)
  3. Merge to main locally (or just push + merge PR for full e2e)
  4. Run `phieu-done P-test-cleanup`
  5. Verify: (a) `phieu/done/P-test-cleanup.md` exists, (b) Debate Log Turn N subsections gone, (c) Tasks intact, (d) Final consensus intact, (e) worktree removed, (f) branch deleted, (g) `.backup/P-test/` gone if existed
- [ ] **Test backwards compat (Tầng 2 phiếu without Debate Log)**:
  1. Create test phiếu without "### Turn N — Worker Challenge" lines
  2. Run `phieu-done P-test-tang2`
  3. Verify: file moved active→done, content unchanged (no over-strip), branch deleted clean
- [ ] **Test doc size warn**:
  1. Pad `docs/CHANGELOG.md` to >40k bytes (`yes "padding" | head -2000 >> docs/CHANGELOG.md` — revert after)
  2. Run banner script directly: `bash scripts/session-start-banner.sh`
  3. Verify warn line shows: `⚠️  docs/CHANGELOG.md (Xk > 40k threshold) — gọi thợ trim, archive cũ ra docs/archive/`
  4. Revert CHANGELOG.md
- [ ] **Test cleanup nudge**:
  1. Edit `phieu/active/P038-*.md` Final consensus section: set `Approved by Chủ nhà: 2026-05-02`
  2. Ensure feat/P038-* branch is in `git branch --merged main` (mock: `git checkout -b feat/P-test && git checkout main && git merge feat/P-test`)
  3. Run banner → verify `🧹 Phiếu P038 approved + merged. Run: phieu-done P038-...`
  4. Revert
- [ ] **Test no `gh` CLI scenario** (graceful degrade): nudge logic uses only `git branch --merged main` — verify by running banner with `PATH` excluding gh.
- [ ] **Test pre-phiếu snapshot**: in fresh worktree, run snippet from TICKET_TEMPLATE.md Pre-phiếu snapshot block → verify `.backup/P038/` created with `main-head.txt` (other files optional based on existence).
- [ ] **Test Worker safety rule visibility**: read `agents/worker.md`, confirm new sections render correctly + don't break MarkdownToolUse parser (no broken backticks).
- [ ] **Test orchestrator.md cap (V3 [O1.1])**: post-edit, run `wc -l agents/orchestrator.md` → must equal 90 (or 89). If 91+ → blank-line drift, fix before commit.
- [ ] **Test template line 4 dual-path note (V3 [O1.2])**: read `phieu/TICKET_TEMPLATE.md` line 4, confirm it documents both `phieu/active/` and `docs/ticket/` paths with reference to phieu-done location detect.

### Regression
- [ ] Existing `phieu` create flow (`_phieu_impl`) unchanged — counter increment, worktree create, claude launch all still work (verify by `phieu sos-kit chore test-regression-p038`).
- [ ] Tier routing flow (P036) unchanged — Tầng 2 phiếu still skip CHALLENGE (verify by reading `agents/orchestrator.md:40-46` post-edit, confirm bytes identical pre vs post).
- [ ] No new permission prompt added (`templates/claude-settings.local.json` unchanged — verify by diff vs main).
- [ ] `hooks/pre-commit` line 117 + 144 (DISCOVERIES.md references) still pass on new index-only file (verify by staging a phiếu file + DISCOVERIES.md index update + run pre-commit).
- [ ] SessionStart banner Active sprint surface unchanged (Architect Rule 0 still works) — visible in banner output despite new size warn + nudge sections.
- [ ] **`phieu/phieu.sh:89` line 89 unchanged** (V3 [O1.2] DEFEND) — `local ticket_file="docs/ticket/${id}-${slug}.md"` still hardcoded (downstream-first). `phieu-create` semantics preserved for users of sos-kit who run it in their own repos.

### Docs Gate
- [ ] `CHANGELOG.md` — entry P038 (v2.1.7) với 6 sub-scopes summarized + Files changed list + V3 fix notes (O1.1 cap, O1.2 template line 4)
- [ ] `docs/ORCHESTRATION.md` — section "Phiếu lifecycle (post-ship cleanup, P038)" added between Failure modes và Concrete example
- [ ] `docs/BACKLOG.md` — v2.2 wave entry marked `[x]` closed; P038 entry in Recently shipped (top of list)
- [ ] `phieu/TICKET_TEMPLATE.md` — Pre-phiếu snapshot Task 0 subsection + Discovery Report path updated + **line 4 dual-path note** (V3 [O1.2])
- [ ] `agents/worker.md` — Destructive op safety rails subsection + Anti-patterns section
- [ ] `agents/orchestrator.md` — Phiếu cleanup nudge section (**2 lines, V3 [O1.1]**, total file ≤90 lines)
- [ ] `docs/DISCOVERIES.md` — converted to index (old content at `docs/archive/DISCOVERIES_pre-2026-05.md`)

### Discovery Report
- [ ] Per new convention: `docs/discoveries/P038.md` (eat own dogfood — P038 itself uses new pattern)
  - Assumptions in phiếu vs Anchor results (8/9 ✅, 1/9 ⚠️ DRIFT confirmed at Anchor #9, resolved in V3 via Task 3 sub-edit 1B + scope clarification on phieu.sh:89)
  - V3 debate notes: O1.1 ACCEPT (orchestrator.md cap), O1.2 ACCEPT-partial (template line 4 fixed, phieu.sh:89 defended)
  - Edge cases discovered during EXECUTE (e.g., did awk strip over-match? did `git branch --list` glob work cross-shell? did orchestrator.md hit 90 or 89 exactly?)
  - Cross-platform issues (bash vs zsh testing notes)
  - **[needs Worker verify]**: does sos-kit have existing `docs/DISCOVERIES.md`? If not, migration step #2 (move to archive) skip; just create index from scratch.
  - Migration completion: confirm old DISCOVERIES.md archived (or N/A if didn't exist), index file populated, `docs/discoveries/` directory created with `.gitkeep` if empty pre-P038.md
  - Tier escalations: should be "None" (Tầng 1 phiếu, debate-driven, no mid-EXECUTE escalation expected)
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md` linking to `discoveries/P038.md`

---

## Architect notes (for Worker CHALLENGE)

**Confidence levels per anchor**: 8/9 ✅ verified directly via Read. 1/9 ⚠️ (Anchor #9 — drift confirmed, V3 resolved via Task 3 sub-edit 1B). Zero anchors marked `[needs Worker verify]` for code reality (Architect read all relevant files directly).

**One open verify**: Task 4 mentions sos-kit may not have existing `docs/DISCOVERIES.md`. Architect did not Read it (skill: only Read what was requested). Worker confirm via `ls docs/DISCOVERIES.md` before migration step. If absent → skip archive copy, just create new index file.

**No Tầng 2 prescriptions**: phiếu does not specify local var names (e.g., `phieu_id` vs `pid` — Worker decides), CSS / formatting in banner output (only structural format), or test fixture names. These = Tầng 2 self-decide.

**Cross-spawn coordination**: Tasks 1, 5, 6 all touch shell scripts. Worker can edit in one pass; verify each task's Tìm/Thay block independently. Tasks 2, 3, 4, 7 = markdown-only.

**V3 changes summary**:
- [O1.1] ACCEPT — Task 7 File 2 (`agents/orchestrator.md` insert) condensed from ~5 lines to **exactly 2 lines** to fit CLAUDE.md:106 ≤90 cap. New Constraint #10 + Automated nghiệm thu `wc -l ≤ 90`.
- [O1.2] ACCEPT-partial — Task 3 expanded with sub-edit 1B (`phieu/TICKET_TEMPLATE.md` line 4 dual-path note). `phieu.sh:89` deliberately unchanged (DEFEND — phieu-create is downstream-first command). Files KHÔNG sửa table updated to make stance explicit.

**Estimated effort**: half-day (sub-4 hours). Largest task = Task 5 (phieu.sh awk strip + branch detect + location handle) ~1.5 hours including testing. Tasks 1+6 (banner additions) ~30min total. Tasks 2+3+4+7 (markdown) ~1.5 hours total. Manual testing ~30min. V3 micro-edits add ~5 min total (template line 4 single-line replace + orchestrator.md condensed insert).
