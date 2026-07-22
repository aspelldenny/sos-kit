#!/usr/bin/env bash
# Lane-field contract test (OA-01 regression guard) — P082.
#
# Verifies `phieu/TICKET_TEMPLATE.md` still carries a parseable `**Lane:**`
# field via the real `doctor lane-check` oracle (not a fixture/unit-test).
# Doctor absent → warn-skip (exit 0), fresh-env friendly — this is a
# degraded-mode check, NOT a security gate, so it does not fail-closed like
# scripts/block-unsafe-merge.sh.
#
# Exit-code contract (matches `doctor lane-check` semantics):
#   doctor absent → warn-skip,        this script exits 0
#   doctor exit 2 (missing/unparseable Lane field) → FAIL LOUD, exits 1
#   doctor exit 1 (over Normal budget)              → WARN, exits 0
#   doctor exit 0 (parseable + within budget)        → OK, exits 0
#
# Usage: scripts/lane-check-contract.sh [optional-ticket-path]
#   No arg  → checks phieu/TICKET_TEMPLATE.md only.
#   With arg → also checks the given phiếu path (e.g. the phiếu itself).

set -u

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT" || exit 1

TEMPLATE="phieu/TICKET_TEMPLATE.md"
EXTRA_TICKET="${1:-}"

if ! command -v doctor >/dev/null 2>&1; then
    echo "⚠️  doctor absent — lane contract SKIPPED"
    exit 0
fi

check_one() {
    local path="$1"
    doctor lane-check --ticket "$path" >/tmp/lane-check-contract.out 2>&1
    local code=$?
    local out
    out=$(cat /tmp/lane-check-contract.out)
    rm -f /tmp/lane-check-contract.out

    case "$code" in
        2)
            echo "❌ template missing/unparseable Lane field — OA-01 regression ($path): $out"
            return 1
            ;;
        1)
            echo "⚠️  template over Normal budget ($path): $out"
            return 0
            ;;
        0)
            echo "✅ template Lane field parseable + within budget ($path): $out"
            return 0
            ;;
        *)
            echo "❌ unexpected doctor lane-check exit ($code) on ($path): $out"
            return 1
            ;;
    esac
}

FAIL=0
check_one "$TEMPLATE" || FAIL=1

if [ -n "$EXTRA_TICKET" ]; then
    check_one "$EXTRA_TICKET" || FAIL=1
fi

exit "$FAIL"
