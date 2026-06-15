#!/usr/bin/env bash
# trust-gate.sh — pre-commit gate: auto-exec surface baseline-diff + hidden-unicode check.
#
# Purpose: sos-kit ships auto-exec surfaces as its product (hooks, scripts, .mcp.json,
#   .claude/settings.json, bin/sos.sh, install.sh, phieu/phieu.sh, templates/setup-dev.sh).
#   Any malicious modification to these surfaces that ships silently = irreversible trust breach
#   for every `git pull`-ing user (Rules-File-Backdoor class, BACKLOG line 246).
#   This gate provides the CONTENT INTEGRITY layer (Tier-1 GitHub hardening = already on;
#   this phiếu = content layer. Port of thanhtra v1.2 trust gate, P073).
#
# Deviations from thanhtra v1.2:
#   (a) Baseline-diff NOT hard-fail: sos-kit ships auto-exec as product; reviewed changes
#       are accepted by running `scripts/trust-gate.sh rebaseline` after review → diff visible in PR.
#   (b) Hidden-unicode gate: scans instruction/doc files for U+FEFF, U+200B/C/D, bidi, etc.
#   (c) Paired with SECURITY.md: honest threat model for adopters (see repo root).
#
# Gotcha: `git ls-files` is the enumerator for both check and rebaseline.
#   A NEW auto-exec file MUST be `git add`ed BEFORE running rebaseline, or it is invisible.
#   (git ls-files only lists tracked files — untracked = silently missed by baseline.)
#
# Doctrine: WORKFLOW_V2.2.md §7 Sub-mech F (runtime state) + BACKLOG line 246 threat model.

set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}" 2>/dev/null || exit 0

# ─────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────

BASELINE_FILE=".sos-trust-baseline"

# Auto-exec surfaces tracked by the baseline (git ls-files enumerator).
# IMPORTANT: this list must be kept in sync with SECURITY.md surface inventory.
# settings.local.json is EXCLUDED — it is globally gitignored (per-machine, not shipped).
# Document this in SECURITY.md.
SURFACE_GLOBS=(
    ".claude/settings.json"
    ".mcp.json"
    "bin/sos.sh"
    "hooks/pre-commit"
    "hooks/pre-push"
    "scripts/*.sh"
    "install.sh"
    "phieu/phieu.sh"
    "templates/setup-dev.sh"
)

# Unicode gate: scan instruction/doc files Claude loads into context.
# Broad scope per Sếp-ratified decision 2026-06-15.
# docs/ticket + docs/discoveries INCLUDED (broad scope decision).
# Codepoints: U+FEFF BOM, U+200B/C/D zero-width, U+200E/F bidi marks,
#             U+2060 word joiner, U+180E Mongolian vowel separator,
#             U+E0000–U+E007F tag range (prompt injection vector).
UNICODE_SCAN_DIRS=(
    "CLAUDE.md"
    "agents/"
    "skills/"
    "phieu/"
    "docs/"
    ".claude/"
    "README.md"
)
# Add SECURITY.md if it exists (it's a loaded doc too)
[ -f "SECURITY.md" ] && UNICODE_SCAN_DIRS+=("SECURITY.md")

# ─────────────────────────────────────────────────────────────────────
# Hash tool resolver (cross-platform: macOS sha256sum + shasum fallback)
# Mirrors P071/P072/install.sh probe pattern.
# ─────────────────────────────────────────────────────────────────────
SHA_CMD=""
SHA_ARG=""

if command -v sha256sum >/dev/null 2>&1 && sha256sum --version >/dev/null 2>&1; then
    SHA_CMD="sha256sum"
    SHA_ARG=""
elif command -v shasum >/dev/null 2>&1 && shasum --version >/dev/null 2>&1; then
    SHA_CMD="shasum"
    SHA_ARG="-a 256"
else
    echo "BLOCKED: trust-gate: no sha256 tool found (tried sha256sum, shasum)." >&2
    echo "  Install coreutils (Linux: apt/dnf) or use macOS built-in shasum." >&2
    exit 1
fi

# Wrapper: compute sha256 of a single file → outputs "<hash>  <file>"
hash_file() {
    local f="$1"
    if [ "$SHA_CMD" = "sha256sum" ]; then
        sha256sum "$f"
    else
        shasum -a 256 "$f"
    fi
}

