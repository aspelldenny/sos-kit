#!/usr/bin/env bash
# status — a few lines of live repo state for the start of an agent session (any agent).
# Prints: repo, branch, uncommitted count, then the first "## " section of the state doc
# (its open "- [ ]" items, or its first lines when it has none), and a note when that section
# or the entry files outgrow their budget (same numbers as `sos check`). State doc: $SOS_STATE_FILE,
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
    n=$(awk '/^## /{s++} s==1' "$state" | wc -l | tr -d ' ')
    [ "$n" -gt 80 ] && echo "Note: that section is $n lines (keep under 80): open items only; done ones go to the changelog."
fi
# A reviewer from the other family runs through `sos role`, not the session's own subagent: say so where
# Quản đốc looks each session (0.3.6–0.3.7: `sos role` was never called in seven apps).
if [ -f .sos.toml ]; then
    route=$(awk '
        /^[[:space:]]*\[/ { t=$0; sub(/#.*/,"",t); gsub(/[[:space:]]/,"",t) }
        (t=="[roles.reviewer.claude]" || t=="[roles.reviewer.codex]") && /^[[:space:]]*backend[[:space:]]*=/ {
            v=$0; sub(/^[^=]*=[[:space:]]*["\047]/,"",v); sub(/["\047].*/,"",v)
            from=substr(t, 17, length(t) - 17)
            if (v != from) printf "%s%s-built work → %s", (n++ ? "; " : ""), from, v
        }' .sos.toml)
    [ -n "$route" ] && echo "Người soát: ${route} — run it with \`sos role reviewer --from <builder> --brief <file>\`."
fi
entry=$(cat AGENTS.md CLAUDE.md 2>/dev/null | wc -l | tr -d ' ')
[ "$entry" -gt 200 ] && echo "Note: AGENTS.md + CLAUDE.md are $entry lines (keep under 200): route to docs, move reference into them."
# The advisor fails silently from a hook (a lost login, 15–17/10/2026): say so where Quản đốc looks.
if [ -f .advise-state/calls.jsonl ]; then
    "${SOS_PYTHON:-python3}" - <<'PY' 2>/dev/null  # the adapters pass their own interpreter (Windows)
import json
last = None
for line in open(".advise-state/calls.jsonl", errors="replace"):
    try:
        d = json.loads(line)
    except ValueError:
        continue
    if d.get("kind") != "role":
        last = d
if last and not last.get("ok", True) and last.get("fix"):
    print(f"Note: the advisor's last call failed ({last.get('ts', '?')}): {str(last.get('error') or '')[:120]}."
          f" Fix: {last['fix']}, then `harness-lite/scripts/advise --check`.")
PY
fi
exit 0
