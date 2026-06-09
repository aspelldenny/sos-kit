#!/usr/bin/env bash
# architect-guard.sh — PreToolUse hook: enforce the Architect envelope (read + write)
#
# How it works:
#   - Hook fires on Read / Glob / Write / Edit tool calls (matcher in settings.json)
#   - Reads JSON from stdin (Claude Code hook payload), dispatches by tool_name:
#     · Read/Glob  → block Architect READING source code (src/, code extensions)
#     · Write/Edit → ALLOWLIST: Architect may ONLY Write phiếu files (P<NNN>-*.md)
#   - Detects Architect via marker file .sos-state/architect-active
#   - Out-of-envelope → exits 2 with an error
#
# P069: the Write-allowlist branch closes a doc-vs-hook gap — agents/architect.md says
# Architect "only Write phiếu files" but the read-only guard let any .md/non-product
# write through (orchestrator-guard only denylists product-source). Symmetric to
# orchestrator-guard (Quản đốc can't write product); allowlist, not denylist.
#
# Setup: this script is referenced from .claude/settings.json under hooks.PreToolUse.
# Architect agent must create marker file `.sos-state/architect-active` on spawn.
# (Marker lives outside .claude/ so YOLO mode doesn't prompt — .claude/ is gated
# even with --dangerously-skip-permissions because it holds settings/hooks.)
#
# Note: NO external deps (no jq) — uses pure shell + sed/grep for cross-platform
# compatibility (esp. Windows msys2 bash where jq is not bundled).

set -euo pipefail

# cwd-independent: Claude Code may fire this hook from any cwd (subdir/home). Resolve repo
# root via $CLAUDE_PROJECT_DIR (Claude Code-provided), else from this script's own location,
# so internal relative refs (.sos-state/, docs/) bind to the project — not the caller's cwd.
cd "${CLAUDE_PROJECT_DIR:-$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)}" || exit 0

MARKER_FILE=".sos-state/architect-active"

# If no marker → not running as Architect → allow everything
[ -f "$MARKER_FILE" ] || exit 0

# Read tool input JSON from stdin
INPUT_JSON=$(cat)

# Extract tool name (PreToolUse payload has top-level "tool_name")
TOOL_NAME=$(echo "$INPUT_JSON" | sed -n 's/.*"tool_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

# Extract path argument from JSON (file_path for Read/Write/Edit, pattern for Glob).
# Strategy: greedy regex on flat JSON — fragile for nested quotes, but Claude Code paths
# don't contain them. NO external deps (no jq) for cross-platform (Windows msys2) safety.
PATH_ARG=$(echo "$INPUT_JSON" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
if [ -z "$PATH_ARG" ]; then
    PATH_ARG=$(echo "$INPUT_JSON" | sed -n 's/.*"pattern"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
fi

# If we couldn't parse a path, allow (don't block on unparseable input)
[ -z "$PATH_ARG" ] && exit 0

# Strip leading ./ for matching
NORMALIZED_PATH="${PATH_ARG#./}"

case "$TOOL_NAME" in
    Write|Edit|MultiEdit|NotebookEdit)
        # ── P069: Architect WRITE-allowlist — ONLY phiếu files (P<NNN>-<slug>.md) ──
        # agents/architect.md: "you only Write new phiếu files". The Read/Glob branch
        # allows any .md (docs domain) — too loose for Write. On Write, allowlist the
        # phiếu-file signature; everything else (code, scripts, other docs) is blocked.
        case "$(basename "$NORMALIZED_PATH")" in
            P[0-9]*-*.md) exit 0 ;;   # phiếu file → allow
        esac
        cat >&2 <<EOF
🚫 Architect envelope violation (Write)

Architect may ONLY Write phiếu files (P<NNN>-<slug>.md): $PATH_ARG is outside the envelope.

The Architect writes the phiếu (plan + Task 0 anchors); code, scripts, and other docs
belong to a spawned Worker or the orchestrator. If you need to write this, it's a sign
the work should be a phiếu TASK — not an Architect-direct write.
EOF
        exit 2
        ;;
    *)
        # ── Read / Glob: block Architect READING source code (original logic) ──
        # Allow .md anywhere — docs are Architect's domain even alongside code.
        case "$NORMALIZED_PATH" in
            *.md) exit 0 ;;
        esac
        case "$NORMALIZED_PATH" in
            src/*|*/src/*|lib/*|*/lib/*|app/*|*/app/*|crates/*/src/*|pkg/*|*/pkg/*)
                BLOCKED=1 ;;
            tests/*|*/tests/*|test/*|*/test/*|__tests__/*)
                BLOCKED=1 ;;
            node_modules/*|target/*|dist/*|build/*|.next/*|.nuxt/*|.svelte-kit/*)
                BLOCKED=1 ;;
            *.rs|*.ts|*.tsx|*.js|*.jsx|*.py|*.go|*.java|*.cpp|*.c|*.h|*.hpp)
                BLOCKED=1 ;;
            *)
                BLOCKED=0 ;;
        esac
        if [ "${BLOCKED:-0}" = "1" ]; then
            cat >&2 <<EOF
🚫 Architect envelope violation (Read)

Architect cannot read source code: $PATH_ARG

What to do instead: write a Task 0 anchor in the phiếu — Worker (separate subagent)
grep-verifies it for you. The constraint IS the feature.
EOF
            exit 2
        fi
        exit 0
        ;;
esac
