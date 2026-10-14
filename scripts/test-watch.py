#!/usr/bin/env python3
"""test-watch — agent-neutral advisor trigger.

Feed it every shell command an agent ran plus its output; it prints advisor guidance
when the same agent's tests have failed ADVISE_AFTER times in a row, and nothing otherwise.
Adapters (adapters/<agent>/hook.py) translate their agent's hook payload into this call
and wrap the printed text in whatever that agent uses to add context.

  scripts/test-watch.py --key <session/agent id> --cmd "<command>" [--exit <code>] [--agent claude|codex] < output

Rules (docs/plans/V3_TRIAGE_2026-10-08.md §F):
  - only commands matching TEST_RE count (tests and builds), one streak per runner (`xcodebuild test`,
    `make check`, `cargo test`…, flags, filters and log paths ignored, so `cargo test -p a` and `-p b`
    share one streak); its passing run resets it. A script or heredoc that we cannot classify still
    counts when its output carries a known runner's verdict (reading a log with `tail` does not);
  - failure = non-zero --exit OR failure text in the output (agents pipe through `| tail`,
    which hides the exit code);
  - after `after` (default 2) consecutive failures, call scripts/advise;
  - at most `max` (default 2) advisor calls per failing streak; then tell the agent to stop
    and report to Quản đốc instead of calling again;
  - settings come from `advise --config` (.sos.toml [advisor], env); backend "off" or
    SOS_ADVISOR=off disables it. An internal error is reported in one line, never raised.
"""
import argparse, hashlib, json, os, re, shlex, subprocess, sys
from pathlib import Path

FAIL_RE = os.environ.get("ADVISE_FAIL_RE",
    r"FAILED \(|^FAILED |^FAIL:|^ERROR:|\*\* TEST FAILED \*\*|Test Suite .* failed|test result: FAILED|"
    r"Tests? failed|BUILD FAILED|failures?=[1-9]|errors?=[1-9]|\b[1-9]\d* failed")
# Matched against the start of each command segment (split on ; && || |), after env
# assignments and wrappers, so `cat pytest.ini` or `grep cargo test` is not a test run.
TEST_RE = os.environ.get("ADVISE_TEST_RE",
    r"(python3? -m )?(pytest|unittest)\b|make (check|test|ready|build)\b|xcodebuild\b|swift (test|build)\b|"
    r"cargo (test|nextest|build|check)\b|go (test|build)\b|(npm|pnpm|yarn|bun) (run )?(test|build)\b|(npx )?(vitest|jest)\b")
# A command whose kind we cannot tell from its words: a script or heredoc that may run tests.
SCRIPT_RE = r"(python3?|bash|sh|zsh|ruby|node|swift)\s+(-\s|-c\b|\S+\.(py|sh|rb|js|swift)\b)|.*<<"
# Verdict lines that name the runner when such a script ran tests (not when a log is merely read).
OUTPUT_SUITES = [
    (r"\*\* TEST FAILED \*\*|Testing failed:", r"\*\* TEST SUCCEEDED \*\*", "xcodebuild test"),
    (r"\*\* BUILD FAILED \*\*", r"\*\* BUILD SUCCEEDED \*\*", "xcodebuild build"),
    (r"test result: FAILED", r"test result: ok", "cargo test"),
]
WRAPPER = re.compile(r"^(?:\w+=\S*\s+|(?:timeout|gtimeout)\s+\S+\s+|time\s+|(?:uv|poetry|pipenv) run\s+|bundle exec\s+)*")


def segments(cmd: str):
    """Command segments split on ; && || | outside quotes (shell grammar, not a regex)."""
    lex = shlex.shlex(cmd, posix=True, punctuation_chars=";&|")
    lex.whitespace_split = True
    seg = []
    try:
        for tok in lex:
            if tok and set(tok) <= set(";&|"):
                yield " ".join(seg); seg = []
            else:
                seg.append(tok)
    except ValueError:  # unbalanced quotes: not something we can classify
        return
    yield " ".join(seg)


def runner(seg: str) -> str:
    """The streak key of a test segment: its leading words up to the first flag or path
    (`xcodebuild -scheme X -only-testing:Y test > log` -> `xcodebuild test`), so reruns with
    other flags, filters or log files continue the same streak."""
    words = seg.split()
    if words[0] == "xcodebuild" and any(w in words for w in ("-list", "-version", "-showsdks", "-showBuildSettings", "-showdestinations")):
        return None
    head = []
    for w in words:
        limit = 3 if len(head) > 1 and head[1] == "run" else 2  # `npm run test`, else `cargo test`
        if w.startswith("-") or any(c in w for c in "/=><.:") or len(head) == limit:
            break
        head.append(w)
    if len(words) > 2 and words[1] == "-m":  # python3 -m pytest
        return " ".join(words[:3])
    if head and head[0] == "xcodebuild":
        return "xcodebuild test" if "test" in words or "test-without-building" in words else "xcodebuild build"
    return " ".join(head) or words[0]


