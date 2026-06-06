# PHIẾU P050: No-code-on-default-branch git gate (pre-commit)

> **ID format:** `P` + 3 digits. Assigned: **P050** (harvest batch, Active sprint).
> **Filename:** `docs/ticket/P050-no-code-on-default-gate.md`
> **Branch:** `feat/P050-no-code-on-default-gate`

---

> **Loại:** Feature (new pre-commit gate)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — touches `hooks/pre-commit` SECTION + adds a `scripts/*.sh` enforcement gate + reads `.sos-stack.toml` contract → shared-contract + workflow-invariant surface. Per `docs/LAYERS.md` §2-tier: a gate-logic mistake LANs to every commit in every adopting repo. AUTO Tầng 1.) **LOC KHÔNG quyết Tầng.**
> **Ảnh hưởng:** `scripts/no-code-on-default.sh` (new), `hooks/pre-commit` (new SECTION), `CLAUDE.md` (Hook chain + repo-structure script list + DOCS GATE mapping row), `docs/SETUP.md` (hook section).
> **Dependency:** None. (Conceptually pairs with P053 deadlock fix but does not block it.)

---

## Context

### Vấn đề hiện tại

"Branch-before-code" is a JUDGMENT step — the orchestrator (or any agent) is expected to *remember* to cut a feature branch before touching product code. Per memory `enforce_via_mechanism_not_memory`: a step that depends on someone remembering = coin-flip. Live failure: **ket P020** — the orchestrator drafted + coded directly on `main`, forgot to branch; caught by a human, not by any gate.

The cure is to **gate the INVARIANT (no product code committed on the default branch), not the PROCEDURE (when to branch)**. This dissolves the entire "when exactly do I branch" debate: you simply *cannot* commit code on `main`; the gate forces a feature branch. Docs-only commits on `main` stay allowed (kit maintenance, README typo, doctrine edits land on `main` directly — see constraint (e)).

**Why git-level (not a Claude PreToolUse hook):** this is harvest constraint (d) — a pre-commit hook is **agent-agnostic**. It survives Codex / opencode / any non-Claude agent and a human committing by hand. PreToolUse hooks (`orchestrator-guard.sh`) die the moment the agent isn't Claude Code. The whole point of the P049–P052 harvest thread is to push gates down to git so they survive any agent.

**Why `git commit` not `git merge` — and the MERGE_HEAD escape (V2, [O1.1] ACCEPT):** PR merge into `main` is the *intended* path — feature branch → PR → merge. ~~The gate fires on `git commit` only; pre-commit does not run on merge.~~ **CORRECTION (Worker CHALLENGE Turn 1, [O1.1]):** a non-fast-forward `git merge` DOES create a `git commit` → pre-commit DOES fire on merge commits. The V1 claim "fires on git commit, never git merge" was **wrong**. Without a guard, the gate would BLOCK legitimate PR-merges of a feature branch's code into `main` — breaking the exact intended path. **Fix (baked into Task 1):** at the very TOP of `scripts/no-code-on-default.sh`, before any branch detection, detect an in-progress merge via `MERGE_HEAD` and `exit 0`. This lets code enter `main` cleanly via PR merge while still blocking a *direct* `git commit` of code on `main`.

### Giải pháp

Add a new pre-commit SECTION (a standalone `scripts/no-code-on-default.sh` invoked from `hooks/pre-commit`, mirroring how `[4/5]` security-gate and `[5/5]` case-collision are shelled out) that:

0. **(V2, [O1.1])** Detect merge commit (`MERGE_HEAD` exists) → `exit 0` FIRST (PR merge is the intended code path).
1. Resolves the **current branch** and the **default branch**.
2. If current branch ≠ default branch → `exit 0` (not on default → nothing to gate).
3. If on the default branch → inspect the **staged** file list:
   - Filter out `*.md` FIRST (harvest constraint (a) — docs are never product code), THEN
   - grep the *remaining* (filtered) stream for product-code paths derived from `.sos-stack.toml` (constraint (b)).
   - If the filtered stream still contains product code → **BLOCK** (exit 1) unless override marker present.
4. Override marker `.sos-state/allow-code-on-default` present → warn + allow (constraint (d)).

**(V2, [O1.2]) When `.sos-stack.toml` is ABSENT → fall back to a full extension-union and BLOCK** (NOT warn+allow). Greenfield/early commits are the primary harvest target (ket P020 was exactly a pre-stack-file commit on main). See Task 2 + constraint (8) — flagged for Chủ nhà APPROVAL_GATE.

The gate is **shipped to `scripts/` but self-no-ops inside sos-kit's own repo** (constraint (e) — see Task 3).

