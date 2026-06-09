# PHIẾU P052: Git-level `.env` block (pre-commit gate)

> **ID format:** `P` + 3 digits. Assigned: **P052** (Két-harvest, Active sprint — promoted 2026-06-08).
> **Filename:** `docs/ticket/P052-git-level-env-block.md`
> **Branch:** `harvest-env-gate` (per BACKLOG Active sprint header).

---

> **Loại:** Feature (new pre-commit gate)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — AUTO Tầng 1: security surface / secret-leak prevention. Adds a `scripts/*.sh` enforcement gate + touches `hooks/pre-commit` SECTION. Per `docs/LAYERS.md` §2-tier + CLAUDE.md "security boundary touch → AUTO Tầng 1": a gate-logic mistake LANs to every commit in every adopting repo AND a miss lets a secret into git history irreversibly. **LOC KHÔNG quyết Tầng** — even if the script is 30 lines.)
> **Ảnh hưởng:** `scripts/block-env-commit.sh` (new), `hooks/pre-commit` (new SECTION `[7/7]` + relabel `[N/6]`→`[N/7]`), `CLAUDE.md` (repo-structure script list + DOCS GATE mapping row), `docs/SETUP.md` (hook section).
> **Dependency:** None. Complement (not dependency) to [P046] PreToolUse `block-env-edit.sh`. P050 (no-code-on-default) already shipped + relabeled chain to `[1/6]…[6/6]` — this phiếu extends to `[7/7]`.

---

## Scope question (P058 — Architect fills BEFORE drafting; "cái gì có thể over-engineer ở đây?")

This gate is one `grep` against staged basenames. The temptations to over-build, and the explicit NO to each:

- **❌ Config TOML for the env-pattern list.** Do NOT add a `[env_block] patterns = [...]` to `.sos-stack.toml` or a new config. The pattern `^\.env($|\.)` is a constant proven in `block-env-edit.sh:39-45` (P046). YAGNI — no repo has asked to tune it; a config knob is ceremony + a migration burden on every adopting repo.
- **❌ SHA-scoping / head-binding.** That's P055 territory (a different gate, `block-unsafe-merge`). Irrelevant here — a staged `.env` is a staged `.env` regardless of SHA.
- **❌ Touching `block-env-edit.sh` (P046, the edit-time sibling) or `block-unsafe-merge.sh`.** One-disease-one-mechanism: this phiếu is **emit-side only** (1 new script + 1 wire). The edit-time guard stays exactly as-is; the two layers complement, they do not merge.
- **❌ Content scanning / entropy / secret-pattern detection inside files.** This gate is filename-only (`.env*`). Detecting secrets *inside* arbitrary files (API keys in `config.json`) is a different, much larger problem (`gitleaks`-class) — explicitly NOT this phiếu.
- **❌ Covering `.envrc` (direnv).** Do NOT extend the regex to catch `.envrc`. `.envrc` is direnv config that is **usually committed on purpose** — it is shared config that points at `.env` to load secrets; the `.env*` files are the ones holding the real secrets. Blocking `.envrc` = a false-positive (blocking a file people deliberately commit) — precisely the failure-mode this gate must NOT cause. Keeping the regex `^\.env($|\.)` **verbatim** from `block-env-edit.sh` is a **feature**: the same "env file" definition at edit-time and at commit-time means the two layers cannot drift. This is scope-discipline (P058 anti-completeness-bias), NOT a technical debt. (See Debate Log [O1.1].)
- **❌ A new INVARIANT entry.** Considered (`templates/INVARIANTS-template.md`). Decision: this is a **commit-time gate**, not a code-review INV that Giám sát checks on a diff. Keep it a gate; do NOT inflate the 5-INV catalog. (Worker may note in Discovery if a future INV cross-ref is warranted, but do not add one now.)

**The minimal core:** filter staged paths → `basename` each → allow `.env.example` → block anything matching `^\.env($|\.)` → exit 1 with a guidance message. Mirror `no-code-on-default.sh` minimalism exactly.

---

## Context

### Vấn đề hiện tại

A `.env*` file committed to git is a **secret leak into history** — irreversible (the secret is in every clone + the reflog even after a later `rm`). **Grounding (n≥1 real incident):** media audit **SEC-SECRET-01** = `.env.docker` was committed into git history. That is the exact failure this gate prevents at commit-time.

[P046] ships `block-env-edit.sh` as a Claude **PreToolUse** Edit/Write guard — it catches Claude *editing* a `.env` file, but it **dies the moment the agent isn't Claude Code** (Codex / opencode / a human `git add`-ing by hand). PreToolUse is Claude-only and edit-time-only. The secret-leak guard must survive any agent → it must live at **git commit-time**, which is agent-agnostic (this is the P049–P052 harvest thread: push gates down to git so they survive non-Claude agents).

