#!/usr/bin/env bash
# orchestrator-guard.sh — PreToolUse hook: chặn cứng Quản đốc (main session) TỰ CODE product.
#
# How it works:
#   - Hook fires on every Edit / Write tool call.
#   - Reads JSON from stdin (Claude Code hook payload), extracts file_path.
#   - product-source (*.swift / *.pbxproj / src/**) chỉ được sửa khi marker
#     `.sos-state/worker-active` TỒN TẠI (= đang trong cửa sổ Thợ EXECUTE).
#   - Không marker → main session (Quản đốc) hoặc Architect đang định tự code → exit 2 (block).
#
# Companion ngược của architect-guard.sh:
#   architect-guard chặn Architect ĐỌC source (Read|Glob, khi architect-active).
#   orchestrator-guard chặn Quản đốc GHI product code (Edit|Write, khi KHÔNG worker-active).
# Subagent tool-calls CÓ fire PreToolUse (bằng chứng: architect-guard chặn được Architect
# subagent) → marker là cách DUY NHẤT phân biệt Thợ-được-phép vs main-session-việt-vị.
#
# Scope HẸP có chủ đích: chỉ product source (`*.swift`/`*.pbxproj`/`src/**`). KHÔNG gồm
# `*.py`/`*.sh`/docs — để kit-maintenance (Quản đốc sửa thẳng bin/sos.sh, scripts/*.sh,
# docs ở Tầng-2 surgical) KHÔNG bị chặn. Trong sos-kit chính nó: không có *.swift/pbxproj,
# src/ chỉ chứa skeleton → guard gần như no-op nhưng vẫn dogfood + nhất quán.
#
# Setup: referenced từ .claude/settings.json hooks.PreToolUse (matcher Edit|Write).
# Quản đốc PHẢI `touch .sos-state/worker-active` TRƯỚC spawn Thợ, `rm -f` sau khi Thợ về
# (agents/orchestrator.md "Marker file hygiene").
#
# No external deps (no jq) — pure shell + sed for cross-platform (Windows msys2 bash).

set -euo pipefail

# cwd-independent (xem architect-guard.sh): bind to repo root regardless of caller cwd.
cd "${CLAUDE_PROJECT_DIR:-$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)}" || exit 0

# Read tool input JSON from stdin
INPUT_JSON=$(cat)

# Extract file_path (Edit + Write both use tool_input.file_path)
PATH_ARG=$(echo "$INPUT_JSON" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

# Unparseable path → allow (don't block on weird input)
[ -z "$PATH_ARG" ] && exit 0

NORMALIZED_PATH="${PATH_ARG#./}"

# Is this PRODUCT source? (narrow — see header). Anything else (docs/*.py/*.sh/config) → allow.
case "$NORMALIZED_PATH" in
    *.swift|*.pbxproj|src/*|*/src/*) ;;   # product source → gated below
    *) exit 0 ;;                          # not product source → always allow
esac

# Product source: allowed ONLY while a Worker is active (worker-active marker present).
[ -f ".sos-state/worker-active" ] && exit 0

# No worker-active → main session / Architect trying to hand-code product → BLOCK.
cat >&2 <<EOF
🚫 Orchestrator envelope violation

Quản đốc (main session) không được tự sửa product code: $PATH_ARG

Đúng quy trình: spawn Thợ (Worker) để code. Trước khi spawn:
  mkdir -p .sos-state && touch .sos-state/worker-active
Sau khi Thợ về:
  rm -f .sos-state/worker-active

Nếu đây ĐÚNG là Thợ đang EXECUTE mà bị chặn → Quản đốc quên touch marker trước spawn.
(Edit này KHÔNG phải product source? Báo — scope có thể cần chỉnh trong orchestrator-guard.sh.)
EOF
exit 2
