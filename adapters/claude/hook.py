#!/usr/bin/env python3
"""Claude Code adapter: translate Claude Code hook payloads into calls to the agent-neutral
scripts/ tools, and their results back into Claude Code's hook output. No policy lives here.

  hook.py session-start   SessionStart          -> scripts/status.sh (stdout becomes context)
  hook.py pre-edit        PreToolUse Edit|Write  -> scripts/env-guard.sh (exit 2 blocks)
  hook.py post-bash       PostToolUse(+Failure) Bash -> scripts/test-watch.py (additionalContext)

Payload shape (Claude Code): tool_input.file_path / notebook_path; Bash tool_response is
{stdout, stderr, ...}; PostToolUseFailure carries `error`; subagent calls carry agent_id.
"""
import json, os, subprocess, sys
from pathlib import Path

ROOT = Path(os.environ.get("CLAUDE_PROJECT_DIR") or Path(__file__).resolve().parents[2])
SCRIPTS = ROOT / "scripts"


def payload():
    try:
        return json.load(sys.stdin)
    except Exception:
        return None


def session_start():
    r = subprocess.run(["bash", str(SCRIPTS / "status.sh")], cwd=ROOT, capture_output=True, text=True)
    sys.stdout.write(r.stdout)


def pre_edit():
    d = payload()
    if d is None:  # unreadable payload on a guard: block rather than guess
        print("BLOCKED: adapters/claude/hook.py could not read the hook payload.", file=sys.stderr)
        sys.exit(2)
    ti = d.get("tool_input") or {}
    paths = [p for p in (ti.get("file_path"), ti.get("notebook_path")) if p]
    if not paths:
        return
    r = subprocess.run(["bash", str(SCRIPTS / "env-guard.sh"), *paths], capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(r.stderr)
        sys.exit(2)


def post_bash():
    d = payload()
    if not d or d.get("tool_name") != "Bash":
        return
    resp = d.get("tool_response") or {}
    out = "\n".join([resp.get("stdout", ""), resp.get("stderr", "")]) if isinstance(resp, dict) else str(resp)
    event = d.get("hook_event_name", "PostToolUse")
    if event == "PostToolUseFailure":
        out = (d.get("error") or "") + "\n" + out
    key = f"claude-{d.get('session_id', '')}-{d.get('agent_id', 'main')}"
    cmd = ["python3", str(SCRIPTS / "test-watch.py"), "--key", key,
           "--cmd", (d.get("tool_input") or {}).get("command", "")]
    if event == "PostToolUseFailure":
        cmd += ["--exit", "1"]
    r = subprocess.run(cmd, input=out, cwd=ROOT, capture_output=True, text=True, timeout=330)
    if r.stdout.strip():
        print(json.dumps({"hookSpecificOutput": {"hookEventName": event, "additionalContext": r.stdout.strip()}}))


if __name__ == "__main__":
    {"session-start": session_start, "pre-edit": pre_edit, "post-bash": post_bash}[sys.argv[1]]()
