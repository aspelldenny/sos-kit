# Portable Core Boundary

`core/` is the semantic source of truth shared by every host integration and execution engine.

## Dependency rule

```text
user-facing command ─┬─> portable core
                     ├─> selected host integration ─> portable core
                     └─> managed tools

portable core ─X─> host integration
portable core ─X─> host manifest or permission schema
```

The command is the composition root. It selects integrations, resolves capabilities and invokes core policy. Integrations may depend on the core contract. The core must never import or name an integration.

## What belongs here

- stable role IDs and responsibilities;
- workflow states, transitions and gates;
- authority, evidence, scope and safety rules;
- portable asset ownership;
- vocabulary consumed by renderers and policy engines;
- canonical serialization formats for shared tickets, state, approval, edit-scope, review triggers and blocked records (`STATE.md`).

## What does not belong here

- host event, tool or environment names;
- host file layout and permission syntax;
- provider-specific prompts or invocation commands;
- platform download and install implementation;
- historical debate and migration narrative.

Exact current path mapping lives in `docs/RUNTIME_BOUNDARY_INVENTORY.md`. Target packaging and module layout live in `docs/PORTABILITY_ARCHITECTURE.md`.

## Integration obligations

An integration must:

1. Declare which core capabilities it can provide.
2. Refuse or degrade visibly when a required capability is absent.
3. Keep generated artifacts traceable to a core source and version.
4. Preserve role boundaries and workflow transitions.
5. Pass a behavior-parity suite before replacing an existing integration.

## Transitional rule

P075 creates the neutral semantic contract without changing existing execution behavior. Later migration work may render operational files from this core only after parity is demonstrated. Until then, existing operational files remain the executable oracle and must not be silently rewritten.
