# Changelog

All notable changes to sos-kit. Format loosely follows Keep a Changelog. Versions are wave-based, not date-based.

## v2.3 forge (in progress) — Phiếu path + sentinel + agents-drift cure — 2026-06-01

Forge rounds 1–4 of `docs/retro/WORKFLOW_V2.3_RETRO_doc-rotate.md` (doc-rotate pilot vòng 2 retro). Doctrine still in forge — only mechanical/path fixes shipped here; doctrine questions Q-D2…Q-D7 (Lane/Tầng, markers, inject location, vocab-consistency, verify-setup + quality-canary split) remain OPEN.

**Phiếu path unified to `docs/ticket/` (Q-D1 resolved):**
- Migrated sos-kit's own phiếu `phieu/active|done/` → `docs/ticket/` (active root) + `docs/ticket/done/` (14 phiếu archive). No more dogfood-exception — single path matches the fleet default (downstream repos already use `docs/ticket/`).
- `hooks/pre-commit` now **reads `ticket_dir` from `.docs-gate.toml`** instead of hardcoding `^docs/ticket/` — revives the Discovery-enforcement gate (was structurally dead on phieu/ repos) and makes it portable per-repo (single-source principle).
- Swept: `.docs-gate.toml` (`ticket_dir = "docs/ticket"`), `scripts/session-start-banner.sh`, `phieu/phieu.sh` (location-detect), `phieu/TICKET_TEMPLATE.md`, `agents/worker.md` (×2), `CLAUDE.md` structure, `docs/BOOTSTRAP_AUTOMATION_DRAFT.md` default, `docs/ORCHESTRATION.md`.
- Preserved: historical phiếu content + this CHANGELOG (no rewrite history); `phieu/` backbone (TICKET_TEMPLATE/phieu.sh/protocols/VISION_TEMPLATES) retained as the workflow component; legacy `phieu/active` fallback kept in `phieu.sh` + banner for older repos.

**Sentinel mismatch fixed (Step-1):**
- `scripts/block-unsafe-merge.sh` grepped `<!-- SECURITY_REVIEW_START -->` (UPPERCASE/underscore) while `agents/boundary-check.md` + `/security-review` emit `<!-- security-review-start -->` (lowercase/hyphen) — the merge-gate verdict block never matched, breaking the advisory→merge-block chain at its root. Fixed grep to match the emitted marker.

Verified: gate canary fires on `docs/ticket/`; `bash -n` clean on edited scripts. Open production-risk tracked in retro: tarot's Giám sát (boundary-check v166) runs live but has never been quality-canaried.

**Found — fix DEFERRED to next session (in-repo `.claude/agents/` drift, 2026-05-30):**
`scripts/sync-personal-agents.sh` (regen `.claude/agents/` from canonical `agents/`) is a **memory-dependent** sync step that **died ~1 month** → the run/propagated copy went stale → a major cause of the media collapse. = Sub-mech A (trigger-gap) living inside the kit. **Symptom patched** (re-ran sync, `796feb2`); **structural cure deferred** — it's next session's **TOP task, before the tarot→sos-kit sync**. Fix-direction: automate (pre-commit regen) / attach-to-an-action / or eliminate the 2-copy (SOUL: is the `agents/`-Chủ-nhà vs `.claude/agents/`-Sếp split worth a death-point?). Tracked: BACKLOG "🔝 IN-REPO drift" + `GAP_AUDIT_tarot_to_soskit.md`.

**Tầng = consequence (LOC removed) + un-staled `.claude/agents/` (2026-05-30):**
- **Removed the "≤3 files / ≤200 LOC" Tầng criterion from ALL live doctrine** — a small-LOC security fix is still Tầng-1; LOC mis-classifies (the exact bug that collapsed the media pilot). Tầng is now **single-sourced in `docs/LAYERS.md` §2-tier** by CONSEQUENCE (mistake **LAN** or **NOT-reversible** → Tầng-1; security/auth/schema/privacy/payment/`INV-LOCAL` → AUTO Tầng-1 even if 1 line; size is not a signal). `agents/orchestrator.md` (now **reads** the field, does NOT re-judge), `agents/architect.md`, `docs/ORCHESTRATION.md`, `phieu/TICKET_TEMPLATE.md`, `phieu/DISCOVERY_PROTOCOL.md` reference LAYERS.md instead of restating (closes the two-voice drift).
- **Un-staled `.claude/agents/`:** the generated copy (architect/worker) was last synced **2026-04-26 — ~1 month stale**, missing all v2.2 evolution (Oracle/AGENT_MAP/Humility/Bước-0). sos-kit **dogfooded + adopt propagated the stale April agents** — a major cause of the media collapse (media got month-old architect/worker). Re-ran `scripts/sync-personal-agents.sh` → `.claude/agents/` now == canonical (architect 191→276, worker 183→293). Same Sub-mech A: the regen mechanism existed but was never fired. **Flag (not built):** sync script covers only architect+worker; a pre-commit drift-check (`.claude/agents/` == `agents/` name-swapped) would prevent re-staleness.

