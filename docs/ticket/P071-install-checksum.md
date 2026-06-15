# PHIẾU P071: Release-asset integrity — `.sha256` publish + install.sh fetch-verify + version pinning

> **ID format:** `P` + 3 digits. P071 = next active ID (no P07x in `docs/ticket/`, BACKLOG Active sprint pins this).
> **Filename:** `docs/ticket/P071-install-checksum.md`.
> **Branch:** `feat/P071-install-checksum`.

---

> **Loại:** Feature (security hardening)
> **Ưu tiên:** P1 (gates sos-kit going public — open-source-hardening Leg 2)
> **Tầng:** 1 (security surface — supply-chain integrity of the `curl|sh` installer. AUTO Tầng 1 per CLAUDE.md DOCS GATE mapping + WORKFLOW_V2.2 §0.1; touches install download trust anchor.)
> **Ảnh hưởng:** `install.sh` (this repo) + `.github/workflows/release.yml` across 10 sibling tool-repos (OUT-OF-TREE — delegated sub-tasks) + `docs/security/INVARIANTS.md` + `docs/SETUP.md` + `CHANGELOG.md`.
> **Dependency:** None upstream. Internal hard sequence: **Stage 1 (CI ×10 + re-release) MUST land + assets verified-present BEFORE Stage 2 (install.sh enforce)** — see Luật chơi §1 rollout hazard.

---

## Context

### Vấn đề hiện tại

The `curl|sh` installer (`install.sh`) pulls 10 prebuilt release binaries over HTTPS but with **NO checksum/signature verification**, and the URL uses `releases/latest/download/` which is **unpinned** (whatever GitHub currently points "latest" at). Trust anchor today = the GitHub account alone (self-documented gap, `install.sh:35-37`, INV-LOCAL candidate `docs/security/INVARIANTS.md:127-128`). Threat vector = a swapped/poisoned release asset (compromised account, MITM on a redirect, a bad re-tag) is downloaded + `chmod +x` + run with zero detection. This is a supply-chain HIGH and **gates sos-kit going public** (BACKLOG Active sprint, Leg 2).

### Giải pháp

Plain **sha256** (NOT minisign/cosign — that violates WORKFLOW_V2.2 §0.1 cheapest-mechanism and creates a verify-the-verifier bootstrap problem). Two sequenced stages:

- **Stage 1 — CI publishes `.sha256` (×10 repos) + re-tag/re-release** so the `.sha256` assets actually exist on each release. Each repo's `release.yml` "Package asset" step gains ~2 lines: compute `shasum -a 256 "<asset>" > "<asset>.sha256"` and add `<asset>.sha256` to the `softprops/action-gh-release` `files:` list. `shasum -a 256` is present on all 3 runner OSes (macos-14 / ubuntu-22.04 / windows-2022).
- **Stage 2 — install.sh** gains: a `$SHA` cross-platform probe + a verify block inside `fetch_bin()` (fetch `<asset>.sha256`, recompute hash of the downloaded `.tmp`, string-compare first field, mismatch → abort, required-bin with missing `.sha256` → abort) + an optional version-pinning manifest (`releases/latest/download/` → `releases/download/${ver}/`). Then a real **2-OS discrimination test** (good binary verifies green; corrupted binary/hash REJECTED; missing `.sha256` on a required bin aborts).

### Scope

- CHỈ sửa trong tree này: `install.sh`, `docs/security/INVARIANTS.md`, `docs/SETUP.md`, `CHANGELOG.md`.
- Cross-repo (OUT-OF-TREE — delegated, per-repo sub-tasks, NOT in this checkout): each of the 10 tool-repos' `.github/workflows/release.yml` + a re-release/re-tag so assets carry `.sha256`.
- KHÔNG sửa: `bin/sos.sh`, `templates/setup-dev.sh` (dev/cargo path — out of scope), any hook script.

---

## Task 0 — Verification Anchors

