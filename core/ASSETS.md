# Asset Ownership

This file classifies content ownership. It is not an install manifest or a list of host-specific target paths. Exact current path evidence remains in `docs/RUNTIME_BOUNDARY_INVENTORY.md`.

## Classes

| Class | Meaning | Source of truth |
|---|---|---|
| `PORTABLE` | Semantics or assets usable by every integration | Portable repository path |
| `TRANSITIONAL_MIXED` | Portable content still combined with host wiring | Existing operational path until migration |
| `ADAPTER_OWNED` | Host serialization, registration or capability mapping | Integration template or renderer |
| `GENERATED` | Installed state derived from a versioned source | Managed manifest plus source hash |

## Portable assets

| Source | Ownership |
|---|---|
| `SOS.md`, `core/**` | Role, workflow, policy and ownership contract |
| `phieu/**` | Ticket, audit, relay, discovery and vision templates |
| `recipes/**` | Reusable implementation patterns and recipe schema |
| `configs/**` | Stack configuration examples |
| `.docs-gate.toml`, `templates/.docs-gate.toml` | Documentation gate configuration |
| `templates/BACKLOG_template.md`, `templates/INVARIANTS-template.md`, `templates/advisory-inbox.md` | Portable project templates |
| `hooks/pre-commit`, `hooks/pre-push` | Universal Git boundary entrypoints |
| `scripts/block-env-commit.sh`, `scripts/no-code-on-default.sh`, `scripts/security-gate.sh`, `scripts/trust-gate.sh` | Universal Git and security policies |
| `scripts/check-*`, `scripts/parsers/**`, `scripts/install-hooks.sh` | Portable validators, parsers and hook installation |

## Transitional mixed assets

| Source class | Portable part | Integration part | Migration owner |
|---|---|---|---|
| `agents/**` | Role responsibilities and envelopes | Capability declaration and execution metadata | P076-P078 — P076 DECLARED (Claude integration maintains its own artifact-mapping manifest; provenance marker added in body); physical render P077 |
| `skills/**` | Workflow steps and decision rules | Invocation metadata and tool bindings | P076-P078 — P076 DECLARED (Claude integration maintains its own artifact-mapping manifest; provenance marker added in body); physical render P077 |
| Existing workflow/layer/handoff/orchestration guides | Doctrine and evidence | Host-specific operation details | P076 |
| `bin/sos.sh` | State, adoption and synchronization behavior | Host-directed skill delegation and wiring | P077 |
| `crates/**` (repo-root, relocated from `bootstrap/sos-rs/` P077f) | Typed command and state foundation | Transitional delegations and old ownership contract | P077 |
| `install.sh`, `templates/setup-dev.sh` | Platform bootstrap and tool intent | Hard-coded integration/tool installation | P077/P081 |
| Lifecycle guard scripts | Portable policy intent | Host event payload and environment binding | P076/P077 |

## Adapter-owned asset classes

Integrations own only serialized representations:

- host entry instructions;
- agent and skill registration records;
- lifecycle event bindings;
- tool and capability maps;
- permission configuration;
- optional protocol-server registration.

An adapter-owned artifact must identify the portable source or policy it represents. It must not become the only copy of a semantic rule.

## Generated assets

Generated state includes installed registrations, local lifecycle state, trust snapshots, hook stubs and future managed manifests.

**Physical render (P077d1):** the 6-field schema below is now a concrete Rust type — `ManagedManifest` in `crates/sos-core/src/manifest.rs` (relocated from `bootstrap/sos-rs/crates/sos-core/src/manifest.rs` P077f; serde, TOML round-trip tested). Schema only; apply/rollback logic lands in P077d2.

Every managed artifact records:

- owning integration or portable component;
- source product version;
- source identity;
- target path;
- installed content hash;
- previous-state or rollback reference when mutation occurred.

Generated files are updated only when unchanged since installation. Customized files produce a conflict or incoming copy; uninstall removes only content still owned by its managed hash.

## Migration rule

P075 establishes semantic ownership. Later tickets may move or render an asset only after the current executable behavior has a parity oracle. Historical archives and discovery records are evidence, not migration inputs.
