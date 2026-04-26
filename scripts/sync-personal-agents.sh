#!/usr/bin/env bash
# sync-personal-agents.sh — regenerate .claude/agents/ from canonical agents/
#
# Why: agents/ is the public source-of-truth (English-neutral, "Chủ nhà" voice).
# .claude/agents/ is the maintainer's local override ("Sếp" voice for Denny's
# personal flow inside the sos-kit repo). To prevent drift, .claude/agents/ is
# regenerated from agents/ via name-swap only. Edit agents/, run this script.
#
# Usage:
#   bash scripts/sync-personal-agents.sh
#
# Safe to re-run. Idempotent.

set -euo pipefail

cd "$(dirname "$0")/.."

mkdir -p .claude/agents

for f in architect.md worker.md; do
    if [ ! -f "agents/$f" ]; then
        echo "ERROR: agents/$f not found — canonical source missing" >&2
        exit 1
    fi
    sed 's/Chủ nhà/Sếp/g' "agents/$f" > ".claude/agents/$f"
    echo "synced: agents/$f → .claude/agents/$f"
done

echo "done."
