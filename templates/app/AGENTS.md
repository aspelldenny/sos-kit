# <Project name> — working agreement

<One paragraph: what this project is, for whom, and what it must never do.>

## Harness

This repo uses SOS Kit's harness (`harness-lite/`). Chủ nhà owns intent, taste and final acceptance; the agents do the middle (plan, build, check, repair).

- Main session: apply `harness-lite/CONTRACT.md` and take the role in `harness-lite/roles/orchestrator.md` (Quản đốc): delegate when it pays, do small changes directly, call the reviewer on the events the role lists.
- Give each subagent the contract, its own role file (`architect.md`, `worker.md`, `reviewer.md`) and a bounded brief. Claude Code wrappers: `.claude/agents/`.
- Gates run at commit and push (`sos gate all`, `sos gate local-secrets`); rules in `.sos.toml`. Do not bypass with `--no-verify`.
- After repeated failing tests, advice from a stronger model may appear in your context (`.sos.toml` `[advisor]`); weigh it against your evidence.

## Sources of truth

| What | Where |
|---|---|
| Current state and next steps | `docs/BACKLOG.md` (first section) |
| Scope and pass/fail status | `docs/FEATURES.json` |
| <design, wording, architecture> | <paths> |

## Design handoff

Before planning or building screens or wording, read this. If it is empty for the work at hand, ask Chủ nhà; do not start a spike in its place.

- **Approved by Chủ nhà:** <soul / design docs and the version agreed>
- **Reference screens or renders:** <paths or links the result must match>
- **Still a proposal, or waiting on an API:** <items not to build as if final>
- **Accepted when:** <the journeys Chủ nhà will walk to accept it>

## Commands

| Purpose | Command |
|---|---|
| Build / test | <e.g. `make check`> |
| Ready to hand over | <e.g. `make ready`> |
