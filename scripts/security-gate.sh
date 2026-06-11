#!/usr/bin/env bash
# Security gate: run mechanical invariants.
# Minimal template — sos-kit ships only universal INV-009 + INV-010.
# Per-repo: extend with project-specific INV (port-bind, docker, CORS, UFW, etc.)
#
# Exit 0 = all pass. Exit 1 = any violation.
#
# Doctrine: WORKFLOW_V2.2.md §7 Sub-mech B (capability) + §7 Sub-mech F (runtime state).

set -u

MECHANICAL_ONLY=false
for arg in "$@"; do
    case "$arg" in
        --mechanical-only) MECHANICAL_ONLY=true ;;
        *) echo "Unknown flag: $arg"; exit 2 ;;
    esac
done

# INV runner resolution (inv-gate harvest 2026-06-11, action 2 — kill the python3 dep):
#   PRIMARY = `inv-gate` Rust binary (`inv-gate check secrets|runtime|...`), parity-faithful with
#     the golden python (Sprint-1 byte-exact pins). install.sh ships it → fresh repos have it.
#   FALLBACK = python3 interpreter running scripts/check-*.py (the pre-2026-06-11 path), kept so
#     repos that adopted before the binary existed don't regress. Removed when the fleet is binary-only.
# Per-check swap (NOT `gate --all`): the binary's full gate is parity-faithful to the 210-LOC tarot
# golden and would block on repos lacking those files (IG-06); the two UNIVERSAL checks below
# (INV-009 secrets, INV-010 runtime) are exactly what this template already ships, so per-check is sound.
INV_BIN=""
if command -v inv-gate >/dev/null 2>&1 && inv-gate --version >/dev/null 2>&1; then
    INV_BIN="inv-gate"
fi

PY=""
if [ -z "$INV_BIN" ]; then
    # Resolve a Python interpreter that ACTUALLY runs. On Windows `python3` is a Microsoft Store
    # shim sitting on PATH that exits non-zero → `command -v` is NOT enough, must test execution.
    # fail-closed (Sếp directive 2026-06-09): không binary, không Python chạy được = không verify
    # được INV = BLOCK (exit 1), KHÔNG silent-skip.
    for c in python3 python py; do
        if command -v "$c" >/dev/null 2>&1 && "$c" -c "" >/dev/null 2>&1; then PY="$c"; break; fi
    done
    if [ -z "$PY" ]; then
        echo "⛔ BLOCKED: security gate cần \`inv-gate\` binary HOẶC Python — không tìm thấy cái nào chạy được" >&2
        echo "   (đã thử: inv-gate, python3, python, py — fail-closed, không verify được = chặn)." >&2
        echo "   Cài inv-gate (curl … install.sh) hoặc đảm bảo 'python'/'python3' chạy được." >&2
        exit 1
    fi
fi

PASS=0
FAIL=0
FAILED_INVS=()

run() {
    local inv="$1"
    local desc="$2"
    shift 2
    echo "--- $inv: $desc ---"
    if "$@"; then
        echo "  PASS"
        PASS=$((PASS + 1))
    else
        echo "  FAIL"
        FAIL=$((FAIL + 1))
        FAILED_INVS+=("$inv")
    fi
    echo
}

# =============================================================================
# Universal invariants (sos-kit ships these)
# =============================================================================

# INV-009: no hardcoded secret in source files
if [ -n "$INV_BIN" ]; then
    run "INV-009" "No hardcoded secret in source files" inv-gate check secrets
else
    run "INV-009" "No hardcoded secret in source files" "$PY" scripts/check-hardcoded-secrets.py
fi

# INV-010: no secret in runtime state + infra files
if [ -n "$INV_BIN" ]; then
    run "INV-010" "No secret in runtime state + infra files (Sub-mech F)" inv-gate check runtime
else
    run "INV-010" "No secret in runtime state + infra files (Sub-mech F)" "$PY" scripts/check-runtime-secrets.py
fi

# =============================================================================
# CUSTOMIZE — extend with per-repo invariants below this line
# =============================================================================
#
# Example extensions (uncomment + adapt):
#
# # INV-001: port bindings (require scripts/check-port-bind.py)
# run "INV-001" "No host-bind 0.0.0.0 except nginx 80/443" \
#     python3 scripts/check-port-bind.py
#
# # INV-002: no :latest tag in docker-compose
# check_inv002() {
#     ! grep -E '^\s+image:.*:latest$' docker-compose*.yml 2>/dev/null
# }
# run "INV-002" "No :latest tag" check_inv002
#
# # INV-003: .env.example contains no real secrets (project-specific allowlist regex)
# # INV-004: .env.{production,staging,...} gitignored + no history leak
# # INV-005: error-tracking config has secret scrubber (Sentry beforeSend etc.)
# # INV-006: CORS not wildcard
# # INV-007: firewall config (UFW / iptables) — requires SSH, --include-ssh flag
# # INV-008: internal services use expose: not ports:

# =============================================================================
# Summary
# =============================================================================
echo "===================================="
echo "Security gate: $PASS passed, $FAIL failed"
if (( FAIL > 0 )); then
    echo "Failed invariants: ${FAILED_INVS[*]}"
    exit 1
fi
exit 0
