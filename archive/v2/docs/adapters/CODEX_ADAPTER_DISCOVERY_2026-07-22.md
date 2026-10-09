# Codex Adapter Discovery Report (from Codex CLI 0.145.0, 2026-07-22)

> Ground-truth spec for P078 Codex adapter. Produced by Codex itself, verified vs its official manual.
> Save to docs/ when P078 starts. This is the P078 render spec + capability-gap source.

## Codex-native surfaces the adapter must render
1. **AGENTS.md** (root) — orchestrator contract; load SOS.md+core/{ROLES,WORKFLOW,POLICY}; main-thread=orchestrator, must NOT implement active ticket; no EXECUTE before exact-version approval. Precedence: global ~/.codex → repo root → subtree; concatenated root→leaf; 32KiB limit; rebuilt per session. (CLAUDE.md not native to Codex; could be project_doc_fallback but native adapter renders AGENTS.md.)
2. **.codex/agents/<role>.toml** — named subagents (architect/worker/advisory-watch/boundary-check). Required: name, description, developer_instructions. Overrides: model, model_reasoning_effort, sandbox_mode, mcp_servers, skills.config. Global: .codex/config.toml [agents] enabled + max_concurrent_threads_per_session.
3. **.agents/skills/<name>/SKILL.md** — skills (idea/forge/apply/retro). Frontmatter: name+description (only these are mechanical). Invoke: $name, /skills, implicit description-match. Symlinkable.
4. **.codex/config.toml** — [mcp_servers.doctor] command/args/enabled_tools/approval/timeouts; [agents]; sandbox_mode; approval_policy; web_search.
5. **.codex/hooks.json** — events: SessionStart, SubagentStart, SubagentStop, PreToolUse, PermissionRequest, PostToolUse, PreCompact, PostCompact, UserPromptSubmit, Stop. matcher/handler tree. PreToolUse denies via JSON {permissionDecision:"deny"} OR exit-2+stderr (Claude-compatible). Project hooks run only for TRUSTED repos; non-managed hooks need /hooks trust; users can disable; enterprise requirements.toml stronger.
6. **.codex/rules/<name>.rules** — Starlark exec rules (experimental): prefix_rule(pattern, decision allow|prompt|forbidden). Most-restrictive wins. Policy for commands OUTSIDE sandbox, NOT in-sandbox tool allowlist.
7. **scripts/codex/* guards** — Claude guards CANNOT be copied: Codex uses apply_patch (tool_name="apply_patch", patch in tool_input.command), current architect-guard/orchestrator-guard/block-env-edit expect file_path → fail-open on Codex patch unless rewritten. Codex reads via Bash (rg/sed) not Read/Glob → architect read-restriction must inspect shell cmds. block-unsafe-merge maybe portable (Bash tool_input.command) but needs adapter test.

## CRITICAL capability GAPS (label PARTIAL per core/POLICY "absence explicit")
1. **Per-agent built-in tool allowlist: NONE.** No Codex equiv of Claude `tools: Read,Write,Glob`. Can sandbox_mode=read-only (advisory_watch/boundary_check OK) but CANNOT express "architect gets ticket Write but no Bash/Grep/Edit". → architect envelope = PreToolUse-hook + prose enforced, WEAKER than Claude structural removal. SOUND for 0.145.0; schema "may evolve" UNCERTAIN.
2. **Repo-distributed named slash commands: NONE.** .claude/commands/*.md (/security-review) → Codex custom prompts deprecated+personal-only. Replacement = repo skill $security-review.
3. **Skill-level allowed-tools: not mechanical.**
4. **Native semantic ticket-version approval: NONE** (neither runtime). approval_policy=on-request approves OPERATIONS not ticket-V2. Must build via persisted approved-version + PreToolUse guard.
5. **Read/Glob path interception: NONE** (Codex reads via shell).

## Enforcement weakness (report VISIBLY, PARTIAL): adapter NOT unbypassable — config ignored if repo untrusted; non-managed hooks need trust; users disable; hosted tools not hook-visible. → retain Git/CI backstops.

## Portable-core validation (P075 WIN): Codex operates 3-role + lifecycle SEMANTICALLY from neutral contract. ALL semantic areas "Clear". Ambiguities = adapter-rendering questions, NOT core contradictions.

## Core UNDER-SPECIFICATION (adapter-shared — BOTH adapters need; core doesn't specify):
ticket storage path/filename/active-selection; ticket Markdown/TOML schema; machine-readable state-file format (ticket/version/state/approved_version/previous_state/blocked_reason); backlog serialization; approval-record marker+mutation-authority; tier-classifier owner; edit-allowlist glob/normalization/symlink/amendment syntax; concurrent-ownership lock/worktree; publish actor division (orchestrator vs worker commit/push/merge); review-trigger map; BLOCKED storage+resume format.
→ These are IMPLICIT in current .claude wiring/markers; to keep Claude+Codex symmetric, lift into core (or shared adapter-spec).

## What must render for enforcement: AGENTS.md + .codex/agents/*.toml (read-only for specialists) + .codex/hooks.json+scripts (role-state on SubagentStart/Stop, architect write-allowlist, worker edit_allow, block-execute-before-approval, block .env/unsafe-publish, validate lifecycle before delivery) + machine-readable SOS state artifact + deterministic approval gate (persisted approved version + PreToolUse deny on mismatch) + Git/CI backstops.

## Verify env: codex-cli 0.145.0; hooks+multi_agent stable/enabled; no repo AGENTS.md/.codex; codex mcp list = no servers (root .mcp.json NOT Codex config); repo unmodified.
