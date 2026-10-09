#!/usr/bin/env python3
"""Codex adapter: translate Codex hook payloads into calls to the agent-neutral scripts/
tools, and their results back into Codex hook output. No policy lives here.

  hook.py session-start   SessionStart                   -> scripts/status.sh
  hook.py pre-edit        PreToolUse apply_patch          -> scripts/env-guard.sh (exit 2 blocks)
  hook.py post-bash       PostToolUse Bash                -> scripts/test-watch.py (additionalContext)

Payload shape (Codex 0.162, captured live 2026-10-09, tests/adapters/fixtures/codex-*.jsonl):
edits arrive as tool_name "apply_patch" with the V4A patch in tool_input.command
("*** Add|Update|Delete File: <path>", "*** Move to: <path>"), paths often absolute;
Bash tool_response is the output string; no exit code and no PostToolUseFailure.
Known gap: Codex does not run hooks inside custom-role subagents (openai/codex#21753);
the git pre-commit is the backstop.
"""
import json, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
def patch_paths(patch: str) -> list:
    """Every path named on any '*** <header>: <path>' line. Deliberately broader than the V4A
    grammar: Codex's parser trims any whitespace (incl. Unicode) around headers, so a narrow
    regex can be bypassed (review 2026-10-09). Over-matching only means extra paths checked."""
    out = []
    # Split on \n only: str.splitlines() also breaks on \u2028 etc., which Codex keeps
    # inside a path, so 'dir\u2028/.env' would hide the .env (review 2026-10-09).
    for line in patch.split("\n"):
        s = line.strip()
        if s.startswith("***") and ":" in s:
            path = s.split(":", 1)[1].strip()
            if path:
                out.append(path)
    return out


def payload():
    try:
        return json.load(sys.stdin)
    except Exception:
        return None


def context(event, text):
    print(json.dumps({"hookSpecificOutput": {"hookEventName": event, "additionalContext": text}}))


def session_start():
    r = subprocess.run(["bash", str(SCRIPTS / "status.sh")], cwd=ROOT, capture_output=True, text=True)
    if r.stdout.strip():
        context("SessionStart", r.stdout.strip())


def pre_edit():
    d = payload()
    if d is None:
        print("BLOCKED: adapters/codex/hook.py could not read the hook payload.", file=sys.stderr)
        sys.exit(2)
    if d.get("tool_name") != "apply_patch":
        return
    patch = (d.get("tool_input") or {}).get("command", "")
    paths = patch_paths(patch)
    if not paths:  # a patch we cannot parse might touch anything: block rather than guess
        print("BLOCKED: could not find file paths in this apply_patch; re-issue it as a standard "
              "'*** Begin Patch' patch.", file=sys.stderr)
        sys.exit(2)
    r = subprocess.run(["bash", str(SCRIPTS / "env-guard.sh"), *paths], capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(r.stderr)
        sys.exit(2)


def post_bash():
    d = payload()
    if not d or d.get("tool_name") != "Bash":
        return
    resp = d.get("tool_response")
    out = resp if isinstance(resp, str) else json.dumps(resp)
    cmd = ["python3", str(SCRIPTS / "test-watch.py"), "--key", f"codex-{d.get('session_id', '')}",
           "--cmd", (d.get("tool_input") or {}).get("command", "")]
    m = re.match(r"Exit code: (\d+)", out or "")
    if m:
        cmd += ["--exit", m.group(1)]
    r = subprocess.run(cmd, input=out or "", cwd=ROOT, capture_output=True, text=True, timeout=330)
    if r.stdout.strip():
        context("PostToolUse", r.stdout.strip())


if __name__ == "__main__":
    {"session-start": session_start, "pre-edit": pre_edit, "post-bash": post_bash}[sys.argv[1]]()
