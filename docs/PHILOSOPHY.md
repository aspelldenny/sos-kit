# Philosophy — SOS Kit

## The Problem

Solo developers skip steps. Not because they're lazy — because each step has friction. Running tests, updating docs, reviewing your own code, creating PRs, checking production after deploy — each one is "just 2 minutes" but together they add up to "I'll do it later."

Later never comes. Code ships without review. Docs drift from reality. Production breaks at 2 AM and nobody knows until a user complains.

## The Insight

In early 2026, Garry Tan (CEO of Y Combinator) published gstack — a framework that turns Claude Code into a virtual 20-person engineering team. 54,000 GitHub stars in days.

Around the same time, independently, this kit was being built to solve the same problem from a different angle.

The convergence isn't coincidence. When you face the constraint of "one person, production software, real users" — you arrive at the same structure:

1. **Separate roles.** Don't let the same brain design, build, and verify.
2. **Gates between steps.** Each step must pass before the next begins.
3. **Automate the boring parts.** Tests, commits, deploys, health checks.
4. **Learn from mistakes.** Record patterns so you don't repeat them.

## The Difference

gstack replaces your entire methodology with 31 AI skills — from ideation (/office-hours) to retrospective (/retro). It's opinionated about how you should think, plan, and build.

SOS Kit is **just the tail of the pipeline** — from code-ready to production-verified. It doesn't tell you how to plan or think. It trusts that you have your own methodology for that. It just makes sure the code you wrote actually ships safely.

## Principle 0 — Accountability stays human

> *"Whose house is it? The owner's. Does AI bear responsibility? No. When AI makes mistakes, the person losing money and time is still you, so the owner must do the final acceptance."*

Every other principle in this kit serves this one. Roles, envelopes, gates, phiếu, Discovery Reports — they exist to give the human Owner clean checkpoints to inspect and reject AI output. They do NOT exist to remove the human from the loop.

The Owner (Chủ nhà) is structurally required to be human, by design, forever. AI cannot bear the cost of being wrong about your product — you can. So you stay in the chair where it matters: writing vision, approving phiếu, and final acceptance (nghiệm thu) before ship.

The envelopes (Architect can't grep code, Worker can't read vision) exist to make AI output *inspectable* by the Owner. The 3-role split exists to make accountability *unambiguous* — when something ships wrong, the trail is clear: Owner approved this phiếu, Architect wrote these anchors, Worker executed these tasks. No diffusion.

This is the deepest reason SOS Kit refuses "full autonomy" framing even when technically feasible. The human cost of mistakes cannot be delegated to systems that don't pay it.

## The deeper principle: information envelopes (alignment engineering)

The 3-role split isn't only about workflow. It's about **information envelope engineering for LLM alignment**.

LLMs hallucinate in proportion to how much *irrelevant* context they see. An Architect-LLM with grep access invents implementations that "look right" but cite phantom functions. A Worker-LLM with full vision-doc access silently re-architects "while it's there." Both failures are caused by **information leakage across role boundaries**, not by lack of skill.

SOS Kit prevents these failures *structurally*: each role has a different `allowedTools` envelope — Architect reads docs but cannot grep code; Worker reads code but cannot see vision strategy. The same human drives all three, but the AI assisting each role sees only what that role needs. Three envelopes, three accountability surfaces.

This is why we don't share context "for efficiency." Shared context is exactly the leak we're preventing.

## Six Principles

### 1. One Command Per Step
If shipping requires 5 manual steps, you'll eventually skip one. `ship` does all 5 in sequence with gates.

### 2. Gates, Not Guidelines
A pre-commit hook that blocks bad commits is worth more than a wiki page that says "please run tests." docs-gate fails the commit if docs aren't updated. The pipeline stops if tests fail. Enforcement, not hope.

### 3. Cross-Project Learnings
`ship learn add "Prisma needs manual ALTER TABLE on VPS"` saves a lesson that applies next time you touch any project with Prisma. Learnings compound across projects, not just within one.

### 4. Rust for Tools, AI for Judgment
- **Rust CLI:** Fast (< 5ms startup), small binary, deterministic, zero runtime dependency. Perfect for gates and automation.
- **Claude Skills:** Fuzzy judgment — reviewing code for logic bugs, finding edge cases in QA, summarizing a week's work. AI handles what rules can't.

### 5. Solo-First
No multi-user auth. No team dashboards. No Slack integrations. Every feature serves exactly one person shipping code to production. This constraint keeps the kit small, fast, and focused.

### 6. Separate Roles, Separate Brains
One person running a software business wears three hats: **Chủ nhà** (owner — what to build, what to reject, maintain vision), **Kiến trúc sư** (architect — how to spec it, docs-only access), **Thợ** (worker — execute, ship, report reality back). When one brain does all three at once, you get half-finished features, scope explosions, and architectural drift.

SOS Kit enforces role separation through **distinct skills per layer** — `/insight` `/route` `/decide` for Chủ nhà, `/plan` for Kiến trúc sư, `/verify` `/review` `/qa` `/ship` `/retro` for Thợ. Different prompts, different mental modes, same human.

Handoffs between layers are **formalized** (see [`HANDOFF.md`](./HANDOFF.md)): insight briefing into vision docs, 5-bullet brief from Chủ nhà to Kiến trúc sư, phiếu (ticket) from Kiến trúc sư to Thợ, Discovery Report back up, blocker escalation via Chủ nhà as courier. No freestyle, no "just ping me." Format prevents context loss — the only thing more expensive than overhead is redundant work from misaligned assumptions.

See [`LAYERS.md`](./LAYERS.md) for role boundaries and anti-patterns.

## What This Is Not

- **Not a project scaffolder.** Use your own templates.
- **Not a CI/CD replacement.** It complements GitHub Actions, not replaces it.
- **Not an AI coding assistant.** Claude Code does the coding. SOS Kit organizes how you direct it.
- **Not an external planning methodology.** Shape Up, Vibecode, product discovery frameworks — those live above SOS Kit. SOS Kit starts where Chủ nhà has decided "we're doing this" and ends at "it's shipped and healthy in production."
- **Not a team tool pretending to work solo.** Every feature here exists because one person needed it. If it smells like team ceremony (stand-ups, sprint planning poker, architecture review boards), it's out of scope.

## Scope — what SOS Kit does and does not govern

SOS Kit governs **what you build and how you verify it**. It does NOT govern:

- **SSH / VPS authentication** — your own key management, not part of the kit
- **Multi-machine sync** — use git the way you would anyway
- **Server-side state** — production ops are `vps` CLI's job (a separate kit)
- **Time-based planning** — SOS Kit is wave-based (sprint = "until done", not "until Friday")
- **Project scaffolding** — bring your own templates for new projects

Keep these concerns at your infrastructure layer, not inside SOS Kit. Mixing them dilutes the kit's clarity about what it is responsible for.
