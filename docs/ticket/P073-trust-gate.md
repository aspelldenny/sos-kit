# PHIẾU P073: Trust gate — auto-exec baseline-diff + hidden-unicode + SECURITY.md (port thanhtra v1.2)

> **ID format:** `P073` — Leg 3 of the open-source-hardening sprint (BACKLOG line 22, Sếp-ratified 2026-06-15).
> **Filename:** `docs/ticket/P073-trust-gate.md`
> **Branch:** `feat/P073-trust-gate`

---

> **Loại:** Feature (security gate)
> **Ưu tiên:** P1
> **Tầng:** 1 — security surface + contract surface (auto-exec config integrity). AUTO Tầng 1 per CLAUDE.md "security boundary touch → AUTO Tầng 1 dù 1 dòng". A malicious modification to an auto-exec surface that ships silently = irreversible trust breach for every `git pull`-ing user → LAN + KHÔNG-đảo.
> **Ảnh hưởng:** new `scripts/trust-gate.sh`, new `.sos-trust-baseline`, new `SECURITY.md`, `hooks/pre-commit` (phase count `[N/7]`→`[N/8]`), `phieu/DISCOVERY_PROTOCOL.md` (escape hidden-unicode), `CLAUDE.md`, `docs/SETUP.md`, `scripts/install-hooks.sh` note, `templates/INVARIANTS-template.md` (optional).
> **Dependency:** P071 (Leg 2 — install checksum). Sequential per BACKLOG line 22 ("Blocked-by P071"). Worker confirms P071 merged before EXECUTE.

---

## Context

### Vấn đề hiện tại
sos-kit is now **PUBLIC + accepts PRs**. The kit INTENTIONALLY ships auto-exec surfaces as its product: git hooks (`hooks/pre-commit`, `hooks/pre-push`), `.mcp.json`, `.claude/settings*.json`, and 11 `scripts/*.sh`. A user who `git pull`s + trusts the folder in Claude Code will auto-exec these. This is the **"Rules File Backdoor"** class (BACKLOG line 246): a malicious PR/commit could (a) modify an auto-exec surface to run a payload, or (b) embed invisible Unicode (BOM / zero-width / bidi) into an instruction/doc file that Claude loads into context = prompt injection to every adopter. Vendors (Cursor, GitHub) classify this as "user responsibility" → the ecosystem won't catch it → the repo must self-defend at the content layer. Tier-1 GitHub hardening (secret scanning, push protection, ruleset, fork-PR approval) is already on; this phiếu adds the **content-integrity** layer.

### Giải pháp
Port the thanhtra v1.2 trust gate as a single new pre-commit phase `scripts/trust-gate.sh`, with **3 Sếp-ratified deviations** from thanhtra:

1. **(a) Baseline-diff (NOT hard-fail).** thanhtra hard-fails ALL auto-exec config; sos-kit ships auto-exec as product, so instead: snapshot each tracked auto-exec surface's content-hash into a committed `.sos-trust-baseline`. The gate FAILS if any tracked surface's current hash ≠ baseline → message points to the diff + the rebaseline command. A reviewed change is accepted by running `scripts/trust-gate.sh rebaseline` AFTER human review → the diff is visible in the PR.
2. **(b) Hidden-unicode gate.** Scan instruction/doc files for hidden Unicode (U+FEFF BOM, zero-width U+200B/200C/200D, etc.) → FAIL. **Sequence-critical:** `phieu/DISCOVERY_PROTOCOL.md:196` already contains a raw U+FEFF (documenting BOM) — the gate would trip on the kit's OWN commit. MUST escape that occurrence BEFORE enabling the gate.
3. **(c) `SECURITY.md`** — honest, concrete threat model for the kit (what hooks do on trust, invariants, trust anchor, how the baseline gate protects auto-exec surfaces).

Recommendation (cheapest-mechanism §0.1): **ONE `trust-gate` phase** running BOTH the baseline-diff check and the unicode check (one phase, one script, two checks) — justified in Luật chơi #8. Hook chain becomes `[1/8]..[8/8]`.

