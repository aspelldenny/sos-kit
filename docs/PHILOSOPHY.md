# Philosophy — SOS Kit

The v2 version of this page (information envelopes, phiếu, approval gates) is archived at `archive/v2/docs/PHILOSOPHY-v2.md`. What changed and why: `docs/research/HARNESS_SURVEY_2026-10-08.md`.

## The problem

One person building real software with AI agents skips steps, not from laziness but from friction. Each check is "just two minutes"; together they become "later", and later never comes. Agents add a second problem: they produce a lot of plausible work quickly, and plausible is not the same as correct.

## Principle 0 — The human holds both ends

> *"Whose house is it? The owner's. When AI makes mistakes, the person losing money and time is still you, so the owner does the final acceptance."*

Chủ nhà owns intent and taste at the start and acceptance at the end. Agents own the middle: plan, build, check, repair. Everything else in the kit exists to make that middle trustworthy enough that the owner's time goes to the two ends, not to relaying messages between agents or approving every step.

## Principles

**1. Gates, not reminders.** A rule that must always hold becomes a check that blocks and says how to fix the failure, never a sentence in a prompt. A gate that cannot run blocks; a missing tool is not a pass. Judgment stays guidance.

**2. Add only what a real failure demands.** Every rule, role line, hook and file points to an observed failure or a recorded decision. When a stronger model no longer needs a piece of scaffolding, remove it and write down what protection was lost. v3 removed the ticket-per-change workflow, debate rounds, approval gates and the "architect may not read code" envelope because the evidence from four shipped apps showed they cost more than they caught.

**3. The checks live where every agent passes.** The boundary is git and plain command-line tools, not one vendor's hook format. Claude Code, Codex, another harness or a person at the keyboard all meet the same gates at commit. Per-agent adapters only give earlier feedback.

**4. A view the builder does not have.** The builder cannot review itself. Independence comes from a fresh context, a different vantage point (the running product, from the outside in), and sometimes a different model family. It is spent at the moments that matter (a journey first connects, data or money is touched, before handoff), not on every slice. A stronger advisor helps the builder choose a direction when it is stuck; it does not replace review.

**5. Evidence over assertions.** "Done" means a repeatable check on the combined result, with the command and its output. An old green log is not current quality. A test that seeds what production should produce proves nothing.

**6. Tools for rules, models for judgment.** Deterministic checks belong in code: fast, testable, the same every time. Models handle what rules cannot: semantics, taste, finding what nobody wrote down.

**7. Solo-first.** No team ceremony. Every feature serves one person shipping real software.

## Garbage in — check the input, not just the output

Most gates watch what a model produces. Few watch what it consumes. When output is bad and the prompt looks fine, suspect the input first: is it decrypted, from the right source, complete, the shape you assume?

Origin (Soul Signature, 2026-06-05): a monthly letter came out flat. Three reviewers proposed prompt fixes; half a day went into them. The cause was input: the test harness fed ciphertext, and the letter was built from a one-line digest instead of the real conversation. Production had always been fine. *Code clears the table first; the model writes after.* An input gate is on the backlog.

## What this is not

- Not an AI coding assistant: the agents do the coding; the kit organizes how they are directed and checked.
- Not a planning methodology: it starts when Chủ nhà has decided what to build.
- Not a CI/CD replacement, and not a team tool.
- Not project scaffolding: `recipes/` hold verified patterns to apply, not whole-stack templates.
