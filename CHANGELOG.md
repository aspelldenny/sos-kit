# Changelog

All notable changes to sos-kit. Format loosely follows Keep a Changelog. Versions are wave-based, not date-based.

## [v2.1.2] — 2026-04-26

### Fixed
- **BACKLOG format flexibility (P003).** `scripts/session-start-banner.sh` now falls back to the first `## ` section when no `## ... Active sprint` header is present (previously: silent exit, no banner). `agents/architect.md` Hard rule 0 wording softened to match — the active section is resolved by case-insensitive substring "Active sprint" first, then by first `## ` section. `docs/ORCHESTRATION.md` edge-case greeting (line 32) rewritten to no longer falsely claim "BACKLOG chưa có Active sprint" after fallback resolves a header. Sếp no longer needs to rename their BACKLOG sections to satisfy a literal regex. Tarot's restructured-BACKLOG workaround (2026-04-26 dogfood) is no longer required for new installs.

## [v2.1.1] — 2026-04-26

### Added
- **Session opening protocol** (`docs/ORCHESTRATION.md`, new section between "Why a 4th role" and "State machine"). On the first user message in a fresh session, the orchestrator (main session) MUST greet, self-identifying as "Kiến trúc sư" + listing Active sprint items from SessionStart hook context. Without this, the SessionStart hook output (which only injects into the model's context — never visible in the terminal UI) leaves the user without confirmation that the session is alive and context-aware. Edge cases covered: skip greeting if first message is already a concrete brief; alternate greeting if BACKLOG has no Active sprint.
- Tarot's project-local mirror at `~/tarot/docs/ORCHESTRATOR.md` updated with the same Session opening section (Tarot commit `36e626f`).

### Verified (Tarot dogfood, 2026-04-26)
- **Debate flow value proven.** P029 smoke test caught a real anchor mismatch — Architect spec'd `export default` for the Next.js middleware file; Worker CHALLENGE grep'd and found `export async function middleware(...)` (named export). Catch happened pre-code, not post-ship. Without CHALLENGE, the comment header would have shipped describing a non-existent export pattern.
- **Multi-turn debate works end-to-end.** P030 (`.docs-gate.toml` accept chore type): V1 → 2 anchor objections → Architect RESPOND ACCEPT both → V2 → Worker re-CHALLENGE 0 obj → Sếp Approve → Worker EXECUTE → ship. 2 turns total, well under 3-turn cap. Architect RESPOND mode + Worker re-CHALLENGE both verified.
- **Approval gate is value-add, not friction.** Sếp approved every phiếu with one click via AskUserQuestion; never amended brief mid-debate.
- **Token cost realistic.** ~42k/multi-turn phiếu (prompt cache hits across subagent spawns within Anthropic's 5-min TTL). Pre-test estimate of 140k was 3× too high. Future v2.2 optimization tickets should baseline 42k, not 140k. Details: `docs/DISCOVERIES.md` v2.1-dogfood entry.

### Known issues (out of sos-kit scope)
- `docs-gate` CLI default `valid_types` missing `chore` — surfaced when Tarot's P029 commit (type `chore`) was about to fail docs-gate. Fix belongs in `~/docs-gate` Rust binary's default config, not in sos-kit (sos-kit doesn't ship `.docs-gate.toml` templates).

## [v2.1] — 2026-04-26

### Added
- **Periodic audit protocol — RRI-T-lite** (`phieu/AUDIT_PROTOCOL.md`). 354-line protocol harvested from RRI-T Methodology v1.0 (Vietnamese Enterprise Software), scope-reduced for solo / B2C use:
  - 4-result model `PASS / FAIL / PAINFUL / MISSING` (replaces binary pass/fail — captures UX rot + spec gaps)
  - 3 personas (User + QA + Security on-demand) instead of full 5
  - 4 dimensions (UI/UX + API + Data + Edge Cases) instead of full 7; Performance/Security/Infrastructure on-demand
  - 4 stress axes (Time + Data + Error + Locale) instead of full 8
  - 5 phases compact (PREPARE → DISCOVER → STRUCTURE → EXECUTE → ANALYZE) — 4-8h instead of RRI-T full 1-2 days
  - Vietnamese-specific 13 checks (diacritics, VND, GMT+7, font rendering, etc.) — bắt buộc cho B2C VN
  - Triggers: every 5-10 phiếu / wave end / pre-major-release / post-incident / monthly smoke
  - Worker AUDIT mode (read-only) integration into v2.1 debate flow — Worker writes `docs/AUDIT_<wave>.md`, no code changes
  - Coverage release gate ≥85% green / 70-84% yellow / <70% red
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
