# SOS — Portable Operating Contract

SOS is a role-separated operating system for a human owner working with software agents. This file is the runtime-neutral entrypoint.

## Canonical map

| Concern | Canonical source |
|---|---|
| Core boundary and dependency rules | `core/README.md` |
| Roles and capability envelopes | `core/ROLES.md` |
| Ticket lifecycle and state transitions | `core/WORKFLOW.md` |
| Authority, evidence, scope and safety policy | `core/POLICY.md` |
| Asset ownership and migration state | `core/ASSETS.md` |

The files above define semantics. A host integration may map capabilities and render files, but it must not redefine roles, transitions or authority.

## Conflict resolution

1. Human intent and explicit approval govern product scope.
2. `core/POLICY.md` governs authority and safety.
3. `core/WORKFLOW.md` governs lifecycle state.
4. `core/ROLES.md` governs role responsibilities and information access.
5. Host integration files govern only host-specific serialization and capability mapping.

When a host limitation cannot represent a core rule, the integration must report the missing capability. It must not silently weaken the rule.

## Portable minimum

The following remain meaningful without any host integration:

- role ownership and information envelopes;
- ticket drafting, challenge, approval, execution and discovery;
- authority tiers and escalation rules;
- oracle-first verification and edit-scope discipline;
- portable tickets, recipes, templates and universal Git gates.

## Transition status

The portable contract is canonical for newly extracted semantics starting with P075. Existing operational handbooks remain the behavior oracle until their integration parity gate is complete. Historical records remain evidence and are not rewritten.

## Product shape

SOS is delivered as one product and one user-facing command. Internal modules and external managed tools may remain separate implementation units; users interact through the product entrypoint.
