# Portable Serialization Contract

This file owns the *canonical machine-readable format and path/schema* of the artifacts that every integration must render the same way. It does not redefine semantics: lifecycle state and transitions belong to `core/WORKFLOW.md`, authority and safety belong to `core/POLICY.md`, and role responsibilities belong to `core/ROLES.md`. This file serializes those contracts.

## Neutrality and compatibility rules

1. Every format below is **host-neutral**: it names fields, paths and shapes, not host tools or file layouts. An integration renders each artifact through its own mechanism and must not require a specific host to remain meaningful.
2. These formats **describe existing behavior**; an integration must render them without changing established lifecycle behavior. A host limitation that cannot represent a format must be reported per `core/README.md` integration obligations, not silently weakened.
3. Only genuinely shared artifacts live here. Host-specific rendering detail (concrete marker filenames, event names, prompt wording) stays in the integration.

## Ticket storage and schema

- **Path:** an active ticket lives at `docs/ticket/P<NNN>-<slug>.md`; a completed ticket moves to `docs/ticket/done/P<NNN>-<slug>.md`. `<NNN>` is a zero-padded 3-digit monotonic ID; `<slug>` is kebab-case. The canonical directory value is `docs/ticket`; an integration MAY expose it through its own configuration, but the default is this value.
- **Active selection:** at most one ticket is the active delivery unit at a time (the one-delivery-unit rule in `core/WORKFLOW.md`). Core specifies the *semantic* — one active unit — not the mechanism. An integration renders the pointer through its own mechanism (working-tree or branch identity, a state artifact, or explicit selection).
- **Schema (required sections, canonical order):**
  1. header block — `type`, `priority`, `tier`, `lane`, `affected-files`, `dependency`;
  2. context — problem, approach, scope (files touched / files not touched); an optional note for any upstream analysis consulted before drafting;
  3. verification anchors (Task 0) — a table of assumption, verification method and result;
  4. debate log — one entry per challenge/response turn, capped, ending in a recorded consensus;
  5. task list — the ordered units of work with their target files and acceptance conditions;
  6. edit-and-verify file lists — files the ticket may edit versus files it may only read for verification;
  7. constraints — the rules of engagement for this ticket;
  8. acceptance — automated checks, manual checks, regression checks and a discovery-report requirement.

  The `tier` axis (consequence) and the `lane` axis (budget) are independent; neither substitutes for the other.

## Lifecycle state artifact

- Canonical machine-readable fields for the state of one delivery unit:
  - `ticket` — the unit's identifier and path;
  - `version` — the contract version currently in play;
  - `state` — one of the states named in `core/WORKFLOW.md`'s state table (intake, draft, challenge, approval, execute, discovery, review, delivered, blocked);
  - `approved_version` — the version an approval record is bound to, or empty if none;
  - `previous_state` — the state to resume into, used when the unit is in the blocked side-state;
  - `blocked_reason` — the blocking condition, or empty if not blocked.
- **Authoritative-vs-projection rule (backward-compatible with existing behavior):** the ticket's debate log remains the authoritative human record of the draft, challenge and approval history. A machine-readable state artifact is a **derived projection** that an integration MAY materialize to enforce gates deterministically. Where a projection and the debate log disagree, the debate log governs and the projection must be corrected. An integration is not required to materialize a separate file if it derives state directly from the ticket.
- The `state` value vocabulary references the state table in `core/WORKFLOW.md` and must not redefine it here.

## Approval record

- A canonical approval record binds **exactly one ticket version** and records: the approving actor (the owner, or a named bounded delegate), the bound version, a timestamp, and the scope of delegation if the approver is a delegate.
- **Mutation authority:** only the owner, or a bounded delegate acting under the delegation rule in `core/POLICY.md`, may create an approval record. Silence, prior approval of a broader plan, or passing tests do not constitute approval. Any material scope or architecture change invalidates an existing record and returns the ticket to draft or challenge.
- An integration renders the approval interaction and the record's storage location through its own mechanism — typically an interactive confirmation step plus a recorded marker in the ticket or its state artifact.

## Edit allowlist and verify scope

- A ticket declares two path sets: an edit set, narrow and explicit, and a verify set, which may be broader because safe execution needs repository evidence. This asymmetry follows the edit-and-verify rule in `core/POLICY.md`.
- **Path matching:** allowlist entries are repo-relative path patterns, matched either as a literal path or a glob (`*` within a path segment, `**` across segments). **Normalization** means the repo-relative logical path with any leading `./` stripped and no trailing slash.
- **Symlink handling:** matching is performed on the repo-relative logical path as written in the ticket. Whether a symlink is followed or resolved before matching is integration-defined, and any integration whose symlink handling can bypass the allowlist MUST report that as a residual limitation.
- **Amendment:** an edit outside the declared edit set stops execution until the ticket contract is amended. An amendment adds the path to the edit set and increments the ticket version; there is no verbal or implicit widening of scope.

## Review trigger map

- A canonical map from diff surface to required review gate. The neutral surface classes are: authentication, session, permission or privacy paths; implementation source directories; schema or migration files; secret or environment files (excluding example or template files); request-handling middleware; external webhook handlers; and any file that implements or enforces a project-local invariant.
- **Rule:** a diff that touches a surface in this map must pass its configured invariant or security review gate before merge or publication. The map is matched mechanically; a diff is not exempted by a judgment that its scope is small or that it is "docs-only".
- The concrete path globs and the review command that enforces this rule are integration or project configuration. Core specifies which surface classes trigger review, not a host's specific glob or command.

## Blocked state format

- A canonical blocked record formalizes the four fields already named for the blocked side-state in `core/WORKFLOW.md`:
  - `previous_state` — the state to resume into;
  - `reason` — the blocking decision or missing capability;
  - `unblock_owner` — the role or actor who can resolve the block (typically the owner);
  - `resume_state` — the safe state to continue from, usually equal to `previous_state` unless re-verification is required first.
- **Storage and resume:** a blocked record lives with the ticket, either in the ticket itself or in its state projection. On unblock, the blocking condition is verified resolved, then the unit transitions to `resume_state`. A blocked delivery unit must not be silently dropped; it persists until resolved or explicitly abandoned by the owner.
- The persistence mechanism (a ticket section versus a separate state artifact) is integration-defined and is not named here.

## Deferred scope

The following adapter-shared concerns are **not yet codified** here. An integration must not assume a core-canonical format for them until a follow-up ticket lifts them:

- **tier-classifier ownership** — which role classifies a ticket's tier; the underlying authority-tier semantics already live in `core/POLICY.md`, but the canonical ownership/format has not been formalized;
- **concurrent-ownership lock or worktree isolation format** — larger scope because worktree semantics differ across hosts;
- **publish-actor division** — which actor commits, pushes or merges; partially covered by the delivery rule in `core/WORKFLOW.md`, but no canonical format yet;
- **backlog serialization format** — not yet required by any adapter render.

These remain integration-defined until a follow-up ticket lifts them; an integration must not assume a core-canonical format for them yet.
