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

### How envelopes are enforced

SOS Kit prevents these failures *structurally*. Each role has a different `allowedTools` envelope:

- **Quản đốc (Layer 0, orchestrator persona for the main Claude Code session)** — spawns subagents, drives state machine, invokes Skills. NO source-code reads (envelope guard); NO production code edits. Sees the phiếu, the BACKLOG, the Debate Log — enough to route, not enough to second-guess.
- **Kiến trúc sư (Architect subagent)** — `Read`, `Write`, `Glob`. NO Bash, NO Grep, NO Edit on source. Reads docs (PROJECT/SOUL/CHARACTER/guides/BACKLOG/DISCOVERIES) but cannot grep source code. Writes phiếu with Task 0 anchors — every assumption framed as "Worker verify at file:line."
- **Thợ (Worker subagent)** — full code tools (`Read`, `Write`, `Edit`, `Glob`, `Grep`, `Bash`). Cannot Read vision docs (PROJECT.md / SOUL.md / CHARACTER*.md) — prevents silent re-architecture from "knowing" the why beyond the phiếu.

Three envelopes, three accountability surfaces. Plus Layer 0 (Quản đốc) routing between them. The same human drives all four mental modes; the AI assisting each one sees only what that mode needs.

### Why "share context for efficiency" is the trap

The intuitive optimization — give every role more context "so it can help better" — is exactly the leak we prevent. Architect with code access invents anchors. Worker with vision access drifts scope. Quản đốc with source-code access starts coding instead of spawning Worker.

The envelopes are not a workflow inconvenience; they are the **alignment surface**. Removing them removes the alignment.

### Why role separation, not just prompt discipline

Prompt discipline ("please don't read code, Architect") fails because LLMs reach for what they have access to. The fix is structural: don't ship the tool. `allowedTools: [Read, Write, Glob]` in Architect's frontmatter, plus a `PreToolUse` hook (`scripts/architect-guard.sh`) hard-blocking `Read` on `src/` paths when the architect marker is active. Even a misbehaving model cannot bypass.

This is also why we don't lean on "trust the model": the hallucination-by-irrelevant-context failure mode is **load-bearing**, not occasional. The 3-role split is the minimum viable structure for catching it.

## Six Operational Principles

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

In v2.1+ Subagent mode, a 4th persona — **Quản đốc** (Layer 0, the main Claude Code session as orchestrator) — automates the relay between Kiến trúc sư and Thợ. Quản đốc is not a 4th *human* role; it's the AI persona surfacing the orchestrator state machine to Sếp. The human still wears three hats. See [`LAYERS.md`](./LAYERS.md) for Layer 0 specifics.

SOS Kit enforces role separation through **distinct skills per layer** — `/init` `/idea` `/insight` `/route` `/decide` for Chủ nhà, `/plan` `/forge` for Kiến trúc sư, `/verify` `/apply` `/review` `/qa` `/ship` `/retro` for Thợ. Different prompts, different mental modes, same human.

Handoffs between layers are **formalized** (see [`HANDOFF.md`](./HANDOFF.md)): insight briefing into vision docs, 5-bullet brief from Chủ nhà to Kiến trúc sư, phiếu (ticket) from Kiến trúc sư to Thợ, Discovery Report back up, blocker escalation via Chủ nhà as courier. No freestyle, no "just ping me." Format prevents context loss — the only thing more expensive than overhead is redundant work from misaligned assumptions.

See [`LAYERS.md`](./LAYERS.md) for role boundaries and anti-patterns.

## The garbage-in blind spot — gate the input, not just the output

Every gate in this kit watches what the LLM **produces**: docs-gate on docs, giám sát (boundary-check) on PR diffs, doctor on runtime state, the prompt-reviewer on character drift. None watches what the LLM **consumes**.

That asymmetry is a blind spot. When output quality is bad, the institutional reflex points where the tools point — at the prompt, the character, the model. Nobody reflexively asks *"is the input clean?"* So a dirty-input bug masquerades as a prompt bug, and you burn cycles tuning the prompt while the real fault is upstream.

**Origin (Soul Signature, 2026-06-05).** A monthly "letter" feature produced flat, generic output. Three reviewers — the founder plus two external LLMs — independently proposed prompt fixes: loosen the closing, soften the synthesis, restructure the voice. Half a day of prompt archaeology. The root cause was two layers of dirty input: (1) the local test harness fed *ciphertext* — it never applied the decryption extension, so the model received encrypted gibberish; and (2) the letter was assembled from a tiny memory-digest (a lossy one-line summary built for a *different* job — in-chat recall) instead of the user's real conversation. The prompt was never the problem. Production, which decrypted correctly, had always been fine. Garbage in, garbage out — regardless of how smart the model or how good the prompt.

**The doctrine:** before debugging output quality, verify input integrity — is it decrypted, from the right source, complete, the shape you assume it is? A clever model fed garbage produces *clever* garbage, which is worse than obvious garbage because it reads plausible. *Code dọn bàn sạch trước; LLM ngồi viết sau* — code clears a clean table first; the LLM writes after. Don't make the LLM do the janitorial work of filtering messy input; do that deterministically in code, then hand it clean material.

**The gap this names:** the kit guards everything the LLM emits and nothing it ingests. A future *data sentinel* — a reflex or gate that verifies input integrity (decryption, source correctness, completeness, expected shape) before output is judged — would close it. Until then the reflex stays manual but mandatory: **when output is bad and the prompt looks fine, suspect the input first.**

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
