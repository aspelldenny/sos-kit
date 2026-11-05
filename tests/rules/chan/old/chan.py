#!/usr/bin/env python3
"""Cổng chặn cho session chạy trong ~/VibeNotes (Tổng đài và session ghi chú).

Hook PreToolUse của Claude Code: đọc JSON từ stdin, thoát 2 + lời nhắn để chặn.
Vault chỉ đọc, ghi chú và chuyển việc. Việc vận hành, code, máy chủ đi về session
đầu mối của repo (xem 10_Projects/tong-dai/LUAT.md, mục «Chuyển việc»).
"""
import json
import os
import re
import sys

VAULT = os.path.realpath(os.path.expanduser("~/VibeNotes"))
CAM_BASH = [
    (r"(^|[\s;&|(])(ssh|scp|sftp|rsync|sudo|iptables|nft|systemctl|docker|kubectl)\b", "máy chủ / hạ tầng"),
    (r"(^|[\s;&|(])(xcodebuild|xcrun simctl|fastlane|make|pnpm|npm|yarn|cargo|pip3?|uv)\s", "build / cài đặt / chạy dự án"),
    (r"\bgit\s+-C\s+(\S+)\s+(add|commit|push|reset|checkout|switch|merge|rebase|restore|stash|clean)\b", "git ghi vào repo khác"),
]
LOI = ("[Cổng Tổng đài] Chặn: {why}. Session trong VibeNotes chỉ đọc, ghi bảng và chuyển việc. "
       "Mở hoặc nhắn session đầu mối của repo đó (LUAT.md mục «Chuyển việc»); "
       "không có đầu mối thì start_session ở repo gần nhất.")


def chan(why):
    print(LOI.format(why=why), file=sys.stderr)
    sys.exit(2)


def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        sys.exit(0)
    tool = data.get("tool_name", "")
    inp = data.get("tool_input", {}) or {}

    if tool == "Bash":
        cmd = inp.get("command", "")
        for pat, why in CAM_BASH:
            m = re.search(pat, cmd)
            if not m:
                continue
            if why.startswith("git"):
                target = os.path.realpath(os.path.expanduser(m.group(1).strip("'\"")))
                if target.startswith(VAULT):
                    continue
            chan(why)
    elif tool in ("Edit", "Write", "MultiEdit", "NotebookEdit"):
        path = inp.get("file_path") or inp.get("notebook_path") or ""
        real = os.path.realpath(os.path.expanduser(path))
        mem = os.path.realpath(os.path.expanduser("~/.claude/projects"))
        if not (real.startswith(VAULT) or real.startswith(mem)):
            chan(f"sửa file ngoài vault ({path})")
    sys.exit(0)


if __name__ == "__main__":
    main()