**In-repo `.claude/agents/` drift CURED — eliminate via symlink + tên-vai/xưng-hô split (closes the deferred item above) — 2026-06-01:**
The 2026-05-30 deferred TOP task is resolved. Root finding: the `sed 's/Chủ nhà/Sếp/g'` swap was a **category error** — `Chủ nhà` is a **role name** (tên vai), `Sếp` an **address term** (xưng hô); grep confirmed 100% of handbook `Chủ nhà` usage is 3rd-person role-reference, zero vocative. The two-copy split personalized nothing the conversation layer doesn't already handle, so it was pure drift surface.
- **Cure = eliminate (not automate):** `.claude/agents/{architect,worker,advisory-watch,boundary-check}.md` are now **symlinks** → `../../agents/*.md`. One real file per agent → drift is structurally impossible (no second copy to diverge). Removed `scripts/sync-personal-agents.sh` + its SessionStart hook. Stronger than the "automate via hook" option first considered: a symlink has no sync step to fire at all.
- Cleaned stray `Sếp` referents → `Chủ nhà` in `agents/orchestrator.md` (×5) + `agents/worker.md` (×5); canonical handbooks are now pure role-name voice.
- Documented the **tên-vai vs xưng-hô** two-layer rule in `CLAUDE.md` (Language) + `agents/README.md`. The `Sếp`/`anh`/`em` register now lives only in live chat + UI (e.g. the SessionStart banner), never in handbook role-references.
- External adopters copy real files from `agents/` (per `INSTALL.md`), so the symlinks are sos-kit-internal and don't affect downstream copies.

**Top-3 trigger-doctrine sync (gap-audit → kit, 2026-05-30):**
Closes the dominant gap-audit finding — the kit shipped GATES without the DOCTRINE/TRIGGER that fires them (Sub-mech A re-created inside the kit; what made the media pilot collapse).
- `docs/ORCHESTRATION.md` **Rule 10** (pre-merge security gate — `gh pr diff --name-only` before any merge; security surface → `/security-review` `APPROVE` required; `block-unsafe-merge.sh` = mechanical backstop) + **Rule 11** (advisory staleness auto-spawn). Condensed mirror in `agents/orchestrator.md`.
- `scripts/session-start-banner.sh` — advisory staleness check (reads `docs/security/.advisory-scan-state`; 🚨 ≥7d / ⚠️ 3–6d; cross-platform date GNU+BSD; gated on inbox presence so non-advisory repos stay silent) → Trinh sát now fires unprompted.
- `agents/architect.md` **Bước 0** — Layer-1 tool-capability verify (WebFetch GET-only, etc.); pairs with the already-present Worker Task-0 Layer-2 = the "ship≠chạy" 2-layer defense.
- NOT synced (deliberate): tarot product-specifics; sos-kit stays AHEAD on v2.2 (lane/sensor/rubric/multi-stack/INV-4). **Next:** the orchestrator/architect Tầng "≤200 LOC" contamination → single-source to `LAYERS.md` consequence-def (see `GAP_AUDIT_tarot_to_soskit.md` + retro Vòng 14).

**Gap-audit tarot→sos-kit + spawn-discretion findings (2026-05-29):**
- `docs/GAP_AUDIT_tarot_to_soskit.md` — the kit lagged its flagship (tarot). Read-only inventory of GENERIC-doctrine gaps. **Dominant disease:** kit ships the GATE but drops the DOCTRINE that fires it = Sub-mech A "trigger gap" re-created inside the kit (what made the media pilot collapse). **Top-3:** advisory auto-spawn (Rule 10 + banner staleness), pre-merge security-gate doctrine (Rule 9), Architect Layer-1 capability check. **Direction (Sếp):** tarot FROZEN — fix kit→media/new-projects as proving-ground; back-port to tarot later. sos-kit is AHEAD on v2.2 (lane/sensor/rubric/multi-stack/INV-4) — do NOT sync down.
- `docs/retro/WORKFLOW_V2.3_RETRO_doc-rotate.md` Vòng 13/14: spawn-discretion bug (mechanical proof — doc-rotate 23 spawns vs media 0, same delegation prompt) + corrected fix-direction (sound/partial: Tầng = human judgment via INV-LOCAL/INVARIANTS membership; hooks enforce field-existence + role-ran state-check, NEVER hook-detect-security; closes Q-D2 seedling-disease).

