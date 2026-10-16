#!/usr/bin/env python3
"""List written predictions and pass thresholds in a repo, with their age.

The depth-check skill asks whether each threshold written before a result
was later graded against the actual result. Finding those lines by hand
took every run a long grep. This script finds candidates and dates them;
the agent reading the output decides which ones are real predictions and
whether they were graded. It never edits anything and always exits 0.

Usage: predictions.py [REPO] [--days N] [--limit N] [--min-score N] [--json]

Known limits: it reads only Markdown and text files; a prediction phrased
without a bet word ("kỳ vọng X < 50%" alone) needs --min-score 3; persona
prompts and research notes that quote numbers show up as noise.
"""
import argparse
import datetime as dt
import json
import os
import re
import subprocess
import sys

# Words that mark a bet on a future result. "mục tiêu", "target" and
# "threshold" matched mostly engineering specs in trial runs, and "kỳ vọng" /
# "expect" mostly test expectations, so those count for little on their own.
KEYWORDS = re.compile(
    r"dự đoán|du-doan|giả thuyết|thước đo|đạt nếu|trượt nếu|nếu trượt|"
    r"predict|hypothes|pass if|fail if|success metric|kill criteria|"
    r"\b(build|làm|giữ|bỏ|dừng|ship|bật|tắt|mở) nếu",
    re.IGNORECASE,
)
FOLLOW_UP = re.compile(r"đo lại|đếm lại|đo hiệu quả|đọc lại số|re-?measure|follow-?up", re.IGNORECASE)
WEAK_KEYWORDS = re.compile(r"kỳ vọng|expect|ngưỡng|threshold|KPI", re.IGNORECASE)
# What real users or markets do: the results a product prediction is about.
OUTCOME = re.compile(
    r"view|xem TB|xem hết|lượt (tải|xem|mua)|tải về|download|mua|trial|retention|"
    r"giữ chân|quay lại|signup|đăng ký|conversion|chuyển đổi|doanh thu|revenue|"
    r"refund|hoàn tiền|win rate|expectancy|tỉ lệ|tỷ lệ|người dùng|users?\b|khách|"
    r"comment|bình luận|lưu|chia sẻ|share|click|phiên|session",
    re.IGNORECASE,
)
WINDOW = re.compile(r"@\s?\d+\s?h|\btuần \d|\d+\s?ngày|\bD\d{1,2}\b|\bweek|\d+\s?days", re.IGNORECASE)
COMPARATOR = re.compile(
    r"(≥|≤|>=|<=|(?<![-=])>(?!=)|<(?![-!/a-z])|tối thiểu|tối đa|ít nhất|"
    r"không quá|trên|dưới|at least|at most|more than|less than|under|over)"
    r"\s*~?\s*\$?\d",
    re.IGNORECASE,
)
NUMBER_UNIT = re.compile(r"\d+([.,]\d+)?\s*(%|s\b|giây|ngày|tuần|views?|lượt|R\b|x\b)", re.IGNORECASE)
RESULT_WORDS = re.compile(
    r"thực tế|kết quả|đã chấm|đạt|trượt|bị bác|đứng vững|actual|result|"
    r"graded|passed|failed|\bPASS\b|\bFAIL\b|✅|❌",
    re.IGNORECASE,
)
DATE = re.compile(r"\b(20\d\d)-(\d\d)-(\d\d)\b|\b(\d{1,2})/(\d{1,2})/(20\d\d)\b")
SKIP_DIRS = {"node_modules", ".git", "build", "dist", "target", "archive", "vendor",
             "deriveddata", ".venv", "venv", "__pycache__", ".next", "pods", "done",
             "references", "cold-storage", "runs", "evidence", "tmp", "scratch"}
# History files and earlier depth-check reports restate predictions made
# elsewhere; a prediction is graded where it was made.
SKIP_FILES = re.compile(r"(changelog|history|lich-su|discoveries|depth-check)[^/]*$", re.IGNORECASE)
EXTS = (".md", ".markdown", ".txt")


def list_files(repo):
    try:
        out = subprocess.run(
            ["git", "-C", repo, "ls-files", "--cached", "--others", "--exclude-standard"],
            capture_output=True, text=True, check=True).stdout.splitlines()
        files = [f for f in out if f.endswith(EXTS)]
    except (subprocess.CalledProcessError, FileNotFoundError):
        files = []
        for root, dirs, names in os.walk(repo):
            dirs[:] = [d for d in dirs if d.lower() not in SKIP_DIRS and not d.startswith(".")]
            files += [os.path.relpath(os.path.join(root, n), repo) for n in names if n.endswith(EXTS)]
    return [f for f in files
            if not {part.lower() for part in f.split("/")[:-1]} & SKIP_DIRS
            and not SKIP_FILES.search(f)]


def line_dates(repo, path):
    """Line number -> author date from one git blame; empty when untracked."""
    try:
        out = subprocess.run(["git", "-C", repo, "blame", "--line-porcelain", "--", path],
                             capture_output=True, text=True, check=True).stdout
    except (subprocess.CalledProcessError, FileNotFoundError):
        return {}
    dates, current, lineno = {}, None, None
    for row in out.splitlines():
        if re.match(r"^[0-9a-f]{40} ", row):
            lineno = int(row.split()[2])
            uncommitted = row.startswith("0" * 40)
        elif row.startswith("author-time "):
            current = dt.date.fromtimestamp(int(row.split()[1]))
        elif row.startswith("\t") and lineno is not None and not uncommitted:
            dates[lineno] = current
    return dates


