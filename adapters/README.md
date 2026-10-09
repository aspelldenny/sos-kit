# adapters/ — one thin translator per agent

SOS Kit's checks live in agent-neutral places; an adapter only translates one agent's hook
payload into a call to them and translates the answer back. No rule is decided here.

| Layer | What | Works with |
|---|---|---|
| **Git gates** (`hooks/pre-commit`, `hooks/pre-push`) | Secrets, `.env` commits, case collisions, branch rule, product gates | Every agent and a human at the keyboard. **This is the real gate.** |
| **Neutral CLIs** (`scripts/`) | `env-guard.sh`, `test-watch.py` (advisor trigger), `advise`, `status.sh` | Anything that can run a command |
| **Prompts** (`AGENTS.md` → `harness-lite/`) | Contract + roles | Codex reads `AGENTS.md`; Claude via `CLAUDE.md` importing it; most other harnesses read `AGENTS.md` |
| **Adapters** (`adapters/<agent>/`) | Earlier feedback inside the agent loop | Only that agent |

**Rule:** every check an adapter makes must also have a git or CLI backstop. Adapters give
earlier feedback; they are not the security boundary. (Codex does not run hooks inside
custom-role subagents, `openai/codex#21753`; any agent can write `.env` through the shell.)

## What each adapter wires

Wiring templates for app repos: `templates/app/claude-settings.json` and `templates/app/codex-hooks.json` in sos-kit (installed as `harness-lite/templates/`). The adapters find the scripts relative to themselves (`../../scripts/`), so the same files work in sos-kit and in an app's `harness-lite/`.

| Hook | Claude Code | Codex | Calls |
|---|---|---|---|
| Session start | `SessionStart` → stdout to context | `SessionStart` → `additionalContext` | `scripts/status.sh` |
| Before an edit | `PreToolUse` Edit/Write/MultiEdit/NotebookEdit, `file_path` | `PreToolUse` `apply_patch`, every path in the patch; unparseable patch is blocked | `scripts/env-guard.sh` (exit 2 blocks) |
| After a shell command (optional advisor) | `PostToolUse` + `PostToolUseFailure` Bash; key = session + `agent_id` | `PostToolUse` Bash (no failure event, no exit code; detected from output) | `scripts/test-watch.py` → `scripts/advise` |

Payload shapes were captured live on 2026-10-09 (Claude Code, Codex 0.162) and are the test
fixtures: `tests/adapters/fixtures/`. Run `tests/adapters/run.sh` (advisor stubbed, no model call).

## Another harness

Nothing to install for the gates: they run at `git commit`. Point it at `AGENTS.md`. To add
in-loop feedback, write `adapters/<name>/hook.py` that maps its payload to the same three
CLI calls, add its captured payloads to `tests/adapters/fixtures/`, and extend `run.sh`.

## Advisor settings (env)

`SOS_ADVISOR=off` disables it · `ADVISE_AFTER` consecutive failures before a call (2) ·
`ADVISE_MAX` calls per failing streak before "stop and report" (2) · `ADVISE_BACKEND`
`claude`|`codex` · `ADVISE_MODEL` (claude: `opus`; codex: Codex config default).
sos-kit itself wires only session start and the edit guard (`.claude/settings.json`, `.codex/hooks.json`); the app templates also wire the advisor.