### Scope
- CHỈ tạo/sửa: `scripts/trust-gate.sh` (new), `.sos-trust-baseline` (new), `SECURITY.md` (new), `hooks/pre-commit` (add phase + bump count), `phieu/DISCOVERY_PROTOCOL.md` (escape unicode), `CLAUDE.md` (scripts list + DOCS-GATE row + phase count + baseline note), `docs/SETUP.md` (hook chain + rebaseline workflow), `scripts/install-hooks.sh` (optional note), `templates/INVARIANTS-template.md` (optional INV).
- KHÔNG sửa: any other `scripts/*.sh`, `bin/sos.sh`, `.mcp.json`, `.claude/settings*.json` (these are TRACKED by the gate, not modified by it), `docs/WORKFLOW_V2.X.md` (doctrine — forbidden ad-hoc).
- KHÔNG làm: porting the pattern to `claude-hooks [P012]` — that is a SEPARATE future phiếu (BACKLOG line 249 "copy pattern sang claude-hooks"). Note it in SECURITY.md as future work, do NOT do it here.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Auto-exec surfaces present: `.claude/settings.json`, `.claude/settings.local.json`, `.mcp.json`, `hooks/pre-commit`, `hooks/pre-push`, `bin/sos.sh` exist `[verified]` (Glob) | `git ls-files .claude/settings*.json .mcp.json hooks/pre-commit hooks/pre-push bin/sos.sh` | ✅ all present (Architect Glob confirmed) |
| 2 | 11 `scripts/*.sh` present: check-case-collision, orchestrator-guard, no-code-on-default, block-env-commit, block-env-edit, architect-guard, install-hooks, block-unsafe-merge, idea-smell, session-start-banner, security-gate `[verified]` (Glob) | `git ls-files 'scripts/*.sh' \| wc -l` → expect 11 | ✅ 11 present (Architect Glob confirmed) |
| 3 | `hooks/pre-commit` currently uses phase labels `[1/7]`..`[7/7]` + a `# Runs in order:` header list (7 items) `[verified]` (Read) | `grep -nE '\[[0-9]/[0-9]\]' hooks/pre-commit` | ✅ `[1/7]`..`[7/7]` at lines 32,97,123,207,228,248,268; header list lines 5-12 |
| 4 | Raw U+FEFF BOM char exists in `phieu/DISCOVERY_PROTOCOL.md` ~line 196 (the `[U+FEFF]` between "Added" and "BOM") `[verified]` (Read — Architect saw the char rendered) | `grep -nP '\x{feff}\|\x{200b}\|\x{200c}\|\x{200d}' phieu/DISCOVERY_PROTOCOL.md` | ✅ line 196 (`Added <U+FEFF> BOM to response`). `[needs Worker verify]` whether MORE occurrences exist repo-wide |
| 5 | No `SECURITY.md` or `.sos-trust-baseline` exists yet `[verified]` (Glob — no files found) | `ls SECURITY.md .sos-trust-baseline 2>/dev/null` | ✅ neither exists — both are new |
| 6 | Pre-commit phases wire as `if [ -f "scripts/X.sh" ]; then bash scripts/X.sh; ... else ⏭ missing; fi` and bump `FAIL_COUNT` on non-zero (style template) `[verified]` (Read pre-commit phases 4-7) | `sed -n '243,260p' hooks/pre-commit` | ✅ phases 4-7 follow this exact shape (lines 207-279) |
| 7 | Gate-script style: `set -uo pipefail`, `cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"`, fail-closed on missing dep `[verified]` (Read no-code-on-default.sh + security-gate.sh) | `sed -n '1,12p' scripts/no-code-on-default.sh` | ✅ confirmed (no-code:8-10; security-gate:42-47 fail-closed pattern) |
| 8 | `git ls-files` is the rebaseline enumerator → a NEW auto-exec file must be `git add`ed BEFORE rebaseline or it's missed (thanhtra gotcha) `[unverified]` — design constraint, not a code fact | n/a — encoded as Luật chơi #5 + script comment | ⚠️ design gotcha, encode in script + docs |
| 9 | Cross-platform hash tool: `shasum -a 256` (macOS) vs `sha256sum` (Linux) — same probe issue P071 hit `[needs Worker verify]` | `command -v sha256sum \|\| command -v shasum` | ⏳ TO VERIFY — Worker resolves both, mirror security-gate/install.sh probe pattern |
| 10 | Hidden-unicode scan target set: `CLAUDE.md`, `agents/*.md`, `skills/**/*.md`, `phieu/**/*.md`, `.claude/**` (instruction/doc files Claude loads) `[unverified]` — final glob is Worker's call at EXECUTE | `git ls-files 'agents/*.md' 'skills/**/*.md' 'phieu/**/*.md' CLAUDE.md` | ⚠️ `[needs Worker verify]` exact glob; START from this set, expand if other loaded-doc dirs found |

**❌ count:** 0. **⚠️ / needs-Worker-verify count:** anchors #4 (more occurrences?), #8 (design gotcha), #9 (hash probe), #10 (scan glob).

### Pre-phiếu snapshot (Worker auto first-step)

