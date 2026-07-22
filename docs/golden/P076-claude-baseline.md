# Golden Oracle — Claude Code adapter baseline (P076)

> Frozen d16bf0a on branch P076-claude-adapter-parity, pre-extraction.
> Parity target: after adapter extraction, EVERY item below must reproduce identically.
> Capture method: read-only probes on tracked tree. Re-run the same probes at P076 acceptance and diff.

## 1. Tracked .claude tree + git modes (symlinks = 120000)
```
120000 .claude/agents/advisory-watch.md
120000 .claude/agents/architect.md
120000 .claude/agents/boundary-check.md
120000 .claude/agents/worker.md
100644 .claude/commands/advisory-scan.md
100644 .claude/commands/security-review.md
100644 .claude/settings.json
120000 .claude/skills/apply
120000 .claude/skills/forge
120000 .claude/skills/idea
120000 .claude/skills/init
120000 .claude/skills/retro
```

## 2. Symlink topology
```
  .claude/agents/advisory-watch.md -> ../../agents/advisory-watch.md
  .claude/agents/architect.md -> ../../agents/architect.md
  .claude/agents/boundary-check.md -> ../../agents/boundary-check.md
  .claude/agents/worker.md -> ../../agents/worker.md
  .claude/skills/init -> ../../skills/init
  .claude/skills/apply -> ../../skills/apply
  .claude/skills/idea -> ../../skills/idea
  .claude/skills/forge -> ../../skills/forge
  .claude/skills/retro -> ../../skills/retro
```

## 3. Role capability matrix (agents/*.md frontmatter)
```
advisory-watch.md: name=advisory-watch model=sonnet tools=[Read, Grep, Glob, WebFetch, WebSearch, Bash]
architect.md: name=architect model=opus tools=[Read, Write, Glob, TaskCreate, TaskUpdate, TaskList, AskUserQuestion]
boundary-check.md: name=boundary-check model=sonnet tools=[Read, Grep, Glob, Bash, mcp__doctor__runtime_scan, mcp__doctor__validate_map]
orchestrator.md: name=orchestrator model=opus tools=[[]]
worker.md: name=worker model=sonnet tools=[Read, Write, Edit, Glob, Grep, Bash, TaskCreate, TaskUpdate, TaskList, AskUserQuestion]
```

## 4. Skills registration (skills/*/SKILL.md frontmatter)
```
apply: name=apply caller="sos apply <recipe> CLI (bin/sos.sh)"
forge: name=forge caller="sos recipe new <category>/<name> CLI (bin/sos.sh)"
idea: name=idea caller="UserPromptSubmit hook scripts/idea-smell.sh (regex idea-smell in Sếp message → inject /idea reminder); banner + orchestrator.md prose"
init: name=init caller="sos init CLI (bin/sos.sh prints: run skill /init)"
retro: name=retro caller="weekly cron — advisory-cron register fires `claude -p \"/retro\"` (per-repo opt-in). PENDING: register blocked on advisory-cron fire_task no-timeout DEBT (its own BACKLOG warns claude -p may hang) — see sos-kit BACKLOG"
```

## 5. Slash commands (.claude/commands/)
```
advisory-scan.md
security-review.md
```

## 6. Hook wiring (.claude/settings.json events → scripts)
```json
{
 "SessionStart": [
  {
   "hooks": [
    {
     "type": "command",
     "command": "bash \"$CLAUDE_PROJECT_DIR/scripts/session-start-banner.sh\""
    }
   ]
  }
 ],
 "PreToolUse": [
  {
   "matcher": "Read|Glob|Write|Edit",
   "hooks": [
    {
     "type": "command",
     "command": "bash \"$CLAUDE_PROJECT_DIR/scripts/architect-guard.sh\""
    }
   ]
  },
  {
   "matcher": "Edit|Write|MultiEdit|NotebookEdit",
   "hooks": [
    {
     "type": "command",
     "command": "bash \"$CLAUDE_PROJECT_DIR/scripts/block-env-edit.sh\""
    },
    {
     "type": "command",
     "command": "bash \"$CLAUDE_PROJECT_DIR/scripts/orchestrator-guard.sh\""
    }
   ]
  },
  {
   "matcher": "Bash",
   "hooks": [
    {
     "type": "command",
     "command": "bash \"$CLAUDE_PROJECT_DIR/scripts/block-unsafe-merge.sh\""
    }
   ]
  }
 ],
 "UserPromptSubmit": [
  {
   "hooks": [
    {
     "type": "command",
     "command": "bash \"$CLAUDE_PROJECT_DIR/scripts/idea-smell.sh\""
    }
   ]
  }
 ]
}
```

## 7. sos CLI surface (bin/sos.sh subcommands)
```
adopt
apply
blueprint
contract
init
launch
map
new
recipe
status
sync
--- dispatch case labels ---
      --stack)
      --pilot)
      --force)
      --stack)
    new)
    adopt)
    sync)
    map)
    init)
    blueprint)
    contract)
    apply)
    recipe)
    launch)
    status)
    help|--help|-h)
```

## 8. MCP servers (.mcp.json)
```
doctor
```

## 9. doctor connectivity (verify-setup)
```
  [WIRED ] J1 sentinel-contract — emit 'security-review-start' present; hook-side grep delegated to claude-hooks binary (B+3 shim)
  [WIRED ] J2 rubric-source — agent handbook enumerates INV rubric inline
  [WIRED ] J4 invariants-file — docs/security/INVARIANTS.md present
  [WIRED ] J5 merge-gate — block-unsafe-merge.sh present + registered in settings.json + gates `gh pr merge`
  [WIRED ] J6 verdict-contract — agent emits `Verdict:`/APPROVE; hook-side parse delegated to claude-hooks binary (B+3 shim)
boundary-check: CONNECTED — all wiring joints intact
```

## 10. Acceptance re-run protocol
At P076 acceptance, re-run sections 1-9 probes and diff against this frozen copy.
Additional fire-tests (not captured statically — run live at acceptance):
- Each lifecycle guard (architect-guard, orchestrator-guard, block-env-edit, block-unsafe-merge, idea-smell) with VALID + INVALID payload → exit code + allow/block unchanged.
- SessionStart banner renders BACKLOG.
- `sos new` greenfield → buildable repo; `sos adopt` brownfield → non-clobber, hooks wired.
- Agents/skills/commands discoverable in a Claude session.
