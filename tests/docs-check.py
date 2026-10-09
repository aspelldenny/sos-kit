#!/usr/bin/env python3
"""Docs stay true to the repo: every repo path and every `sos …` command named in the live
Markdown must exist. Run: tests/docs-check.py [path-to-sos]   (default target/debug/sos)

Scope: tracked *.md outside archive/ and the historical logs (CHANGELOG.md, docs/research/,
docs/plans/). Paths resolve against the file's directory, the repo root, and — for files that
are installed into apps under harness-lite/ — the kit layout (scripts/, adapters/, templates/app/).
"""
import re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOS = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "target/debug/sos"
SKIP = ("archive/", "docs/research/", "docs/plans/", "CHANGELOG.md")
TOP = {p.name for p in ROOT.iterdir()} | {".claude", ".codex", ".github"}
# Paths that exist only inside a project after `sos install`, or are examples, not kit files.
APP_ONLY = re.compile(r"^(docs/(STATE|FEATURES|BACKLOG|DESIGN)|\.sos\.toml$|AGENTS\.md$|CLAUDE\.md$|harness-lite/(UPSTREAM\.json|hooks/|templates/)|"
                      r"\.claude/(agents|settings)|\.codex/(hooks|config)|\.git/|\.env|evidence/|\.advise-state|hooks/(pre-commit|pre-push)\.local|"
                      r"~/|\$)")
# Where a kit file lands in an app: harness-lite/<x> comes from these kit paths.
APP_TO_KIT = {"harness-lite/scripts/": "scripts/", "harness-lite/adapters/": "adapters/",
              "harness-lite/hooks/": "templates/app/hooks/", "harness-lite/templates/": "templates/app/"}

files = [ROOT / f for f in subprocess.run(["git", "ls-files", "*.md"], cwd=ROOT, capture_output=True, text=True).stdout.split()
         if not f.startswith(SKIP)]

def sos_commands():
    try:
        out = subprocess.run([str(SOS), "--help"], capture_output=True, text=True).stdout
        gate = subprocess.run([str(SOS), "gate", "--help"], capture_output=True, text=True).stdout
    except OSError:
        sys.exit(f"docs-check: build sos first ({SOS} not found)")
    names = lambda s: set(re.findall(r"^\s{2}([a-z][a-z-]+)\s", s.split("Commands:")[1].split("Options:")[0], re.M))
    return names(out), names(gate)

CMDS, GATES = sos_commands()

def exists(rel: str, md: Path) -> bool:
    rel = rel.split("#")[0].rstrip("/").rstrip(".,;:")
    if not rel:
        return True
    cands = [md.parent / rel, ROOT / rel]
    for app, kit in APP_TO_KIT.items():
        if rel.startswith(app):
            cands.append(ROOT / (kit + rel[len(app):]))
    if md.parent.name == "harness-lite" or "harness-lite/" in str(md.relative_to(ROOT)):
        for app, kit in APP_TO_KIT.items():
            short = app.removeprefix("harness-lite/")
            if rel.startswith(short):
                cands.append(ROOT / (kit + rel[len(short):]))
    return any(c.exists() for c in cands)

problems = []
for md in files:
    rel_md = md.relative_to(ROOT)
    text = md.read_text()
    for n, line in enumerate(text.splitlines(), 1):
        for link in re.findall(r"\]\(([^)\s]+)\)", line):
            if re.match(r"^(https?:|mailto:|#)", link):
                continue
            if not exists(link, md):
                problems.append(f"{rel_md}:{n}: broken link {link}")
        for tok in re.findall(r"`([^`\n]+)`", line):
            t = tok.strip()
            m = re.match(r"^sos\s+([a-z-]+)(?:\s+([a-z-]+))?", t)
            if m:
                if m.group(1) not in CMDS:
                    problems.append(f"{rel_md}:{n}: unknown command `sos {m.group(1)}`")
                elif m.group(1) == "gate" and m.group(2) and m.group(2) not in GATES:
                    problems.append(f"{rel_md}:{n}: unknown gate `sos gate {m.group(2)}`")
                continue
            if " " in t or any(c in t for c in "<>*{}|$=()") or "/" not in t and not re.search(r"\.(md|toml|json|sh|py|rs|yml)$", t):
                continue
            first = t.split("/")[0]
            if first not in TOP and not t.startswith(("harness-lite/", "docs/", "scripts/", "adapters/", "templates/", "crates/", "tests/", "recipes/", "skills/")):
                continue
            if APP_ONLY.search(t) and not exists(t, md):
                continue
            if not exists(t, md):
                problems.append(f"{rel_md}:{n}: path not found `{t}`")

for p in problems:
    print(p)
print(f"docs-check: {len(files)} files, {len(problems)} problem(s)")
sys.exit(1 if problems else 0)
