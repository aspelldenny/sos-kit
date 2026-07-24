# DOGFOOD LOG — P071 Task 6: install.sh checksum verify, 3-OS discrimination

> Goal: confirm the live `curl|sh` installer's checksum-verify works on ALL 3 platforms.
> mac is DONE (Worker, 4/4 on the sha256sum path). Linux + Windows pending Sep's dogfood.
> Fill in results per OS. Friction/bugs route back to sos-kit as kit-bugs (like the mon-tamer SOS_KIT_FEEDBACK pattern).
>
> install.sh: probe = `sha256sum` primary / `shasum -a 256` fallback. Verify block in `fetch_bin()`
> recomputes the downloaded `.tmp` hash and first-field-compares to the published `.sha256`.

---

## Per-OS checklist (run the SAME 6 on each)

For each platform, record: PASS / FAIL + 1 line of evidence.

| # | Check | How | mac | Linux | Win (Git Bash) |
|---|-------|-----|-----|-------|----------------|
| 0 | Which hash tool does the probe pick? | `command -v sha256sum; command -v shasum` | sha256sum (`/sbin`) | sha256sum (`/usr/bin`, GNU 9.4) | _ |
| A | Good binary + real `.sha256` -> install verifies GREEN | real `curl -fsSL .../install.sh \| sh` | PASS | PASS | _ |
| B | Corrupted binary/hash -> verify FAILS, required ABORTS | flip a byte of the `.tmp` or tamper the `.sha256` | PASS | PASS | _ |
| C | Missing `.sha256`: required ABORTS / optional WARN-skips | point at a nonexistent `.sha256` | PASS | PASS | _ |
| D | Full e2e `curl\|sh` completes (6 core installed + verified) | run the real public one-liner from GitHub | (mac path) | PASS | _ |
| E | OS-specific watch (see below) | -- | n/a | PASS | _ |

### OS-specific watch (check E)
- **Linux**: line-endings ok (no CRLF breakage); `sha256sum` used; `~/.local/bin` on PATH or the warning fires.
- **Windows Git Bash**: `.exe` suffix handled on download + chmod; path separators (the `ship` to_string_lossy backslash class); CRLF; **advisory-cron warn-skips cleanly** (it has no Windows target); which sha tool Git Bash picks.

### Optional — the one branch no default OS hits
- [ ] **`shasum` fallback**: temporarily mask `sha256sum` (e.g. `PATH` without `/sbin`/`/usr/bin/sha256sum`, or alias it away) and re-run case A -> confirm the `shasum -a 256` branch verifies correctly. (All 3 OS default to `sha256sum`, so this branch is otherwise untested.)

---

## Results

### Linux (distro: WSL2 Ubuntu x86_64 kernel 6.6.87.2, date: 2026-07-24)
- Probe picked: `/usr/bin/sha256sum` (GNU coreutils 9.4; `shasum` also present, lower priority)
- A / B / C / D / E: PASS / PASS / PASS / PASS / PASS
  - A+D: real public `curl|sh` → 7× `✓ sha256 verified` (6 required + sos-bin), optional guard/vps/doc-rotate warn-skip, kit dir untouched, wrapper OK.
  - B: curl PATH-shim tampered doctor's `.sha256` → `✗ CHECKSUM MISMATCH … ABORTING` exit 1, no binary left.
  - C: shim 404'd doctor's `.sha256` → `✗ No .sha256 published for required bin … ABORTING` exit 1. Optional-class proven live: advisory-cron has NO published `.sha256` → warn-skip (route this back: publish it — finding L2).
  - E: no CRLF breakage; `sha256sum` used; PATH warning fires.
- Findings: full round log `docs/retro/DOGFOOD_LINUX_2026-07-24.md` (L1–L5; kit-bugs L3 sos-new trust-baseline, L5 new/sync spine drift; release-bug L2 advisory-cron .sha256).

### Windows Git Bash (version: ___, date: ___)
- Probe picked: ___
- A / B / C / D / E: ___
- Findings:

### shasum-fallback (any OS):
- Result: **PASS on Linux 2026-07-24** — PATH stripped of `sha256sum` (symlink-farm minus the binary) → full case-A install re-run verifies green 7/7 via the `shasum -a 256` branch. The one branch no default OS hits is now proven.

---

## Verdict
- [ ] All 3 OS: case A green AND case B/C red (discrimination holds) -> Task 6 CLOSED, P071 fully accepted.
- Friction to route back to sos-kit:
- Friction to route to a specific tool repo (e.g. a per-binary release issue):
