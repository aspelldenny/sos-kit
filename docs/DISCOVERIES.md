# Discoveries Log

> Newest on top, like CHANGELOG. Each entry: phiếu ID + date + what assumptions held vs broke + scope expansions + edge cases + docs touched.

## [P001] — 2026-04-26 — Architect ↔ Worker debate loop

### Assumptions in phiếu — CORRECT
- All 10 Task 0 anchors verified during smoke test (see updated phiếu Task 0 table).
- `scripts/architect-guard.sh` regression-free — still exits 2 on `src/*.rs/.ts/.py` reads when `.architect-active` marker present.
- Sync script `sync-personal-agents.sh` idempotent — running twice is safe.
- 8 tasks of phiếu were sufficient — no scope expansion needed.

### Assumptions in phiếu — WRONG / Adapted
- **Anchor #2/#4 (Tầng 1, ACCEPTED):** phiếu assumed `.claude/agents/*.md` differed from `agents/*.md` only by `Sếp` ↔ `Chủ nhà`. Reality (pre-implement): 7 additional lines differed (e.g., `(Sếp's standing instruction)` vs `(standing instruction)`, two Vietnamese-specific question examples). **Resolution:** Task 6's harmonization fixed this — `.claude/agents/` is now purely a name-swapped sed copy of `agents/`. Trade-off: lost a few Vietnamese-specific example phrases in `.claude/agents/`. Documented in `agents/README.md`.
- **Tầng 2:** phiếu spec'd `docs/ORCHESTRATION.md` ≤ 250 lines; shipped at 150 (tighter than budgeted, no info loss).

### Scope expansions
- None. Stayed within 8 tasks of phiếu. Considered + rejected: writing actual orchestration helper script — left as documentation only because the orchestrator IS Claude main session, no script needed.

### Edge cases / limitations found
- Sync script lossy: Vietnamese-specific phrases in old `.claude/agents/*` ("Sếp gửi vision doc nội dung gì?") replaced with English ("What content goes in the vision doc?") for maintainability. Documented in `agents/README.md`.
- Marker file `.claude/.architect-active` lifecycle is **entirely orchestrator's responsibility** — no automated cleanup if main session crashes mid-debate. `docs/ORCHESTRATION.md` Hard Rule 6 says "rm before every spawn" but this is convention, not enforcement. Future risk: if marker leaks, all subsequent Read/Glob calls in any subagent would be blocked. Mitigation: `bash scripts/architect-guard.sh` is itself a no-op when marker absent, and orchestrator instruction includes defensive cleanup.
- Trigger-phrase parsing in subagents (e.g., "challenge phiếu" → CHALLENGE mode) is convention-based, not enforced — a misbehaving orchestrator could spawn Worker with ambiguous prompt and Worker would default to EXECUTE (backward-compat). For now, accepted; orchestrator instruction document carries the discipline.

### Docs updated to match reality
- `README.md` v2 description updated to mention debate loop
- `INSTALL.md` workflow steps + verify smoke test extended
- `docs/HANDOFF.md` — Handoff 2.5 added, summary table appended
- `phieu/TICKET_TEMPLATE.md` — Debate Log section schema inserted between Task 0 and Nhiệm vụ
- `agents/architect.md` + `agents/worker.md` — Invocation modes section + DRAFT/RESPOND or CHALLENGE/EXECUTE workflows
- `agents/README.md` — TẠO MỚI (source-of-truth doc explaining canonical vs. local-override)
- `scripts/sync-personal-agents.sh` — TẠO MỚI
- `docs/ORCHESTRATION.md` — TẠO MỚI (state machine + example session)
- `docs/DISCOVERIES.md` — TẠO MỚI (this file)
- `CHANGELOG.md` — TẠO MỚI

### Notes for future phiếu
- The next phiếu (P002) will be the first to run through the actual debate loop end-to-end (P001 was implemented single-pass since debate flow didn't exist yet — chicken-and-egg).
- If P002 surfaces edge cases the orchestrator state machine doesn't cover, update `docs/ORCHESTRATION.md` rather than letting workarounds drift.
