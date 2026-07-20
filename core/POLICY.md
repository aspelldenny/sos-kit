# Core Policy

This file is the single canonical location for authority, evidence, scope and safety policy.

## Human accountability

Automation may recommend and route; the owner remains accountable for product intent, acceptable risk and irreversible decisions.

The owner must decide when a change:

- changes vision, scope or user-visible behavior beyond the approved brief;
- selects between materially different architecture contracts;
- changes security, privacy, payment or data-loss posture;
- requires destructive or hard-to-recover action;
- accepts a known failed gate or missing required capability.

Mechanical implementation details remain with the assigned role when they preserve the approved contract.

## Authority tiers

### Foundation-level

A change is foundation-level when a wrong choice propagates across modules, users or future work. Examples include architecture, public interfaces, schemas, authorization, privacy, security invariants and irreversible migration.

Required path: draft, challenge, explicit approval, execute, discovery and review.

### Local and reversible

A change is local when its consequence is contained, reversible and does not touch a foundation boundary. It may skip challenge, but still requires an explicit approved ticket and acceptance evidence.

Tier is determined by consequence, not line count. Escalation from local to foundation-level is allowed; silent demotion is not.

## Bounded delegation

The owner may delegate approval for a named ticket or sprint. The delegation must state its boundary and be recorded in the ticket.

A delegate may self-approve only when:

- challenge is complete with evidence;
- no objection requires owner judgment;
- the change stays within the delegated scope;
- no security, privacy, irreversible or risk-acceptance decision is introduced.

Otherwise the workflow returns to the owner.

## Information envelopes

Each role receives the minimum context and capabilities needed for its responsibility.

- The architect receives doctrine and approved product context, not unrestricted implementation context.
- The worker receives the approved contract and implementation evidence, not authority to reinterpret vision.
- Read-only specialists receive only the diff, metadata and invariants required for their check.
- The orchestrator sees lifecycle state and reports but does not use that access to implement the change.

Envelopes are enforced mechanically where the host permits it. Prompt-only boundaries must be reported as weaker guarantees.

## Oracle-first claims

Every important claim should name an oracle: a test, parser, validator, diff, external response or human acceptance step that can prove or falsify it.

- `SOUND`: the oracle closes the stated claim under the tested conditions.
- `PARTIAL`: the oracle covers only named dimensions; residual risk is explicit.
- `MISSING`: no oracle exists; approval must treat the claim as unverified.

Existence is not capability. A file, command or declaration being present does not prove the intended behavior runs successfully.

## Edit and verify scope

Ticket permissions are asymmetric:

- `edit_allow` is narrow and explicit.
- `verify_read` may be broader because safe execution needs repository evidence.
- Reading for verification does not grant permission to edit.
- A required edit outside the allowlist stops execution until the contract is amended.

Unrelated user changes are preserved. A delivery must not absorb, revert or publish them.

## Safe mutation

Before mutation, resolve exact targets and record a rollback point proportional to risk.

- Prefer additive and non-clobber behavior.
- Never silently overwrite user-customized generated files.
- Destructive actions require explicit scope and recoverability.
- Missing required gates fail visibly.
- Optional integrations may warn and skip only when the resulting state remains safe and is reported.

## Evidence and discovery

Discovery records only what changed understanding:

- assumptions proven wrong;
- implementation adaptations;
- fired mechanisms;
- residual limitations;
- documentation brought back into alignment.

Historical evidence is immutable context. Do not rewrite old reports merely to match current vocabulary.

## Mechanism discipline

Use the cheapest reliable mechanism that catches the actual observed failure class. Judgment remains guidance; deterministic checks may become gates. A new blocking mechanism must cite the concrete failure it prevents and provide a recovery path.
