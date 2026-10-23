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
  - an advisor that cannot run (advise exit 3: not logged in, CLI unusable) does not use up a call;
    Chủ nhà gets a desktop notification (SOS_NOTIFY_CMD overrides, SOS_NOTIFY=off disables), the
    agent is told to report it now, then the hook stays quiet for ADVISE_UNAVAILABLE_QUIET seconds (default 900);
  - settings come from `advise --config` (.sos.toml [advisor], env); backend "off" or
    SOS_ADVISOR=off disables it. An internal error is reported in one line, never raised.
"""
import argparse, hashlib, json, os, re, shlex, shutil, subprocess, sys, time
from pathlib import Path

if os.name == "nt" and not sys.flags.utf8_mode:  # Windows: files, pipes and stdout in UTF-8, not cp1252
    os.environ["PYTHONUTF8"] = "1"                  # (Vietnamese status text broke the hook, 23/10/2026)
    sys.exit(subprocess.run([sys.executable, *sys.argv]).returncode)

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


def token_segments(cmd: str):
    """Command segments as word lists, split on ; && || | outside quotes (shell grammar, not a regex)."""
    lex = shlex.shlex(cmd, posix=True, punctuation_chars=";&|")
    lex.whitespace_split = True
    seg = []
    try:
        for tok in lex:
            if tok and set(tok) <= set(";&|"):
                yield seg; seg = []
            else:
                seg.append(tok)
    except ValueError:  # unbalanced quotes: not something we can classify
        return
    yield seg


def segments(cmd: str):
    for seg in token_segments(cmd):
        yield " ".join(seg)


def unwrap(words: list) -> list:
    """The words after env assignments and wrappers (`CI=1 timeout 60 x` -> `x`), as WRAPPER does."""
    while words:
        if re.fullmatch(r"\w+=.*", words[0], re.S) or words[0] == "time":
            words = words[1:]
        elif words[0] in ("timeout", "gtimeout") or words[:2] in (["uv", "run"], ["poetry", "run"], ["pipenv", "run"], ["bundle", "exec"]):
            words = words[2:]
        else:
            return words
    return words


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


def toml_string_array(text: str, table: str, key: str) -> list:
    """`key = [ "…", '…' ]` in `[table]`, for Python < 3.11 (no tomllib): strings and comments
    are read as TOML has them, so `#` or `]` inside a string and strings inside comments are right."""
    lines = text.splitlines()
    start = next((i for i, l in enumerate(lines) if re.fullmatch(rf"\s*\[{table}\]\s*(#.*)?", l)), None)
    if start is None:
        return []
    rest = []
    for l in lines[start + 1:]:
        if re.match(r"\s*\[", l):
            break
        rest.append(l)
    m = re.search(rf"^\s*{key}\s*=\s*\[", "\n".join(rest), re.M)
    if not m:
        return []
    body, out, i = "\n".join(rest)[m.end():], [], 0
    while i < len(body):
        c = body[i]
        if c == "]":
            return out
        if c == "#":
            nl = body.find("\n", i)
            i = len(body) if nl < 0 else nl
            continue
        if c in "\"'":
            j, buf = i + 1, []
            while j < len(body) and body[j] != c:
                if c == '"' and body[j] == "\\" and j + 1 < len(body):
                    j += 1
                buf.append(body[j]); j += 1
            out.append("".join(buf)); i = j + 1
            continue
        i += 1
    return []  # no closing bracket: not something to guess at


def project_tests() -> list:
    """`.sos.toml` [advisor] tests as word lists: the project's own test commands (POD-agent's
    `node scripts/check.mjs` never reached the advisor, 23/10/2026). Read here, not through
    `advise --config`, so a non-test command costs no extra process."""
    f = next((d / ".sos.toml" for d in (Path.cwd(), *Path.cwd().parents) if (d / ".sos.toml").is_file()), None)
    if f is None:
        return []
    text = f.read_text(errors="replace")
    try:
        import tomllib
        tests = tomllib.loads(text).get("advisor", {}).get("tests", [])
    except ImportError:
        tests = toml_string_array(text, "advisor", "tests")
    except ValueError:
        return []
    if not isinstance(tests, list):  # `sos check` reports a wrong type; the hook just ignores it
        return []
    out = []
    for t in tests:
        try:
            words = shlex.split(t) if isinstance(t, str) else []
        except ValueError:
            words = []
        if words:
            out.append(words)
    return out


def test_segment(cmd: str, own=()):
    """The streak key of the first test-running segment of the command, or None. `own` are
    the project's test commands (word lists): a segment whose words start with one is a run of it."""
    for words in token_segments(cmd):
        s = WRAPPER.sub("", " ".join(words).strip())
        if re.match(TEST_RE, s):
            return runner(" ".join(s.split()))
        words = unwrap(words)
        for t in own:
            if words[:len(t)] == t:
                return shlex.join(t)
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