> Architect is docs-only (Read/Write/Glob, no Bash/Grep). Anchors below are tagged `[verified]` (Architect Read the file this session), `[unverified]` (from survey/docs, Architect did not open), or `[needs Worker verify]` (Architect cannot know — Worker MUST verify before applying).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `fetch_bin()` builds `url=".../releases/latest/download/${bin}-${TARGET}${EXT}"`, curls to `${dest}.tmp`, then `mv`+`chmod +x`, Darwin strips quarantine, runs `--version`. NO verify step. | Read `install.sh` | ✅ `[verified]` lines 70-86 (url L72, curl→`.tmp` L75, mv L76, chmod L77, xattr L80, `--version` L81) |
| 2 | `BINARIES="doctor claude-hooks docs-gate ship advisory-inbox inv-gate"` (6 required/fail-closed); `OPTIONAL_BINARIES="guard vps doc-rotate advisory-cron"` (4 warn-continue) = 10 total | Read `install.sh` | ✅ `[verified]` L38 (required), L46 (optional) |
| 3 | `TARGET` ∈ aarch64-apple-darwin / x86_64-unknown-linux-gnu / x86_64-pc-windows-msvc; `EXT=.exe` Windows-only; Intel-Mac→arm64 via Rosetta | Read `install.sh` | ✅ `[verified]` L49-65 (case L50-65, EXT L54, Rosetta L57) |
| 4 | Self-deferral comment referencing [P071] | Read `install.sh` | ✅ `[verified]` L35-37 ("NO checksum/signature verification yet … .sha256 publishing + verify is the planned cure") |
| 5 | `docs/security/INVARIANTS.md` INV-LOCAL section names `install.sh download integrity [P071]` as a candidate | Read `docs/security/INVARIANTS.md` | ✅ `[verified]` L127-128 ("No local INV … Candidates … install.sh download integrity [P071]") |
| 6 | `docs/SETUP.md` exists and has an install section that may reference the curl\|sh flow | `grep -n "install.sh\|curl" docs/SETUP.md` | ⚠️ `[needs Worker verify]` — file exists (Glob), Architect did not open. Worker confirm whether user-facing steps change. |
| 7 | All 10 source repos live under `~/<name>` each with `.github/workflows/release.yml`, byte-identical template except the `BIN=<name>` line | `for r in doctor claude-hooks docs-gate ship advisory-inbox inv-gate guard vps doc-rotate advisory-cron; do ls ~/$r/.github/workflows/release.yml; done` | ⚠️ `[needs Worker verify]` — OUT-OF-TREE, not in this checkout. Survey ground truth; Worker verify per-repo before editing. |
| 8 | `release.yml` "Package asset" step copies binary → `<bin>-<target><EXT>`, exports `ASSET`, uploads via `softprops/action-gh-release@v2` `files: ${{ env.ASSET }}` | `grep -n "ASSET\|action-gh-release\|files:" ~/<repo>/.github/workflows/release.yml` | ⚠️ `[needs Worker verify]` — OUT-OF-TREE. Survey says so; confirm exact var name + upload syntax per-repo. |
| 9 | **inv-gate** release.yml DRIFTS: checkout@v5 / action-gh-release@v3 / adds `prerelease:` — do NOT blind-copy other repos' YAML onto it | Read `~/inv-gate/.github/workflows/release.yml` | ⚠️ `[needs Worker verify]` — OUT-OF-TREE. Survey-flagged drift; verify before editing inv-gate. |
| 10 | **advisory-cron** builds only 2 targets (mac+linux, no Windows — compile_error) | Read `~/advisory-cron/.github/workflows/release.yml` | ⚠️ `[needs Worker verify]` — OUT-OF-TREE. Survey-flagged; verify matrix before editing. |
| 11 | `shasum -a 256` available on macos-14 / ubuntu-22.04 / windows-2022 GitHub runners | Stage-1 CI run prints hash on all 3 OS jobs (the re-release run IS the verify) | ⏳ `[needs Worker verify]` — capability assumption; CI run confirms. If any runner lacks it, fall back to `sha256sum` in that job's step. |
| 12 | At install time: macOS has `shasum -a 256` but not `sha256sum`; Linux has `sha256sum` (maybe not `shasum`); Git Bash has both | `command -v sha256sum; command -v shasum` on each target OS during the 2-OS discrimination test | ⏳ `[needs Worker verify]` — drives the `$SHA` probe (Task 3). Discrimination test (Task 6) confirms on real mac + real linux. |
| 13 | Version-pinning manifest format (a `versions.env`-style `bin→tag` map) does not already exist in this repo under another name | `ls install.sh versions.env 2>/dev/null; grep -rn "releases/download/" install.sh` | ⚠️ `[needs Worker verify]` — Architect saw only `releases/latest/download/` (L72). Worker confirm no existing manifest before creating one. |

**Note on ❌/⚠️:** anchors 6-13 are unverifiable by docs-only Architect (out-of-tree repos + runtime capability). They are punted to Worker per the "đá bóng cho Thợ" doctrine — this is correct, not a gap. Worker grep-verifies each at CHALLENGE/EXECUTE.

### Pre-phiếu snapshot (Worker auto first-step)