The two layers **complement**: PreToolUse `block-env-edit.sh` (fast, in-session, Claude-only) + pre-commit `block-env-commit.sh` (the backstop that survives any agent). Neither replaces the other.

### Giải pháp

Add a new pre-commit SECTION — a standalone `scripts/block-env-commit.sh` invoked from `hooks/pre-commit`, mirroring exactly how `[6/6] no-code-on-default` is shelled out (P050 precedent). The script:

0. **MERGE_HEAD escape FIRST** — a non-fast-forward `git merge` IS a `git commit` and DOES fire pre-commit (the same git semantics P050 [O1.1] established, `[verified]` there). A PR merging a feature branch into `main` is the intended path; do NOT block it. `[ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ] && exit 0` at the very top.
1. Take the **staged** file list (`git diff --cached --name-only --diff-filter=ACM`).
2. For **each path**, take its `basename` (a staged `.env` may live at `config/.env.docker`, not just root — so match on basename, mirroring `block-env-edit.sh`'s `BASE=$(basename …)` logic). Match across the whole path, not just root.
3. **Allowlist** `.env.example` → skip (it carries no secrets, it is the template).
4. Any remaining basename matching `^\.env($|\.)` (catches `.env`, `.env.docker`, `.env.local`, `.env.production`, …) → collect as an offender.
5. If any offender → **BLOCK** (exit 1) with a guidance message. Else exit 0.

**Override marker** `.sos-state/allow-env-commit` (file marker, mirrors `.sos-state/allow-code-on-default` from P050) → warn + allow. This is the clean escape (NOT `--no-verify`) for the rare intentional case — but see constraint (5): the bar for using it is high because a leaked secret is irreversible.

**No `.sos-stack.toml` dependency.** Unlike P050, the env-pattern is a **constant** (`^\.env($|\.)`) — it does NOT vary by stack `type`. There is no absent-stack fallback question here; the gate is fully active in every repo regardless of stack detection. (This is *simpler* than P050 — no type-derivation, no extension-union.)

**sos-kit self behavior — DOES NOT opt out (decision, see Task 0 anchor #5 + decisions section).** Unlike P050's `no-code-on-default` (which self-opts-out because the kit commits maintenance code to `main`), this gate has **no self-opt-out**: sos-kit also must never commit a `.env*`. The gate runs live in the kit too. If sos-kit has no `.env*` tracked (Task 0 #5), it is a near-no-op here by absence-of-trigger, not by an opt-out marker.

### Scope
- CHỈ thêm / sửa: `scripts/block-env-commit.sh` (new), `hooks/pre-commit` (1 new SECTION `[7/7]` + mechanical relabel of the 6 existing `[N/6]` labels → `[N/7]`), `CLAUDE.md` (2 spots — repo-structure scripts list + DOCS GATE mapping row), `docs/SETUP.md` (hook section note), `scripts/install-hooks.sh` (only IF it enumerates scripts — verify Task 0 #4).
- KHÔNG sửa: `scripts/block-env-edit.sh` (READ-ONLY — P046 sibling, pattern source; one-disease-one-mechanism = do NOT touch), `scripts/no-code-on-default.sh` (READ-ONLY — structural template only), `scripts/block-unsafe-merge.sh` (untouched), `.sos-stack.toml` schema (no new field — gate uses a constant pattern), `templates/INVARIANTS-template.md` (no new INV — see Scope question).
- KHÔNG cover `.envrc` (direnv config) — deliberately excluded, not a deferred gap; see Scope question `.envrc` item + Debate Log [O1.1] + constraint (9).

---

## Task 0 — Verification Anchors

> Architect (docs-only envelope) cannot grep runtime. Markers: `[verified]` = Architect Read the file; `[unverified]` = per docs/precedent, Worker re-checks; `[needs Worker verify]` = Worker MUST grep/confirm at EXECUTE.
>
> **NOTE:** Architect did NOT Read any `scripts/*.sh` (shell source = outside docs-only envelope). Anchors #1–#4 are sourced from the Quản đốc's verified spawn brief + the P050 phiếu (a `docs/ticket/` file Architect CAN read). Worker MUST re-grep each before coding (verify skill, Task-0-grep-first).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `block-env-edit.sh` env-match logic: `BASE=$(basename "$FILE_PATH")`, allow `.env.example`, block `echo "$BASE" \| grep -qE '^\.env($\|\.)'` | `grep -n "basename\|\.env\.example\|\^\\\\.env" scripts/block-env-edit.sh` (expect ~line 39-45) | ✅ Confirmed: `basename` line 39, `.env.example` allowlist line 42, regex `grep -qE '^\.env($|\.)'` line 45. Regex reused verbatim. Side-finding: `^\.env($|\.)` does NOT match `.envrc` — see Debate Log O1.1. |
| 2 | `no-code-on-default.sh` is the structural template: `cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"`, MERGE_HEAD escape first, `STAGED=$(git diff --cached --name-only --diff-filter=ACM)`, `.sos-state/<marker>` override, exit 1 = BLOCK | read `scripts/no-code-on-default.sh` | ✅ Confirmed: `cd` line 10, MERGE_HEAD escape line 17, `STAGED` line 100, marker line 51. Skeleton matches phiếu spec. |
| 3 | `hooks/pre-commit` chain is `[1/6]…[6/6]` after P050; sections shell out standalone scripts + bump `FAIL_COUNT` on nonzero; `[6/6]` = no-code-on-default at ~line 229 | `grep -n '\[./6\]\|FAIL_COUNT' hooks/pre-commit` | ✅ Confirmed: `[1/6]` line 31, `[2/6]` line 81, `[3/6]` line 107, `[4/6]` line 188, `[5/6]` line 209, `[6/6]` line 229. Summary (line 245+) uses `$FAIL_COUNT`/`$WARN_COUNT` — NO hardcoded `/6` count to bump. |
| 4 | `scripts/install-hooks.sh` sets `core.hooksPath hooks` and does NOT enumerate per-script (so a new script needs no registration there); `no-code-on-default.sh` required no install-hooks edit | read `scripts/install-hooks.sh` + check P050 Files-cần-sửa (did NOT list install-hooks.sh) | ✅ Confirmed: `git config core.hooksPath hooks` line 33; no per-script enumeration. Task 4 = N/A. |
| 5 | sos-kit repo has NO `.env*` file tracked in git (so the live gate is a near-no-op by absence, AND there is no risk the gate self-blocks kit maintenance) | `git ls-files \| grep -E '(^\|/)\.env($\|\.)'` (expect: empty, OR only `.env.example`) | ✅ Confirmed: `git ls-files | grep -E '\.env'` = empty. No `.env*` tracked. Gate is near-no-op by absence. Safe to ship. |
| 6 | A non-FF `git merge` fires pre-commit; `$(git rev-parse --git-dir)/MERGE_HEAD` exists only during an in-progress merge | runtime git (same as P050 anchor #11) | `[verified]` (via P050 anchor #11, already SHIPPED + dogfood-confirmed): non-FF merge IS a commit → hook fires; MERGE_HEAD guard cleanly distinguishes PR-merge (allow) from direct commit (gate). Reuse the identical guard line. |
| 7 | ticket_dir = `docs/ticket` (where this phiếu lives) | `.docs-gate.toml` `ticket_dir` | `[verified]` — P050 anchor #7 confirmed `ticket_dir = "docs/ticket"`; this phiếu is filed there. |

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

> Schema: 1 turn = 1 Worker Challenge + Architect Response. Cap = 3 turns. Tầng 1 → Worker MUST CHALLENGE.

**Phiếu version:** V2 (Architect responded to Turn 1)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**
- Anchor #1 ✅ `block-env-edit.sh:39-45` confirmed: `BASE=$(basename "$FILE_PATH")` line 39, allowlist `[ "$BASE" = ".env.example" ]` line 42, regex `grep -qE '^\.env($|\.)'` line 45. Exact match to phiếu spec.
- Anchor #2 ✅ `no-code-on-default.sh` skeleton confirmed: `cd "${CLAUDE_PROJECT_DIR:-...}"` line 10, MERGE_HEAD escape line 17 (`$(git rev-parse --git-dir)/MERGE_HEAD`), `STAGED=$(git diff --cached --name-only --diff-filter=ACM)` line 100, `.sos-state/allow-code-on-default` marker line 51.
- Anchor #3 ✅ Chain: `[1/6]` line 31, `[2/6]` line 81, `[3/6]` line 107, `[4/6]` line 188, `[5/6]` line 209, `[6/6]` line 229. Summary block (line 245+) does NOT hardcode `/6` — uses `$FAIL_COUNT`/`$WARN_COUNT` only. Relabel is purely mechanical (grep-replace 6 occurrences of `[./6]` → `[./7]`).
- Anchor #4 ✅ `install-hooks.sh` sets `core.hooksPath hooks` (line 33), does NOT enumerate per-script. New tracked+executable scripts are auto-picked up. Task 4 = N/A confirmed.
- Anchor #5 ✅ `git ls-files | grep -E '\.env'` = empty. No `.env*` tracked in sos-kit. Gate is near-no-op by absence.

**Objections (Tầng 1 only):**

- [O1.1] `.envrc` is NOT caught by the regex `^\.env($|\.)`.
  - Claim: the regex requires `.env` followed by end-of-string or a dot; `.envrc` has `r` immediately after `env` so the pattern exits no-match.
  - Oracle: `echo ".envrc" | grep -qE '^\.env($|\.)'; echo $?` → exits 1. SOUND oracle. Self-closed (gap confirmed by oracle).
  - Impact: `.envrc` (direnv config, can carry `export SECRET=...`) would not be blocked by either this gate or the P046 sibling — both share the blind spot because the phiếu mandates verbatim reuse.
  - This is a **design-scope question for Architect**: the phiếu's explicit rule is "reuse verbatim, if sibling differs the sibling wins." Adding `.envrc` coverage would require either (a) diverging from the sibling (violates one-disease-one-mechanism) or (b) fixing both simultaneously (but phiếu marks `block-env-edit.sh` READ-ONLY). Worker cannot self-decide this.

**Proposed alternatives:**
- A. **Accept the gap now, document in Discovery as a known limitation.** Keep both layers consistent. Reserve `.envrc` fix for a follow-on phiếu that touches both `block-env-edit.sh` and `block-env-commit.sh` together. (Worker recommends — minimal scope, one-disease-one-mechanism rule preserved, `.envrc` in sos-kit is unlikely/non-existent per anchor #5.)
- B. **Extend the regex to `^\.env($|\.|rc)` in this phiếu only** (emit-side only), accepting temporary divergence from `block-env-edit.sh`. Document the divergence in Discovery and open a follow-on for the sibling.

**Status:** ✅ RESPONDED

### Turn 1 — Architect Response (phiếu V2)

- **[O1.1] → ACCEPT Option A (do NOT cover `.envrc`) — Quản đốc-weighed Tầng-1 decision.** Not a deferred gap; a deliberate, principled exclusion:
  1. **`.envrc` (direnv) is usually committed on purpose** — it is shared config that points at `.env` to load secrets. The `.env*` files hold the real secrets, not `.envrc`.
  2. **Blocking `.envrc` = false-positive** — it blocks a file people deliberately commit, which is exactly the failure-mode this gate must NOT introduce.
  3. **Keeping `^\.env($|\.)` verbatim from the sibling is a feature, not debt** — one definition of "env file" at both edit-time (`block-env-edit.sh`) and commit-time (`block-env-commit.sh`) means the two layers cannot drift. Option B's "diverge now, reconcile later" deliberately introduces the drift this design avoids. This is P058 scope-discipline (anti-completeness-bias).
  - **Actions taken in phiếu:** (1) added an explicit `❌ Covering .envrc` item to the Scope question NO-list; (2) added `.envrc` exclusion line to the Scope NO-list + constraint (9); (3) added a fire-test PASS case (stage `.envrc` → exit 0) to lock the "deliberately excluded" decision as a test, not a comment that can rot; (4) added a Discovery note recording the decision + the rule "if `.envrc` coverage is ever needed, fix BOTH `block-env-edit.sh` + `block-env-commit.sh` in one phiếu to keep the two layers in sync."

**Status:** ✅ RESPONDED — phiếu bumped to V2. [O1.1] resolved (ACCEPT Option A). No DEFER. Ready for Worker CHALLENGE re-verify / approval gate.

### Final consensus
- Phiếu version: V<N>
- Approved by Chủ nhà: [date] — code execution may begin

---

## Nhiệm vụ

### Task 1: Write `scripts/block-env-commit.sh`

**File:** `scripts/block-env-commit.sh` (new, `chmod +x` → 100755)

**Thêm:** a standalone gate. Clone the skeleton of `scripts/no-code-on-default.sh` (Task 0 #2 — `cd` / `set` / MERGE_HEAD escape / STAGED / marker / exit-codes), and reuse the env-match regex VERBATIM from `block-env-edit.sh` (Task 0 #1). This is the spec, not verbatim code — Worker adapts naming/quoting to house style:

```sh
#!/usr/bin/env bash
# block-env-commit.sh — pre-commit gate: BLOCK committing any .env* secret file.
# Allows .env.example (the template). Agent-agnostic (git-level) — survives
# non-Claude agents, complements the Claude-only PreToolUse block-env-edit.sh (P046).
# Grounding: media audit SEC-SECRET-01 = .env.docker committed to git history (irreversible leak).
# Override: touch .sos-state/allow-env-commit  (warn + allow; high bar — leak is irreversible).
# NOTE: .envrc (direnv) is DELIBERATELY not covered — it is usually committed on purpose
#       (points at .env to load secrets). Regex stays verbatim with block-env-edit.sh (P046)
#       so the two layers share one env-file definition. See phiếu P052 Debate Log [O1.1].
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}" 2>/dev/null || exit 0

# --- MERGE COMMIT ESCAPE — MUST be first (same semantics as no-code-on-default.sh, [verified] P050 #11). ---
#     A non-FF `git merge` creates a commit → pre-commit fires. PR-merge into main is the intended path.
[ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ] && exit 0

# --- Override marker (file marker, mirrors .sos-state/allow-code-on-default) ---
if [ -f ".sos-state/allow-env-commit" ]; then
    echo "⚠️  block-env-commit: override marker .sos-state/allow-env-commit present — allowing .env* commit. (Secret leaks are IRREVERSIBLE — be sure.)" >&2
    exit 0
fi

# --- Inspect staged files; match on BASENAME (a .env may live at config/.env.docker) ---
STAGED=$(git diff --cached --name-only --diff-filter=ACM)
OFFENDERS=""
for f in $STAGED; do
    base=$(basename "$f")
    [ "$base" = ".env.example" ] && continue          # allowlist the template
    if echo "$base" | grep -qE '^\.env($|\.)'; then    # reuse block-env-edit.sh regex VERBATIM
        OFFENDERS="${OFFENDERS}${f}"$'\n'
    fi
done

if [ -n "$OFFENDERS" ]; then
    cat >&2 <<EOF
🚫 block-env-commit: a .env* secret file is staged — committing it leaks secrets into git history (IRREVERSIBLE).

Offending files:
$OFFENDERS
Fix:
  git restore --staged <file>        # unstage it
  echo '<file>' >> .gitignore        # keep it out of git
  Use .env.example (no real values) as the committed template instead.
Override (RARE, intentional, you accept the leak): touch .sos-state/allow-env-commit
EOF
    exit 1
fi
exit 0
```

**Lưu ý:**
- **Match on `basename`, across the whole path — NOT root-only.** A `.env.docker` can be staged as `config/.env.docker` or `deploy/.env.prod`. Loop the staged list, `basename` each, match. (Mirrors `block-env-edit.sh` which already `basename`s `$FILE_PATH`.) Do NOT anchor the path to repo-root.
- **Reuse the EXACT regex `^\.env($|\.)` from `block-env-edit.sh` (Task 0 #1).** Do NOT reinvent a regex — verify the sibling's regex by grep first, then copy it character-for-character so the two layers stay consistent. If the sibling's regex differs from what's written here, the sibling wins (it's the proven one) — note the discrepancy in Discovery. **Do NOT add `rc` / `.envrc` coverage** — see constraint (9) + Debate Log [O1.1].
- **`.env.example` allowlist is the ONLY exception.** Match it by exact basename equality (`[ "$base" = ".env.example" ]`), same as the sibling — do NOT broaden to a glob like `.env.*.example` unless the sibling does (verify; if it does, mirror it).
- **MERGE_HEAD escape is the first executable line** (after `cd`/`set`), identical to `no-code-on-default.sh` — `$(git rev-parse --git-dir)` not a hardcoded `.git/` (worktree/submodule safe). `[verified]` via P050 #11/#6.
- **NO sos-kit self-opt-out marker.** Deliberately omitted (unlike `no-code-on-default.sh`'s `.sos-state/sos-kit-self`): the kit also must never commit a `.env*`. The gate runs live in the kit. (See decisions section.)
- **No `.sos-stack.toml` read, no type-derivation, no `jq`, no external deps.** Pure shell + `grep`/`basename`. Windows msys2 compatible.
- `--diff-filter=ACM` (Added/Copied/Modified; ignore Deleted/Renamed-source) — mirror `no-code-on-default.sh` + `hooks/pre-commit`.

### Task 2: Wire into `hooks/pre-commit` as SECTION `[7/7]` + relabel

**File:** `hooks/pre-commit`

**Tìm:** the `[6/6] No-code-on-default gate` SECTION block (per Task 0 #3, ~line 229 — the last numbered section before the Summary). Confirm the exact line by grep before editing.

**Thay bằng / Thêm:** insert a new SECTION **after** `[6/6]` and **before** the Summary block. Renumber the chain to `[1/7]`…`[7/7]` — Worker mechanically relabels the 6 existing `[N/6]` labels → `[N/7]` (same kind of relabel P050 did `[N/5]`→`[N/6]`; grep all `[./6]` occurrences first, Task 0 #3). New section (mirror the `[6/6]` block shape exactly):

```sh
# ─────────────────────────────────────────────────────────────────────
# 7. Block .env* secret-file commits (allow .env.example).
#    Agent-agnostic git-level backstop to the Claude-only PreToolUse block-env-edit.sh (P046).
#    Grounding: media audit SEC-SECRET-01 (.env.docker committed → irreversible leak).
# ─────────────────────────────────────────────────────────────────────
blue "[7/7] Block .env* commit"

if [ -f "scripts/block-env-commit.sh" ]; then
    if bash scripts/block-env-commit.sh; then
        green "  ✅ No .env* secret file staged"
    else
        red "  ❌ .env* secret file staged (detail above)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
else
    echo "  ⏭  scripts/block-env-commit.sh missing — run scripts/install-hooks.sh after bootstrap"
fi

echo ""
```

**Lưu ý:**
- Match the EXACT block shape of the existing `[6/6]` section (the `blue`/`if -f`/`bash`/`green`/`red FAIL_COUNT`/`else ⏭`/`echo ""` idiom) — Worker reads the shipped `[6/6]` block and clones its structure so the chain stays uniform. Do NOT invent a different reporting style.
- If the Summary line prints a total section count (e.g. "N/6 checks"), bump it to reflect 7 — Worker greps the Summary block for any hardcoded `/6` and updates it.
- This is purely additive + a mechanical relabel — no existing section's logic changes.

### Task 3: Docs (Tầng 1 — REQUIRED, see Nghiệm thu Docs Gate)

**File:** `CLAUDE.md`

**Thêm (2 spots):**
1. Repo-structure `scripts/` list (the `scripts/` subtree comment block — same place P050 added `no-code-on-default.sh`). Add a line: `block-env-commit.sh   # pre-commit — block .env* secret-file commits (allow .env.example); git-level backstop to P046 PreToolUse guard`.
2. DOCS GATE Tầng 1 mapping table — add a sibling row next to the P050 `no-code-on-default.sh` row: `| scripts/block-env-commit.sh add/remove | CLAUDE.md scripts list + docs/SETUP.md hook section | Gate inventory (P052) |`.
   - **Note (per P050 Discovery):** `CLAUDE.md` has NO enumerated "Hook chain" list — the DOCS GATE table *references* the phrase but there's no chain to renumber there. So there is NO `[7/7]` to add in `CLAUDE.md`. Do nothing for that spot; note N/A in Discovery.

**File:** `docs/SETUP.md`

**Tìm:** section `### 5. Install git hooks` (per the SETUP.md `**Pre-commit chain ([1/6]…[6/6]):**` line, ~line 150).

**Thay bằng / Thêm:**
- Update the chain label `[1/6]…[6/6]` → `[1/7]…[7/7]`.
- Add a short paragraph (mirror the existing no-code-on-default paragraph) noting the chain now includes a **`.env*` commit-block gate** (`scripts/block-env-commit.sh`): blocks staging any `.env*` (allows `.env.example`) so a secret-bearing env file cannot enter git history; agent-agnostic backstop to the Claude-only PreToolUse `block-env-edit.sh` (P046). Override (rare): `touch .sos-state/allow-env-commit`. **Unlike the no-code gate, this one does NOT self-opt-out** — sos-kit also must never commit a `.env*`.

**Lưu ý:** Do NOT touch `docs/WORKFLOW_V2.2.md` (FORBIDDEN ad-hoc edit per CLAUDE.md — doctrine goes through retro). This is a mechanism ship, not a doctrine edit.

### Task 4: (Conditional) register in `scripts/install-hooks.sh`

**File:** `scripts/install-hooks.sh`

**Tìm:** whether install-hooks enumerates per-script (Task 0 #4). P050 did NOT edit this file → strong signal `core.hooksPath` mode auto-picks-up new scripts.

**Thay bằng / Thêm:** **Only if** install-hooks explicitly lists scripts to copy/chmod (it likely does NOT) → add `block-env-commit.sh` to that list. If it just sets `core.hooksPath hooks` and the script is already executable + tracked → NOTHING to do here; record N/A in Discovery. Do NOT add a registration step speculatively.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `scripts/block-env-commit.sh` | Task 1: new gate — MERGE_HEAD escape, override marker `.sos-state/allow-env-commit`, basename match, `.env.example` allowlist, reuse `^\.env($\|\.)` regex |
| `hooks/pre-commit` | Task 2: new `[7/7]` SECTION + mechanical relabel `[N/6]`→`[N/7]` (+ Summary count if hardcoded) |
| `CLAUDE.md` | Task 3: scripts-list line + DOCS GATE mapping row (Hook-chain enumerated spot = N/A per P050) |
| `docs/SETUP.md` | Task 3: chain label `[1/6]`→`[1/7]` + new gate paragraph |
| `scripts/install-hooks.sh` | Task 4 (CONDITIONAL — only if it enumerates scripts; likely N/A) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/block-env-edit.sh` | READ-ONLY — P046 sibling. Source the EXACT `^\.env($\|\.)` regex + `.env.example` allowlist + `basename` idiom from here (Task 0 #1). One-disease-one-mechanism: do NOT edit it. (Including: do NOT add `.envrc` coverage to it — see [O1.1].) |
| `scripts/no-code-on-default.sh` | READ-ONLY — structural template (Task 0 #2). Clone its skeleton (cd/MERGE_HEAD/STAGED/marker/exit) + its `[6/6]` hook-block shape. Do NOT edit it. |
| `scripts/block-unsafe-merge.sh` | Untouched — unrelated gate (merge sentinel). |
| `.sos-stack.toml` schema | NOT changed — env-pattern is a constant, no stack-type dependence, no new field. |
| `templates/INVARIANTS-template.md` | NOT changed — this is a gate, not a new 5-INV entry (see Scope question). |

---

## Luật chơi (Constraints)

1. **`.env.example` is the ONLY allowlisted file** — matched by exact basename equality, mirroring `block-env-edit.sh`. Everything matching `^\.env($|\.)` is blocked.
2. **Match on `basename` across the whole staged path**, not root-only — `config/.env.docker` must be caught (the SEC-SECRET-01 incident file was a `.env.docker`).
3. **Reuse the proven regex from `block-env-edit.sh` verbatim** (Task 0 #1) — do NOT author a new regex; if the sibling's differs from this phiếu, the sibling wins, note in Discovery.
4. **MERGE_HEAD escape first** — identical to `no-code-on-default.sh` (`[verified]` P050 #6/#11). PR-merge into main must not be blocked.
5. **Override `.sos-state/allow-env-commit` is a high-bar escape** (file marker, NOT `--no-verify` — so no `--no-verify`-death). The message states the leak is IRREVERSIBLE. It exists for the genuinely-intentional rare case only.
6. **NO sos-kit self-opt-out** (deliberate divergence from P050) — the kit also must never commit a `.env*`; the gate runs live in the kit. (Task 0 #5 verifies the kit has no `.env*` tracked → near-no-op by absence, not by an opt-out marker.)
7. **No external deps** (no `jq`, no `.sos-stack.toml` read) — pure shell + `grep`/`basename`, Windows msys2 compatible.
8. **One-disease-one-mechanism: emit-side only.** This phiếu adds 1 script + 1 wire. It does NOT touch `block-env-edit.sh` (the edit-time sibling) — the two complement, they do not merge.
9. **`.envrc` is DELIBERATELY excluded** (Debate Log [O1.1], ACCEPT Option A) — direnv config is usually committed on purpose; blocking it = false-positive. The regex `^\.env($|\.)` stays verbatim with the P046 sibling so both layers share one env-file definition (feature, not drift). Do NOT add `rc`/`.envrc` coverage. If `.envrc` coverage is ever genuinely needed, it must be a future phiếu that fixes BOTH `block-env-edit.sh` + `block-env-commit.sh` in one shot (keep the two layers in sync).

---

## Nghiệm thu

### Automated
- [ ] `bash -n scripts/block-env-commit.sh` — shell syntax clean.
- [ ] `bash -n hooks/pre-commit` — clean after relabel.
- [ ] `shellcheck scripts/block-env-commit.sh` if available (warn-only, match house tooling).

### Manual Testing — FIRE-TEST "kéo cò xem nổ" (P057 — MANDATORY, RUN it, don't just write the script)

> Build a throwaway test (mirror P050's 17/17 discrimination matrix). Each case = stage file(s) → run `bash scripts/block-env-commit.sh` (or attempt a real `git commit`) → assert exit code. **Both a BLOCK case and a PASS case must actually fire** — proving the gate discriminates, not just that it exists.

- [ ] **NỔ (block):** stage `.env.fake` (a throwaway, no real secret) on any branch → `git commit` is BLOCKED, exit 1, guidance message shown. *(The core "kéo cò xem nổ".)*
- [ ] **NỔ (block):** stage `config/.env.docker` (non-root path) → BLOCKED, exit 1. *(Proves basename-across-path match — the SEC-SECRET-01 shape.)*
- [ ] **NỔ (block):** stage `.env.local` + `.env.production` together → BLOCKED, both listed as offenders.
- [ ] **KHÔNG nổ (pass):** stage `.env.example` alone → PASSES, exit 0. *(Allowlist works.)*
- [ ] **KHÔNG nổ (pass):** stage `.envrc` (direnv config) → PASSES, exit 0. *(Locks the [O1.1] "deliberately excluded" decision as a test — `.envrc` is NOT a secret file, must NOT be a false-positive. If this case ever flips to BLOCK, the scope decision was silently violated.)*
- [ ] **KHÔNG nổ (pass):** stage a normal file (`README.md`, `src/foo.ts`) → PASSES, exit 0. *(No false-positive on non-env files.)*
- [ ] **Discrimination (pass+block together):** stage `.env.example` + `.env.docker` together → BLOCKED (the real `.env.docker` is caught even though `.env.example` is present; allowlist does not "rescue" the sibling offender).
- [ ] **Override:** stage `.env.fake`, `touch .sos-state/allow-env-commit` → PASSES + warns. Then `rm .sos-state/allow-env-commit`, re-stage → BLOCKS again. *(Escape works AND is not sticky.)*
- [ ] **Merge escape:** merge a feature branch into default via non-FF `git merge` while a `.env*` exists in the tree (already committed earlier) → gate ALLOWS (MERGE_HEAD escape); confirm a fresh direct `git commit` staging a NEW `.env*` still BLOCKS.

### Regression
- [ ] Existing `[1/6]`…`[6/6]` sections still run + report correctly after the `[N/7]` relabel (no section dropped, no label collision).
- [ ] Summary line (if it prints a count) reflects 7 sections.
- [ ] `scripts/install-hooks.sh` still wires `core.hooksPath` and the new script is picked up + executable.
- [ ] Committing a normal change (no `.env*` staged) in sos-kit still passes the full chain — gate is a no-op by absence (Task 0 #5).
- [ ] [P046] `block-env-edit.sh` PreToolUse guard still behaves identically (untouched — confirm no regression by inspection only; it's READ-ONLY here).

### Docs Gate (Tầng 1 — REQUIRED)
- [ ] `CLAUDE.md` — `scripts/` repo-structure list includes `block-env-commit.sh`.
- [ ] `CLAUDE.md` — DOCS GATE Tầng 1 mapping table has the new gate row.
- [ ] `docs/SETUP.md` — `### 5. Install git hooks` chain label updated to `[1/7]…[7/7]` + new gate paragraph (incl. the "does NOT self-opt-out" note).
- [ ] `CHANGELOG.md` — entry for P052.
- [ ] `CLAUDE.md` "Hook chain" enumerated list = **N/A** (no enumerated chain exists per P050 Discovery) — note in Discovery, nothing to update.

### Discovery Report
- [ ] Write to `docs/discoveries/P052.md`:
  - Anchors #1 (exact `block-env-edit.sh` regex — confirm reused verbatim, cite `file:line`), #2 (no-code-on-default skeleton clone points), #3 (exact `hooks/pre-commit` line numbers + the 6 labels relabeled), #4 (install-hooks enumerate? N/A or edited), #5 (**kit `.env*` absence — git ls-files output**) — CORRECT / WRONG with citations.
  - Fire-test matrix results — which cases fired NỔ / KHÔNG-nổ, exit codes observed (the actual "kéo cò" evidence, P057). **Include the `.envrc` PASS case (exit 0) explicitly** — it is the lock on the [O1.1] decision.
  - **[O1.1] resolved:** `.envrc` deliberately excluded (NOT a deferred gap) — false-positive risk (direnv config is committed on purpose) + cross-layer consistency (regex stays verbatim with `block-env-edit.sh`). If `.envrc` coverage is ever needed, fix BOTH `block-env-edit.sh` + `block-env-commit.sh` in one phiếu (keep the two layers in sync).
  - Did `block-env-edit.sh`'s regex match this phiếu's `^\.env($|\.)` exactly? Any drift between the two layers found?
  - Override marker: confirmed not-sticky (block returns after `rm`)?
  - install-hooks.sh: N/A or edited (Task 4)?
  - Docs updated (list) — note "Hook chain enumerated list" = N/A.
  - Tier escalations (none expected — born Tầng 1).
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md` (newest on top).
