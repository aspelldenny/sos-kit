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
| Agent-neutral scripts | `scripts/env-guard.sh`, `status.sh`, `test-watch.py`, `advise`, `bite` | Anything that runs a command |
| Agent adapters (earlier feedback) | `adapters/claude/hook.py`, `adapters/codex/hook.py` | That agent only; see `adapters/README.md` |

`sos gate all` covers: staged secrets (gitleaks), `.env` commits, case collisions, the default-branch rule, wording rules, state-doc size caps and `FEATURES.json` identity. Configure them in `.sos.toml` (template: `templates/sos.toml`). Extra project checks go in `hooks/pre-commit.local` / `hooks/pre-push.local` (executable, project-owned).

**Models.** `.sos.toml` `[roles]` sets backend, model and effort per role; the Claude wrappers in `.claude/agents/` mirror it and `sos check` compares them. `sos role` runs a role from any agent on that table (stdin closed, time limit, logged); use it instead of hand-written `codex exec` / `claude -p` calls.

**Test bite.** `scripts/bite` measures whether a change's tests catch a wrong version of it: with the changed source put back to the base, new or edited tests must fail; and one-token mutants (`==` → `!=`, `<` → `>=`, `n` → `n+1` …) on the changed lines must each turn some test red. Người soát runs it on slices that change logic and reports survivors (`.sos.toml` `[bite]`: test command, source and test regexes; each mutant runs the test command once, so keep it narrow for slow suites).

**Advisor.** After two consecutive failing runs of the same test or build runner, the adapter asks a stronger model (`scripts/advise`) for a direction and adds the answer to the worker's context; the advisor is shown the files the failure and the change point at. Each call needs two more failures; after the second call, the next two failures tell the worker to stop and report. Choose the backend in `.sos.toml` `[advisor]` (`claude`, `codex`, `auto` for the other model family, `off`); calls and costs are logged in `.advise-state/calls.jsonl`. An advisor that cannot run (its CLI is logged out) does not use up a call: Chủ nhà gets a desktop notification, the worker is told to report it at once, the session-start status and `sos check` repeat it until a call succeeds, and `scripts/advise --check` proves the fix.

## Commands

```bash
sos install            # once per repo (--dry-run to preview)
sos check              # kit files, hooks, tools, .sos.toml, agent wiring
sos update             # after upgrading sos
sos gate all           # what the pre-commit runs
sos role reviewer --from claude --brief brief.md   # run a role on its configured model
```

Requirements: `git`, `gitleaks`, `python3`, and the `sos` binary on `PATH`.

## Templates

`templates/` holds the starting versions of project-owned files (`AGENTS.md`, `CLAUDE.md`, `.sos.toml`, `docs/BACKLOG.md`, `docs/FEATURES.json` with a filled example in `FEATURES.example.json`, `.claude/settings.json`, `.codex/hooks.json`, `.claude/agents/*`). `sos install` copies each one only if the project lacks it; compare with them when merging by hand.

## Docs

Agents read what is loaded at session start and grep the rest. Docs stay useful when each fact has one home and each layer has one job:

| Layer | Files (examples) | Budget | Update when |
|---|---|---|---|
| Entry | `AGENTS.md`, `CLAUDE.md` | under 200 lines together | how work is done changes, or a task needs a new pointer. A router (task → file), not a reference |
| State | first section of `docs/BACKLOG.md` (`[state] file`) | under 80 lines | an item opens or closes. Open items only; finished ones go to the changelog |
| Sources | soul, character, voice, design spec, product, invariants, constants | one fact, one file | intent, voice, an invariant or infrastructure changes |
| Map guides | backend, frontend, prompts, data | none | a flow or boundary changes. Flows, reasons and pitfalls, pointing at code; never copies of signatures, schemas or prompts |
| History | changelog, discoveries, old tickets, archives | `.sos.toml` [docs] caps | always appended; grepped, never preloaded |

Anything else lives in the code and the changelog. `sos check` and the session-start status warn when the entry or state budget is exceeded.

## Limits

Role behavior is guidance, not enforcement; gates and runtime permissions are the boundary. Agent hooks do not see shell writes, and Codex runs no hooks inside custom-role subagents (`openai/codex#21753`); the git gates cover both. History of this harness: SOS Kit `docs/research/` (pilot 2026-10-05, survey 2026-10-08).
