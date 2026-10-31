#!/usr/bin/env python3
"""Codex adapter: translate Codex hook payloads into calls to the agent-neutral scripts/
tools, and their results back into Codex hook output. No policy lives here.

  hook.py session-start   SessionStart                   -> scripts/status.sh
  hook.py pre-edit        PreToolUse apply_patch          -> scripts/env-guard.sh (block(): exit 2,
                                                             Windows: deny JSON, exit 0)
  hook.py pre-bash        PreToolUse Bash                 -> scripts/sim-guard (blocks like pre-edit)
  hook.py post-bash       PostToolUse Bash                -> scripts/test-watch.py (additionalContext)

Payload shape (Codex 0.162, captured live 2026-10-09, tests/adapters/fixtures/codex-*.jsonl):
edits arrive as tool_name "apply_patch" with the V4A patch in tool_input.command
("*** Add|Update|Delete File: <path>", "*** Move to: <path>"), paths often absolute;
Bash tool_response is the output string; no exit code and no PostToolUseFailure.
Known gap: Codex does not run hooks inside custom-role subagents (openai/codex#21753);
the git pre-commit is the backstop.
"""
import json, os, re, shutil, subprocess, sys
from pathlib import Path

if os.name == "nt" and not sys.flags.utf8_mode:  # Windows: files, pipes and stdout in UTF-8, not cp1252
    os.environ["PYTHONUTF8"] = "1"                  # (Vietnamese status text broke the hook, 23/10/2026)
    sys.exit(subprocess.run([sys.executable, *sys.argv]).returncode)
os.environ.setdefault("SOS_PYTHON", sys.executable)  # for status.sh

# The kit's files sit two levels above this file: the sos-kit repo root, or an app's
# vendored harness-lite/. The project root is the git top level of the session.
KIT = Path(__file__).resolve().parents[2]
SCRIPTS = KIT / "scripts"
try:  # no git on PATH must not crash the hook: a crash (exit 1) does not block an edit
    _top = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True)
except OSError:
    _top = subprocess.CompletedProcess([], 1, "", "")
ROOT = Path(_top.stdout.strip()) if _top.returncode == 0 else KIT
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


def block(msg):
    """Refuse the edit. Codex blocks on exit 2 with the reason on stderr. On Windows the hook may run
    under `powershell -Command`, which turns any nonzero exit into 1, and exit 1 does not block
    (23/10/2026); there it answers with PreToolUse permissionDecision "deny" and exit 0 instead."""
    msg = msg.strip()
    if os.name == "nt":
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                                                 "permissionDecisionReason": msg}}), flush=True)
        try:  # the reason is in the JSON; a broken stderr must not turn the block into a crash
            print(msg, file=sys.stderr)
        except Exception:
            pass
        sys.exit(0)
    print(msg, file=sys.stderr)
    sys.exit(2)


def payload():
    try:
        return json.load(sys.stdin)
    except Exception:
        return None


def context(event, text):
    print(json.dumps({"hookSpecificOutput": {"hookEventName": event, "additionalContext": text}}))


def session_start():
    r = run_script("status.sh", cwd=ROOT)
    if r.stdout.strip():
        context("SessionStart", r.stdout.strip())


def pre_edit():
    d = payload()
    if d is None:
        block("BLOCKED: adapters/codex/hook.py could not read the hook payload.")
    if d.get("tool_name") != "apply_patch":
        return
    patch = (d.get("tool_input") or {}).get("command", "")
    paths = patch_paths(patch)
    if not paths:  # a patch we cannot parse might touch anything: block rather than guess
        block("BLOCKED: could not find file paths in this apply_patch; re-issue it as a standard "
              "'*** Begin Patch' patch.")
    r = run_script("env-guard.sh", *paths)
    if r.returncode != 0:
        block(r.stderr or f"BLOCKED: env-guard.sh failed (exit {r.returncode})")


def pre_bash():
    d = payload()
    if d is None:
        block("BLOCKED: adapters/codex/hook.py could not read the hook payload.")
    if d.get("tool_name") != "Bash":
        return
    r = subprocess.run([sys.executable, str(SCRIPTS / "sim-guard"), "--cmd", (d.get("tool_input") or {}).get("command", "")],
                       capture_output=True, text=True)
    # A guard that crashed or is missing (python exits 2 too) blocks only where a simulator exists:
    # elsewhere it would stop every command.
    if (r.returncode == 2 and "BLOCKED:" in r.stderr) or (r.returncode != 0 and sys.platform == "darwin"):
        block(r.stderr or f"BLOCKED: sim-guard failed (exit {r.returncode})")


def post_bash():
    d = payload()
    if not d or d.get("tool_name") != "Bash":
        return
    resp = d.get("tool_response")
    out = resp if isinstance(resp, str) else json.dumps(resp)
    cmd = [sys.executable, str(SCRIPTS / "test-watch.py"), "--key", f"codex-{d.get('session_id', '')}",
           "--cmd", (d.get("tool_input") or {}).get("command", ""), "--agent", "codex"]
    m = re.match(r"Exit code: (\d+)", out or "")
    if m:
        cmd += ["--exit", m.group(1)]
    r = subprocess.run(cmd, input=out or "", cwd=ROOT, capture_output=True, text=True, timeout=300)
    if r.stdout.strip():
        context("PostToolUse", r.stdout.strip())


if __name__ == "__main__":
    try:
        {"session-start": session_start, "pre-edit": pre_edit, "pre-bash": pre_bash, "post-bash": post_bash}[sys.argv[1]]()
    except Exception as e:  # a crash (exit 1) would let the edit or command through
        if sys.argv[1:2] != ["pre-edit"] and not (sys.argv[1:2] == ["pre-bash"] and sys.platform == "darwin"):
            raise
        block(f"BLOCKED: adapters/codex/hook.py failed ({type(e).__name__}: {e})")
