#!/usr/bin/env bash
# env-guard — agent-neutral check: may these paths be edited by an agent?
# Usage: scripts/env-guard.sh <path>...
# Exit 0 = allowed. Exit 2 = at least one path is a real .env file (prints how to fix).
# .env.example is a template and is allowed. Called by adapters/<agent>/hook.py before
# an edit; the git pre-commit (block-env-commit.sh) is the backstop for every agent.
set -uo pipefail

blocked=()
for p in "$@"; do
    # Lower-case: macOS and Windows filesystems are case-insensitive, so .ENV is .env.
    base=$(basename -- "$p" | tr '[:upper:]' '[:lower:]')
    [ "$base" = ".env.example" ] && continue
    case "$base" in
        .env|.env.*) blocked+=("$p") ;;
    esac
done

[ ${#blocked[@]} -eq 0 ] && exit 0

{
    echo "BLOCKED: agents must not edit real .env files: ${blocked[*]}"
    echo "Why: .env holds live secrets; an agent edit puts them in its context and logs."
    echo "How to fix: edit .env.example (placeholders only) and ask Chủ nhà to put the real value in .env by hand."
} >&2
exit 2
