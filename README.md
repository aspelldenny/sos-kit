# sos-kit v2 draft — Architect-as-Subagent

This is a **draft / proof of concept** to test whether Claude Code subagents can replicate the "Architect = docs-only envelope" that Claude Web Project currently provides — but autonomously, no human relay.

## What's in here

```
sos-kit-v2-draft/
├── .claude/
│   ├── agents/
│   │   ├── architect.md        # Kiến trúc sư — tools: Read, Write, Glob (no Bash, no Grep, no Edit)
│   │   └── worker.md            # Thợ — full code tools, but cannot read vision docs
│   └── settings.json            # Hooks config (PreToolUse → architect-guard.sh)
├── scripts/
│   └── architect-guard.sh       # Hook that hard-blocks Read/Glob on src/ when Architect active
└── README.md                    # This file
```

## The mindset shift in 1 paragraph

In v1 sos-kit, Architect lives in Claude Web Project (separate session). Sếp manually relays Architect ↔ Worker via copy-paste. This works but Sếp is the bottleneck.

In v2 draft, Architect is a **subagent** in the same Claude Code session as Worker. Same envelope (docs-only), but enforced via:
1. **`tools` allowlist** in agent frontmatter — Architect literally has no Bash/Grep/Edit
2. **PreToolUse hook** on Read/Glob — blocks paths under `src/` even if agent tries
3. **System prompt** — explicit "do not read code" with the *why*

Trade-off: lose Web Project's persistent doc context (must reload each spawn, ~3-5s). Gain: orchestrator can spawn Architect → wait for phiếu → spawn Worker → wait for results, all without Sếp pasting between sessions. Sếp only does duyệt + nghiệm thu.

## How to test (5 min)

### Setup
1. Copy this draft into a real project that has the sos-kit phiếu workflow set up:
   ```bash
   cp -r sos-kit-v2-draft/.claude/agents/* ~/your-project/.claude/agents/
   cp -r sos-kit-v2-draft/scripts/* ~/your-project/scripts/
   chmod +x ~/your-project/scripts/architect-guard.sh
   # Merge .claude/settings.json hooks block into project settings
   ```

2. Make sure your project has docs needed for Architect:
   - `docs/PROJECT.md`, `docs/SOUL.md` (vision)
   - `docs/DISCOVERIES.md` (init empty if not exists)
   - `docs/ticket/TICKET_TEMPLATE.md`

3. Touch the marker file before invoking Architect agent:
   ```bash
   touch .claude/.architect-active
   ```
   (Later: do this from the orchestrator agent automatically before spawn.)

### Test 1: Architect cannot read source code (envelope works)

Open Claude Code in the project, then:
```
/agents architect "Try to read src/main.rs"
```

Expected: hook fires, you see:
```
🚫 Architect envelope violation
Architect cannot read source code: src/main.rs
What to do instead: write a Task 0 anchor in the phiếu.
```

Pass criterion: Architect cannot bypass the block, even if it tries.

### Test 2: Architect can read docs and write phiếu

```
/agents architect "Build CSV export for /history page"
```

Expected: Architect reads `CLAUDE.md`, `PROJECT.md`, `SOUL.md`, `DISCOVERIES.md`, `TICKET_TEMPLATE.md`, then writes `docs/ticket/P00X-csv-export.md` with Task 0 anchors phrased as "Thợ verify tại src/...".

Pass criterion: phiếu file exists, Task 0 has ⏳ TO VERIFY rows, no ❌ envelope violations triggered, no fabricated function names not cited from docs.

### Test 3: Worker executes (and can read code, cannot read vision)

After Architect writes phiếu:
```bash
rm .claude/.architect-active   # remove marker so Worker isn't blocked
```

Then:
```
/agents worker "Execute docs/ticket/P00X-csv-export.md"
```

Expected: Worker greps the Task 0 anchors, runs tests, edits code, writes Discovery, commits.

Pass criterion: Worker doesn't try to "improve" the phiếu, doesn't read PROJECT.md/SOUL.md, completes tasks in phiếu order.

### Test 4: End-to-end loop (the actual goal)

In Claude Code main session (acting as Orchestrator):
```
You are the Orchestrator. Sếp wants: <feature description>.

1. Touch .claude/.architect-active
2. Invoke architect subagent to draft phiếu
3. Show phiếu file path to Sếp, wait for "go" or "sửa X"
4. Remove marker, invoke worker subagent to execute
5. Show results to Sếp for nghiệm thu
```

Pass criterion: Sếp only types 2 things — "go" after seeing phiếu, "ok" after seeing results. No copy-paste between Web and Code sessions.

## What this draft does NOT include yet

- **QA/Reviewer subagent** — easy add, follow same pattern: tools restricted to Read+Bash, system prompt enforces "test only, don't fix"
- **Vision-doc preload caching** — currently Architect re-reads on every spawn. Could cache in MCP server.
- **Phiếu counter increment** — Architect uses Glob to find next P-number; not as robust as `.phieu-counter` file. Acceptable for v2 draft.
- **Marker file lifecycle automation** — Orchestrator should touch/rm `.architect-active` automatically. Currently manual for testing.
- **Hook env var detection** — better than marker files, if Claude Code exposes `CLAUDE_AGENT_NAME`. Check the docs of your installed version; if available, replace marker-file logic in `architect-guard.sh`.
- **Windows compat** — `architect-guard.sh` is bash. On Windows, either run in WSL/Git Bash, or rewrite as PowerShell.

## Open questions for Sếp to decide

1. **Drop Claude Web Project entirely?** v2 makes it optional. If iterative phiếu refinement (multi-turn chat about a draft) is important, keep Web Project as Architect-mode-with-UI; v2 adds a programmatic alternative.

2. **Where does `/decide` (Chủ nhà skill) live?** Currently a separate skill. Question: should Chủ nhà also be a subagent? Or stay as the human typing into orchestrator? Em recommend keep Chủ nhà = human until Sếp wants more autonomy.

3. **Lightweight phiếu mode** (cảm hứng tier from earlier conversation) — should Orchestrator auto-detect and skip Architect spawn for trivial work? Heuristic: if request fits "1 file, < 30 min, no schema/auth/payment" → skip phiếu. Worker self-verify.

## What success looks like

After 1-2 weeks of testing, Sếp can ship a phiếu without copy-pasting between Claude Web and Claude Code. Two interactions: "go" + "ok". Drift in Architect output (phantom function names, etc.) should be ≤ Web Project baseline.

If yes → migrate sos-kit v1 docs to v2 architecture, deprecate `RELAY_PROTOCOL.md`, refactor PHILOSOPHY.md to lead with the "context envelope for LLM alignment" framing.

If no (envelope leaks, hallucination higher than Web Project, hooks unreliable) → v2 stays as draft, Web Project remains canonical. Worth knowing either way.
