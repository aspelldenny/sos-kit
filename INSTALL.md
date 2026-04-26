# sos-kit v2 — Install Guide

> Cài v2 vào project hiện có (đã có git, đã có docs/ basic) hoặc project trống.
> v2 = 3-role envelope (Chủ nhà / Kiến trúc sư / Thợ) + BACKLOG forcing function + hooks.

## Prerequisites

- Project là git repo
- Bash (macOS/Linux/WSL/Git Bash trên Windows)
- Claude Code v2.1+ (để hỗ trợ subagent + SessionStart hook)
- (Optional nhưng recommended) sos-kit v1 Rust tools đã cài: `ship`, `docs-gate`, `vps`

## What gets installed

```
<your-project>/
├── .claude/
│   ├── agents/
│   │   ├── architect.md          ← Kiến trúc sư subagent (Read/Write/Glob only)
│   │   └── worker.md              ← Thợ subagent (full code tools, no vision)
│   ├── skills/
│   │   └── idea/SKILL.md          ← /idea intake skill
│   └── settings.json              ← Hooks: SessionStart banner + PreToolUse architect-guard
├── hooks/
│   └── pre-commit                 ← Git pre-commit hook (NEW in v2: docs-gate + Discovery enforcement)
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

## Install steps (5 phút)

### 1. Copy v2 files vào project

Giả sử sos-kit v2 đã clone tại `~/sos-kit` (clone từ aspelldenny/sos-kit khi v2 được merge).

```bash
cd ~/your-project

# Agents (canonical, English-neutral, "Chủ nhà" voice)
mkdir -p .claude/agents
cp ~/sos-kit/agents/architect.md .claude/agents/
cp ~/sos-kit/agents/worker.md .claude/agents/

# Skills
mkdir -p .claude/skills/idea
cp ~/sos-kit/.claude/skills/idea/SKILL.md .claude/skills/idea/

# Hooks
mkdir -p scripts
cp ~/sos-kit/scripts/architect-guard.sh scripts/
cp ~/sos-kit/scripts/session-start-banner.sh scripts/
chmod +x scripts/architect-guard.sh scripts/session-start-banner.sh