**Bootstrap Cat-A freeze completed — skills + doctor MCP wiring (2026-05-29):**
- `bin/sos.sh` `sos new` + `sos adopt` now copy the **13 generic SOS skills** (`skills/` → `.claude/skills/`) and wire **OUR `doctor` MCP gate** in `.mcp.json` (PATH-rel `doctor`, no hardcoded path). Closes a real freeze gap found dogfooding media-rating: adopt copied the role agents but not the skills they invoke (`/plan` `/verify` …) nor the `.mcp.json` that wires Giám sát's `mcp__doctor__*` tools → both were dead in the adopted repo.
- `adopt` is ADDITIVE/non-clobber: keeps a repo's own domain skills (Cat-C) + its own external MCP (`.mcp.json` flagged, never overwritten), adds only the generic spine.
- Doctrine (`docs/BOOTSTRAP_AUTOMATION_DRAFT.md` §7.2): freeze-tier is an **attribute filter** (same-across-repo + in-control = freeze; per-repo = survey/declare; external-dep = drop), not an item-type list. Skills split generic-freeze / domain-Cat-C; MCP split by ownership (our tools wired, external dropped — *LLM reasons, our tools are the hands*). The deeper per-repo survey step (same root as the map-lie gap) is tracked in BACKLOG, not built.
- Tested green: greenfield `sos new` (skills=13, `.mcp.json` valid PATH-rel, verify-setup CONNECTED) + 2 brownfield `sos adopt` (own skill/MCP kept, generic added, non-clobber confirmed).

## v2.2 Backport Group A — Security hooks + agents + commands — 2026-05-28

