# SOS Kit v2 — Install Guide

> Install v2 into an existing project (with git + basic docs/) or a fresh project.
> v2 = 3-role envelope (Chủ nhà / Kiến trúc sư / Thợ) as Claude Code subagents + BACKLOG forcing function + docs gate hooks.

## Prerequisites

- Project is a git repo
- Bash (macOS/Linux/WSL/Git Bash on Windows)
- Claude Code v2.1+ (subagent + SessionStart hook support)
- (Optional, recommended) sos-kit v1 Rust tools installed: `ship`, `docs-gate`, `vps`

## What gets installed

```
<your-project>/
├── .claude/
│   ├── agents/
│   │   ├── architect.md          ← Kiến trúc sư subagent (Read/Write/Glob only)
│   │   └── worker.md              ← Thợ subagent (full code tools, no vision docs)
│   ├── skills/
│   │   └── idea/SKILL.md          ← /idea intake skill
│   └── settings.json              ← Hooks: SessionStart banner + PreToolUse architect-guard
├── hooks/
│   └── pre-commit                 ← Git pre-commit hook (docs-gate + Discovery enforcement)
├── scripts/
│   ├── architect-guard.sh         ← PreToolUse hook (block code reads when architect mode)
│   └── session-start-banner.sh    ← SessionStart hook (show backlog at session start)
└── docs/
    ├── BACKLOG.md                 ← Live work-in-progress list (NEW in v2)
    ├── PROJECT.md                 ← Vision (already in v1)
    ├── SOUL.md                    ← Why (already in v1)
    ├── DISCOVERIES.md              ← Worker → Architect feedback (already in v1)
    └── ticket/
        └── TICKET_TEMPLATE.md      ← Phiếu format (already in v1)
```

## Install steps (~5 minutes)

### 1. Copy v2 files into your project

Assuming sos-kit is cloned at `~/sos-kit`:

```bash
cd ~/your-project

# Agents (Claude Code subagents)
mkdir -p .claude/agents
cp ~/sos-kit/agents/architect.md .claude/agents/
cp ~/sos-kit/agents/worker.md .claude/agents/

# Skills
mkdir -p .claude/skills/idea
cp ~/sos-kit/skills/idea/SKILL.md .claude/skills/idea/

# Hook scripts
mkdir -p scripts
cp ~/sos-kit/scripts/architect-guard.sh scripts/
cp ~/sos-kit/scripts/session-start-banner.sh scripts/
chmod +x scripts/architect-guard.sh scripts/session-start-banner.sh
```

### 2. Setup Claude Code hooks (settings.json)

Create or merge into `.claude/settings.json`:

```json
{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          { "type": "command", "command": "bash scripts/session-start-banner.sh" }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Read|Glob",
        "hooks": [
          { "type": "command", "command": "bash scripts/architect-guard.sh" }
        ]
      }
    ]
  }
}
```

If you already have other PreToolUse hooks → merge under the same matcher or add a new entry.

### 3. Bootstrap docs (if missing)

```bash
# BACKLOG.md (NEW in v2 — required for /idea skill and Architect Rule 0)
cp ~/sos-kit/templates/BACKLOG_template.md docs/BACKLOG.md
# Edit docs/BACKLOG.md: fill in project name, current sprint, tasks

# Vision docs (from v1, if missing)
[ ! -f docs/PROJECT.md ] && cp ~/sos-kit/phieu/VISION_TEMPLATES/PROJECT_template.md docs/PROJECT.md
[ ! -f docs/SOUL.md ] && cp ~/sos-kit/phieu/VISION_TEMPLATES/SOUL_template.md docs/SOUL.md

# Discovery log (from v1, if missing)
[ ! -f docs/DISCOVERIES.md ] && echo '# Discoveries Log' > docs/DISCOVERIES.md

# Ticket template (from v1, if missing)
mkdir -p docs/ticket
[ ! -f docs/ticket/TICKET_TEMPLATE.md ] && cp ~/sos-kit/phieu/TICKET_TEMPLATE.md docs/ticket/TICKET_TEMPLATE.md
```

### 4. Setup pre-commit hook (CRITICAL — enforces docs gate)

**Without this, stale docs can be committed → Architect writes phiếu based on wrong assumptions.**

```bash
# Copy hook script
mkdir -p hooks
cp ~/sos-kit/hooks/pre-commit hooks/pre-commit
chmod +x hooks/pre-commit

# Tell git to use it
git config core.hooksPath hooks
```

**Recommended:** set `[docs_gate] blocking = true` in `.ship.toml` so `ship` also enforces.

