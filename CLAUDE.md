# CLAUDE.md — SOS Kit

> Read this before editing anything in this repo.
> This is a **meta-kit** (documentation + templates + skill definitions). It is **not** a production app.

**Workflow doctrine source (single-source-of-truth):** `docs/WORKFLOW_V2.2.md`. All agent handbooks (`agents/*.md`) reference v2.2 — conflict resolution: WORKFLOW_V2.2.md wins. Retro trace: `docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md` (CLOSED 2026-05-28, 7-round forge).

## What this repo is

SOS Kit = "Solo Operating System" — a distribution center that packages a **3-role workflow + orchestrator persona** for one-person software teams: **Chủ nhà** (owner / vision / routing), **Kiến trúc sư** (architect / ticket writer / docs-only), **Thợ** (worker / code executor), plus **Quản đốc** (Layer 0 — the main Claude Code session's orchestrator persona in v2.1+ Subagent mode). See `docs/LAYERS.md` for layer specifics.

What's inside:
- `docs/LAYERS.md` — the 3-role model, access matrix, 2-tier authority, anti-patterns
- `docs/HANDOFF.md` — 5 handoff protocols (insight briefing, routing, phiếu, escalation, discovery)
- `docs/PHILOSOPHY.md` — 7 principles (role separation #6, adopt-hiểu-repo #7)
- `docs/SETUP.md` — install guide for Rust tools + skills + phiếu shell function
- `phieu/` — ticket workflow backbone
  - `README.md`, `TICKET_TEMPLATE.md`, `phieu.sh` — the core
  - `DISCOVERY_PROTOCOL.md` — Thợ → Kiến trúc sư feedback + mismatch classification
  - `RELAY_PROTOCOL.md` — Chủ nhà's courier workflow (Thợ cannot ping Kiến trúc sư directly)
  - `VISION_TEMPLATES/` — day-1 skeletons for `PROJECT.md`, `SOUL.md`, `CHARACTER.md`
- `skills/` — Claude Code skills (5 LIVING — each declares a mechanical `caller:` in frontmatter; `skills/attic/` = 8 parked after the 2026-06-11 full dogfood, see `docs/retro/SKILLS_DOGFOOD_2026-06-11.md` + `docs/LAYERS.md` skills map)
- `configs/` — `.ship.toml` templates per stack (nextjs, flask, rust, python)
- `hooks/pre-commit` — git hook script (type-check + docs-gate). `[3/8]` section includes sub-check `3f` (`scripts/lane-check-contract.sh`, P082) — does NOT add a new phase, count stays `[8/8]`.
- `integrations/` — CI snippets (GitHub Actions canary) + Telegram uptime monitor
- `README.md` — entry point for new users

## What this repo is NOT

- **Runtime monorepo (as of P077e, relocated to repo-root P077f).** This repo now contains the canonical runtime source for the `sos` CLI — a Rust workspace at repo-root (`Cargo.toml` + `crates/`: `sos-cli`/`sos-core`/`sos-install`/`sos-adapter-claude`/`sos-hooks`). The 6 heavy `sos` subcommands (`new`/`adopt`/`sync`/`map`/`install`/`tools`) dispatch to this binary; `bin/sos.sh` is a thin launcher that keeps only the 7 Claude-flavored guidance commands (`init`/`blueprint`/`contract`/`apply`/`recipe`/`launch`/`status`) in Bash until P078 renders them per-runtime. The **sister** CLIs (`ship`, `docs-gate`, `guard`, `vps`) STILL live in their own repos (`~/ship` etc.) — this repo references + version-pins them via `tool-manifest.toml`, it does not vendor their source.
- **Not a boilerplate project scaffolder.** `recipes/` provides battle-tested **patterns** (DNA snippets, decision rationale) that `/apply` consumes — but the kit doesn't generate full app templates from a blank directory. SOS Kit picks up after "code is ready," not "project is empty."
- **Not a planning methodology.** Use your own (Shape Up, Vibecode, whatever). SOS Kit picks up after "code is ready."
- **Not a place for experimental features.** If a skill or config hasn't been used on a real project for ≥2 weeks, don't add it here.

## Repo structure

```
sos-kit/
├── README.md               # User-facing entry point — MUST reflect reality
├── CLAUDE.md               # This file — for Claude Code contributors
├── SECURITY.md             # Threat model, invariants, trust anchor, rebaseline workflow (P073)
├── .sos-trust-baseline     # Committed sha256 snapshot of auto-exec surfaces (P073 trust gate). Rebaseline: `scripts/trust-gate.sh rebaseline` after any reviewed change.
├── .claude/
│   └── commands/           # Slash command files (P041+: advisory-scan.md, security-review.md)
├── adapters/
│   └── claude/             # Claude adapter boundary (declarative, P076) — README.md + MAPPING.md trỏ artifact → core source ID; physical render P077. KHÔNG runtime binary.
├── Cargo.toml              # Rust workspace root (relocated from bootstrap/sos-rs/ P077f) — members = crates/*
├── Cargo.lock
├── CHANGELOG.md            # Release history — newest entry on top
├── INSTALL.md              # v2 install guide (5-min, with verify steps)
├── LICENSE
├── agents/                 # Orchestrator + role subagent definitions
│   ├── orchestrator.md     # Quản đốc handbook (main-session orchestrator persona, ≤105 lines, session contract — includes deferred-tool loading section)
│   ├── architect.md        # Kiến trúc sư subagent (Read/Write/Glob, no Bash/Grep/Edit)
│   ├── worker.md           # Thợ subagent (full code tools, no vision docs)
│   ├── advisory-watch.md   # Trinh sát specialist subagent (P041 — scoped Bash, queries GHSA)
│   ├── boundary-check.md   # Giám sát specialist subagent (P042 — scoped Bash for git+grep, checks 5 INV)
│   └── README.md           # Agent setup instructions
├── bin/
│   └── sos.sh              # Thin launcher (P077e cutover): 6 heavy subcommands (new/adopt/sync/map/install/tools) exec the Rust `sos` binary; 7 guidance subcommands (init/blueprint/contract/apply/recipe/launch/status) stay Bash until P078
├── configs/                # .ship.toml examples per stack
│   ├── nextjs.toml
│   ├── flask.toml
│   ├── rust.toml
│   └── python.toml
├── crates/                 # Canonical Rust workspace for the `sos` binary (P077e cutover, relocated from bootstrap/sos-rs/ P077f) — sos-cli/sos-core/sos-install/sos-adapter-claude/sos-hooks. See crates/README.md.
├── docs/
│   ├── BACKLOG.md          # Live sprint tracker — surfaced by SessionStart hook
│   ├── COMPARISON.md       # SOS Kit vs gstack
│   ├── DISCOVERIES.md      # Phiếu discovery log — newest on top
│   ├── GENESIS.md          # Kit origin story
│   ├── HANDOFF.md          # 5 inter-layer protocols (insight, routing, phiếu, escalation, discovery)
│   ├── LAYERS.md           # 3-role model (Chủ nhà / Kiến trúc sư / Thợ). Foundation doc.
│   ├── ORCHESTRATION.md    # Full orchestrator spec (state machine, failure modes)
│   ├── PHILOSOPHY.md       # Stable — 6 operational principles + Principle 0, change carefully
│   ├── SETUP.md            # Install guide — MUST match actual binary names + cargo paths
│   └── ticket/             # Phiếu dir — active (root) + done/ archive. Canonical: .docs-gate.toml ticket_dir
├── hooks/
│   └── pre-commit          # type-check + docs-gate + BACKLOG + Discovery enforcement
├── integrations/           # CI snippets + uptime monitoring
│   ├── github-actions/     # canary.yml snippet
│   └── jarvis/             # uptime_monitor.py
├── phieu/                  # Ticket workflow backbone
│   ├── README.md           # Setup + how to use phiếu workflow
│   ├── TICKET_TEMPLATE.md  # Phiếu format (header, Task 0, tasks, nghiệm thu)
│   ├── AUDIT_PROTOCOL.md   # RRI-T-lite periodic audit protocol
│   ├── DISCOVERY_PROTOCOL.md  # Thợ → Kiến trúc sư feedback loop + mismatch classification
│   ├── GENESIS_TEMPLATE.md # P000 genesis phiếu skeleton
│   ├── LAUNCH_CHECKLIST.md # Pre-launch gate checklist
│   ├── RELAY_PROTOCOL.md   # Chủ nhà's courier workflow (Thợ ↔ Kiến trúc sư cross-session)
│   ├── VISION_TEMPLATES/   # Day-1 skeletons — Chủ nhà copies + fills (PROJECT, SOUL, CHARACTER, ...)
│   └── phieu.sh            # Shell function: phieu / phieu-init / phieu-done / phieu-list  (phiếu files live in docs/ticket/)
├── recipes/                # DNA snippets — patterns /apply consumes
│   ├── README.md
│   ├── _TEMPLATE.md        # New recipe skeleton
│   ├── ai/
│   │   └── multi-model-fallback.md
│   └── payment/
│       └── payos-vn.md
├── scripts/                # SessionStart + PreToolUse hooks + security gate
│   ├── architect-guard.sh  # PreToolUse hook — block code reads when architect active
│   ├── block-env-commit.sh    # pre-commit [7/7] — block .env* secret-file commits (allow .env.example); git-level backstop to P046 PreToolUse guard
│   ├── block-env-edit.sh   # PreToolUse hook — block .env edits
│   ├── idea-smell.sh       # UserPromptSubmit hook — regex idea-smell in Sếp message → inject /idea reminder (skills dogfood 2026-06-11)
│   ├── block-unsafe-merge.sh  # PreToolUse hook — B+3 fail-closed shim → `claude-hooks block-unsafe-merge` binary (gates `gh pr merge <N>` without security APPROVE; binary absent = BLOCK LOUD) [P064]
│   ├── lane-check-contract.sh  # pre-commit [3/8] sub-check 3f — OA-01 lane-field contract guard (runs `doctor lane-check` on phiếu/TICKET_TEMPLATE.md when staged; degraded warn-skip when `doctor` absent) [P082]
│   ├── no-code-on-default.sh  # pre-commit [6/8] — block product code committed on default branch (force feature branch; agent-agnostic)
│   ├── security-gate.sh, check-*.py, parsers/  # commit-time security gate + advisory lockfile parsers
│   ├── session-start-banner.sh  # SessionStart hook — show BACKLOG on session open
│   └── trust-gate.sh          # pre-commit [8/8] — auto-exec surface baseline-diff + hidden-unicode scan (P073, port thanhtra v1.2). Baseline committed to .sos-trust-baseline; rebaseline after reviewed change: `scripts/trust-gate.sh rebaseline`
│   # (.claude/agents/ is symlinked to agents/ — no sync script; see agents/README.md)
├── skills/                 # 5 LIVING skills — each declares mechanical `caller:` (no caller = attic)
│   ├── idea/SKILL.md       # Chủ nhà — intake → BACKLOG. Caller: UserPromptSubmit idea-smell hook
│   ├── retro/SKILL.md      # Thợ — weekly velocity/hotspot. Caller: weekly cron (advisory-cron, opt-in)
│   ├── init/SKILL.md       # Chủ nhà — 0→1 vision capture. Caller: sos init CLI (⚠ name-collides built-in /init)
│   ├── apply/SKILL.md      # Thợ — apply 1 recipe. Caller: sos apply CLI
│   ├── forge/SKILL.md      # Kiến trúc sư — write new recipe. Caller: sos recipe new CLI
│   └── attic/              # 8 PARKED (plan verify decide route insight qa review ship) — see attic/README.md
└── templates/              # Chủ nhà-ready starters
    ├── BACKLOG_template.md  # BACKLOG.md skeleton (Active / Next / Open / Park)
    ├── advisory-inbox.md    # Empty queue template for security advisories (P041)
    ├── INVARIANTS-template.md  # 5-INV skeleton + user-added section (P042)
    └── claude-settings.local.json  # Pre-approved marker Bash ops template
```

## Common tasks

### Edit a skill (`skills/<name>/SKILL.md`)
1. Change the markdown
2. Verify the skill still belongs to exactly ONE layer (Chủ nhà / Kiến trúc sư / Thợ). If it spans layers, split it into two.
3. Test: `cp -r skills/<name> ~/.claude/skills/<name>` in a real project, then invoke `/<name>` in Claude Code
4. If the skill's role or trigger changed, update the row in `README.md` "Claude Code Skills" table AND the table in `docs/LAYERS.md`

### Edit phiếu template or shell function (`phieu/`)
1. If `TICKET_TEMPLATE.md` changes, update `docs/HANDOFF.md` section "Handoff 2 — Kiến trúc sư → Thợ" which references the required sections
2. If `phieu.sh` changes function behavior (not just bug fix), update `phieu/README.md` + any skill that invokes it
3. Test: `phieu-init ~/some-test-repo` + create a phiếu end-to-end

### Edit vision doc templates (`phieu/VISION_TEMPLATES/`)
1. Keep templates generic — no project-specific wording. Placeholders use `<angle brackets>`.
2. If you add/remove a section in a template, update `/insight` skill's "Target section" list in `skills/insight/SKILL.md`.
3. Also reflect the change in `docs/HANDOFF.md` Handoff 0 section.

### Edit RELAY_PROTOCOL.md
1. If relay format changes (e.g. escalation fields), update the example session in `skills/decide/SKILL.md` (Worker-side escalation format must match).
2. Also update `docs/HANDOFF.md` Handoff 3 format.

### Add a new layer-specific skill
1. Decide layer FIRST. If unsure, read `docs/LAYERS.md` "Which layer am I in right now?"
2. Create `skills/<name>/SKILL.md`
3. Add to `README.md` skill table under the right layer section
4. Add to `docs/LAYERS.md` skills map
5. If the skill introduces a new handoff, document it in `docs/HANDOFF.md`

### Add a new stack config (`configs/<stack>.toml`)
1. Create the `.toml` template
2. Add a per-stack subsection in `docs/SETUP.md`
3. Add an expandable example in `README.md` "Example configs"

### Edit orchestrator behavior (`agents/orchestrator.md` + `docs/ORCHESTRATION.md`)
1. `agents/orchestrator.md` is the condensed Quản đốc handbook (~148 lines after v2.2 cụm A additions — lane budget + rubric inject + sensor arm) — system-prompt contract for the main session in every sos-kit project. Keep terse + imperative.
2. `docs/ORCHESTRATION.md` is the full spec (state machine, failure modes, concrete example session). When changing state machine logic, update BOTH.
3. If you add a new orchestrator hard rule, mirror it as a one-liner in `agents/orchestrator.md` "Hard rules" section AND a fuller entry in `docs/ORCHESTRATION.md` "Hard rules".
4. SessionStart banner (`scripts/session-start-banner.sh`) references both files — verify the banner still surfaces them after edit.

### Edit Workflow doctrine (`docs/WORKFLOW_V2.2.md`)
1. **PRIMARY doctrine — single-source-of-truth.** Agent handbooks (`agents/*.md`) reference v2.2; conflict → v2.2 wins.
2. Doctrine changes go through **retro process** (see `docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md` for 7-round forge precedent). KHÔNG edit doctrine ad-hoc trong production session.
3. Round flow: pilot end-to-end → retro CHẨN ĐOÁN → forge through multi-reviewer rounds → ĐƠN THUỐC v2.X → hạ vào sos-kit template → next pilot.
4. New retro file naming: `WORKFLOW_V<N>_RETRO_<pilot-name>.md` (e.g., `WORKFLOW_V2.3_RETRO_python-pilot.md`). KHÔNG edit closed retro.
5. **3 luật cứng** (v2.2 §0.1) — every doctrine change must check:
   - Mỗi fix gắn cờ `[gate]` / `[hook]` / `[guidance]` — KHÔNG prose
   - Một bệnh, một cơ chế rẻ nhất bắt 80% — cấm 3 tầng cho 1 bệnh
   - Mechanical mới gate, judgment giữ guidance

### Edit docs
- `README.md` — any tool/skill/integration table MUST match actual folders and binaries. Contributor onboarding breaks if they drift.
- `docs/PHILOSOPHY.md` — stable. Don't add principles without strong justification (Sếp ratification = the bar; #7 adopt-hiểu-repo added 2026-06-11). The 7 operational principles (plus Principle 0 = accountability) are load-bearing.
- `docs/SETUP.md` — must match real binary names and `cargo install` instructions.
- `docs/ORCHESTRATION.md` — orchestrator full spec; condensed handbook at `agents/orchestrator.md` mirrors hard rules. Edit both together.

### Add a new integration (`integrations/<name>/`)
1. Create a folder with a README inside explaining the purpose + setup
2. Update root `README.md` "Integrations" table
3. No secrets in committed files (tokens, webhook URLs) — use env var placeholders

## Rules

> Each rule is flagged `[mechanical]` (a cheap automated gate can/does enforce it soundly) or `[judgment]` (relies on contributor discretion — stays guidance, no gate). Per `docs/WORKFLOW_V2.2.md` §0.1: *mechanical → gate; judgment → guidance.* These tags are a doc-convention (executable-contract clarity, like a type annotation); a `doc-lint` that machine-verifies the tags is deliberately NOT built (over-build — Q-D3/WORKFLOW_V2.3 retro).

1. `[judgment]` **The `sos` CLI runtime source lives here; sister-tool source does not.** Since P077e this repo IS the canonical Rust workspace for the `sos` binary (`Cargo.toml` + `crates/` at repo-root, relocated from `bootstrap/sos-rs/` in P077f). Sister CLIs (`ship`, `docs-gate`, `guard`, `vps`) still belong in their own repos — do not vendor their source here. Beyond the `sos` workspace, this repo stays documentation, templates, and skill markdown. `phieu/phieu.sh` and `bin/sos.sh` are Bash exceptions (a sourced shell function + the thin launcher) doing only git/file ops + dispatch.
2. `[judgment]` **Every new file must justify its existence.** No `TODO.md`, no placeholder directories, no "might use later" stubs.
3. `[mechanical]` **No hardcoded personal paths.** Replace `/Users/<name>/...` with `~/` or a generic example before committing. _(Greppable: `/Users/` or `/home/<user>/` — candidate for a pre-commit grep gate.)_
4. `[judgment]` **README is the single source of truth.** If a tool is listed in `README.md` but not in `docs/SETUP.md`, that's a bug. Fix the gap in both places. Same for `docs/LAYERS.md` skill table.
5. `[judgment]` **Skills are for repeated workflows, not one-off tasks.** If a skill only applies to one project, keep it in that project's `.claude/skills/`, not here.
   **+ Caller law (Sếp-ratified 2026-06-11):** a skill enters `skills/` ONLY with a declared mechanical `caller:` in frontmatter (hook / cron / CLI / gate / handbook-contract). No caller = `skills/attic/`. Evidence: tarot carried all 13 registered for months — 0 invocations ("skill đẹp mà không dùng = không tác dụng").
6. `[judgment]` **One skill, one layer, one responsibility.** If you're tempted to make a skill that "routes AND plans" or "plans AND implements," stop — split it. Layer leaks are anti-pattern #1.
7. `[judgment]` **Handoffs stay formatted.** If you're tempted to add a new inter-layer handoff ("Architect pings Worker directly on Slack"), document the format in `docs/HANDOFF.md` first. Freestyle handoffs = context loss.
8. `[mechanical+judgment]` **DOCS GATE Tầng 1 — code change BẮT BUỘC update relevant doc.** Universal rule (added 2026-05-28 post doc-rotate pilot setup). Trigger: change function signature / constant / data flow / API / schema / surface boundary / prompt / security pattern → BẮT BUỘC update relevant doc TRƯỚC commit. Missing = phiếu CHƯA XONG. **Security boundary touch → AUTO Tầng 1** (KHÔNG mark Tầng 2). Tầng 2 (cosmetic, local var) tùy. See "DOCS GATE Tầng 1 mapping" section below for per-surface table. **Knowledge durability:** Durable doctrine → `CLAUDE.md` / `agents/*.md` / `docs/WORKFLOW_V2.X.md` (no rotate). Operational evidence → `docs/DISCOVERIES.md` (rotate ≥ 1000 lines via `doc-rotate` tool pilot vòng 2).
9. `[judgment]` **Contract-surface doc edit → nên qua 1 vòng CHALLENGE dù nhỏ.** Editing a *contract surface* — `agents/*.md` (role envelopes/handbooks), `docs/ORCHESTRATION.md`, `docs/LAYERS.md` access matrix, `docs/HANDOFF.md` formats, `phieu/TICKET_TEMPLATE.md` — changes how other agents behave, so it deserves a second eye even when the diff is 2 lines. The orchestrator's default is to self-edit these solo (no forcing function — the gates only block *code* on default, never *doctrine*), which is the `model-self-invoke = coin-flip` gap at the process layer ([[feedback_enforce_via_mechanism_not_memory]]). **Guidance, NOT a gate** (§0.1: judgment → guidance — a hook that diff-detects "contract surface" would over-block the constant legit small edits): when you (Quản đốc or a contributor) edit one of these, spawn ONE Worker CHALLENGE pass (or ask Chủ nhà to eyeball) before commit — unless Chủ nhà's direct request IS the approval (then note it in the commit/Discovery). Curation (BACKLOG capture, CHANGELOG, findings, DISCOVERIES) is correctly single-role — do NOT phiếu those (that's the completeness-bias this rule's own §0.1 caveat guards against).

## DOCS GATE Tầng 1 mapping (sos-kit specific)

Per Rule #8 above — when contributor edits these, BẮT BUỘC update target doc(s).

| Code/config change | Target doc(s) | Why |
|---|---|---|
| `agents/architect.md` envelope (tools/role) | `docs/LAYERS.md` access matrix + `docs/HANDOFF.md` Handoff 2 | Subagent contract surface |
| `agents/worker.md` envelope | `docs/LAYERS.md` + `docs/HANDOFF.md` Handoff 2-3 | Same |
| `agents/orchestrator.md` hard rule add | `docs/ORCHESTRATION.md` Hard rules + `scripts/session-start-banner.sh` if banner refs rule | Orchestrator spec mirror |
| `agents/boundary-check.md` or `advisory-watch.md` | `docs/LAYERS.md` specialist subagents subsection + `README.md` row | Specialist agent inventory |
| `phieu/TICKET_TEMPLATE.md` format | `phieu/README.md` + `docs/HANDOFF.md` Handoff 2 | Format contract Architect ↔ Worker |
| `phieu/phieu.sh` function behavior | `phieu/README.md` + `docs/SETUP.md` install step | User-facing CLI |
| `phieu/VISION_TEMPLATES/*.md` section change | `skills/insight/SKILL.md` "Target section" list + `docs/HANDOFF.md` Handoff 0 | Insight skill template binding |
| `hooks/pre-commit` SECTION add/remove (⚠️ this changes the **phase COUNT** `[N/M]`) | `CLAUDE.md` "Hook chain" + `docs/SETUP.md` — **update the M everywhere it appears** (every `[N/M]` label, the `# Runs in order` header list, AND any prose "Phase 5"/"5 phases" in CLAUDE.md + ARCHITECTURE) | Hook chain integrity. P062: phase-count drifted silently 3 phiếu (doc said "Phase 5" while hook was `[8/8]`) because the trigger only flagged section add/remove, not the count it implies — Architect reads stale doc → wrong phase in next phiếu. |
| `scripts/no-code-on-default.sh` add/remove | `CLAUDE.md` scripts list + `docs/SETUP.md` hook section | Gate inventory (P050) |
| `scripts/block-env-commit.sh` add/remove | `CLAUDE.md` scripts list + `docs/SETUP.md` hook section | Gate inventory (P052) |
| `scripts/trust-gate.sh` add/remove OR `.sos-trust-baseline` format change | `CLAUDE.md` scripts list + `SECURITY.md` + `docs/SETUP.md` rebaseline workflow | Auto-exec integrity surface (P073). `.sos-trust-baseline` = committed sha256 snapshot; rebaseline via `scripts/trust-gate.sh rebaseline` after any reviewed auto-exec surface change. |
| `scripts/security-gate.sh` INV add/remove OR runner-swap | `templates/INVARIANTS-template.md` + `docs/SETUP.md` Security pipeline | Security surface contract. (2026-06-11: gate is now `inv-gate` binary-first w/ python3 fallback — kills the python dep when the binary is present; install.sh ships inv-gate.) |
| `scripts/check-*.py` pattern add/remove | `templates/INVARIANTS-template.md` + `docs/HANDOFF.md` Handoff 5 | Mechanical gate inventory |
| `scripts/parsers/*.py` add (new lock format support) | `agents/advisory-watch.md` Bước 1 stack-parse list + `docs/SETUP.md` | Trinh sát stack coverage |
| `recipes/<cat>/<name>.md` add/remove | `README.md` recipes table + `recipes/README.md` index | Recipe catalog |
| `configs/<stack>.toml` add | `docs/SETUP.md` per-stack section + `README.md` "Example configs" | Stack support catalog |
| `templates/*.md` add | `README.md` templates section | Template inventory |
| `skills/<name>/SKILL.md` role/trigger change | `README.md` skill table + `docs/LAYERS.md` skill map | Skill ownership |
| `.docs-gate.toml` rule change | `CLAUDE.md` rule references + `hooks/pre-commit` if section logic mirrors | Pre-commit chain mutation |
| `.mcp.json` server add/remove | `docs/SETUP.md` MCP section (if exists) | MCP inventory |
| CHANGELOG version bump (e.g. `## [0.21.0]`) | `Cargo.toml`/`pyproject.toml` `[package] version` **must sync** (else `--version` prints the stale scaffold value) | F13 (doc-rotate dogfood): `Cargo.toml` sat at `0.0.0` while CHANGELOG reached `[0.20.0]` — ~20 versions of silent drift, invisible until something reads `CARGO_PKG_VERSION`. Mechanizable: `grep '^version' Cargo.toml` vs first `## [` in CHANGELOG. |
| **Language port / module rename / file move** (e.g. `.py` → `.rs`) | `docs/AGENT_MAP.yaml` `edit:`/surface paths **must update** → re-run `doctor validate-map` (it path-checks AGENT_MAP) | F10 (doc-rotate dogfood): RP07b retired Python but AGENT_MAP kept stale `.py` paths → Architect (docs-only) would spec non-existent files. AGENT_MAP is a separate doc category not caught by the other rows; `validate-map` is the mechanical net but must be RUN. |
| `docs/WORKFLOW_V2.X.md` | ⛔ FORBIDDEN ad-hoc edit — must go through retro process (see "Edit Workflow doctrine" Common task above) | Doctrine versioning |

**Enforcement:**
- **Mechanical** (pre-commit hook): `docs-gate --all` enforces changelog freshness + architecture file exists + ticket dir + changelog staged.
- **Judgment** (contributor discipline): Worker BẮT BUỘC ghi trong Discovery Report "Tầng 1 docs updated: <list>" hoặc "Tầng 1 N/A (cosmetic only)". Reviewer (Architect Turn 2 or Sếp) challenges if diff touches trigger row but Discovery claims Tầng 2.

**Why this rule exists:** Without enforcement, Architect drafts next phiếu reading stale doc → wrong assumption → cascading failure. Same root cause as Sub-mech D (persistence lifecycle gap) — doctrine ship ≠ doctrine survive. Per `docs/WORKFLOW_V2.2.md §7`, durable knowledge must live in non-rotating location with hook enforcement where possible.

## Related repos (maintained separately)

| Repo | Local path | Role |
|---|---|---|
| `ship` | `~/ship` | Release pipeline CLI — test, commit, PR, deploy, canary |
| `docs-gate` | `~/docs-gate` | Pre-commit documentation enforcement |
| `guard` | `~/guard` | Pre-deploy infrastructure gate — schema drift, env sync |
| `vps` | `~/vps` | Production ops CLI — logs, status, restart, metrics |

Changes to these belong in their respective repos, not here. SOS Kit only references and documents them.

## Language

Public-facing docs (`README.md`, `docs/SETUP.md`, `docs/PHILOSOPHY.md`, `SKILL.md` files) are English — this repo may be published open-source.

Internal conversations with the maintainer (Denny / Nguyen) are in Vietnamese; see the maintainer's personal tarot/CLAUDE.md for that convention. This `CLAUDE.md` stays in English so external contributors can read it.

**Role names (tên vai) vs address (xưng hô) — keep separate, never conflate.** Role names — `Chủ nhà`, `Quản đốc`, `Kiến trúc sư`, `Thợ`, `Trinh sát`, `Giám sát` — are fixed doctrine constants and appear **identically in every handbook** (`agents/*.md`), public and personal. They are third-person role references ("escalate to Chủ nhà", "Chủ nhà approves the phiếu") — never forms of address. The maintainer's address register — agents call the human **"Sếp"/"anh"** and refer to themselves as **"em"** — is a *separate conversational layer* that lives in live chat and UI (e.g. the SessionStart banner's "Sếp's project"), NOT in handbook role-references. Do **not** rename a role to an address term: the old `sed 's/Chủ nhà/Sếp/g'` swap (with a per-repo `.claude/agents/` copy + `sync-personal-agents.sh`) conflated the two layers, rotted into drift, and was removed in favor of symlinking `.claude/agents/ → agents/` (see `agents/README.md`).

## Deferred-tool loading (Claude Code session start)

Claude Code's `AskUserQuestion`, `TaskCreate`, `TaskUpdate`, `TaskList` are **deferred** tools — they don't auto-load in fresh sessions. Direct invocation fails with `InputValidationError: tool not loaded`.

The main-session orchestrator (Quản đốc) and Architect subagent both rely on these:
- `AskUserQuestion` — APPROVAL_GATE + FORCE_ESCALATION (orchestrator), multi-choice escalation (Architect)
- `TaskCreate` / `TaskUpdate` / `TaskList` — sprint tracking visibility (both)

**On every new Claude Code session in a sos-kit project**, the first turn MUST invoke `ToolSearch` to register them:

```
ToolSearch query="select:AskUserQuestion,TaskCreate,TaskUpdate,TaskList"
```

This is documented as a mandatory orchestrator session-start step in `agents/orchestrator.md` "Deferred-tool loading" section. Contributors editing the orchestrator handbook must preserve this instruction — removing it causes silent state-machine failure on later turns (approval gate cannot fire = phiếu cannot be approved = workflow halts).

## Maintainer-only conventions

The maintainer (Denny) uses:
- **Phiếu workflow** — ticket IDs `P<NNN>-<slug>`, shell function `phieu`, worktree per ticket. Lives in his `~/.zshrc`, not in this repo (yet).
- **Vietnamese communication with Claude** — em/anh xưng hô, Vietnamese in chat + commits in English.

If you're the maintainer talking to Claude, that context applies. If you're an external contributor, follow the repo's English + PR-based flow as described in `README.md`.
