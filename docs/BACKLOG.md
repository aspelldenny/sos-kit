# sos-kit Backlog

> **Single source of truth for "what to do next on sos-kit itself."**
> Live tracker. SessionStart hook surfaces Active sprint into the model's context on every new Claude Code session in this repo. Pick an item or capture a new idea via `/idea` skill.
>
> **Architect Rule 0:** Only write phiếu for items in **Active sprint**, or for items the maintainer has explicitly promoted from "Next sprint." No phiếu for "Open backlog" / "Park" without explicit promotion.

---

## 🔥 Active sprint: Drift fixes after Tarot dogfood

> **Goal:** Fix the 2 upstream gaps surfaced when v2.1 was installed into Tarot. Both leak into any new project install — must close before shipping a single-command installer.
> **Done when:** Both phiếu shipped (PR merged + CHANGELOG entry + Discovery Report) and a fresh install dry-run on a scratch repo shows zero workarounds needed.
> **Started:** 2026-04-26

- [ ] **[P003]** BACKLOG format flexibility — `scripts/session-start-banner.sh` currently hard-codes `^## .*Active sprint` grep; fall back to first `## ` section if header missing. Architect Rule 0 reads BACKLOG more flexibly. *(Tarot worked around by restructuring its BACKLOG; that workaround should not be mandatory.)*
- [ ] **[P004]** Vision doc naming flex — `agents/architect.md` envelope rule says "cannot read `docs/CHARACTER.md`" but Tarot's file is `CHARACTER_CHI_HA.md`. Architect should glob `docs/CHARACTER*.md`, OR INSTALL.md should document a rename / symlink convention. *(Tarot worked around with a symlink.)*

---

## 🎯 Next sprint candidates: Single-command install

> **Trigger:** Active sprint shipped + maintainer signs off "drift = 0 on fresh install."
> **Theme:** Make `sos-kit init <project>` a one-command experience (no manual copy-paste of 8 files like current INSTALL.md Step 1).

- [ ] **[P032]** Phase 1 MVP — bash installer + `init` subcommand as bash script. `curl -fsSL .../install.sh | bash` adds `sos-kit` shell function to `~/.zshrc`; `sos-kit init <project>` bootstraps `.claude/agents/`, `scripts/`, `docs/BACKLOG.md`, hooks, vision templates.
- [ ] **[P033]** Phase 2 main — proper Rust CLI `sos-kit` (matches `ship`/`docs-gate` pattern). Subcommands: `init`, `upgrade` (sync `.claude/agents/` from canonical), `doctor` (verify install state), `phieu <slug>` (port shell function as proper subcommand). Distributed via `cargo install sos-kit`.
- [ ] **[P034]** Phase 2 distribution — Homebrew tap `aspelldenny/homebrew-sos`. Pre-built binaries on GitHub Releases for macOS / Linux / Windows (Git Bash or WSL for hooks).

---

## 🌊 Future waves (low commitment)

- [ ] **v2.2 — Debate token optimization.** Park until ≥5 multi-turn phiếu deliver real cost-distribution data. Candidates: skip-CHALLENGE for trivial phiếu (needs criteria), Haiku for Architect DRAFT, inline doc snippets in spawn prompt to skip subagent's Read step. Baseline target: 42k → 25k tokens per multi-turn phiếu.
- [ ] **Multi-project support.** Single sos-kit install serving N projects with centralized `agents/` + `scripts/` + project-local override. Avoids the "8 files copied per project" bootstrap cost. Likely depends on P033 Rust CLI.

---

## 💡 Open backlog (triaged, not yet sprinted)

- [ ] **[P008]** Frontend-design plugin workflow doc (`phieu/FRONTEND_WORKFLOW.md`). When phiếu touches FE/UI/UX → Worker invokes `frontend-design` plugin (claude-plugins-official) for design tokens + component spec, instead of ad-hoc design.
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

---

## 📌 Recurring routines (not items, but reminders for the maintainer)

- **Pre-merge any PR:** run `/ultrareview <PR#>` for multi-agent cloud review.
- **After 5–10 phiếu or wave end:** run AUDIT_PROTOCOL (Worker AUDIT mode, read-only, writes `docs/AUDIT_<wave>.md`).
- **Per phiếu:** Worker writes Discovery Report to `docs/DISCOVERIES.md` (newest on top) before reporting "done."

---

## ✅ Recently shipped

> Quick reference. Full detail in `CHANGELOG.md`.

- ✅ **v2.1.1** — `c786359` (2026-04-26) — Session opening protocol + Tarot dogfood verification (P029 smoke + P030 multi-turn debate, value proven, ~42k tokens/multi-turn cost baseline)
- ✅ **v2.1 (audit)** — `c172507` (2026-04-26) — `phieu/AUDIT_PROTOCOL.md` (RRI-T-lite periodic audit, 4-result model, Worker AUDIT mode)
- ✅ **P002** — `4079e41` (2026-04-26) — Vision templates harvest from Tarot (CHARACTER enriched, VOICE / TEST_CASES / DESIGN_SPEC new)
- ✅ **P001** — `1642d83` (2026-04-26) — Architect ↔ Worker pre-code debate loop (state machine, Debate Log section, marker hygiene, sed-sync script)

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