PATH_DATE = re.compile(r"(20\d\d)-(\d\d)-(\d\d)|(20\d\d)-(\d\d)-W(\d)")


def path_date(path):
    """A date written in the path (2026-10-15, or a week like 2026-10-W3)."""
    m = PATH_DATE.search(path)
    if not m:
        return None
    try:
        if m.group(1):
            return dt.date(int(m.group(1)), int(m.group(2)), int(m.group(3)))
        return dt.date(int(m.group(4)), int(m.group(5)), 1) + dt.timedelta(days=7 * (int(m.group(6)) - 1))
    except ValueError:
        return None


def stated_dates(text):
    dates = []
    for m in DATE.finditer(text):
        try:
            if m.group(1):
                dates.append(dt.date(int(m.group(1)), int(m.group(2)), int(m.group(3))))
            else:
                dates.append(dt.date(int(m.group(6)), int(m.group(5)), int(m.group(4))))
        except ValueError:
            continue
    return dates


MIN_SCORE = 4  # set from --min-score; 3 tripled the noise in trial runs


def scan_file(repo, path):
    try:
        lines = open(os.path.join(repo, path), encoding="utf-8").read().splitlines()
    except (UnicodeDecodeError, OSError):
        return []
    found, heading = [], ""
    for i, line in enumerate(lines):
        if line.lstrip().startswith("#"):
            heading = line.strip("# ").strip()
            continue
        text = line.strip()
        if len(text) < 8 or text.startswith(("```", "<!--")):
            continue
        if not (COMPARATOR.search(text) or NUMBER_UNIT.search(text)):
            continue
        weak = bool(WEAK_KEYWORDS.search(text))
        score = (3 * bool(KEYWORDS.search(text)) + 2 * bool(KEYWORDS.search(heading))
                 + (2 if weak and COMPARATOR.search(text) else int(weak))
                 + 2 * bool(OUTCOME.search(text)) + bool(WINDOW.search(text))
                 + bool(FOLLOW_UP.search(text)))
        if score < MIN_SCORE:
            continue
        window = " ".join(lines[i + 1:i + 16])
        found.append({
            "file": path, "line": i + 1, "heading": heading[:60],
            "text": re.sub(r"\s+", " ", text)[:180], "score": score,
            "result_nearby": bool(RESULT_WORDS.search(window)),
            "stated_dates": [d.isoformat() for d in stated_dates(text)],
        })
    return found


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("repo", nargs="?", default=".")
    ap.add_argument("--days", type=int, default=0, help="only lines at least this many days old")
    ap.add_argument("--limit", type=int, default=60)
    ap.add_argument("--min-score", type=int, default=4,
                    help="4 (default) keeps bet word + outcome; 3 also catches bare 'kỳ vọng < N%%' lines, with much more noise")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    global MIN_SCORE
    MIN_SCORE = args.min_score
    repo = os.path.abspath(args.repo)
    today = dt.date.today()

    rows = []
    for path in list_files(repo):
        found = scan_file(repo, path)
        if not found:
            continue
        blame = line_dates(repo, path)
        mtime = dt.date.fromtimestamp(os.path.getmtime(os.path.join(repo, path)))
        in_path = path_date(path)
        for row in found:
            # Uncommitted lines have no blame date; an edit refreshes mtime, so a
            # date in the path (content/2026-10-W1/...) is the better guess.
            written, row["date_source"] = blame.get(row["line"]), "git"
            if written is None:
                written, row["date_source"] = (in_path, "path") if in_path else (mtime, "mtime")
            row["written"] = written.isoformat()
            row["age_days"] = (today - written).days
            if row["age_days"] >= args.days:
                rows.append(row)
    rows.sort(key=lambda r: (-r["score"], r["result_nearby"], -r["age_days"]))
    shown = rows[:args.limit]

    if args.json:
        json.dump({"repo": repo, "today": today.isoformat(), "total": len(rows), "rows": shown},
                  sys.stdout, ensure_ascii=False, indent=1)
        print()
        return
    print(f"# Candidate predictions and thresholds — {repo}")
    print(f"{len(rows)} candidate lines at least {args.days} days old; showing {len(shown)}, "
          "strongest first (bet word + user/market outcome + time window), then no result words nearby, then oldest. "
          "Candidates only: open each line and judge whether it is a prediction and whether it was graded.\n")
    print("| File:line | Score | Written | Age (d) | Result words nearby | Heading | Text |")
    print("|---|---|---|---|---|---|---|")
    for r in shown:
        src = "" if r["date_source"] == "git" else f" ({r['date_source']})"
        text = r["text"].replace("|", "\\|")
        print(f"| {r['file']}:{r['line']} | {r['score']} | {r['written']}{src} | {r['age_days']} | "
              f"{'yes' if r['result_nearby'] else 'no'} | {r['heading'].replace('|', '/')} | {text} |")


if __name__ == "__main__":
    main()
