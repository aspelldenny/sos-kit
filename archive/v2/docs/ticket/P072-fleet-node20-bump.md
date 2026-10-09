# PHIẾU P072: Fleet Node20-action bump (10 repos, sequential, rc-verified)

> **Loại:** Hotfix (deadline-forced CI bump)
> **Ưu tiên:** P0 — GitHub deprecates Node20 action runtime 16/06/2026; after deadline re-releases hard-fail.
> **Tầng:** 1 — CI surface, fleet-wide blast radius (one bad push × 10 repos = chùm toác). Per IG-09 mitigated by SEQUENTIAL per-repo.
> **Ảnh hưởng:** `.github/workflows/release.yml` across 10 repos (doctor · claude-hooks · docs-gate · ship · advisory-inbox · inv-gate · guard · vps · doc-rotate · advisory-cron)
> **Dependency:** None (P009 SHIPPED is the oracle reference; P071 executed today is the github-script@v7 source)

---

## Context

### Vấn đề hiện tại
GitHub deprecates the Node20 action runtime on **16/06/2026** (tomorrow), forcing Node24. Every fleet repo's `release.yml` still pins Node20-runtime actions (`actions/checkout@v4`, `softprops/action-gh-release@v2`, `actions/github-script@v7`, plus some `cache@v4` / `upload-artifact@v4` / `download-artifact@v4`). After the deadline, re-releases either hard-fail or emit Node-deprecation annotations on every run.

**Why the oracle (P009) is INCOMPLETE:** P009 bumped `checkout@v4→v5` + `action-gh-release@v2→v3` + added `prerelease: ${{ contains(github.ref_name, '-rc') }}` — but predates **P071 (executed TODAY)**, which added `actions/github-script@v7` to the immutable-release "publish" job across doctor/claude-hooks/docs-gate/ship/advisory-inbox (inv-gate already had it). `github-script@v7` is ALSO a Node20 action → must bump `→v8`. **Even inv-gate (the oracle repo) still carries `github-script@v7` and needs the v8 bump.**

### Giải pháp
Per-repo SEQUENTIAL bump of all Node20 actions in `release.yml`, each repo gated by a throwaway `vX.Y.Z-rc1` tag whose run is the **completeness oracle**: 3/3 build jobs green + **0 Node deprecation annotations** + `prerelease` flag correct + `latest` pointer UNCHANGED. If any Node20 action remains, the annotation persists → not done. Start with a proven-shape repo (doctor) as re-validation of the procedure, then fan out.

### Scope
- CHỈ sửa: each repo's `.github/workflows/release.yml` (action `@version` pins only; the P009 `prerelease:` conditional is already present per oracle — verify, don't re-add).
- KHÔNG sửa: any non-release workflow unless it carries a Node20 action that also emits annotations on the rc run; any product source; any other repo's files. Never `git add -A` (3 repos have WIP).

---

## Task 0 — Verification Anchors

> Worker EXECUTE per repo. checkout@v5 + action-gh-release@v3 + github-script@v8 are oracle-proven targets (exist, released). cache/artifact node24-availability is `[needs Worker verify]` — confirm a node24 major exists BEFORE bumping; if none, escalate (do NOT force a nonexistent version). The "0 annotations" rc oracle reveals this per repo.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `actions/checkout@v5` exists + is the node24 target | inv-gate `release.yml` already on `@v5` (oracle-proven) | ✅ `[verified]` per P009 oracle |
| 2 | `softprops/action-gh-release@v3` exists + is node24 target | inv-gate `release.yml` already on `@v3` (oracle-proven) | ✅ `[verified]` per P009 oracle |
| 3 | `actions/github-script@v8` exists + is node24 target | check Marketplace / repo tags before bump | ⏳ `[needs Worker verify]` — oracle predates this; bump target is v8 but Worker confirms v8 is published |
| 4 | `actions/cache@v4` has a node24 major (v5+?) | check before bump on docs-gate + advisory-inbox | ⏳ `[needs Worker verify]` — if no node24 version, annotation unclearable → escalate, do NOT force |
| 5 | `actions/upload-artifact@v4` has a node24 major | check before bump on docs-gate + ship | ⏳ `[needs Worker verify]` — same escalation rule |
| 6 | `actions/download-artifact@v4` has a node24 major | check before bump on docs-gate + ship | ⏳ `[needs Worker verify]` — same escalation rule |
| 7 | inv-gate residual: only `github-script@v7→v8` remains (checkout/release already bumped) | grep `release.yml` in `~/inv-gate` | ⏳ `[needs Worker verify]` — confirm checkout@v5 + release@v3 present, only github-script needs bump |
| 8 | advisory-inbox checkout is INCONSISTENT (`@v4` AND `@v5` in same file) → normalize all to `@v5` | grep `actions/checkout@` in advisory-inbox `release.yml` | ⏳ `[needs Worker verify]` — normalize every occurrence to v5 |
| 9 | advisory-cron matrix is mac+linux only (2-target, no windows) — rc run shows 2 build jobs not 3 | grep `runs-on` / matrix in advisory-cron `release.yml` | ⏳ `[needs Worker verify]` — adjust "3/3 green" → "2/2 green" for this repo only |
| 10 | guard / vps / doc-rotate are PRIVATE but still need the bump | repos accessible; rc tag + verify same as public | ⏳ `[needs Worker verify]` — private does not exempt from deadline |