# ─────────────────────────────────────────────────────────────────────
# Unicode grep resolver (cross-platform)
# Strategy: use byte-pattern grep with -r. This avoids grep -P (not on BSD macOS)
# and the /u perl unicode flag complexity. Byte patterns for key hidden codepoints:
#   U+FEFF  BOM              = \xef\xbb\xbf
#   U+200B  zero-width space = \xe2\x80\x8b
#   U+200C  ZWNJ             = \xe2\x80\x8c
#   U+200D  ZWJ              = \xe2\x80\x8d
#   U+200E  LRM              = \xe2\x80\x8e
#   U+200F  RLM              = \xe2\x80\x8f
#   U+2060  word joiner      = \xe2\x81\xa0
#   U+180E  Mongolian sep    = \xe1\xa0\x8e
# (Tag range U+E0000+ = 4-byte UTF-8, use grep -P OR perl for those; rare.)
# grep -E with \x escapes works on GNU grep; BSD grep uses -P for \x.
# Best fallback: rg (ripgrep) > grep -P (GNU) > grep (with LC_ALL=C byte match).
# ─────────────────────────────────────────────────────────────────────
unicode_grep() {
    local targets=("$@")
    # Strategy: try each tool in order of reliability/availability.
    # (1) ripgrep — most reliable, Unicode-aware, fast
    if command -v rg >/dev/null 2>&1; then
        rg --hidden --no-heading -n \
            -e $'\xef\xbb\xbf' \
            -e $'\xe2\x80\x8b' \
            -e $'\xe2\x80\x8c' \
            -e $'\xe2\x80\x8d' \
            -e $'\xe2\x80\x8e' \
            -e $'\xe2\x80\x8f' \
            -e $'\xe2\x81\xa0' \
            -e $'\xe1\xa0\x8e' \
            "${targets[@]}" 2>/dev/null
    # (2) grep with LC_ALL=C and byte patterns via POSIX character class (most portable)
    else
        # Use LC_ALL=C so grep treats input as bytes (not charset-decoded).
        # Pattern matches the UTF-8 byte sequences for our target codepoints.
        # Works on both GNU grep and BSD grep (macOS) — no -P needed.
        LC_ALL=C grep -rn \
            -e $'\xef\xbb\xbf' \
            -e $'\xe2\x80\x8b' \
            -e $'\xe2\x80\x8c' \
            -e $'\xe2\x80\x8d' \
            -e $'\xe2\x80\x8e' \
            -e $'\xe2\x80\x8f' \
            -e $'\xe2\x81\xa0' \
            -e $'\xe1\xa0\x8e' \
            "${targets[@]}" 2>/dev/null
    fi
}

# ─────────────────────────────────────────────────────────────────────
# Enumerate surfaces via git ls-files
# ─────────────────────────────────────────────────────────────────────
enumerate_surfaces() {
    # shellcheck disable=SC2068
    git ls-files ${SURFACE_GLOBS[@]} 2>/dev/null | sort
}

# ─────────────────────────────────────────────────────────────────────
# SUBCOMMAND: rebaseline
# ─────────────────────────────────────────────────────────────────────
cmd_rebaseline() {
    echo "trust-gate: regenerating ${BASELINE_FILE} ..."

    # Warn about any untracked auto-exec matching files (the gotcha)
    local untracked_warning=""
    # Check each glob pattern for untracked files matching that pattern
    for glob in "${SURFACE_GLOBS[@]}"; do
        # Use git ls-files --others to find untracked matches
        while IFS= read -r untracked; do
            [ -n "$untracked" ] && untracked_warning="${untracked_warning}\n  ${untracked}"
        done < <(git ls-files --others "$glob" 2>/dev/null)
    done

    if [ -n "$untracked_warning" ]; then
        echo ""
        echo "WARNING: trust-gate: untracked auto-exec-matching file(s) found:" >&2
        printf "%b\n" "$untracked_warning" >&2
        echo "  These files are NOT in the baseline (git ls-files skips untracked)." >&2
        echo "  Run: git add <file> THEN rebaseline to include them." >&2
        echo ""
    fi

    local surfaces
    surfaces=$(enumerate_surfaces)
    if [ -z "$surfaces" ]; then
        echo "WARNING: trust-gate: no surfaces matched. Is this a sos-kit repo?" >&2
    fi

    # Generate baseline: "<sha256>  <path>" sorted by path
    local tmp_baseline
    tmp_baseline=$(mktemp)
    while IFS= read -r f; do
        if [ -f "$f" ]; then
            hash_file "$f"
        else
            echo "WARNING: trust-gate: surface listed by git ls-files but not found on disk: $f" >&2
        fi
    done <<< "$surfaces" | sort -k2 > "$tmp_baseline"

    mv "$tmp_baseline" "${BASELINE_FILE}"

    local count
    count=$(wc -l < "${BASELINE_FILE}" | tr -d ' ')
    echo "trust-gate: baseline written → ${BASELINE_FILE} (${count} surfaces)"
    echo ""
    echo "REMINDER: run \`git add ${BASELINE_FILE}\` to stage the updated baseline."
    echo "REMINDER: new auto-exec file must be \`git add\`ed BEFORE rebaseline — git ls-files won't see untracked files."
}

