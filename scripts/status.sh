#!/usr/bin/env bash
# status — a few lines of live repo state for the start of an agent session (any agent).
# Prints: repo, branch, uncommitted count, then the first "## " section of the state doc
# (its open "- [ ]" items, or its first lines when it has none). State doc: $SOS_STATE_FILE,
# else .sos.toml [state] file, else docs/BACKLOG.md, else STATE.md.
set -uo pipefail
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$root" || exit 0

branch=$(git branch --show-current 2>/dev/null || echo "?")
dirty=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
echo "$(basename "$root") — branch: ${branch:-detached} · uncommitted files: ${dirty}"

state="${SOS_STATE_FILE:-}"
if [ -z "$state" ] && [ -f .sos.toml ]; then
    state=$(awk '/^[[:space:]]*\[/{t=$0; sub(/#.*/,"",t); gsub(/[[:space:]]/,"",t)} t=="[state]" && /^[[:space:]]*file[[:space:]]*=/{sub(/^[^=]*=[[:space:]]*["\047]/,""); sub(/["\047].*/,""); print; exit}' .sos.toml)
fi
if [ -z "$state" ]; then
    for f in docs/BACKLOG.md STATE.md; do [ -f "$f" ] && { state=$f; break; }; done
fi
if [ -n "$state" ] && [ -f "$state" ]; then
    awk '
        /^## / { n++; if (n == 1) { print; next } else exit }
        n == 1 && /^- \[ \]/ { if (++k <= 10) print; else more++; next }
        n == 1 && NF && !/^- \[x\]/ { if (++o <= 8) other[o] = $0 }
        END {
            if (more) print "  … +" more " more open items"
            if (!k) for (i = 1; i <= o && i <= 8; i++) print other[i]
        }
    ' "$state" | cut -c1-220
    echo "Detail: $state"
fi