### Pre-phiếu snapshot (Worker auto first-step)

> Standard rollback point per TICKET_TEMPLATE — but note this phiếu touches MULTIPLE external repos, not just the worktree. The per-repo safety is the **delete-rc-tag** discipline + never-real-release; the worktree snapshot covers only sos-kit-local files (this phiếu file, discovery). Take it anyway:

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

Per-repo rollback = delete the rc tag (`git push --delete origin vX.Y.Z-rc1` + local), revert the `release.yml` edit. NEVER force-recreate a deleted rc tag name — bump to `-rc2` if a re-test is needed (P071 gotcha 3: GitHub may reject recreating a deleted tag).

---

## Debate Log

> Cap = 3 turns. After Turn 3 without consensus → force-escalate Chủ nhà.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification (Task 0, oracle via `gh api` + `curl action.yml` + `grep release.yml`):**

Verified target-version table (all `action.yml` `runs.using` confirmed via raw.githubusercontent.com):

| Action | Current (fleet) | Node24 target | Exists? | `runs.using` |
|--------|----------------|---------------|---------|--------------|
| `actions/checkout` | v4 | **v5** | ✅ v5.0.1 published | node24 |
| `softprops/action-gh-release` | v2 | **v3** | ✅ v3.0.0 published | node24 |
| `actions/github-script` | v7 | **v8** | ✅ v8 tag published (+ v9.0.0) | node24 |
| `actions/cache` | v4 | **v5** | ✅ v5.0.5 published | node24 |
| `actions/upload-artifact` | v4 | **v7** | ✅ v7.0.1 published | node24 (v4 = node20) |
| `actions/download-artifact` | v4 | **v8** | ✅ v8.0.1 published | node24 (v4 = node20) |

Note on artifact major jumps: upload-artifact skips v5/v6 straight to **v7**; download-artifact skips to **v8**. Both v7/v8 confirmed node24 via action.yml. The phiếu says "v5+" which is correct in spirit but the exact targets are v7 and v8 respectively — Architect should patch the inventory table for precision.

- Anchor #1 (`checkout@v5` node24): ✅ — `runs.using: node24` confirmed.
- Anchor #2 (`action-gh-release@v3` node24): ✅ — `runs.using: "node24"` confirmed.
- Anchor #3 (`github-script@v8` exists + node24): ✅ — v8 tag exists, `runs.using: node24` confirmed. (v9.0.0 also exists — v8 is safe target; v9 is newer but untested in fleet.)
- Anchor #4 (`cache@v4` has node24 major): ✅ — `cache@v5` exists, `runs.using: 'node24'` confirmed.
- Anchor #5 (`upload-artifact@v4` has node24 major): ✅ — but target is **v7** (not v5). v7.0.1 published, node24 confirmed.
- Anchor #6 (`download-artifact@v4` has node24 major): ✅ — but target is **v8** (not v5). v8.0.1 published, node24 confirmed.
- Anchor #7 (inv-gate: only github-script residual): ✅ — `~/inv-gate/.github/workflows/release.yml` grep: checkout@v5 ✓ (line 28), action-gh-release@v3 ✓ (line 47), github-script@**v7** ✗ (line 59) — exactly as phiếu states. Only the v7→v8 bump needed.
- Anchor #8 (advisory-inbox checkout mixed v4/v5): ❌ **PHIẾU WRONG** — grep shows ONLY `checkout@v4` (line 28), no v5 occurrence. The "inconsistency" (v4 AND v5 in same file) does NOT exist. advisory-inbox also has NO `cache@v4` (phiếu inventory says it does). See O1.1.
- Anchor #9 (advisory-cron 2-target mac+linux): ✅ — matrix confirmed: `macos-14` + `ubuntu-22.04` only. 2/2 acceptance correct.
- Anchor #10 (private repos accessible): ✅ — guard/vps/doc-rotate grepped successfully; release.yml present and readable.

