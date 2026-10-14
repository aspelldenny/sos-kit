# Field report — `codex exec` hangs on open stdin (TurnSigil dogfood, 09/10/2026)

Reported by TurnSigil's Quản đốc (Claude Code, Opus) for Chủ nhà to hand to the sos-kit session. Not committed: the fix belongs in a kit PR.

## What happened

TurnSigil runs Người soát on Codex (`gpt-6.1-sol`, effort high, read-only) from the Claude Code main session, as its AGENTS.md prescribes:

```
codex exec -m gpt-6.1-sol -c model_reasoning_effort=high -s read-only -C <repo> -o <out.md> "<brief>" > <log> 2>&1
```

Run with Bash `run_in_background: true`, the process printed one line and stopped:

```
Reading additional input from stdin...
```

It stayed at 0% CPU with no output for **40 minutes** (17:49 → 18:28) until Chủ nhà asked whether Codex had frozen. The model was healthy: a ping with stdin closed answered in 13 s, and the same review rerun with `< /dev/null` started at once.

Cause: when a prompt argument is given and stdin is not a TTY, `codex exec` (0.162.0) also reads stdin to append to the prompt. A background shell leaves stdin open and never sends EOF, so Codex waits forever. An earlier review in the same session happened to run with stdin closed and worked, which hid the problem.

The orchestrator compounded it: it waited for the completion notification and never checked the log or CPU, so a hang looked the same as a slow review.

## Where the kit is exposed

- **Prose, not scripts.** The kit does not ship a Codex call for Người soát or Kiến trúc sư; each app's AGENTS.md writes its own `codex exec` line (TurnSigil had two, both without stdin closed). Every app that copies this pattern can hit the hang.
- **Not affected:** `harness-lite/scripts/advise` passes the prompt through `input=` (stdin closed after write) and has `timeout=240`.

## Suggested fixes (for the kit session to decide)

1. **One wrapper for cross-model roles.** Add something like `harness-lite/scripts/codex-run` that every role call uses. It would:
   - close stdin, or pass the prompt with `-` on stdin as `advise` does;
   - take the model, effort, sandbox and output file;
   - enforce a wall-clock timeout;
   - fail fast with a clear message if the only output after N seconds is the stdin-wait line.

   AGENTS templates then name the wrapper instead of a raw `codex exec`.
2. **Template and role text.** Wherever a Codex reviewer, architect or destroyer is suggested (`harness-lite/roles/orchestrator.md` mentions "Codex read-only"), state "always `< /dev/null`", or point to the wrapper.
3. **`sos check` warning.** Warn when AGENTS.md or CLAUDE.md contains `codex exec` without `< /dev/null` or a `-` stdin prompt.
4. **Orchestrator liveness rule.** Add to `harness-lite/roles/orchestrator.md`: after starting a long background agent or CLI, check its log or CPU within a couple of minutes. A hang must not be indistinguishable from slow work.

## Local mitigation already applied in TurnSigil

- AGENTS.md: the Kiến trúc sư and Người soát commands end in `< /dev/null`, and the hang signature is noted.
- Orchestrator memory: always close stdin, and check background CLI liveness within about 2 minutes.
