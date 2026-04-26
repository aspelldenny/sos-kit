# Changelog

All notable changes to sos-kit. Format loosely follows Keep a Changelog. Versions are wave-based, not date-based.

## [v2.1] — 2026-04-26

### Added
- **Tarot voice/character template harvest** (P002). 4 templates harvested from production `tarot` app — battle-tested patterns now generalized:
  - `phieu/VISION_TEMPLATES/CHARACTER_template.md` — enriched (Phenotype table, expanded Voice DNA with reactive/proactive + ambiguous-question + fatigue-aware patterns + prompt-engineer-ready section, UX Tempo Principles, character ↔ product-domain mapping)
  - `phieu/VISION_TEMPLATES/VOICE_template.md` — NEW (separate narrator/voice file when product has non-character voice alongside the character)
  - `phieu/VISION_TEMPLATES/TEST_CASES_template.md` — NEW (P0/P1/P2 test-tier grid for character / voice QA)
  - `phieu/VISION_TEMPLATES/DESIGN_SPEC_template.md` — NEW (visual ↔ voice traceability spec)
- **Architect ↔ Worker pre-code debate loop** (P001). Worker (CHALLENGE mode) verifies phiếu against real code before any code is written; Architect (RESPOND mode) judges Worker's objections; multi-turn until consensus or 3-turn cap. Chủ nhà only enters at brief and approval gate — no longer plays courier between agents.
- New invocation modes for both subagents:
  - Architect: `DRAFT` (write fresh phiếu) | `RESPOND` (respond to debate)
  - Worker: `CHALLENGE` (verify pre-code, no commits) | `EXECUTE` (original ship workflow)
- New `## Debate Log` section in `phieu/TICKET_TEMPLATE.md` — append-only debate trail lives in the phiếu file itself; audit trail = git history.
- New doc `docs/ORCHESTRATION.md` — state machine for the main-session orchestrator role.
- New doc section `docs/HANDOFF.md > Handoff 2.5` — debate loop format spec.
- New `agents/README.md` — declares `agents/` as canonical source-of-truth.
- New `scripts/sync-personal-agents.sh` — regenerates `.claude/agents/` from canonical `agents/` via name swap (Chủ nhà → Sếp). Prevents drift.
- `docs/DISCOVERIES.md` and `CHANGELOG.md` files (this one).

### Changed
- `agents/architect.md` and `agents/worker.md` reorganized: each gains an "Invocation modes" table + per-mode workflow sections.
- `INSTALL.md` Step 1 install command updated: copy from `agents/` (canonical) instead of `.claude/agents/` (which is the maintainer's local override).
- `INSTALL.md` Workflow section rewritten for v2.1 auto-debate.
- `README.md` "Two ways to run the 3-role envelope" — Subagent mode description expanded to mention debate loop.
- `.claude/agents/architect.md` and `.claude/agents/worker.md` are now sed-generated from `agents/*.md`. **Do not edit `.claude/agents/` directly** — changes will be overwritten by `sync-personal-agents.sh`.

### Deprecated
- `phieu/RELAY_PROTOCOL.md` — still valid for v1 Web Project mode. For v2.1 Subagent mode, the orchestrator automates the relay role; see `docs/ORCHESTRATION.md`.

## [v2.0] — Earlier

- Initial Subagent mode: `agents/architect.md`, `agents/worker.md`, `architect-guard.sh`, `session-start-banner.sh`, BACKLOG forcing function, `/idea` skill.
- See git history before this changelog was started.

## [v1] — Earliest

- Phiếu workflow, Rust CLIs (ship/docs-gate/guard/vps), original 9 skills, vision templates, Web Project mode.
