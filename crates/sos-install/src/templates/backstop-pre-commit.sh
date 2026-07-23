#!/usr/bin/env bash
# SOS backstop minimal hook — enforces 2 security invariants only
# (.env block + no-code-on-default); full [8/8] dev hook via `sos new`.
#
# P078i — rendered by `sos install` into a brownfield/adopted repo's
# hooks/pre-commit. Unlike the dev [8/8] hook (hooks/pre-commit at
# sos-kit repo root, rendered only by `sos new` into a fresh dev
# project), this backstop delegates to ONLY the 2 guard scripts below —
# no doc-checker/trust-checker/type-checker phases, since those tools
# are not guaranteed present outside a full sos-kit dev checkout. It
# enforces exactly the 2 invariants that round-4 dogfood (P079) found
# silently fail-open on a brownfield install: `.env*` secret commit,
# and product code committed on the default branch.
#
# fail-CLOSED: if either guard script this hook delegates to is
# missing, the commit is BLOCKED (exit 1) — never "missing -> allowed"
# (that was the round-4 A3/A4 bug in the dev [8/8] hook's own
# else-branches, which this backstop must not repeat).
set -uo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT" || exit 1

echo "[1/2] .env* secret-file block"
if [ ! -f scripts/block-env-commit.sh ]; then
    echo "BLOCKED: SOS backstop guard scripts/block-env-commit.sh missing — cannot verify .env* is not staged." >&2
    exit 1
fi
bash scripts/block-env-commit.sh || exit 1

echo "[2/2] no-code-on-default gate"
if [ ! -f scripts/no-code-on-default.sh ]; then
    echo "BLOCKED: SOS backstop guard scripts/no-code-on-default.sh missing — cannot verify no code staged on default branch." >&2
    exit 1
fi
bash scripts/no-code-on-default.sh || exit 1

exit 0
