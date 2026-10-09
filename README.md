# SOS Kit — Solo Operating System

One person, AI agents in the middle, real apps out the other end.

SOS Kit is the harness a solo developer uses to build and ship apps with Claude Code and Codex. **v3** keeps one idea from the start: *the human holds both ends* — intent and taste at the beginning, acceptance at the end — and AI does the work in between: plan, build, check, repair.

> v3 replaces the v2 workflow (ticket per change, architect↔worker debate rounds, an approval gate before every execution, 200–300-line role handbooks). The reasons and the evidence from four shipped apps are in [`docs/research/HARNESS_SURVEY_2026-10-08.md`](docs/research/HARNESS_SURVEY_2026-10-08.md). v2 is preserved under [`archive/v2/`](archive/v2/) and tag `v2-final`.

## What's in the box

| Part | What it does |
|---|---|
| [`harness-lite/`](harness-lite/README.md) | The core: one shared contract plus four short role prompts — **Quản đốc** (orchestrator), **Kiến trúc sư** (architect, only for structural uncertainty), **Thợ** (worker), **Người soát** (independent reviewer). Model-neutral; works in Claude Code and Codex. |
| [`adapters/`](adapters/README.md) | Agent-neutral by design: the checks are git gates and plain CLIs in `scripts/`; `adapters/claude` and `adapters/codex` only translate each agent's hook payload. Another harness needs no adapter for the gates. |
| Git gates | A few fail-closed checks, each printing how to fix the failure: secrets (gitleaks), `.env` commits, case collisions, code on the default branch, plus type checks. App repos add product gates such as wording rules (`quality-gate`) and a protected feature list (`features-guard`). |
| [`recipes/`](recipes/README.md) | Implementation patterns verified against shipped code (payments, auth, rate limiting, PII encryption, SSE keepalive, multi-model fallback). Applied with the `apply` skill. |
| Sister tools | Pinned in [`tool-manifest.toml`](tool-manifest.toml), installed by `install.sh`: `claude-hooks`, `quality-gate`, `doctor` (runtime secret scan), `doc-rotate` (keeps state docs small); server pack `ship`, `guard`, `vps` for web projects. |

## How a project runs

1. **Brief.** Chủ nhà states the outcome, scope, references and acceptance. Product research and design decisions happen before the repo (spy, Hub).
2. **Blueprint, when needed.** The architect resolves structural questions once; the worker does not wait on it for routine changes.
3. **Build in slices.** Each worker checks the brief against the real code before building, runs focused tests with independently computed expectations, and reports what it actually verified.
4. **Check at the right moments, not every slice.** An independent reviewer looks from the outside in — user journeys, data lifecycle, promises the UI makes, edge environments — when a journey first connects, when a slice touches data, money, privacy or sync, and before handoff.
5. **Accept.** Chủ nhà uses the real app. An optional adversarial pass, ideally by a different model family, can precede it.

State lives in the repo: a feature list with pass/fail status, one current-state file, and git history.

## Install

v3's installer (`sos lite install / update / check`) is in progress — see [`docs/BACKLOG.md`](docs/BACKLOG.md). Until it lands:

1. Copy `harness-lite/` into the app repo and record the source commit and file hashes in `harness-lite/UPSTREAM.json`.
2. Point the app's `AGENTS.md` (and a one-line `CLAUDE.md` that imports it) at `harness-lite/CONTRACT.md` and the orchestrator role.
3. Add git hooks for secrets and `.env`, and the product gates the app needs.
4. Install sister tools with `./install.sh` (pinned versions and checksums from `tool-manifest.toml`); install `gitleaks` with Homebrew.

## Philosophy

See [`docs/PHILOSOPHY.md`](docs/PHILOSOPHY.md). In short: accountability for intent and acceptance stays human; rules that must always hold are gates, not reminders; extra roles and checks are added only where they catch what the builder cannot see.

## License

[MIT](LICENSE)
