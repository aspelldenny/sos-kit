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
| 0 | Which hash tool does the probe pick? | `command -v sha256sum; command -v shasum` | sha256sum (`/sbin`) | _ | _ |
| A | Good binary + real `.sha256` -> install verifies GREEN | real `curl -fsSL .../install.sh \| sh` | PASS | _ | _ |
| B | Corrupted binary/hash -> verify FAILS, required ABORTS | flip a byte of the `.tmp` or tamper the `.sha256` | PASS | _ | _ |
| C | Missing `.sha256`: required ABORTS / optional WARN-skips | point at a nonexistent `.sha256` | PASS | _ | _ |
| D | Full e2e `curl\|sh` completes (6 core installed + verified) | run the real public one-liner from GitHub | (mac path) | _ | _ |
| E | OS-specific watch (see below) | -- | n/a | _ | _ |

### OS-specific watch (check E)
- **Linux**: line-endings ok (no CRLF breakage); `sha256sum` used; `~/.local/bin` on PATH or the warning fires.
- **Windows Git Bash**: `.exe` suffix handled on download + chmod; path separators (the `ship` to_string_lossy backslash class); CRLF; **advisory-cron warn-skips cleanly** (it has no Windows target); which sha tool Git Bash picks.

### Optional — the one branch no default OS hits
- [ ] **`shasum` fallback**: temporarily mask `sha256sum` (e.g. `PATH` without `/sbin`/`/usr/bin/sha256sum`, or alias it away) and re-run case A -> confirm the `shasum -a 256` branch verifies correctly. (All 3 OS default to `sha256sum`, so this branch is otherwise untested.)

---

## Results

### Linux (distro: ___, date: ___)
- Probe picked: ___
- A / B / C / D / E: ___
- Findings:

### Windows Git Bash (version: ___, date: ___)
- Probe picked: ___
- A / B / C / D / E: ___
- Findings:

### shasum-fallback (any OS):
- Result:

---

## Verdict
- [ ] All 3 OS: case A green AND case B/C red (discrimination holds) -> Task 6 CLOSED, P071 fully accepted.
- Friction to route back to sos-kit:
- Friction to route to a specific tool repo (e.g. a per-binary release issue):