Phase 2 of `WORKFLOW_V2.2.md` §13 backport from tarot evolution (P230/P273/P297/P305/P306/#581). Genericized for sos-kit template.

**Hooks + scripts (7 new + 2 modified):**
- `scripts/block-env-edit.sh` — PreToolUse block Edit/Write to `.env*` (allow `.env.example`)
- `scripts/block-unsafe-merge.sh` — PreToolUse block `gh pr merge <N>` if security surface + no `/security-review` APPROVE sentinel (§7 Sub-mech A + §8)
- `scripts/security-gate.sh` — minimal template (INV-009 + INV-010 universal); per-repo extends
- `scripts/check-hardcoded-secrets.py` — INV-009 enforcer (8 prefix patterns + generic high-entropy)
- `scripts/check-runtime-secrets.py` — INV-010 enforcer Sub-mech F (dotfile token leak: `.git/config`, `.mcp.json`, `.claude/settings.local.json`, infra)
- `scripts/install-hooks.sh` — bootstrap pre-commit + pre-push installation
- `scripts/pre-push-hook.sh` — advisory warn (KHÔNG block) for security-surface push
- `hooks/pre-commit` — add SECTION 4 wiring `security-gate.sh --mechanical-only`
- `.claude/settings.json` — merge 3 PreToolUse hooks

**Agents + commands (4 changed):**
- `.claude/agents/boundary-check.md` (NEW snapshot) + `agents/boundary-check.md` — wire `mcp__doctor__runtime_scan` + `mcp__doctor__validate_map` per tarot #581
- `.claude/agents/advisory-watch.md` (NEW snapshot from `agents/`)
- `.claude/commands/advisory-scan.md` — rewrite to advisory-inbox binary form (P013-tarot)

**Tracked `.mcp.json`:** Doctor MCP server registration (binary serve mode for `lane-check`, `validate-map`, `rotate-check`, `runtime-scan`).

**Out of scope (Phase 3):** CLAUDE.md doctrine refactor + worker.md Task 0 matrix + architect.md §2 oracle checklist + `AGENT_MAP.example.yaml`.

**Verification:** `security-gate.sh --mechanical-only` on sos-kit itself → 2/2 INV PASS, 12 files scanned.

## Sprint close: Tarot port wave 1 — 2026-05-25

All 4 phiếu complete: P040 (stack-detect `sos init security`) + P041 (Trinh sát advisory-watch) + P042 (Giám sát boundary-check) + P043 (Quản đốc persona codify). Security pipeline both sides shipped: `/advisory-scan` (external CVE/GHSA scan via Trinh sát) + `/security-review` (internal invariant scan via Giám sát). Kit now ships: stack detection → advisory scan → boundary check → ADVISORY PR comment flow. CHANGELOG range: v2.2.0 → v2.2.3.

## [v2.2.3] — 2026-05-25

### Added
- **P042: Giám sát (boundary-check) specialist subagent — generic port from tarot, strips 7 INV → 5 generic (drop tarot's nginx + users.credits).** Adds `agents/boundary-check.md` (read-only-output specialist, tools: Read/Grep/Glob/Bash-scoped-to-git-and-grep), `templates/INVARIANTS-template.md` (5-INV skeleton + user-added section), `.claude/commands/security-review.md` (orchestrator-side spawn-only caller, posts PR comment in ADVISORY mode — KHÔNG block merge). 5 generic INV: env var template / external service timeout / cross-user binding / webhook signature / dep major changelog audit. Sentinel markers: `<!-- security-review-start -->` / `<!-- security-review-end -->`. Silent-when-clean rule preserved from tarot P275 lesson. `docs/LAYERS.md` Giám sát column filled. `docs/HANDOFF.md` stub expanded to full handoff entry. `README.md` boundary-check subagent row + Security section extended. `docs/SETUP.md` Security pipeline Step 5 added. `CLAUDE.md` repo tree updated. Wave 1 final phiếu shipped — P040+P041+P042+P043 complete.

### Files changed
- New: `agents/boundary-check.md`, `templates/INVARIANTS-template.md`, `.claude/commands/security-review.md`
- Modified: `docs/LAYERS.md`, `docs/HANDOFF.md`, `README.md`, `docs/SETUP.md`, `CLAUDE.md`, `CHANGELOG.md`, `docs/DISCOVERIES.md`

## [v2.2.2] — 2026-05-25

### Added
- **P041: Trinh sát (advisory-watch) specialist subagent — generic port from tarot, strips tarot-specific paths.** Adds `agents/advisory-watch.md` (read-only-output specialist, tools: Read/Grep/Glob/WebFetch/WebSearch/Bash-scoped-to-parsers), `templates/advisory-inbox.md` (queue with sentinel wrappers `<!-- advisory-start --> / <!-- advisory-end -->`), `.claude/commands/advisory-scan.md` (orchestrator-side spawn-only slash command — first file in `.claude/commands/`). Implements `scripts/parsers/pnpm_lock_v9.py` (PyYAML, 2-level importers layout, peer-suffix strip) + `scripts/parsers/package_lock_v3.py` (JSON flat layout, v2/v3 compatible). Other 4 parsers stay P040 stubs. `docs/LAYERS.md` Specialist subagents subsection. `docs/HANDOFF.md` appendix for specialist-subagent pattern. `README.md` advisory-watch subagent row + Security pipeline mention. `docs/SETUP.md` Security pipeline subsection (PyYAML install + sos init security + /advisory-scan flow). `CLAUDE.md` repo tree updated. State persistence + vendor-page expansion deferred to follow-on phiếu.

### Files changed
- New: `agents/advisory-watch.md`, `templates/advisory-inbox.md`, `.claude/commands/advisory-scan.md`, `docs/discoveries/P041.md`
- Modified: `scripts/parsers/pnpm_lock_v9.py`, `scripts/parsers/package_lock_v3.py`, `docs/LAYERS.md`, `docs/HANDOFF.md`, `README.md`, `docs/SETUP.md`, `CLAUDE.md`, `CHANGELOG.md`, `docs/DISCOVERIES.md`

## [v2.2.1] — 2026-05-25

### Changed
- **P043: Doc drift consolidate — Quản đốc persona codify + alignment engineering + deferred-tool loading.** Main-session orchestrator persona renamed from "Kiến trúc sư" → "Quản đốc" across all operational docs (disambiguates from Kiến trúc sư subagent). Layer 0 Quản đốc row added to `docs/LAYERS.md` access matrix + ASCII diagram. `docs/PHILOSOPHY.md` "alignment engineering" subsection expanded with 3 sub-headings (How envelopes are enforced / Why "share context" is the trap / Why role separation not just prompt discipline). `docs/ORCHESTRATION.md` greeting script + edge case updated; "Why Quản đốc persona" subsection rewritten + 3 new subsections added (Greeting turn template / Tier priority routing rationale / Session opening script). Deferred-tool loading instruction added to `agents/orchestrator.md` + `CLAUDE.md`. `CLAUDE.md:149` orchestrator handbook cap raised ≤90 → ≤105 (Sếp decision 2026-05-25). Cross-ref pass: README.md + CLAUDE.md + HANDOFF.md updated. Dangling staged files (agents/orchestrator.md + docs/BACKLOG.md) committed atomically.

### Files changed
- Modified: `docs/LAYERS.md`, `docs/PHILOSOPHY.md`, `docs/ORCHESTRATION.md`, `agents/orchestrator.md`, `CLAUDE.md`, `README.md`, `docs/HANDOFF.md`, `docs/BACKLOG.md`, `docs/DISCOVERIES.md`
- New: `docs/discoveries/P043.md`

## [v2.2.0] — 2026-05-25

### Added
- **P040: Bootstrap stack detection — `sos init security` subcommand** auto-detects Node/Python/Rust/Go via manifest files (`package.json`, `pyproject.toml`, `requirements.txt`, `Cargo.toml`, `go.mod`) and lock files, writes `.sos-stack.toml` schema at project root. Adds 6 parser skeleton stubs at `scripts/parsers/` (all return `[]`; P041 fills implementations). Foundation for advisory-scan (P041) + security-review (P042). Schema: `schema_version`, `detected_at`, `sos_kit_version`, `[[stack]]` with `type`/`manifest`/`lock_file`/`lock_format`/`parser`. `sos init` (no args) Phase 0 behavior unchanged. `sos help` updated.
- `templates/.sos-stack.toml.example` — example schema for inspection/manual authoring.
- `.gitignore` — `__pycache__/` + `*.pyc` (P040 adds first Python files to repo).

### Files changed
- New: `scripts/parsers/pnpm_lock_v9.py`, `scripts/parsers/package_lock_v3.py`, `scripts/parsers/requirements_txt.py`, `scripts/parsers/pyproject_toml.py`, `scripts/parsers/cargo_lock.py`, `scripts/parsers/go_sum.py`, `templates/.sos-stack.toml.example`, `docs/discoveries/P040.md`
- Modified: `bin/sos.sh`, `docs/SETUP.md`, `README.md`, `.gitignore`, `CHANGELOG.md`

## [v2.1.10] — 2026-05-10

### Changed
- **P005: Worker Skill access — option B locked (Skills are Orchestrator-only).** ~2 weeks of A/B/C debate (started 2026-04-26) closed 2026-05-10. Option B: Orchestrator (main Claude Code session) invokes skills BEFORE spawning Architect/Worker, captures output verbatim, embeds in phiếu Context under `### Skills consulted` subsection as frozen artifact. Subagent `tools:` allowlists unchanged — `Skill` intentionally absent from both `agents/architect.md` and `agents/worker.md` (audit trail: option B = handbook codification, NOT tools-list change). Reproducibility: re-running a phiếu yields the same skill output.
- Files changed: `agents/orchestrator.md` (new "Invoking skills" section), `agents/architect.md` (1 bullet in DRAFT load-context), `agents/worker.md` (1 sentence in Hard envelope rules), `docs/ORCHESTRATION.md` (Hard rule #9 + example session paragraph), `phieu/TICKET_TEMPLATE.md` (optional `### Skills consulted` subsection), `docs/LAYERS.md` (access matrix Skills row + footnote), `docs/BACKLOG.md` (flip P005 + re-scope P008), `docs/discoveries/P005.md` (new), `docs/DISCOVERIES.md` (index row).

## [v2.1.9] — 2026-05-10

### Fixed
- **P006: Pre-commit fresh-install friction — docs-gate bootstrap.** Three incidents (P035, P037, media-rating-app P001) showed `hooks/pre-commit` failing ungracefully on repos without `.docs-gate.toml`. Fix: (1) guard preamble in hook — missing config prints yellow warning + skips docs-gate check (no hard fail, other checks still run); (2) `templates/.docs-gate.toml` reference template for downstream sos-kit-style projects; (3) sos-kit root `.docs-gate.toml` dogfood config so kit validates itself; (4) `docs/SETUP.md` bootstrap step added after hook copy instruction.
- Files changed: `templates/.docs-gate.toml` (new), `.docs-gate.toml` (new), `hooks/pre-commit`, `docs/SETUP.md`, `docs/discoveries/P006.md` (new), `docs/DISCOVERIES.md`.

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
