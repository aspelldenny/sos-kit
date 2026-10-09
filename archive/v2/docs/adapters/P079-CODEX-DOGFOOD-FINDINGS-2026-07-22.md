# P079 Codex live-dogfood findings (fix spec for P078d) — 2026-07-22
Codex ran a real phiếu end-to-end on the installed adapter. Delivery f18ec99, throwaway clone.

## SOUND (validated — the approach WORKS)
- Install: 17 artifacts + exit 3.
- AGENTS.md → main thread acted as orchestrator (delegated, didn't self-implement).
- Full workflow DRAFT→CHALLENGE→APPROVAL(exact V2)→EXECUTE→DISCOVERY→REVIEW→MERGE→DELIVERED.
- Worker CHALLENGE found a REAL contract omission (publish actors must have publish_changes) → architect ACCEPT V2.
- State file records version/approval/actors/commits. docs-gate 33/33.
- MAIN-THREAD guards ENFORCED: orchestrator-guard blocked src probe; block-env-edit blocked .env. No residual.
- 5 delivery files, --no-ff merge, branch deleted, clean.

## PARTIAL (known/accepted)
- tool envelope weaker than declared (architect/worker both workspace-write — known tool-scoping gap).
- git branch/index writes FAILED under workspace-write sandbox → needed danger-full-access override (publish-capability mismatch for approval=never; interactive on-request may escalate).
- advisory-watch read-only/network-egress untested (docs-only diff skipped it).

## BROKE/MISSING (7 real bugs — all cite crates/sos-adapter-codex/src/templates.rs)

### STARTUP-BLOCKERS (untouched install can't start):
1. templates.rs:258 — config.toml emits root settings (sandbox_mode/approval_policy) AFTER [agents] → become [agents] members. Err: `.codex/config.toml:8: invalid type string "workspace-write", expected struct AgentRoleToml`. FIX: emit sandbox_mode+approval_policy BEFORE [agents].
2. templates.rs:687 — rules emit string patterns; Codex 0.145.0 wants token LISTS. Err: `pattern doesn't match, expected list, actual string`. FIX: `pattern = ["git","push","--force"]`.
3. templates.rs:291 — hooks.json emits unsupported `_provenance`+`_partial_note`. Err: `unknown field _provenance, expected description or hooks`. FIX: combine into top-level `description`.

### ENFORCEMENT/SECURITY:
4. templates.rs:302 — SubagentStart/Stop hooks NEVER created .sos-state/architect-active or worker-active → architect-guard + approval enforcement FAILED OPEN inside spawned agents (architect's forbidden Rust apply_patch succeeded; worker's pre-approval STATE.md patch succeeded). Main-thread guards worked; SUBAGENT marker lifecycle broken.
5. templates.rs:620 — approval-gate blocks ALL non-ticket patch when state-file missing, INCLUDING creating .sos-state/ticket-state.env itself → chicken-egg deadlock. No safe init.
6. templates.rs:379/481/538/606 — ALL guards extract only FIRST patch path (`head -n1`). Bypass: allowed-ticket-first + .sos-state/ticket-state.env (or .env/src) second → guard exits on first-path exemption, allows both. SECURITY HOLE — must parse+check EVERY apply_patch path.
7. First custom-agent spawn failed: "Full-history forked agents inherit parent agent type; omit agent_type or spawn without full-history fork." AGENTS guidance missing this caveat.
