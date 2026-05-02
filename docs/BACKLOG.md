# sos-kit Backlog

> **Single source of truth for "what to do next on sos-kit itself."**
> Live tracker. SessionStart hook surfaces Active sprint into the model's context on every new Claude Code session in this repo. Pick an item or capture a new idea via `/idea` skill.
>
> **Architect Rule 0:** Only write phiếu for items in **Active sprint**, or for items the maintainer has explicitly promoted from "Next sprint." No phiếu for "Open backlog" / "Park" without explicit promotion.

---

## 🔥 Active sprint: Worker capability + install UX gaps (resumed)

> **Goal:** Close 2 gaps post-Foundation-v2.2: (1) P005 — Worker Skill access (DECISION PENDING — Sếp pick A/B/C); (2) P006 — pre-commit fresh-install friction (`docs-gate` fails on fresh repo). Both now benefit from P036 tier rule — P006 likely Tầng 2 surgical → skip CHALLENGE expected.
> **Done when:** Both phiếu shipped + dry-run fresh-install zero-workaround.
> **Started:** 2026-04-26 (paused for Foundation v2.2; resumed 2026-04-27 after Foundation v2.2 ship).

- [ ] **[P005]** Worker Skill access — `agents/worker.md:4` `tools:` allowlist không có `Skill`. **DECISION PENDING:**
  - **A.** Add `Skill` vào worker tools allowlist (1-line edit). Pragmatic.
  - **B.** *(em recommend)* Architect/Orchestrator run skill trước CHALLENGE, đổ output vào phiếu. Worker chỉ apply.
  - **C.** Hybrid — Worker invoke skill chỉ khi phiếu có flag `requires_skill: <name>`.
  - Memory ref: `project_tarot_frontend_design_plugin.md`. Existing [P008] DEPENDS on outcome.
- [ ] **[P006]** Pre-commit fresh-install friction — `hooks/pre-commit` shells `docs-gate` failing on fresh repo. **Options:** A (soft-fail), B (bootstrap CHANGELOG/ARCHITECTURE skeleton in INSTALL.md), C (loosen hook). Note: cũng nên xét default `.docs-gate.toml` template trong `templates/`. **Strong P006 evidence accumulated:** P035 + P037 EXECUTE both reported "docs-gate not runnable in sos-kit root (no `.docs-gate.toml`)" — friction confirmed in real motion, not theoretical.

---

## 🎯 Next sprint candidates: Distribution — plugin + Rust CLI cohabit

> **Trigger:** Active sprint (P003 + P004 drift fixes) shipped + maintainer signs off "drift = 0 on fresh install."
> **Theme:** Two install paths, complementary not competing. Plugin = the brain (skills/agents/hooks Claude Code consumes natively). Rust CLI = the hands (filesystem ops, settings.json merge, doctor check, CI scriptable). User picks: plugin-only (simple) or plugin + CLI (recommended for own dogfood).
> **Plan basis:** Claude Code plugin spec confirmed via `claude-code-guide` agent — manifest at `.claude-plugin/plugin.json`, bundles agents + skills + hooks + bash scripts + templates + MCP. Cannot auto-modify user's `.claude/settings.json`, but plugin's bash script (in `bin/`) can with user permission.

