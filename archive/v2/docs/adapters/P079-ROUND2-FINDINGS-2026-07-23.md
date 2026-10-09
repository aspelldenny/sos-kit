# P079 round-2 findings (P078e spec) — 2026-07-23, clone @72ff1c9

## ALL P078d fixes CONFIRMED live:
- Startup PASS (3 schema bugs gone; 11 hooks trusted).
- Multi-path guard PASS 3/3 (blocks .env / .sos-state/ticket-state.env / src/x.rs after allowed ticket-first).
- Bootstrap-create PASS. Honest-MISSING declaration matches. Git boundary blocks .env + no-code-on-default WHEN armed.

## 2 NEW gaps → P078e:
### Gap #1 — Approval transition DEADLOCK (blocks unattended workflow):
Bootstrap creates state (version=V1, approved_version=empty). First legit approval update (→ version=V2, approved_version=V2) BLOCKED by approval-gate because V1 not-yet-approved → circular: approving requires writing approved_version into ticket-state.env, but that write is blocked because not-yet-approved. Manual state edit unblocked it; EXECUTE+REVIEW then succeeded.
FIX: approval-gate must EXEMPT .sos-state/ticket-state.env writes ENTIRELY (control-plane state owner/orchestrator maintains — not a product edit). d2a exempted create-when-missing only; extend to update (approval write). Safe: multi-path check (d2a) still blocks a patch that bundles ticket-state.env + code.

### Gap #2 — Fresh `sos install --runtime codex` does NOT arm Git hooks:
After install, core.hooksPath unset → Git boundary OFF. Had to run scripts/install-hooks.sh manually. Then .env-commit + no-code-on-default blocked correctly.
PROBLEM: d2b honest-MISSING story = "rely on Git/CI backstop" but install doesn't arm it → the declared backstop is off by default. FIX: `sos install` must arm hooks (core.hooksPath=hooks, like sos new/adopt). Apply to claude+codex (install engine or adapter plan).

Note (not a bug): no-code-on-default required removing .sos-state/sos-kit-self because a sos-kit clone self-exempts; real user project wouldn't have that marker. Git boundary works once armed.
