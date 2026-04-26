# sos-kit Backlog

> **Single source of truth for "what to do next on sos-kit itself."**
> Live tracker. SessionStart hook surfaces Active sprint into the model's context on every new Claude Code session in this repo. Pick an item or capture a new idea via `/idea` skill.
>
> **Architect Rule 0:** Only write phiếu for items in **Active sprint**, or for items the maintainer has explicitly promoted from "Next sprint." No phiếu for "Open backlog" / "Park" without explicit promotion.

---

## 🔥 Active sprint: Worker capability + install UX gaps

> **Goal:** Close 2 gaps surfaced post-drift-sprint: (1) Worker subagent cannot invoke Skill tool — blocks frontend-design plugin workflow at Worker level, forces fallback inline (Tarot PR #257 pattern); (2) fresh sos-kit install fails first `git commit` because pre-commit hook calls `docs-gate` which requires `docs/CHANGELOG.md` + `docs/ARCHITECTURE.md` not yet bootstrapped — INSTALL.md says docs-gate is "optional" but hook treats it as required.
> **Done when:** Both phiếu shipped (PR merged + CHANGELOG + Discovery Report) and a re-run of fresh-install dry-run shows zero workarounds end-to-end (including first commit).
> **Started:** 2026-04-26 (drift-sprint complete same day; this is the follow-on)

- [ ] **[P005]** Worker Skill access — `agents/worker.md:4` `tools:` allowlist không có `Skill` → Worker không invoke `/frontend-design` hay skill nào khác. **DECISION PENDING — Sếp pick A/B/C trước khi Architect draft phiếu:**
  - **A.** Add `Skill` vào worker tools allowlist (1-line edit). Pragmatic, Worker invoke `/frontend-design` trực tiếp. Risk: over-invoke skill ngoài scope — quản qua Architect spec rõ trong phiếu.
  - **B.** *(em recommend)* Architect/Orchestrator run skill trước CHALLENGE, đổ output (tokens/spec) vào phiếu. Worker chỉ apply. Giữ envelope sạch (Worker không "smart routing"). Fit role separation.
  - **C.** Hybrid — Worker invoke skill chỉ khi phiếu có flag `requires_skill: <name>` (Architect set). Middle ground.
  - Trigger: Sếp chọn → Architect draft phiếu cập nhật `agents/worker.md` + có thể `agents/architect.md` (option B/C) + `docs/HANDOFF.md` (mới 1 handoff cho skill output) + `docs/ORCHESTRATION.md` (option B mention skill-pre-CHALLENGE phase).
  - Memory ref: `project_tarot_frontend_design_plugin.md`. Existing Open backlog [P008] `frontend-design` workflow doc DEPENDS on outcome — option B sẽ thu hẹp P008 scope (skill chạy ở Architect, không cần Worker workflow doc).
- [ ] **[P006]** Pre-commit fresh-install friction — `hooks/pre-commit` shells ra `docs-gate` binary which fails on fresh repo (no `docs/CHANGELOG.md` / `docs/ARCHITECTURE.md`). User mới phải `--no-verify` lần đầu hoặc tạo CHANGELOG/ARCHITECTURE skeleton trước. **Options:**
  - **A.** Soft-fail: hook detect "first commit hoặc 0 prior commits" → docs-gate warn instead of block.
  - **B.** Bootstrap: INSTALL.md Step 3.5 thêm `cp templates/CHANGELOG_skeleton.md docs/CHANGELOG.md` + same cho ARCHITECTURE (cần tạo template chưa có).
  - **C.** Loosen: `hooks/pre-commit` skip docs-gate nếu file `docs/CHANGELOG.md` không tồn tại (warn "docs-gate skipped — create docs/CHANGELOG.md to enable").
  - Trigger: Architect đọc `hooks/pre-commit` + decide A/B/C kết hợp. Verified bằng dry-run mới (re-run /tmp/test-sos-install flow, expect commit success out-of-box).
  - Note: cũng cần xét `~/docs-gate` Rust binary — có nên ship default `.docs-gate.toml` template trong sos-kit `templates/` không? (Nằm 1 phần ở docs-gate repo, không phải sos-kit.)

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

- [ ] **v2.2 — Debate token optimization.** Park until ≥5 multi-turn phiếu deliver real cost-distribution data. Candidates: skip-CHALLENGE for trivial phiếu (needs criteria), Haiku for Architect DRAFT, inline doc snippets in spawn prompt to skip subagent's Read step. Baseline target: 42k → 25k tokens per multi-turn phiếu.
- [ ] **Multi-project support.** Single sos-kit install serving N projects with centralized `agents/` + `scripts/` + project-local override. Avoids the "8 files copied per project" bootstrap cost. Likely depends on P033 Rust CLI.

---

## 💡 Open backlog (triaged, not yet sprinted)

- [ ] **[P007]** *(Tầng 2 housekeeping leftover từ P004)* `bin/sos.sh:94` echo help text vẫn còn literal `docs/CHARACTER.md` — cosmetic, không ảnh hưởng agent envelope rule. 1-line edit thành `docs/CHARACTER*.md` cho consistency. Worker đã classify cosmetic exclusion ở P004 EXECUTE — promote khi rảnh hoặc gom với phiếu housekeeping khác.
- [ ] **[P008]** Frontend-design plugin workflow doc (`phieu/FRONTEND_WORKFLOW.md`). When phiếu touches FE/UI/UX → Worker invokes `frontend-design` plugin (claude-plugins-official) for design tokens + component spec, instead of ad-hoc design. **DEPENDS ON P005 outcome:** if Sếp picks Option B (Architect runs skill, not Worker), P008 scope shrinks to "doc workflow ở Architect side", không phải Worker handbook entry. Re-scope sau khi P005 ship.
- [ ] **[P010]** `phieu/AUDIT_TEMPLATE.md` — skeleton fill for AUDIT_PROTOCOL. Currently audit-runner has to build the report structure from scratch; a template halves prep time.
- [ ] **[P011]** Worker AUDIT mode handbook section in `agents/worker.md`. Currently AUDIT mode is documented in `phieu/AUDIT_PROTOCOL.md` only; Worker handbook should declare the mode and trigger phrase.
- [ ] **[P012]** Orchestrator auto-detect "≥N phiếu since last audit" → suggest running AUDIT. State in `docs/ORCHESTRATION.md` or a small `.audit-counter`.
- [ ] **[P013]** Vietnamese 13-checks (diacritics, VND, GMT+7, font rendering, PDF export, etc.) → CI gate that runs pre-deploy. Currently a manual checklist in AUDIT_PROTOCOL.
- [ ] **[P035]** Orchestrator contract — full refactor. Banner Option A (shipped 2026-04-26) injects 6-line reminder mỗi session, nhưng main session vẫn có thể drift sau context-compress. Full fix: tạo `agents/orchestrator.md` (system prompt cho main session, condensed từ `docs/ORCHESTRATION.md` ~80 lines), banner reference orchestrator.md, `INSTALL.md` Step 4 CLAUDE.md template thêm explicit anti-pattern warnings ("không fake-gate giữa phase"), sos-kit's own `CLAUDE.md` ref `docs/ORCHESTRATION.md` cho contributors. *(Trigger: verify Option A ≥1 tuần — nếu main session vẫn drift, promote vào sprint.)*
- [ ] **CLAUDE.md tree refresh** — current tree in `CLAUDE.md` does not list `CHANGELOG.md`, `DISCOVERIES.md`, `BACKLOG.md`, `docs/ORCHESTRATION.md`. Minor doc drift; refresh when next touching CLAUDE.md.
- [ ] **External (out of sos-kit scope)** — `~/docs-gate` repo: default `valid_types` should include `chore`. Currently every project that uses `chore`-typed phiếu must add it manually to local `.docs-gate.toml` (Tarot fixed in tarot PR #253).

---

## 🅿️ Park / consider further

- [ ] **Slash command `/build <item>` that runs the full state machine** (DRAFT → CHALLENGE → RESPOND → approval → EXECUTE) end-to-end with one user input. Heavy abstraction; may hide useful debate state. Reconsider after P032/P033 ship.
- [ ] **Telemetry** — opt-in usage stats (which skills, which modes, debate-turn distribution). Useful for evidence-based v2.2 optimization. Privacy + complexity trade-off.

---

## 📌 Recurring routines (not items, but reminders for the maintainer)

- **Pre-merge any PR:** run `/ultrareview <PR#>` for multi-agent cloud review.
- **After 5–10 phiếu or wave end:** run AUDIT_PROTOCOL (Worker AUDIT mode, read-only, writes `docs/AUDIT_<wave>.md`).
- **Per phiếu:** Worker writes Discovery Report to `docs/DISCOVERIES.md` (newest on top) before reporting "done."

---

## ✅ Recently shipped

> Quick reference. Full detail in `CHANGELOG.md`.

- ✅ **Drift-sprint COMPLETE** — (2026-04-26) — P003 + P004 merged on main (`91d62af` + `14819b3`). Dry-run on `/tmp/test-sos-install` verified: P003 banner fallback + P004 CHARACTER glob both work zero-workaround. Separate gap surfaced (pre-commit + docs-gate friction) → tracked as P006 in next sprint, NOT a P003/P004 regression.
- ✅ **P004 / v2.1.3** — (2026-04-26) — Vision doc naming flex: `docs/CHARACTER*.md` glob in agents + doc consistency (HANDOFF, LAYERS, SETUP, GENESIS)
- ✅ **P003 / v2.1.2** — (2026-04-26) — BACKLOG format flexibility: banner + Architect Rule 0 + ORCHESTRATION.md all tolerate non-"Active sprint" section headers via fallback
- ✅ **v2.1.1** — `c786359` (2026-04-26) — Session opening protocol + Tarot dogfood verification (P029 smoke + P030 multi-turn debate, value proven, ~42k tokens/multi-turn cost baseline)
- ✅ **P002 + P001 + v2.1 (audit)** — (2026-04-26) — Vision templates harvest, debate loop, AUDIT_PROTOCOL (pruned to 1 line per maintenance rule "keep last ~4 entries")

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