- [ ] **[P032]** Phase 1 (MVP) — **sos-kit Claude Code plugin**. Bundle: `agents/architect.md` + `worker.md`, all 9 skills + new `/sos:init` + new `/phieu`, hooks (`hooks/hooks.json` for PreToolUse architect-guard + SessionStart banner), bash scripts in `bin/`, markdown templates (TICKET, BACKLOG, vision/*). Manifest at `.claude-plugin/plugin.json`. User flow: `/plugin install --url https://github.com/aspelldenny/sos-kit` → `/sos:init <project>` → bash script in plugin's `bin/` scaffolds project files + prompts user to merge `.claude/settings.json` (permission gate).
  - **[P032.1]** `/sos:init <project>` skill + companion bash script (creates `docs/BACKLOG.md`, vision templates, `.phieu-counter`, settings.json merge with permission)
  - **[P032.2]** `/phieu <slug>` skill — port `phieu.sh` shell function as cross-platform skill (Windows OK without bash setup)
- [ ] **[P033]** Phase 2 (main, confirmed direction) — **sos-kit Rust CLI**. Standalone binary matching `ship`/`docs-gate`/`guard`/`vps` pattern. Subcommands:
  - `sos-kit init <project>` — clean JSON merge into `.claude/settings.json` + scaffold (no permission prompt friction since it's the user's own CLI)
  - `sos-kit upgrade` — sync project's `.claude/agents/` + scripts from canonical sos-kit, detect + report drift
  - `sos-kit doctor` — verify install (hooks wired, agents register, BACKLOG present, vision docs present)
  - `sos-kit phieu <slug>` — port shell function as proper subcommand with worktree support
  - **Unique value over plugin:** runs outside Claude Code session (useful in CI / scripts / cron), proper JSON merge without per-call permission prompts, cross-project ops, can be invoked from other Rust tools. Companion to existing Rust ecosystem.
- [ ] **[P034]** Distribution channels —
  - **Plugin:** GitHub URL install (immediately works) + marketplace submission via `platform.claude.com/plugins/submit`
  - **Rust CLI:** `cargo install sos-kit` + Homebrew tap `aspelldenny/homebrew-sos` + GitHub Releases pre-built binaries (macOS / Linux / Windows)
  - Documentation in `INSTALL.md` showing both paths side by side

---

## 🌊 Future waves (low commitment)

- [x] **v2.2 — Debate token optimization.** Closed 2026-05-02 — partially shipped via **P038** (per-phiếu Discovery file pattern, Debate Log strip on phieu-done, banner size-warn at 40k threshold). Remaining candidates (skip-CHALLENGE for trivial phiếu — already done via P036 tier routing; Haiku for Architect DRAFT — still parked, awaiting evidence; inline doc snippets — parked).
- [ ] **Multi-project support.** Single sos-kit install serving N projects with centralized `agents/` + `scripts/` + project-local override. Avoids the "8 files copied per project" bootstrap cost. Likely depends on P033 Rust CLI.

---

## 💡 Open backlog (triaged, not yet sprinted)

- [ ] **[P007]** *(Tầng 2 housekeeping leftover từ P004)* `bin/sos.sh:94` echo help text vẫn còn literal `docs/CHARACTER.md` — cosmetic, không ảnh hưởng agent envelope rule. 1-line edit thành `docs/CHARACTER*.md` cho consistency. Worker đã classify cosmetic exclusion ở P004 EXECUTE — promote khi rảnh hoặc gom với phiếu housekeeping khác.
- [ ] **[P009]** Notification hook contract — orchestrator fire event sau mỗi state transition (Architect DRAFT/RESPOND done, Worker CHALLENGE/EXECUTE done, APPROVAL_GATE pending), invoke `integrations/notify/notify.sh <event> <payload-json>` nếu exists, no-op nếu không. Kit ship CONTRACT.md + 3 example scripts (`telegram.sh`, `slack.sh`, `macos.sh`); user symlink hoặc copy. **Lý do cần:** subagent runs 2-7 phút (drift sprint P004 RESPOND mất 4:01) → AFK cost cao. Tarot evidence: P040 phiếu 1h29m / 158k tokens, AFK = mất focus block. **Trade-off:** kit complexity +1 hook layer, nhưng pattern-clean (orchestrator chỉ "fire event", không biết Telegram). **Trigger để promote vào Active sprint:** Sếp ship personal Telegram script trước (~30 phút, reuse `integrations/jarvis/` pattern), dùng ≥2 tuần, confirm valuable → kit-level phiếu (~2-3h: CONTRACT.md + orchestrator hook + 3 example + INSTALL.md note + dry-run test). Memory cross-ref: session log 2026-04-26 có full eval (5-yếu-tố matrix + so sánh với github-actions/jarvis pattern).
- [ ] **[P008]** Frontend-design plugin workflow doc (`phieu/FRONTEND_WORKFLOW.md`). When phiếu touches FE/UI/UX → Worker invokes `frontend-design` plugin (claude-plugins-official) for design tokens + component spec, instead of ad-hoc design. **DEPENDS ON P005 outcome:** if Sếp picks Option B (Architect runs skill, not Worker), P008 scope shrinks to "doc workflow ở Architect side", không phải Worker handbook entry. Re-scope sau khi P005 ship.
- [ ] **[P010]** `phieu/AUDIT_TEMPLATE.md` — skeleton fill for AUDIT_PROTOCOL. Currently audit-runner has to build the report structure from scratch; a template halves prep time.
- [ ] **[P011]** Worker AUDIT mode handbook section in `agents/worker.md`. Currently AUDIT mode is documented in `phieu/AUDIT_PROTOCOL.md` only; Worker handbook should declare the mode and trigger phrase.
- [ ] **[P012]** Orchestrator auto-detect "≥N phiếu since last audit" → suggest running AUDIT. State in `docs/ORCHESTRATION.md` or a small `.audit-counter`.
- [ ] **[P013]** Vietnamese 13-checks (diacritics, VND, GMT+7, font rendering, PDF export, etc.) → CI gate that runs pre-deploy. Currently a manual checklist in AUDIT_PROTOCOL.
- [ ] **CLAUDE.md tree refresh** — current tree in `CLAUDE.md` does not list `CHANGELOG.md`, `DISCOVERIES.md`, `BACKLOG.md`, `docs/ORCHESTRATION.md`. Minor doc drift; refresh when next touching CLAUDE.md.
- [ ] **External (out of sos-kit scope)** — `~/docs-gate` repo: default `valid_types` should include `chore`. Currently every project that uses `chore`-typed phiếu must add it manually to local `.docs-gate.toml` (Tarot fixed in tarot PR #253).

---

## 🅿️ Park / consider further

- [ ] **Slash command `/build <item>` that runs the full state machine** (DRAFT → CHALLENGE → RESPOND → approval → EXECUTE) end-to-end with one user input. Heavy abstraction; may hide useful debate state. Reconsider after P032/P033 ship.
- [ ] **Telemetry** — opt-in usage stats (which skills, which modes, debate-turn distribution). Useful for evidence-based v2.2 optimization. Privacy + complexity trade-off.
- [ ] **Bidirectional Telegram control** — Sếp gửi command từ phone (e.g. `/idea X`, `/status`, `/approve P005`) → bot trigger Claude Code action remote. Depends on P009 (one-way notification) shipping first + Anthropic `RemoteTrigger` deferred tool maturity. Big concept (auth, security, command parsing). Reconsider sau khi P009 + Sếp dùng Telegram one-way ≥1 tháng.

---

## 📌 Recurring routines (not items, but reminders for the maintainer)

- **Pre-merge any PR:** run `/ultrareview <PR#>` for multi-agent cloud review.
- **After 5–10 phiếu or wave end:** run AUDIT_PROTOCOL (Worker AUDIT mode, read-only, writes `docs/AUDIT_<wave>.md`).
- **Per phiếu:** Worker writes Discovery Report to `docs/discoveries/P<NNN>.md` (per-phiếu file, P038 pattern) + appends 1-line index entry to `docs/DISCOVERIES.md` before reporting "done."

---

## ✅ Recently shipped

> Quick reference. Full detail in `CHANGELOG.md`.

- ✅ **P038 / v2.1.7** — (2026-05-02) — Phiếu lifecycle cleanup + safety rails. `phieu-done` extended (strip Debate Log, move active→done, delete branch -d, cleanup .backup/). Banner doc size-warn at 40k + cleanup-nudge for approved+merged phiếu. Worker safety rules (no force-push, no edit memory/settings outside scope, no rm-rf on absolute paths). Pre-phiếu snapshot Task 0 standard. DISCOVERIES.md per-phiếu file pattern (decouple from monolithic). Trigger: Tarot dogfood 2 weeks → 80% week usage Max plan; cost optimization.
- ✅ **Foundation v2.2 sprint COMPLETE** — (2026-04-27) — P036 + P035 + P037 shipped same day (PRs #3 + #4 + #5 merged). Total ~632k tokens / ~45m drive time across all 3. **P037 first Tầng 2 dogfood:** ~5min/81k tokens (68% reduction vs Tầng 1 baseline). **Rule B working:** 0 anchor mismatches at EXECUTE across all 3 phiếu — humility markers prevented hallucination cleanly.
- ✅ **P037 / v2.1.6** — (2026-04-27) — `templates/claude-settings.local.json` pre-approves marker file Bash ops + INSTALL.md Step 2.5 (PR #5)
- ✅ **P035 / v2.1.5** — (2026-04-27) — `agents/orchestrator.md` (~88-line condensed handbook) + ORCHESTRATION.md Hard rule #8 (bulk input → 1 gate) + INSTALL anti-patterns + CLAUDE.md contributor section (PR #4)
- ✅ **P036 / v2.1.4** — (2026-04-27) — Workflow tier routing (state machine `tầng==2` skip-CHALLENGE) + Architect humility markers (`[verified]` / `[needs Worker verify]`). Foundation rules specced (PR #3)
- ✅ **Drift-sprint COMPLETE** — (2026-04-26) — P003 + P004 merged on main. Dry-run zero-workaround.

---

## ❌ Rejected (kept here so we don't reconsider in 6 months)

*(empty)*

---

## 📌 Maintenance rules

1. **New idea** → `/idea` skill → appends to "Open backlog" or directly to "Active sprint" depending on triage.
2. **Phiếu shipped** → move from Active sprint to "Recently shipped" (keep last ~4 entries).
3. **Sprint complete** → summarize in CHANGELOG, prune "Recently shipped" if it grows past 4 entries.
4. **Discovery debt** new → from `DISCOVERIES.md` → append to "Open backlog" with `[DEBT]` prefix.
5. **Architect rule (hard):** no phiếu for items outside "Active sprint" without explicit promotion from the maintainer.
6. **Monthly Park review:** read Park, decide promote / hold / move to Rejected with reason.

---

*This file is LIVE. Maintainer edits it directly. Architect / Worker subagents READ it during sprint planning but never edit it mid-phiếu.*
