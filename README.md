# SOS Kit — Solo Operating System

One person. No team. Full operating system from inbound request to production health.

SOS Kit is a collection of Rust CLI tools, Claude Code skills, subagents, and role-separation protocols that let one human run a software business without dropping context.

## Why

Building software alone means wearing three hats every day:
- **Chủ nhà** (Owner) — deciding what's worth doing, vetoing scope creep, approving plans, maintaining vision docs
- **Kiến trúc sư** (Architect) — reading docs (not code), writing phiếu, specifying architecture
- **Thợ** (Worker) — reading code, executing the phiếu, running tests, shipping, monitoring, reporting discoveries back

If one brain does all three at once, features ship half-finished, tickets expand mid-build, and production breaks because nobody checked. SOS Kit enforces **role separation** — distinct skills per layer, formalized handoffs, and structural envelopes (tool allowlists + hooks) — so the same human snaps into different modes cleanly.

See [`docs/LAYERS.md`](./docs/LAYERS.md) for the role boundaries and [`docs/HANDOFF.md`](./docs/HANDOFF.md) for how the layers pass work.

## Two ways to run the 3-role envelope

SOS Kit ships **two enforcement modes** for the Architect ↔ Worker boundary. Pick the one that fits your project; both share the same phiếu format, vision docs, and skills.

| Mode | Architect lives in | Enforcement | Best for |
|---|---|---|---|
| **Subagent mode** (default in v2) | Claude Code subagent (`.claude/agents/architect.md`) | `tools` allowlist + `PreToolUse` hook blocks code reads | Single-session flow — orchestrator spawns Architect, then Worker, no copy-paste |
| **Web Project mode** (v1, still supported) | Separate Claude Web Project session | Human discipline + separate session | Iterative phiếu refinement via multi-turn chat with Architect |

Subagent mode adds two forcing functions:
1. **BACKLOG.md gate** — Architect only writes phiếu for items in "Active sprint"; a SessionStart hook surfaces the backlog every time you open Claude Code.
2. **Pre-code debate loop (v2.1)** — Worker challenges the phiếu against real code BEFORE coding; Architect responds; multi-turn until consensus. Chủ nhà only steps in at 2 points: initial brief and final approval gate. See [`docs/ORCHESTRATION.md`](./docs/ORCHESTRATION.md) and [`docs/HANDOFF.md`](./docs/HANDOFF.md) (Handoff 2.5) for details.

See [`INSTALL.md`](./INSTALL.md) for setup.

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

### Claude Code Subagents (v2 — Subagent mode)

Two role-bound subagents live in `.claude/agents/` and run inside the same Claude Code session:

| Subagent | File | Tools allowed | Cannot |
|---|---|---|---|
| **architect** | `.claude/agents/architect.md` | Read, Write, Glob, TaskCreate/Update/List, AskUserQuestion | Bash, Grep, Edit, read source files (blocked by hook) |
| **worker** | `.claude/agents/worker.md` | Read, Write, Edit, Glob, Grep, Bash, TaskCreate/Update/List, AskUserQuestion | Read PROJECT.md / SOUL.md / CHARACTER.md (vision docs) |

Enforcement is structural: a `PreToolUse` hook (`scripts/architect-guard.sh`) hard-blocks Read/Glob on `src/` paths when the architect marker is active, so even a misbehaving model cannot bypass the envelope.

### Claude Code Skills (grouped by layer)

**Chủ nhà layer** — vision, intake, routing, decisions:

| Skill | Location | Purpose |
|---|---|---|
| `/idea` | `.claude/skills/idea/` | Intake new ideas, route into the right BACKLOG.md section (Active / Next / Open / Park). |
| `/init` | `skills/init/` | **0→1 only.** Vision capture for new project (empty folder → docs/PROJECT.md, SOUL.md, CHARACTER.md skeleton). |
| `/insight` | `skills/insight/` | Distill raw research / user interviews / competitor observations into structured bullets for PROJECT.md / SOUL.md / CHARACTER.md. |
| `/route` | `skills/route/` | Classify inbound request: code / marketing / design / strategy / skip. Produces 5-bullet brief for Architect. |
| `/decide` | `skills/decide/` | Trade-off triage. Present 2-3 concrete options with user-visible impact, recommend one. |

**Kiến trúc sư layer** — spec what gets built (docs-only access, no code):

| Skill | Location | Purpose |
|---|---|---|
| `/plan` | `skills/plan/` | Read vision + guide docs → write phiếu (ticket) in `phieu/TICKET_TEMPLATE.md` format with Task 0 verification anchors for Thợ to grep-verify. |
| `/forge` | `skills/forge/` | **Recipe library extension.** Research official docs + write new recipe to `recipes/<category>/<name>.md` when blueprint demands a recipe library doesn't have yet. |

