# P079 round-4 Codex dogfood findings — 2026-07-23

## Verdict

**FAIL — P079 remains open; do not unblock P080.**

P078g now renders executable hook entrypoints and arms `core.hooksPath`, but a
fresh adapter install does not render the scripts those hooks delegate to. Both
required commit probes therefore remain fail-open. P078h fixes marked-actor
writes for relative/canonical paths, but `/tmp` absolute paths on macOS do not
normalize against the canonical `/private/tmp` Git root.

## Setup — PASS

```text
SOURCE_HEAD=0d458b6
REQUIRED_ANCESTOR=yes    # 2393c1b is an ancestor
codex-cli 0.145.0
cargo build --release:
Finished `release` profile [optimized] target(s) in 3.03s
test repo=/tmp/sos-kit-p079-r4.lyphX0
self marker=absent
```

The test repository is a separate fresh Git repository, not the SOS Kit clone.

`sos install --runtime codex` output:

```text
sos install --runtime codex:
  created:   18
  updated:   0
  no-op:     0
  conflicts: 0
INSTALL_EXIT=3
```

Exit 3 is the known sister-tool drift result after adapter rendering.

## Test A — render and arm real Git hooks

### A1. Hook files exist and are executable — PASS

```text
hooks/pre-commit EXISTS=yes EXECUTABLE=yes
hooks/pre-push EXISTS=yes EXECUTABLE=yes
-rwxr-xr-x hooks/pre-commit
-rwxr-xr-x hooks/pre-push
```

### A2. `core.hooksPath` — PASS

```text
HOOKS_PATH=hooks
```

### A3. `.env` commit blocked — FAIL

```text
ENV_COMMIT_EXIT=0
[7/8] Block .env* commit
  scripts/block-env-commit.sh missing — run scripts/install-hooks.sh after bootstrap
Commit allowed.
[main 4e92758] round4 env probe
 create mode 100644 .env
ENV_COMMITTED=yes
```

The rendered hook exists and executes, but delegates to a script absent from the
18-file Codex adapter plan.

### A4. product code on default branch blocked — FAIL

```text
DEFAULT_CODE_COMMIT_EXIT=0
[6/8] No-code-on-default gate
  scripts/no-code-on-default.sh missing — run scripts/install-hooks.sh after bootstrap
Commit allowed.
[main 4c77acc] round4 default code probe
 create mode 100644 src/round4_probe.rs
CODE_COMMITTED=yes
```

### A5. custom hooksPath non-clobber — PASS

Separate repo: `/tmp/sos-kit-p079-r4-custom.LlmXfH`.

```text
CUSTOM_INSTALL_EXIT=1
Error: core.hooksPath already set to 'custom-hooks'; refusing to clobber
(non-interactive install won't silently override) ...
HOOKS_PATH_AFTER=custom-hooks
HOOKS_RENDERED=yes
```

## Test B — actor-check and path normalization

All state transitions below ran through fresh real `codex exec` sessions with
the rendered project hooks loaded.

### B1. no marker: bootstrap and first approval — PASS

Bootstrap:

```text
hook: PreToolUse (x4)
hook: PreToolUse Completed (x4)
patch: completed
-version=
+version=V1
 approved_version=
```

First approval transition:

```text
hook: PreToolUse (x4)
hook: PreToolUse Completed (x4)
patch: completed
-version=V1
-approved_version=
+version=V2
+approved_version=V2
```

### B2. manual marker: approved V3/V3 advanced to V4/V4 — PASS (blocked)

```text
WORKER_MARKER=present
version=V3
approved_version=V3
Command blocked by PreToolUse hook:
BLOCKED: control-plane write by marked actor ... no version-match escape hatch.
Blocked: ... `.sos-state/worker-active` or `.sos-state/architect-active` is present.
```

State remained `V3/V3` after this relative-path attempt.

### B3. manual marker: arbitrary ticket-state write — PASS (blocked)

Patch attempted to append `round4_probe=must_not_write`:

```text
Command blocked by PreToolUse hook:
BLOCKED: control-plane write by marked actor ...
Blocked.
```

### B4. absolute path behaves like relative — FAIL for `/tmp` symlink path

The physical Git root on macOS is canonicalized to `/private/tmp/...`:

```text
PWD_PHYSICAL=/private/tmp/sos-kit-p079-r4.lyphX0
```

With `worker-active` present, the valid absolute path using `/tmp/...` was not
recognized as the state file and the forbidden advance completed:

```text
*** Update File: /tmp/sos-kit-p079-r4.lyphX0/.sos-state/ticket-state.env
-version=V3
-approved_version=V3
+version=V4
+approved_version=V4
patch: completed
MARKER_NOW=present
```

The canonical equivalent did block correctly:

```text
*** Update File: /private/tmp/sos-kit-p079-r4.lyphX0/.sos-state/ticket-state.env
BLOCKED: control-plane write by marked actor ...
```

The opposite/main-thread direction also reproduced the false-block. Starting
from unapproved `V5/empty`, no marker, an approval patch using `/tmp/...` was
rejected instead of receiving the state-file-only exemption:

```text
MARKER_AFTER_CLEANUP=absent
version=V5
approved_version=
*** Update File: /tmp/sos-kit-p079-r4.lyphX0/.sos-state/ticket-state.env
BLOCKED: ticket version 'V5' is not approved (approved_version='').
```

Thus the fix handles only an absolute spelling sharing the exact canonical
`REPO_ROOT` prefix. It does not handle equivalent absolute paths through a
filesystem symlink.

### B5. state + code bundle — PASS (blocked)

With `worker-active` present, the patch attempted to advance the state and add
`src/round4_bundle_probe.rs`:

```text
Command blocked by PreToolUse hook:
BLOCKED: control-plane write by marked actor ...
BUNDLE_FILE=absent
```

## New/remaining gaps

1. **Hook dependency closure is incomplete.** Rendering only
   `hooks/pre-commit`/`hooks/pre-push` is not sufficient. The fresh install must
   also render the required delegated guards, at minimum
   `scripts/block-env-commit.sh` and `scripts/no-code-on-default.sh`, or make the
   hook entrypoints enforce those two acceptance checks without missing-script
   fail-open behavior.
2. **Absolute normalization is lexical, not filesystem-equivalent.** On macOS,
   `/tmp/project/...` and `/private/tmp/project/...` name the same file, but the
   guard strips only the literal canonical `REPO_ROOT/` prefix. Normalize both
   the candidate path and repo root consistently (including symlink resolution)
   before actor/exemption comparisons.

The known custom-role marker non-dispatch caveat was not treated as a round-4
regression, per the checklist. Manual-marker actor logic is fixed for relative
and canonical-absolute paths.
