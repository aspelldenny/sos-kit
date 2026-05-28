#!/usr/bin/env bash
# Install pre-commit hook into .git/hooks/.
# Idempotent — overwrite existing.
# Note: .git/hooks/ may already have active pre-push (separate hook name, no conflict).
#
# Doctrine: WORKFLOW_V2.2.md §7 (hook ship pattern — pre-commit fire mechanical gates).

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
HOOK_SRC="$REPO_ROOT/hooks/pre-commit"
HOOK_DST="$REPO_ROOT/.git/hooks/pre-commit"

if [[ ! -f "$HOOK_SRC" ]]; then
    echo "ERROR: $HOOK_SRC not found"
    exit 1
fi

cp "$HOOK_SRC" "$HOOK_DST"
chmod +x "$HOOK_DST"
echo "✓ Installed pre-commit hook to $HOOK_DST"

# Optional: install pre-push hook nếu có
PRE_PUSH_SRC="$REPO_ROOT/scripts/pre-push-hook.sh"
PRE_PUSH_DST="$REPO_ROOT/.git/hooks/pre-push"
if [[ -f "$PRE_PUSH_SRC" ]]; then
    cp "$PRE_PUSH_SRC" "$PRE_PUSH_DST"
    chmod +x "$PRE_PUSH_DST"
    echo "✓ Installed pre-push hook to $PRE_PUSH_DST"
fi
