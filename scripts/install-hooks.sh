#!/usr/bin/env bash
# Activate git hooks by pointing core.hooksPath at the tracked hooks/ dir.
#
# Why core.hooksPath (NOT cp into .git/hooks/): the tracked hook IS the running
# hook — an edit to hooks/pre-commit is live immediately, no stale untracked copy
# drifting from source. Under the old copy method a tracked-hook fix sat dead in
# hooks/ while .git/hooks/ ran the old copy (Két dogfood 2026-06-03). core.hooksPath
# is local git state (not in the diff) → a fresh clone re-runs this script. Idempotent.
#
# IMPORTANT: core.hooksPath redirects ALL hook lookups to hooks/, so EVERY hook must
# live there under its canonical git name — hooks/pre-commit, hooks/pre-push. (pre-push
# was relocated scripts/pre-push-hook.sh → hooks/pre-push for this; a naive switch
# without that move would silently kill pre-push.) Hooks must be tracked +x (100755) —
# git skips non-executable hooks silently.
#
# Doctrine: WORKFLOW_V2.2.md §7 (hook ship pattern). Két dogfood harvest 2026-06-03.

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

if [[ ! -f hooks/pre-commit ]]; then
    echo "ERROR: hooks/pre-commit not found (run from a sos-kit-spawned repo)" >&2
    exit 1
fi

# Ensure executable (working tree). Tracked mode is fixed once via git update-index
# --chmod=+x in the repo that ships hooks/; a clone inherits 100755.
chmod +x hooks/pre-commit 2>/dev/null || true
[[ -f hooks/pre-push ]] && chmod +x hooks/pre-push 2>/dev/null || true

# ── GUARD (F09 — doc-rotate dogfood 2026-06-09): don't SILENTLY hijack an adopter's
#    existing hook setup. core.hooksPath redirects ALL hook lookups to hooks/; if the
#    repo already points hooksPath elsewhere (its own security gate) or has a real
#    .git/hooks/pre-commit, blindly overriding silently DISABLES it (sos-kit's hooks/
#    may not include the adopter's checks). 1 installer, 2 audiences = nuốt-hook-security
#    class bug. So: detect prior setup → confirm (TTY) / abort (non-TTY) before hijack. ──
EXISTING_HP=$(git config --local core.hooksPath 2>/dev/null || true)
if [[ -n "$EXISTING_HP" && "$EXISTING_HP" != "hooks" ]]; then
    echo "⚠️  core.hooksPath is already set to '$EXISTING_HP' (this repo's own hook chain)." >&2
    echo "    Pointing it at sos-kit's hooks/ would REDIRECT every hook → silently disabling" >&2
    echo "    your existing hooks (e.g. a security gate). sos-kit's hooks/ may not contain them." >&2
    echo "    → Merge sos-kit's checks INTO your hook instead (additive), don't override." >&2
    if [[ -t 0 ]]; then
        read -r -p "    Override core.hooksPath → hooks/ anyway? [y/N] " _ans
        [[ "$_ans" =~ ^[Yy]$ ]] || { echo "    Aborted."; exit 1; }
    else
        echo "    ABORTED (non-interactive — won't silently clobber). Re-run in a terminal to override." >&2
        exit 1
    fi
fi
for h in pre-commit pre-push; do
    if [[ -f ".git/hooks/$h" ]] && ! head -1 ".git/hooks/$h" 2>/dev/null | grep -qi 'sample'; then
        echo "⚠️  Existing .git/hooks/$h → moved to .git/hooks/$h.pre-hookspath.bak (core.hooksPath overrides it; .bak = escape hatch)." >&2
    fi
done

git config core.hooksPath hooks
echo "✓ core.hooksPath → hooks/ (tracked hooks now live: $(ls hooks/ 2>/dev/null | tr '\n' ' '))"

# Retire any stale copies the old copy-method left in .git/hooks/ — core.hooksPath
# overrides them, but a leftover .git/hooks/pre-commit reads as 'installed' and
# confuses. Rename (don't delete) — keep an escape hatch.
for h in pre-commit pre-push; do
    if [[ -f ".git/hooks/$h" ]]; then
        mv ".git/hooks/$h" ".git/hooks/$h.pre-hookspath.bak" 2>/dev/null || true
    fi
done