**Objections (Tầng 1 only):**

- [O1.1] **Inventory table mismatch — docs-gate, ship, advisory-inbox artifact/cache columns are WRONG** (`~/docs-gate/.github/workflows/release.yml`, `~/ship/.github/workflows/release.yml`, `~/advisory-inbox/.github/workflows/release.yml` grepped — ZERO occurrences of `cache`, `upload-artifact`, `download-artifact` in any of these three files). The phiếu's Task 1 inventory says docs-gate has `cache@v4, upload-artifact@v4, download-artifact@v4` and ship has `download-artifact@v4, upload-artifact@v4` — these columns are false positives. The actual Node20 actions in those repos are ONLY `checkout@v4 + action-gh-release@v2 + github-script@v7`. The "cache caveat" note for advisory-inbox is also wrong: advisory-inbox has no cache action at all.
  - Claim: inventory table's "other Node20" column for docs-gate / ship / advisory-inbox is correct.
  - Oracle: `grep -n "actions/cache\|upload-artifact\|download-artifact" ~/docs-gate/.github/workflows/release.yml` → 0 hits; same for ship and advisory-inbox.
  - Soundness: SOUND.
  - Verdict: self-closed via oracle. Inventory table is wrong. This is Tầng 1 because it changes the scope of what gets bumped (3 repos × 3 false-positive actions = 9 phantom bumps that don't exist, and Worker would waste time + risk touching non-existent lines).

- [O1.2] **advisory-inbox checkout-v4/v5 inconsistency does NOT exist** — anchor #8 assumes mixed `@v4` AND `@v5` in the same file (`release.yml`). Reality: exactly one `checkout@v4` on line 28, no v5 occurrence. Normalization instruction "normalize ALL to v5" is correct in outcome (there's one v4 to bump) but the premise of inconsistency is wrong. Low-blast (outcome identical), but Architect should correct the anchor text to avoid confusion during EXECUTE.
  - Claim: advisory-inbox release.yml has both @v4 and @v5 checkout references.
  - Oracle: `grep -n "actions/checkout" ~/advisory-inbox/.github/workflows/release.yml` → single hit `@v4` line 28.
  - Soundness: SOUND.
  - Verdict: self-closed via oracle. Anchor #8 premise is false; outcome (bump to v5) unchanged.

- [O1.3] **upload-artifact and download-artifact node24 targets are v7 and v8, not v5** — the phiếu says "node24 major (v5+?)" for both. The actual targets are `upload-artifact@v7` and `download-artifact@v8` (confirmed node24 via action.yml). This matters for EXECUTE: Worker must write the correct version pin, and if Architect's plan says "v5" in a later phiếu turn, it would ship a node20 action (`upload-artifact@v5` = node20 per release history). Document exact targets now.
  - Claim: node24 major for upload-artifact and download-artifact is "v5+" (ambiguous).
  - Oracle: `gh api repos/actions/upload-artifact/releases --jq '.[].tag_name'` → v7.0.1 is latest; `curl action.yml @v7 runs.using` → node24. Same pattern for download-artifact @v8.
  - Soundness: SOUND.
  - Verdict: self-closed via oracle. Targets are v7 (upload) and v8 (download). Architect should patch inventory table column "other Node20" to reflect exact targets.

**Proposed alternatives (for O1.1 — the only material blocker):**

- A. **Architect patches inventory table only** (Worker recommends — because): Remove the false `cache@v4 / upload-artifact@v4 / download-artifact@v4` columns from docs-gate and ship rows. Remove the `cache@v4` from advisory-inbox row. Correct upload-artifact target to v7, download-artifact to v8 where they do appear (none in these 3 repos). The rc-oracle procedure stays intact — 0 annotations will confirm no residual Node20 actions. Minimal change; zero risk.
- B. **EXECUTE with inline correction** — Worker self-corrects during EXECUTE (Tầng 2 judgment: local column fix, not architectural). Risk: the cache/artifact "escalation if no node24 version" branch in Task 1 is now dead code but Worker would still have to reason through it repo-by-repo. Cleaner to fix the phiếu first.

**Worker leans A** — the inventory table is the plan's backbone for 10 sequential repos. A stale table with phantom columns increases cognitive load + risk of Worker touching non-existent lines. 5-line patch to phiếu = lower risk than inline correction at EXECUTE.

**Status:** ⏳ AWAITING ARCHITECT RESPONSE

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date] — code execution may begin

