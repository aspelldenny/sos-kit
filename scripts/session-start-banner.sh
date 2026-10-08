#!/usr/bin/env bash
# SessionStart banner (v3): a few lines of live state, not the whole BACKLOG.
# Full tracker: docs/BACKLOG.md · harness: harness-lite/CONTRACT.md
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-.}" 2>/dev/null || exit 0

branch=$(git branch --show-current 2>/dev/null || echo "?")
dirty=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
echo "sos-kit — branch: ${branch} · uncommitted files: ${dirty}"

if [ -f docs/BACKLOG.md ]; then
    # First "## " section = active sprint: its heading + open items only.
    awk '
        /^## / { n++; if (n == 1) { print; next } else exit }
        n == 1 && /^- \[ \]/ { if (++k <= 10) print; else more++ }
        END { if (more) print "  … +" more " more open items" }
    ' docs/BACKLOG.md | cut -c1-220
fi
echo "Detail: docs/BACKLOG.md · Harness: harness-lite/CONTRACT.md"
