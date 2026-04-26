# Orchestration — main session as the 4th role (v2.1+)

> SOS Kit v2.1 introduces a 4th, non-human role: **Orchestrator** = the main Claude Code session that spawns subagents. It is not a separate file — it is whatever Claude is running in the user's main chat window. This doc specifies how that session should mediate the Architect ↔ Worker debate loop.

## Why a 4th role

In v2.0 (single-pass), the user manually said "spawn architect" then "spawn worker." If Worker hit a Tầng 1 mismatch, the user had to relay it back to Architect by hand — same courier role as v1 Web Project mode, just within one session.

v2.1 automates the relay. The main session detects "Worker wrote Debate Log → Architect needs to respond" and spawns the right subagent in the right mode without user input. The user's role contracts to **brief in + nghiệm thu out**, with a single approval gate before EXECUTE.

## Session opening (first user message)

Before the state machine starts, the orchestrator MUST perform a session opening. SessionStart hook stdout is injected into the model's context only — it does not render to the user's terminal UI. Without an explicit greeting, the user has no signal that the session is alive and aware of Active sprint.

**Required behavior on the first user message in a fresh session:**

1. Read SessionStart context (Active sprint block from `docs/BACKLOG.md`, already injected by the hook).
2. Reply briefly (max 5 lines), greeting as the visible "Architect" persona:
   ```
   Em là Kiến trúc sư project <name>.
   Sprint hiện có {N} item: <short list>.
   Anh muốn pick item nào, có idea mới, hay đã có công việc cụ thể?
   ```
3. Wait. Do not spawn Architect/Worker. Do not run Bash, Read, or grep on this turn.
4. Branch on the user's reply:
   - "Pick item X" → DRAFT_PHASE (spawn Architect DRAFT)
   - "New idea Y" → IDEA_INTAKE (`/idea` skill or append to BACKLOG)
   - Concrete brief ("build feature X for item Y") → DRAFT_PHASE directly

**Edge cases:**
- If the first user message is already a concrete brief → skip the greeting, go straight to DRAFT_PHASE.
- If BACKLOG has no recognizable section (no `## ` headings at all → SessionStart banner stayed silent) → greet without list: "Em là Kiến trúc sư. BACKLOG chưa có item nào — anh có việc gì cần viết phiếu không?" (After P003: a project whose top section is e.g. `## Now` instead of `## Active sprint` resolves via fallback and DOES get a sprint block — this edge case fires only for truly empty/malformed BACKLOGs.)

**Why "Kiến trúc sư" persona for the orchestrator:**
- Solo workflow has 1 human (Chủ nhà) + 1 visible AI counterpart + 1 invisible Worker subagent. Surfacing the orchestrator as a 4th distinct role bloats the mental model.
- Internally the main session is still the orchestrator. It still delegates ticket writing to the `architect` subagent (sandboxed, no code access) when DRAFT_PHASE fires. The persona is UX framing, not a role merger.
- The 8-câu checklist, debate loop, and envelope guard all still run in the subagent — the persona does not let main session bypass them.

## State machine

```
IDLE
 │ user gives brief ("build feature X for BACKLOG item Y")
 ▼
DRAFT_PHASE                                spawn Architect (DRAFT)
 │ Architect writes phiếu V1 with Debate Log section initialized
 ▼
CHALLENGE_PHASE                            spawn Worker (CHALLENGE)
 │ Worker verifies Task 0 + reads code + writes Debate Log Turn N
 ├── Worker accepted (no objection) ─────────────► APPROVAL_GATE
 ├── Worker raised objections ─────────────────► RESPOND_PHASE
 ▼                                                    │
RESPOND_PHASE                              spawn Architect (RESPOND)
 │ Architect responds per objection, bumps phiếu version
 ├── all objections resolved (no DEFER) ─────► CHALLENGE_PHASE (Turn N+1)
 ├── any DEFER TO CHỦ NHÀ ────────────────────► FORCE_ESCALATION
 ├── Turn 3 reached, still objections ────────► FORCE_ESCALATION
 ▼
APPROVAL_GATE                              orchestrator runs AskUserQuestion
 │ User reviews final phiếu + Debate Log → approve / abandon / amend brief
 ├── approve ─────────────────────────────────► EXECUTE_PHASE
 ├── amend brief ─────────────────────────────► DRAFT_PHASE
 ├── abandon ─────────────────────────────────► IDLE
 ▼
EXECUTE_PHASE                              spawn Worker (EXECUTE)
 │ Worker codes, tests, Discovery Report, commits
 ▼
DONE                                       hand back to user for nghiệm thu

FORCE_ESCALATION                           orchestrator runs AskUserQuestion
 │ Surface deadlock or vision question to user
 ├── user resolves → respond on Architect's behalf ─► CHALLENGE_PHASE (Turn N+1)
 ├── user changes scope ─────────────────────────────► DRAFT_PHASE
 ├── user proceeds anyway ───────────────────────────► EXECUTE_PHASE
 └── abandon ─────────────────────────────────────────► IDLE
```

## Trigger phrases (orchestrator → subagent spawn prompt)

The subagent files (`agents/architect.md`, `agents/worker.md`) parse the spawn prompt for these phrases to choose mode:

| Target mode | Phrase to include in spawn prompt |
|---|---|
| Architect DRAFT | "Spawn architect viết phiếu cho X" / "plan X" / "write phiếu for X" |
| Architect RESPOND | "Architect respond to Debate Log Turn <N> in P<NNN>" |
| Worker CHALLENGE | "Worker challenge phiếu P<NNN>" / "review phiếu pre-code" |
| Worker EXECUTE | "Worker execute phiếu P<NNN>" / "implement P<NNN>" |

