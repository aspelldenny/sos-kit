# SOS Kit — Solo Operating System

One person. No team. Full pipeline from code to production to monitoring.

SOS Kit is a collection of Rust CLI tools and Claude Code skills that give a solo developer the shipping velocity of a full engineering team.

## Why

Building software alone means you're the architect, developer, reviewer, QA, DevOps, and on-call — all at once. Most solo devs skip steps. SOS Kit makes every step a single command so you never skip.

## The Pipeline

```
CODE → REVIEW → QA → SHIP → DEPLOY → MONITOR → LEARN → RETRO
  │       │       │     │       │        │         │       │
  you   /review  /qa   ship   ship     ship      ship   /retro
                       deploy  canary   learn
```

## Components

### Rust CLI Tools

| Tool | Binary | What it does |
|------|--------|-------------|
| **[ship](https://github.com/aspelldenny/ship)** | 4.7MB | Full release pipeline — test, commit, push, PR in one command |
| **[docs-gate](https://github.com/aspelldenny/docs-gate)** | 4.7MB | Enforce documentation compliance before every commit |

### ship subcommands

```bash
ship                    # Full pipeline: test → docs-gate → version → changelog → commit → push → PR
ship check              # Pre-flight only (test + docs-gate, no commit)
ship init               # Auto-detect project stack, generate .ship.toml
ship canary             # Post-deploy health check (HTTP + Docker via SSH)
ship deploy             # Deploy to production (SSH, GitHub Actions, Render, cargo, custom)
ship learn add "msg"    # Record a cross-project learning
ship learn search "q"   # Search learnings by keyword
ship learn list         # List recent learnings
ship serve              # Start MCP server for Claude integration
```

### Claude Code Skills

| Skill | Role | When to use |
|-------|------|-------------|
| `/review` | Staff Engineer | Before merge — find SQL injection, N+1 queries, auth bypass, logic bugs |
| `/qa` | QA Lead | After build — run tests, find bugs, fix with regression tests, verify |
| `/retro` | Team Lead | End of week — shipping velocity, hotspots, patterns, action items |
| `/ship` | Release Engineer | Ship it — triggers the full ship pipeline |

### Integrations

| Integration | What it does |
|-------------|-------------|
| **GitHub Actions canary** | Post-deploy health check in CI pipeline |
| **Pre-commit hooks** | type-check + docs-gate before every commit |
| **Jarvis uptime monitor** | Ping production every 10 min, Telegram alert on down |
| **MCP server** | 4 tools for Claude Desktop/Code integration |

## Install

### Prerequisites
- Rust toolchain (`rustup`)
- `gh` CLI (for PR creation)
- Claude Code (for skills)

### Ship CLI
```bash
git clone https://github.com/aspelldenny/ship.git
cd ship && cargo install --path .
```

### docs-gate
```bash
git clone https://github.com/aspelldenny/docs-gate.git
cd docs-gate && cargo install --path .
```

### Skills
```bash
# Copy skills to Claude Code
cp -r skills/review ~/.claude/skills/review
cp -r skills/qa ~/.claude/skills/qa
cp -r skills/retro ~/.claude/skills/retro
cp -r skills/ship ~/.claude/skills/ship
```

## Project Setup

Run `ship init` in any project to generate `.ship.toml`:

```bash
cd my-project
ship init
# 🔍 Detected: my-project (Next.js)
# ✅ Created .ship.toml
```

### Example configs

<details>
<summary>Next.js project (Tarot)</summary>

```toml
name = "tarot"
stack = "nextjs"
base_branch = "main"

[test]
command = "pnpm test --run"

[canary]
url = "https://www.soulsign.me"

[deploy]
provider = "ssh"
ssh = "deploy@myserver.com:1994"
command = "cd /opt/app && git pull && docker compose build && docker compose up -d"
maintenance_mode = true
```
</details>

<details>
<summary>Flask project (Media Rating)</summary>

```toml
name = "media-rating"
stack = "flask"

[test]
command = "./venv/bin/pytest tests/ -x"

[canary]
url = "https://my-app.onrender.com"

[deploy]
provider = "render"
```
</details>

<details>
<summary>Rust project (docs-gate)</summary>

```toml
name = "docs-gate"
stack = "rust"

[docs_gate]
blocking = true

[deploy]
provider = "cargo"
```
</details>

## Daily Workflow

```bash
# Morning — check what's running
ship canary

# Code your feature on a branch
git checkout -b feat/my-feature

# Before commit — pre-commit hook auto-runs:
#   ✅ type-check
#   ✅ docs-gate

# Ready to review
/review                 # Claude reviews your diff

# Ready to test
/qa                     # Claude runs tests, finds bugs, fixes them

# Ready to ship
ship                    # test → commit → push → PR (one command)

# After merge + deploy
ship canary             # verify production is healthy

# End of week
/retro                  # what shipped, velocity, hotspots, action items

# Record what you learned
ship learn add "always run migrations before deploy" -t deploy,db
```

## Architecture

```
sos-kit/
├── README.md                   # This file
├── docs/
│   ├── PHILOSOPHY.md           # Why SOS Kit exists
│   └── SETUP.md                # Detailed setup guide
├── skills/                     # Claude Code skills
│   ├── ship/SKILL.md           # /ship — release engineer
│   ├── review/SKILL.md         # /review — code review
│   ├── qa/SKILL.md             # /qa — QA verification
│   └── retro/SKILL.md         # /retro — retrospective
├── configs/                    # Example .ship.toml configs
│   ├── nextjs.toml
│   ├── flask.toml
│   ├── rust.toml
│   └── python.toml
├── hooks/                      # Git hooks
│   └── pre-commit              # type-check + docs-gate
└── integrations/
    ├── github-actions/         # Canary workflow snippet
    └── jarvis/                 # Uptime monitor for Telegram bots
```

## Comparison with gstack

| | SOS Kit | gstack |
|---|---|---|
| Author | aspelldenny | Garry Tan (YC CEO) |
| Language | Rust | Bun/TypeScript |
| Binary size | 4.7MB + 4.7MB | 58MB |
| Approach | CLI tools + Skills | 31 Markdown skills |
| Browser automation | No (not needed) | Playwright daemon |
| Planning skills | No (use your own methodology) | 6 planning skills |
| Design skills | No | 4 design skills |
| Ship automation | ✅ Full (test → PR) | ✅ Full (test → PR) |
| Deploy | ✅ SSH/Render/Actions/cargo | ❌ Manual |
| Health monitoring | ✅ Canary + Uptime | ✅ Canary |
| Cross-project learnings | ✅ JSONL | ✅ JSONL |
| Docs enforcement | ✅ docs-gate (Rust) | ❌ Manual |
| MCP server | ✅ 4 tools | ❌ No |
| Pre-commit hooks | ✅ | ❌ No |

**SOS Kit is smaller, faster, and focused on the ship-to-monitor pipeline.** It doesn't try to replace your planning methodology — it complements whatever process you already use.

## Philosophy

1. **One command per step.** If it takes more than one command, automate it.
2. **Gates, not guidelines.** Pre-commit hooks enforce quality. Pipeline gates stop bad code. Don't rely on memory.
3. **Cross-project learnings.** A mistake in project A should prevent the same mistake in project B.
4. **Rust for tools, AI for judgment.** CLI tools are fast, deterministic, zero-dependency. AI skills handle the fuzzy stuff (review, QA, retro).
5. **Solo-first.** No multi-user, no team features, no overhead. Every feature serves one person shipping fast.

## License

MIT
