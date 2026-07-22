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
#   - `sync` (P077c2 re-froze) no longer depends on real sos-kit HEAD: it builds a
#     SYNTHETIC self-contained fake-kit git repo (own history, own commits) and
#     points SOS_KIT_DIR at that instead — see tests/README.md "sos sync — synthetic
#     fake-kit fixture" section. Two goldens: `sync.golden` (stdout report) and
#     `sync.tree.golden` (post-sync file-manifest, sorted, order-independent).

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

run_isolated_kit() {  # <override-kit-dir> <fn-name> <args...>
  # Same as run_isolated, but SOS_KIT_DIR points at a caller-supplied dir
  # (used by `sync`'s synthetic fake-kit fixture) instead of the real $KIT.
  # bin/sos.sh itself is still sourced from the real $KIT (canonical script,
  # never edited) — only the *provenance oracle* dir is overridden.
  local override_kit="$1"; shift
  bash -c '
    export SOS_KIT_DIR="'"$override_kit"'"
    source "'"$KIT"'/bin/sos.sh"
    "$@"
  ' _ "$@"
}

build_fake_kit() {  # <fake-kit-dir> -> git init + 2-commit history, minimal spine
  # Synthetic self-contained fake-kit (P077c2): decouples sync.golden from
  # real sos-kit HEAD (see tests/README.md). Exercises all 4 sync outcomes:
  #   ADDED      — present in fake-kit HEAD, absent at target
  #   UPDATED    — target has fake-kit's v1 blob (historical), HEAD is v2
  #   FLAGGED    — target has content matching NO historical blob
  #   ALREADY-CURRENT — target byte-identical to fake-kit HEAD
  local kit="$1"
  mkdir -p "$kit"
  (
    cd "$kit"
    git init -q
    git config user.email "sos-kit-fixture@example.com"
    git config user.name "sos-kit fixture"
    mkdir -p scripts phieu templates .claude/agents .claude/commands skills/demo agents
    echo "kit content v1" > scripts/updated.sh   # will get a v2 below
    echo "kit content"    > scripts/added.sh
    echo "kit content"    > scripts/flagged.sh
    echo "kit content"    > scripts/current.sh
    echo "# demo skill"   > skills/demo/SKILL.md
    echo "# demo agent"   > agents/demo.md
    git add -A
    git commit -q -m "v1"
    echo "kit content v2" > scripts/updated.sh
    git add -A
    git commit -q -m "v2"
  )
}

build_sync_target() {  # <target-dir>
  local tgt="$1"
  mkdir -p "$tgt/scripts" "$tgt/.claude/agents"
  echo "kit content v1"                      > "$tgt/scripts/updated.sh"  # stale v1 -> UPDATED
  echo "custom content, never seen by kit"   > "$tgt/scripts/flagged.sh"  # unknown blob -> FLAGGED
  echo "kit content"                         > "$tgt/scripts/current.sh" # matches HEAD -> ALREADY-CURRENT
  # scripts/added.sh, agents/demo.md, skills/demo/SKILL.md absent at target -> ADDED
}

sha256_of() {  # <path> -> sha256 hex (portable across macOS/Linux)
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

freeze_sync_tree() {  # <target-dir> -> writes sorted "<verb> <relpath> <sha256>" manifest to stdout
  local tgt="$1" f rel
  {
    for f in "$tgt/scripts/added.sh" "$tgt/.claude/agents/demo.md" "$tgt/.claude/skills/demo/SKILL.md"; do
      [[ -f "$f" ]] || continue
      rel="${f#"$tgt"/}"
      echo "ADDED $rel $(sha256_of "$f")"
    done
    f="$tgt/scripts/updated.sh"
    [[ -f "$f" ]] && echo "UPDATED scripts/updated.sh $(sha256_of "$f")"
    f="$tgt/.sos-sync-incoming/scripts/flagged.sh"
    [[ -f "$f" ]] && echo "INCOMING .sos-sync-incoming/scripts/flagged.sh $(sha256_of "$f")"
  } | sort
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

echo "== capturing sync (synthetic fake-kit fixture, P077c2 re-froze) =="
fake_kit="$WORK/sync-fake-kit"
tgt="$WORK/sync-fixture"
build_fake_kit "$fake_kit"
build_sync_target "$tgt"
run_isolated_kit "$fake_kit" sos_sync "$tgt" | normalize "$tgt" | sed -e "s#${fake_kit}#<SOS_KIT_DIR>#g" > "$OUT/sync.golden"
freeze_sync_tree "$tgt" > "$OUT/sync.tree.golden"

echo "Done. Diff against committed tests/golden/*.golden to verify reproducibility:"
echo "  diff $OUT/new.golden <(committed new.golden)   (etc.)"