Default if no phrase matches: Architect → DRAFT, Worker → EXECUTE (backward compat with v2.0).

## Hard rules

1. **Max 3 turns.** After Turn 3, regardless of state, force-escalate to user. Loops are a sign of either bad phiếu or genuinely under-specified vision — both need human judgment.
2. **No silent state.** Every phase transition is visible in the chat (orchestrator narrates: "Worker raised 2 objections → spawning Architect RESPOND").
3. **Debate trail lives in the phiếu file.** No external log, no database. Audit trail = git history of the phiếu.
4. **Approval gate is mandatory.** Even if Worker accepted V1 with no challenges, orchestrator MUST run AskUserQuestion before EXECUTE_PHASE. Only the human approves code execution.
5. **User can interrupt anytime.** State machine is suggestive, not enforced — if the user types into the main session mid-debate, orchestrator handles their input first.
6. **Marker file hygiene.** Architect-guard hook uses `.sos-state/architect-active` marker. Orchestrator must `mkdir -p .sos-state && touch .sos-state/architect-active` before spawning Architect (any mode), `rm -f .sos-state/architect-active` before spawning Worker. Never leave stale markers. (Marker lives outside `.claude/` so YOLO mode doesn't prompt — `.claude/` is gated even with `--dangerously-skip-permissions`.)

## Failure modes + recovery

| Failure | Recovery |
|---|---|
| Architect RESPOND didn't bump phiếu version | Orchestrator re-spawns once with explicit "bump version to V<N+1>". Second failure → escalate. |
| Worker CHALLENGE wrote objection without `file:line` citation | Orchestrator rejects, asks Worker to redo with citations. Architect cannot judge an evidence-free objection. |
| Stale `.architect-active` marker | Orchestrator runs `rm -f .sos-state/architect-active` before every spawn. Defensive; cheap. |
| Phiếu version went backwards (V3 → V2) | Refuse — orchestrator escalates as a bug in Architect output. |
| Same objection raised in 2 consecutive Worker turns | Indicates Architect didn't actually fix the underlying issue. Force-escalate. |

## Concrete example session

```
USER: build a phiếu cho item "Add user export" ở Active sprint

ORCHESTRATOR: spawning architect (DRAFT)...

  [ARCHITECT DRAFT]
  Read CLAUDE.md, BACKLOG.md, PROJECT.md, DISCOVERIES.md.
  Glob docs/ticket/*.md → next ID is P042.
  Wrote docs/ticket/P042-user-export.md (V1) with 3 anchors, 4 tasks.
  Returns: 1 ⚠️ anchor (export format not in docs).

ORCHESTRATOR: phiếu V1 written → spawning worker (CHALLENGE)...

  [WORKER CHALLENGE]
  Read phiếu, ran Task 0:
    Anchor #1 (function fooExport at src/lib/export.ts) → ❌ doesn't exist
    Anchor #2 (route src/app/api/export/route.ts) → ✅
    Anchor #3 (constant EXPORT_FORMATS) → ⚠️ inline strings instead
  Read src/app/api/users/route.ts (the only existing /users handler).
  Objections:
    - O1.1: phiếu Task 2 says "import fooExport from src/lib/export.ts"
            but src/lib/export.ts has barExport not fooExport (cite src/lib/export.ts:14)
    - O1.2: phiếu Task 3 spec'd EXPORT_FORMATS constant; codebase uses
            inline strings (cite src/lib/export.ts:22, 31)
  Alternatives: A. rename to barExport (Worker lean), B. add fooExport alias.
  Wrote Debate Log Turn 1 → AWAITING ARCHITECT RESPONSE.

ORCHESTRATOR: 2 objections → spawning architect (RESPOND)...

  [ARCHITECT RESPOND]
  Read Debate Log Turn 1.
  Verdict:
    O1.1 → ACCEPT (use barExport, update Task 2)
    O1.2 → REFRAME (Tầng 2, Worker self-decides at EXECUTE — log to Discovery)
  Phiếu bumped to V2.

ORCHESTRATOR: V2 ready, no DEFER → spawning worker (CHALLENGE) again...

  [WORKER CHALLENGE V2]
  Re-ran Task 0 with V2 anchors → all ✅.
  No new objections.
  Wrote Debate Log: "Worker accepted V2 — ready for approval."

ORCHESTRATOR: consensus reached after 1 turn. Running approval gate...

  [AskUserQuestion]
  Phiếu P042-user-export.md V2 (Debate Log: 1 turn, 1 ACCEPT, 1 REFRAME).
  Options:
    A. Approve — proceed to EXECUTE  (recommended)
    B. Amend brief and redraft
    C. Abandon

USER: A

ORCHESTRATOR: approved → spawning worker (EXECUTE)...

  [WORKER EXECUTE]
  Task 0 ✅ (re-checked).
  Coded 4 tasks, ran tests (pass), wrote Discovery Report (1 Tầng 2 adapt for O1.2),
  committed `feat(P042): user-export`.

ORCHESTRATOR: shipped. Hand back to Chủ nhà for nghiệm thu.
```

## Replaces

This doc supersedes RELAY_PROTOCOL.md for v2.1 Subagent mode. RELAY_PROTOCOL.md remains valid for v1 Web Project mode users.
