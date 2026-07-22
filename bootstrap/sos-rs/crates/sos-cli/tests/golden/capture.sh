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
#   - `new` (P077c3 re-froze) also uses a SYNTHETIC minimal fake-kit — simpler than
#     sync's (plain directory tree, no git history needed, since `new` only COPIES
#     kit assets verbatim). Also forces `DOCTOR_BIN=/nonexistent/doctor` so [4/4]'s
#     verify-setup branch is deterministic regardless of whether `doctor` happens to
#     be installed on the machine running capture.sh (the PREVIOUS `new.golden` was
#     accidentally captured on the CONNECTED-path — a host artifact, not intentional
#     — see tests/README.md "sos new — synthetic fake-kit + doctor-absent" section).
#     Three goldens: `new.golden` (stdout), `new.tree.golden` (path-shape manifest,
#     sorted, no content, excludes `.git/`), `new.gen.golden` (content-hash manifest,
#     GENERATED-authored files only — copied kit assets are tree-only, never hashed,
#     to avoid kit-content-coupling).
#   - `adopt` (P077c4 re-froze) uses a synthetic fake-kit (same 2-gotcha shape as
#     `new`'s, PLUS a real `templates/claude-settings.local.json`) + a BROWNFIELD
#     target seeded with 4 collision cases (absent/spine-collision/doc-existing/
#     source-non-spine), committed clean (no dirty-warn), `DOCTOR_BIN=/nonexistent`.
#     The PREVIOUS `adopt.golden` (P077a) was captured on the CONNECTED doctor path
#     (`[WIRED] J1..J6` + `validate-map: paths resolve`) — a host artifact, same
#     class as `new`'s pre-c3 golden — re-froze under doctor-absent. Four "goldens":
#     `adopt.golden` (stdout), `adopt.tree.golden` (path-shape, sorted, excludes
#     `.git/`, INCLUDES `.sos-adopt-incoming/**`), `adopt.gen.golden` (content-hash,
#     GENERATED-authored only), and an in-test-only preservation-assert (NOT a
#     golden file — universal property: pre-existing seeded files' sha256 must be
#     identical before/after, `.sos-adopt-incoming/<path>` must byte-match kit
#     source — see tests/README.md "sos adopt" section + parity.rs).

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

strip_timestamp() {  # reads stdin, writes stdout with full ISO-8601 timestamps
  # (e.g. `sos_init_security`'s `.sos-stack.toml` `detected_at = "2026-07-22T14:23:01Z"`)
  # collapsed to <TIMESTAMP>. MUST run BEFORE normalize()'s bare-date rule, else the
  # date portion gets replaced but the time-of-day suffix (`T14:23:01Z`) leaks through
  # non-deterministic (anchor #10, P077c3).
  sed -e 's/[0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}T[0-9]\{2\}:[0-9]\{2\}:[0-9]\{2\}Z/<TIMESTAMP>/g'
}

