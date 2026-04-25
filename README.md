# SOS Kit — Solo Operating System

One person. No team. Full operating system from inbound request to production health.

SOS Kit is a collection of Rust CLI tools, Claude Code skills, and role-separation protocols that let one human run a software business without dropping context.

## Why

Building software alone means wearing three hats every day:
- **Chủ nhà** (Owner) — deciding what's worth doing, vetoing scope creep, approving plans, maintaining vision docs
- **Kiến trúc sư** (Architect) — reading docs (not code), writing phiếu, specifying architecture
- **Thợ** (Worker) — reading code, executing the phiếu, running tests, shipping, monitoring, reporting discoveries back

If one brain does all three at once, features ship half-finished, tickets expand mid-build, and production breaks because nobody checked. SOS Kit enforces **role separation** — distinct skills per layer, formalized handoffs — so the same human snaps into different modes cleanly.

See [`docs/LAYERS.md`](./docs/LAYERS.md) for the role boundaries and [`docs/HANDOFF.md`](./docs/HANDOFF.md) for how the layers pass work.

## The Pipeline

For an **existing project** (you've already shipped, adding features):

```
ROUTE → PLAN → CODE → REVIEW → QA → SHIP → GUARD → DEPLOY → MONITOR → LEARN → RETRO
  │       │      │       │       │     │       │        │         │         │       │
/route  /plan   you   /review  /qa   ship    guard    ship       vps      ship   /retro
(Chủ   (Kiến   (Thợ)  (Thợ)         deploy   check   canary   logs/stats  learn
 nhà)  trúc                                                                 (Thợ)
       sư)
```

For a **new project from scratch** (0→1 — empty folder to launch):

```
VISION → BLUEPRINT → CONTRACT → SCAFFOLD → ITERATE → LAUNCH
  │          │           │          │         │         │
/init   sos blueprint  sos contract  /apply×N  phiếu loop  sos launch
(Chủ    (Chủ nhà →    (Kiến trúc sư) (Thợ)    (như cũ)  (gate Chủ nhà)
 nhà)    Kiến trúc sư)
```

Each stage belongs to exactly one layer. Crossing layers without a handoff is the anti-pattern SOS Kit is built to prevent. See [`docs/GENESIS.md`](./docs/GENESIS.md) for 0→1 details.

## Components

### Rust CLI Tools

| Tool | Binary | What it does |
|------|--------|-------------|
| **[ship](https://github.com/aspelldenny/ship)** | 3.4MB | Full release pipeline — test, commit, push, PR in one command |
| **[docs-gate](https://github.com/aspelldenny/docs-gate)** | 5.2MB | Enforce documentation compliance before every commit |
| **[guard](https://github.com/aspelldenny/guard)** | 1.9MB | Pre-deploy infrastructure gate — catch schema drift, env sync, canary mismatch before they hit production |
| **[vps](https://github.com/aspelldenny/vps)** | 1.2MB | Production ops — status, logs, restart, metrics for Docker Compose projects over SSH |
| **sos** (in `bootstrap/sos-rs/`) | (planned) | 0→1 bootstrap — `sos init` / `blueprint` / `contract` / `apply` / `launch`. Bash MVP at `bin/sos.sh`. See [`docs/GENESIS.md`](./docs/GENESIS.md). |

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

### guard subcommands

```bash
guard                   # Run all pre-deploy checks (schema drift, env sync, canary)
guard --dry-run         # Show what would be checked without running SSH
guard --skip-canary     # Skip canary pre-check (faster, less safe)
guard serve             # Start MCP server for Claude integration
```

### vps subcommands

```bash
vps status              # Docker Compose status for all projects (or one with --name)
vps logs                # Stream docker compose logs (optional grep filter)
vps restart             # Restart a project's docker compose
vps docker-stats        # Per-container CPU and memory usage
vps info                # Server info (uptime, memory, disk)
vps init                # Generate example ~/.vps.toml
vps serve               # Start MCP server (stdio transport)
```

### Claude Code Skills (grouped by layer)

**Chủ nhà layer** — vision, routing, decisions, relay:

| Skill | Purpose |
|---|---|
| `/init` | **0→1 only.** Vision capture for new project (empty folder → docs/PROJECT.md, SOUL.md, CHARACTER.md skeleton). |
| `/insight` | Distill raw research / user interviews / competitor observations into structured bullets for PROJECT.md / SOUL.md / CHARACTER.md. |
| `/route` | Classify inbound request: code / marketing / design / strategy / skip. Produces 5-bullet brief for Architect. |
| `/decide` | Trade-off triage. Present 2-3 concrete options with user-visible impact, recommend one. |

**Kiến trúc sư layer** — spec what gets built (docs-only access, no code):

| Skill | Purpose |
|---|---|
| `/plan` | Read vision + guide docs → write phiếu (ticket) in `phieu/TICKET_TEMPLATE.md` format with Task 0 verification anchors for Thợ to grep-verify. (v0.3.0 enters plan mode in Claude Code env) |
| `/forge` | **Recipe library extension.** Research official docs + write new recipe to `recipes/<category>/<name>.md` when blueprint demands a recipe library doesn't have yet. |

**Thợ layer** — execute + ship (full code access):

| Skill | Purpose |
|---|---|
| `/verify` | Task 0 grep-first: verify every file/function/constant anchor in the phiếu against real code BEFORE coding. |
| `/apply` | **0→1 only.** Apply 1 recipe from `recipes/` library — auto-generate sub-phiếu P000.N, run Task 0, execute steps, verify, commit. |
| `/review` | Staff-engineer review before merge — SQL injection, N+1 queries, auth bypass, logic bugs. |
| `/qa` | QA lead — run tests, find bugs, fix with regression tests, verify. |
| `/ship` | Release engineer — full ship pipeline (test → commit → PR → deploy → canary). |
| `/retro` | Weekly retrospective — shipping velocity, hotspots, patterns, action items. |

One skill = one layer + one responsibility. Skills never span layers. See [`docs/LAYERS.md`](./docs/LAYERS.md) for boundaries and the 2-tier authority split (architectural vs detail).

### Phiếu — the ticket workflow

The spine that connects Kiến trúc sư and Thợ. Every non-trivial change goes through a phiếu (Vietnamese for "ticket"):

- Format: `<type>/P<NNN>-<slug>` — e.g. `feat/P042-user-export`
- Lives at `docs/ticket/P<NNN>-<slug>.md` in the project
- Written by Kiến trúc sư (using `/plan`), executed by Thợ
- Discovery Report appended to `docs/DISCOVERIES.md` after each ticket closes

Shell function `phieu <slug>` (sourced from `phieu/phieu.sh`) creates worktree + branch + ticket file in one command, using a per-project counter for unique IDs. See [`phieu/README.md`](./phieu/README.md).

### Vision docs — Chủ nhà's foundation

Before any phiếu can be written, Chủ nhà must maintain:

- `PROJECT.md` — what the product is (vision, personas, monetization, architecture)
- `SOUL.md` — why it exists (philosophy, positioning, 3 hard lines, anti-product)
- `CHARACTER.md` — voice / persona (if the product has an AI character)

Skeletons are in [`phieu/VISION_TEMPLATES/`](./phieu/VISION_TEMPLATES/). Copy into your project's `docs/` on day 1, fill iteratively as research matures. Use `/insight` skill to distill raw material into these docs.

For brand-new projects, `/init` skill runs the capture interactively (3 questions max → 3 docs).

### Recipes — atomic, composable building blocks

For 0→1 (and beyond), SOS Kit replaces "stack-locked scaffolds" with a **recipe library**. Each recipe is one Markdown file solving one concrete need:

```
recipes/
├── infra/        docker-compose-postgres, nginx, vps-bootstrap-ubuntu, ...
├── auth/         nextauth-google-email, supabase-auth, jwt-custom, ...
├── payment/      payos-vn, stripe-checkout, lemonsqueezy, ...
├── ai/           multi-model-fallback, credit-atomic-deduct, ...
├── observability/sentry, umami, canary-github-actions, ...
└── framework-starter/ nextjs, sveltekit, flask, fastapi, tauri, ...
```

Kiến trúc sư picks recipes per-project in `BLUEPRINT.md` → Thợ runs `/apply` per recipe. Combo lạ → `/forge` makes a new recipe → save to library → next project benefits. See [`recipes/README.md`](./recipes/README.md).

### Genesis — the master phiếu (P000)

For new projects, `sos contract` generates `phieu/P000-genesis.md` — a single phiếu locking entire MVP scope by SHA256 spec_hash. No phiếu after P000 may add scope without re-locking + audit trail. See [`phieu/GENESIS_TEMPLATE.md`](./phieu/GENESIS_TEMPLATE.md) and [`phieu/LAUNCH_CHECKLIST.md`](./phieu/LAUNCH_CHECKLIST.md) (20-mục launch gate).

### Relay Protocol — Chủ nhà as the courier

Kiến trúc sư (Claude Web Project) and Thợ (Claude Code) are **separate sessions** — they cannot talk directly. When Thợ hits an architectural blocker mid-ticket, Chủ nhà routes between them manually.

The 2-3 minute protocol is in [`phieu/RELAY_PROTOCOL.md`](./phieu/RELAY_PROTOCOL.md).

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

### guard
```bash
git clone https://github.com/aspelldenny/guard.git
cd guard && cargo install --path .
```

### vps
```bash
git clone https://github.com/aspelldenny/vps.git
cd vps && cargo install --path .
vps init                  # generate ~/.vps.toml with your SSH + project paths
```

### Skills
```bash
# Copy all skills (Chủ nhà + Kiến trúc sư + Thợ) to Claude Code
# Chủ nhà layer
cp -r skills/insight ~/.claude/skills/insight
cp -r skills/route   ~/.claude/skills/route
cp -r skills/decide  ~/.claude/skills/decide
# Kiến trúc sư layer
cp -r skills/plan    ~/.claude/skills/plan
# Thợ layer
cp -r skills/verify  ~/.claude/skills/verify
cp -r skills/review  ~/.claude/skills/review
cp -r skills/qa      ~/.claude/skills/qa
cp -r skills/retro   ~/.claude/skills/retro
cp -r skills/ship    ~/.claude/skills/ship
```

### Phiếu shell function
```bash
# Add to ~/.zshrc (or ~/.bashrc)
echo "source ~/path/to/sos-kit/phieu/phieu.sh" >> ~/.zshrc
source ~/.zshrc
# Then for each project you want phiếu workflow on:
phieu-init ~/my-project
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

## Daily Workflow (3-layer version)

```bash
# Morning — Thợ checks production (Worker)
ship canary

# -- Inbound request arrives (user email, bug report, your own idea) --

# Layer 1: Chủ nhà — in your preferred Claude Code/Web
/route                  # code? marketing? design? skip? outputs 5-bullet brief
/insight                # if this is raw context needing distillation → vision doc update

# Layer 2: Kiến trúc sư — IN CLAUDE WEB PROJECT (separate session)
/plan                   # reads vision + guide docs (NOT code) → writes phiếu with
                        #   Task 0 anchors "thợ kiểm tra tại [file]:[function]"
                        # Chủ nhà reviews phiếu, gives one-word go/veto

# Layer 3: Thợ — IN CLAUDE CODE (separate session)
phieu feat user-export  # shell function: creates worktree + branch + phiếu file
                        # Chủ nhà pastes phiếu content from Claude Web into this file
/verify                 # Task 0 — grep every anchor against real code
                        # if ❌ architectural → stop, escalate via Chủ nhà → Architect
                        #   (see RELAY_PROTOCOL.md)
                        # if ⚠️ detail → self-decide + log Discovery
                        # if ✅ all → proceed to code

# Ready to review
/review                 # find logic bugs, SQL injection, auth bypass, N+1

# Ready to test
/qa                     # Claude runs tests, finds bugs, fixes, verifies

# Ready to ship
ship                    # test → docs-gate → commit → push → PR (one command)

# After merge + deploy
ship canary             # verify production is healthy

# End of ticket — Discovery Report (Thợ)
#   → append to docs/DISCOVERIES.md: what phiếu assumed vs reality,
#     what edge cases appeared, what docs got updated
#   → Kiến trúc sư reads this BEFORE writing next phiếu

# End of week (Thợ)
/retro                  # what shipped, velocity, hotspots, action items

# Record what you learned (any layer)
ship learn add "always run migrations before deploy" -t deploy,db
```

Each step is single-layer. Handoffs between them are formatted (see [`docs/HANDOFF.md`](./docs/HANDOFF.md)) — not freestyle Slack threads. Because Kiến trúc sư (Claude Web) and Thợ (Claude Code) cannot talk directly, Chủ nhà is the courier for every cross-session handoff. See [`phieu/RELAY_PROTOCOL.md`](./phieu/RELAY_PROTOCOL.md).

## Architecture

```
sos-kit/
├── README.md                   # This file — entry point
├── CLAUDE.md                   # Contributor guide for Claude Code
├── docs/
│   ├── PHILOSOPHY.md           # 6 principles
│   ├── LAYERS.md               # 3-role model (Chủ nhà / Kiến trúc sư / Thợ)
│   ├── HANDOFF.md              # Inter-layer handoff protocols
│   └── SETUP.md                # Detailed install guide
├── phieu/                      # Ticket workflow — spine connecting Kiến trúc sư ↔ Thợ
│   ├── README.md               # Setup + philosophy
│   ├── TICKET_TEMPLATE.md      # Phiếu format with Task 0 Verification Anchors
│   ├── DISCOVERY_PROTOCOL.md   # Thợ → Kiến trúc sư feedback loop + mismatch classification
│   ├── RELAY_PROTOCOL.md       # Chủ nhà's courier workflow (Thợ ↔ Kiến trúc sư)
│   ├── VISION_TEMPLATES/       # Day-1 skeletons for Chủ nhà
│   │   ├── PROJECT_template.md
│   │   ├── SOUL_template.md
│   │   └── CHARACTER_template.md
│   └── phieu.sh                # Shell function: phieu / phieu-list / phieu-done / phieu-init
├── skills/                     # Claude Code skills (one per layer+responsibility)
│   ├── insight/SKILL.md        # Chủ nhà — distill raw research → vision docs
│   ├── route/SKILL.md          # Chủ nhà — classify inbound
│   ├── decide/SKILL.md         # Chủ nhà — trade-off triage
│   ├── plan/SKILL.md           # Kiến trúc sư — write phiếu (docs-only, no code access)
│   ├── verify/SKILL.md         # Thợ — Task 0 grep-first (gate before coding)
│   ├── review/SKILL.md         # Thợ — code review
│   ├── qa/SKILL.md             # Thợ — QA verification
│   ├── ship/SKILL.md           # Thợ — release pipeline
│   └── retro/SKILL.md          # Thợ — retrospective
├── configs/                    # .ship.toml examples per stack
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

How SOS Kit differs from gstack and when to pick each → [`docs/COMPARISON.md`](./docs/COMPARISON.md).

## Philosophy

1. **One command per step.** If it takes more than one command, automate it.
2. **Gates, not guidelines.** Pre-commit hooks enforce quality. Pipeline gates stop bad code. Don't rely on memory.
3. **Cross-project learnings.** A mistake in project A should prevent the same mistake in project B.
4. **Rust for tools, AI for judgment.** CLI tools are fast, deterministic, zero-dependency. AI skills handle the fuzzy stuff (review, QA, retro).
5. **Solo-first.** No multi-user, no team features, no overhead. Every feature serves one person shipping fast.

## License

MIT
