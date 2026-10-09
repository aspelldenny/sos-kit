# Workflow Contract

This file owns lifecycle states and transition rules. Authority and safety rules are defined once in `core/POLICY.md`; role responsibilities are defined in `core/ROLES.md`.

## States

| State | Owner | Exit condition |
|---|---|---|
| `INTAKE` | `owner` + `orchestrator` | Brief maps to an approved backlog item or explicit scope |
| `DRAFT` | `architect` | Versioned ticket contains anchors, scope, constraints and acceptance |
| `CHALLENGE` | `worker` | Assumptions verified; objections accepted, defended, reframed or escalated |
| `APPROVAL` | `owner` or bounded delegate | Ticket version explicitly approved, amended or abandoned |
| `EXECUTE` | `worker` | Allowed edits complete and acceptance evidence collected |
| `DISCOVERY` | `worker` | Assumption mismatches, adaptations and limitations recorded |
| `REVIEW` | `orchestrator` + reviewers | Required quality and security gates pass |
| `DELIVERED` | `orchestrator` | One delivery unit committed and published or merged |
| `BLOCKED` | current role | Blocking decision or capability becomes available |

`BLOCKED` is a side state. It records the previous state, reason, owner of the unblock and safe resume state.

## Primary transition

```text
INTAKE
  → DRAFT
  → CHALLENGE      when the ticket is foundation-level
  → APPROVAL
  → EXECUTE
  → DISCOVERY
  → REVIEW
  → DELIVERED
```

A local, reversible ticket may transition from `DRAFT` directly to `APPROVAL`. A ticket already classified as foundation-level cannot be silently demoted. A local ticket that discovers foundation impact returns to `CHALLENGE` before further execution.

## Draft and challenge loop

1. The architect creates ticket version 1 from approved context.
2. The worker verifies anchors against repository reality without implementing.
3. Every objection includes concrete evidence and impact.
4. The architect responds with exactly one verdict:
   - `ACCEPT`: change the contract.
   - `DEFEND`: retain the contract with evidence.
   - `REFRAME`: record a bounded implementation choice for execute time.
   - `ESCALATE`: request an owner decision.
5. A changed contract increments its version and returns to challenge.
6. Three unresolved turns force owner escalation.

The ticket debate log is the authoritative record. Notification order is not lifecycle state.

## Approval

Approval binds one exact ticket version. Any material scope or architecture change invalidates approval and returns to `DRAFT` or `CHALLENGE`.

A bounded delegate may approve only under the delegation rule in `core/POLICY.md`. Silence, prior approval of a roadmap, or successful tests do not implicitly approve a changed contract.

## Execute

The worker executes in this order:

1. Record a rollback point for mutable local state.
2. Re-verify ticket anchors.
3. Confirm the edit allowlist and verify-only scope.
4. Apply the smallest change satisfying the contract.
5. Run automated and manual acceptance proportional to risk.
6. Record assumption mismatches and edge cases.
7. Produce a reviewable diff and delivery evidence.

If a required capability is unavailable, transition to `BLOCKED` with the missing capability and owning decision. Do not replace execution evidence with a written claim.

## Review and delivery

Review runs all gates triggered by the diff. Security-sensitive surfaces require their configured invariant review before merge or publication.

One ticket is one delivery unit:

```text
gates pass → commit → push or merge → verify remote state → close branch → next ticket
```

Do not execute a second ticket on top of an unpublished first ticket. This preserves fault isolation, makes remote policy failures local to one unit and keeps rollback understandable.

## Completion evidence

A delivery is complete only when all are true:

- the approved ticket acceptance is checked with evidence;
- discovery and change history are updated;
- required gates pass;
- the commit exists in the intended remote branch;
- the working tree contains no unaccounted changes from the delivery;
- the orchestrator records `DELIVERED` before starting another ticket.