def test_segment(cmd: str):
    """The streak key of the first test-running segment of the command, or None."""
    for s in segments(cmd):
        s = WRAPPER.sub("", s.strip())
        if re.match(TEST_RE, s):
            return runner(" ".join(s.split()))
    return None


def is_script(cmd: str) -> bool:
    """A script or heredoc whose own output is the only clue that it ran tests."""
    first = WRAPPER.sub("", cmd.strip())
    return bool(re.match(SCRIPT_RE, first, re.S))


def suite_from_output(output: str):
    """(runner, failed) from a known runner's verdict line in the output, or None."""
    for fail, ok, name in OUTPUT_SUITES:
        if re.search(fail, output):
            return name, True
        if re.search(ok, output):
            return name, False
    return None


def is_test_command(cmd: str) -> bool:
    return test_segment(cmd) is not None


def repo_root() -> Path:
    try:
        out = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, timeout=10)
        if out.returncode == 0:
            return Path(out.stdout.strip())
    except Exception:
        pass
    return Path.cwd()


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--key", default="default")
    ap.add_argument("--cmd", required=True)
    ap.add_argument("--exit", type=int, default=None)
    ap.add_argument("--agent", default="")
    a = ap.parse_args()
    if os.environ.get("SOS_ADVISOR", "on") == "off":
        return
    seg = test_segment(a.cmd)
    output = sys.stdin.read() if not sys.stdin.isatty() else ""
    if seg is None:
        found = suite_from_output(output) if is_script(a.cmd) else None
        if found is None:
            return
        seg, failed = found
    else:
        failed = (a.exit not in (None, 0)) or bool(re.search(FAIL_RE, output, re.M))

    root = repo_root()
    advise = Path(__file__).resolve().parent / "advise"  # shipped next to this script
    if not advise.exists():
        return
    cfg = json.loads(subprocess.run([sys.executable, str(advise), "--config"], cwd=root,
                                    capture_output=True, text=True, timeout=30).stdout or "{}")
    if cfg.get("backend") == "off":
        return
    state = root / ".advise-state"
    state.mkdir(exist_ok=True)
    # One streak per runner, so another suite passing does not hide this failure.
    key = re.sub(r"[^A-Za-z0-9_-]", "_", a.key)[:100] + "-" + hashlib.sha1(seg.encode()).hexdigest()[:8]
    count_f, calls_f = state / f"{key}.fails", state / f"{key}.calls"
    if not failed:
        count_f.unlink(missing_ok=True); calls_f.unlink(missing_ok=True)
        return

    fails = int(count_f.read_text() or 0) + 1 if count_f.exists() else 1
    count_f.write_text(str(fails))
    if fails < int(cfg.get("after", 2)):
        return
    calls = int(calls_f.read_text() or 0) if calls_f.exists() else 0
    cap = int(cfg.get("max", 2))
    if calls >= cap:
        if calls == cap:  # say it once, then stay quiet for the rest of the streak
            calls_f.write_text(str(calls + 1))
            print(f"ADVISOR LIMIT: the advisor was already consulted {cap} times while these tests kept failing. "
                  "Stop changing code; report to Quản đốc what you tried, the failing evidence and your best hypothesis.")
        return
    calls_f.write_text(str(calls + 1))
    count_f.write_text("0")  # next advice needs another `after` failures

    err = state / f"{key}.last-error.txt"
    err.write_text(output[-8000:])
    q = (f"The same tests have now failed {fails} times in a row (command: {a.cmd}). "
         "What is the root cause, and should I change approach?")
    r = subprocess.run([sys.executable, str(advise), "--agent", a.agent, "--error-file", str(err), q],
                       cwd=root, capture_output=True, text=True, timeout=270)
    if r.returncode == 0:
        print("ADVISOR (automatic, after repeated test failures). Weigh it against your own evidence; "
              "if you disagree, say why in your report.\n\n" + r.stdout.strip())
    else:
        print(f"(advisor unavailable: {r.stderr.strip()[:300]} — continue on your own evidence)")


try:
    main()
except Exception as e:  # visible but harmless: the agent keeps working
    print(f"(advisor hook error, continuing without advice: {type(e).__name__}: {e})")
