#!/usr/bin/env python3
"""PostToolUse / PostToolUseFailure hook for Bash.

After ADVISE_AFTER (default 2) consecutive failing test runs, call tools/advise and put
its guidance into the agent's context (hookSpecificOutput.additionalContext).
A passing test run resets the count. Never blocks; on any internal error it stays silent.
"""
import json, os, re, subprocess, sys
from pathlib import Path

# Agents often pipe tests through `| tail`, which masks the exit code, so failures are
# detected from the output text, not only from PostToolUseFailure.
FAIL_RE = os.environ.get("ADVISE_FAIL_RE",
    r"FAILED \(|^FAIL:|^ERROR:|\*\* TEST FAILED \*\*|Test Suite .* failed|test result: FAILED|"
    r"Tests? failed|BUILD FAILED|failures?=\d|\d+ failed")
TEST_RE = os.environ.get("ADVISE_TEST_RE",
    r"unittest|pytest|make (check|test|ready)|xcodebuild .*test|swift test|cargo test|npm (run )?test")

def main():
    try:
        d = json.load(sys.stdin)
    except Exception:
        return
    event = d.get("hook_event_name", "")
    cmd = (d.get("tool_input") or {}).get("command", "")
    if d.get("tool_name") != "Bash" or not re.search(TEST_RE, cmd):
        return
    root = Path(os.environ.get("CLAUDE_PROJECT_DIR") or d.get("cwd") or ".")
    state = root / ".advise-state"; state.mkdir(exist_ok=True)
    key = re.sub(r"[^A-Za-z0-9_-]", "_", f"{d.get('session_id','')}-{d.get('agent_id','main')}")
    counter = state / f"{key}.count"
    resp = d.get("tool_response") or {}
    output = (resp.get("stdout", "") + "\n" + resp.get("stderr", "")) if isinstance(resp, dict) else str(resp)
    failed = event == "PostToolUseFailure" or (event == "PostToolUse" and re.search(FAIL_RE, output, re.M))
    if event not in ("PostToolUse", "PostToolUseFailure"):
        return
    if not failed:                         # tests passed
        counter.unlink(missing_ok=True); return
    n = int(counter.read_text() or 0) + 1 if counter.exists() else 1
    if n < int(os.environ.get("ADVISE_AFTER", "2")):
        counter.write_text(str(n)); return
    counter.unlink(missing_ok=True)
    err = state / "last-error.txt"; err.write_text(d.get("error") or output[-8000:])
    q = (f"The same tests have now failed {n} times in a row (command: {cmd}). "
         "What is the root cause, and should I change approach?")
    try:
        r = subprocess.run([str(root / "tools/advise"), "--error-file", str(err), q],
                           cwd=root, capture_output=True, text=True, timeout=300)
        advice = r.stdout.strip() if r.returncode == 0 else f"(advisor unavailable: {r.stderr.strip()[:300]})"
    except Exception as e:
        advice = f"(advisor unavailable: {e})"
    print(json.dumps({"hookSpecificOutput": {"hookEventName": event, "additionalContext":
        "ADVISOR (automatic, after repeated test failures). Weigh it against your own evidence; "
        "if you disagree, say why in your report.\n\n" + advice}}))

try:
    main()
except Exception:
    pass  # never disturb the agent
