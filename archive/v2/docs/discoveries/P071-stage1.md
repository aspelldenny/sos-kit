# P071 Stage 1 Fan-Out Discovery Report

**Date:** 2026-06-15
**Phiếu:** P071-install-checksum (Stage 1 FAN-OUT — 5 REQUIRED repos)
**Worker:** Thợ (Sonnet 4.6)

---

## Summary

Applied the doctor-validated `.sha256` publish pattern (sha256sum/shasum conditional + draft/publish job split) to the 5 REQUIRED repos: claude-hooks, docs-gate, ship, advisory-inbox, inv-gate. All 5 CI runs completed GREEN. All 5 releases now carry `.sha256` for all 3 targets.

---

## Assumptions in Phiếu — CORRECT

- All 5 repos had identical release.yml structure (BIN=<name>, ASSET var, softprops/action-gh-release, files scalar). Confirmed before editing.
- inv-gate drift confirmed exactly as phiếu documented: checkout@v5, action-gh-release@v3, prerelease flag.
- No `no-code-on-default` gate blocks `.github/workflows/*.yml` — only `.rs/.ts/.tsx/.js/.jsx/.py/.go/.swift/.pbxproj` extensions blocked.
- WIP dirty files in 3 repos (docs-gate: `.gitignore` M; advisory-inbox: `docs/runlog/` untracked; inv-gate: `.sos-sync-incoming/` untracked) were NOT staged in any commit. Confirmed via `git status --short` before and after.

## Assumptions in Phiếu — WRONG / ADAPTED

- **docs-gate has no CHANGELOG** — phiếu assumed all repos might need CHANGELOG update. docs-gate has no `CHANGELOG.md`; no docs-gate pre-commit hook is active (`core.hooksPath` not set, `.git/hooks/pre-commit.sample` only). Tầng 2 self-adapt: no CHANGELOG needed for docs-gate.
- **inv-gate requires CHANGELOG dated today** — inv-gate has active pre-commit hook (`core.hooksPath = hooks`) with docs-gate check requiring changelog_max_age_days = 1. The `[Unreleased]` entry was dated 2026-06-11 (too old). Added v0.1.1 entry dated 2026-06-15 above the `[Unreleased]` block, leaving WIP content untouched.
- **ship and advisory-inbox have no pre-commit hooks** — no CHANGELOG gate fired. Committed directly without CHANGELOG updates.
- **docs-gate and ship have pre-existing CI workflow failures** (cargo fmt / unrelated lint). These pre-existed before this phiếu — confirmed by checking prior run history (v0.1.0 also had CI workflow failure). NOT caused by P071 changes. The `release` workflow (the one that matters for .sha256 assets) was GREEN for both.

## Per-Repo Results

| Repo | Branch | New version | CI (release) | .sha256 all 3 targets | Dirty WIP untouched | Notes |
|------|--------|-------------|--------------|----------------------|---------------------|-------|
| claude-hooks | main | 0.9.1 → 0.9.2 | GREEN (run 27538526417) | aarch64-apple-darwin, x86_64-pc-windows-msvc.exe, x86_64-unknown-linux-gnu | N/A (clean) | CHANGELOG added; all 7 pre-commit checks PASS |
| docs-gate | main | 0.1.0 → 0.1.1 | GREEN (run 27538581921) | aarch64-apple-darwin, x86_64-pc-windows-msvc.exe, x86_64-unknown-linux-gnu | .gitignore M preserved | No pre-commit hook; no CHANGELOG (repo has none); CI workflow failure pre-existing (cargo fmt) |
| ship | main | 0.1.0 → 0.1.1 | GREEN (run 27538621927) | aarch64-apple-darwin, x86_64-pc-windows-msvc.exe, x86_64-unknown-linux-gnu | N/A (clean) | No pre-commit hook; CI workflow failure pre-existing |
| advisory-inbox | main | 0.1.0 → 0.1.1 | GREEN (run 27538658701) | aarch64-apple-darwin, x86_64-pc-windows-msvc.exe, x86_64-unknown-linux-gnu | docs/runlog/ preserved | No pre-commit hook |
| inv-gate | main | 0.1.0 → 0.1.1 | GREEN (run 27538731987) | aarch64-apple-darwin, x86_64-pc-windows-msvc.exe, x86_64-unknown-linux-gnu | .sos-sync-incoming/ preserved | Preserved @v5/@v3/prerelease; added CHANGELOG entry; all 7 pre-commit checks PASS |

## inv-gate @v3 Behavior (gotcha-2 adaptation)

The phiếu asked to verify whether `softprops/action-gh-release@v3` has the same immutable-release race as @v2. Applied the same draft/publish fix (draft: true on build matrix + publish job via github-script@v7). CI ran and all 3 build jobs created draft release independently, then publish job flipped it live — pattern works identically on @v3. No immutable-release conflict observed. The `prerelease:` field is preserved on the build step (before draft flip) so RC tags still get `prerelease: true`.

## FLEET-NODE20 Note

All 5 repos received Node.js 20 deprecation warnings in CI annotations. These are warnings, not failures — GitHub will force Node24 default starting June 16th, 2026. FLEET-NODE20 bump is a SEPARATE phiếu (confirmed by Sếp). The warning is expected and pre-existing; Stage 1 `.sha256` assets still published correctly.

## Edge Cases / Limitations Found

- **docs-gate `CI` workflow**: has cargo fmt failure pre-existing. Bumping Cargo.toml version may have triggered it to run again but it was already failing at v0.1.0. Does not affect release assets. FLEET-CODE quality follow-up needed for docs-gate separately.
- **ship `CI` workflow**: same pre-existing failure pattern. The `release` workflow is the install.sh-facing one and was GREEN.
- **advisory-inbox `CI` on main (not tag)**: completed success — its CI workflow tests pass. Only the tag-triggered `release` workflow was needed for assets.

## Scope Expansions

None — exactly the 5 REQUIRED repos changed. OPTIONAL repos (guard, vps, doc-rotate, advisory-cron) NOT touched per Sếp instruction. Stage 2 (install.sh enforce) NOT touched per phiếu sequencing: must verify all 6 REQUIRED bins before flipping.

## Docs Updated

- This per-phiếu Discovery Report: `docs/discoveries/P071-stage1.md`
- `docs/DISCOVERIES.md` index: 1-line entry appended

Tầng 1 docs gate: Stage 1 is OUT-OF-TREE CI config changes + version bumps only. No change to install.sh (Stage 2), INVARIANTS.md, SETUP.md, or CHANGELOG.md in sos-kit yet — those are Stage 2 tasks (pending Stage-2 phiếu after .sha256 assets verified on all 6 REQUIRED repos). Stage-2 docs updates deferred per phiếu sequencing.

## Tier Escalations

None. All changes were CI config + version bumps — Tầng 2 by scope (no schema/API/auth boundary touched). inv-gate CHANGELOG addition to satisfy its own docs-gate hook = Tầng 2 self-adapt.

## For Stage 2 / install.sh Dogfood

- All 6 REQUIRED bins now have `.sha256` on their `latest` release for all 3 targets. Stage-2 gate condition met.
- doctor was already verified (P071 pilot). 5 new repos now carry .sha256. Stage 2 can proceed.
- Recommend verifying via `curl -fsI` for each bin+target before flipping install.sh verify block active.
- FLEET-NODE20 should be done before or alongside any future release cycle — June 16th is tomorrow, warnings become errors.