> Worker EXECUTE FIRST ACTION (before any edit, before Task 0 grep): take a rollback point.

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/ — auto-cleaned on phieu-done"
```

`.backup/` is gitignored. `phieu-done` cleans up automatically.

---

## Debate Log

> Auto-populated by Worker (CHALLENGE) and Architect (RESPOND). Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**

- #1 ✅ `fetch_bin()` L70-86 confirmed: url L72 `releases/latest/download`, curl→`.tmp` L75, mv L76, chmod L77, xattr L80, `--version` L81. NO verify step. Exactly as phiếu says.
- #2 ✅ `BINARIES` L38 = "doctor claude-hooks docs-gate ship advisory-inbox inv-gate" (6). `OPTIONAL_BINARIES` L46 = "guard vps doc-rotate advisory-cron" (4). Matches phiếu.
- #3 ✅ L49-65 confirmed: case `Darwin-arm64`→aarch64, `Linux-x86_64`→x86_64-linux, MINGW/MSYS/CYGWIN→windows+EXT=.exe, `Darwin-x86_64`→aarch64 (Rosetta). EXT L54, Rosetta L57.
- #4 ✅ Self-deferral comment L35-37 confirmed verbatim: "NO checksum/signature verification yet … .sha256 publishing + verify is the planned cure".
- #5 ✅ (Architect-verified, not re-read by Worker — docs-only anchor, trust carries).
- #6 ✅ `docs/SETUP.md` has no `install.sh` or `curl` install-step references in user-facing flow. Install section references `cargo install` (dev path) only. If Task 3/4 adds no new user-facing step, SETUP.md needs no change. Worker will confirm again at EXECUTE.
- #7 ✅ All 10 repos present under `~/<name>/.github/workflows/release.yml`. None MISSING.
- #8 ✅ **EXACT shape confirmed for 8/10 repos:** var name is `ASSET` (not a survey guess), set via `echo "ASSET=..." >> "$GITHUB_ENV"`. Upload via `softprops/action-gh-release@v2` with `files: ${{ env.ASSET }}` (single-file, not a list). **Task 1 patch must change `files:` from a scalar to a `|`-block with two entries.** Shape is identical across doctor/claude-hooks/docs-gate/ship/advisory-inbox/guard/vps/doc-rotate.
- #9 ✅ inv-gate DRIFT confirmed: uses `actions/checkout@v5` (others @v4), `softprops/action-gh-release@v3` (others @v2), adds `prerelease: ${{ contains(github.ref_name, '-rc') }}`. All else identical. Phiếu's warning is accurate — do NOT blind-copy.
- #10 ⚠️ advisory-cron: matrix is mac+linux only (no Windows) as phiếu says. BUT: advisory-cron uses `softprops/action-gh-release@v2` (same as others) and **has `permissions: contents: write`** at job level (different structure from the others which have it at workflow level — all others have `permissions: contents: write` at top, advisory-cron has it at `jobs.build` scope). Task 1 patch must locate it at the right YAML indentation level. Low risk — just flag for Worker at EXECUTE.
- #11 ⏳ CI runner capability — confirmed only on local mac (see #12). Stage-1 CI run itself is the verify per phiếu — correct deferral.
- #12 ✅ **Mac confirmed:** `/sbin/sha256sum` EXISTS (Darwin 1.0, produces `<hash>  -` format). `/usr/bin/shasum` also exists (v6.02). **CRITICAL FINDING: The Task 3 probe prefers `sha256sum` first — and on macOS `/sbin/sha256sum` IS on PATH** (`/sbin` in `$PATH`). This means the macOS install path will NOT exercise `shasum -a 256` — it will use `sha256sum` instead. The comment in Task 3 says "mac=shasum -a 256" but the probe will actually select `sha256sum` on any mac with `/sbin/sha256sum`. This is NOT a correctness bug (sha256sum works fine on mac) but it IS a documentation mismatch and means the 2-OS discrimination test may not exercise the `shasum` branch at all unless Linux also has `sha256sum` first. See objection [O1.1].
- #13 ✅ No `versions.env` exists in this repo. `grep releases/download/` in install.sh returns zero hits — only `releases/latest/download/` is present. No existing manifest. Task 5 starts from blank.

**`$2`-in-`fetch_bin` scope — CRITICAL (Architect flagged `[needs Worker verify]`):**

`fetch_bin()` body (L70-86): only assigns `bin="$1"`, then `url=`, `dest=`. It does NOT assign `$2` to any named variable. The body only uses `bin`, `url`, `dest` — `$2` is never read inside the function. Call sites: L88 `fetch_bin "$bin" required` and L95 `fetch_bin "$bin" optional`. In POSIX `sh`, positional parameters `$1`/`$2` ARE available inside a function body — `$2` is the second positional parameter of the function call, not the outer script's `$2`. So `$2` IS in scope inside `fetch_bin` when called with two args. **The Task 4 verify block using `if [ "$2" = required ]` is CORRECT in POSIX sh.** Self-closed via oracle: `sh -c 'f() { echo "$2"; }; f a b'` → prints `b`. No Tầng-1 objection here.

Claim: Is `$2` readable inside `fetch_bin()` as the second call argument?
Oracle: POSIX sh positional parameter semantics — SOUND.
Soundness: SOUND.
Verdict: self-closed via oracle — `$2` is correct as written in Task 4.

**Objections (Tầng 1 only):**

- [O1.1] **`sha256sum` vs `shasum` probe order — macOS exercises wrong branch + comment mismatch**
  File: `install.sh` (the Task 3 probe in the phiếu, not yet written, but the spec at Task 3 + Lưu ý says "mac=shasum -a 256").
  Claim: On macOS, `/sbin/sha256sum` is present and in `$PATH`, so the probe (which checks `sha256sum` first) will select `sha256sum`, NOT `shasum -a 256`. The Task 3 Lưu ý states "macOS lacks `sha256sum` by default" — this is false on this mac. The 2-OS discrimination test (Task 6) also says "mac path uses `shasum -a 256`" but that branch may never fire on a mac with `/sbin/sha256sum`.
  Code evidence: `command -v sha256sum` → `/sbin/sha256sum` (Darwin 1.0); `/sbin` is in `$PATH`; `command -v shasum` → `/usr/bin/shasum`.
  Impact: (a) The comment in install.sh would be wrong/misleading; (b) the discrimination test's stated "two branches exercised" goal (Task 6) is weaker than claimed — both mac and Linux may use `sha256sum`; (c) if a CI runner or user machine is a minimal macOS without `/sbin/sha256sum` (possible on older macOS or clean Docker mac), the behavior reverses. **However: both tools produce identical hash format; correctness is unaffected.** This is a documentation accuracy / test-coverage assertion issue, NOT a security correctness issue.
  Tầng classification: Tầng 2 — the gate's correctness is fine; the comment is cosmetic. But the Task 6 "both branches exercised" claim in the Nghiệm thu is technically incorrect as written. Worker can self-adapt (fix the comment + amend the test description) without Architect respawn.
  Verdict: **Tầng 2, Worker self-adapt at EXECUTE** — fix Task 3 comment from "macOS lacks sha256sum" to "macOS may have sha256sum (/sbin) — probe prefers it; shasum is the fallback"; update Task 6 note that "two branches exercised" means Linux uses sha256sum and the shasum branch is tested by temporarily masking sha256sum. Log in Discovery.

- [O1.2] **advisory-cron `permissions:` indentation level differs from other repos**
  File: `~/advisory-cron/.github/workflows/release.yml` L11-12.
  Claim: advisory-cron's `permissions: contents: write` is at `jobs.build` scope, not at the workflow top level (as in the other 9 repos). Task 1 adds the hash step with no YAML indentation guidance for advisory-cron.
  Tầng classification: Tầng 2 — the step content is identical; only the YAML nesting level differs. Worker reads the file at EXECUTE and inserts at the correct indentation. Log in Discovery.
  Verdict: **Tầng 2, Worker self-adapt at EXECUTE**.

- [O1.3] **Task 5 manifest-before-loop ordering is a real blocker — phiếu flags it but leaves it open**
  File: `install.sh` L87-98 (binary loop runs BEFORE L101-106 kit clone).
  Claim: If `versions.env` ships IN the kit checkout (cloned at L101-106), it cannot be sourced before the binary loop at L87-98. The phiếu's Task 5 Lưu ý acknowledges this ("Worker resolves ordering") but leaves the resolution unspecified. Two viable approaches: (A) inline the versions directly in install.sh as shell variables (no external file needed); (B) curl the manifest from raw.githubusercontent.com before the loop. Approach A is simpler and keeps everything in one file. Approach B adds a dependency on raw.githubusercontent.com availability.
  Tầng classification: The phiếu explicitly punts the ordering resolution to Worker ("Worker resolves ordering; this is a real edge, not hand-wave") and Task 5 is already marked OPTIONAL at APPROVAL_GATE. If pinning is approved, the resolution is a Tầng-2 implementation detail (no schema change, no API change, no new external dependency beyond what's already in install.sh). Worker recommends Approach A (inline in install.sh) at EXECUTE if pinning is approved.
  Verdict: **Tầng 2, Worker self-adapt at EXECUTE if Task 5 approved**.

**No Tầng-1 blockers found.** All objections are Tầng 2, self-closeable by Worker. The phiếu's security logic is sound: `$2` scope is correct, rollout hazard sequence is correctly specified, recompute-not-shasum-c rationale is correct, all 10 repos exist and have the expected shape (with documented drift for inv-gate).

**Status:** Worker accepted V1 — no Tầng-1 challenges. 3 Tầng-2 self-adapt items logged above (probe comment fix, advisory-cron indentation, Task 5 ordering). Ready for Chủ nhà approval gate.

APPROVAL_GATE decisions needed from Chủ nhà before EXECUTE:
1. Rollout path: A (Stage 1 fully land before Stage 2 flip) vs B (transitional grace mode) — phiếu recommends A.
2. Task 5 (version pinning): approve or skip — phiếu marks OPTIONAL.

### Final consensus
- Phiếu version: V1 (no RESPOND needed — 0 Tầng-1 objections; 3 Tầng-2 items self-adapt by Worker at EXECUTE)
- Total turns: 1 (Worker CHALLENGE only)
- **Approved by Chủ nhà: 2026-06-15** — code execution may begin, with these APPROVAL_GATE decisions:
  - **Rollout = Path A (clean).** Stage 1 (CI `.sha256` + re-release on all 6 REQUIRED repos) MUST fully land + assets verified-present BEFORE Stage 2 flips install.sh to enforce. No transitional grace mode.
  - **Task 5 (version pinning) = SKIP.** Checksum alone closes the supply-chain HIGH; pinning is reproducibility (separate concern) → split to follow-up `[P-pin]` in BACKLOG. Do NOT create `versions.env`; leave `releases/latest/download/` as-is. Objection [O1.3] (manifest ordering) is therefore moot for this phiếu.
  - **3 Tầng-2 self-adapts confirmed for EXECUTE:** [O1.1] fix Task 3 probe comment ("macOS may have /sbin/sha256sum — probe prefers it; shasum is fallback") + amend Task 6 so the `shasum` branch is exercised by temporarily masking `sha256sum`; [O1.2] advisory-cron `permissions:` at job scope — insert hash step at correct indent; `files:` scalar→`|`-block (#8).
  - **Task 6 (2-OS discrimination test) = acceptance-gated on Sếp's Linux + Windows dogfood** (BACKLOG Active sprint item). P071 NOT done until probe verifies green on Linux (`sha256sum`) + Windows Git Bash + mac.

---

## Nhiệm vụ

> **STAGE 1 (Tasks 1-2) MUST land + assets verified-present on every required repo's release BEFORE Stage 2 (Tasks 3-6) flips install.sh to enforce.** A half-rollout = fail-closed abort for every user (Luật chơi §1).

### Task 1: CI — publish `.sha256` alongside each release asset (×10 repos, OUT-OF-TREE)

**File:** `~/<repo>/.github/workflows/release.yml` for each of: `doctor claude-hooks docs-gate ship advisory-inbox inv-gate guard vps doc-rotate advisory-cron` — **delegated per-repo sub-task, NOT in this checkout.** Edit one repo at a time, verify, then next (per IG-09 "sai 1 phát push 10 = toác chùm").

**Tìm:** the "Package asset" step that exports the asset path (survey: a var named `ASSET`, value `<bin>-<target><EXT>`) and the `softprops/action-gh-release` upload step with `files:` pointing at that asset. `[needs Worker verify]` exact var name + step text per-repo (anchor #8).

**Thay bằng / Thêm:** after the asset is packaged, add a hash line; add the `.sha256` to the upload list. Conceptual diff (Worker adapts to each repo's exact var/syntax):

```yaml
# in the Package asset step, after $ASSET is set:
shasum -a 256 "$ASSET" > "$ASSET.sha256"
# in the action-gh-release step, files: list — add the second line:
#   files: |
#     ${{ env.ASSET }}
#     ${{ env.ASSET }}.sha256
```

**Lưu ý:**
- **inv-gate (anchor #9):** uses checkout@v5 / action-gh-release@v3 / has `prerelease:`. Do NOT blind-copy another repo's YAML onto it — only add the 2 hash/upload lines, preserve its drift. `[needs Worker verify]`.
- **advisory-cron (anchor #10):** matrix is mac+linux only (no Windows). Add the hash line in whatever per-target job packages the asset; do NOT add a Windows job. `[needs Worker verify]`.
- `shasum -a 256` on all 3 runner OSes (anchor #11) — if a runner job lacks it, fall back to `sha256sum` in that job. The Stage-1 CI run itself is the capability verify.
- guard / vps / doc-rotate are PRIVATE repos — still edit their `release.yml` (the assets are pulled as OPTIONAL by install.sh; verify present once published). `[needs Worker verify]`.

### Task 2: Re-tag / re-release so `.sha256` assets exist on the releases install.sh pulls (×10, OUT-OF-TREE)

**File:** N/A (release/tag operation per repo, delegated sub-task).

**Tìm:** the tag/release that install.sh's URL resolves to (today `releases/latest`). For each repo, after Task 1 CI lands, cut a release (or re-run the release workflow on the existing tag) so the published assets include `<bin>-<target><EXT>.sha256`.

**Thay bằng / Thêm:** confirm each repo's `latest` (or the pinned tag chosen in Task 5) release now lists the `.sha256` asset for every target install.sh requests.

**Lưu ý:**
- This is the gate for Stage 2. Worker MUST verify-present (e.g. `curl -fsI ".../releases/latest/download/<bin>-<target>.sha256"` → 200) for **all 6 required** bins on **all targets the installer requests**, before touching install.sh. `[needs Worker verify]`.
- OPTIONAL bins (guard/vps/doc-rotate private; advisory-cron) — verify present where published; install.sh handles missing OPTIONAL gracefully (Task 4 verify rule treats OPTIONAL missing-`.sha256` as warn-skip, NOT abort).
- Coordinate with the FLEET-NODE Node20-bump (BACKLOG) if touching the same `release.yml` — but that is a SEPARATE phiếu; do NOT bundle. Note the overlap in the Discovery Report.

### Task 3: install.sh — add cross-platform `$SHA` hash-tool probe

**File:** `install.sh`

**Tìm:** the platform-detection block ending at L66 (`echo "▶ Platform: $OS $ARCH → $TARGET"`), before the `# ── 2. Prebuilt binaries` section (L68). `[verified]` anchor #3.

