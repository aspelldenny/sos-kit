#!/usr/bin/env python3
"""PreToolUse hook (Claude Code, Bash): việc nặng phải chạy trên máy Win.

Luật Sếp 20/10/2026 (CLAUDE.md, "Retrieval and resource safety"): OCR, render
trang, chuyển định dạng, trích/kiểm nguồn mới chạy qua `ssh win`; Mac chỉ là
dự phòng khi Win tắt. Hook chặn (exit 2) lệnh nặng chạy trên Mac nếu Win đang
trả lời. Lệnh đã đi qua `ssh win`, hoặc mở đầu bằng `MAC_FALLBACK=1` (chỉ dùng
sau khi đã kiểm Win không lên), được cho qua. Lỗi của chính hook không chặn gì.
"""
import json
import re
import subprocess
import sys

# Chỉ bắt ở vị trí lệnh (đầu dòng, sau ; & | $(), để chữ trong văn xuôi/heredoc không bị chặn nhầm.
HEAVY = re.compile(
    r"(?:^|[;&|\n]|\$\()\s*(?:(?:sudo|env|nice|time|timeout\s+\S+|\w+=\S*)\s+)*"
    r"(?:\S*/)?("
    r"ocrmypdf|tesseract|pdftoppm|pdftocairo|ebook-convert|gs|magick|hub\s+ocr"
    r"|python\S*\s+(?:-\S+\s+)*\S*(?:inspect_sources|qc_books|check_books)\.py"
    r")(?=\s|$|['\";|&)])",
    re.M,
)


def main():
    try:
        cmd = json.load(sys.stdin).get("tool_input", {}).get("command", "")
    except Exception:
        return 0
    if not cmd or not HEAVY.search(cmd):
        return 0
    if re.match(r"\s*MAC_FALLBACK=1\b", cmd) or re.search(r"\bssh\s+(-\S+\s+(\S+\s+)?)*win\b", cmd):
        return 0
    try:
        up = subprocess.run(
            ["ssh", "-o", "ConnectTimeout=3", "-o", "BatchMode=yes", "win", "true"],
            capture_output=True, timeout=6,
        ).returncode == 0
    except Exception:
        up = False
    if not up:
        return 0
    print(
        "Chặn: lệnh nặng (OCR/render/chuyển định dạng/kiểm nguồn) đang chạy trên Mac "
        "trong khi máy Win đang bật. Luật Sếp 20/10: chạy qua `ssh win` "
        "(WSL ~/BOOK → E:\\BOOK, .venv/.ocrvenv). Chỉ khi Win tắt mới chạy trên Mac, "
        "mở đầu lệnh bằng MAC_FALLBACK=1 và ghi rõ trong báo cáo. Xem CLAUDE.md.",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
