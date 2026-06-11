# sos-kit — Install Guide

> Cài sos-kit vào project (mới hoặc hiện có).
> = 3-role envelope + Orchestrator + Workflow v2.2+ doctrine (lane budgets, oracle-first, AGENT_MAP, hooks-not-prose, watchlist sensors).
>
> **Doctrine source:** `~/sos-kit/docs/WORKFLOW_V2.2.md` — read once per project setup. Retro trace: `~/sos-kit/docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md` (CLOSED 7-round forge).

## ⚡ The 1-command path (USE THIS)

Pick by repo state — one command does the whole install below (copy + born-wire + validate):

| Your repo is… | Command | What it does |
|---|---|---|
| **Empty / new** | `sos new <dir> --stack <python\|rust\|ts>` | Bootstrap from golden: spine + skeletons + git init + hooks armed + validator |
| **Existing code, no kit** | `sos adopt <dir>` | Retrofit: ADDITIVE + NON-CLOBBER copy → **born-wire** (arm hooks via F09-guarded install-hooks + `sos init security` stack detect + jq-merge `.mcp.json` doctor entry & `settings.local.json` marker perms) → `doctor verify-setup` |
| **Adopted from an OLDER kit** | `sos sync <dir>` | Re-sync spine: take-newer unmodified files (provenance = kit git history), flag customized to `.sos-sync-incoming/` |

**Getting `sos` on PATH first** (one-time):

```bash
git clone https://github.com/aspelldenny/sos-kit ~/sos-kit
ln -s ~/sos-kit/bin/sos.sh ~/.local/bin/sos   # or: echo 'source ~/sos-kit/bin/sos.sh' >> ~/.zshrc
```

After `sos adopt`, the report tells you the 2-3 things only YOU can do (fill `docs/BACKLOG.md` Active sprint, a fresh CHANGELOG entry for your first gated commit, restart Claude Code so agents load). Everything else is wired automatically. **The manual steps below are the REFERENCE for what adopt does under the hood** — read them to understand the pieces, or to hand-merge a file adopt flagged.

## Prerequisites

- Project là git repo
- Bash (macOS/Linux/WSL/Git Bash trên Windows) + `jq` (optional — enables JSON auto-merge during adopt)
- Claude Code v2.1+ (để hỗ trợ subagent + SessionStart hook)
- (Optional nhưng recommended) sos-kit Rust tools đã cài: `ship`, `docs-gate`, `doctor`, `vps`

## What gets installed

```
<your-project>/
├── .claude/
│   ├── agents/
│   │   ├── architect.md          ← Kiến trúc sư subagent (Read/Write/Glob only)
│   │   ├── worker.md              ← Thợ subagent (full code tools, no vision)
│   │   ├── advisory-watch.md      ← Trinh sát specialist (GHSA/CVE scan)
│   │   └── boundary-check.md      ← Giám sát specialist (5-INV boundary review)
│   ├── commands/
│   │   ├── security-review.md     ← /security-review (spawns Giám sát)
│   │   └── advisory-scan.md       ← /advisory-scan (spawns Trinh sát)
│   ├── skills/
│   │   └── idea/SKILL.md          ← /idea intake skill
│   └── settings.json              ← Hooks: SessionStart banner + PreToolUse (architect-guard + block-env-edit + block-unsafe-merge)
├── hooks/
│   └── pre-commit                 ← Git pre-commit hook (NEW in v2: docs-gate + Discovery enforcement)
├── scripts/
│   ├── architect-guard.sh         ← PreToolUse hook (block code reads when architect mode)
│   ├── block-env-edit.sh          ← PreToolUse hook (block .env edits)
│   ├── block-unsafe-merge.sh      ← PreToolUse hook (block force-push / unsafe merge — Giám sát backstop)
│   └── session-start-banner.sh    ← SessionStart hook (show backlog at session start)
└── docs/
    ├── BACKLOG.md                 ← Live work-in-progress list (NEW in v2)
    ├── PROJECT.md                 ← Vision (already in v1)
    ├── SOUL.md                    ← Why (already in v1)
    ├── DISCOVERIES.md              ← Worker → Architect feedback (already in v1)
    └── ticket/
        └── TICKET_TEMPLATE.md      ← Phiếu format (already in v1)
```

## Appendix — manual install steps (reference: what `sos adopt` automates)

> ⚠️ **Prefer `sos adopt` / `sos new` above.** This section exists so you can (a) understand each piece, (b) hand-merge a file adopt flagged to `.sos-adopt-incoming/`, or (c) install on a machine without the kit checkout. Hand-copying is how installs go incomplete — the media-rating collapse traced partly to a skipped security pair.

### 1. Copy v2 files vào project

