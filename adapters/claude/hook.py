#!/usr/bin/env python3
"""Claude Code adapter: translate Claude Code hook payloads into calls to the agent-neutral
scripts/ tools, and their results back into Claude Code's hook output. No policy lives here.

  hook.py session-start   SessionStart          -> scripts/status.sh (stdout becomes context)
  hook.py pre-edit        PreToolUse Edit|Write  -> scripts/env-guard.sh (exit 2 blocks)
  hook.py pre-bash        PreToolUse Bash        -> scripts/sim-guard (exit 2 blocks)
  hook.py post-bash       PostToolUse(+Failure) Bash -> scripts/test-watch.py (additionalContext)

Payload shape (Claude Code): tool_input.file_path / notebook_path; Bash tool_response is
{stdout, stderr, ...}; PostToolUseFailure carries `error`; subagent calls carry agent_id.
"""
import json, os, shutil, subprocess, sys
from pathlib import Path

if os.name == "nt" and not sys.flags.utf8_mode:  # Windows: files, pipes and stdout in UTF-8, not cp1252
    os.environ["PYTHONUTF8"] = "1"                  # (Vietnamese status text broke the hook, 23/10/2026)
    sys.exit(subprocess.run([sys.executable, *sys.argv]).returncode)
os.environ.setdefault("SOS_PYTHON", sys.executable)  # for status.sh

# The kit's files sit two levels above this file: the sos-kit repo root, or an app's
# vendored harness-lite/. The project root is where Claude Code runs.
KIT = Path(__file__).resolve().parents[2]
SCRIPTS = KIT / "scripts"
ROOT = Path(os.environ.get("CLAUDE_PROJECT_DIR") or KIT)


def bash():
    """bash for scripts/*.sh. Windows: Git's own bash. A bare "bash" there is WSL's System32/bash.exe
    (CreateProcess looks in System32 before PATH) or missing when PATH has only Git/cmd, as from
    cmd or PowerShell (23/10/2026: the .env guard then crashed, which does not block)."""
    if os.name != "nt":
        return "bash"
    core = subprocess.run(["git", "--exec-path"], capture_output=True, text=True).stdout.strip()
    for rel in ("../../../bin/bash.exe", "../../../usr/bin/bash.exe"):  # <Git>/mingw64/libexec/git-core
        p = os.path.normpath(os.path.join(core, rel))
        if core and os.path.isfile(p):
            return p
    return shutil.which("bash") or "bash"


def run_script(name, *args, **kw):
    try:
        return subprocess.run([bash(), str(SCRIPTS / name), *args], capture_output=True, text=True, **kw)
    except OSError as e:
        return subprocess.CompletedProcess([name], 127, "", f"BLOCKED: cannot run {name} ({e}); install Git for Windows (Git Bash).\n")


def payload():
    try:
        return json.load(sys.stdin)
    except Exception:
        return None


def session_start():
    r = run_script("status.sh", cwd=ROOT)
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
    r = run_script("env-guard.sh", *paths)
    if r.returncode != 0:
        sys.stderr.write(r.stderr)
        sys.exit(2)


def pre_bash():
    d = payload()
    if d is None:
        print("BLOCKED: adapters/claude/hook.py could not read the hook payload.", file=sys.stderr)
        sys.exit(2)
    if d.get("tool_name") != "Bash":
        return
    r = subprocess.run([sys.executable, str(SCRIPTS / "sim-guard"), "--cmd", (d.get("tool_input") or {}).get("command", "")],
                       capture_output=True, text=True)
    # A guard that crashed or is missing (python exits 2 too) blocks only where a simulator exists:
    # elsewhere it would stop every command.
    if (r.returncode == 2 and "BLOCKED:" in r.stderr) or (r.returncode != 0 and sys.platform == "darwin"):
        sys.stderr.write(r.stderr or f"BLOCKED: sim-guard failed (exit {r.returncode})\n")
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
    cmd = [sys.executable, str(SCRIPTS / "test-watch.py"), "--key", key,
           "--cmd", (d.get("tool_input") or {}).get("command", ""), "--agent", "claude"]
    if event == "PostToolUseFailure":
        cmd += ["--exit", "1"]
    r = subprocess.run(cmd, input=out, cwd=ROOT, capture_output=True, text=True, timeout=300)
    if r.stdout.strip():
        print(json.dumps({"hookSpecificOutput": {"hookEventName": event, "additionalContext": r.stdout.strip()}}))


if __name__ == "__main__":
    {"session-start": session_start, "pre-edit": pre_edit, "pre-bash": pre_bash, "post-bash": post_bash}[sys.argv[1]]()
