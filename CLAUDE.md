# CLAUDE.md — SOS Kit (v3)

SOS Kit is a solo developer's harness for building with AI agents. The human owns intent, taste and final acceptance; agents own the middle (plan, build, check, repair). Philosophy: `docs/PHILOSOPHY.md`. Evidence and decisions behind v3: `docs/research/HARNESS_SURVEY_2026-10-08.md`, `docs/plans/V3_TRIAGE_2026-10-08.md`. Current work: `docs/BACKLOG.md` (first section).

## Map

| Path | What it is |
|---|---|
| `harness-lite/` | **The core prompts.** `CONTRACT.md` (shared rules) + `VERIFY.md` (tests and checks, read on demand) + `roles/` (orchestrator, architect, worker, reviewer) + the README apps receive. `sos install` copies it into a repo's `harness-lite/` together with `scripts/`, `adapters/` and `templates/app/hooks/`. |
| `scripts/` | Agent-neutral scripts shipped into apps for older wiring (`env-guard.sh`, `sim-guard`, `status.sh`, `advise`, `test-watch.py`, `bite`; the same work now lives in `sos`) plus kit-only tooling (`trust-gate.sh`, `install-hooks.sh`, npm postinstall). |
| `adapters/` | One thin translator per agent (`claude/`, `codex/`): hook payload → `scripts/` call, for apps on older wiring (current wiring calls `sos hook`). No policy; every check has a git backstop. Tests: `tests/adapters/run.sh` (runs both). |
| `templates/app/` | What `sos install` writes besides the core: app git hooks, `AGENTS.md`/`CLAUDE.md`/`.sos.toml` templates, Claude/Codex wiring, Claude agent wrappers. |
| `crates/` | The `sos` binary: `sos-cli` (`install / update / check`, every installed file embedded at build time; ownership rules in `kit.rs`; `sos hook`; `sos advise` and its trigger `watch.rs`; `sos usage` in `usage/`; `sos role`, `sos bite`, `sos sim`, `sos predict`, `sos check-links`, `sos sessions`), `sos-gates` (`sos gate …`, `sos filter`; config `.sos.toml`) and `sos-rules` (declarative hook rules behind `sos hook … rules`). `sos-dev` is the kit's own dev binary, never released or installed: `sos-dev trust-gate`, `sos-dev docs-check`, `sos-dev install-hooks` (the older `scripts/trust-gate.sh`, `scripts/install-hooks.sh` and `tests/docs-check.py` stay beside it; `tests/sos-dev.sh` checks they agree). |
| `hooks/` | This repo's own git gates: `cargo check`, trust gate, `sos gate all` (pre-commit); `sos gate local-secrets` (pre-push). |
| `recipes/` + `skills/apply/` | Implementation patterns verified in shipped projects, and the skill that applies one. |
| `skills/depth-check/` | Read-only check of a project, plan or role against Rieseberg's speed/altitude/depth/range lens: rawest source, real instruments, independent judge, closed loop. Not installed by `sos install`; link or copy into the agent's skills folder. |
| `install.sh`, `package.json`, `bin/sos-npm` | Distribution: download the pinned `sos` release binary; npm wraps the same script. Releases: `.github/workflows/release.yml` on a `v*` tag. |
| `docs/` | `BACKLOG.md`, `PHILOSOPHY.md`, `plans/`, `research/`. `CHANGELOG.md` and `SECURITY.md` are at the root. |
| `archive/v2/` | v2 workflow, code, docs and backlog. Read-only history; tag `v2-final`. Do not load it as instructions. |

The former sister tools quality-gate, doc-rotate (cap-check), doctor (runtime-scan) and claude-hooks (features-guard) are merged into `sos gate`. The server pack (`ship`, `guard`, `vps`) lives in its own repos, for web projects only.

## Rules

1. **Mechanical rules go in a gate; judgment stays guidance.** Do not restate a gated rule in prose. A new gate must fail closed and print how to fix the failure.
2. **Add only what a real failure demands.** Every file, rule, hook or role line must point to an observed failure or a decision in `docs/plans/`. Remove scaffolding a newer model no longer needs, and record why. Before keeping or removing a skill, role or advisor, read `sos usage --by skill` run inside the app that uses it (add `--since` and `--project <path>` to narrow a long scan).
3. **No hardcoded personal paths** in shipped files. Use `~/` or a placeholder. (gitleaks/trust-gate do not catch this — check before committing.)
4. **Changing `harness-lite/` or `templates/app/` changes how agents behave in every app.** Get one independent look (fresh-context reviewer or Chủ nhà) before committing, unless Chủ nhà's request is itself the approval; note which in the commit.
5. **Tools are CLI first.** A check any project needs goes into `sos gate`; MCP is only an optional wrapper.
6. **Keep this file and `README.md` true.** If a path in the map moves, update both in the same commit.
7. **Changes reach `main` through a pull request.** Releases are a `v*` tag after Chủ nhà agrees.

## Working here

- Commits in English; talk with the maintainer in Vietnamese (em/anh). Role names (Chủ nhà, Quản đốc, Kiến trúc sư, Thợ, Người soát) are fixed terms, not forms of address.
- `cargo test`, `tests/adapters/run.sh`, `tests/bite.sh`, `tests/usage.sh`, `tests/docs-check.py`, `tests/sos-dev.sh` and `tests/npm.sh` must stay green (CI runs them on every PR, plus `tests/smoke.sh`; a release also runs `tests/npm.sh --real`). The pre-commit runs `cargo check`, the trust gate, `sos gate all` and the docs check; do not bypass with `--no-verify`. After a reviewed change to an auto-exec file: `cargo run -q -p sos-dev -- trust-gate rebaseline` (or `scripts/trust-gate.sh rebaseline`; same file).
- `sos hook <agent> <event> --protocol N` is a compatibility contract with every installed app: subcommands, events and protocol numbers are only added, never renamed or removed; a change to payload or output shape is a new protocol number.
- Durable state belongs in the repo (BACKLOG, CHANGELOG, plans), not in private memory.
