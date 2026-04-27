# Discoveries Log

> Newest on top, like CHANGELOG. Each entry: phiếu ID + date + what assumptions held vs broke + scope expansions + edge cases + docs touched.

## [P035] — 2026-04-27 — Orchestrator handbook + bulk-input rule + INSTALL anti-patterns (V3)

### Assumptions in phiếu — CORRECT
- Anchor #1 (`[needs Worker verify]`): `agents/orchestrator.md` did NOT exist pre-EXECUTE. Bash `ls agents/orchestrator.md` returned exit:1. File created as planned. P036 dogfood confirmed: anchor stayed `[needs Worker verify]` end-to-end; Worker grep-verified at EXECUTE time and found "file absent."
- Anchor #2: `docs/ORCHESTRATION.md` Hard rules section at line 108, rules 1-7, `## Failure modes + recovery` heading immediately following rule 7. Confirmed. Edit succeeded (rule #8 inserted at line 117, file now 199 lines).
- Anchor #4: `scripts/session-start-banner.sh` lines 70-77 confirmed. Line 77 = `echo "    Spec đầy đủ: docs/ORCHESTRATION.md"`. No prior `agents/orchestrator.md` reference. New line inserted at line 77 (Handbook line), Spec line shifted to 78.
- Anchor #5: `INSTALL.md` fenced template at lines 143-168. "Workflow (v2.1 — auto-debate)" at line 157. Line 167 = "6. Chủ nhà nghiệm thu, deploy". Closing ``` at line 168. Anti-pattern block inserted before closing fence.
- Anchor #10 (`[needs Worker verify]`): `phieu/active/` confirmed to exist (P035 phiếu was there). `phieu/done/` confirmed to exist (P036 lives there). No directory creation needed.
- Anchor #11: `wc -l docs/ORCHESTRATION.md` = 198 pre-edit. Confirmed.
- Anchor #12 (`[needs Worker verify]` in V2, upgraded to `[verified]` in V3): `wc -l agents/orchestrator.md` post-Write = 88 lines. Within ≤90 cap (1-line trim margin remained). Architect's V3 count of 89 was off by 1 — likely trailing-newline difference in counting method. Tầng 2 self-adapt: 88 < 90, cap satisfied, no trim needed.
- Anchors #3, #6, #7, #8, #9, #13: All held (not re-verified at EXECUTE since no changes needed for those paths; V2 Debate Log confirmations stand).

### Assumptions in phiếu — WRONG
- Anchor #12 count: Architect counted 89 lines in V3 phiếu block; on-disk write produced 88 lines. Off-by-1 difference (likely trailing newline accounting). Not a violation — 88 < 90 cap. Tầng 2 adaptation, logged here.

### Scope expansions
- None. All 5 tasks executed as specified. No additional files touched.

