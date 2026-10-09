# CLAUDE.md — SOS Kit (v3)

SOS Kit is a solo developer's harness for building apps with AI agents. v3 keeps the philosophy and drops the ceremony: the human owns intent, taste and final acceptance; AI owns the middle (plan, build, check, repair). Evidence and decisions behind v3: `docs/research/HARNESS_SURVEY_2026-10-08.md`, `docs/plans/V3_TRIAGE_2026-10-08.md`. Current work: `docs/BACKLOG.md` (first section).

## Map

| Path | What it is |
|---|---|
| `harness-lite/` | **The core.** `CONTRACT.md` (shared rules) + `roles/` (orchestrator, architect, worker, reviewer). Vendored into app repos with `UPSTREAM.json` hashes. |
| `hooks/` | Git gates (6, fail-closed, each prints a fix). The real gate for every agent. |
| `scripts/` | Agent-neutral CLIs: git-gate helpers, `env-guard.sh`, `status.sh`, `advise` + `test-watch.py` (advisor). `scripts/trust-gate.sh rebaseline` after a reviewed change to an auto-exec file. |
| `adapters/` | One thin translator per agent (`claude/`, `codex/`): hook payload → `scripts/` call. No policy; every check has a git/CLI backstop. Tests: `tests/adapters/run.sh`. See `adapters/README.md`. |
| `recipes/` + `skills/apply/` | Verified implementation patterns and the skill that applies one. |
| `crates/`, `bin/` | The `sos` Rust binary. `crates/sos-gates` = the v3 gates (`sos gate text|docs|features|local-secrets|all`, `sos filter`; config `.sos.toml`). The rest is v2 install/bootstrap code being shrunk to `sos lite install / update / check` (BACKLOG step 2.4); `core/` belongs to it until then (v2 adapter docs: `archive/v2/adapters/`). |
| `configs/`, `templates/`, `integrations/` | Per-stack `.ship.toml` examples, starter files, CI/uptime snippets. |
| `tool-manifest.toml` | Version pins + checksums for tools installed by `install.sh` (still lists the merged tools until step 2.4). |
| `archive/v2/` | The v2 workflow (phiếu, debate rounds, approval gate, role handbooks, INV gates). Read-only history; tag `v2-final` is its last live state. |
| `docs/` | `BACKLOG.md`, `CHANGELOG.md` (repo root), `PHILOSOPHY.md`, `DISCOVERIES.md`, `research/`, `plans/`, `retro/`. |

quality-gate, doc-rotate (cap-check), doctor (runtime-scan) and claude-hooks (features-guard) are merged into `sos gate` (triage §G); their repos are superseded. The server pack `~/ship`, `~/guard`, `~/vps` stays separate, pinned for web projects only.

## Rules

1. **Mechanical rules go in a gate; judgment stays guidance.** Do not restate a gated rule in prose. A new gate must fail closed and print how to fix the failure.
2. **Add only what a real failure demands.** Every file, rule, hook or role line must point to an observed failure or a decision in `docs/plans/`. Remove scaffolding a newer model no longer needs, and record why.
3. **No hardcoded personal paths** in shipped files. Use `~/` or a placeholder. (gitleaks/trust-gate do not catch this — check before committing.)
4. **Changing `harness-lite/` changes how agents behave in every app.** Get one independent look (fresh-context reviewer or Chủ nhà) before committing, unless Chủ nhà's request is itself the approval; note which in the commit.
5. **Tools are CLI first.** A check any project needs goes into `sos gate`; MCP is only an optional wrapper. Server-pack tools (`ship`, `guard`, `vps`) stay in their own repos.
6. **Keep this file and `README.md` true.** If a path in the map moves, update both in the same commit.

## Working here

- Commits in English; talk with the maintainer in Vietnamese (em/anh). Role names (Chủ nhà, Quản đốc, Kiến trúc sư, Thợ, Người soát) are fixed terms, not forms of address.
- `cargo test` and `tests/adapters/run.sh` must stay green. The pre-commit runs `cargo check`, gitleaks and the trust gate; do not bypass with `--no-verify`.
- Durable state belongs in the repo (BACKLOG, CHANGELOG, plans), not in private memory.