### Scope
- CHỈ sửa / thêm: `scripts/no-code-on-default.sh` (new), `hooks/pre-commit` (1 new SECTION block), `CLAUDE.md` (3 spots — see Docs Gate), `docs/SETUP.md` (1 subsection), `.sos-state/sos-kit-self` (new self-marker), `.gitignore` (negation for self-marker).
- KHÔNG sửa: `scripts/orchestrator-guard.sh` (READ-ONLY reference — per [P054] FINDING, sos-kit's canonical is already correct; DO NOT "fix" it), `scripts/block-unsafe-merge.sh` (READ-ONLY reference for marker idiom), `.sos-stack.toml.example` schema (do NOT add a new field — see Task 0 anchor #5 + decision below), any parser under `scripts/parsers/`.

---

## Task 0 — Verification Anchors

> Architect (docs-only envelope) cannot grep runtime. Markers: `[verified]` = Architect Read the file; `[needs Worker verify]` = Worker must grep/confirm at EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `orchestrator-guard.sh` exempts `.md` via `*.md) exit 0` at ~line 64 | `grep -n '\*\.md) exit 0' scripts/orchestrator-guard.sh` | ✅ `[verified]` line 64: `case "$NORMALIZED_PATH" in *.md) exit 0 ;; esac` |
| 2 | `orchestrator-guard.sh` product-source pattern = extension+`src/` based | read `scripts/orchestrator-guard.sh:77-80` | ✅ `[verified]` line 78: `*.swift\|*.pbxproj\|src/*\|*/src/*` |
| 3 | `block-unsafe-merge.sh` uses an override-marker idiom + `.sos-state` style markers exist | read `scripts/block-unsafe-merge.sh:56` + `orchestrator-guard.sh:83` | ✅ `[verified]` `block-unsafe-merge.sh:56` uses inline `[security-review-skip:<reason>]`; `.sos-state/worker-active` marker file used at `orchestrator-guard.sh:83`. P050 marker `.sos-state/allow-code-on-default` matches the **file-marker** style (like `worker-active`), not the inline-string style. |
| 4 | `hooks/pre-commit` shells out standalone scripts per SECTION (`[4/5]`, `[5/5]`) and gates on exit code | read `hooks/pre-commit:186-218` | ✅ `[verified]` `[4/5]` runs `bash scripts/security-gate.sh`, `[5/5]` runs `bash scripts/check-case-collision.sh`; both bump `FAIL_COUNT` on nonzero. New section follows this pattern. |
| 5 | `.sos-stack.toml` schema has a field naming the product-code DIR or code-glob | read `templates/.sos-stack.toml.example` | ❌ `[verified]` **NO such field.** Schema = `schema_version`, `[[stack]]` with `type` / `manifest` / `lock_file` / `lock_format` / `parser` ONLY. There is NO "source dir" or "code pattern" field. → Worker MUST derive the code pattern from `type` (decision below), NOT read a nonexistent field, and MUST NOT add a field to the schema. |
| 6 | `type` enum values in `.sos-stack.toml` | `templates/.sos-stack.toml.example:14` comment | ✅ `[verified]` `type` ∈ `node \| python \| rust \| go` (line 13 comment). **Swift NOT yet a type** ([P048] RESEARCH, not shipped) — Worker handles `swift`/unknown via fallback (decision below). |
| 7 | ticket_dir = `docs/ticket` (where this phiếu lives) | `.docs-gate.toml:39` | ✅ `[verified]` `ticket_dir = "docs/ticket"` |
| 8 | `git branch --show-current` returns empty on detached HEAD | runtime git behavior | `[needs Worker verify]` — Architect cannot run git. Worker confirm: `git -c core.hooksPath= checkout --detach HEAD; git branch --show-current` → expect empty string. Drives constraint (c) fail-safe. |
| 9 | default-branch detection from `origin/HEAD` symref | runtime git behavior | `[needs Worker verify]` — `git symbolic-ref --short refs/remotes/origin/HEAD` yields `origin/main`; may be UNSET on a repo never `remote set-head`'d. Worker confirm behavior + the unset fallback (decision below). |
| 10 | sos-kit's own commits land on `main` directly (kit maintenance) | `CLAUDE.md` "Maintainer-only conventions" + git log shows recent `docs(backlog)` commits on `main` | ✅ `[verified]` recent commits (`c2c81d9`, `29f19d2`) are doc/backlog edits on `main`; kit would self-block on any `scripts/*.sh` edit committed to `main` without this opt-out → constraint (e) load-bearing. |
| 11 | **(V2, [O1.1])** a non-FF `git merge` creates a commit → pre-commit fires; `MERGE_HEAD` exists during a merge commit and is absent for a plain `git commit` | runtime git behavior | ✅ `[verified]` (Worker CHALLENGE Turn 1, git semantics): non-FF merge IS a commit → hook fires; `$(git rev-parse --git-dir)/MERGE_HEAD` exists ONLY during an in-progress merge. Guard `[ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ] && exit 0` cleanly distinguishes PR-merge (allow) from direct-commit (gate). |

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V2 (Architect responded to Turn 1 — folded [O1.1] MERGE_HEAD + [O1.2] greenfield-block)

### Turn 1 — Worker Challenge

**Objections (Tầng 1 only):**
- **[O1.1]** V1 claims the gate "fires on git commit, never git merge." WRONG: a non-fast-forward `git merge` IS a `git commit`, so pre-commit fires on merge commits → the gate BLOCKS legitimate PR-merges of code into `main`. Need an explicit merge-commit escape.
- **[O1.2]** V1's `.sos-stack.toml`-absent fallback = warn+allow (silent skip). But that absent-stack window is EXACTLY the harvest target (ket P020: orchestrator coded on main at project start, before any stack file existed). Warn+allow there defeats the whole gate. Worker leans: full extension-union + BLOCK on absent stack.

**Status:** ✅ RESPONDED (see Turn 2)

### Turn 2 — Architect Respond (phiếu V2)

- **[O1.1] → ACCEPT** (mechanical, sound — Worker confirmed via git semantics, `[verified]`). The V1 "never git merge" claim is wrong: a non-FF merge is a commit and fires pre-commit. **Action baked:** Task 1 now adds, at the very TOP of `scripts/no-code-on-default.sh` (before branch detection), `[ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ] && exit 0`. This exits cleanly for merge commits (intended PR-merge path) while still blocking a direct `git commit` of code on main. Also updated: Context "Why git commit not git merge", constraint (6) [rewritten], Task 0 anchor #11 [new, `[verified]`], nghiệm thu (merge test case added). Constraint (6) no longer relies on the unsound `--no-verify`-for-merge note.
- **[O1.2] → RESOLVE as RECOMMENDED, flagged for APPROVAL_GATE.** The greenfield gap (no `.sos-stack.toml`) is the primary failure target; V1's warn+allow there defeats the harvest. **Action baked (RECOMMENDED):** Task 2 now, when `.sos-stack.toml` is ABSENT, falls back to the **full extension-union** `\.(rs|ts|tsx|js|jsx|py|go|swift)$` (union of all type→ext mappings) and **BLOCKS**. Over-block is mitigated by the clean escape `.sos-state/allow-code-on-default` (NOT `--no-verify`, so no `--no-verify`-death). Warn+allow is retained ONLY for the genuinely-unknowable residue (detached HEAD; unresolvable default branch) — those stay fail-open. This changes gate behavior vs V1 → **[APPROVAL_GATE decision — Chủ nhà confirms strictness]**. Alternative (one-line flip): keep V1 warn+allow on absent stack (fail-open everywhere). Updated: Giải pháp, Task 2 Lưu ý [rewritten], constraint (8) [rewritten], nghiệm thu (absent-stack test flips to BLOCK).
- **Also (Worker confirmations folded):**
  - `.gitignore` negation `!.sos-state/sos-kit-self` is now **MANDATORY** (Worker confirmed blanket `.sos-state/` ignore) — promoted from `[needs Worker verify]` conditional to a required step in Task 3 + Files-cần-sửa.
  - The `CLAUDE.md` "Hook chain" **enumerated list does NOT exist** (Worker grep). Task 4 spot #3 (update enumerated chain) = **N/A**. Still REQUIRED: `CLAUDE.md` repo-structure scripts list (add `no-code-on-default.sh`) + DOCS GATE mapping row + `docs/SETUP.md` hook section. Nghiệm thu "Hook chain" item flipped to N/A.

**Status:** ✅ RESPONDED — phiếu bumped to V2. One Chủ-nhà decision outstanding: [O1.2] strictness (block vs warn+allow on absent `.sos-stack.toml`) → APPROVAL_GATE.

### Final consensus
- Phiếu version: V<N>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1: Write `scripts/no-code-on-default.sh`

**File:** `scripts/no-code-on-default.sh` (new, `chmod +x` → 100755)

**Thêm:** a standalone gate. Structure (Worker writes real shell; this is the spec, not verbatim code — adapt naming/quoting to match house style of `orchestrator-guard.sh` + `check-case-collision.sh`):

```sh
#!/usr/bin/env bash
# no-code-on-default.sh — pre-commit gate: BLOCK committing PRODUCT CODE on the default branch.
# Forces a feature branch for code; docs-only (*.md) commits on default stay allowed.
# Agent-agnostic (git-level) — survives non-Claude agents (P049–P052 harvest thread).
# Override: touch .sos-state/allow-code-on-default  (warn + allow; style mirrors .sos-state/worker-active).
# Doctrine: gate the INVARIANT (no code on default), not the PROCEDURE (when to branch).
#           enforce_via_mechanism_not_memory. ket P020 live failure.
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}" 2>/dev/null || exit 0

# --- (V2, [O1.1]) MERGE COMMIT ESCAPE — MUST be first, before any branch logic. ---
#     A non-fast-forward `git merge` creates a commit → pre-commit fires. PR-merge of a
#     feature branch into main is the INTENDED path code enters main; it must NOT be blocked.
#     MERGE_HEAD exists ONLY during an in-progress merge (absent for a plain `git commit`).
[ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ] && exit 0

# --- (e) sos-kit self opt-out: this repo commits maintenance to main directly. ---
[ -f ".sos-state/sos-kit-self" ] && exit 0   # template-only here; see Task 3.

# --- Resolve current branch (c: detached-HEAD fail-safe) ---
CURRENT=$(git branch --show-current 2>/dev/null || echo "")
if [ -z "$CURRENT" ]; then
    # Detached HEAD (rebase / bisect / CI checkout) — "on default branch" is undefined.
    # Fail-safe = WARN + ALLOW (blocking would break rebase/bisect mid-flight).
    echo "⚠️  no-code-on-default: detached HEAD — cannot determine branch, allowing." >&2
    exit 0
fi

# --- Resolve default branch (c: unset origin/HEAD fallback) ---
DEFAULT=$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null | sed 's#^origin/##')
if [ -z "$DEFAULT" ]; then
    # origin/HEAD never set (no `remote set-head`, or no remote). Fall back to the
    # first of main/master that EXISTS as a local branch — do NOT silently assume "main"
    # and do NOT silently pass.
    if   git show-ref --verify --quiet refs/heads/main;   then DEFAULT="main"
    elif git show-ref --verify --quiet refs/heads/master; then DEFAULT="master"
    else
        echo "⚠️  no-code-on-default: cannot resolve default branch (origin/HEAD unset, no main/master) — allowing." >&2
        exit 0
    fi
fi

# Not on default → nothing to gate.
[ "$CURRENT" != "$DEFAULT" ] && exit 0

# --- Override marker (d) ---
if [ -f ".sos-state/allow-code-on-default" ]; then
    echo "⚠️  no-code-on-default: override marker .sos-state/allow-code-on-default present — allowing code on $DEFAULT." >&2
    exit 0
fi

# --- Build product-code pattern from .sos-stack.toml (b) ---
#   Schema has NO code-dir field (Task 0 #5) → derive from `type`. See Task 2.
#   (V2, [O1.2]) If .sos-stack.toml is ABSENT → CODE_PATTERN = full extension-union + BLOCK
#                (NOT warn+allow). Greenfield is the harvest target. See Task 2.
#   Result = a grep -E pattern, e.g. '\.(rs)$|(^|/)src/'.

# --- (a) ORDER IS LOAD-BEARING: filter .md FIRST, THEN grep the FILTERED stream ---
STAGED=$(git diff --cached --name-only --diff-filter=ACM)
FILTERED=$(echo "$STAGED" | grep -vE '\.md$' || true)     # drop docs first
CODE_HITS=$(echo "$FILTERED" | grep -E "$CODE_PATTERN" || true)  # grep the FILTERED var, NOT $STAGED

if [ -n "$CODE_HITS" ]; then
    # BLOCK
    cat >&2 <<EOF
🚫 no-code-on-default: product code staged on default branch ($DEFAULT).

Offending files:
$CODE_HITS

Cut a feature branch first:  git switch -c feat/<slug>
(Docs-only *.md commits on $DEFAULT are allowed.)
Override (kit-maintenance / intentional): touch .sos-state/allow-code-on-default
EOF
    exit 1
fi
exit 0
```

**Lưu ý:**
- **(V2, [O1.1]) The `MERGE_HEAD` escape is the FIRST executable line of gate logic** (after `cd`/`set`). It MUST precede branch detection — otherwise a PR-merge onto `main` is seen as "on default + code staged" and false-blocks. `$(git rev-parse --git-dir)` (not a hardcoded `.git/`) so it works inside worktrees + submodules. `[verified]` Task 0 anchor #11.
- **`CODE_HITS` MUST grep `$FILTERED`, NOT `$STAGED`.** This is the *exact* edge-bug ket's verifier helper hit (caught by Giám sát: "helper grepped original input not the filtered stream"). If you grep `$STAGED` you re-introduce `.md` and over-block docs under a `src/` dir. Filter once, grep the filtered result.
- The `.md` filter must come BEFORE the code grep so that e.g. `src/components/README.md` is dropped before the `src/`-pattern can match it. (Harvest edge-hole: ket's 4 live cases tested docs at ROOT only; missing docs-under-source was the hole.)
- `--diff-filter=ACM` mirrors `hooks/pre-commit:135,168` (Added/Copied/Modified; ignore Deleted/Renamed-source).
- Marker check is a **file** marker (`.sos-state/allow-code-on-default`), consistent with `orchestrator-guard.sh:83`'s `.sos-state/worker-active` — NOT the inline-string idiom of `block-unsafe-merge.sh`. Justification: a commit has no command-line to embed a string in; a file marker is the only workable form at pre-commit.
- **sos-kit self opt-out mechanism — see Task 3 (decision baked: self-marker file).** The `.sos-state/sos-kit-self` check sits right after the MERGE_HEAD escape.
- No `jq` / no external deps — pure shell + `sed`/`grep`, matching `orchestrator-guard.sh` portability note (Windows msys2 bash).

### Task 2: Build the product-code pattern from `.sos-stack.toml` `type` (constraint (b)) — incl. absent-stack BLOCK fallback (V2, [O1.2])

**File:** inside `scripts/no-code-on-default.sh` (the `CODE_PATTERN` construction).

**Tìm:** there is **NO code-dir field** in `.sos-stack.toml` (Task 0 anchor #5 — `[verified]`). Do NOT invent one; do NOT hardcode `^Ket/`.

**Thay bằng / Thêm:** derive `CODE_PATTERN` from the `type` value(s) in `.sos-stack.toml`, mirroring the **extension-based** approach already proven in `orchestrator-guard.sh:78` and `hooks/pre-commit:168`. Read each `[[stack]] type =` line:

```sh
TYPES=$(grep -E '^[[:space:]]*type[[:space:]]*=' .sos-stack.toml 2>/dev/null | sed -E 's/.*=[[:space:]]*"([^"]+)".*/\1/')
```

Map each type → file-extension regex fragment, then OR them together. Baseline mapping (mirror existing patterns; extend as `[[stack]]` types grow):

| `type` | extension fragment |
|---|---|
| `node` | `\.(ts\|tsx\|js\|jsx)$` (matches `hooks/pre-commit:168`) |
| `python` | `\.py$` |
| `rust` | `\.rs$` |
| `go` | `\.go$` |
| `swift` (future, [P048]) | `\.(swift\|pbxproj)$` (matches `orchestrator-guard.sh:78`) — **add now defensively** |
| (unknown / unmapped) | see fallback below |

Then OR a generic `(^|/)src/` arm so a project whose code lives under `src/` is caught regardless of type (mirrors `orchestrator-guard.sh:78` `*/src/*`). Final pattern shape: `\.(rs)$|(^|/)src/` (example for a rust repo).

**The single extension-union constant (define once, reuse for the absent-stack fallback):**

```sh
EXT_UNION='\.(rs|ts|tsx|js|jsx|py|go|swift)$'   # union of ALL type→ext mappings above
```

**Lưu ý:**
- **(V2, [O1.2]) `.sos-stack.toml` ABSENT** (repo never ran `sos init security` — e.g. fresh `sos new`): **DECISION (RECOMMENDED, baked): full extension-union + BLOCK.** Set `CODE_PATTERN="$EXT_UNION|(^|/)src/"` and proceed to the BLOCK path (do NOT `exit 0`). Rationale:
  - **Greenfield/early commits are the PRIMARY harvest target** — ket P020 was an orchestrator coding directly on `main` *before any stack file existed*. V1's warn+allow skipped exactly that window → defeated the harvest.
  - **Over-block is mitigated:** the override `.sos-state/allow-code-on-default` is a clean, reasoned escape that is NOT `--no-verify` — so no `--no-verify`-death (the failure mode of the Rejected vocab-tool).
  - **`[APPROVAL_GATE decision — Chủ nhà confirms strictness]`** — this CHANGES gate behavior from V1 (warn+allow → block). Chủ nhà confirms at the approval gate. **Alternative (one-line flip):** keep V1 warn+allow on absent stack — print `⚠️  no-code-on-default: no .sos-stack.toml — gate skipped` and `exit 0`. Sếp picks one.
- **Unknown/unmapped type** (e.g. a future `java`, present in `.sos-stack.toml` but not in the table): fall back to the generic `(^|/)src/` arm only (don't block on an extension you can't map), and warn once. This is distinct from absent-stack: here a stack IS declared, just an unmapped type.
- **Detached HEAD / unresolvable default branch** stay **warn+allow** (constraint c) — these are genuinely-unknowable residue, NOT the greenfield window. Only the absent-stack case flips to BLOCK.
- Multi-stack (`[[stack]]` repeated): OR all type fragments together — a monorepo with node+rust gates both.
- Do NOT add a "code_dir" / "code_glob" field to `.sos-stack.toml` schema — that's a separate schema-versioning change ([P040] surface, schema_version bump), out of scope here, and would force every adopting repo to regenerate. Extension-from-type is sufficient and zero-migration.

### Task 3: Wire into `hooks/pre-commit` + sos-kit self opt-out (constraint (e))

**File:** `hooks/pre-commit`

**Tìm:** the `[5/5] Case-collision gate` SECTION block (`hooks/pre-commit:203-218`) — the last numbered section before the Summary.

**Thay bằng / Thêm:** insert a new SECTION **before** the Summary block. Renumber: the chain becomes `[1/6]`…`[6/6]` (Worker updates the existing `[N/5]` labels → `[N/6]`. This is mechanical relabel of `hooks/pre-commit:28,78,104,186,207`). New section:

```sh
# ─────────────────────────────────────────────────────────────────────
# 6. No-code-on-default-branch gate (force feature branch for product code)
#    Doctrine: gate INVARIANT (no code on default) not PROCEDURE (when to branch).
#    ket P020 live failure. Agent-agnostic git-level (P049–P052 harvest).
# ─────────────────────────────────────────────────────────────────────
blue "[6/6] No-code-on-default gate"

if [ -f "scripts/no-code-on-default.sh" ]; then
    if bash scripts/no-code-on-default.sh; then
        green "  ✅ No product code on default branch (or on a feature branch)"
    else
        red "  ❌ Product code staged on default branch (detail above)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
else
    echo "  ⏭  scripts/no-code-on-default.sh missing — run scripts/install-hooks.sh after bootstrap"
fi

echo ""
```

**sos-kit self opt-out (constraint (e)) — DECISION (baked): self-marker file `.sos-state/sos-kit-self`.**

Two viable mechanisms were considered (see decisions section). Chosen: a committed self-marker. Worker:
1. Create file `.sos-state/sos-kit-self` (empty, committed — NOT gitignored; it must travel with the kit repo and ONLY the kit repo).
2. The check is already at the TOP of `scripts/no-code-on-default.sh` (right after the MERGE_HEAD escape, before branch logic):
   ```sh
   [ -f ".sos-state/sos-kit-self" ] && exit 0   # sos-kit's own repo commits maintenance to main; gate is template-only here.
   ```
3. **(V2 — MANDATORY, Worker confirmed) `.gitignore` negation.** Worker confirmed `.gitignore` blanket-ignores `.sos-state/`, which would prevent committing the self-marker. Add a negation line so ONLY this one file is committable:
   ```gitignore
   !.sos-state/sos-kit-self
   ```
   (Other `.sos-state/*` markers like `worker-active`/`allow-code-on-default` STAY gitignored — they are runtime-ephemeral; only `sos-kit-self` is committed.) Place the negation AFTER the `.sos-state/` ignore rule (git applies last-match-wins). Worker confirms the negation actually un-ignores via `git check-ignore -v .sos-state/sos-kit-self` (expect: NOT ignored) before commit.

**Lưu ý:**
- The opt-out makes the gate **near-no-op on sos-kit itself** (consistent with how `orchestrator-guard.sh` is near-no-op on the kit, header lines 19-21) while still SHIPPING the gate to every downstream repo via `hooks/pre-commit` + `scripts/`. Downstream repos do NOT get `.sos-state/sos-kit-self`, so the gate is live for them.
- Adopting repos that genuinely want code-on-main (rare) use the **runtime override** `.sos-state/allow-code-on-default` per-commit, NOT the self-marker.

### Task 4: Docs (Tầng 1 — REQUIRED, see Nghiệm thu Docs Gate)

**File:** `CLAUDE.md`

**Thêm (2 spots — spot #3 is N/A, see below):**
1. Repo-structure `scripts/` list (`CLAUDE.md` "Repo structure" block, the `scripts/` subtree comment lines) — add `no-code-on-default.sh # pre-commit — block product code on default branch (force feature branch)`.
2. DOCS GATE Tầng 1 mapping table — the `hooks/pre-commit SECTION add/remove` row already exists (targets `CLAUDE.md "Hook chain" + docs/SETUP.md`); add a sibling row: `| scripts/no-code-on-default.sh add/remove | CLAUDE.md scripts list + docs/SETUP.md hook section | Gate inventory |` (mirror the `scripts/check-*.py` row style).
3. ~~If a "Hook chain" enumerated list exists in `CLAUDE.md`, add the `[6/6]` gate.~~ **(V2 — N/A, Worker grep confirmed):** `CLAUDE.md` has NO enumerated "Hook chain" list (the DOCS GATE mapping table *references* the phrase, but there is no enumerated chain to update). This spot is **N/A**; the `pre-commit` SECTION count lives only in `hooks/pre-commit` itself. Note N/A in Discovery, do nothing in `CLAUDE.md` for this spot.

**File:** `docs/SETUP.md`

**Thêm:** in section `### 5. Install git hooks` (`docs/SETUP.md:137-148`), add one line noting the pre-commit chain now includes a no-code-on-default gate, and that downstream repos get it live while sos-kit self-opts-out via `.sos-state/sos-kit-self`.

**Lưu ý:** Do NOT touch `docs/WORKFLOW_V2.2.md` (FORBIDDEN ad-hoc per CLAUDE.md — doctrine goes through retro). This harvest is a mechanism ship, not a doctrine edit.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `scripts/no-code-on-default.sh` | Task 1+2+3: new gate, MERGE_HEAD escape (V2), type-derived pattern + absent-stack BLOCK union (V2), self opt-out, `.md`-first filter |
| `.sos-state/sos-kit-self` | Task 3: new committed self-marker (kit-only opt-out) |
| `hooks/pre-commit` | Task 3: new `[6/6]` SECTION + relabel `[N/5]`→`[N/6]` |
| `.gitignore` | Task 3 (V2 — MANDATORY): add `!.sos-state/sos-kit-self` negation after the `.sos-state/` ignore rule (Worker confirmed blanket ignore present) |
| `CLAUDE.md` | Task 4: scripts list + DOCS GATE mapping row (Hook-chain spot = N/A) |
| `docs/SETUP.md` | Task 4: hook section note |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/orchestrator-guard.sh` | READ-ONLY — mirror its `.md` exemption (line 64) + ext pattern (line 78). [P054]: canonical already correct, DO NOT edit. |
| `scripts/block-unsafe-merge.sh` | READ-ONLY — marker idiom reference only. |
| `templates/.sos-stack.toml.example` | Schema NOT changed — gate derives pattern from existing `type`, no new field. |
| `scripts/parsers/*.py` | Untouched — gate reads `.sos-stack.toml` directly, no parser invocation. |

---

## Luật chơi (Constraints)

1. **(a) `.md` exempt, filtered FIRST.** Drop `*.md` from the staged list BEFORE the product-code grep, and grep the FILTERED variable — never the original staged list. (ket's verifier-helper edge-bug; Giám sát-caught.)
2. **(b) Pattern from `.sos-stack.toml`, no hardcoded dir.** Derive from `type` (extension mapping mirroring `orchestrator-guard.sh:78`). Multi-stack OR's all types. No `^Ket/`-style per-repo prefix. No new schema field.
3. **(c) Robustness:** detached-HEAD (empty `git branch --show-current`) → warn+allow (do NOT silently pass without a reason; the allow is *reasoned* — branch undefined). Unset `origin/HEAD` → fall back to main/master that exists, else warn+allow. **(V2)** These genuinely-unknowable cases stay fail-open; the absent-stack case does NOT (it blocks — constraint 8).
4. **(d) Override marker** `.sos-state/allow-code-on-default` (file marker, mirrors `worker-active`) → warn+allow.
5. **(e) sos-kit self opt-out** via committed `.sos-state/sos-kit-self` → `exit 0` near top (after MERGE_HEAD escape). Near-no-op on kit, live downstream. Requires `.gitignore` negation (V2, mandatory).
6. **(V2, [O1.1] — REWRITTEN) Merge commits are EXEMPT via MERGE_HEAD escape, NOT via "pre-commit doesn't run on merge".** The V1 assumption was wrong: a non-fast-forward `git merge` IS a `git commit` and DOES fire pre-commit. The gate's FIRST action is `[ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ] && exit 0`, which allows PR-merges of code into `main` (intended path) while a *direct* `git commit` of code on `main` is still gated. No `--no-verify`-for-merge workaround needed. `[verified]` Task 0 anchor #11.
7. **No external deps** (no `jq`). Pure shell + `sed`/`grep`, Windows msys2 compatible.
8. **(V2, [O1.2] — REWRITTEN) `--no-verify` death avoidance + greenfield strictness.** The gate fails-OPEN (warn+allow) ONLY for the genuinely-unknowable residue: detached HEAD, unresolvable default branch, unmapped `type`. **When `.sos-stack.toml` is ABSENT it fails-CLOSED (full extension-union + BLOCK)** because greenfield/early-commit-on-main is the primary harvest target (ket P020). The clean escape `.sos-state/allow-code-on-default` (not `--no-verify`) prevents bypass-death even in block mode. **[APPROVAL_GATE — Chủ nhà confirms this strictness; alternative = keep V1 warn+allow on absent stack.]**

---

## Nghiệm thu

### Automated
- [ ] `bash -n scripts/no-code-on-default.sh` — shell syntax clean.
- [ ] `bash -n hooks/pre-commit` — clean after relabel.
- [ ] `shellcheck scripts/no-code-on-default.sh` if available (warn-only, match house tooling).

### Manual Testing (edge-case matrix — ket shipped a 15-case test; replicate the load-bearing cases)
- [ ] On a feature branch, stage a `.rs`/`.ts` file → gate ALLOWS (not on default).
- [ ] On default branch, stage a product-code file → gate BLOCKS.
- [ ] On default branch, stage `CHANGELOG.md` (docs at ROOT) → gate ALLOWS. *(ket's 4 original cases)*
- [ ] On default branch, stage `src/components/README.md` (docs UNDER source dir) → gate ALLOWS. **(the edge-hole — must pass; proves `.md`-first filter + grep-filtered-stream)**
- [ ] On default branch, stage `src/foo.rs` + `README.md` together → gate BLOCKS (code present after filter).
- [ ] On default branch with `.sos-state/allow-code-on-default` present, stage code → gate ALLOWS + warns.
- [ ] Detached HEAD, stage code → gate ALLOWS + warns (constraint c).
- [ ] **(V2, [O1.1]) Merge a feature branch (with code) into default via non-FF `git merge` → gate ALLOWS (MERGE_HEAD escape).** Verify `$(git rev-parse --git-dir)/MERGE_HEAD` exists during the merge commit; confirm a plain `git commit` of code on default still BLOCKS.
- [ ] **(V2, [O1.2]) Repo with NO `.sos-stack.toml`, on default, stage code → gate BLOCKS** (full extension-union fallback). *(Flipped from V1's warn+allow — pending APPROVAL_GATE; if Sếp picks the alternative, this case reverts to WARNS+ALLOWS.)*
- [ ] **(V2, [O1.2]) Repo with NO `.sos-stack.toml`, on default, stage code, then `touch .sos-state/allow-code-on-default` → gate ALLOWS + warns** (clean escape works even in block-mode).
- [ ] In sos-kit itself (`.sos-state/sos-kit-self` present), stage `scripts/x.sh` on main → gate no-ops (constraint e).

### Regression
- [ ] Existing `[1/5]`…`[5/5]` sections still run + report correctly after `[N/6]` relabel.
- [ ] `scripts/install-hooks.sh` still wires `core.hooksPath` (new script picked up, executable bit set).
- [ ] Committing a docs-only change on `main` in sos-kit still works (the whole point — kit maintenance unblocked).
- [ ] **(V2) `git check-ignore -v .sos-state/sos-kit-self` → NOT ignored** (negation works); `git check-ignore -v .sos-state/worker-active` → still ignored (other markers stay ephemeral).

### Docs Gate (Tầng 1 — REQUIRED)
- [ ] `CLAUDE.md` — `scripts/` repo-structure list includes `no-code-on-default.sh`.
- [ ] `CLAUDE.md` — DOCS GATE Tầng 1 mapping table has the new gate row.
- [ ] `docs/SETUP.md` — `### 5. Install git hooks` notes the new gate + self opt-out.
- [ ] `CHANGELOG.md` — entry for P050.
- [ ] **(V2) `CLAUDE.md` "Hook chain" enumerated list = N/A** (Worker grep: no enumerated chain exists). Noted in Discovery; nothing to update for that spot.

### Discovery Report
- [ ] Write to `docs/discoveries/P050.md`:
  - Anchors #8, #9 (git runtime), #11 (MERGE_HEAD semantics) — CORRECT / WRONG with `file:line` or command output.
  - **(V2)** MERGE_HEAD escape: confirmed a non-FF merge fires pre-commit AND the escape allowed it cleanly? Any worktree/submodule `git-dir` edge?
  - **(V2)** Absent-stack BLOCK fallback: did the extension-union over-block any real file it shouldn't? Did the `allow-code-on-default` escape work in block-mode?
  - `.sos-stack.toml` no-code-field confirmation (#5) — did the type→ext mapping cover all real repos?
  - **(V2)** `.gitignore` negation: did `git check-ignore` confirm `sos-kit-self` committable while other markers stay ignored?
  - sos-kit self opt-out: self-marker vs the rejected "template-not-armed" alternative — did it work cleanly?
  - Edge-case matrix results (esp. the docs-under-source hole + the two V2 cases).
  - Docs updated (list) — note "Hook chain enumerated list" = N/A.
  - Tier escalations (none expected — born Tầng 1).
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md` (newest on top).
