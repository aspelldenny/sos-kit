# Changelog

All notable changes to sos-kit. Format loosely follows Keep a Changelog. Versions are wave-based, not date-based.

## [v2.1.8] — 2026-05-05

### Changed
- **P039: Doc drift + symmetry sweep (10 surgical edits, Tầng 2).** Refreshed `CLAUDE.md` repo structure tree (13 skills, all real folders including `agents/`, `bin/`, `bootstrap/`, `recipes/`, `scripts/`, `templates/`). Removed 5 hardcoded personal paths (`/Users/nguyenhuuanh/tarot/...` ×4, `/c/Users/Admin/...` ×1). Added `bin/`, `recipes/`, `bootstrap/` to `README.md` architecture tree + all 13 skills listed. Renamed `## Six Principles` → `## Six Operational Principles` in `docs/PHILOSOPHY.md` + synced `CLAUDE.md` from "5 principles" to "6 operational principles". Refreshed `docs/PHILOSOPHY.md` skills list to include `init`, `idea`, `forge`, `apply`. Added `*` cross-layer marker to `/verify` in `docs/LAYERS.md` (both Architect + Worker columns) with footnote. Clarified `/decide (on Worker side)` cell in `docs/HANDOFF.md` → `Worker frames choices → Chủ nhà invokes /decide`. Ported `Active sprint` fallback from `session-start-banner.sh` into `hooks/pre-commit` (P003 pattern). Replaced 5 `/blueprint` slash-form occurrences in `skills/init/SKILL.md` with `sos blueprint` CLI form. Rephrased `CLAUDE.md` "Not a project scaffolder" to clarify `recipes/` role. No logic change except pre-commit fallback.
- Files changed: `CLAUDE.md`, `README.md`, `docs/PHILOSOPHY.md`, `docs/LAYERS.md`, `docs/HANDOFF.md`, `hooks/pre-commit`, `skills/init/SKILL.md`, `skills/retro/SKILL.md`, `recipes/ai/multi-model-fallback.md`, `recipes/payment/payos-vn.md`.
- **Note on collision:** Originally drafted as P038 in session 2026-05-05 before fetching upstream; renumbered to P039 after discovering P038 was already taken by upstream `feat(P038): phieu-lifecycle-cleanup-and-safety` (PR #6, merged 2026-05-02). Lesson logged: orchestrator must `git fetch origin main` before promoting phiếu IDs into BACKLOG.

## [v2.1.7] — 2026-05-02

### Added
- **P038: Phiếu lifecycle cleanup + safety rails + DISCOVERIES decoupling.** Trigger: 2-week Tarot dogfood pushed Max plan to 80% week usage; root cause analysis (`docs/discoveries/P038.md`) identified 6 sub-scopes — token bloat from monolithic DISCOVERIES.md (110k bytes / 28k tokens auto-loaded per Architect spawn), missing phiếu-done cleanup (Debate Log retained, local branches accumulate, no backup cleanup), missing Worker safety rails (force-push / memory edit / settings overwrite all possible), no pre-phiếu rollback point, no doc size warning, no cleanup nudge for approved+merged phiếu.
- **`phieu/phieu.sh`** — `_phieu_done_impl` extended: strips Debate Log Turn N subsections (awk preserve-Final-consensus), moves phiếu file `active/` → `done/` (location-detect: `phieu/active/` for sos-kit, `docs/ticket/` for downstream), `git branch -d` safe-delete (refuses unmerged), removes `.backup/<phiếu-id>/` snapshot. Backwards-compat: phiếu without Debate Log = no-op strip.
- **`scripts/session-start-banner.sh`** — doc size warn (40k byte threshold for CHANGELOG/DISCOVERIES) + phiếu cleanup nudge (scan `phieu/active/` for "Approved by Chủ nhà: <date>" + `git branch --merged main` match → echo `🧹 Phiếu P<NNN> approved + merged. Run: phieu-done P<NNN>`). No `gh` CLI dependency.
- **`agents/worker.md`** — new "Destructive op safety rails" subsection in Hard envelope rules (no force-push, no reset-hard outside phiếu, no edit memory/settings outside scope, no `.sos-state/` deletion, no `rm -rf` on absolute paths) + new top-level "Anti-patterns" section (memory edits, force-push for rebase, pkill -f, mass rm). Discovery Report path updated to `docs/discoveries/P<NNN>.md` per-phiếu.
- **`phieu/TICKET_TEMPLATE.md`** — new "Pre-phiếu snapshot" subsection in Task 0 (Worker auto first-step: `mkdir .backup/<P>` + cp settings.local.json + cp .sos-state + git rev-parse HEAD). Discovery Report path updated: `docs/DISCOVERIES.md` → `docs/discoveries/P<NNN>.md` per-phiếu + 1-line index entry. **Line 4 dual-path note** (V3 [O1.2] fix Anchor #9 drift): filename now documents both `phieu/active/` (sos-kit) and `docs/ticket/` (downstream).
- **`docs/DISCOVERIES.md`** — converted to index-only (table linking to per-phiếu files). Old monolithic content archived at `docs/archive/DISCOVERIES_pre-2026-05.md`.
- **`docs/ORCHESTRATION.md`** — new "Phiếu lifecycle (post-ship cleanup, P038)" section between "Failure modes" and "Concrete example session".
- **`agents/orchestrator.md`** — new "Phiếu cleanup nudge (P038)" section after "Marker file hygiene" — condensed to 2 lines (V3 [O1.1] fix CLAUDE.md ≤90 cap), file goes 88 → 90 lines exactly at cap.
- **`.gitignore`** — added `.backup/`.

### Files changed
- New: `docs/discoveries/P038.md`, `docs/archive/DISCOVERIES_pre-2026-05.md`
- Modified: `phieu/phieu.sh`, `scripts/session-start-banner.sh`, `agents/worker.md`, `phieu/TICKET_TEMPLATE.md`, `docs/DISCOVERIES.md`, `docs/ORCHESTRATION.md`, `agents/orchestrator.md`, `docs/BACKLOG.md`, `.gitignore`, `CHANGELOG.md`

### Cost baseline shift
- Pre-P038: $4.82 / Tầng 2 phiếu (P109 baseline 2026-05-02). Driver: ~28k token DISCOVERIES.md auto-load + cache write 230k Opus.
- Post-P038 expected: per-phiếu Discovery selective-load → 5-10k token avg (vs 28k flat). Architect cache write reduced proportionally. Real measurement after 5+ phiếu post-ship.

## [v2.1.6] — 2026-04-27

### Added
- **P037: Pre-approve marker file Bash ops via `templates/claude-settings.local.json` template + INSTALL.md Step 2.5.** Eliminates per-spawn permission prompt for `Bash(touch .sos-state/architect-active)` / `Bash(rm -f .sos-state/architect-active)` / `Bash(mkdir -p .sos-state)` observed on Tarot 2026-04-27. New template ships 3-entry `permissions.allow` list; INSTALL.md gets Step 2.5 (copy-or-merge instruction) and a Common gotchas row.
- Files changed: `templates/claude-settings.local.json` (new), `INSTALL.md`.

## [v2.1.5] — 2026-04-27

### Added
- **P035: Orchestrator handbook (`agents/orchestrator.md`) + bulk-input rule + INSTALL anti-patterns.** Created `agents/orchestrator.md` (~88 lines, ≤90 cap) — condensed system-prompt contract for the main Claude Code session (4th role / orchestrator). Added Hard rule #8 "Bulk input → auto-triage + ONE gate" to `docs/ORCHESTRATION.md`. Updated `scripts/session-start-banner.sh` to reference new handbook. Added 4 anti-pattern bullets to `INSTALL.md` Step 4 CLAUDE.md template. Added "Edit orchestrator behavior" contributor section to sos-kit's `CLAUDE.md` + pointer to `docs/ORCHESTRATION.md` in "Edit docs" list.
- Files changed: `agents/orchestrator.md` (new), `scripts/session-start-banner.sh`, `INSTALL.md`, `CLAUDE.md`, `docs/ORCHESTRATION.md`.

## [v2.1.4] — 2026-04-27

### Added
- **P036: Tier routing in state machine + Architect humility markers + path-drift fixes (V2).** Architect now sets `Tầng: 1|2` in every phiếu header during DRAFT. Orchestrator routes Tầng 2 phiếu via DRAFT → APPROVAL → EXECUTE (skip CHALLENGE). Tầng 1 retains full debate flow. Worker can escalate 2→1 mid-EXECUTE with `file:line` evidence. Architect humility markers (`[verified]` / `[unverified]` / `[needs Worker verify]`) are now mandatory on all code-level anchors — bare anchors are rejected. V2 scope expansions: fixed stale `docs/ticket/TICKET_TEMPLATE.md` path in `agents/architect.md` (now `phieu/TICKET_TEMPLATE.md`) and generalised hardcoded `docs/ticket/P<NNN>-<slug>.md` in `agents/worker.md` to support both sos-kit (`phieu/active/`) and downstream (`docs/ticket/`) layouts.
- Files changed: `phieu/TICKET_TEMPLATE.md`, `docs/ORCHESTRATION.md`, `phieu/DISCOVERY_PROTOCOL.md`, `agents/architect.md`, `agents/worker.md`.

## [v2.1.3] — 2026-04-26

### Fixed

- **Vision doc naming flex (P004).** `agents/architect.md` and `agents/worker.md` now reference `docs/CHARACTER*.md` (glob) instead of literal `docs/CHARACTER.md` — projects with named characters (e.g. Tarot's `docs/CHARACTER_CHI_HA.md`) work without symlink workaround. Architect globs and reads every match; Worker MUST NOT Read any match (Glob/Grep for detection only). Companion edits in `docs/SETUP.md` (canonical-name recommendation), `docs/HANDOFF.md` (Handoff 0: 3 sites — vision-doc list, workflow ASCII block, session-open reading order), `docs/LAYERS.md` (access matrix + Layer 1 inner box + Chủ nhà responsibility #1), `docs/GENESIS.md` (Phase 0 Vision row). Sibling fix to P003; same principle: sos-kit consumes Sếp-owned docs, doesn't dictate names.

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