**Thợ layer** — execute + ship (full code access):

| Skill | Location | Purpose |
|---|---|---|
| `/verify` | `skills/verify/` | Task 0 grep-first: verify every file/function/constant anchor in the phiếu against real code BEFORE coding. |
| `/apply` | `skills/apply/` | **0→1 only.** Apply 1 recipe from `recipes/` library — auto-generate sub-phiếu P000.N, run Task 0, execute steps, verify, commit. |
| `/review` | `skills/review/` | Staff-engineer review before merge — SQL injection, N+1 queries, auth bypass, logic bugs. |
| `/qa` | `skills/qa/` | QA lead — run tests, find bugs, fix with regression tests, verify. |
| `/ship` | `skills/ship/` | Release engineer — full ship pipeline (test → commit → PR → deploy → canary). |
| `/retro` | `skills/retro/` | Weekly retrospective — shipping velocity, hotspots, patterns, action items. |

One skill = one layer + one responsibility. Skills never span layers. See [`docs/LAYERS.md`](./docs/LAYERS.md) for boundaries and the 2-tier authority split (architectural vs detail).

`skills/` are project-agnostic and copied into `~/.claude/skills/` for global use. `.claude/skills/idea/` is a project-local skill that ships alongside the v2 subagent envelope.

### Phiếu — the ticket workflow

The spine that connects Kiến trúc sư and Thợ. Every non-trivial change goes through a phiếu (Vietnamese for "ticket"):

- Format: `<type>/P<NNN>-<slug>` — e.g. `feat/P042-user-export`
- Lives at `docs/ticket/P<NNN>-<slug>.md` in the project
- Written by Kiến trúc sư (using `/plan` or the `architect` subagent), executed by Thợ
- Discovery Report appended to `docs/DISCOVERIES.md` after each ticket closes

Shell function `phieu <slug>` (sourced from `phieu/phieu.sh`) creates worktree + branch + ticket file in one command, using a per-project counter for unique IDs. See [`phieu/README.md`](./phieu/README.md).

### Vision docs — Chủ nhà's foundation

Before any phiếu can be written, Chủ nhà must maintain:

- `BACKLOG.md` — live work-in-progress list (Active sprint / Next sprint / Open backlog / Park). The forcing function for v2: Architect refuses to write phiếu for items not in Active sprint.
- `PROJECT.md` — what the product is (vision, personas, monetization, architecture)
- `SOUL.md` — why it exists (philosophy, positioning, 3 hard lines, anti-product)
- `CHARACTER.md` — voice / persona (if the product has an AI character)

Skeletons for `PROJECT.md` / `SOUL.md` / `CHARACTER.md` / `VOICE.md` / `TEST_CASES.md` / `DESIGN_SPEC.md` live in [`phieu/VISION_TEMPLATES/`](./phieu/VISION_TEMPLATES/). The BACKLOG skeleton is in [`templates/BACKLOG_template.md`](./templates/BACKLOG_template.md). Copy into your project's `docs/` on day 1, fill iteratively as research matures. Use `/insight` to distill raw material into vision docs and `/idea` to feed BACKLOG.

The voice / character / test-cases / design-spec templates were harvested from a real production app (`tarot`) — they encode patterns proven at scale: phenotype tables, prompt-engineer-ready voice patterns, refusal templates, anti-pattern diagnostics, P0/P1/P2 test tiers, voice-↔-design traceability. Use them when your product has strong character voice; skip when it doesn't (e.g. a B2B dashboard).

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

### Relay Protocol — Chủ nhà as the courier (Web Project mode)

In v1 / Web Project mode, Kiến trúc sư (Claude Web Project) and Thợ (Claude Code) are **separate sessions** — they cannot talk directly. When Thợ hits an architectural blocker mid-ticket, Chủ nhà routes between them manually.

The 2-3 minute protocol is in [`phieu/RELAY_PROTOCOL.md`](./phieu/RELAY_PROTOCOL.md). v2 Subagent mode bypasses this — orchestrator spawns Architect → Worker in the same session, no copy-paste required.

### Integrations

| Integration | What it does |
|-------------|-------------|
| **GitHub Actions canary** | Post-deploy health check in CI pipeline |
| **Pre-commit hook** | type-check + docs-gate + (v2) BACKLOG and Discovery enforcement |
| **SessionStart banner** | Surfaces BACKLOG Active sprint every time Claude Code opens (v2) |
| **Architect guard** | `PreToolUse` hook hard-blocks code reads when architect marker active (v2) |
| **Jarvis uptime monitor** | Ping production every 10 min, Telegram alert on down |
| **MCP server** | Tools for Claude Desktop/Code integration via `ship serve` / `guard serve` / `vps serve` |

