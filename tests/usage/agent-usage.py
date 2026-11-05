#!/usr/bin/env python3
"""Token usage per agent role for this repo, read from Claude Code's local transcripts.

Reads ~/.claude/projects/<repo path, non-alphanumerics as '-'>/:
  <session>.jsonl                     main session (Quản đốc)
  <session>/subagents/agent-*.jsonl   subagents; role = agentType in agent-*.meta.json
Each API message is counted once: streamed entries share a message id and only the
last one carries the final output count. Tokens only; prices change, look them up.

  python3 tools/agent-usage.py                      every session
  python3 tools/agent-usage.py --since 2026-09-23   entries from that UTC date on
  python3 tools/agent-usage.py --session <id>       one session
"""
import argparse
import glob
import json
import os
import re
from collections import defaultdict

FIELDS = ("input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens", "output_tokens")
HEADERS = ("role", "model", "runs", "calls", "input", "cache write", "cache read", "output")


def default_projects_dir():
    repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    return os.path.join(os.path.expanduser("~/.claude/projects"), re.sub(r"[^A-Za-z0-9]", "-", repo))


def final_usages(path, since, main_session):
    """(model, usage) per message id, keeping the entry with the most output tokens."""
    final = {}
    with open(path, encoding="utf-8") as transcript:
        for number, line in enumerate(transcript):
            try:
                entry = json.loads(line)
            except ValueError:
                continue
            if entry.get("type") != "assistant" or (main_session and entry.get("isSidechain")):
                continue
            if since and entry.get("timestamp", "")[:10] < since:
                continue
            message = entry.get("message") or {}
            usage, model = message.get("usage"), message.get("model", "?")
            if not usage or model == "<synthetic>":
                continue
            key = message.get("id") or entry.get("requestId") or number
            if key not in final or usage.get("output_tokens", 0) >= final[key][1].get("output_tokens", 0):
                final[key] = (model, usage)
    return final.values()


def transcripts(projects_dir, session):
    for main in sorted(glob.glob(os.path.join(projects_dir, (session or "*") + ".jsonl"))):
        yield main, "main (Quản đốc)", True
        for sub in sorted(glob.glob(os.path.join(main[: -len(".jsonl")], "subagents", "agent-*.jsonl"))):
            try:
                with open(sub[: -len(".jsonl")] + ".meta.json", encoding="utf-8") as meta:
                    role = json.load(meta).get("agentType") or "subagent"
            except (OSError, ValueError):
                role = "subagent"
            yield sub, role, False


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--since", help="UTC date YYYY-MM-DD; earlier entries are skipped")
    parser.add_argument("--session", help="one session id (file name without .jsonl)")
    parser.add_argument("--projects-dir", default=default_projects_dir())
    args = parser.parse_args()

    totals = defaultdict(lambda: defaultdict(int))
    runs = defaultdict(set)
    for path, role, main_session in transcripts(args.projects_dir, args.session):
        for model, usage in final_usages(path, args.since, main_session):
            row = totals[(role, model)]
            row["calls"] += 1
            for field in FIELDS:
                row[field] += usage.get(field) or 0
            runs[(role, model)].add(path)
    if not totals:
        print("no transcripts with usage under " + args.projects_dir)
        return

    lines = [HEADERS]
    grand = defaultdict(int)
    for (role, model), row in sorted(totals.items()):
        lines.append((role, model, str(len(runs[(role, model)])), "{:,}".format(row["calls"]))
                     + tuple("{:,}".format(row[field]) for field in FIELDS))
        for key, value in row.items():
            grand[key] += value
    lines.append(("total", "", "", "{:,}".format(grand["calls"])) + tuple("{:,}".format(grand[f]) for f in FIELDS))
    widths = [max(len(line[i]) for line in lines) for i in range(len(HEADERS))]
    for line in lines:
        print("  ".join(cell.ljust(widths[i]) if i < 2 else cell.rjust(widths[i]) for i, cell in enumerate(line)))


if __name__ == "__main__":
    main()
