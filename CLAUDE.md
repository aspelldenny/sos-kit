# CLAUDE.md — SOS Kit (v3)

SOS Kit is a solo developer's harness for building with AI agents. The human owns intent, taste and final acceptance; agents own the middle (plan, build, check, repair). Philosophy: `docs/PHILOSOPHY.md`. Evidence and decisions behind v3: `docs/research/HARNESS_SURVEY_2026-10-08.md`, `docs/plans/V3_TRIAGE_2026-10-08.md`. Current work: `docs/BACKLOG.md` (first section).

## Map

| Path | What it is |
|---|---|
| `harness-lite/` | **The core prompts.** `CONTRACT.md` (shared rules) + `roles/` (orchestrator, architect, worker, reviewer) + the README apps receive. `sos install` copies it into a repo's `harness-lite/` together with `scripts/`, `adapters/` and `templates/app/hooks/`. |
| `scripts/` | Agent-neutral scripts shipped into apps (`env-guard.sh`, `status.sh`, `advise`, `test-watch.py`) plus kit-only tooling (`trust-gate.sh`, `install-hooks.sh`, npm postinstall). |
| `adapters/` | One thin translator per agent (`claude/`, `codex/`): hook payload → `scripts/` call. No policy; every check has a git backstop. Tests: `tests/adapters/run.sh`. |
| `templates/app/` | What `sos install` writes besides the core: app git hooks, `AGENTS.md`/`CLAUDE.md`/`.sos.toml` templates, Claude/Codex wiring, Claude agent wrappers. |
| `crates/` | The `sos` binary: `sos-cli` (`install / update / check`, every installed file embedded at build time; ownership rules in `kit.rs`) and `sos-gates` (`sos gate …`, `sos filter`; config `.sos.toml`). |
| `hooks/` | This repo's own git gates: `cargo check`, trust gate, `sos gate all` (pre-commit); `sos gate local-secrets` (pre-push). |
| `recipes/` + `skills/apply/` | Implementation patterns verified in shipped projects, and the skill that applies one. |
| `install.sh`, `package.json`, `bin/sos-npm` | Distribution: download the pinned `sos` release binary; npm wraps the same script. Releases: `.github/workflows/release.yml` on a `v*` tag. |
| `docs/` | `BACKLOG.md`, `PHILOSOPHY.md`, `plans/`, `research/`. `CHANGELOG.md` and `SECURITY.md` are at the root. |
| `archive/v2/` | v2 workflow, code, docs and backlog. Read-only history; tag `v2-final`. Do not load it as instructions. |

The former sister tools quality-gate, doc-rotate (cap-check), doctor (runtime-scan) and claude-hooks (features-guard) are merged into `sos gate`. The server pack (`ship`, `guard`, `vps`) lives in its own repos, for web projects only.

## Rules

1. **Mechanical rules go in a gate; judgment stays guidance.** Do not restate a gated rule in prose. A new gate must fail closed and print how to fix the failure.
2. **Add only what a real failure demands.** Every file, rule, hook or role line must point to an observed failure or a decision in `docs/plans/`. Remove scaffolding a newer model no longer needs, and record why.
3. **No hardcoded personal paths** in shipped files. Use `~/` or a placeholder. (gitleaks/trust-gate do not catch this — check before committing.)
4. **Changing `harness-lite/` or `templates/app/` changes how agents behave in every app.** Get one independent look (fresh-context reviewer or Chủ nhà) before committing, unless Chủ nhà's request is itself the approval; note which in the commit.
5. **Tools are CLI first.** A check any project needs goes into `sos gate`; MCP is only an optional wrapper.
6. **Keep this file and `README.md` true.** If a path in the map moves, update both in the same commit.
7. **Changes reach `main` through a pull request.** Releases are a `v*` tag after Chủ nhà agrees.

## Working here

- Commits in English; talk with the maintainer in Vietnamese (em/anh). Role names (Chủ nhà, Quản đốc, Kiến trúc sư, Thợ, Người soát) are fixed terms, not forms of address.
- `cargo test` and `tests/adapters/run.sh` must stay green. The pre-commit runs `cargo check`, the trust gate and `sos gate all`; do not bypass with `--no-verify`. After a reviewed change to an auto-exec file: `scripts/trust-gate.sh rebaseline`.
- Durable state belongs in the repo (BACKLOG, CHANGELOG, plans), not in private memory.