## Install

For the v2 subagent envelope (recommended), see [`INSTALL.md`](./INSTALL.md) — 5-minute install with verify steps.

For Rust CLIs and global skills:

### Prerequisites
- Rust toolchain (`rustup`)
- `gh` CLI (for PR creation)
- Claude Code v2.1+ (for subagents + SessionStart hook)

### Ship CLI
```bash
git clone https://github.com/aspelldenny/ship.git
cd ship && cargo install --path .
```

### docs-gate / guard / vps
```bash
git clone https://github.com/aspelldenny/docs-gate.git && (cd docs-gate && cargo install --path .)
git clone https://github.com/aspelldenny/guard.git && (cd guard && cargo install --path .)
git clone https://github.com/aspelldenny/vps.git && (cd vps && cargo install --path .)
vps init                  # generate ~/.vps.toml with your SSH + project paths
```

### Skills (global)
```bash
# Chủ nhà layer
cp -r skills/init    ~/.claude/skills/init
cp -r skills/insight ~/.claude/skills/insight
cp -r skills/route   ~/.claude/skills/route
cp -r skills/decide  ~/.claude/skills/decide
# Kiến trúc sư layer
cp -r skills/plan    ~/.claude/skills/plan
cp -r skills/forge   ~/.claude/skills/forge
# Thợ layer
cp -r skills/verify  ~/.claude/skills/verify
cp -r skills/apply   ~/.claude/skills/apply
cp -r skills/review  ~/.claude/skills/review
cp -r skills/qa      ~/.claude/skills/qa
cp -r skills/ship    ~/.claude/skills/ship
cp -r skills/retro   ~/.claude/skills/retro
```

> `/idea` is project-local (lives in `.claude/skills/idea/`) — it ships with the v2 subagent envelope per project, not globally.

### Phiếu shell function
```bash
echo "source ~/path/to/sos-kit/phieu/phieu.sh" >> ~/.zshrc
source ~/.zshrc
phieu-init ~/my-project   # initialize phiếu workflow in a project
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

### v2 Subagent mode (default)

```bash
# Morning — Thợ checks production
ship canary

# Open Claude Code → SessionStart hook prints BACKLOG Active sprint
claude

# -- Inbound idea arrives --
/idea                   # Chủ nhà routes idea into BACKLOG (Active / Next / Open / Park)

# Pick an Active sprint item, then in main session (orchestrator):
"Spawn architect subagent to write phiếu for item X"
                        # Architect reads docs (PROJECT/SOUL/BACKLOG/DISCOVERIES),
                        #   writes docs/ticket/P<NNN>-<slug>.md with Task 0 anchors.
                        # Hook blocks any attempt to read src/ — envelope is structural.

# Chủ nhà reviews phiếu, types "go"
"Spawn worker subagent to execute P<NNN>-<slug>.md"
                        # Worker runs Task 0 (grep anchors), codes, tests,
                        #   appends Discovery Report, commits.

# Pre-commit hook enforces: type-check + docs-gate + BACKLOG + Discovery
ship                    # full pipeline → PR → deploy → canary
```

### v1 Web Project mode (alternative)

```bash
# Layer 1: Chủ nhà — in your preferred Claude Code/Web
/route                  # code? marketing? design? skip? outputs 5-bullet brief
/insight                # if raw context needs distillation → vision doc update

# Layer 2: Kiến trúc sư — IN CLAUDE WEB PROJECT (separate session)
/plan                   # reads vision + guide docs (NOT code) → writes phiếu

# Layer 3: Thợ — IN CLAUDE CODE
phieu feat user-export  # creates worktree + branch + phiếu file
                        # Chủ nhà pastes phiếu content from Web into this file
/verify                 # Task 0 — grep every anchor against real code
/review                 # logic bugs, SQL injection, auth bypass, N+1
/qa                     # run tests, find + fix bugs, verify
ship                    # test → docs-gate → commit → push → PR
ship canary             # verify production after merge
/retro                  # end-of-week retrospective