# Settings (MERGE if .claude/settings.json already exists — see Step 2)
cp ~/sos-kit/.claude/settings.json .claude/settings.json
```

### 2. Merge settings.json (nếu project đã có)

Nếu `.claude/settings.json` đã có, **merge** thay vì overwrite. Add hai hook block sau vào field `hooks`:

```json
{
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

Nếu đã có `PreToolUse` hooks khác → merge cùng matcher hoặc thêm entry mới.

### 3. Bootstrap docs (nếu thiếu)

```bash
# BACKLOG.md (mới — required for /idea skill và Architect Rule 0)
cp ~/sos-kit/templates/BACKLOG_template.md docs/BACKLOG.md
# Edit docs/BACKLOG.md: điền tên project, sprint hiện tại, tasks

# Vision docs (từ v1, nếu thiếu)
[ ! -f docs/PROJECT.md ] && cp ~/sos-kit/phieu/VISION_TEMPLATES/PROJECT_template.md docs/PROJECT.md
[ ! -f docs/SOUL.md ] && cp ~/sos-kit/phieu/VISION_TEMPLATES/SOUL_template.md docs/SOUL.md

# Discovery log (từ v1, nếu thiếu)
[ ! -f docs/DISCOVERIES.md ] && echo '# Discoveries Log' > docs/DISCOVERIES.md

# Ticket template (từ v1, nếu thiếu)
mkdir -p docs/ticket
[ ! -f docs/ticket/TICKET_TEMPLATE.md ] && cp ~/sos-kit/phieu/TICKET_TEMPLATE.md docs/ticket/TICKET_TEMPLATE.md
```

### 3.5. Setup pre-commit hook (CRITICAL — enforces docs gate)

**Without this, docs có thể commit lỗi thời → Architect viết phiếu sai.**

```bash
# Copy hook script
mkdir -p hooks
cp ~/sos-kit/hooks/pre-commit hooks/pre-commit
chmod +x hooks/pre-commit

# Tell git to use it
git config core.hooksPath hooks
```

**Recommend:** đổi `.ship.toml` `[docs_gate] blocking = true` để `ship` cũng enforce.

**Hook checks (3 layers):**
1. Stack-aware type/syntax check (cargo check / pnpm type-check / python ast.parse)
2. `docs-gate` Rust binary v1 (CHANGELOG + ARCHITECTURE 9 sections)
3. **v2 checks**:
   - `docs/BACKLOG.md` exists + Active sprint không trống
   - **New phiếu file (`docs/ticket/P*-*.md`) staged → REQUIRE matching Discovery entry**
   - Code + phiếu changed → warn nếu thiếu DISCOVERIES + CHANGELOG

**Bypass khi cần** (rare): `git commit --no-verify`. NOT recommended cho normal flow.

### 4. Update CLAUDE.md (project root)

Thêm section sau vào `CLAUDE.md` của project (nếu chưa có sos-kit v1 mindset):

```markdown
## Sos-kit v2 — 3-role envelope

Đây là project dùng sos-kit v2 framework. 3 role:
- **Chủ nhà** (con người) — vision, priorities, approve, nghiệm thu
- **Kiến trúc sư** (subagent `architect`) — đọc docs, viết phiếu, KHÔNG đọc code
- **Thợ** (subagent `worker`) — execute phiếu, full code access, KHÔNG đọc vision

**Forcing functions:**
- `docs/BACKLOG.md` — Architect chỉ viết phiếu cho item ở "Active sprint"
- `/idea` skill — intake idea mới, route vào BACKLOG đúng section
- Hook `architect-guard.sh` — chặn cứng .py/.rs/.ts read khi marker `.claude/.architect-active`
- Hook `session-start-banner.sh` — show BACKLOG mỗi lần mở Claude Code

**Workflow (v2.1 — auto-debate):**
1. Mở Claude Code → SessionStart hook show BACKLOG → Chủ nhà pick item
2. **Chủ nhà đưa 1 câu brief** (e.g., "build feature X cho item Y ở Active sprint")
3. Main session orchestrate (xem `docs/ORCHESTRATION.md`):
   a. Spawn architect (DRAFT) → phiếu V1 with Debate Log section
   b. Spawn worker (CHALLENGE) → verify Task 0 + đọc code thật → Debate Log Turn 1
   c. (nếu có objection) Spawn architect (RESPOND) → phiếu V2
   d. Loop tới consensus hoặc max 3 turns
4. **Chủ nhà approval gate** — orchestrator AskUserQuestion show phiếu cuối + Debate Log → Chủ nhà duyệt
5. Spawn worker (EXECUTE) → Task 0 → code → test → Discovery → commit
6. Chủ nhà nghiệm thu, deploy
```

### 5. Verify install

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
# → Try: /idea Em test — should invoke idea skill with AskUserQuestion

# Verify v2.1 debate flow (smoke test, see "First phiếu" section below)
grep -A2 "Debate Log" .claude/agents/worker.md | head -5
# → expect: "CHALLENGE" mode trigger phrase listed
```

## First phiếu (smoke test)

1. Edit `docs/BACKLOG.md`, add 1 item vào Active sprint:
   ```
   - [ ] **[NEW]** Test sos-kit v2 install — viết phiếu chore nhỏ để verify flow
   ```

2. Trong Claude Code:
   ```
   Spawn architect subagent để viết phiếu cho item "Test sos-kit v2 install" ở Active sprint.
   ```

3. Architect đọc docs, viết phiếu tại `docs/ticket/P001-test-install.md` với Task 0 anchors.

4. Sau khi duyệt:
   ```
   Spawn worker để execute phiếu P001-test-install.md.
   ```

5. Worker chạy Task 0, code, test, Discovery, commit.

## Common gotchas

| Gotcha | Fix |
|--------|-----|
| `bash: scripts/architect-guard.sh: command not found` (Windows native) | Cài Git Bash hoặc WSL — script là bash, không PowerShell |
| `Agent type 'architect' not found` | Restart Claude Code (`/exit` + `claude`) — agents load lúc start |
| `architect-guard.sh` không block | Check `.claude/.architect-active` có tồn tại không (`ls -la .claude/`) |
| Hook block worker khi spawn | Worker spawn cần marker NOT exist — `rm .claude/.architect-active` trước |
| `/idea` slash không nhận | Skill load lúc Claude Code start — restart |
| BACKLOG.md không tồn tại | Bootstrap: `cp ~/sos-kit/templates/BACKLOG_template.md docs/BACKLOG.md` |

## Uninstall

```bash
rm -rf .claude/agents .claude/skills/idea
rm scripts/architect-guard.sh scripts/session-start-banner.sh
# Edit .claude/settings.json: remove SessionStart and architect-guard PreToolUse hooks
# (Keep docs/BACKLOG.md if you want to keep the work tracking; sos-kit v1 doesn't need it)
```

## What's NOT included in v2

Sos-kit v2 governs **what to build and how to verify**. It does NOT govern:
- SSH/VPS authentication (use your own key management)
- Multi-machine sync (use git as you would normally)
- Server-side state (production ops are `vps` CLI's job, separate kit)
- Time-based planning (sos-kit is wave-based, not sprint-by-time)

Keep these out of sos-kit; mix at your own infrastructure level.

---

*v2 install path. After install, run a smoke phiếu to verify all 4 component (BACKLOG + /idea + Architect Rule 0 + Worker envelope) work end-to-end.*
