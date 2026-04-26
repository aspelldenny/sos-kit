# Discoveries Log

> Newest on top, like CHANGELOG. Each entry: phiếu ID + date + what assumptions held vs broke + scope expansions + edge cases + docs touched.

## [P002] — 2026-04-26 — Tarot voice/character template harvest

### Assumptions in phiếu — CORRECT
- Tarot `CHARACTER_CHI_HA.md`, `VOICE_SACH_CO.md`, `TEST_CASES.md`, `NHA_CHI_HA_DESIGN_SPEC.md` all exist and are battle-tested (Sprint Voice Unify v1.0–v1.3, multiple production phiếu reference).
- Patterns are generalizable: hard-rules-max-3, voice DNA, anti-pattern diagnostics, phrase bank, pre-flight checklist, voice boundaries, P0/P1/P2 tier model, visual ↔ voice traceability.

### Assumptions in phiếu — WRONG / Adapted
- **CHARACTER_template.md was less shallow than the gap report claimed.** It already had 12 sections covering Tarot's main spine. The "enrichment" turned out to be: add Phenotype table (Tarot §3 patterned 16 traits), expand Voice DNA from 5 sub-sections to 12 (reactive/proactive, ambiguous-question handling, fatigue-aware, self-disclose cap, prompt-engineer-ready patterns), add UX Tempo Principles, and add "How character relates to product domain." Net: replaced the file rather than incremental edits; full-overwrite with `Write` was cleaner than 5+ Edit calls.
- **TEST_CASES grid format is more useful than a checklist.** Tarot uses per-test setup + script + check + sample-pass + sample-fail format. Adopted that rather than the flat checklist I'd initially pictured.

### Scope expansions
- None beyond planned 4 templates. Considered + rejected: harvesting `NARRATIVE.md` and `PROMPT_HANDOFF.md` from Tarot. Both are too product-specific to generalize without diluting the templates.

### Edge cases / limitations found
- Templates are intentionally long (200–300 lines each) — they're meant to be filled in for one product, not skimmed. Future projects should not feel obligated to fill EVERY section; `(optional)` markers help.
- TEST_CASES_template references CHARACTER + VOICE + SOUL by section number, but a project might number sections differently. Trade-off: too prescriptive vs. too vague. Chose section-number references — assumes user follows the templates' numbering.
- DESIGN_SPEC template assumes Next.js + Tailwind file paths (e.g. `tailwind.config.ts`). Generic-enough that projects using SvelteKit / Flask + Jinja can adapt, but a project lead should sanity-check before adoption.

### Docs updated to match reality
- `phieu/VISION_TEMPLATES/CHARACTER_template.md` — overwritten with enriched version (200 → 250 lines, 14 sections)
- `phieu/VISION_TEMPLATES/VOICE_template.md` — TẠO MỚI
- `phieu/VISION_TEMPLATES/TEST_CASES_template.md` — TẠO MỚI
- `phieu/VISION_TEMPLATES/DESIGN_SPEC_template.md` — TẠO MỚI
- `README.md` — VISION_TEMPLATES section + Architecture section updated to mention new templates and their Tarot lineage

### Notes for future phiếu
- Next time a new project onboards, copy these templates into `docs/`, then run `/insight` skill to populate from raw research. The skill should be aware of all 6 templates now (PROJECT/SOUL/CHARACTER/VOICE/TEST_CASES/DESIGN_SPEC) — currently it references only 3. Mark as P003 candidate: extend `/insight` skill to know about new templates.

---

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