> **Worker EXECUTE FIRST ACTION** (before any edit): take a rollback point.

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/ — auto-cleaned on phieu-done"
```

---

## Debate Log

> Auto-populated by Worker (CHALLENGE) and Architect (RESPOND). Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge (phiếu V1) — 2026-06-15

**Anchor verification:**
- #1 ✅ `git ls-files` returns 15 files: `.claude/settings.json`, `.mcp.json`, `bin/sos.sh`, `hooks/pre-commit`, `hooks/pre-push`, and 10 `scripts/*.sh` (no `settings.local.json` — see O1.2)
- #2 ⚠️ `git ls-files 'scripts/*.sh' | wc -l` = **10**, not 11. Actual list: `architect-guard, block-env-commit, block-env-edit, block-unsafe-merge, check-case-collision, idea-smell, install-hooks, no-code-on-default, orchestrator-guard, security-gate, session-start-banner` = 11 names but `check-case-collision.sh` vs Architect's list uses `orchestrator-guard.sh` — count discrepancy needs confirmation. After adding `trust-gate.sh` the phiếu says "12 scripts" — but the current count must be confirmed accurate.
- #3 ✅ `grep -nE '\[[0-9]/7\]' hooks/pre-commit` returns 7 hits at lines 32,97,123,207,228,248,268; header list present.
- #4 ✅ `phieu/DISCOVERY_PROTOCOL.md:196` confirmed U+FEFF. Repo-wide scan (CLAUDE.md, agents/, skills/, phieu/, .claude/, docs/) found **exactly 2 occurrences**: `phieu/DISCOVERY_PROTOCOL.md:196` (the known one) AND `docs/ticket/P073-trust-gate.md:45` (the phiếu itself — the Task 0 anchor table cell embeds the raw U+FEFF char to illustrate the hit). The phiếu file is in `docs/ticket/` — see O1.1 below.
- #5 ✅ `SECURITY.md` and `.sos-trust-baseline` absent — confirmed new files.
- #6 ✅ `sed -n '243,260p' hooks/pre-commit` confirms phases 4-7 follow the `if [ -f ... ]; then bash ...; ... FAIL_COUNT` shape.
- #7 ✅ `set -uo pipefail` + `cd "${CLAUDE_PROJECT_DIR:-...}"` + fail-closed on missing dep confirmed in `no-code-on-default.sh` + `security-gate.sh`.
- #8 ⚠️ Design gotcha (acknowledged) — `git add` required before rebaseline; encode in script + docs.
- #9 ✅ Both tools present on this machine: `sha256sum` (Darwin 1.0 at `/sbin/sha256sum`) AND `shasum` (6.02 at `/usr/bin/shasum`). Probe order `sha256sum` → `shasum -a 256` fallback is sound and mirrors existing kit pattern — self-closed via oracle (`command -v sha256sum && command -v shasum`). No portability objection.
- #10 ⚠️ Scan glob confirmed via live grep: primary hits only at `phieu/DISCOVERY_PROTOCOL.md:196` and `docs/ticket/P073-trust-gate.md:45`. `templates/`, `recipes/`, `configs/`, `integrations/`, `bootstrap/` clean. `README.md` clean. Glob is adequate; the phiếu's `[needs Worker verify]` note is answered — expand gate glob to include `docs/ticket/` OR exclude phiếu files from scan (policy decision — see O1.1).

**Objections (Tầng 1 only):**

- [O1.1] **Unicode pre-fix list is incomplete — sequencing blocker.** `docs/ticket/P073-trust-gate.md:45` contains a raw U+FEFF embedded in the Task 0 anchor table cell (the grep showing the match literally contains the character). This file is under `docs/` which is in the gate's scan scope (`docs/ticket/` is not excluded). If the unicode gate scans `docs/` it WILL fire on the phiếu itself — the very commit that enables the gate fails on a file it doesn't own (Luật chơi #2). Three options:
  - A. Exclude `docs/ticket/` (and `docs/discoveries/`) from the unicode scan scope — phiếu/discovery files are transient audit trail, not Claude instruction files; they should not be in the unicode gate scope anyway. **(Worker lean — narrower scope = fewer false positives; phiếu files are not "instruction/doc files Claude loads into context")**
  - B. Include `docs/ticket/` in scope AND add it to Task 1's fix list — escape the U+FEFF in `P073-trust-gate.md:45`. This modifies the phiếu file post-CHALLENGE (Worker is forbidden to modify the phiếu contract; it would need to happen during EXECUTE).
  - C. Include `docs/ticket/` in scope AND rely on Worker to escape it in EXECUTE (as part of Task 1 "fix ALL occurrences"). Riskiest — the phiếu's Task 1 currently says "fix DISCOVERY_PROTOCOL.md" first; a missed occurrence in `docs/ticket/` leaves the gate broken on the enabling commit.
  - Claim: "scope boundary of the unicode gate must be explicitly decided before EXECUTE to avoid gate-blocks-self on day 1."
  - Oracle: `grep -rn 'docs/ticket' phieu/TASK1` — no oracle for a design decision. NONE → must go through Architect.

- [O1.2] **`settings.local.json` is globally gitignored — cannot be tracked in baseline, is an unguarded auto-exec surface.**
  `git check-ignore -v .claude/settings.local.json` → match in `~/.config/git/ignore:1:**/.claude/settings.local.json`. This file is globally gitignored system-wide, so `git ls-files` will NEVER return it — it cannot be in the baseline. The phiếu's Task 0 anchor #1 lists `.claude/settings.local.json` as a surface to baseline, but `git ls-files` returns it only if a user force-adds it (`git add -f`). The phiếu's surface enumeration glob `git ls-files .claude/settings*.json` currently returns ONLY `.claude/settings.json` on this machine (confirmed). Three options:
  - A. Remove `settings.local.json` from the gate's surface list and document in `SECURITY.md` that per-machine local settings are outside the baseline scope (reasonable — local overrides are intentionally per-developer). **(Worker lean — honest scoping, documents the gap)**
  - B. Document in `SECURITY.md` that `settings.local.json` requires `git add -f` by each adopter and rebaseline — fragile, adopter-error-prone.
  - C. Add a NOTE in the gate output: "settings.local.json not tracked by git (globally gitignored on this machine) — verify manually."
  - Claim: "surface count and enumeration accuracy depend on this scoping decision; the phiếu's anchor #1 currently overstates what git ls-files returns."
  - Oracle: `git ls-files .claude/settings*.json` → returns `.claude/settings.json` only (SOUND for the count claim). Self-closeable: confirm the count and flag the gap in this log. Verdict: **self-closed via oracle** — the gap is real but the resolution is: the phiếu's surface count should be 15 (not 16+1 for settings.local.json), and the Architect must choose A/B/C above for documentation stance.

- [O1.3] **Three tracked auto-exec-capable files missing from the 18-surface enumeration:** `install.sh`, `phieu/phieu.sh`, and `templates/setup-dev.sh` are all tracked (`git ls-files` confirms), all executable shell scripts, and all absent from the gate's surface glob. `install.sh` is the primary user-facing installer (`curl | sh` entry point — BACKLOG/P064) — a malicious modification is arguably higher-risk than a scripts/*.sh internal helper. `phieu/phieu.sh` is sourced into adopter shells. `templates/setup-dev.sh` runs on contributor machines. None match `scripts/*.sh`, `hooks/*`, or `bin/sos.sh` globs.
  - A. Expand surface glob to include `install.sh phieu/phieu.sh templates/setup-dev.sh`. **(Worker lean — install.sh is the highest-risk surface; curl|sh attack vector)**
  - B. Declare them out-of-scope in `SECURITY.md` with justification (e.g. "phieu/phieu.sh is sourced not exec'd; templates/ are for adopters to review before use"). Only include if the threat model explicitly excludes them.
  - C. Gate only the top-risk one (`install.sh`) and document the others as advisory.
  - Claim: "the 18-surface count is actually 15 currently tracked (settings.local.json excluded); and 3 additional auto-exec-capable files are absent from the enumeration."
  - Oracle: `git ls-files install.sh phieu/phieu.sh templates/setup-dev.sh` → all 3 present (SOUND for existence). Design choice (A/B/C) = NONE → Architect respond.

**Status:** AWAITING ARCHITECT RESPONSE

O1.1 (unicode scope boundary), O1.2 (settings.local.json — partially self-closed, needs documentation stance), O1.3 (3 unguarded surfaces) are Tầng-1 design decisions that affect EXECUTE scope. Recommend: Architect respond to all three before Worker proceeds to code.

### Final consensus
- Phiếu version: V2 (post-CHALLENGE resolutions)
- Approved by Chủ nhà: 2026-06-15 — code execution authorized
- O1.1 resolution: BROAD scan scope (incl. `docs/ticket/` + `docs/discoveries/`). Phiếu's U+FEFF at :45 fixed by Worker in Task 1 EXECUTE (not excluded from scope).
- O1.2 resolution: `.claude/settings.local.json` EXCLUDED from baseline. Documented in `SECURITY.md` + `docs/SETUP.md`. Surface count = 20.
- O1.3 resolution: `install.sh`, `phieu/phieu.sh`, `templates/setup-dev.sh` ADDED to surface list. Install.sh is the curl|sh attack vector (highest risk).
- Surface list (final): git ls-files globs `scripts/*.sh hooks/* .claude/settings.json .mcp.json bin/sos.sh` + `install.sh phieu/phieu.sh templates/setup-dev.sh` = 20 surfaces. `.claude/settings.local.json` excluded (globally gitignored, per-machine).

---

## Nhiệm vụ

### Task 1: Escape the hidden-unicode in DISCOVERY_PROTOCOL.md — MUST be FIRST

**File:** `phieu/DISCOVERY_PROTOCOL.md`

**Tìm:** line ~196 (Task 0 anchor #4), the sentence currently reading `Added ` + a raw U+FEFF char + ` BOM to response Content-Type charset.` The invisible char sits between "Added" and "BOM".

**Thay bằng:** rewrite so NO raw hidden-unicode char remains — describe it as literal text. Example:
```
- Added a `U+FEFF` BOM to response Content-Type charset. Tested on Excel + Google Sheets + macOS Numbers.
```
(i.e. the byte-order-mark is referenced by its codepoint name `U+FEFF`, not embedded as the actual character.)

**Lưu ý:**
- `[needs Worker verify]` — FIRST run `grep -rnP '\x{feff}|\x{200b}|\x{200c}|\x{200d}|\x{200e}|\x{200f}|\x{e0000}-\x{e007f}' .` (or the gate itself in scan-only mode) across the repo to find ALL occurrences, not just line 196. There may be more than one (BACKLOG says "any others found"). Fix every one BEFORE Task 2.
- This MUST land before the gate is enabled (Task 4 wires it into pre-commit) — else the gate fails on the kit's own commit (Luật chơi #2).
- When fixing, NEVER paste a raw hidden char anywhere (use the codepoint-name escape).

### Task 2: Write `scripts/trust-gate.sh`

**File:** `scripts/trust-gate.sh` (new, +x / 100755)

**Thêm:** a bash gate following the canonical style (Task 0 #7 — mirror `no-code-on-default.sh` header + `security-gate.sh` fail-closed). Structure:

- Header comment: purpose, doctrine ref (BACKLOG line 246 threat model + thanhtra v1.2 oracle), the 3 deviations, the `git ls-files` rebaseline gotcha.
- `set -uo pipefail`; `cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}" 2>/dev/null || exit 0`.
- **Hash-tool resolver** (Task 0 #9): probe `sha256sum` then `shasum -a 256`; if neither runs → fail-closed BLOCK with a loud message (consistent with `security-gate.sh:42-47`). `[needs Worker verify]` exact probe — mirror existing pattern.
- **Subcommand dispatch:** `scripts/trust-gate.sh rebaseline` regenerates `.sos-trust-baseline`; no-arg (or `check`) runs the two checks. (Decide: a `--scan-only` flag for Task 1's repo-wide unicode sweep is convenient — add it.)
- **Surface enumeration** (single source for both check + rebaseline):
  ```bash
  git ls-files .claude/settings.json .claude/settings.local.json .mcp.json \
               hooks/pre-commit hooks/pre-push 'scripts/*.sh' bin/sos.sh
  ```
  `[needs Worker verify]` the final glob produces exactly the 18 files (6 from #1 + 11 scripts + already-counted). Note: `trust-gate.sh` itself is a `scripts/*.sh` → it WILL be in its own baseline (self-tracking, intended — a malicious edit to the gate is exactly what we want to catch).
- **CHECK 1 — baseline-diff:** for each enumerated surface, compute sha256; compare against the matching line in `.sos-trust-baseline`. On any mismatch / new-untracked-surface / removed surface → print the offending file(s) + FAIL with:
  ```
  BLOCKED: trust-gate: auto-exec surface changed vs baseline.
  Changed: <file list>
  Review the diff in this PR, then rebaseline: scripts/trust-gate.sh rebaseline
  (A new auto-exec file must be `git add`ed BEFORE rebaseline — git ls-files won't see it otherwise.)
  ```
- **CHECK 2 — hidden-unicode:** scan the instruction/doc set (Task 0 #10 glob) for hidden codepoints (U+FEFF, U+200B/C/D, U+200E/F bidi, U+2060, U+E0000–U+E007F tag range). On any hit → print `file:line` + FAIL. `[needs Worker verify]` the exact `grep -P` class works on the target platform (`grep -P` unavailable on BSD/macOS default grep → may need `perl -ne` fallback or `rg`; mirror how other scripts handle the `grep -P` portability gap — see P059 finding row #5).
- **Baseline file format** (decide + justify in Discovery): recommend `<sha256>  <relative-path>` per line, sorted by path (so diffs are stable + human-readable in PR). One file, plaintext, committed.
- **Fail-closed:** if the script is invoked but a required tool is missing → BLOCK (exit non-zero), never silent-pass (Luật chơi #6, consistent with kit's other fail-closed gates).

**Lưu ý:**
- This script is itself an auto-exec surface → self-tracked in the baseline (intended).
- Do NOT commit any raw hidden-unicode in this script or any test fixture — use codepoint escapes (`\x{feff}` etc.) only (Luật chơi #3).

### Task 3: Generate the initial `.sos-trust-baseline`

**File:** `.sos-trust-baseline` (new, committed — NOT gitignored)

**Thêm:** run `scripts/trust-gate.sh rebaseline` AFTER Task 1 + Task 2 are complete (so the baseline reflects the post-escape DISCOVERY_PROTOCOL.md and includes trust-gate.sh itself). Commit the generated file.

**Lưu ý:**
- Order matters: rebaseline LAST among the script-producing tasks. If you rebaseline before Task 1's escape lands, the baseline locks in the bad hash but the unicode check still trips — sequence per Luật chơi #2.
- Verify all enumerated surfaces appear (Task 0 #2 count = 11 scripts incl. the new trust-gate.sh = 12 scripts now; +6 non-script = 18 lines). `[needs Worker verify]` the count.

### Task 4: Wire `trust-gate` as pre-commit phase `[8/8]` + bump phase count everywhere

**File:** `hooks/pre-commit`

**Tìm:** the phase-7 block (lines ~263-279, `[7/7] Block .env* commit`).

**Thêm:** a new phase `[8/8]` AFTER phase 7, following the exact shape of phases 4-7 (Task 0 #6):
```bash
# ─────────────────────────────────────────────────────────────────────
# 8. Trust gate (auto-exec baseline-diff + hidden-unicode — P073, port thanhtra v1.2)
#    Doctrine: BACKLOG Rules-File-Backdoor threat model. Baseline-diff (not hard-fail)
#    because the kit ships auto-exec as product; change = FAIL until reviewed + rebaselined.
# ─────────────────────────────────────────────────────────────────────
blue "[8/8] Trust gate (auto-exec baseline + hidden-unicode)"

if [ -f "scripts/trust-gate.sh" ]; then
    if bash scripts/trust-gate.sh; then
        green "  ✅ Auto-exec surfaces match baseline; no hidden unicode"
    else
        red "  ❌ Trust gate failed (detail above)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
else
    echo "  ⏭  scripts/trust-gate.sh missing — run scripts/install-hooks.sh after bootstrap"
fi

echo ""
```

**Bump the phase count `[N/7]` → `[N/8]` EVERYWHERE** (P062 drift precedent, CLAUDE.md DOCS-GATE `hooks/pre-commit` row):
- Every existing label `[1/7]`..`[7/7]` → `[1/8]`..`[7/8]` (lines 32, 97, 123, 207, 228, 248, 268 per Task 0 #3).
- The `# Runs in order:` header list (lines 5-12) → add item `8. Trust gate (auto-exec baseline-diff + hidden-unicode)`.

**Lưu ý:**
- `[verified]` the 7 label locations from Task 0 #3 — but grep to confirm none drifted since draft: `grep -nE '\[[0-9]/7\]' hooks/pre-commit` should return exactly 7, all flip to `/8`.
- Do NOT renumber phases 1-7 content; only the `/7`→`/8` denominator + the new phase 8.

### Task 5: Write `SECURITY.md`

**File:** `SECURITY.md` (new, repo root)

**Thêm:** an honest, concrete threat model for a public security-tooling kit. Sections (decide exact wording — this is content, Tầng-2 latitude on prose, but the invariants are Tầng-1 contract):
- **What this kit auto-executes when you trust the folder** — list the auto-exec surfaces (hooks, `.mcp.json`, `.claude/settings*.json`, scripts) + what each does at a high level (git hooks run at commit; MCP servers are local Rust binaries; no network fetch).
- **Invariants** (the trust contract):
  1. No runtime URL fetch by hooks/scripts (they operate on local files + git only).
  2. Nothing is hidden from the user (hence the hidden-unicode gate).
  3. Install only does declared symlink/copy (point to `scripts/install-hooks.sh` + `bin/sos.sh adopt`).
  4. Release binaries are verified by checksum (P071 — cross-ref Leg 2).
- **Trust anchor** — what the user is trusting (the GitHub repo + maintainer account + the committed `.sos-trust-baseline`), and how the **baseline-diff gate** protects auto-exec surfaces (any change to a tracked surface fails CI/pre-commit until reviewed + rebaselined → the diff is visible in the PR).
- **Reporting** — how to report a vulnerability (private channel / GitHub security advisory).
- **Future work** — note: porting this gate to `claude-hooks` is tracked separately ([P012], BACKLOG line 249); GitHub Tier-1 hardening (secret scanning, push protection, ruleset) is already on.

**Lưu ý:**
- Keep honest + concrete — read by would-be adopters + contributors. No aspirational claims (tag `[intent]` vs `[verified]` if any forward-looking statement, per P063 lesson).
- Do NOT embed any raw hidden-unicode (Luật chơi #3).

### Task 6: Tầng-1 docs updates (security surface = AUTO Tầng 1)

**File:** `CLAUDE.md`
- **Scripts list** (the repo-structure block) — add `scripts/trust-gate.sh` with a one-line description.
- **DOCS GATE Tầng 1 mapping table** — the `hooks/pre-commit SECTION add/remove` row already covers the phase-count bump; add a NEW row: `scripts/trust-gate.sh add/remove OR .sos-trust-baseline` → target `CLAUDE.md scripts list + SECURITY.md + docs/SETUP.md` → Why: "auto-exec integrity surface (P073)". Mirror the `no-code-on-default.sh` / `block-env-commit.sh` row style.
- **A `.sos-trust-baseline` note** — one line in the repo-structure block or scripts section explaining it's the committed content-hash snapshot the trust-gate diffs against; rebaseline via `scripts/trust-gate.sh rebaseline` after review.

**File:** `docs/SETUP.md`
- Hook chain / security pipeline section: bump phase count `[N/7]`→`[N/8]`, add the trust-gate phase, document the **rebaseline workflow** (when a legitimate auto-exec change is made: edit → review → `scripts/trust-gate.sh rebaseline` → commit baseline + change together; new file must be `git add`ed first).
- `[needs Worker verify]` exact section name + current phase-count references in SETUP.md (grep `\[.*/7\]` and any prose "7 phases"/"Phase 7").

**File:** `scripts/install-hooks.sh` (optional, low-risk)
- If it enumerates hooks or scripts, no change needed (it points `core.hooksPath` at `hooks/`). `[needs Worker verify]` — likely N/A; note in Discovery if untouched.

**File:** `templates/INVARIANTS-template.md` (optional)
- IF the trust-gate introduces a generic, reusable INV worth surfacing to downstream repos (e.g. `INV-TRUST-01: auto-exec surfaces match committed baseline`), add it. Decide at EXECUTE; if it's sos-kit-specific only, write "N/A — sos-kit-specific gate, not a generic INV" in Discovery.

**Lưu ý:**
- This is the one-disease-one-mechanism check: the phase-count bump is enforced by the existing DOCS-GATE row (guidance) + this task makes it explicit. Worker writes "Tầng 1 docs updated: <list>" in Discovery.

### Task 7: Fire-test the gate (P057 verify-cò — gate/hook = MUST fire-test in same phiếu)

**No file** — execution + verification step. The phiếu creates a new gate → BẮT BUỘC prove it fires (P057, BACKLOG line 301):
- **Baseline check fires:** modify a tracked surface (e.g. append a comment to a `scripts/*.sh` in a scratch commit), run `bash scripts/trust-gate.sh` → expect FAIL + correct message. Revert.
- **Rebaseline accepts:** make a legit change, `scripts/trust-gate.sh rebaseline`, re-run check → expect PASS.
- **New-file gotcha:** add a new fake auto-exec file WITHOUT `git add`, rebaseline → confirm it's MISSED (documents the gotcha); then `git add` + rebaseline → confirm caught.
- **Unicode check fires:** create a scratch `.md` with a `\x{200b}` (written via printf, never pasted raw), run check → expect FAIL with `file:line`. Delete.
- **Clean state passes:** with everything committed + baselined → `bash scripts/trust-gate.sh` exits 0.
- **Fail-closed:** simulate missing hash tool (PATH trick) → expect BLOCK, not silent-pass.

**Lưu ý:** record results in Discovery Report (P057 DoD).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `phieu/DISCOVERY_PROTOCOL.md` | Task 1: escape raw U+FEFF (+ any other found) — FIRST |
| `scripts/trust-gate.sh` (new) | Task 2: the gate (baseline-diff + unicode + rebaseline subcommand) |
| `.sos-trust-baseline` (new) | Task 3: initial committed hash snapshot |
| `hooks/pre-commit` | Task 4: add phase `[8/8]` + bump all `/7`→`/8` + header list |
| `SECURITY.md` (new) | Task 5: threat model + invariants + trust anchor |
| `CLAUDE.md` | Task 6: scripts list + DOCS-GATE row + baseline note |
| `docs/SETUP.md` | Task 6: hook chain phase count + rebaseline workflow |
| `scripts/install-hooks.sh` | Task 6: optional note `[needs Worker verify]` likely N/A |
| `templates/INVARIANTS-template.md` | Task 6: optional generic INV (decide at EXECUTE) |
| `CHANGELOG.md` | Docs Gate: P073 entry |
| `docs/discoveries/P073.md` (new) | Discovery Report |
| `docs/DISCOVERIES.md` | Discovery index row |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `.mcp.json`, `.claude/settings*.json`, `bin/sos.sh`, other `scripts/*.sh` | These are TRACKED by the gate, not modified. After baseline, committing them unchanged must NOT trip the gate. |
| `docs/WORKFLOW_V2.X.md` | Doctrine — forbidden ad-hoc edit. |

---

## Luật chơi (Constraints)

1. **Baseline-diff, NOT hard-fail.** The kit ships auto-exec as product; the gate gates CHANGES, allows reviewed ones via `rebaseline`. Do NOT port thanhtra's blanket hard-fail.
2. **Sequence hard rule:** escape the DISCOVERY_PROTOCOL.md hidden-unicode (+ any others found repo-wide) BEFORE enabling the unicode gate (Task 1 before Task 4), else the gate fails on the kit's own commit.
3. **Never commit raw hidden-unicode anywhere** — in the gate, fixtures, SECURITY.md, or docs. Use codepoint escapes (`U+FEFF` literal text, `\x{feff}` in regex, `printf` in tests).
4. **Rebaseline requires `git add` of new files first** — `git ls-files` is the enumerator; a new auto-exec file not yet staged is invisible to rebaseline. Encode in script message + SETUP.md.
5. **Fail-closed:** gate/script absent OR required tool (sha256, grep -P/perl) missing → BLOCK, never silent-pass. Consistent with `security-gate.sh:42-47`.
6. **Scope = sos-kit only.** Porting to `claude-hooks [P012]` is a SEPARATE future phiếu — note it in SECURITY.md, do NOT build it.
7. **Cross-platform:** resolve sha256 tool (`sha256sum`/`shasum -a 256`) + handle `grep -P` BSD/macOS gap (P059 row #5) — mirror existing kit probe patterns. The gate must work on macOS + Linux + Git Bash.
8. **One phase, two checks (§0.1 cheapest-mechanism):** the unicode check + baseline-diff check live in ONE `trust-gate` phase / ONE script — they share the "auto-exec/content integrity" disease and a single fail-closed envelope. Justify in Discovery if Worker finds a reason to split.
9. **Phase-count bump everywhere (P062):** `[N/7]`→`[N/8]` in every label + the `# Runs in order` header + any prose in CLAUDE.md + SETUP.md.

---

## Nghiệm thu

### Automated
- [ ] `bash scripts/trust-gate.sh` exits 0 on a clean, baselined tree.
- [ ] Pre-commit runs all 8 phases; `[8/8]` label prints.
- [ ] `grep -rnP '\x{feff}|\x{200b}|\x{200c}|\x{200d}' .` (or gate `--scan-only`) returns 0 hits across instruction/doc files post-Task-1.

### Manual Testing (P057 fire-test — Task 7)
- [ ] Baseline check FAILS on a modified tracked surface, with correct message.
- [ ] `rebaseline` regenerates baseline → check PASSES.
- [ ] New auto-exec file NOT `git add`ed → missed by rebaseline (gotcha documented); after `git add` → caught.
- [ ] Unicode check FAILS on a scratch file with `\x{200b}`, prints `file:line`.
- [ ] Fail-closed: missing hash tool → BLOCK, not silent-pass.

### Regression
- [ ] Phases 1-7 still run + still pass on a normal commit (no behavior change).
- [ ] Committing `.mcp.json` / scripts UNCHANGED after baseline does NOT trip the gate.
- [ ] `scripts/install-hooks.sh` still wires hooks correctly.

### Docs Gate (Tầng 1 — security surface)
- [ ] `CHANGELOG.md` — P073 entry.
- [ ] `CLAUDE.md` — scripts list + DOCS-GATE row + `.sos-trust-baseline` note + phase count.
- [ ] `docs/SETUP.md` — hook chain phase count + rebaseline workflow.
- [ ] `SECURITY.md` — created, threat model complete.
- [ ] Phase count `[N/7]`→`[N/8]` bumped EVERYWHERE (hook labels + header + prose in CLAUDE.md/SETUP.md) — P062 check.

### Discovery Report
- [ ] Write `docs/discoveries/P073.md`:
  - Anchors CORRECT / WRONG (file:line citations) — esp. #4 (extra unicode occurrences?), #9 (hash probe), #10 (final scan glob), surface count.
  - Baseline file format chosen + justification.
  - One-phase-vs-two decision + reason.
  - INVARIANTS-template INV added or N/A.
  - Fire-test results (Task 7).
  - Tầng 1 docs updated: <list>.
  - Tier escalations (None expected).
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
