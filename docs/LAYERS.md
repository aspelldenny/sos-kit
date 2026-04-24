# Layers — The 3-Role Solo OS

> SOS Kit is not a shipping framework. It is an operating system for one person running a full software business. That means three separate roles — Chủ nhà, Kiến trúc sư, Thợ — each with different access, different authority, different handoffs.

## Why three roles

A solo developer running serious work wears three hats every day:

- **Chủ nhà (Owner / CEO-PM hybrid)** — decides what's worth doing, maintains vision, approves plans, vetoes scope creep, relays between the other two.
- **Kiến trúc sư (Architect)** — reads *docs only* (not code), writes tickets (phiếu), decides architecture big-picture.
- **Thợ (Worker)** — reads code + docs, executes the ticket, challenges the Architect when code reality differs from docs.

If one brain plays all three at once, you get half-finished features, scope explosions, and architectural drift.

The fix is **role separation, even when the same human is in every chair**. Different tools, different contexts, different skills — so the brain snaps into a different mode.

## Access matrix — who can see what

| | Chủ nhà | Kiến trúc sư | Thợ |
|---|---|---|---|
| Vision/strategy docs (PROJECT, SOUL, CHARACTER) | ✏️ maintain | 📖 read | 📖 read |
| Code (src/, tests/) | 📖 read optional | ❌ **NO access** | ✏️ read+edit |
| Tickets (phiếu) | 📖 read, approve | ✏️ write | 📖 read, execute |
| Discovery Reports | 📖 read | 📖 read before next phiếu | ✏️ write |
| Running commands (bash, pnpm, git) | ❌ delegates | ❌ cannot | ✏️ runs |

**Critical**: Kiến trúc sư lives in Claude Web Project. No Bash, no Grep on source, no filesystem access beyond project's attached docs. This is why Task 0 grep-first + Discovery Report exist — they are the Architect's only connection to code reality.