# ─────────────────────────────────────────────────────────────────────
# CHECK 1: baseline-diff
# ─────────────────────────────────────────────────────────────────────
check_baseline() {
    local fail=0

    if [ ! -f "${BASELINE_FILE}" ]; then
        echo "BLOCKED: trust-gate: ${BASELINE_FILE} not found." >&2
        echo "  Generate it: scripts/trust-gate.sh rebaseline" >&2
        echo "  Then: git add ${BASELINE_FILE} && git commit" >&2
        return 1
    fi

    local surfaces
    surfaces=$(enumerate_surfaces)

    # Build current hashes
    local tmp_current
    tmp_current=$(mktemp)
    while IFS= read -r f; do
        if [ -f "$f" ]; then
            hash_file "$f"
        else
            echo "WARNING: trust-gate: tracked surface missing on disk: $f" >&2
            # Treat as a change (missing file = surface removed)
            echo "MISSING  $f"
        fi
    done <<< "$surfaces" | sort -k2 > "$tmp_current"

    # Compare current vs baseline
    local changed_files
    changed_files=$(diff "${BASELINE_FILE}" "$tmp_current" 2>/dev/null | grep '^[<>]' | awk '{print $NF}' | sort -u || true)
    rm -f "$tmp_current"

    # Also check for surfaces in baseline not in current (removed surfaces)
    local baseline_paths current_paths
    baseline_paths=$(awk '{print $NF}' "${BASELINE_FILE}" | sort)
    current_paths=$(enumerate_surfaces | sort)

    local added_surfaces removed_surfaces
    added_surfaces=$(comm -13 <(echo "$baseline_paths") <(echo "$current_paths") || true)
    removed_surfaces=$(comm -23 <(echo "$baseline_paths") <(echo "$current_paths") || true)

    if [ -n "$changed_files" ] || [ -n "$added_surfaces" ] || [ -n "$removed_surfaces" ]; then
        echo "" >&2
        echo "BLOCKED: trust-gate: auto-exec surface(s) changed vs baseline." >&2
        if [ -n "$changed_files" ]; then
            echo "  Changed:" >&2
            echo "$changed_files" | sed 's/^/    /' >&2
        fi
        if [ -n "$added_surfaces" ]; then
            echo "  Added (not in baseline):" >&2
            echo "$added_surfaces" | sed 's/^/    /' >&2
        fi
        if [ -n "$removed_surfaces" ]; then
            echo "  Removed (was in baseline):" >&2
            echo "$removed_surfaces" | sed 's/^/    /' >&2
        fi
        echo "" >&2
        echo "  Review the diff in this PR, then rebaseline:" >&2
        echo "    scripts/trust-gate.sh rebaseline" >&2
        echo "    git add ${BASELINE_FILE}" >&2
        echo "  (A new auto-exec file must be \`git add\`ed BEFORE rebaseline.)" >&2
        fail=1
    fi

    return $fail
}

# ─────────────────────────────────────────────────────────────────────
# CHECK 2: hidden-unicode scan
# ─────────────────────────────────────────────────────────────────────
check_unicode() {
    local fail=0

    # Filter scan dirs to only those that exist
    local existing_dirs=()
    for d in "${UNICODE_SCAN_DIRS[@]}"; do
        [ -e "$d" ] && existing_dirs+=("$d")
    done

    if [ ${#existing_dirs[@]} -eq 0 ]; then
        return 0
    fi

    local hits
    hits=$(unicode_grep "${existing_dirs[@]}" 2>/dev/null || true)

    if [ -n "$hits" ]; then
        echo "" >&2
        echo "BLOCKED: trust-gate: hidden Unicode codepoint(s) found in instruction/doc files." >&2
        echo "  (U+FEFF BOM, U+200B/C/D zero-width, U+200E/F bidi, U+2060 word-joiner," >&2
        echo "   U+180E Mongolian vowel sep, U+E0000 tag = prompt injection vector)" >&2
        echo "" >&2
        echo "$hits" | sed 's/^/  /' >&2
        echo "" >&2
        echo "  Fix: replace the raw codepoint with its name (e.g. U+FEFF) in the source." >&2
        echo "  Never paste raw hidden chars — use codepoint escapes or literal names." >&2
        fail=1
    fi

    return $fail
}

# ─────────────────────────────────────────────────────────────────────
# Main dispatch
# ─────────────────────────────────────────────────────────────────────
MODE="${1:-check}"

case "$MODE" in
    rebaseline)
        cmd_rebaseline
        exit 0
        ;;
    check|"")
        FAIL=0
        check_baseline || FAIL=1
        check_unicode  || FAIL=1
        exit $FAIL
        ;;
    *)
        echo "Usage: $0 [check|rebaseline]" >&2
        echo "  check      (default) — run baseline-diff + hidden-unicode checks" >&2
        echo "  rebaseline — regenerate .sos-trust-baseline from current git ls-files surfaces" >&2
        exit 2
        ;;
esac
