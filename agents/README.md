# agents/ — canonical subagent definitions

> Source of truth for sos-kit's Claude Code subagents.

## Files

**Spawnable subagents** (symlinked into `.claude/agents/` — see below):

- `architect.md` — Kiến trúc sư subagent (Read/Write/Glob only — no code access)
- `worker.md` — Thợ subagent (full code tools — no vision docs access)
- `advisory-watch.md` — Trinh sát specialist (scoped Bash, queries GHSA)
- `boundary-check.md` — Giám sát specialist (scoped Bash for git+grep, checks 5 INV)

**Main-session persona** (NOT a spawnable subagent):

- `orchestrator.md` — Quản đốc handbook; the main Claude Code session's contract. Has no `.claude/agents/` entry because the main session is never spawned as a subagent.

These are the **canonical** versions, English, "Chủ nhà" role-name voice. When installing sos-kit into a project (per [`../INSTALL.md`](../INSTALL.md)), copy the spawnable subagents from here:

```bash
cp ~/sos-kit/agents/{architect,worker,advisory-watch,boundary-check}.md <your-project>/.claude/agents/
```

## Why `.claude/agents/` exists — and why it can't drift

Claude Code spawns subagents from `.claude/agents/`, not from this top-level `agents/` dir. So the four spawnable handbooks must also be present under `.claude/agents/`. To avoid a second copy that drifts, **`.claude/agents/*.md` are symlinks** to the canonical files here:

```
.claude/agents/architect.md  →  ../../agents/architect.md   (symlink)
.claude/agents/worker.md     →  ../../agents/worker.md
.claude/agents/advisory-watch.md  →  ../../agents/advisory-watch.md
.claude/agents/boundary-check.md  →  ../../agents/boundary-check.md
```

One real file per agent, pointed at twice — drift is **impossible** (there is no second copy to diverge). Edit `agents/*.md`; `.claude/agents/` reflects it instantly. No sync step, no hook, nothing to remember.

> **History:** earlier this repo kept a *separate* `.claude/agents/` copy with `Chủ nhà` swapped to `Sếp` via `scripts/sync-personal-agents.sh`, kept fresh "by hand." That script went un-run for a month → the copy drifted → contributed to a downstream incident. The swap itself was a category error: `Chủ nhà` is a **role name** (tên vai), not a form of address; the `Sếp`/`anh`/`em` register is a separate conversational layer (see `CLAUDE.md` → Language). Removing the swap let us symlink instead. Mechanism beats memory.

**Hard rules:**
- Edit `agents/*.md` only (canonical). The symlinks need no maintenance.
- If you add a spawnable subagent, create its canonical file here, then `ln -s ../../agents/<name>.md .claude/agents/<name>.md`.

External users / Tarot / other projects copy real files from `agents/` into their own `.claude/agents/` (per `../INSTALL.md`) and re-copy on kit update.

## Invocation modes

Both subagents support 2 modes triggered by phrases in the orchestrator's spawn prompt — see body of each file for the table. Summary:

- **Architect:** `DRAFT` (write new phiếu) | `RESPOND` (respond to Worker challenge)
- **Worker:** `CHALLENGE` (verify phiếu against code, write Debate Log) | `EXECUTE` (code, test, commit)

The Architect ↔ Worker debate loop is documented in [`../docs/ORCHESTRATION.md`](../docs/ORCHESTRATION.md) and [`../docs/HANDOFF.md`](../docs/HANDOFF.md) (Handoff 2.5).