**The hook checks (3 layers):**
1. Stack-aware type/syntax check (cargo check / pnpm type-check / python ast.parse)
2. `docs-gate` Rust binary v1 (CHANGELOG + ARCHITECTURE 9 sections)
3. **v2 checks**:
   - `docs/BACKLOG.md` exists + Active sprint not empty
   - **New phiếu file (`docs/ticket/P*-*.md`) staged → REQUIRES matching Discovery entry**
   - Code + phiếu changed → warn if DISCOVERIES + CHANGELOG not staged

**Bypass when needed** (rare): `git commit --no-verify`. NOT recommended for normal flow.

### 5. Update CLAUDE.md (project root)

Add this section to your project's `CLAUDE.md` (if you don't have v1 mindset already):

```markdown
## sos-kit v2 — 3-role envelope

This project uses sos-kit v2. Three roles:
- **Chủ nhà** (human) — vision, priorities, approval, acceptance
- **Kiến trúc sư** (subagent `architect`) — reads docs, writes phiếu, does NOT read code
- **Thợ** (subagent `worker`) — executes phiếu, full code access, does NOT read vision docs

**Forcing functions:**
- `docs/BACKLOG.md` — Architect only writes phiếu for items in "Active sprint"
- `/idea` skill — intakes new ideas, routes them to the right BACKLOG section
- `architect-guard.sh` hook — hard-blocks .py/.rs/.ts reads when marker `.claude/.architect-active` is present
- `session-start-banner.sh` hook — shows BACKLOG every time Claude Code starts

**Workflow:**
1. Open Claude Code → SessionStart shows BACKLOG → Chủ nhà picks an item
2. `Spawn architect subagent to write phiếu for item X` (X must be in Active sprint)
3. Architect writes phiếu → Chủ nhà approves → `Spawn worker to execute`
4. Worker runs Task 0 → codes → tests → Discovery Report → local commit
5. Chủ nhà accepts, deploys
```

### 6. Verify install

```bash
# Test hook script offline
bash scripts/session-start-banner.sh
# → should print BACKLOG Active sprint banner

# Test architect-guard.sh
touch .claude/.architect-active
echo '{"tool_input":{"file_path":"src/main.rs"}}' | bash scripts/architect-guard.sh
# → should exit 2 with "🚫 Architect envelope violation"
echo "exit code: $?"   # → 2
rm .claude/.architect-active

# Restart Claude Code
exit
claude
# → SessionStart banner should appear
# → Try: /agents — should list 'architect' and 'worker'
# → Try: /idea I want to test — should invoke idea skill with AskUserQuestion
```

## First phiếu (smoke test)

1. Edit `docs/BACKLOG.md`, add 1 item to Active sprint:
   ```
   - [ ] **[NEW]** Test sos-kit v2 install — write a small chore phiếu to verify the flow
   ```

2. In Claude Code:
   ```
   Spawn architect subagent to write a phiếu for "Test sos-kit v2 install" in Active sprint.
   ```

3. Architect reads docs, writes phiếu at `docs/ticket/P001-test-install.md` with Task 0 anchors.

4. After approval:
   ```
   Spawn worker to execute phiếu P001-test-install.md.
   ```

5. Worker runs Task 0, codes, tests, writes Discovery, commits.

## Common gotchas

| Gotcha | Fix |
|--------|-----|
| `bash: scripts/architect-guard.sh: command not found` (Windows native) | Install Git Bash or WSL — script is bash, not PowerShell |
| `Agent type 'architect' not found` | Restart Claude Code (`/exit` + `claude`) — agents load at session start |
| `architect-guard.sh` doesn't block | Check `.claude/.architect-active` exists (`ls -la .claude/`) |
| Hook blocks worker on spawn | Worker spawn requires marker NOT present — `rm .claude/.architect-active` first |
| `/idea` slash command not recognized | Skill loads at Claude Code start — restart |
| BACKLOG.md doesn't exist | Bootstrap: `cp ~/sos-kit/templates/BACKLOG_template.md docs/BACKLOG.md` |

## Uninstall

```bash
rm -rf .claude/agents .claude/skills/idea
rm scripts/architect-guard.sh scripts/session-start-banner.sh
# Edit .claude/settings.json: remove SessionStart and architect-guard PreToolUse hooks
# (Keep docs/BACKLOG.md if you want to keep work tracking; v1 doesn't need it)
```

## What v2 does NOT cover

sos-kit v2 governs **what to build and how to verify**. It does NOT govern:
- SSH/VPS authentication (use your own key management)
- Multi-machine sync (use git as you would normally)
- Server-side state (production ops are `vps` CLI's job, separate kit)
- Time-based planning (sos-kit is wave-based, not deadline-driven)

Keep these out of sos-kit; mix at your own infrastructure level.

---

*v2 install path. After install, run a smoke phiếu to verify all 4 components (BACKLOG + /idea + Architect Rule 0 + Worker envelope) work end-to-end.*
