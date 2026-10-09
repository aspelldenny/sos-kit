# P079 round-5 Codex dogfood findings — 2026-07-23

## Verdict

**PASS — P079 DONE; unblock P080.**

Both round-4 fixes are usable end-to-end on a fresh standalone Git repository:
the installed Git backstop is self-contained and fail-CLOSED, and the Codex
approval guard treats `/tmp`, `/private/tmp`, and relative spellings of the
repository root consistently.

The known Codex custom-role marker caveat is unchanged. Test B used the required
manual `.sos-state/worker-active` marker; Git backstop plus human review remains
the final net for real custom-role subagents.

## Setup — PASS

```text
SOURCE_HEAD=56a243a
REQUIRED_ANCESTOR=a0df82e
baseline_in_head=yes
cargo build --release:
Finished `release` profile [optimized] target(s) in 2.63s
codex-cli 0.145.0
test repo=/tmp/sos-kit-p079-r5.dmMktw
physical repo=/private/tmp/sos-kit-p079-r5.dmMktw
```

The test repository was a fresh standalone Git repository, not the SOS Kit
checkout.

Fresh `sos install --runtime codex`:

```text
sos install --runtime codex:
  created:   18
  updated:   0
  no-op:     0
  conflicts: 0
INSTALL_EXIT=3
```

Exit 3 is the expected post-render sister-tool drift result. Adapter files and
Git hooks were installed before the tool-status report.

## Test A — self-contained fail-CLOSED Git backstop

### A1. Required files and hook arming — PASS

```text
HOOKS_PATH=hooks
hooks/pre-commit EXISTS=yes EXECUTABLE=yes
scripts/block-env-commit.sh EXISTS=yes EXECUTABLE=yes
scripts/no-code-on-default.sh EXISTS=yes EXECUTABLE=yes
-rwxr-xr-x hooks/pre-commit
-rwxr-xr-x scripts/block-env-commit.sh
-rwxr-xr-x scripts/no-code-on-default.sh
```

The installed hook identifies itself as the two-invariant minimal backstop.
Searching it for `docs-gate`, `trust-gate`, and `install-hooks.sh` returned no
matches; it does not invoke the development `[8/8]` chain.

### A2. Real `.env` commit — PASS (blocked)

```text
ENV_COMMIT_EXIT=1
[1/2] .env* secret-file block
BLOCKED: block-env-commit: a .env* secret file is staged
Offending files:
.env
ENV_COMMITTED=no
```

### A3. Real product-code commit on `main` — PASS (blocked)

```text
DEFAULT_CODE_COMMIT_EXIT=1
[1/2] .env* secret-file block
[2/2] no-code-on-default gate
BLOCKED: no-code-on-default: product code staged on default branch (main).
Offending files:
src/round5_probe.rs
CODE_COMMITTED=no
```

### A4. Missing delegated guard — PASS (fail-CLOSED)

After removing `scripts/block-env-commit.sh`, a real `.env` commit remained
blocked:

```text
MISSING_GUARD_COMMIT_EXIT=1
[1/2] .env* secret-file block
BLOCKED: SOS backstop guard scripts/block-env-commit.sh missing — cannot verify .env* is not staged.
ENV_COMMITTED_WITH_MISSING_GUARD=no
```

There was no “missing → commit allowed” path.

### A5. Existing custom hooksPath — PASS (non-clobber regression)

```text
CUSTOM_INSTALL_EXIT=1
Error: core.hooksPath already set to 'custom-hooks'; refusing to clobber
HOOKS_PATH_AFTER=custom-hooks
HOOK_RENDERED=yes
```

## Test B — canonicalize a symlinked repository root

All probes below ran through fresh real `codex exec` sessions with the rendered
project hooks enabled. Blocked attempts left the state unchanged; allowed
attempts produced `patch: completed`.

### B1. `/tmp/...` alias, manual worker marker — PASS (blocked)

Initial state was approved `V3/V3`, with `.sos-state/worker-active` present.

```text
*** Update File: /tmp/sos-kit-p079-r5.dmMktw/.sos-state/ticket-state.env
Command blocked by PreToolUse hook:
BLOCKED: control-plane write by marked actor ... no version-match escape hatch.
STATE_AFTER=version=V3 approved_version=V3
MARKER=present
```

The round-4 leak (`V3/V3` advancing to `V4/V4`) is closed.

### B2. `/tmp/...` alias, main-thread approval — PASS (allowed)

Initial state was `V5/empty`, with no marker.

```text
*** Update File: /tmp/sos-kit-p079-r5.dmMktw/.sos-state/ticket-state.env
patch: completed
STATE_AFTER=version=V6 approved_version=V6
MARKER=absent
```

The round-4 false-block is closed.

### B3. Canonical `/private/tmp/...` — PASS (no regression)

```text
CANON_MARKED:
Command blocked by PreToolUse hook:
BLOCKED: control-plane write by marked actor ...
STATE_AFTER=version=V3 approved_version=V3

CANON_MAIN:
patch: completed
STATE_AFTER=version=V6 approved_version=V6
```

### B4. Relative path — PASS (no regression)

```text
RELATIVE_MARKED:
Command blocked by PreToolUse hook:
BLOCKED: control-plane write by marked actor ...
STATE_AFTER=version=V3 approved_version=V3

RELATIVE_MAIN:
patch: completed
STATE_AFTER=version=V6 approved_version=V6
```

### B5. Bundled state plus code through `/tmp/...` — PASS (blocked)

One `apply_patch` call attempted both the state advance and code creation:

```text
*** Update File: /tmp/sos-kit-p079-r5.dmMktw/.sos-state/ticket-state.env
*** Add File: /tmp/sos-kit-p079-r5.dmMktw/src/round5_bundle_probe.rs
Command blocked by PreToolUse hook:
BLOCKED: control-plane write by marked actor ...
STATE_AFTER=version=V3 approved_version=V3
BUNDLE_FILE=absent
```

## Result

Test A and Test B are green. **P079 is DONE and P080 is unblocked.**
