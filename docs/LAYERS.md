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

> Note: Quản đốc = Layer 0 = main-session orchestrator persona. Still 3-role model — Quản đốc orchestrates, doesn't replace Chủ nhà / Kiến trúc sư / Thợ.

| | Chủ nhà | Quản đốc (Layer 0, main session) | Kiến trúc sư | Thợ |
|---|---|---|---|---|
| Vision/strategy docs (PROJECT, SOUL, CHARACTER*) | ✏️ maintain | 📖 read (briefing context only) | 📖 read | 📖 read |
| Code (src/, tests/) | 📖 read optional | ❌ **NO access** (spawn-only) | ❌ **NO access** | ✏️ read+edit |
| Tickets (phiếu) | 📖 read, approve | 📖 read, route between subagents | ✏️ write | 📖 read, execute |
| Discovery Reports | 📖 read | 📖 read (next-phiếu briefing) | 📖 read before next phiếu | ✏️ write |
| Running commands (bash, pnpm, git) | ❌ delegates | ⚠️ marker file ops only (mkdir/touch/rm `.sos-state/`) | ❌ cannot | ✏️ runs |
| Skills (`/frontend-design`, `/security-review`, etc.) | ❌ delegates | ✏️ invoke (Orchestrator-only per P005) | ❌ NO access | ❌ NO access |

**Skills note:** `Skill` tool is **Quản đốc-only** (the main Claude Code session, Layer 0 orchestrator per `docs/ORCHESTRATION.md`). Subagents (Architect / Worker) cannot invoke skills — outputs come pre-frozen in phiếu Context per `phieu/TICKET_TEMPLATE.md` `### Skills consulted` (P005, option B).

**Adapter boundary note (P076):** role/skill semantics above now have a canonical core source at `core/ROLES.md#<role_id>` (owner/orchestrator/architect/worker/advisory_watch/boundary_check). Claude-specific capability serialization (frontmatter `tools:`/`model:`/`caller:`, hook wiring, symlink registration) is mapped artifact-by-artifact in `adapters/claude/MAPPING.md`. This is a declarative boundary only — the access-matrix VALUES above are unchanged; physical extraction/render is P077.

**Mechanical enforcement of the matrix (not just convention):** two PreToolUse guards make the ❌-cells real, not honor-system. `scripts/architect-guard.sh` enforces "Kiến trúc sư ❌ Code" (blocks the Architect subagent from READING `src/`/source while `.sos-state/architect-active` is set). `scripts/orchestrator-guard.sh` enforces "Quản đốc ❌ Code (spawn-only)" (blocks the main session from WRITING product source — `*.swift`/`*.pbxproj`/`src/**` — unless `.sos-state/worker-active` is set, i.e. a Worker is mid-EXECUTE). Scope is product-source only, so Quản đốc's kit-maintenance edits (`docs/`, `bin/`, `scripts/`, `*.md`) stay free. Doctrine: `docs/ORCHESTRATION.md` Hard rules 6 + 12.

### Specialist subagents (P041+)

Specialist subagents sit **beside** the 3 main roles — not replacing them. They are read-only-output verifiers for narrow security audits, spawned by Quản đốc on demand. Specialist subagents (Trinh sát, Giám sát) are read-only-output verifiers that sit beside the 3 main roles; they don't replace them. Spawned by Quản đốc for narrow security audits.

| | Trinh sát (advisory-watch) | Giám sát (boundary-check) |
|---|---|---|
| Role | Specialist subagent — soi advisory ngoài (external CVE/GHSA) | Specialist subagent — soi INVARIANT trong (PR diff against 5 generic boundary rules) |
| Spawned by | Quản đốc via `/advisory-scan` | Quản đốc via `/security-review` |
| Tools | Read, Grep, Glob, WebFetch, WebSearch, **Bash (scoped: parser scripts only)** | Read, Grep, Glob, **Bash (scoped: `git diff/show/log` + `grep` only)** |
| Cannot | Edit, Write, Task, Skill, arbitrary Bash | Edit, Write, WebFetch, WebSearch, Task, Skill, `gh pr comment`, arbitrary Bash |
| Output | Sentinel-wrapped advisory rows → caller appends to inbox | Sentinel-wrapped verdict block (5 INV + APPROVE/NEEDS_REVIEW) → caller posts as PR comment (or local fallback file) |

**Critical**: Kiến trúc sư lives in Claude Web Project. No Bash, no Grep on source, no filesystem access beyond project's attached docs. This is why Task 0 grep-first + Discovery Report exist — they are the Architect's only connection to code reality.

## The 3 layers in detail