# Cross-session escalation goes through Chủ nhà as courier (RELAY_PROTOCOL.md)
ship learn add "always run migrations before deploy" -t deploy,db
```

Each step is single-layer. Handoffs between them are formatted (see [`docs/HANDOFF.md`](./docs/HANDOFF.md)) — not freestyle Slack threads.

## Architecture

```
sos-kit/
├── README.md                   # This file — entry point
├── INSTALL.md                  # v2 install guide (5-min, with verify)
├── CLAUDE.md                   # Contributor guide for Claude Code
├── .claude/                    # v2 subagent envelope (project-local)
│   ├── agents/
│   │   ├── architect.md        # Kiến trúc sư subagent (Read/Write/Glob, no Bash/Grep/Edit)
│   │   └── worker.md           # Thợ subagent (full code tools, no vision docs)
│   ├── skills/
│   │   └── idea/SKILL.md       # /idea intake skill (Chủ nhà tier)
│   └── settings.json           # Hooks: SessionStart banner + PreToolUse architect-guard
├── docs/
│   ├── PHILOSOPHY.md           # 6 principles
│   ├── LAYERS.md               # 3-role model (Chủ nhà / Kiến trúc sư / Thợ)
│   ├── HANDOFF.md              # Inter-layer handoff protocols
│   ├── COMPARISON.md           # SOS Kit vs gstack
│   └── SETUP.md                # Detailed install guide
├── phieu/                      # Ticket workflow — spine connecting Kiến trúc sư ↔ Thợ
│   ├── README.md               # Setup + philosophy
│   ├── TICKET_TEMPLATE.md      # Phiếu format with Task 0 Verification Anchors
│   ├── DISCOVERY_PROTOCOL.md   # Thợ → Kiến trúc sư feedback loop + mismatch classification
│   ├── RELAY_PROTOCOL.md       # Chủ nhà's courier workflow (Web Project mode)
│   ├── VISION_TEMPLATES/       # Day-1 skeletons for Chủ nhà (PROJECT, SOUL, CHARACTER, VOICE, TEST_CASES, DESIGN_SPEC)
│   │   ├── PROJECT_template.md
│   │   ├── SOUL_template.md
│   │   └── CHARACTER_template.md
│   └── phieu.sh                # Shell function: phieu / phieu-list / phieu-done / phieu-init
├── skills/                     # Global Claude Code skills (one per layer+responsibility)
│   ├── insight/SKILL.md        # Chủ nhà — distill raw research → vision docs
│   ├── route/SKILL.md          # Chủ nhà — classify inbound
│   ├── decide/SKILL.md         # Chủ nhà — trade-off triage
│   ├── plan/SKILL.md           # Kiến trúc sư — write phiếu (docs-only)
│   ├── verify/SKILL.md         # Thợ — Task 0 grep-first
│   ├── review/SKILL.md         # Thợ — code review
│   ├── qa/SKILL.md             # Thợ — QA verification
│   ├── ship/SKILL.md           # Thợ — release pipeline
│   └── retro/SKILL.md          # Thợ — retrospective
├── templates/
│   └── BACKLOG_template.md     # BACKLOG.md skeleton (Active / Next / Open / Park)
├── configs/                    # .ship.toml examples per stack
│   ├── nextjs.toml
│   ├── flask.toml
│   ├── rust.toml
│   └── python.toml
├── hooks/
│   └── pre-commit              # type-check + docs-gate + (v2) BACKLOG + Discovery enforcement
├── scripts/
│   ├── architect-guard.sh      # PreToolUse hook — block code reads when architect active
│   └── session-start-banner.sh # SessionStart hook — show BACKLOG on session open
└── integrations/
    ├── github-actions/         # Canary workflow snippet
    └── jarvis/                 # Uptime monitor for Telegram bots
```

## Comparison with gstack

How SOS Kit differs from gstack and when to pick each → [`docs/COMPARISON.md`](./docs/COMPARISON.md).

## Philosophy

1. **One command per step.** If it takes more than one command, automate it.
2. **Gates, not guidelines.** Pre-commit hooks enforce quality. `tools` allowlists enforce role envelope. Don't rely on memory or model discipline.
3. **Cross-project learnings.** A mistake in project A should prevent the same mistake in project B.
4. **Rust for tools, AI for judgment.** CLI tools are fast, deterministic, zero-dependency. AI skills + subagents handle the fuzzy stuff (review, QA, retro, planning).
5. **Solo-first.** No multi-user, no team features, no overhead. Every feature serves one person shipping fast.
6. **Role separation is a context envelope, not workflow ergonomics.** Architect cannot read code because LLMs hallucinate proportional to irrelevant context. Worker cannot read vision because vision drifts implementation. The boundary is structural alignment, not bureaucracy.

See [`docs/PHILOSOPHY.md`](./docs/PHILOSOPHY.md) for the full set.

## License

MIT