WIN_TOAST = (
    "$n=[Windows.UI.Notifications.ToastNotificationManager,Windows.UI.Notifications,ContentType=WindowsRuntime];"
    "$x=$n::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02);"
    "$t=$x.GetElementsByTagName('text');"
    "[void]$t.Item(0).AppendChild($x.CreateTextNode($env:SOS_TITLE));"
    "[void]$t.Item(1).AppendChild($x.CreateTextNode($env:SOS_TEXT));"
    "$n::CreateToastNotifier('{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe')"
    ".Show([Windows.UI.Notifications.ToastNotification]::new($x))")


def notify(text: str) -> None:
    """Tell the human directly: an agent may not pass the message on (Chủ nhà, 20/10/2026:
    "có lỗi phải báo, không được im lặng")."""
    if os.environ.get("SOS_NOTIFY") == "off":
        return
    title = "SOS Kit: advisor cannot run"
    try:  # never raise: the message to the worker comes after this
        if os.environ.get("SOS_NOTIFY_CMD"):
            cmd = [*shlex.split(os.environ["SOS_NOTIFY_CMD"]), f"{title}: {text}"]
        elif sys.platform == "darwin" and shutil.which("osascript"):
            cmd = ["osascript", "-e", f"display notification {json.dumps(text)} with title {json.dumps(title)} sound name \"Basso\""]
        elif shutil.which("notify-send"):
            cmd = ["notify-send", "-u", "critical", title, text]
        elif os.name == "nt" and shutil.which("powershell"):  # Windows toast, shown as from PowerShell
            cmd = ["powershell", "-NoProfile", "-NonInteractive", "-Command", WIN_TOAST]
        else:
            return
        subprocess.run(cmd, capture_output=True, timeout=10, env={**os.environ, "SOS_TITLE": title, "SOS_TEXT": text})
    except Exception:
        pass


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
    seg = test_segment(a.cmd, project_tests())
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
    try:
        quiet = int(os.environ.get("ADVISE_UNAVAILABLE_QUIET", "900"))
    except ValueError:
        quiet = 900
    down = state / "unavailable"  # one marker for the repo: a lost login breaks every session
    if down.exists() and time.time() - down.stat().st_mtime < quiet:
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
    if r.returncode == 3:
        calls_f.write_text(str(calls))  # not a consultation: give the call back
        down.write_text(r.stderr.strip()[:500])
        notify(f"{root.name}: {r.stderr.strip()[-200:]}")
        print(f"ADVISOR UNAVAILABLE (no advisor call used): {r.stderr.strip()[:400]}\n"
              "Tell Chủ nhà now, in your next message, that the advisor needs this fix; "
              "do not leave it for the end of your report. Then continue on your own evidence.")
        return
    down.unlink(missing_ok=True)
    if r.returncode == 0:
        print("ADVISOR (automatic, after repeated test failures). Weigh it against your own evidence; "
              "if you disagree, say why in your report.\n\n" + r.stdout.strip())
    else:
        print(f"(advisor unavailable: {r.stderr.strip()[:300]} — continue on your own evidence)")


try:
    main()
except Exception as e:  # visible but harmless: the agent keeps working
    print(f"(advisor hook error, continuing without advice: {type(e).__name__}: {e})")
