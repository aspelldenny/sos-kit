#!/usr/bin/env python3
"""Run each case of tests/rules/<dir>/cases.jsonl through the OLD script that the rules file
replaces, and record what it did as `"old": "allow"|"block"`.

    python3 -I tests/rules/record_old.py            # compare recorded `old` with a fresh run
    python3 -I tests/rules/record_old.py --write    # (re)write `old` into every cases.jsonl
    python3 -I tests/rules/record_old.py chan       # only some dirs

The old script runs as its wiring ran it: only for the hook event and tool matcher it was
registered with (anything else = allow without running), with the case's files laid out in a
fresh temp dir T exactly like `sos hook rules --cases` does (T/home = HOME, T/work = working
directory, tokens {HOME} {WORK} {T}). Programs the old script calls are faked from the case's
`old_fake` (else `fake`): a fake is a small script on PATH (`hub` goes to ~/.local/bin, where the
old hook looked). ssh, gh and hub are always trapped, so a real one never runs; a trapped call
with no fake exits 255 and is reported. `{"error": …}` = the program fails (exit 255; for hub:
not installed). Exit 2 = block, anything else = allow (Claude Code's hook convention).

Unix only (the old scripts are bash/python3); run on macOS or Linux.
"""
import json
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))

CONF = {
    "chan": (["python3", "-I", "chan.py"], "PreToolUse", "Bash|Edit|Write|MultiEdit|NotebookEdit"),
    "win-heavy": (["python3", "-I", "win_heavy_guard.py"], "PreToolUse", "Bash"),
    "worker-guard": (["bash", "worker-guard.sh"], "PreToolUse", "Read|Glob"),
    "tarot-merge": (["bash", "block-unsafe-merge.sh"], "PreToolUse", "Bash"),
    "hub-audit": (["bash", "hub-audit.sh"], "PostToolUse", "Write|Edit|MultiEdit"),
}
TRAPPED = ["ssh", "gh", "hub"]

FAKE_PY = r'''
import json, sys
name, spec_file, log = sys.argv[1], sys.argv[2], sys.argv[3]
args = sys.argv[4:]
spec = json.load(open(spec_file)).get(name)
with open(log, "a") as f:
    f.write(json.dumps([name] + args + ([] if spec is not None else ["<TRAP>"])) + "\n")
if spec is None:
    sys.exit(255)
if isinstance(spec, list):
    spec = next((e for e in spec if all(any(n in a for a in args) for n in e.get("args_has", []))), None)
    if spec is None:
        sys.exit(255)
if "error" in spec:
    sys.exit(255)
sys.stdout.write(spec.get("stdout", ""))
sys.exit(spec.get("exit", 0))
'''


def tokens(s, t, escape):
    def f(p):
        return json.dumps(p)[1:-1] if escape else p
    return (s.replace("{HOME}", f(os.path.join(t, "home")))
             .replace("{WORK}", f(os.path.join(t, "work")))
             .replace("{T}", f(t)))


def place(key, t):
    k = tokens(key, t, False)
    return k if os.path.isabs(k) else os.path.join(t, k)


def run_old(name, case):
    cmd, event, matcher = CONF[name]
    payload = case["payload"]
    ev = payload.get("hook_event_name") or "PreToolUse"
    if ev != event or not re.fullmatch(f"(?:{matcher})", payload.get("tool_name") or ""):
        return "allow", []
    with tempfile.TemporaryDirectory() as tmp:
        t = os.path.realpath(tmp)
        for d in ("home", "work", "bin", "home/.local/bin"):
            os.makedirs(os.path.join(t, d), exist_ok=True)
        for k, v in (case.get("files") or {}).items():
            p = place(k, t)
            os.makedirs(os.path.dirname(p), exist_ok=True)
            with open(p, "w") as f:
                f.write(v)
        for k, v in (case.get("symlinks") or {}).items():
            p = place(k, t)
            os.makedirs(os.path.dirname(p), exist_ok=True)
            os.symlink(place(v, t), p)
        fakes = case.get("old_fake", case.get("fake")) or {}
        spec_file, log = os.path.join(t, "fakes.json"), os.path.join(t, "calls.log")
        with open(spec_file, "w") as f:
            json.dump(fakes, f)
        fake_py = os.path.join(t, "fake.py")
        with open(fake_py, "w") as f:
            f.write(FAKE_PY)
        for prog in sorted(set(TRAPPED) | set(fakes)):
            spec = fakes.get(prog)
            where = "home/.local/bin" if prog == "hub" else "bin"
            if prog == "hub" and (spec is None or (isinstance(spec, dict) and "error" in spec)):
                where = "bin"   # the old hook checks ~/.local/bin/hub: absent = not installed
            p = os.path.join(t, where, prog)
            with open(p, "w") as f:
                f.write(f'#!/bin/sh\nexec python3 -I "{fake_py}" "{prog}" "{spec_file}" "{log}" "$@"\n')
            os.chmod(p, 0o755)
        env = {k: v for k, v in os.environ.items() if not k.startswith(("CLAUDE_", "MAC_FALLBACK"))}
        # LC_ALL=C: under a UTF-8 locale macOS bash 3.2 reads `«$ROLE»` in worker-guard.sh as the
        # unset variable `ROLE\xc2` and exits 1 (= allow, fail-open). We record what each script
        # does when it runs as written; that locale bug is listed in the plan's progress notes.
        env = {k: v for k, v in env.items() if not k.startswith(("LC_", "LANG"))}
        env.update(HOME=os.path.join(t, "home"), PATH=os.path.join(t, "bin") + os.pathsep + env.get("PATH", ""),
                   CLAUDE_PROJECT_DIR=os.path.join(t, "work"), LC_ALL="C")
        text = tokens(json.dumps(payload, ensure_ascii=False), t, True)
        argv = cmd[:-1] + [os.path.join(HERE, name, "old", cmd[-1])]
        r = subprocess.run(argv, input=text.encode(), cwd=os.path.join(t, "work"), env=env,
                           capture_output=True, timeout=60)
        calls = [json.loads(l) for l in open(log)] if os.path.exists(log) else []
        return ("block" if r.returncode == 2 else "allow"), calls


def ordered(case, old):
    out = {}
    for k, v in case.items():
        if k == "old":
            continue
        out[k] = v
        if k == "want":
            out["old"] = old
    if "old" not in out:
        out["old"] = old
    return out


def main():
    argv = sys.argv[1:]
    write = "--write" in argv
    names = [a for a in argv if not a.startswith("--")] or list(CONF)
    bad = 0
    for name in names:
        path = os.path.join(HERE, name, "cases.jsonl")
        lines = open(path, encoding="utf-8").read().splitlines()
        out = []
        for line in lines:
            s = line.strip()
            if not s or s.startswith("//"):
                out.append(line)
                continue
            case = json.loads(s)
            old, calls = run_old(name, case)
            traps = [c for c in calls if c[-1] == "<TRAP>"]
            flag = ""
            if traps:
                flag = f"  [gọi {traps[0][0]} không có fake]"
                bad += 1
            if not write and case.get("old") != old:
                flag += f"  [ghi {case.get('old')} ≠ chạy lại {old}]"
                bad += 1
            mark = "  " if old == case.get("want") else "≠ "
            print(f"{mark}{name}: {case.get('name')}: old={old} want={case.get('want')}{flag}")
            out.append(json.dumps(ordered(case, old), ensure_ascii=False))
        if write:
            with open(path, "w", encoding="utf-8") as f:
                f.write("\n".join(out) + "\n")
    if bad:
        print(f"{bad} vấn đề")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
