#!/usr/bin/env bash
# session-start-banner.sh — SessionStart hook for sos-kit projects
#
# Behavior:
#   - When Claude Code starts in a sos-kit project, show docs/BACKLOG.md "Active sprint" section
#   - Reminds Sếp what's in flight, pre-empts "I forgot what I was doing"
#   - Architect Rule 0 reminder: phiếu only for items in Active sprint
#
# Configured in .claude/settings.json under hooks.SessionStart
#
# No external deps (no jq) — pure shell + sed/grep/awk for cross-platform.

set -uo pipefail
# Note: NOT using `set -e` because grep -c with 0 matches exits 1, which we treat as normal.

BACKLOG="docs/BACKLOG.md"

# Silent if no BACKLOG.md (project not sos-kit-equipped, OK)
[ ! -f "$BACKLOG" ] && exit 0

# Find the Active sprint header line number
HEADER_LINE=$(grep -n "^## .*Active sprint" "$BACKLOG" 2>/dev/null | head -1 | cut -d: -f1)

# Silent if no Active sprint section
[ -z "$HEADER_LINE" ] && exit 0

# Find the next "^## " section after Active sprint (to know where to stop)
NEXT_SECTION_LINE=$(awk -v start="$HEADER_LINE" 'NR > start && /^## / {print NR; exit}' "$BACKLOG")

# Default end = end of file if no next section
if [ -z "$NEXT_SECTION_LINE" ]; then
    END_LINE=$(wc -l < "$BACKLOG")
else
    END_LINE=$((NEXT_SECTION_LINE - 1))
fi

# Extract Active sprint block
SPRINT_BLOCK=$(sed -n "${HEADER_LINE},${END_LINE}p" "$BACKLOG")

# Count open items ([ ]) and done items ([x]) in this sprint
# grep -c with 0 matches exits 1 but still prints "0" — capture that, ignore exit.
OPEN_COUNT=$(echo "$SPRINT_BLOCK" | grep -c "^- \[ \]" 2>/dev/null) || OPEN_COUNT=0
DONE_COUNT=$(echo "$SPRINT_BLOCK" | grep -c "^- \[x\]" 2>/dev/null) || DONE_COUNT=0

# Banner output
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🏠 Sếp's project — Active sprint status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "$SPRINT_BLOCK" | head -25
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Active sprint: $OPEN_COUNT items đang treo, $DONE_COUNT đã xong"
echo ""
echo "🤖 Orchestrator contract (main session — đọc kỹ, ép tuân thủ):"
echo "    State machine: DRAFT → CHALLENGE → [RESPOND ⇄ CHALLENGE] → APPROVAL_GATE → EXECUTE"
echo "    KHÔNG hỏi user giữa các phase. APPROVAL_GATE là gate DUY NHẤT (trước EXECUTE)."
echo "    KHÔNG đẩy đọc phiếu/code về user — Worker CHALLENGE rà & report ≤5 dòng."
echo "    Marker: touch .sos-state/architect-active trước spawn architect; rm -f trước spawn worker."
echo "    Deferred tools MANDATORY (load đầu session, KHÔNG fallback markdown 1/2/3):"
echo "        ToolSearch select:AskUserQuestion,TaskCreate,TaskUpdate"
echo "    Spec đầy đủ: docs/ORCHESTRATION.md"
echo ""
echo "📌 Architect Rule 0: chỉ viết phiếu cho item trong Active sprint."
echo "    Idea mới → /idea skill (intake vào BACKLOG.md)."
echo "    Pick item hay add idea?"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