## The 3 layers in detail

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 1 — CHỦ NHÀ (Owner / Router / Vision keeper)             │
│  Skills: /insight  /route  /decide                              │
│  Tools: Claude Code OR Claude Web (usually wherever the human is)│
│  Owns:                                                          │
│    • Vision docs (PROJECT.md, SOUL.md, CHARACTER.md)            │
│    • Inbound triage (user feedback, ideas, bug reports)         │
│    • Approve/veto on phiếu before Worker executes               │
│    • User-visible wording (email, UI copy — final cut)          │
│    • Scope / timeline / quality trade-offs                      │
│    • Relay between Architect ↔ Worker (Architect can't ping     │
│      Worker directly; Chủ nhà is the courier)                   │
│  Does NOT:                                                      │
│    • Write phiếu (that's Architect)                             │
│    • Implement (that's Worker)                                  │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2 — KIẾN TRÚC SƯ (Architect / Ticket writer)             │
│  Skills: /plan  /verify                                         │
│  Tools: Claude Web Project — docs access ONLY                   │
│  Owns:                                                          │
│    • Phiếu file with full context, tasks, constraints, Task 0   │
│    • Task 0 anchors (specify — Worker grep-verifies)            │
│    • File structure, routing, API shape, data flow, naming      │
│    • Read DISCOVERIES.md before each new phiếu                  │
│  Does NOT:                                                      │
│    • Grep code directly (no shell access)                       │
│    • Approve own phiếu for merge (that's Chủ nhà)               │
│    • Decide implementation detail (that's Worker's Tầng 2)      │
├─────────────────────────────────────────────────────────────────┤
│  Layer 3 — THỢ (Worker / Executor / Field reporter)             │
│  Skills: /verify  /review  /qa  /ship  /retro                   │
│  Tools: Claude Code — full shell + code access                  │
│  Owns:                                                          │
│    • Execute phiếu Nhiệm vụ after Task 0 passes                 │
│    • Run tests, commit, PR, deploy, canary                      │
│    • Write Discovery Report (what phiếu assumed vs reality)     │
│    • Detail-level decisions (variable names, CSS, internal      │
│      helpers — Tầng 2, see below)                               │
│    • Challenge Architect when architectural assumption is wrong │
│  Does NOT:                                                      │
│    • Decide scope or architecture unilaterally                  │
│    • Ping Architect directly (goes through Chủ nhà)             │
│    • Ship without passing gates                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 2-Tier authority — who decides what

The core principle: **Architect decides the house layout. Worker decides where to put the power outlet in each room.**

| Tier | Examples | Decider |
|---|---|---|
| **Tầng 1 — Architectural** | File structure, routing, API shape, public function signatures, data flow, naming conventions, pattern (Context vs Zustand), module boundaries, schema changes | **Kiến trúc sư** |
| **Tầng 2 — Detail/Implementation** | Local variable names, CSS class names, error message wording (non user-visible), internal helper functions, import ordering, inline types, loop variables | **Thợ** (self-decide + log to Discovery) |

### When in doubt, ask: "Would another Worker need to know this to maintain the code later?"
- **YES** → Tầng 1 → Architect's call
- **NO** → Tầng 2 → Worker's call

### Examples

| Decision | Tier | Rationale |
|---|---|---|
| Use `zustand` or Context API for session state | Tầng 1 | Affects every consumer, long-term pattern |
| Name the hook `useReadingSession` or `useReading` | Tầng 1 | Public API, phiếu references it |
| Name a local `const items` vs `const entries` inside a helper | Tầng 2 | Invisible outside function |
| Error message "Lỗi tải dữ liệu" vs "Không tải được dữ liệu" | Tầng 1 | USER-VISIBLE → actually Chủ nhà's final cut |
| Internal error message "Prisma query failed at line 42" (dev log) | Tầng 2 | Not user-visible |
| Add migration column `user_id` vs `userId` | Tầng 1 | Schema = long-term, Prisma convention |
| Break a function into 2 smaller helpers in same file | Tầng 2 | Internal refactor, no external impact |

## Chủ nhà's 7 responsibilities

This role is often misunderstood. Chủ nhà is NOT just the CEO router. Chủ nhà is the **source of truth provider** for everything domain-related:

1. **Maintain vision docs** — `PROJECT.md` (what it is), `SOUL.md` (why it exists), `CHARACTER.md` (voice). Architect reads these but doesn't write them.
2. **Integrate research** — user feedback, competitor lessons, market observation → distill into vision doc sections. `/insight` skill helps.
3. **Lock principles** — max 3 "hard lines" that cannot be violated. Meta-principle: "Character book, not rule book."
4. **Route inbound** — classify incoming requests (code / marketing / design / strategy / skip). `/route` skill.
5. **Trade-off decisions** — scope, timeline, quality, pricing. `/decide` skill.
6. **Approve user-visible wording** — email copy, UI strings, error messages users see. Final cut.
7. **Relay Architect ↔ Worker** — Architect in Claude Web cannot ping Worker in Claude Code directly. Chủ nhà is the human courier. See `phieu/RELAY_PROTOCOL.md`.

## Which role am I in right now?

| If the current task is… | You're in… |
|---|---|
| "Should we even do this?" / "What's our vision for X?" | **Chủ nhà** |
| "User emailed this feedback — what lane?" | **Chủ nhà** — `/route` |
| "Which pricing tier, 15 or 20 credits?" | **Chủ nhà** — `/decide` |
| "Lots of raw research, need to distill into SOUL section 12" | **Chủ nhà** — `/insight` |
| "Given this approved brief, write the phiếu" | **Kiến trúc sư** — `/plan` |
| "Build what the phiếu says (Task 0 first)" | **Thợ** — `/verify` then code |
| "Code works, write Discovery Report" | **Thợ** |
| "Phiếu's assumption is wrong — architecturally" | **Thợ → Chủ nhà** — escalate, don't self-fix |
| "Phiếu said `items`, code has `entries`, both work" | **Thợ** self-decides (Tầng 2), logs Discovery |

If you can't tell, pick one, finish it, come back for the next role's work.

## Anti-patterns

### 1. Architect fabricates code assumptions
Symptom: phiếu says "function `foo` in `lib/x.ts`" — `foo` doesn't exist.
Why: Architect has no code access, guessed from docs that were stale.
Fix: Every assumption in phiếu cites `thợ kiểm tra tại [file]:[function]`. Worker runs `/verify` Task 0 FIRST.

### 2. Worker silently re-architects
Symptom: phiếu says "add column X", Worker also renames table Y "while I'm here."
Fix: Scope expansion is Architect's call. Tầng 1 changes escalate to Chủ nhà → Architect. Worker does NOT silently expand.

### 3. Worker pings Architect directly
Symptom: Worker sees Architect's assumption is wrong, tries to chat with Claude Web session.
Reality: Worker cannot. Claude Code and Claude Web are separate sessions. Chủ nhà is the human courier.
Fix: Worker writes escalation to Chủ nhà → Chủ nhà paste into Claude Web → Architect responds → Chủ nhà pastes back.

### 4. Chủ nhà skips vision docs
Symptom: Chủ nhà starts routing inbound without having written PROJECT.md / SOUL.md first.
Reality: Without vision docs, Architect has no context to write coherent phiếu.
Fix: On day 1 of a project, Chủ nhà writes PROJECT.md (who/what/why) before anything else. Use `phieu/VISION_TEMPLATES/` as starting skeleton.

### 5. Architect quietly rewrites vision
Symptom: Architect changes PROJECT.md to match their assumption.
Reality: Vision docs are Chủ nhà's. Architect reads, never edits.
Fix: If Architect notices a vision gap, escalates to Chủ nhà with recommendation. Chủ nhà decides to edit or not.

### 6. Skills that span layers
Symptom: One skill that does "route + plan + implement."
Fix: One skill = one layer + one responsibility. Split.

## Skills map (8 total)

| Skill | Layer | Purpose |
|---|---|---|
| `/insight` | Chủ nhà | Distill raw research / feedback into structured vision-doc-ready bullets |
| `/route` | Chủ nhà | Classify inbound (code / marketing / design / strategy / skip) |
| `/decide` | Chủ nhà | Trade-off triage, present options + recommend |
| `/plan` | Kiến trúc sư | Write phiếu in `phieu/TICKET_TEMPLATE.md` format with Task 0 |
| `/verify` | Thợ (architect-assigned) | Task 0 grep-first anchor check before coding |
| `/review` | Thợ | Code review before merge |
| `/qa` | Thợ | Test execution, bug fix, regression |
| `/ship` | Thợ | Full release pipeline (test → PR → deploy → canary) |
| `/retro` | Thợ | Weekly retrospective, velocity, hotspots |

One skill = one layer (or cross-layer gate as with `/verify`). No skill does work for two layers at once.

## Related docs

- [`HANDOFF.md`](./HANDOFF.md) — the 5 formal handoffs between layers
- [`PHILOSOPHY.md`](./PHILOSOPHY.md) — 6 principles (#6 is role separation)
- [`../phieu/README.md`](../phieu/README.md) — ticket workflow glue
- [`../phieu/VISION_TEMPLATES/`](../phieu/VISION_TEMPLATES/) — templates for PROJECT.md / SOUL.md / CHARACTER.md
- [`../phieu/RELAY_PROTOCOL.md`](../phieu/RELAY_PROTOCOL.md) — Chủ nhà's Worker ↔ Architect relay workflow
