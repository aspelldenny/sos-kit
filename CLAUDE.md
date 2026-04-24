# CLAUDE.md — SOS Kit

> Read this before editing anything in this repo.
> This is a **meta-kit** (documentation + templates + skill definitions). It is **not** a production app.

## What this repo is

SOS Kit = "Solo Operating System" — a distribution center that packages a **3-role workflow** for one-person software teams: **Chủ nhà** (owner / vision / routing), **Kiến trúc sư** (architect / ticket writer / docs-only), **Thợ** (worker / code executor).

What's inside:
- `docs/LAYERS.md` — the 3-role model, access matrix, 2-tier authority, anti-patterns
- `docs/HANDOFF.md` — 5 handoff protocols (insight briefing, routing, phiếu, escalation, discovery)
- `docs/PHILOSOPHY.md` — 6 principles (role separation is #6)
- `docs/SETUP.md` — install guide for Rust tools + skills + phiếu shell function
- `phieu/` — ticket workflow backbone
  - `README.md`, `TICKET_TEMPLATE.md`, `phieu.sh` — the core
  - `DISCOVERY_PROTOCOL.md` — Thợ → Kiến trúc sư feedback + mismatch classification
  - `RELAY_PROTOCOL.md` — Chủ nhà's courier workflow (Thợ cannot ping Kiến trúc sư directly)
  - `VISION_TEMPLATES/` — day-1 skeletons for `PROJECT.md`, `SOUL.md`, `CHARACTER.md`
- `skills/` — Claude Code skills grouped by layer (3 Chủ nhà + 1 Kiến trúc sư + 5 Thợ = 9 total)
- `configs/` — `.ship.toml` templates per stack (nextjs, flask, rust, python)
- `hooks/pre-commit` — git hook script (type-check + docs-gate)
- `integrations/` — CI snippets (GitHub Actions canary) + Telegram uptime monitor
- `README.md` — entry point for new users

## What this repo is NOT

- **Not a runtime binary source.** The Rust CLIs (`ship`, `docs-gate`, `guard`, `vps`) live in their own repos. This repo only references them.
- **Not a project scaffolder.** It doesn't generate your app; it ships your app.
- **Not a planning methodology.** Use your own (Shape Up, Vibecode, whatever). SOS Kit picks up after "code is ready."
- **Not a place for experimental features.** If a skill or config hasn't been used on a real project for ≥2 weeks, don't add it here.

## Repo structure

```
sos-kit/
├── README.md               # User-facing entry point — MUST reflect reality
├── CLAUDE.md               # This file — for Claude Code contributors
├── docs/
│   ├── LAYERS.md           # 3-role model (Chủ nhà / Kiến trúc sư / Thợ). Foundation doc.
│   ├── HANDOFF.md          # 5 inter-layer protocols (insight, routing, phiếu, escalation, discovery)
│   ├── PHILOSOPHY.md       # Stable — 6 principles, change carefully
│   └── SETUP.md            # Install guide — MUST match actual binary names + cargo paths
├── phieu/                  # Ticket workflow backbone
│   ├── README.md           # Setup + how to use phiếu workflow
│   ├── TICKET_TEMPLATE.md  # Phiếu format (header, Task 0, tasks, nghiệm thu)
│   ├── DISCOVERY_PROTOCOL.md  # Thợ → Kiến trúc sư feedback loop + mismatch classification (Tầng 1 vs Tầng 2)
│   ├── RELAY_PROTOCOL.md   # Chủ nhà's courier workflow (Thợ ↔ Kiến trúc sư cross-session)
│   ├── VISION_TEMPLATES/   # Day-1 skeletons — Chủ nhà copies + fills
│   │   ├── PROJECT_template.md
│   │   ├── SOUL_template.md
│   │   └── CHARACTER_template.md
│   └── phieu.sh            # Shell function: phieu / phieu-init / phieu-done / phieu-list
├── skills/                 # One skill per layer+responsibility, never spans layers
│   ├── insight/SKILL.md    # Chủ nhà — distill raw research → vision docs
│   ├── route/SKILL.md      # Chủ nhà — classify inbound
│   ├── decide/SKILL.md     # Chủ nhà — trade-off triage
│   ├── plan/SKILL.md       # Kiến trúc sư — write phiếu (docs-only, no code access)
│   ├── verify/SKILL.md     # Thợ — Task 0 grep-first (gate before coding)
│   ├── review/SKILL.md     # Thợ — code review
│   ├── qa/SKILL.md         # Thợ — QA verification
│   ├── ship/SKILL.md       # Thợ — release pipeline
│   └── retro/SKILL.md      # Thợ — retrospective
├── configs/                # .ship.toml examples per stack
├── hooks/pre-commit        # type-check + docs-gate
└── integrations/
    ├── github-actions/     # canary.yml snippet
    └── jarvis/             # uptime_monitor.py
```

## Common tasks

### Edit a skill (`skills/<name>/SKILL.md`)
1. Change the markdown
2. Verify the skill still belongs to exactly ONE layer (Chủ nhà / Kiến trúc sư / Thợ). If it spans layers, split it into two.
3. Test: `cp -r skills/<name> ~/.claude/skills/<name>` in a real project, then invoke `/<name>` in Claude Code
4. If the skill's role or trigger changed, update the row in `README.md` "Claude Code Skills" table AND the table in `docs/LAYERS.md`

### Edit phiếu template or shell function (`phieu/`)
1. If `TICKET_TEMPLATE.md` changes, update `docs/HANDOFF.md` section "Handoff 2 — Kiến trúc sư → Thợ" which references the required sections
2. If `phieu.sh` changes function behavior (not just bug fix), update `phieu/README.md` + any skill that invokes it
3. Test: `phieu-init ~/some-test-repo` + create a phiếu end-to-end

### Edit vision doc templates (`phieu/VISION_TEMPLATES/`)
1. Keep templates generic — no project-specific wording. Placeholders use `<angle brackets>`.
2. If you add/remove a section in a template, update `/insight` skill's "Target section" list in `skills/insight/SKILL.md`.
3. Also reflect the change in `docs/HANDOFF.md` Handoff 0 section.

### Edit RELAY_PROTOCOL.md
1. If relay format changes (e.g. escalation fields), update the example session in `skills/decide/SKILL.md` (Worker-side escalation format must match).
2. Also update `docs/HANDOFF.md` Handoff 3 format.

### Add a new layer-specific skill
1. Decide layer FIRST. If unsure, read `docs/LAYERS.md` "Which layer am I in right now?"
2. Create `skills/<name>/SKILL.md`
3. Add to `README.md` skill table under the right layer section
4. Add to `docs/LAYERS.md` skills map
5. If the skill introduces a new handoff, document it in `docs/HANDOFF.md`

### Add a new stack config (`configs/<stack>.toml`)
1. Create the `.toml` template
2. Add a per-stack subsection in `docs/SETUP.md`
3. Add an expandable example in `README.md` "Example configs"

### Edit docs
- `README.md` — any tool/skill/integration table MUST match actual folders and binaries. Contributor onboarding breaks if they drift.
- `docs/PHILOSOPHY.md` — stable. Don't add a 6th principle without strong justification. The 5 principles are load-bearing.
- `docs/SETUP.md` — must match real binary names and `cargo install` instructions.

### Add a new integration (`integrations/<name>/`)
1. Create a folder with a README inside explaining the purpose + setup
2. Update root `README.md` "Integrations" table
3. No secrets in committed files (tokens, webhook URLs) — use env var placeholders

## Rules

1. **No runtime code in this repo.** Rust source belongs in their own repos (`ship`, `docs-gate`, `guard`, `vps`). This repo is documentation, templates, and skill markdown only. `phieu/phieu.sh` is an exception — a single shell function file users source — but it does no computation beyond git and file ops.
2. **Every new file must justify its existence.** No `TODO.md`, no placeholder directories, no "might use later" stubs.
3. **No hardcoded personal paths.** Replace `/Users/<name>/...` with `~/` or a generic example before committing.
4. **README is the single source of truth.** If a tool is listed in `README.md` but not in `docs/SETUP.md`, that's a bug. Fix the gap in both places. Same for `docs/LAYERS.md` skill table.
5. **Skills are for repeated workflows, not one-off tasks.** If a skill only applies to one project, keep it in that project's `.claude/skills/`, not here.
6. **One skill, one layer, one responsibility.** If you're tempted to make a skill that "routes AND plans" or "plans AND implements," stop — split it. Layer leaks are anti-pattern #1.
7. **Handoffs stay formatted.** If you're tempted to add a new inter-layer handoff ("Architect pings Worker directly on Slack"), document the format in `docs/HANDOFF.md` first. Freestyle handoffs = context loss.

## Related repos (maintained separately)

| Repo | Local path | Role |
|---|---|---|
| `ship` | `~/ship` | Release pipeline CLI — test, commit, PR, deploy, canary |
| `docs-gate` | `~/docs-gate` | Pre-commit documentation enforcement |
| `guard` | `~/guard` | Pre-deploy infrastructure gate — schema drift, env sync |
| `vps` | `~/vps` | Production ops CLI — logs, status, restart, metrics |

Changes to these belong in their respective repos, not here. SOS Kit only references and documents them.

## Language

Public-facing docs (`README.md`, `docs/SETUP.md`, `docs/PHILOSOPHY.md`, `SKILL.md` files) are English — this repo may be published open-source.

Internal conversations with the maintainer (Denny / Nguyen) are in Vietnamese; see the maintainer's personal tarot/CLAUDE.md for that convention. This `CLAUDE.md` stays in English so external contributors can read it.

## Maintainer-only conventions

The maintainer (Denny) uses:
- **Phiếu workflow** — ticket IDs `P<NNN>-<slug>`, shell function `phieu`, worktree per ticket. Lives in his `~/.zshrc`, not in this repo (yet).
- **Vietnamese communication with Claude** — em/anh xưng hô, Vietnamese in chat + commits in English.

If you're the maintainer talking to Claude, that context applies. If you're an external contributor, follow the repo's English + PR-based flow as described in `README.md`.
