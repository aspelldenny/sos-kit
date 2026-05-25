---
description: Run boundary-check security review on a PR / branch / commit range. Spawns Giám sát subagent which checks 5 generic INV (env var / external service / cross-user / webhook / dep major). Posts ADVISORY comment to PR (silent if clean). KHÔNG block merge.
---

# /security-review

You are the orchestrator (Quản đốc) running the security-review slash command. Execute these steps in order — DO NOT skip, DO NOT improvise. Boundary checks happen INSIDE the Giám sát subagent, NOT in this main session.

**ADVISORY mode reminder:** This command surfaces evidence for Chủ nhà review. It does NOT block merge, does NOT auto-fix, does NOT call `gh pr merge --block`.

## Step 0 — Determine review scope

User invokes one of:
- `/security-review <PR-number>` → review PR #<N> via `gh pr diff <N>`.
- `/security-review <branch>` → review branch vs main: `git diff main..<branch>`.
- `/security-review <commit-range>` → review explicit range: `git diff <range>`.
- `/security-review` (no arg) → review current HEAD vs `git merge-base origin/main HEAD` (default: review current branch's commits).

Resolve the diff source from user's argument. If ambiguous → ask user via 1-question multi-choice (NOT free-form).

## Step 1 — Capture diff content

Capture diff via Bash:
- PR mode: `gh pr diff <N>` (if `gh` available + authenticated)
- Branch mode: `git diff <base>..<head>`
- Range mode: `git diff <range>`

Capture file list: `gh pr diff --name-only <N>` OR `git diff --name-only <base>..<head>`.

Capture PR body (PR mode only, for INV-5 changelog check): `gh pr view <N> --json body --jq .body`.

**If `gh` is not available + PR mode requested** → fall back to branch mode using user-provided merge base, OR tell user to pass `--branch` argument.

**If diff > 100KB** → write to `/tmp/security-review-diff-<id>.txt` and pass path to subagent; otherwise inline in spawn prompt.

## Step 2 — Spawn Giám sát subagent

Use `Task` tool with `subagent_type: "boundary-check"`. Prompt format:

```
You are Giám sát. Run your full workflow (Bước 0 receive context → Bước 1 identify scope per INV → Bước 2 check rubric → Bước 3 compose verdict → Bước 4 emit final report).

Review scope: <PR #N | branch <name> | range <range>>
Diff content: <inline diff OR path to /tmp/security-review-diff-<id>.txt>
Files touched: <list>
PR body (for INV-5 changelog check, optional): <body OR "N/A — not a PR">

Return your final report with `<!-- security-review-start -->` ... `<!-- security-review-end -->` block as specified.
```

Wait for subagent return. Subagent handles 5-INV scan + verdict composition entirely on its own (scoped Bash for cross-INV correlation if needed).

## Step 3 — Extract sentinel block from subagent output

Use `Grep` or string parsing to locate the block between `<!-- security-review-start -->` and `<!-- security-review-end -->` in the subagent's return.

- If verdict line inside block = `APPROVE` AND 0 FLAG → **silent-when-clean rule fires.** Do NOT post comment. Tell user: "Security review complete. APPROVE (0 flags). No comment posted."
- If verdict = `NEEDS_REVIEW` OR ≥1 FLAG → continue to Step 4.

## Step 4 — Post advisory comment (or fallback to local file)

**PR mode (preferred):**
- `gh pr comment <N> --body "<sentinel-block-content>"` — post the full sentinel-wrapped block as a PR comment.
- Verify post: `gh pr view <N> --json comments` should show the new comment.

**Branch/range mode (no PR context):**
- Write sentinel block to `docs/security/last-review.md` (or filename user prefers).
- Tell user the path; user reviews locally.

**If `gh pr comment` fails** (auth issue, no PR for branch yet, etc.):
- Fall back to local file (same path as branch mode).
- Surface error to user with one-line note: "PR comment failed; review at <path>".

## Step 5 — Report to user

Tell user:
- Verdict: `APPROVE` or `NEEDS_REVIEW`.
- Per-INV summary (1-line each): `INV-1 PASS/FLAG`, `INV-2 PASS/FLAG`, `INV-3 PASS/FLAG`, `INV-4 PASS/FLAG`, `INV-5 PASS/FLAG`.
- Where comment posted (PR #N) OR file written (`<path>`).
- ADVISORY reminder: merge gate is NOT affected. Chủ nhà reads the comment and decides.

## Hard rules

- Giám sát is the WORKHORSE. Diff inspection, 5-INV rubric (INV-1 through INV-5), verdict composition all happen INSIDE the subagent (scoped Bash for cross-INV correlation only). Main session ONLY captures diff + spawns + posts comment.
- ADVISORY mode is structural: this slash command does NOT call `gh pr merge --block` or set any blocking status. KHÔNG bao giờ.
- Sentinel markers `<!-- security-review-start -->` / `<!-- security-review-end -->` are LOAD-BEARING. Do not rename, do not duplicate, do not move.
- Silent-when-clean rule (generic anti-approve-fatigue principle): `APPROVE + 0 FLAG → no comment`. Apply this rule HERE in slash command, NOT in Giám sát (Giám sát always returns sentinel block; silent decision is caller's).
- 5 INV are the contract from P042. Adding INV-6+ requires updating BOTH `agents/boundary-check.md` rubric + `templates/INVARIANTS-template.md` user-added section in a new phiếu.
- If Giám sát reports "diff capture failed / no diff content" → relay verbatim, NOT a silent success.