> Note: Quản đốc = Layer 0 = main-session orchestrator persona. Still 3-role model — Quản đốc orchestrates between layers, doesn't replace any of the three human roles. **Specialist subagents** (Trinh sát / advisory-watch P041, Giám sát / boundary-check P042) are read-only-output verifiers that sit beside the 3 main roles — spawned by Quản đốc for narrow security audits (`/advisory-scan` + `/security-review`), not replacing any layer.

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 0 — QUẢN ĐỐC (Orchestrator / Main session)               │
│  Tools: Read, Write, Glob, Grep, Bash (marker file ops only),   │
│         Task, AskUserQuestion, Skill                            │
│  Owns:                                                          │
│    • State machine (DRAFT → CHALLENGE → RESPOND → APPROVAL      │
│      → EXECUTE)                                                 │
│    • Subagent spawn (architect + worker), marker file hygiene   │
│    • Skill invocation (Orchestrator-only per P005), output      │
│      capture into phiếu Context as frozen artifact              │
│    • Approval gate via AskUserQuestion (one mandatory gate      │
│      before EXECUTE_PHASE)                                      │
│    • Narration of every state transition (no silent state)      │
│  Does NOT:                                                      │
│    • Write production code (Worker's surface)                   │
│    • Read source files (src/, lib/, etc.) for "context"         │
│    • Edit vision docs (Chủ nhà's surface)                       │
│    • Skip APPROVAL_GATE — even for V1-accepted phiếu            │
│  Full spec: `docs/ORCHESTRATION.md`                             │
├─────────────────────────────────────────────────────────────────┤
│  Layer 1 — CHỦ NHÀ (Owner / Router / Vision keeper)             │
│  Skills: /init  /idea  /insight  /route  /decide                │
│  Tools: Claude Code OR Claude Web (usually wherever the human is)│
│  Owns:                                                          │
│    • Vision docs (PROJECT.md, SOUL.md, CHARACTER*.md)           │
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
│  Skills: /plan  /forge  /verify *                                 │
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
│  Skills: /verify *  /apply  /review  /qa  /ship  /retro           │
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

### What makes it Tầng 1 — by CONSEQUENCE, not size (SINGLE SOURCE)

The deciding question is the **blast / reversibility of being wrong**, never the diff size:

- **Tầng 1 (móng)** — a mistake here **LAN** (affects many consumers / shared contract / schema / API / data flow) **OR is NOT REVERSIBLE** (data, money, auth, privacy, migration, an email/charge already sent). Any **security / auth / schema / privacy / payment / `INV-LOCAL-*`** touch → **AUTO Tầng 1, even if the diff is 1 line.**
- **Tầng 2** — a mistake is **LOCAL and FIXABLE in place** (one button, copy, CSS, a local helper): no external blast, reversible.
- **LOC / file-count is NOT a Tầng signal.** A 3-line privacy check is Tầng 1; a 300-line internal refactor with no external impact is Tầng 2. Size only affects how long a Tầng-1 review is — never whether it's Tầng 1.
- **Direction is asymmetric:** escalate to Tầng 1 on consequence; **never downgrade to Tầng 2 because "it looks small".** (Proving something is truly local is harder than spotting that it spreads.)

> This subsection is the **single source** for the Tầng definition. `agents/orchestrator.md`, `agents/architect.md`, `phieu/TICKET_TEMPLATE.md`, `docs/ORCHESTRATION.md` reference it — they MUST NOT restate the criterion (restating it re-creates two-voice drift, the bug that collapsed the media pilot).

## Chủ nhà's 7 responsibilities

This role is often misunderstood. Chủ nhà is NOT just the CEO router. Chủ nhà is the **source of truth provider** for everything domain-related:

1. **Maintain vision docs** — `PROJECT.md` (what it is), `SOUL.md` (why it exists), `CHARACTER*.md` (voice — glob covers `CHARACTER.md` and named variants like `CHARACTER_CHI_HA.md`). Architect reads these but doesn't write them.
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

## Skills map (5 living + attic)

| Skill | Layer | Caller (mechanical) | Purpose |
|---|---|---|---|
| `/idea` | Chủ nhà | UserPromptSubmit hook `scripts/idea-smell.sh` | Intake new ideas → BACKLOG (dedup + click section + date) |
| `/retro` | Thợ | weekly cron via `advisory-cron register` (per-repo opt-in) | Velocity + hotspot + learnings review |
| `/init` | Chủ nhà | `sos init` CLI — **0→1 only** (⚠ name-collides with Claude Code built-in `/init`) | Vision capture — PROJECT/SOUL/CHARACTER |
| `/apply` | Thợ | `sos apply` CLI — **0→1 only** | Apply 1 recipe — sub-phiếu P000.N → Task 0 → execute → commit |
| `/forge` | Kiến trúc sư | `sos recipe new` CLI | Research + write new recipe to `recipes/` |

**Caller law (Sếp-ratified 2026-06-11):** skill vào kit PHẢI khai `caller:` cơ học trong frontmatter — hook / cron / CLI / gate / handbook-contract. Không caller → `skills/attic/`. Bằng chứng: tarot register đủ 13 skill nhiều tháng, 0 invocation — thứ duy nhất sống là thứ được hook/cron/gate gọi hộ (report: `docs/retro/SKILLS_DOGFOOD_2026-06-11.md`).

**Attic (parked 2026-06-11):** `plan` `verify` (ruột sống inline trong `agents/architect.md`/`worker.md` — định danh mạnh nhất là handbook), `decide` `route` `insight` `qa` `review` `ship` — lý do + điều kiện hồi sinh: `skills/attic/README.md`.

One skill = one layer (or cross-layer gate as with `/verify *`). No skill does work for two layers at once.

\* `/verify` is a cross-layer gate: Architect specifies what must be verified (Task 0 anchors); Worker runs the verification. Listed in both columns above for that reason.

## Related docs

- [`HANDOFF.md`](./HANDOFF.md) — the 5 formal handoffs between layers
- [`PHILOSOPHY.md`](./PHILOSOPHY.md) — 6 principles (#6 is role separation)
- [`../phieu/README.md`](../phieu/README.md) — ticket workflow glue
- [`../phieu/VISION_TEMPLATES/`](../phieu/VISION_TEMPLATES/) — templates for PROJECT.md / SOUL.md / CHARACTER.md
- [`../phieu/RELAY_PROTOCOL.md`](../phieu/RELAY_PROTOCOL.md) — Chủ nhà's Worker ↔ Architect relay workflow
