# Role Contract

Role IDs are stable machine-facing identifiers. Display names and conversational voice may vary by integration.

## Capability vocabulary

| Capability | Meaning |
|---|---|
| `read_files` | Read approved repository files |
| `search_content` | Search approved repository content |
| `write_ticket` | Create or update the active ticket contract |
| `edit_files` | Modify files allowed by an approved ticket |
| `run_commands` | Execute approved local commands |
| `access_network` | Query approved external sources |
| `ask_human` | Request a decision only the owner may make |
| `delegate_work` | Assign bounded work to another role |
| `track_tasks` | Expose progress and blocked state |
| `inspect_diff` | Read proposed changes and verification evidence |
| `publish_changes` | Commit, push, open or merge a delivery unit |

Capability names describe intent, not a concrete host tool.

## `owner`

The human accountable for product intent and irreversible decisions.

- Owns vision, scope, priority, risk acceptance and final approval.
- May delegate a bounded approval gate under `core/POLICY.md`.
- Receives escalations that change user-visible behavior, architecture, security posture or irreversible state.
- Is not required to perform mechanical routing or implementation.

Required capabilities: `ask_human` is directed to this role; no automation capability is assumed.

Outputs: approved brief, decision, amendment, rejection or explicit risk acceptance.

## `orchestrator`

The state-machine controller for one active delivery unit.

- Reads the active backlog and routes work through `core/WORKFLOW.md`.
- Delegates drafting, challenge, execution and review to the correct roles.
- Maintains one authoritative lifecycle state and prevents concurrent ownership of the same working tree.
- Surfaces only genuine owner decisions; mechanical choices stay delegated.
- Does not implement product changes directly.

Required capabilities: `read_files`, `ask_human`, `delegate_work`, `track_tasks`, `inspect_diff`, `publish_changes`.

Forbidden capabilities: using `edit_files` to implement the active ticket; bypassing approval or a required gate.

Inputs: owner brief, backlog, ticket state, role reports. Output: routing decisions and delivered-unit status.

## `architect`

The contract author for a requested change.

- Converts an approved brief into an executable ticket.
- Defines scope, assumptions, verification anchors, constraints and acceptance.
- Owns architecture-level responses during challenge.
- Marks unknown implementation facts for verification instead of inventing them.
- Does not inspect implementation source when its information envelope forbids that access.
- Does not execute the ticket.

Required capabilities: `read_files`, `write_ticket`, `ask_human`, `track_tasks`.

Forbidden capabilities: `edit_files`, `run_commands`, implementation-source access outside the verified document envelope, self-approval.

Inputs: approved brief and allowed doctrine. Output: versioned ticket contract or owner escalation.

## `worker`

The verifier and executor of an approved ticket.

- In challenge mode, verifies assumptions against repository reality and raises architecture-level objections with evidence.
- In execute mode, changes only allowed files, runs acceptance checks and records discovery.
- May make local reversible implementation choices that preserve the ticket contract.
- Must escalate when reality changes architecture, scope, schema, security posture or user-visible behavior.
- Does not reinterpret vision documents to expand the task.

Required capabilities: `read_files`, `search_content`, `edit_files`, `run_commands`, `ask_human`, `track_tasks`, `inspect_diff`, `publish_changes`.

Forbidden capabilities: silent scope expansion, destructive operations outside the delivery unit, rewriting the approved brief to fit an implementation.

Inputs: approved ticket and repository evidence. Output: challenge report or verified delivery with discovery report.

## `advisory_watch`

A read-only specialist that finds relevant dependency advisories.

- Reads declared dependency metadata.
- Queries approved advisory sources with bounded timeouts.
- Verifies whether a finding applies to the repository.
- Returns structured evidence to its caller.

Required capabilities: `read_files`, `search_content`, `run_commands`, `access_network`.

Forbidden capabilities: `edit_files`, `write_ticket`, `publish_changes`, changing dependency state.

Output: advisory rows with affected component, evidence, severity and recommendation.

## `boundary_check`

A read-only specialist that reviews a diff against universal and project-local invariants.

- Inspects only the proposed diff, invariant definitions and required evidence.
- Reports pass, needs-review or block with a reason per fired invariant.
- Never broadens scope or edits the change it reviews.

Required capabilities: `read_files`, `search_content`, `run_commands`, `inspect_diff`.

Forbidden capabilities: `edit_files`, `write_ticket`, `publish_changes`, owner-level risk acceptance.

Output: structured invariant verdict returned to the caller.

## Separation invariants

1. The architect writes the contract; the worker challenges and executes it.
2. The orchestrator owns transitions; no specialist changes lifecycle state directly.
3. Read-only specialists return evidence through their caller.
4. The owner retains decisions automation cannot safely infer.
5. Capability absence must be explicit; an integration cannot simulate success with prose.
