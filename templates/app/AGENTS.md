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

## Commands

| Purpose | Command |
|---|---|
| Build / test | <e.g. `make check`> |
| Ready to hand over | <e.g. `make ready`> |
