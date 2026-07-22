#!/usr/bin/env bash
# capture.sh — freeze bin/sos.sh output for new/adopt/map/sync into golden fixtures.
#
# P077a Task 2: bin/sos.sh is the ORACLE — this script only READS it, never edits it.
# Run this script to REPRODUCE the committed .golden files (reproducibility check),
# or to RE-FREEZE them if bin/sos.sh's canonical behavior legitimately changes
# (that change itself would be its own phiếu — .golden is not meant to be re-frozen
# casually; a diff here without a corresponding bin/sos.sh phiếu is a regression signal).
#
# Usage:
#   SOS_KIT_DIR=/path/to/sos-kit bash tests/golden/capture.sh [output-dir]
#
# Determinism notes (see tests/README.md for full detail):
#   - stdout only (no exit code capture needed for these 4 — none exercised a non-zero path)
#   - absolute paths (kit dir + fixture target dir) are normalized to <SOS_KIT_DIR> / <TARGET>
#   - dates (YYYY-MM-DD) are normalized to <DATE>
#   - `new`'s "Category C placeholders to fill" block is filesystem-enumeration-order
#     dependent (grep -rl, unsorted) -> normalizer sorts that block before diff/freeze
#   - `sync` additionally depends on sos-kit's OWN git history (see tests/README.md
#     "sync HEAD-pin" section) — reproducible ONLY if SOS_KIT_DIR is checked out at the
#     same commit the fixture was frozen against.

set -euo pipefail

KIT="${SOS_KIT_DIR:?set SOS_KIT_DIR to the sos-kit checkout}"
OUT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

normalize() {  # <target-abs-path> -> reads stdin, writes normalized stdout
  local target="$1"
  sed -e "s#${target}#<TARGET>#g" \
      -e "s#${KIT}#<SOS_KIT_DIR>#g" \
      -e 's/[0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}/<DATE>/g'
}

sort_todo_block() {
  # `new`'s "Category C placeholders to fill (# TODO):" list is emitted via
  # `grep -rl` (filesystem order, not sorted) — sort just that block so the
  # fixture doesn't flap between capture runs.
  awk '
    /Category C placeholders to fill/ { print; in_block=1; next }
    in_block && /^    - / { buf[n++]=$0; next }
    in_block && !/^    - / { for (i=0;i<n;i++) print buf[i] | "sort"; close("sort"); in_block=0 }
    { print }
    END { if (in_block) for (i=0;i<n;i++) print buf[i] | "sort" }
  '
}

run_isolated() {  # <fn-name> <args...> -> stdout+stderr, set -e contained to subshell
  bash -c '
    export SOS_KIT_DIR="'"$KIT"'"
    source "'"$KIT"'/bin/sos.sh"
    "$@"
  ' _ "$@"
}

echo "== capturing new =="
tgt="$WORK/new-fixture"
run_isolated sos_new "$tgt" --stack python | normalize "$tgt" | sort_todo_block > "$OUT/new.golden"

echo "== capturing adopt =="
tgt="$WORK/adopt-fixture"
mkdir -p "$tgt/src"
echo "print('hi')" > "$tgt/src/app.py"
run_isolated sos_adopt "$tgt" --stack python | normalize "$tgt" > "$OUT/adopt.golden"

echo "== capturing map =="
tgt="$WORK/map-fixture"
mkdir -p "$tgt/src/routes" "$tgt/src/models"
echo "def h(): pass" > "$tgt/src/routes/api.py"
echo "class M: pass" > "$tgt/src/models/user.py"
run_isolated sos_map "$tgt" | normalize "$tgt" > "$OUT/map.golden"
# P077c1 (additive): map's REAL work-product is the file it writes, not just
# the 1-line stdout confirmation above. Freeze that file's content too, so
# the parity harness isn't blind to scan-correctness (see Debate Log V2).
cat "$tgt/docs/AGENT_MAP.yaml" | normalize "$tgt" > "$OUT/map.agent_map.golden"

echo "== capturing sync (reuses adopt fixture as adopted-repo target) =="
tgt="$WORK/sync-fixture"
cp -r "$WORK/adopt-fixture" "$tgt"
run_isolated sos_sync "$tgt" | normalize "$tgt" > "$OUT/sync.golden"

echo "Done. Diff against committed tests/golden/*.golden to verify reproducibility:"
echo "  diff $OUT/new.golden <(committed new.golden)   (etc.)"