**Thay bằng / Thêm:** after platform detect, add a probe that picks the available hash tool (string form so it can be word-split into the command):

```sh
# ── Hash tool probe (cross-platform: linux=sha256sum, mac=shasum -a 256, GitBash=both) ──
if command -v sha256sum >/dev/null 2>&1; then
  SHA_CMD="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
  SHA_CMD="shasum -a 256"
else
  echo "✗ No sha256 tool (need sha256sum or shasum) — cannot verify downloads. ABORTING." >&2
  exit 1
fi
```

**Lưu ý:**
- macOS lacks `sha256sum` by default; Linux lacks `shasum`; Git Bash has both (anchor #12). Probe order above prefers `sha256sum` (Linux-native) then falls back. `[needs Worker verify]` on a real mac + real linux in Task 6.
- No-hash-tool on a required-bin install = fail-closed abort (the whole point of the gate). This is correct fail-closed behavior, not over-strict.
- `SHA_CMD` is intentionally unquoted at call site (Task 4) so `shasum -a 256` word-splits into command + args. Keep it a plain string, not an array (this is `sh`, not `bash`).

### Task 4: install.sh — verify block inside `fetch_bin()`

**File:** `install.sh`

**Tìm:** inside `fetch_bin()`, the success branch after the binary downloads — between the `curl ... -o "${dest}.tmp"` success (L75) and the `mv "${dest}.tmp" "$dest"` (L76). `[verified]` anchor #1.

**Thay bằng / Thêm:** before `mv`, fetch the `.sha256`, recompute the hash of `.tmp`, compare first field; mismatch or (required-bin) missing-`.sha256` → fail. Pass `<required|optional>` (already `$2` in the call sites L88/L95) to decide abort-vs-skip:

```sh
# inside fetch_bin(); $2 = required|optional. After curl success, BEFORE mv:
sha_url="${url}.sha256"
if curl -fSL --proto '=https' --connect-timeout 30 --max-time 60 -o "${dest}.sha256.tmp" "$sha_url"; then
  expected=$(cut -d' ' -f1 "${dest}.sha256.tmp")
  actual=$($SHA_CMD "${dest}.tmp" | cut -d' ' -f1)
  rm -f "${dest}.sha256.tmp"
  if [ "$expected" != "$actual" ]; then
    echo "✗ CHECKSUM MISMATCH for $bin — expected $expected, got $actual. ABORTING (poisoned/corrupt asset?)." >&2
    rm -f "${dest}.tmp"
    return 1
  fi
  echo "  ✓ sha256 verified"
else
  rm -f "${dest}.sha256.tmp"
  if [ "$2" = required ]; then
    echo "✗ No .sha256 published for required bin $bin — cannot verify. ABORTING." >&2
    rm -f "${dest}.tmp"
    return 1
  fi
  echo "  ⚠ no .sha256 for optional $bin — skipping verify (see BACKLOG transitional grace)."
fi
# (existing) mv "${dest}.tmp" "$dest"; chmod +x; ...
```

**Lưu ý:**
- **Recompute-and-string-compare** the `.tmp` file — do NOT use `shasum -c` / `sha256sum -c`, because `-c` matches by the filename recorded in the `.sha256` (the CI wrote `<bin>-<target>` but we downloaded to `<dest>.tmp` — names differ across the rename). First-field compare sidesteps this. (Survey ground truth.)
- `$2` is the required/optional flag the callers already pass (L88 `required`, L95 `optional`) — `[verified]` anchor #1/#2 call sites. Confirm `$2` is in scope inside `fetch_bin()` `[needs Worker verify]` (currently `fetch_bin` reads only `bin="$1"` at L71).
- Required missing-`.sha256` → abort (Stage 1 must have published it; if absent, Stage 1 is incomplete → correct to fail). OPTIONAL missing → warn + install unverified (these are private/Windows-skip bins; the grace keeps installs working).
- `return 1` propagates to the existing caller loops (L88 abort for required, L95 warn for optional) — reuse that wiring, don't add new exit paths.

### Task 5: install.sh — optional version-pinning manifest  ⛔ SKIPPED at APPROVAL_GATE (2026-06-15) → follow-up `[P-pin]`. Do NOT execute. Leave `releases/latest/download/` as-is.

**File:** `install.sh` (+ a new manifest file in this repo, e.g. `versions.env`) `[needs Worker verify]` anchor #13 (no manifest exists yet).

**Tìm:** `url="https://github.com/$GH_OWNER/$bin/releases/latest/download/${bin}-${TARGET}${EXT}"` at L72. `[verified]` anchor #1.

**Thay bằng / Thêm:** drive the version from a manifest mapping `bin→tag`; fall back to `latest` if a bin is unpinned (transitional). Conceptual:

```sh
# top of install.sh: source the pin manifest if present
[ -f "$KIT_DIR/versions.env" ] && . "$KIT_DIR/versions.env"   # exports DOCTOR_VER, CLAIM_HOOKS_VER, ...
# in fetch_bin(): resolve per-bin tag (manifest var or 'latest')
ver_var=$(echo "$bin" | tr 'a-z-' 'A-Z_')_VER     # doctor→DOCTOR_VER, claude-hooks→CLAUDE_HOOKS_VER
ver=$(eval "echo \${$ver_var:-}")
if [ -n "$ver" ]; then
  url="https://github.com/$GH_OWNER/$bin/releases/download/${ver}/${bin}-${TARGET}${EXT}"
else
  url="https://github.com/$GH_OWNER/$bin/releases/latest/download/${bin}-${TARGET}${EXT}"
fi
```

**Lưu ý:**
- **This task is OPTIONAL within the phiếu** (the survey + BACKLOG frame pinning as a tradeoff: reproducible installs vs recurring bump cost). If Chủ nhà declines pinning at APPROVAL_GATE, skip Task 5 — Tasks 1-4 + 6 still close the supply-chain HIGH (checksum is the integrity gate; pinning is reproducibility). Flag this as the APPROVAL_GATE decision below.
- The manifest is a NEW file — `[needs Worker verify]` no existing one (anchor #13). Worker fills tags from each repo's current `latest` after Stage 1 re-release.
- `KIT_DIR` is the cloned-kit path (L24) — manifest ships in the kit checkout, sourced after clone. But `fetch_bin` runs BEFORE the clone (L87-98 vs clone L101-106). `[needs Worker verify]` ordering — if pinning is kept, the manifest must be fetched/sourced before the binary loop (e.g. inline the versions in install.sh itself, or curl the manifest from raw.githubusercontent first). Worker resolves ordering; this is a real edge, not hand-wave.

### Task 6: 2-OS discrimination test (real, no forging)

**File:** N/A (verification — Worker runs on a real mac AND a real linux; if only one OS available, escalate the gap, do NOT forge the other).

**Tìm:** N/A.

**Thay bằng / Thêm:** three cases, each on both OSes:
1. **Good binary** → `.sha256` matches → installs green.
2. **Corrupted binary or wrong hash** (e.g. flip a byte of `.tmp`, or point at a `.sha256` with a tampered value) → verify FAILS → required-bin ABORTS, optional-bin warn-skips.
3. **Missing `.sha256`** on a required bin → ABORTS; on an optional bin → warn + installs unverified.

**Lưu ý:**
- This is the single most error-prone part (cross-platform hash tool). The test must DISCRIMINATE: case 1 green AND case 2/3 red on BOTH a real mac and a real linux. A test where the "corrupted" case still passes = the gate is dead. `[needs Worker verify]`.
- macOS path exercises `shasum -a 256`; Linux path exercises `sha256sum` (anchor #12). Confirm BOTH probe branches actually run (not just the same OS twice).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `install.sh` | Task 3 ($SHA probe), Task 4 (verify block in `fetch_bin`), Task 5 (optional pinning) + Task 1 self-deferral comment L35-37 → "now verified" |
| `~/<repo>/.github/workflows/release.yml` ×10 (OUT-OF-TREE) | Task 1: publish `.sha256` — delegated per-repo sub-task |
| `docs/security/INVARIANTS.md` | L127-128: flip `install.sh download integrity [P071]` candidate → active/closed INV-LOCAL (now verified) |
| `docs/SETUP.md` | Install section if verify changes user-facing steps `[needs Worker verify]` anchor #6 |
| `CHANGELOG.md` | Entry for P071 |
| `versions.env` (NEW, this repo) | Task 5 ONLY if pinning approved at APPROVAL_GATE |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `bin/sos.sh` | adopt/sync flow unaffected by install-time verify |
| `templates/setup-dev.sh` | dev/cargo path — out of scope, must still work |
| `install.sh` L48-66 (platform detect) | TARGET/EXT logic unchanged; probe (Task 3) inserts AFTER it |

---

## Luật chơi (Constraints)

1. **ROLLOUT HAZARD (hard sequence).** Stage 1 (CI ×10 + re-release, assets verified-present) MUST fully land before Stage 2 (install.sh enforce). A half-rollout — install.sh enforces while some of the 6 required repos lack `.sha256` — = **fail-closed abort for every user**. Two safe paths: **(A)** ship all CI + re-tag for all 6 required FIRST, then flip install.sh (clean, recommended); **(B)** ship install.sh in a transitional **"verify-if-present" grace mode** (missing-`.sha256` on a required bin → warn + install unverified instead of abort) — this WEAKENS the gate to no-better-than-today for un-rolled bins. **Flag A-vs-B for APPROVAL_GATE** (see below).
2. **Plain sha256 ONLY.** No minisign/cosign/gpg (WORKFLOW_V2.2 §0.1 cheapest-mechanism + avoids verify-the-verifier bootstrap). If Worker is tempted to "do it properly with signatures" — STOP, that's scope creep, escalate instead.
3. **Recompute + first-field string-compare**, never `shasum -c`/`sha256sum -c` (filename mismatch across the `.tmp` rename — Task 4 Lưu ý).
4. **`sh`, not `bash`** (`install.sh` is `#!/bin/sh`, `set -eu`, L1+L21) — no arrays, no bashisms. `SHA_CMD` stays a word-split string.
5. **Real 2-OS test, no forging** (Task 6). If only one OS is reachable, escalate the missing-OS gap; do NOT mark a forged pass.
6. **Per-repo CI = delegated, one-at-a-time** (IG-09 — don't push 10 repos at once). Preserve inv-gate's drift (checkout@v5/release@v3/prerelease) and advisory-cron's 2-target matrix.
7. **Tầng-1 docs gate** (CLAUDE.md mapping — security surface = AUTO Tầng 1): Discovery Report MUST list docs updated. install.sh deferral comment + INVARIANTS L127-128 + SETUP (if user-facing change) + CHANGELOG are all in-scope.

---

## Nghiệm thu

### Automated
- [ ] `install.sh` parses clean under `sh -n install.sh` (syntax) — no bashisms introduced.
- [ ] Stage-1: each required repo's `release.yml` CI run is green on all its target OS jobs (the run prints/uploads the `.sha256` — this IS the `shasum` capability verify, anchor #11).
- [ ] `curl -fsI ".../releases/latest/download/<bin>-<target>.sha256"` → 200 for all 6 required bins × all installer-requested targets (Stage-1 gate before Stage 2).

### Manual Testing (2-OS discrimination — Task 6, real mac + real linux)
- [ ] Good binary → `.sha256` matches → install green (mac path uses `shasum -a 256`; linux path uses `sha256sum` — both branches exercised).
- [ ] Corrupted binary / tampered hash → verify FAILS → required ABORTS, optional warn-skips.
- [ ] Missing `.sha256` on required bin → ABORTS; on optional bin → warn + installs unverified.
- [ ] No hash tool present (simulate) → required install ABORTS with clear message (Task 3).

### Regression
- [ ] Optional bins (guard/vps/doc-rotate/advisory-cron) still warn-skip gracefully when their repo/target is absent (existing L94-98 behavior unchanged).
- [ ] Intel-Mac→Rosetta path (L57) + Windows Git Bash path (L53) still resolve TARGET correctly with the probe inserted.
- [ ] `sos` launcher + kit clone (L100-117) unaffected.

### Docs Gate (Tầng 1 — security surface AUTO Tầng 1)
- [ ] `install.sh` L35-37 self-deferral comment → updated to "now verified (.sha256 fetch + recompute + compare)".
- [ ] `docs/security/INVARIANTS.md` L127-128 → INV-LOCAL flipped candidate → active/closed.
- [ ] `docs/SETUP.md` — install section updated IF user-facing steps changed (else Discovery notes "N/A").
- [ ] `CHANGELOG.md` — P071 entry. (Per F13: if any tool repo is re-released with a version bump in Stage 2, that repo's Cargo.toml version must sync — note per-repo in Discovery.)

### Discovery Report
- [ ] Write to `docs/discoveries/P071.md`:
  - Anchors 6-13 — CORRECT / WRONG (file:line citations, esp. out-of-tree release.yml shape #7/#8, inv-gate/advisory-cron drift #9/#10, `$2`-in-`fetch_bin` scope, Task 5 manifest-before-loop ordering).
  - Stage 1 per-repo status: which of 10 repos re-released with `.sha256`, which deferred (private/scrub-pending).
  - Rollout path chosen (A clean vs B grace) + APPROVAL_GATE decision on Task 5 pinning.
  - Cross-repo Cargo.toml version syncs (F13) if any re-release bumped version.
  - Docs updated (or "None — cosmetic only").
  - Tier escalations (write "None" if no 2→1).
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