Giả sử sos-kit v2 đã clone tại `~/sos-kit` (clone từ aspelldenny/sos-kit khi v2 được merge).

```bash
cd ~/your-project

# Agents — ALL 4 spawnable (canonical, English, "Chủ nhà" role-name voice)
# (architect + worker = core workflow; advisory-watch + boundary-check = security
#  specialists. Copy all 4 — skipping the security pair is what left media-rating
#  with no Giám sát = a root cause of its collapse.)
mkdir -p .claude/agents
cp ~/sos-kit/agents/architect.md      .claude/agents/
cp ~/sos-kit/agents/worker.md         .claude/agents/
cp ~/sos-kit/agents/advisory-watch.md .claude/agents/
cp ~/sos-kit/agents/boundary-check.md .claude/agents/

# Commands (so Quản đốc can invoke the con-mắt agents)
mkdir -p .claude/commands
cp ~/sos-kit/.claude/commands/security-review.md .claude/commands/
cp ~/sos-kit/.claude/commands/advisory-scan.md   .claude/commands/

# Skills
mkdir -p .claude/skills/idea
cp ~/sos-kit/.claude/skills/idea/SKILL.md .claude/skills/idea/

# Hooks — ALL PreToolUse guard scripts referenced by settings.json below.
# (block-unsafe-merge = the mechanical backstop for "no merge without Giám sát";
#  block-env-edit = .env protection. Copying settings.json without these = broken
#  hooks pointing at missing scripts.)
mkdir -p scripts
cp ~/sos-kit/scripts/architect-guard.sh     scripts/
cp ~/sos-kit/scripts/block-env-edit.sh      scripts/
cp ~/sos-kit/scripts/block-unsafe-merge.sh  scripts/
cp ~/sos-kit/scripts/session-start-banner.sh scripts/
chmod +x scripts/*.sh

# Settings (MERGE if .claude/settings.json already exists — see Step 2)
cp ~/sos-kit/.claude/settings.json .claude/settings.json

# NOTE: commit-time security gate (security-gate.sh + check-*.py + parsers/) and the
# CVE advisory pipeline need `sos init security` (stack detection → .sos-stack.toml).
# See docs/SETUP.md → Security pipeline. The above = the session-level guards + agents.
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
      },
      {
        "matcher": "Edit|Write",
        "hooks": [
          { "type": "command", "command": "bash scripts/block-env-edit.sh" }
        ]
      },
      {
        "matcher": "Bash",
        "hooks": [
          { "type": "command", "command": "bash scripts/block-unsafe-merge.sh" }
        ]
      }
    ]
  }
}
```

Nếu đã có `PreToolUse` hooks khác → merge cùng matcher hoặc thêm entry mới. **All three matchers must be present** — `block-unsafe-merge` (Bash) is the mechanical backstop for the no-merge-without-Giám-sát invariant; omitting it is how a repo ends up able to merge security changes unreviewed.

### 2.5. Pre-approve marker file Bash ops (skip per-spawn permission prompts) — `sos adopt` jq-merges this automatically