### Edge cases / limitations found
- **Docs-gate P006 friction confirmed:** `docs-gate` installed but no `.docs-gate.toml` in sos-kit root. Default config expects `docs/CHANGELOG.md` and `docs/ARCHITECTURE.md` — both non-existent at those paths (sos-kit's CHANGELOG.md is at root, and there is no ARCHITECTURE.md). `docs-gate` exited 1 with "File not found" errors. This is the P006 known friction. Pre-commit hook uses docs-gate — if hook is installed, commits would fail on fresh-install without a `.docs-gate.toml`. Next phiếu should add a minimal `.docs-gate.toml` to sos-kit root (or disable docs-gate for this meta-kit repo).
- **Bulk-input rule dry-run (procedural):** Rule #8 wording in `docs/ORCHESTRATION.md` (steps a-d, "MUST NOT ask pick order before steps a-c") and mirror in `agents/orchestrator.md` are consistent and use same a-d labels. On cold-read, the rule is unambiguous: classify first, propose wave second, single gate third. No ambiguity found.
- **orchestrator.md cold-read test:** file contains session-opening behavior (4-step "Session opening" section), state machine ASCII with peer branches (DEFER + Turn-3 cap as siblings under RESPOND_PHASE), tier routing, trigger phrases, marker hygiene, bulk input, hard rules, anti-patterns. Sufficient for a cold-start session to drive the state machine correctly without reading ORCHESTRATION.md first.
- **State machine ASCII peer-branch test:** DEFER and Turn-3 cap are `├──`/`└──` siblings under `RESPOND_PHASE → objections` subtree, matching ORCHESTRATION.md lines 55-59 structure. V2 O3 fix confirmed on-disk.
- **V2 RESPOND confirmation:** O1.1 (line count), O1.2 (session opening), O1.3 (state machine indentation), frontmatter Option C (tools: [] + model: opus + HTML comment) — all present and verified on-disk. `tools: []` field present (grep confirmed). Subagent loader safety: empty allowlist means no tool capability can be accidentally granted.
- **V3 RESPOND confirmation:** O2.1 (line count) resolved — `wc -l agents/orchestrator.md` = 88 ≤ 90. No Tầng 1 escalation required.
- **Marker dogfood note (P036 acceptance):** Anchor #1 stayed `[needs Worker verify]` throughout debate (Luật chơi #5 in phiếu). Worker verified at EXECUTE time: file absent. Dogfood pattern confirmed working.

### Docs updated to match reality
- `CHANGELOG.md` — appended v2.1.5 entry for P035.
- `docs/DISCOVERIES.md` — this entry (newest on top).
- `docs/BACKLOG.md` — NOT updated in this phiếu (Nghiệm thu Docs Gate item says move P035 from Active sprint to Recently shipped — left for orchestrator post-commit per BACKLOG maintenance convention; not in phiếu Nhiệm vụ scope).

## [P036] — 2026-04-27 — Tier routing + Architect humility markers + path-drift fixes (V2)

### Assumptions in phiếu — CORRECT
- Anchor #1: `phieu/TICKET_TEMPLATE.md:10-13` — exact 4 fields `Loại / Ưu tiên / Ảnh hưởng / Dependency`, no `Tầng` field. Confirmed pre-edit.
- Anchor #2: `docs/ORCHESTRATION.md` state-machine fenced block opens at line 41, inner DRAFT_PHASE text at line 45. CHALLENGE Turn 1 noted off-by-one in V1 prose (45 vs 41) — reconciled in V2. Tìm matched on content, not line number. Edit succeeded.
- Anchor #3: `phieu/DISCOVERY_PROTOCOL.md` Tầng 2 sub-heading at line 28, Tầng 1 at line 37. Tìm matched on fence-close + section boundary. Edit succeeded.
- Anchor #4: `agents/architect.md:134-140` "Source your assumptions" section. End of section line text matched exactly. Edit succeeded.
- Anchor #5: `agents/worker.md` CHALLENGE mode at lines 44-84, EXECUTE at 86-112, Tầng 1/2 table at 114-131. All Tìm strings matched.
- Anchor #6: No collision at `phieu/active/P036-*` — confirmed pre-EXECUTE.
- Anchor #7: `agents/architect.md:131` Hard rule 5 exact text matched.
- Anchor #8: `agents/worker.md:120-130` 8-row Tầng 1/2 table. Confirmed.
- Anchor #9: `docs/ORCHESTRATION.md` Hard rules at line 91 with 6 rules. Confirmed pre-edit.
- Anchor #10: `phieu/active/` directory exists; phiếu lives there as expected.
- Anchor #11: `agents/architect.md:57` stale `docs/ticket/TICKET_TEMPLATE.md` text matched exactly. Fixed to `phieu/TICKET_TEMPLATE.md`.
- Anchor #12: `agents/worker.md:48` hardcoded `docs/ticket/P<NNN>-<slug>.md` in CHALLENGE step 1. Matched exactly.
- Anchor #13: `agents/worker.md:90` hardcoded `docs/ticket/P<NNN>-<slug>.md` in EXECUTE step 1. Matched exactly.
- Anchor #14: `CHANGELOG.md` exists at sos-kit root. Appended P036 entry.

### Assumptions in phiếu — WRONG
- None. All 14 anchors held.

### Scope expansions
- V2 scope expansions (Tasks 4b + 5c) landed cleanly — 2 single-line edits co-located with planned Task 4 and Task 5 edits. No secondary-edit fallout observed. Both verify (Anchors #11-#13) confirmed pre-EXECUTE.

### Edge cases / limitations found
- **Nghiệm thu check drift:** Phiếu Nghiệm thu states "`grep -n "docs/ticket/TICKET_TEMPLATE.md" agents/architect.md` returns 0 hits" — but Task 4b's own replacement text includes `docs/ticket/TICKET_TEMPLATE.md` as a parenthetical downstream-compat note. The residual hit is intentional (disambiguation note, not a stale path). The old standalone reference (the path as the only location) is gone. Tầng 2 adaptation — logged here, not escalated.
- **Nghiệm thu check drift (Task 5c):** Phiếu states `grep -nE "docs/ticket/P<NNN>-<slug>\.md" agents/worker.md` should return 0 hits. The new replacement text itself lists `docs/ticket/P<NNN>-<slug>.md` as the downstream path inside the parenthetical. Residual hits are intentional. Same pattern as Task 4b above.
- **`phieu/done/` directory absent** — created at move time (post-commit). No pre-existing conflict.
- Worker CHALLENGE mode update (Task 5): CHALLENGE mode workflow step 1 did not need separate change — Task 5c covered lines 48 and 90 directly. Step 6 in CHALLENGE mode still says "Tầng 1 objections only" which is consistent with new routing (CHALLENGE only runs for Tầng 1).

### Docs updated to match reality
- `phieu/TICKET_TEMPLATE.md`: `Tầng:` field added (Task 1).
- `docs/ORCHESTRATION.md`: tier-routing branch in state machine + new "Tier routing" section + Hard rule 7 + 2 failure-mode rows (Tasks 2a-2d).
- `phieu/DISCOVERY_PROTOCOL.md`: "Tier as a routing key" sub-section + 2→1 escalation (Task 3).
- `agents/architect.md`: "Humility markers" section + Hard rules 5→5/6/7 renumber + stale path fix at line 57 (Tasks 4, 4b).
- `agents/worker.md`: CHALLENGE row updated + EXECUTE step 4 marker-handling + new step 4a tier-escalation + "Tier escalation 2→1" + "Anchor markers" sections + hardcoded paths generalised (Tasks 5, 5c).
- `CHANGELOG.md`: v2.1.4 entry appended.

## [P004] — 2026-04-26 — Vision doc naming flex (CHARACTER*.md glob)

### Assumptions in phiếu — CORRECT
- Anchor #1: `agents/architect.md:55` contained exact text `- \`docs/CHARACTER.md\` — voice (only if voice-facing work)` — matched V3 Task 1 Tìm block.
- Anchor #2: `agents/worker.md:17` contained exact text (line corrected V1→V2 per [O1.3]). Single occurrence. Content matched Task 2 Tìm block.
- Anchor #3: `agents/architect.md:4` — `tools:` includes `Glob`. No frontmatter change needed.
- Anchor #4: `agents/worker.md:4` — `tools:` includes both `Glob` and `Grep`. Confirmed.
- Anchor #5: `scripts/sync-personal-agents.sh` loops `for f in architect.md worker.md` — both handled. Script ran without error; diffs show ONLY `Chủ nhà` → `Sếp` substitutions.
- Anchor #6: Repo-wide grep at EXECUTE found one NEW out-of-scope hit not in V3 expected set: `bin/sos.sh:94` — shell echo string in user-facing help text ("Generate docs/...CHARACTER.md (if persona)"). This is conceptual description, not an agent envelope rule — same class as `README.md:112` (documented Tầng 2 cosmetic exclusion in phiếu). No escalation required.
- Anchors #7, #8, #9, #10, #11, #12, #13, #14, #15, #16: All ✅ at V3. Architect Read-confirmed at respective version bumps; no drift detected at EXECUTE re-verify.

### Assumptions in phiếu — WRONG
- **[Tầng 2 — adapted in phiếu V2 per Turn 1]** Anchor #2 V1 stated "line 19" — actual line was 17. Corrected in V2 per [O1.3]. Exact-text match still succeeded in V1/CHALLENGE; only the stated line number was wrong.
- **[Scope expansion — Tầng 1 accepted in V2 per Turn 1 [O1.1]]** Anchor #6 V1 expected "5-file scope." V2 expanded to "6-file scope" when Worker Turn 1 grep found `docs/GENESIS.md:16` and `docs/LAYERS.md:37,107` as additional rule-binding sites.
- **[Scope expansion — Tầng 1 accepted in V3 per Turn 2 [O2.1]]** Anchor #8 V2 expected "lines 21, 33" in HANDOFF.md. V3 expanded to "lines 21, 28, 33" when Worker Turn 2 grep found intra-section inconsistency at line 28 inside the same Handoff 0 section.

### Scope expansions
- V1 → V2: +2 files in scope (GENESIS.md Task 7, LAYERS.md lines 37+107 expansion of Task 6). Both accepted per debate consensus.
- V2 → V3: +1 sub-edit within already-scoped HANDOFF.md (line 28, Task 5 expanded from 2 to 3 sub-edits). Zero new files in scope.
- EXECUTE: `bin/sos.sh:94` found — Tầng 2 exclusion applied per established out-of-scope class. No scope expansion.

### Edge cases / limitations found
- `scripts/sync-personal-agents.sh` correctly handled both files — idempotent, no drift beyond `Chủ nhà` → `Sếp` substitution.
- No additional rule-binding `docs/CHARACTER.md` literals beyond the 6-file scope. `bin/sos.sh:94` is descriptive echo text only.
- LAYERS.md ASCII box line 37: 1-char addition (`CHARACTER.md` → `CHARACTER*.md`) required 1 trailing-space removal to preserve box border alignment. Edit succeeded with correct alignment. Tầng 2 cosmetic — self-decided.
- HANDOFF.md line 28 ASCII workflow block: leading 2-space indent + `→ ` arrow preserved after `*` insertion. Block reads cleanly top-to-bottom with lines 21, 28, 33 all in glob form (no mid-section convention shift).
- GENESIS.md table: column count stays at 4 after cell-width expansion in line 16. Markdown tables don't enforce column widths — render confirmed correct.
- SETUP.md Step 4d: 3 comment lines added (all `#`-prefixed) — valid shell comments; step remains copy-paste scriptable.
- macOS filesystem (case-insensitive HFS+/APFS): glob `CHARACTER*.md` will match case-insensitively. Projects with `character_foo.md` (lowercase) would also match — acceptable extension of tolerance, not a regression.
- Debate Log meta: Turn 1 [O1.1] + [O1.2] ACCEPT → V2 (3 micro-edits, 2 new files). Turn 2 [O2.1] ACCEPT → V3 (1 micro-edit, 0 new files). Turn count: 2/3 used. 1h estimate held.

### Docs updated to match reality
- `agents/architect.md` — Task 1: line 55 glob fix
- `agents/worker.md` — Task 2: line 17 glob fix
- `.claude/agents/architect.md` — Task 3: regenerated via sync-personal-agents.sh
- `.claude/agents/worker.md` — Task 3: regenerated via sync-personal-agents.sh
- `docs/SETUP.md` — Task 4: 3-line comment block added above `cp` command
- `docs/HANDOFF.md` — Task 5 (3 sub-edits: lines 21, 28, 33)
- `docs/LAYERS.md` — Task 6 (3 sub-edits: lines 21, 37, 107)
- `docs/GENESIS.md` — Task 7: line 16 Phase 0 Vision row
- `CHANGELOG.md` — v2.1.3 entry added
- `docs/BACKLOG.md` — P004 marked [x] + added to "Recently shipped"

---

## [P003] — 2026-04-26 — BACKLOG format flexibility (banner fallback + Architect Rule 0 + ORCHESTRATION.md)

### Assumptions in phiếu — CORRECT
- All 10 Task 0 anchors verified pre-EXECUTE (anchors #1–#9 from V1, #10 added in V2 after Debate Turn 1).
- `scripts/session-start-banner.sh` line 22 had the exact strict-regex `^## .*Active sprint`. `set -uo pipefail` (no `-e`) confirmed lines 13-14. `awk` boundary finder confirmed line 28 (now line 38 after Task 1 expansion).
- `agents/architect.md` Hard rule 0 at lines 118-121 contained the exact literal "Active sprint" text.
- `scripts/sync-personal-agents.sh` exists, does `sed 's/Chủ nhà/Sếp/g'` for both agents. Task 4 ran clean.
- `docs/BACKLOG.md` line 10 has `## 🔥 Active sprint:` — strict-match path still fires, no fallback note shown (backwards compat verified via manual test).
- No other script in `scripts/` referenced "Active sprint" beyond `session-start-banner.sh` (anchor #8).
- `agents/worker.md` had zero "Active sprint" references (anchor #9).
- `docs/ORCHESTRATION.md` line 32 contained the exact literal string (anchor #10) — patched in Task 6.

### Assumptions in phiếu — WRONG / Adapted
- **Task 2 "Tìm" block was stale (Tầng 2 self-adapted).** The phiếu's Task 2 assumed the banner's closing block was 7 lines ending directly after the Rule 0 line. In reality the v2.1.1 banner has a 9-line "Orchestrator contract" block between the `📊 Active sprint:` count line and the `📌 Architect Rule 0:` line (added by P001/v2.1.1 which landed after this phiếu was drafted). Worker adapted by: (a) inserting the `FALLBACK_USED` conditional directly after the count line (correct placement), and (b) updating only the Rule 0 closing line in-situ. Semantics match phiếu intent exactly; only the insertion point differed. Tầng 2 — no escalation needed.

### Scope expansions
- **V1 → V2 (pre-EXECUTE debate, resolved before EXECUTE):** Worker CHALLENGE Turn 1 raised `docs/ORCHESTRATION.md` line 32 as out-of-scope drift. Architect RESPOND accepted, added Task 6 + anchor #10, bumped to V2. Debate-loop mechanic worked as designed — Docs Gate caught a missing scope item in CHALLENGE mode, not during EXECUTE. This is the first P00N phiếu where the debate loop caught a real scope gap in CHALLENGE mode on sos-kit itself (P029 was the Tarot equivalent).
- No further expansions during EXECUTE phase. Stayed within 6 tasks of V2.

### Edge cases / limitations found
- **Fallback resolves to the FIRST `## ` section, not necessarily the user's intended active section.** If a BACKLOG places e.g. `## Future waves` before `## Now`, the fallback picks `## Future waves`. Mitigation: the fallback note tells Sếp which header was used; Sếp reorders BACKLOG if wrong. Strict-match path sidesteps this entirely.
- **HEADER_TEXT includes emoji if the header has one** (e.g. `## 🔥 Active sprint:` strips to `🔥 Active sprint:` — the `sed 's/^## *//'` is permissive by design). The fallback note then reads `Treating "🔥 Active sprint:" as Active sprint...` — technically redundant on strict-match path (FALLBACK_USED=0) so never fires, but documented for future edge-case awareness.
- **CHANGELOG.md now contains the literal "BACKLOG chưa có Active sprint" string** in the new v2.1.2 Fixed bullet (describing what was changed). This is historical context in a changelog entry, not a live constraint. The Docs Gate check explicitly scopes exclusions to the phiếu file; CHANGELOG was not listed. This is a Tầng 2 judgment: changelog entries quoting removed behavior are expected; adding CHANGELOG.md to the exclusion list would be noise. Future Docs Gate checks for this string should exclude both `docs/ticket/P003-*.md` AND `CHANGELOG.md`.

### Docs updated to match reality
- `scripts/session-start-banner.sh` — Tasks 1 + 2: fallback header resolution + fallback-used note
- `agents/architect.md` — Task 3: Hard rule 0 softened with active-section resolution sub-rule
- `.claude/agents/architect.md` — Task 4: regenerated via `scripts/sync-personal-agents.sh`
- `CHANGELOG.md` — Task 5: v2.1.2 entry
- `docs/ORCHESTRATION.md` — Task 6: line 32 edge-case greeting rewritten
- `docs/BACKLOG.md` — P003 marked `[x]`, moved to "Recently shipped"

### Notes for future phiếu
- P004 (vision doc naming flex, sibling drift fix) should follow the same "format-tolerant, never silent" pattern as P003: emit a note when falling back, never silently pick wrong.
- Future doc-touching phiếu should pre-grep `docs/` for ALL strings whose semantics they change. P003 V1 missed `docs/ORCHESTRATION.md` only because the Architect anchor table didn't enumerate doc-side surfaces exhaustively — Worker's CHALLENGE Docs Gate requirement caught it. Architect should add a standing anchor "grep docs/ for the literal string being removed" for any phiếu that retires a user-visible string.
- The CHANGELOG historical-quote issue (see Edge cases above) suggests the Docs Gate rule for "no file contains string X" should always list exact exclusions: `docs/ticket/P<NNN>-*.md` (phiếu quotes old string in Tìm/Debate) AND `CHANGELOG.md` (may quote it in Fixed bullet).

---

## [v2.1-dogfood] — 2026-04-26 — debate flow proven on Tarot (P029 + P030)

### Assumptions in design — CORRECT
- **Architect ↔ Worker isolation catches real anchor mismatches.** P029 smoke test: Architect spec'd `export default` for `src/middleware.ts`; Worker CHALLENGE grep'd, found actual `export async function middleware(...)` named export. Caught pre-code, not post-ship. This is the core value of subagent isolation — fresh contexts force grep-based verification instead of trusting Architect's claims.
- **Tầng 2 mismatch self-handle works.** P029: Architect didn't know file already had 19-line JSDoc; Worker `head -20` found it, used REPLACE-path instruction in phiếu V1 — no Architect RESPOND needed. Saved one debate turn. Confirms `phieu/DISCOVERY_PROTOCOL.md` Tầng 2 rule is correctly scoped.
- **Multi-turn debate converges fast for non-trivial phiếu.** P030: V1 had 2 anchor objections → Architect ACCEPT both → V2 → Worker re-CHALLENGE → 0 obj → ship. 2 turns total, well under 3-turn cap.
- **Approval gate adds value, not friction.** Sếp confirmed every phiếu with one click via AskUserQuestion; never amended brief mid-debate.

### Assumptions in design — WRONG / Adapted
- **Token cost estimate was ~3× too high.** Pre-test estimate: ~140k/multi-turn phiếu. Realistic measured: ~42k including prompt cache hits across subagent spawns within Anthropic's 5-min cache TTL. Future v2.2 optimization tickets should baseline against 42k, not 140k.
- **SessionStart hook stdout does NOT render to user UI.** v2.1 design assumed `bash scripts/session-start-banner.sh` stdout would show as a UI banner. Anthropic Claude Code docs confirm SessionStart stdout only injects into the model's context — never user-visible directly. Fix: added "Session opening" greeting protocol to `docs/ORCHESTRATION.md` (and Tarot's `docs/ORCHESTRATOR.md`) so the model echoes context to the user in its first reply. Banner script kept for context-injection purpose.

### Scope expansions
- **Session opening section** added to `docs/ORCHESTRATION.md` and Tarot's `docs/ORCHESTRATOR.md` (project-local). Not in v2.1 plan; surfaced as requirement after Tarot install showed silent SessionStart hook leaving Sếp without confirmation.
- **Realistic cost data point** captured for v2.2 optimization planning (~42k/multi-turn phiếu, breakdown roughly: Architect DRAFT ~55k, Worker CHALLENGE ~40k, Worker EXECUTE ~45k — heavily reduced by prompt cache).

### Edge cases / limitations found
- **`docs-gate` CLI default `valid_types` is missing `chore`.** Tarot fixed locally in P030 (`.docs-gate.toml` adds `chore` + `Chore`). The same gap will hit any new project running `docs-gate init`. **Action:** open ticket against `~/docs-gate` repo (the Rust binary's default config), NOT sos-kit. sos-kit doesn't ship `.docs-gate.toml` templates — that file is generated by `docs-gate` itself.
- **Worker re-CHALLENGE V2 took 13 min wall-clock + 28k output tokens** in P030 (slowest single phase observed). Reason: Worker re-grep'd + `xxd` byte-verified comment ranges. This is correct rigor; cost trade-off accepted.
- **Architect DRAFT may bịa anchors** (P029 example). This is expected and the entire reason Worker CHALLENGE exists. The framework is robust to Architect hallucination as long as Worker rigorously grep-verifies.

### Docs updated to match reality
- `docs/ORCHESTRATION.md` — added "Session opening (first user message)" section between "Why a 4th role" and "State machine"
- `~/tarot/docs/ORCHESTRATOR.md` — mirrored Session opening section in Tarot's project-local copy (committed at tarot `36e626f`)

### Notes for future phiếu
- v2.2 candidates surfaced: (1) skip-CHALLENGE for explicitly-trivial phiếu (comment-only, rename, docs-pure) — needs criteria; (2) Haiku for Architect DRAFT — DRAFT mostly synthesizes docs, not creative reasoning; (3) inline doc snippets in spawn prompt to skip subagent's Read step. Park until 5–10 more multi-turn phiếu deliver real cost-distribution data.

---

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
