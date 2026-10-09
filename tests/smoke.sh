#!/usr/bin/env bash
# End-to-end smoke of a built `sos`: install into a scratch repo, commit through the gates,
# and check that a staged .env is refused. Needs git, gitleaks, python3.
# Usage: tests/smoke.sh <path-to-sos-binary>
set -euo pipefail
SOS=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
export PATH="$(dirname "$SOS"):$PATH" GIT_AUTHOR_NAME=smoke GIT_AUTHOR_EMAIL=s@s GIT_COMMITTER_NAME=smoke GIT_COMMITTER_EMAIL=s@s
cd "$T" && git init -q -b main
sos install --dry-run >/dev/null
sos install >/dev/null
sos check
git add -A && git commit -qm "install sos" || { echo "FAIL: clean commit was blocked"; exit 1; }
echo "A=1" > .env.local && git add -f .env.local
if git commit -qm "leak" 2>/dev/null; then echo "FAIL: .env commit was allowed"; exit 1; fi
git restore --staged .env.local
sos update >/dev/null
echo "smoke ok"
