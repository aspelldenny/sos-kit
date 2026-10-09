# SOS Kit

A small harness for one person building with AI coding agents (Claude Code, Codex, or any agent that reads `AGENTS.md`). It works for apps, web services, command-line tools and libraries, and for writing projects.

**The idea:** the human holds both ends: intent and taste at the start, acceptance at the end. Agents do the middle: plan, build, check, repair. SOS Kit gives that middle a few roles, a few checks that cannot be skipped, and a stronger model to ask when the work is stuck.

## Quick start

```bash
curl -fsSL https://raw.githubusercontent.com/aspelldenny/sos-kit/main/install.sh | sh   # installs the `sos` binary
cd your-project
sos install --dry-run      # see what it would add
sos install                # add the harness
sos check                  # confirm it is wired
```

Needs `git`, `gitleaks` and `python3`, and `~/.local/bin` on `PATH`. Prebuilt binaries: macOS on Apple silicon and Linux x64 (also on npm: `npm install -g sos-kit`); elsewhere install Rust, clone this repo and run `cargo install --path crates/sos-cli`. Run `sos install` at the top level of a git repository (`git init` first for a new project).

Then fill in the placeholders in `AGENTS.md` (what the project is, where its sources of truth live, how to build and test), write the first item in `docs/BACKLOG.md`, and start your agent in the repo. The main session takes the Quản đốc role and delegates as the role file says.

## What `sos install` adds

| Added | Owner | What it is |
|---|---|---|
| `harness-lite/` | SOS Kit (`sos update` refreshes it) | The shared contract and four roles: **Quản đốc** (orchestrator), **Kiến trúc sư** (architect, only for structural questions), **Thợ** (worker), **Người soát** (independent reviewer). Plus the scripts, agent adapters and git hooks below. |
| Git hooks | SOS Kit | Every commit runs `sos gate all`; every push runs `sos gate local-secrets`. Project-specific checks (type checks, tests) go in `harness-lite/hooks/pre-commit.local`. |
| `AGENTS.md`, `CLAUDE.md` | Your project | Project instructions; `CLAUDE.md` imports `AGENTS.md` so Claude Code and Codex read the same text. |
| `.sos.toml` | Your project | Rules for the gates (wording, doc size, feature list, branch) and the advisor. |
| `docs/BACKLOG.md`, `docs/FEATURES.json` | Your project | Current work (agents see its first section at session start) and the feature list with pass/fail status. |
| `.claude/`, `.codex/` | Your project | Hooks for Claude Code and Codex: a status line at session start, a block on editing real `.env` files, and the advisor. |

Existing files are never overwritten; the only change to an existing file is three lines appended to `.gitignore`. If the project already has git hooks, `sos install` leaves them active and tells you how to chain them; `--force-hooks` copies them to `harness-lite/hooks/` (`pre-commit` and `pre-push` become `*.local`, run after the gates) and switches git to the new hooks.

## The gates

`sos gate all` runs at every commit, reads what is staged, and blocks with a "how to fix" line when something fails. A gate that cannot run (for example `gitleaks` missing, or an unreadable file) blocks too.

| Gate | Blocks |
|---|---|
| `secrets` | Secrets in the staged diff (gitleaks) |
| `env-commit` | A real `.env` file being committed (`.env.example` is fine) |
| `case` | Two paths that differ only by letter case (breaks macOS and Windows checkouts) |
| `branch` | Non-Markdown commits on the default branch, when `[git] protect_default_branch = true` |
| `text` | In files you list in `[text].files`: your banned phrases and leak patterns, a built-in list of AI vendor and model names (`[text] base = false` turns it off), and `<thinking>`/`<reflection>` blocks |
| `docs` | State documents past a line limit (`[docs]`) |
| `features` | Deleting or malforming entries in `docs/FEATURES.json` |
| `local-secrets` (push) | Tokens in `.git/config` and agent config files (gitleaks rules); a real `.env` file tracked anywhere, or one at the repo root that is not ignored |

`sos filter` applies the wording rules to text on stdin, for filtering model output at runtime.

## How work runs

1. **Brief.** Chủ nhà states the outcome, scope, references and how it will be accepted.
2. **Build.** Quản đốc gives bounded work to Thợ (or does a small change itself), each checking the brief against the real code or text first and verifying with independently computed expectations.
3. **Advice when stuck.** After two failing runs of the same recognised test command (pytest, cargo test, swift test, xcodebuild test, npm/yarn/pnpm test, vitest, jest, go test, make check/test/ready), the agent's hook asks a stronger model (Opus or Codex, configurable) for a direction.
4. **Independent review at the moments that matter**, not every slice: when a journey first works end to end, when data, money, privacy or sync is touched, and before handing over something meant for real use. Người soát uses the result as its user would.
5. **Acceptance.** Chủ nhà uses the real thing. Reports say plainly what ran for real, what ran only on sample data, and what is unverified.

## This repository

| Path | What it is |
|---|---|
| [`harness-lite/`](harness-lite/README.md) | Contract and roles (installed into projects) |
| [`adapters/`](adapters/README.md), `scripts/`, `templates/app/` | Agent adapters, agent-neutral scripts and the templates `sos install` writes |
| `crates/` | The `sos` binary (installer and gates, Rust) |
| [`recipes/`](recipes/README.md), `skills/apply/` | Implementation patterns taken from shipped code, and the skill that applies one |
| [`docs/`](docs/PHILOSOPHY.md) | Philosophy, backlog, plans and research behind v3 |
| `archive/v2/` | The previous, heavier workflow (tag `v2-final`), kept as history |

Maintainers: [`CLAUDE.md`](CLAUDE.md) has the rules for changing this repo; [`SECURITY.md`](SECURITY.md) lists what runs automatically and why it is safe; [`CHANGELOG.md`](CHANGELOG.md) has the history.

## License

[MIT](LICENSE)