Orchestrator (main session) flips two markers (marker hygiene per `docs/ORCHESTRATION.md` Hard rule #6): `.sos-state/architect-active` before/after spawning Architect (gates `architect-guard.sh`), and `.sos-state/worker-active` before/after spawning Worker (gates `orchestrator-guard.sh`). Without pre-approval, Claude Code prompts on every spawn — defeats v2.1 auto-orchestration.

```bash
# Copy template if .claude/settings.local.json doesn't exist
[ ! -f .claude/settings.local.json ] && cp ~/sos-kit/templates/claude-settings.local.json .claude/settings.local.json
```

Nếu `.claude/settings.local.json` đã có, **merge** thêm 5 entry vào `permissions.allow` array:
- `Bash(mkdir -p .sos-state)`
- `Bash(touch .sos-state/architect-active)`
- `Bash(rm -f .sos-state/architect-active)`
- `Bash(touch .sos-state/worker-active)`
- `Bash(rm -f .sos-state/worker-active)`

`.claude/settings.local.json` là per-user (thường `.gitignore` rồi) — không commit.

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

# v2.2 ADDITIONS:

# Security INVARIANTS (cho rubric inject vào boundary-check — v2.2 §8 canary 2 finding)
mkdir -p docs/security
[ ! -f docs/security/INVARIANTS.md ] && cp ~/sos-kit/templates/INVARIANTS-template.md docs/security/INVARIANTS.md
# Project-specific INV-LOCAL-* live trong file này. Quản đốc inject vào spawn prompt khi
# trigger /security-review. Subagent KHÔNG tự grep.

# AGENT_MAP (CHỈ nếu repo > 10 docs OR docs > 500KB total — v2.2 §4)
# Skip cho repo nhỏ — grep convention đủ.
if [ -d docs ] && [ $(find docs -name "*.md" | wc -l) -gt 10 ]; then
  [ ! -f docs/AGENT_MAP.yaml ] && cp ~/sos-kit/configs/AGENT_MAP.example.yaml docs/AGENT_MAP.yaml
  echo "⚠ docs/AGENT_MAP.yaml created — EDIT fill in real surfaces before next phiếu"
  echo "⚠ Validator: doctor validate-map (run pre-commit). Build doctor binary (cụm B pending)."
fi
```

### 3.5. Setup pre-commit hook (CRITICAL — enforces docs gate) — `sos adopt` arms this automatically (born-wire)

**Without this, docs có thể commit lỗi thời → Architect viết phiếu sai.**

```bash
# Copy hook script
mkdir -p hooks
cp ~/sos-kit/hooks/pre-commit hooks/pre-commit
chmod +x hooks/pre-commit

# Arm it (F09-guarded — detects + protects a pre-existing hook setup)
bash scripts/install-hooks.sh   # preferred over raw `git config core.hooksPath hooks`
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

### 3.6. doctor binary (v2.2 §7 — pending cụm B build)

`doctor` binary cung cấp 5 MVP subcmd cho v2.2 gates:

```bash
doctor lane-check       # §1 lane budget
doctor validate-map     # §4 AGENT_MAP path/anchor
doctor rotate-check     # §6 dòng cap DISCOVERIES/CHANGELOG
doctor runtime-scan     # Sub-mech F token leak
```

**Status (2026-05-28):** doctor binary CHƯA build (cụm B pending). Trong khoảng này:
- Lane budget unenforced — manually count phiếu dòng + anchor vs v2.2 §1 budgets.
- validate-map skip — map có thể drift, đề cao manual review pre-commit.
- rotate-check skip — manual rotate khi >1000 dòng.
- runtime-scan skip — manual grep `.git/config` token leak định kỳ.

**Quản đốc PHẢI narrate "v2.2 doctrine ship, doctor binary pending" cho Sếp KHÔNG tự lừa.**
v2.2 chỉ có răng đủ khi doctor binary cài xong (cụm B nhịp 3).

Khi cụm B ship: `cargo install --path ~/doctor` + thêm `.mcp.json` entry `"doctor": { "command": "~/.cargo/bin/doctor", "args": ["serve"] }`.

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
- Hook `architect-guard.sh` — chặn cứng .py/.rs/.ts read khi marker `.sos-state/architect-active`
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

**Anti-patterns (orchestrator MUST NOT):**
- **Không fake-gate giữa phase.** APPROVAL_GATE là gate user DUY NHẤT (trước EXECUTE). Đừng chèn "is this OK?" giữa DRAFT/CHALLENGE/RESPOND.
- **Không hỏi user pick/order khi đã được ủy quyền "tùy em".** Bulk input → auto-classify + propose wave + 1 AskUserQuestion duy nhất confirm wave plan.
- **Không tự code thay vì spawn Worker.** Main session = orchestrator, không phải executor. Code → spawn Worker EXECUTE.
- **Không skip marker hygiene.** `mkdir -p .sos-state && touch .sos-state/architect-active` trước spawn Architect; `rm -f .sos-state/architect-active` trước spawn Worker.

Full contract: `agents/orchestrator.md` (condensed, ~85 lines) + `docs/ORCHESTRATION.md` (full spec).
```

### 5. Verify install

```bash
# Test hook script offline
bash scripts/session-start-banner.sh
# → should print BACKLOG Active sprint banner

# Test architect-guard.sh
mkdir -p .sos-state
touch .sos-state/architect-active
echo '{"tool_input":{"file_path":"src/main.rs"}}' | bash scripts/architect-guard.sh
# → should exit 2 with "🚫 Architect envelope violation"
echo "exit code: $?"   # → 2
rm .sos-state/architect-active

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
| `architect-guard.sh` không block | Check `.sos-state/architect-active` có tồn tại không (`ls -la .sos-state/`) |
| Hook block worker khi spawn | Worker spawn cần marker NOT exist — `rm -f .sos-state/architect-active` trước |
| `/idea` slash không nhận | Skill load lúc Claude Code start — restart |
| BACKLOG.md không tồn tại | Bootstrap: `cp ~/sos-kit/templates/BACKLOG_template.md docs/BACKLOG.md` |
| Per-spawn `Bash(touch .sos-state/architect-active)` permission prompt | Bạn chưa làm Step 2.5 — copy `templates/claude-settings.local.json` vào `.claude/settings.local.json` |

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
