# SubagentStart probe (d2b #4 spec) — Codex CLI 0.145.0, 2026-07-22

## ROOT CAUSE #4 (definitive): Codex custom-role subagent lifecycle hooks DON'T FIRE
- SubagentStart/Stop `matcher` filters on payload `agent_type`.
- DEFAULT subagent: agent_type="default"; matchers "default"/"*"/omitted all FIRE. (payload captured below)
- **CUSTOM-ROLE spawn (architect/worker): NO SubagentStart/Stop hook dispatched AT ALL** — not "architect", not "*", not omitted. Session metadata HAS `agent_role:"architect"` but the hook never fires.
- = Codex 0.145.0 custom-role lifecycle regression/gap. Upstream: github.com/openai/codex/issues/21753.
- Gotcha: project-local .codex/hooks.json needs `git init` to activate; hook trust bypassed for test.

## Default subagent payload (the ONLY one that fires):
SubagentStart: {..."hook_event_name":"SubagentStart","agent_id":"019f8abc...","agent_type":"default"...}
SubagentStop:  {..."hook_event_name":"SubagentStop","agent_id":"...","agent_type":"default","agent_transcript_path":"...","last_assistant_message":"..."}
Identifying field = `agent_type`. matcher matches agent_type ONLY (not tool_name/name/path).

## Implication for the adapter (d2b):
Our marker design (SubagentStart hook matcher "architect" → touch architect-active) CANNOT work — custom-role hooks don't fire on 0.145.0.
Also implies in-subagent PreToolUse likely doesn't fire for custom roles either (dogfood: architect's forbidden apply_patch SUCCEEDED inside spawned agent → consistent with no in-subagent hooks).

## HONEST disposition (per core/POLICY "capability absence explicit"):
- Remove/deprecate the dead SubagentStart-marker mechanism (best-effort at most).
- Declare spawned-role in-subagent envelope enforcement = **MISSING** on Codex 0.145.0 (cite issue 21753). NOT PARTIAL — it genuinely does not enforce inside custom-role subagents.
- Enforcement that DOES work on Codex → rely on it, declare clearly:
  1. MAIN-THREAD PreToolUse guards (dogfood-confirmed: orchestrator-guard + block-env fired on main thread).
  2. Universal Git pre-commit/pre-push hooks (agent-agnostic backstop — survive any runtime).
  3. AGENTS.md role instructions (guidance-only).
- Optionally: orchestrator sets marker via AGENTS.md before delegating (best-effort; helps main-thread ops only, NOT in-subagent).
- Update CAPABILITY.md + SECURITY.md + FindingStatus (relevant gap → MISSING) + AGENTS.md caveat.
