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
# Green slices do not prove the combined product works (Payquill F35 and F31): name the last
# end-to-end result, or that nothing runs one.
if [ -f .sos.toml ]; then
    ready=$(awk 'm {i=index($0,q q q); if (i) $0=substr($0,1,i-1); if ($0 ~ /[^[:space:]]/) {print "multi-line"; exit}; if (i) exit; next}
        /^[[:space:]]*\[/{t=$0; sub(/#.*/,"",t); gsub(/[[:space:]]/,"",t)}
        t=="[ready]" && /^[[:space:]]*cmd[[:space:]]*=/{sub(/^[^=]*=[[:space:]]*/,""); sub(/[[:space:]]*$/,""); q=substr($0,1,1)
            if (q!="\"" && q!="\047") {print; exit}
            if (substr($0,1,3)==q q q) {rest=substr($0,4); i=index(rest,q q q); if (i) {print substr(rest,1,i-1); exit}; if (rest ~ /[^[:space:]]/) {print "multi-line"; exit}; m=1; next}
            rest=substr($0,2); i=index(rest,q); print (i ? substr(rest,1,i-1) : rest); exit}' .sos.toml | tr -d ' \t\r')
    if [ -z "$ready" ]; then
        echo "Note: .sos.toml [ready] cmd is empty: nothing checks the whole product before handover."
    elif [ -f evidence/ready/runs.log ]; then
        last=$(awk 'NF{l=$0} END{print l}' evidence/ready/runs.log)
        logged=$(echo "$last" | cut -d' ' -f2); rev=${logged%+dirty}
        head=$(git rev-parse --short HEAD 2>/dev/null)
        if [ -z "$last" ]; then :
        elif [ "$rev" != "$head" ]; then echo "Ready: $last (HEAD is now $head)"
        elif [ "$rev" != "$logged" ] || [ -n "$(git status --porcelain -- . ':!evidence/ready' 2>/dev/null)" ]; then
            echo "Ready: $last (the tree may differ from that run: rerun before handover)"
        else echo "Ready: $last"; fi
    fi
    if [ -n "$ready" ] && ! grep -q '^PASS ' evidence/ready/runs.log 2>/dev/null; then
        echo "Note: \`sos gate ready\` has never passed here: get one journey working end to end (\`sos gate skeleton\`) before starting another."
    fi
fi
entry=$(cat AGENTS.md CLAUDE.md 2>/dev/null | wc -l | tr -d ' ')
[ "$entry" -gt 200 ] && echo "Note: AGENTS.md + CLAUDE.md are $entry lines (keep under 200): route to docs, move reference into them."
# The advisor fails silently from a hook (a lost login, 15–17/10/2026): say so where Quản đốc looks.
if [ -f .advise-state/calls.jsonl ]; then
    # The adapters pass their own interpreter; run by hand on Windows, `python3` is often the Store alias.
    py=${SOS_PYTHON:-}
    [ -n "$py" ] || for p in python3 python; do "$p" -c '' >/dev/null 2>&1 && { py=$p; break; }; done
    "${py:-python3}" - <<'PY' 2>/dev/null
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
          f" Fix: {last['fix']}, then `sos advise --check`.")
PY
fi
exit 0