---

## Nhiệm vụ

### Task 1: Per-repo Node20-action bump + rc-verify + delete-rc (SEQUENTIAL)

**Procedure per repo (do NOT batch — finish + delete-rc one repo before starting the next, IG-09):**

1. **Edit** the repo's `.github/workflows/release.yml`:
   - `actions/checkout@v4` → `@v5` (all occurrences; advisory-inbox has mixed v4/v5 → normalize ALL to v5).
   - `softprops/action-gh-release@v2` → `@v3`.
   - `actions/github-script@v7` → `@v8` (where present — the P071 publish-job addition + inv-gate residual).
   - `actions/cache@v4` / `actions/upload-artifact@v4` / `actions/download-artifact@v4` → node24 major **only if Task 0 anchors #4-6 confirm one exists**; else leave + escalate (the rc oracle will show whether the annotation is unclearable).
   - Verify (do NOT re-add) the P009 conditional `prerelease: ${{ contains(github.ref_name, '-rc') }}` is present.
   - Stage ONLY `release.yml` (`git add .github/workflows/release.yml`) — NEVER `git add -A` (3 repos have WIP).
2. **Commit + push** to the repo's default branch (CI workflow file; this is the repo's own infra, not sos-kit product code).
3. **Tag a TEST rc**: `git tag vX.Y.Z-rc1 && git push origin vX.Y.Z-rc1` (pick the next patch above current latest; rc number per gotcha 3).
4. **Verify the rc run (the completeness ORACLE — all 4 must pass):**
   - (a) **3/3 build jobs green** (advisory-cron: **2/2** — mac+linux matrix only, anchor #9).
   - (b) **0 Node deprecation annotations** on the run. ← completeness gate. If a `cache`/`artifact` annotation persists with no node24 version available → ESCALATE (anchor #4-6), do not force.
   - (c) **`prerelease` flag correct** on the created rc release (true for `-rc`).
   - (d) **`latest` release pointer UNCHANGED** — the rc must NOT claim latest.
5. **Delete the rc tag + its release**: `git push --delete origin vX.Y.Z-rc1` + delete local tag + the rc GitHub release. Do not leave rc tags around. If a re-test is needed, use `-rc2` (never recreate the deleted name — gotcha 3).
6. **Next repo.**

**Repo order (do doctor FIRST as procedure re-validation, then fan out):**

| # | Repo | checkout | action-gh-release | github-script | other Node20 | notes |
|---|------|----------|-------------------|---------------|--------------|-------|
| 1 | doctor (proven-shape canary) | v4→v5 | v2→v3 | v7→v8 | — | run this first; re-validates the whole procedure |
| 2 | claude-hooks | v4→v5 | v2→v3 | v7→v8 | — | |
| 3 | docs-gate | v4→v5 | v2→v3 | v7→v8 | cache@v4, upload-artifact@v4, download-artifact@v4 | artifact/cache `[needs Worker verify]` node24-avail |
| 4 | ship | v4→v5 | v2→v3 | v7→v8 | download-artifact@v4, upload-artifact@v4 | same artifact caveat |
| 5 | advisory-inbox | v4 AND v5 → normalize v5 | v2→v3 | v7→v8 | cache@v4 | anchor #8 normalize; cache caveat |
| 6 | inv-gate (oracle) | v5 ✓ already | v3 ✓ already | **v7→v8 (residual!)** | — | ONLY the github-script bump; verify the other two are already v5/v3 |
| 7 | guard (private) | v4→v5 | v2→v3 | — | — | private ≠ exempt |
| 8 | vps (private) | v4→v5 | v2→v3 | — | — | private ≠ exempt |
| 9 | doc-rotate (private) | v4→v5 | v2→v3 | — | — | private ≠ exempt |
| 10 | advisory-cron | v4→v5 | v2→v3 | — | — | matrix mac+linux → **2/2 green** (anchor #9) |

**Lưu ý:**
- This phiếu does NOT require cutting a REAL release for any repo. The next real release will be clean. Worker MAY cut a real release if a specific repo needs one — but that's a separate decision, surface to Chủ nhà at APPROVAL_GATE.
- The P071→github-script@v7 interaction is the trap the P009 oracle misses — github-script MUST be bumped wherever P071 added it. Log this interaction in Discovery.
- If ANY repo's rc run shows an unclearable annotation (cache/artifact has no node24 version), STOP that repo, escalate, continue with the remaining repos (don't block the whole fleet on one action's missing version).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `<each repo>/.github/workflows/release.yml` × 10 | Task 1: bump Node20 action pins → node24 majors; verify P009 `prerelease:` conditional present |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `~/inv-gate/.github/workflows/release.yml` checkout/release pins | already v5/v3 — do NOT re-bump; only github-script@v7→v8 |
| any non-release workflow per repo | only touch if it carries a Node20 action that emits annotations on the rc run |
| any product source (`src/`, `crates/`, etc.) | untouched — this is CI-only |

---

## Luật chơi (Constraints)

1. **SEQUENTIAL per-repo (IG-09).** Finish + delete-rc one repo before the next. Never bump+push all 10 at once.
2. **rc-tag test BEFORE any real release.** Never bump + real-release blind — the rc run is the only proof the bump is complete.
3. **"0 Node deprecation annotations" = the completeness oracle.** Green build alone is NOT enough; a residual Node20 action stays green but keeps annotating.
4. **Delete the rc tag + rc release after verify.** Don't leave rc tags. NEVER recreate a deleted tag name — bump to `-rc2` (P071 gotcha 3).
5. **Stage only `release.yml` — never `git add -A`.** 3 repos (advisory-inbox + the private ones likely) have WIP; respect each repo's working tree.
6. **advisory-cron is 2-target** (mac+linux matrix) → acceptance = 2/2 green, not 3/3.
7. **guard / vps / doc-rotate are private but still bumped** — private ≠ deadline-exempt.
8. **If a cache/artifact action has no node24 version → ESCALATE, do NOT force** a nonexistent version. Continue with remaining repos.

---

## Nghiệm thu

### Automated (per repo, the rc oracle)
- [ ] rc run: 3/3 build jobs green (advisory-cron: 2/2)
- [ ] rc run: **0 Node deprecation annotations**
- [ ] rc release: `prerelease` flag correct (true)
- [ ] `latest` release pointer UNCHANGED
- [ ] rc tag + rc release DELETED after verify

### Manual Testing
- [ ] doctor (repo #1) passes all 4 oracle checks → procedure re-validated before fan-out
- [ ] inv-gate: confirmed only github-script@v7→v8 needed (checkout/release already current)

### Regression
- [ ] No real release was triggered unintentionally (rc-only; `latest` unchanged everywhere)
- [ ] Each repo's WIP untouched (only `release.yml` staged)

### Docs Gate
- [ ] `CHANGELOG.md` — entry for P072 fleet Node20 bump (sos-kit-local; the bumped repos are external — note them in Discovery)
- [ ] Tầng 1 docs: this phiếu touches external repos' CI, not a sos-kit contract surface → if a sos-kit doc references the action versions (none expected), update; else write "Tầng 1 N/A — external CI bump" in Discovery.

### Discovery Report
- [ ] Write to `docs/discoveries/P072.md`
  - Per-repo bump status (10 repos: done / escalated / skipped + why)
  - Any unclearable annotation (which repo, which action, no-node24-version)
  - The **P071→github-script@v7 interaction** explicitly noted (oracle gap the P009 reference missed)
  - cache/artifact node24-availability findings (anchors #4-6 resolved)
  - Whether any real release was cut + why (default: none)
  - Tier escalations (write "None" if no 2→1 escalation)
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`
- [ ] Mark `[FLEET-NODE]` done in `docs/BACKLOG.md` line 55
