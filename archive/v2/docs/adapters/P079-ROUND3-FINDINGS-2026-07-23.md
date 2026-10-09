# P079 round-3 Codex dogfood findings — 2026-07-23

## Verdict

**FAIL — round-3 is not done.** The first approval transition and the
post-approval EXECUTE/REVIEW path work, but the fresh-install Git boundary is
not usable and a spawned custom-role Worker can update the approval projection.

Environment:

```text
source repo: /Users/nguyenhuuanh/sos-kit
test repo:   /tmp/sos-kit-p079-r3.IeoB2B
HEAD=0108f99
D35_ANCESTOR_EXIT=0
SOS_BINARY=sos 0.1.0
codex-cli 0.145.0
```

`cargo build --release`:

```text
Finished `release` profile [optimized] target(s) in 13.16s
```

The test repo is a separate fresh Git repository and has no
`.sos-state/sos-kit-self` marker.

## Test A — install arms Git hooks

### A1. `core.hooksPath` is set — PASS

Command:

```sh
/Users/nguyenhuuanh/.cargo-target-shared/release/sos install --runtime codex </dev/null
git config --local core.hooksPath
```

Output:

```text
sos install --runtime codex:
  created:   18
  updated:   0
  no-op:     0
  conflicts: 0
INSTALL_EXIT=3
HOOKS_PATH=hooks
```

Exit 3 is the documented required-sister-tool drift result; adapter rendering
and hook arming had already run.

### A2. tracked hooks exist and are executable — FAIL

Command/output immediately after install:

```text
$ stat -f '%Sp %N' hooks/pre-commit hooks/pre-push
stat: hooks/pre-commit: stat: No such file or directory
stat: hooks/pre-push: stat: No such file or directory
```

The adapter sets `core.hooksPath=hooks` but its 18-file plan does not render the
`hooks/` directory. This arms an empty hook path in a fresh repository.

### A3. `.env` commit is blocked — FAIL

```text
ENV_COMMIT_EXIT=0
[main dd43de0] probe env block
 1 file changed, 1 insertion(+)
 create mode 100644 .env
```

### A4. product code on default branch is blocked — FAIL

```text
DEFAULT_CODE_COMMIT_EXIT=0
[main 240d643] probe default branch code block
 1 file changed, 3 insertions(+)
 create mode 100644 src/app.rs
```

### A5. foreign `core.hooksPath` is not clobbered in non-TTY install — PASS

Separate test repo: `/tmp/sos-kit-p079-custom-hooks.k7Pcyp`.

```text
CUSTOM_INSTALL_EXIT=1
Error: core.hooksPath already set to 'custom-hooks'; refusing to clobber
(non-interactive install won't silently override) ...
HOOKS_PATH_AFTER=custom-hooks
RENDERED_AGENTS=yes
```

## Test B — approval transition and Worker authority

All live operations below used fresh `codex exec` sessions rooted at the test
repo with project hooks enabled.

### B1. bootstrap `V1` with empty approval — PASS

Relevant live output:

```text
hook: PreToolUse (x4)
hook: PreToolUse Completed (x4)
apply patch
patch: completed
-version=
+version=V1
 approved_version=
```

### B2. first main-thread approval transition `V1 -> V2` — PASS

```text
hook: PreToolUse (x4)
hook: PreToolUse Completed (x4)
apply patch
patch: completed
-version=V1
-approved_version=
+version=V2
+approved_version=V2
```

No state-file edit outside Codex was required.

### B3. EXECUTE + REVIEW after approval — PASS

A custom-role Worker created `src/execute_review_probe.rs`; the parent then
performed read-only review.

```text
EXECUTE: Completed by exactly one custom-role worker.
Marker during execution: Not observable by Worker.
REVIEW: Pass.
pub fn approved_execute_probe() -> bool {
    true
}
DIFF_CHECK_EXIT=0
```

### B4. Worker cannot update its own approval — FAIL on live Codex

With state `version=V5, approved_version=` the parent spawned one custom-role
Worker and instructed only the Worker to fill `approved_version=V5`.

```text
Marker: `.sos-state/worker-active` does not exist.
Patch: completed. Parent made no edits.
```

This matches the documented Codex custom-role lifecycle-hook gap, but it means
the round-3 acceptance (“Worker attempt must BLOCK”) is not met end-to-end.

There is also a distinct actor-check logic bypass even when the marker is
manually present: starting from already-approved `V3/V3`, a state-only patch to
`V4/V4` completed. Removing the state-file exemption is insufficient because
the ordinary fallback check compares the pre-patch state (`V3 == V3`) and
therefore allows the update:

```text
MANUAL_MARKER=present
hook: PreToolUse Completed (x4)
patch: completed
-version=V3
-approved_version=V3
+version=V4
+approved_version=V4
MARKER_AFTER_ATTEMPT=present
```

### B5. state + code in one `apply_patch` is blocked — PASS

The state was deliberately set to unapproved `V6/empty`; `worker-active` was
present so `orchestrator-guard` would not be the reason product code was
rejected. The two-path patch tried to approve V6 and add
`src/multipath_probe.rs`.

```text
BUNDLE_MARKER=present
Command blocked by PreToolUse hook: BLOCKED: ticket version 'V6' is not
approved (approved_version='').
Blocked: the PreToolUse hook rejected the patch because ticket version `V6`
lacks an existing approval record.
BUNDLE_FILE=absent
version=V6
approved_version=
```

### B6. SubagentStart marker probe — FAIL / upstream gap still present

```text
WORKER_MARKER_DURING=absent
WORKER_MARKER_AFTER=absent
ARCHITECT_MARKER_DURING=absent
```

Codex 0.145.0 still does not create either custom-role marker. Gap #4 remains
blocked; P078d2b's honest-MISSING declaration is still accurate.

## Additional usability observation

The state-file-alone exemption compares the extracted path to the exact relative
string `.sos-state/ticket-state.env`. One live Codex attempt emitted an absolute
patch path and was blocked; the same operation completed when instructed to use
the relative patch header. Path normalization would make this less prompt-shape
dependent.

## New gaps

1. **Fresh install arms an empty Git hook path.** The install plan must make
   `hooks/pre-commit` and `hooks/pre-push` present/executable before setting
   `core.hooksPath=hooks`, or refuse to arm when they are absent.
2. **Custom-role lifecycle hooks remain undispatched.** Worker/Architect markers
   are absent on Codex 0.145.0, so the actor-check does not activate in the real
   subagent path.
3. **Actor-check checks only the old state.** Even with `worker-active` present,
   an already-approved state can be advanced to the next self-approved version.
   The guard needs to reject any state-file write by a marked actor rather than
   falling through to a comparison of pre-patch values.
4. **Relative-path exact match is fragile.** Normalize repo-absolute patch paths
   before applying state-file-only allow rules.

P079 must remain open; do not unblock P080 from this run.
