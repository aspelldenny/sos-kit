# Harness Lite (SOS Kit v3)

A small harness for building with AI agents. Chủ nhà (the human owner) holds both ends: intent and taste at the start, acceptance at the end. Agents do the middle: plan, build, check, repair. Model-neutral; works in Claude Code, Codex, or any harness that reads `AGENTS.md`.

This folder is **kit-owned**: `sos update` replaces files you have not edited and puts the new version of an edited file next to it as `<file>.sos-new`. Hashes live in `UPSTREAM.json`. Put project-specific rules in the project's own files (`AGENTS.md`, `.sos.toml`, `hooks/*.local`), not here.

## Roles

Runtime input is the [contract](CONTRACT.md), one role, and the task brief. Do not load every role into every agent.

| Who | Role file | When |
|---|---|---|
| Main session | [Quản đốc](roles/orchestrator.md) | Always: holds intent and state, delegates, integrates, reports to Chủ nhà |
| Subagent | [Kiến trúc sư](roles/architect.md) | Only for real structural uncertainty |
| Subagent | [Thợ](roles/worker.md) | Implementation |
| Subagent or fresh session | [Người soát](roles/reviewer.md) | On events, not every slice: a journey first connects; data, money, privacy or sync is touched; before handoff |

An adversarial pass by a different model family before acceptance is optional.

## What runs automatically

| Layer | Files | Applies to |
|---|---|---|
| **Git gates** (the real boundary) | `hooks/pre-commit` → `sos gate all`; `hooks/pre-push` → `sos gate local-secrets` | Every agent and every person |
| Agent-neutral scripts | `scripts/env-guard.sh`, `status.sh`, `test-watch.py`, `advise` | Anything that runs a command |
| Agent adapters (earlier feedback) | `adapters/claude/hook.py`, `adapters/codex/hook.py` | That agent only; see `adapters/README.md` |

`sos gate all` covers: staged secrets (gitleaks), `.env` commits, case collisions, the default-branch rule, wording rules, state-doc size caps and `FEATURES.json` identity. Configure them in `.sos.toml` (template: `templates/sos.toml`). Extra project checks go in `hooks/pre-commit.local` / `hooks/pre-push.local` (executable, project-owned).

**Advisor.** After two consecutive failing test runs, the adapter asks a stronger model (`scripts/advise`) for a direction and adds the answer to the worker's context; at most two calls per failing streak, then the worker stops and reports. `SOS_ADVISOR=off` disables it; `ADVISE_BACKEND=claude|codex`, `ADVISE_MODEL` choose the model.

## Commands

```bash
sos install            # once per repo (--dry-run to preview)
sos check              # kit files, hooks, tools, .sos.toml, agent wiring
sos update             # after upgrading sos
sos gate all           # what the pre-commit runs
```

Requirements: `git`, `gitleaks`, `python3`, and the `sos` binary on `PATH`.

## Templates

`templates/` holds the starting versions of project-owned files (`AGENTS.md`, `CLAUDE.md`, `.sos.toml`, `.claude/settings.json`, `.codex/hooks.json`, `.claude/agents/*`). `sos install` copies each one only if the project lacks it; compare with them when merging by hand.

## Limits

Role behavior is guidance, not enforcement; gates and runtime permissions are the boundary. Agent hooks do not see shell writes, and Codex runs no hooks inside custom-role subagents (`openai/codex#21753`); the git gates cover both. History of this harness: SOS Kit `docs/research/` (pilot 2026-10-05, survey 2026-10-08).
