# DOGFOOD — Linux full-surface round (WSL2 Ubuntu, 2026-07-24)

> **VERDICT: PASS with findings — 1 test-harness FAIL (L1), 1 HIGH greenfield UX bug (L3), 3 LOW.**
> Closes: **P071 Task 6 Linux row** (all 6 checks + the never-tested shasum-fallback branch) and **P080 E2 (Linux, previously DEFERRED per Sếp)**.
> First-ever run of the kit + Rust workspace on Linux — all prior dogfood was macOS.

## Environment

- WSL2 Ubuntu x86_64 (kernel 6.6.87.2-microsoft-standard), ext4.
- cargo/rustc 1.95.0, node v24.15.0 (nvm) + npm 11.12.1, git, jq. No `rg` (validates grep fallback paths).
- Kit: `~/sos-kit` @ `2debb4c` (main, clean). Test repos under `~/dogfood/` (per P079 rule: never dogfood inside the kit checkout).

## Summary table

| Track | Surface | Result |
|---|---|---|
| 0 | `cargo build --release` + `cargo test --workspace` | build clean 8.14s; **101 passed / 1 FAILED** → **L1** |
| 1 | `install.sh` real `curl \| sh` e2e | PASS — 6 required verified+versions match manifest, optional warn-skip, kit dir untouched, wrapper + `--version` forward OK → **L2** |
| 1b | P071 discrimination B/C + shasum fallback | PASS — tampered hash ABORT exit 1; missing required `.sha256` ABORT exit 1; **shasum-fallback full install green (first time this branch ever ran)** |
| 2 | `sos new` greenfield (python) | PASS — full tree, hooks armed, `verify-setup: CONNECTED` → **L3, L4** |
| 3 | `sos adopt` brownfield | PASS — non-clobber, F09 declines arming on custom `core.hooksPath`, idempotent rerun, symlink-root probe (P078j class) clean |
| 4 | pre-commit 8-phase matrix | PASS — [6/8]+[7/8] block correctly; **fail-CLOSED holds when guard scripts deleted (P080-D1 does NOT regress)**; trust-gate additionally catches guard deletion (defense-in-depth) |
| 5 | trust gate (scratch clone) | PASS — clean/tamper/rebaseline/ZWSP(unicode-gate via **GNU grep fallback**, no rg)/untracked-warn all correct; `sha256sum` selected |
| 6 | `npm install -g sos-kit` | PASS — pinned-tag postinstall, delegate → `sos 0.1.0` |
| 7 | `sos sync` + `sos map` smoke | PASS — 4-outcome discrimination correct (user-modified file FLAGGED+kept) → **L5** |
| 8 | `sos install --runtime codex/claude --dry-run` + `sos tools status` | PASS — codex renders 18 artifacts zero-mutation; claude empty-plan (known: physical render = future ticket); tools status table correct, optional-MISSING warn-only |

## Findings

- **L1 — `parity_adopt_enforced` FAILS on Linux (test-harness portability, MEDIUM).**
  `crates/sos-cli/tests/parity.rs:577` — adopt stdout vs `adopt.golden`: two ADDED lines swap order
  (`templates/BACKLOG_template.md` ↔ `templates/claude-settings.local.json`). Golden was captured on
  macOS/APFS with unsorted traversal; ext4 readdir differs → `sos adopt` report order is
  filesystem-dependent. Exactly the gap self-declared in `crates/sos-cli/tests/README.md`.
  Fix direction: sort the copy/report traversal in both bash oracle + Rust (or sort-normalize in
  harness before compare). `map`/`new`/`sync` parity PASS on Linux — only adopt has the
  order-sensitive branch.

- **L3 — freshly `sos new`-ed repo cannot make its FIRST commit (greenfield UX, HIGH).**
  `[8/8]` trust-gate is fail-closed and `sos new` neither seeds `.sos-trust-baseline` nor mentions
  `scripts/trust-gate.sh rebaseline` in its next-steps → every commit blocked out of the box
  (`BLOCKED: trust-gate: .sos-trust-baseline not found`). Workaround verified: rebaseline (20
  surfaces) + `git add` → first commit passes all 8 phases. Fix direction: seed the baseline during
  born-wire (mirror of the adopt/install arm-hooks step) or add to next-steps + a doctor
  verify-setup joint.

- **L2 — `advisory-cron` Linux release asset has no `.sha256` companion (release infra, LOW).**
  Installer correctly warn-skips verify (optional class) but installs unverified. Publish the
  `.sha256` for the linux asset.

- **L4 — `.sos-stack.toml` written with `sos_kit_version = "P040"` (cosmetic, LOW).**
  Stale constant; kit is v0.1.0.

- **L5 — `sos new` and `sos sync` spine sets drift (LOW/MED).**
  `sos sync` on a repo created by `sos new` the same day ADDED `docs/ORCHESTRATION.md` —
  the two commands disagree on what the spine contains. Align the file lists (single source).

- **Obs (no action):** first-commit edge on `[6/8]` reproduces on Linux exactly as documented
  (WARNING + allow — pre-existing known limitation). `sos map` on a repo with only a root
  `main.py` and no manifest → honest `coverage_unknown` stub. `sos install` targets cwd while
  `new/adopt/sync/map` take a positional path — minor CLI inconsistency, caused one usage error
  during this round. `codex` visible in WSL was only the Windows shim via `/mnt/c` → enforcement
  legs N/A this round (dry-run only, per P079 caveat convention).

## P071 Task 6 — Linux row evidence (also recorded in `DOGFOOD_P071-task6_3OS_2026-06-15.md`)

- Check 0: probe picks `/usr/bin/sha256sum` (GNU coreutils 9.4); `shasum` also present but lower priority.
- A: real `curl|sh` → 7× `✓ sha256 verified` (6 required + sos-bin). PASS.
- B: curl-shim tampers doctor's `.sha256` → `✗ CHECKSUM MISMATCH … ABORTING` + `Download FAILED … fail-closed`, exit 1, no binary left. PASS.
- C: curl-shim 404s doctor's `.sha256` → `✗ No .sha256 published for required bin … ABORTING`, exit 1. PASS. (Optional-class C proven live by advisory-cron warn-skip.)
- D: full e2e completes, `sos tools status` all required OK. PASS.
- E (Linux watch): no CRLF breakage (`.gitattributes` holding); `sha256sum` used; PATH warning fires when relevant. PASS.
- Optional shasum-fallback (no OS defaults to it): PATH stripped of `sha256sum` → full install re-run verifies green 7/7 via `shasum -a 256`. PASS — branch now proven.

## Method notes

- Harness: Windows host → `wsl.exe -d Ubuntu bash -l <script>`; scripts written to `~/dogfood/scripts/`
  (3-layer PowerShell→wsl→bash quoting is unreliable; script files + `bash -l` for `~/.cargo/bin` PATH).
- Trust-gate destructive tests ran in a scratch clone (`~/dogfood/kit-scratch`), kit checkout untouched.
- Discrimination B/C used local `install.sh` (identical to remote main @2debb4c) + curl PATH-shims,
  sandboxed via `SOS_BIN_DIR`; real `~/.local/bin` untouched by B/C/fallback runs.