sort_todo_block() {
  # `new`'s "Category C placeholders to fill (# TODO):" list is emitted via
  # `grep -rl` (filesystem order, not sorted) — sort just that block so the
  # fixture doesn't flap between capture runs.
  awk '
    /Category C placeholders to fill/ { print; in_block=1; next }
    in_block && /^    - / { buf[n++]=$0; next }
    in_block && !/^    - / { for (i=0;i<n;i++) print buf[i] | "LC_ALL=C sort"; close("LC_ALL=C sort"); in_block=0 }
    { print }
    END { if (in_block) for (i=0;i<n;i++) print buf[i] | "LC_ALL=C sort" }
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

run_isolated_kit_doctor_absent() {  # <override-kit-dir> <fn-name> <args...>
  # Same as run_isolated_kit, but ALSO forces DOCTOR_BIN to a nonexistent path so
  # [4/4]'s verify-setup branch is deterministic regardless of whether `doctor`
  # happens to be installed on the machine running capture.sh (P077c3).
  local override_kit="$1"; shift
  bash -c '
    export SOS_KIT_DIR="'"$override_kit"'"
    export DOCTOR_BIN=/nonexistent/doctor
    source "'"$KIT"'/bin/sos.sh"
    "$@"
  ' _ "$@"
}

build_fake_kit_new() {  # <fake-kit-dir> -> minimal plain-tree fake-kit for `new` (P077c3).
  # Unlike sync's fake-kit (needs real git history as a provenance oracle), `new`
  # only COPIES kit assets verbatim -- no git semantics needed, so this is a plain
  # directory tree, no `git init`. Covers every "$K/..." path `sos_new` reads
  # (bin/sos.sh:404-565).
  local kit="$1"
  mkdir -p "$kit"/.claude/agents "$kit"/.claude/commands "$kit"/agents \
           "$kit"/templates "$kit"/scripts "$kit"/phieu "$kit"/hooks \
           "$kit"/skills/idea "$kit"/skills/attic/dummy
  echo "# fake worker agent"                          > "$kit/.claude/agents/worker.md"
  echo '{"fake":"settings"}'                           > "$kit/.claude/settings.json"
  echo "# fake idea command"                           > "$kit/.claude/commands/idea.md"
  echo "# fake orchestrator"                           > "$kit/agents/orchestrator.md"
  echo '{"fake":"local-settings"}'                      > "$kit/templates/claude-settings.local.json"
  echo "# INVARIANTS template"                         > "$kit/templates/INVARIANTS-template.md"
  echo "# BACKLOG template"                            > "$kit/templates/BACKLOG_template.md"
  echo "# fake idea skill"                             > "$kit/skills/idea/SKILL.md"
  echo "# fake attic skill (must be SKIPPED by new)"   > "$kit/skills/attic/dummy/SKILL.md"
  echo "fake phieu readme"                             > "$kit/phieu/README.md"
  echo "fake-pre-commit"                                > "$kit/hooks/pre-commit"
  echo "fake-pre-push"                                  > "$kit/hooks/pre-push"
  echo ".DS_Store"                                      > "$kit/.gitignore"
  # install-hooks.sh must actually FUNCTION -- `new`'s [+] git step shells out to
  # it (git config core.hooksPath etc). Reuse the REAL script: its content is
  # never gen-hashed/frozen (path-only, tree-golden), so this is not
  # kit-content-coupling -- only functional correctness of the fixture.
  cp "$KIT/scripts/install-hooks.sh" "$kit/scripts/install-hooks.sh"
  chmod +x "$kit/scripts/install-hooks.sh"
}

# GENERATED-authored files `sos_new` writes (bin/sos.sh Category C + defaults) --
# content-hashed into new.gen.golden. Copied kit assets are NEVER in this list
# (tree-shape only, kit-content-coupling guard -- see tests/README.md).
NEW_GEN_FILES=(
  ".mcp.json"
  "docs/security/INVARIANTS.md"
  "docs/AGENT_MAP.yaml"
  "CLAUDE.md"
  "docs/ARCHITECTURE.md"
  "CHANGELOG.md"
  "pyproject.toml"
  ".docs-gate.toml"
  ".sos-stack.toml"
)

freeze_new_tree() {  # <target-dir> -> sorted path-shape manifest, no content, excludes .git/
  # LC_ALL=C: byte-value sort, locale-independent (a locale-aware `sort` on
  # macOS/Linux can order e.g. "claude" before "INVARIANTS" case-insensitively,
  # which then mismatches Rust's plain byte-order `Vec<String>::sort()` — this
  # bit us during P077c3 EXECUTE, fixed by pinning both sides to byte-order).
  local tgt="$1"
  find "$tgt" -not -path '*/.git*' | sed "s#^${tgt}/\{0,1\}##" | sed '/^$/d' | LC_ALL=C sort
}

freeze_new_gen() {  # <target-dir> -> sorted "<relpath> <sha256>" manifest, GENERATED-authored only
  local tgt="$1" f
  {
    for f in "${NEW_GEN_FILES[@]}"; do
      [[ -f "$tgt/$f" ]] || continue
      local hash
      hash="$(cat "$tgt/$f" | strip_timestamp | normalize "$tgt" | sha256_of_stdin)"
      echo "$f $hash"
    done
  } | LC_ALL=C sort
}

build_fake_kit_adopt() {  # <fake-kit-dir> -> minimal plain-tree fake-kit for `adopt` (P077c4).
  # Same shape as `new`'s fake-kit (build_fake_kit_new), PLUS
  # templates/claude-settings.local.json (2nd fixture gotcha found during
  # Worker CHALLENGE Turn 1 live-probe: [1/4]/[3/4] error without it).
  local kit="$1"
  build_fake_kit_new "$kit"
  echo '{"permissions":{"allow":[]}}' > "$kit/templates/claude-settings.local.json"
}

# GENERATED-authored files `sos_adopt` writes when absent (bin/sos.sh:806-900) --
# content-hashed into adopt.gen.golden. Copied/staged kit assets are NEVER in this
# list (tree-shape only + preservation-assert, not gen-hash -- kit-content-coupling
# guard, same rationale as NEW_GEN_FILES).
ADOPT_GEN_FILES=(
  ".mcp.json"
  "docs/security/INVARIANTS.md"
  "docs/AGENT_MAP.yaml"
  ".docs-gate.toml"
  "CHANGELOG.md"
  "docs/ARCHITECTURE.md"
  ".gitignore"
  ".sos-stack.toml"
)

build_adopt_brownfield() {  # <target-dir> -> brownfield target, 4 collision-case (P077c4)
  local tgt="$1"
  mkdir -p "$tgt/templates" "$tgt/docs" "$tgt/src/routes" "$tgt/src/models"
  # (a) spine ABSENT -- no seed needed, adopt_item will ADD.
  # (b) spine COLLISION -- same path as a kit spine item, DIFFERENT content ->
  #     must be STAGED to .sos-adopt-incoming/, target file itself UNCHANGED.
  echo "custom INVARIANTS content — brownfield repo's own, never seen by kit" > "$tgt/templates/INVARIANTS-template.md"
  # (c) Cat-C doc EXISTING -- generate must SKIP (never overwrite).
  echo "# my own changelog — pre-existing, never touched by adopt" > "$tgt/CHANGELOG.md"
  # (d) source non-spine -- untouched by adopt, but SCANNED by the map-within-adopt
  #     call (OA-02 bug material -- feeds AGENT_MAP.yaml real surfaces).
  echo "def h(): pass" > "$tgt/src/routes/api.py"
  echo "class M: pass" > "$tgt/src/models/user.py"
  echo "[tool.poetry]" > "$tgt/pyproject.toml"
  ( cd "$tgt" && git init -q && git config user.email "sos-kit-fixture@example.com" \
    && git config user.name "sos-kit fixture" && git add -A && git commit -q -m seed )
}

freeze_adopt_tree() {  # <target-dir> -> sorted path-shape manifest, INCLUDES .sos-adopt-incoming/**, excludes .git/
  local tgt="$1"
  find "$tgt" -not -path '*/.git*' | sed "s#^${tgt}/\{0,1\}##" | sed '/^$/d' | LC_ALL=C sort
}

freeze_adopt_gen() {  # <target-dir> -> sorted "<relpath> <sha256>" manifest, GENERATED-authored only
  local tgt="$1" f
  {
    for f in "${ADOPT_GEN_FILES[@]}"; do
      [[ -f "$tgt/$f" ]] || continue
      local hash
      hash="$(cat "$tgt/$f" | strip_timestamp | normalize "$tgt" | sha256_of_stdin)"
      echo "$f $hash"
    done
  } | LC_ALL=C sort
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

sha256_of_stdin() {  # reads stdin -> sha256 hex (portable across macOS/Linux)
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print $1}'
  else
    shasum -a 256 | awk '{print $1}'
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

echo "== capturing new (synthetic fake-kit + doctor-absent, P077c3 re-froze) =="
fake_kit_new="$WORK/new-fake-kit"
build_fake_kit_new "$fake_kit_new"
tgt="$WORK/new-fixture"
run_isolated_kit_doctor_absent "$fake_kit_new" sos_new "$tgt" --stack python \
  | strip_timestamp | normalize "$tgt" | sed "s#${fake_kit_new}#<SOS_KIT_DIR>#g" | sort_todo_block > "$OUT/new.golden"
freeze_new_tree "$tgt" > "$OUT/new.tree.golden"
freeze_new_gen "$tgt" > "$OUT/new.gen.golden"

echo "== capturing adopt (synthetic fake-kit + 4-collision brownfield + doctor-absent, P077c4 re-froze) =="
fake_kit_adopt="$WORK/adopt-fake-kit"
build_fake_kit_adopt "$fake_kit_adopt"
tgt="$WORK/adopt-fixture"
build_adopt_brownfield "$tgt"
run_isolated_kit_doctor_absent "$fake_kit_adopt" sos_adopt "$tgt" --stack python \
  | strip_timestamp | normalize "$tgt" | sed "s#${fake_kit_adopt}#<SOS_KIT_DIR>#g" > "$OUT/adopt.golden"
freeze_adopt_tree "$tgt" > "$OUT/adopt.tree.golden"
freeze_adopt_gen "$tgt" > "$OUT/adopt.gen.golden"

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
