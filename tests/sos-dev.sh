#!/usr/bin/env bash
# `sos-dev` (crates/sos-dev) against the tools it replaces, which stay beside it:
#   docs-check    — tests/docs-check.py and `sos-dev docs-check` print the same problem list, on
#                   this tree and on a copy with planted problems;
#   trust-gate    — the same surface list as scripts/trust-gate.sh's SURFACE_GLOBS expansion, the
#                   same verdict on this tree, and both block a planted hidden code point.
# Known, documented differences (CHANGELOG, Unreleased) are applied before comparing:
#   - Python on Windows builds the "is this file under harness-lite/" test with a backslash path,
#     so a harness-lite/ shortcut path in a nested file (harness-lite/roles/*.md) is reported there;
#     sos-dev resolves it as on macOS and Linux. Python on Windows also prints the file path with
#     backslashes (docs\x.md:3: …); sos-dev prints forward slashes everywhere.
#   - The old unicode scan is a no-op when ripgrep is on PATH (rg rejects the non-UTF-8 tag-block
#     pattern, the error is swallowed); it is run here with rg hidden.
# Usage: tests/sos-dev.sh [path/to/sos] [path/to/sos-dev]
set -uo pipefail
KIT=$(cd "$(dirname "$0")/.." && pwd)
TGT=${CARGO_TARGET_DIR:-$KIT/target}
SOS=${1:-$TGT/debug/sos}
DEV=${2:-$TGT/debug/sos-dev}
[ -x "$SOS" ] || { echo "FAIL no sos binary at $SOS (cargo build -p sos-cli, or pass its path)"; exit 1; }
[ -x "$DEV" ] || { echo "FAIL no sos-dev binary at $DEV (cargo build -p sos-dev, or pass its path)"; exit 1; }
SOS=$(cd "$(dirname "$SOS")" && pwd)/$(basename "$SOS")
DEV=$(cd "$(dirname "$DEV")" && pwd)/$(basename "$DEV")
PY=$(for p in python3 python; do "$p" -c '' >/dev/null 2>&1 && { echo "$p"; break; }; done)
export PYTHONUTF8=1 LC_ALL=C
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) WIN=1 ;; *) WIN=0 ;; esac
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
expect() { if [ "$2" = 0 ]; then pass=$((pass + 1)); echo "ok   $1"; else fail=$((fail + 1)); echo "FAIL $1"; x=${3:-}; printf '%s\n' "${x:0:2000}"; fi; }
same() { # same <label> <file-a> <file-b>
    local d; d=$(diff "$2" "$3"); expect "$1" $? "$d"
}

# Python's report with the Windows-only difference taken out.
py_docs() { # py_docs <root> <out>
    (cd "$1" && "$PY" tests/docs-check.py "$SOS") > "$2.raw"; local rc=$?
    if [ "$WIN" = 1 ]; then
        awk -F: 'BEGIN { OFS = ":" } { gsub(/\\/, "/", $1); print }' "$2.raw" \
            | grep -v '^harness-lite/roles/.*: path not found `\(templates\|hooks\)/' > "$2.f"
        local n; n=$(grep -vc '^docs-check: ' "$2.f")
        sed -E "s/^(docs-check: [0-9]+ files, )[0-9]+ problem/\\1$n problem/" "$2.f" > "$2"
        rc=$([ "$n" = 0 ] && echo 0 || echo 1)
    else
        cp "$2.raw" "$2"
    fi
    return "$rc"
}

# ── docs-check on this tree ──
py_docs "$KIT" "$T/py.txt"; prc=$?
(cd "$KIT" && "$DEV" docs-check "$SOS") > "$T/rs.txt"; rrc=$?
same "docs-check: same report on this tree" "$T/py.txt" "$T/rs.txt"
expect "docs-check: same verdict on this tree (py=$prc rs=$rrc)" $([ "$prc" = "$rrc" ]; echo $?)

# ── a copy of this tree (tracked + untracked, not ignored), committed in a fresh repo ──
C=$T/copy; mkdir -p "$C"
(cd "$KIT" && git ls-files -z --cached --others --exclude-standard | while IFS= read -r -d '' f; do [ -e "$f" ] && printf '%s\0' "$f"; done | tar --null -T - -cf -) | tar -xf - -C "$C"
(cd "$C" && git init -q && git config user.email t@example.com && git config user.name t && git config core.autocrlf false && git add -A && git -c core.hooksPath=.no-hooks commit -qm copy) \
    || { echo "FAIL could not build the copy"; exit 1; }

# ── surfaces: the old SURFACE_GLOBS expansion vs `sos-dev trust-gate list` ──
eval "$(sed -n '/^SURFACE_GLOBS=(/,/^)/p' "$KIT/scripts/trust-gate.sh")"
(cd "$C" && git ls-files ${SURFACE_GLOBS[@]} 2>/dev/null | sort) > "$T/surf-old.txt"
(cd "$C" && "$DEV" trust-gate list | sort) > "$T/surf-new.txt"
same "trust-gate: same surface list ($(wc -l < "$T/surf-new.txt" | tr -d ' ') files)" "$T/surf-old.txt" "$T/surf-new.txt"
[ -s "$T/surf-new.txt" ]; expect "trust-gate: surface list not empty" $?

# A file named with a wildcard: bash expands the glob to that name and git reads it as a pattern,
# which widens the match to the dotfile beside it. Windows file names cannot hold `*` (Git Bash
# stores it as U+F02A and hands git a `*` again), so the case runs on Unix and WSL only.
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) win=1 ;; *) win=0 ;; esac
n=0
for name in '*.yml' '[[:punct:]]*.yml'; do
    n=$((n + 1)); G=$T/glob$n; mkdir -p "$G/.github/workflows" "$G/scripts"
    if [ "$win" = 0 ] && (: > "$G/.github/workflows/$name") 2>/dev/null && [ -e "$G/.github/workflows/$name" ]; then
        : > "$G/.github/workflows/.hidden.yml"; printf '#!/bin/sh\n' > "$G/scripts/a.sh"
        (cd "$G" && git init -q && git add -A)
        (cd "$G" && git ls-files ${SURFACE_GLOBS[@]} 2>/dev/null | sort) > "$T/glob-old.txt"
        (cd "$G" && "$DEV" trust-gate list | sort) > "$T/glob-new.txt"
        same "trust-gate: same list with a file named $name" "$T/glob-old.txt" "$T/glob-new.txt"
        grep -qx '.github/workflows/.hidden.yml' "$T/glob-new.txt"; expect "trust-gate: $name name widens to the dotfile" $?
    else
        echo "skip trust-gate: no file named $name on this system"
    fi
done

# The old script with ripgrep hidden from PATH (see the header), or skipped when rg sits in a
# system folder that cannot be dropped.
NORG_PATH=""; RG_OK=1
IFS=: read -r -a dirs <<< "$PATH"
for d in "${dirs[@]}"; do
    if [ -x "$d/rg" ] || [ -x "$d/rg.exe" ]; then
        case "$d" in /bin|/usr/bin) RG_OK=0 ;; *) continue ;; esac
    fi
    NORG_PATH="${NORG_PATH:+$NORG_PATH:}$d"
done
old_gate() { (cd "$C" && unset CLAUDE_PROJECT_DIR && PATH="$NORG_PATH" bash scripts/trust-gate.sh "$@"); }

old_gate >/dev/null 2>"$T/old.err"; orc=$?
(cd "$C" && "$DEV" trust-gate) >/dev/null 2>"$T/new.err"; nrc=$?
expect "trust-gate: same verdict on this tree (old=$orc new=$nrc)" $([ "$orc" = "$nrc" ]; echo $?) "$(cat "$T/old.err" "$T/new.err")"

# A surface whose name is not UTF-8, beside a good one: both block it (the old script hands the
# name to git and lists it as Added; sos-dev stops on the name). macOS and Windows refuse such
# names (Git Bash stores the byte as U+00FF, a valid name), so this runs on Linux and WSL only.
N=$T/nonutf8; mkdir -p "$N/scripts"; cp "$KIT/scripts/trust-gate.sh" "$N/scripts/"; printf 'echo a\n' > "$N/scripts/a.sh"
(cd "$N" && git init -q && git config user.email t@example.com && git config user.name t && git add -A && "$DEV" trust-gate rebaseline >/dev/null 2>&1 && git add -A)
(cd "$N" && unset CLAUDE_PROJECT_DIR && PATH="$NORG_PATH" bash scripts/trust-gate.sh) >/dev/null 2>"$T/old.err"
expect "trust-gate (old): passes the clean non-UTF-8 fixture" $? "$(cat "$T/old.err")"
if [ "$WIN" = 0 ] && (printf 'echo b\n' > "$N/scripts/$(printf '\377').sh") 2>/dev/null && [ -e "$N/scripts/$(printf '\377').sh" ]; then
    (cd "$N" && git add -A)
    (cd "$N" && unset CLAUDE_PROJECT_DIR && PATH="$NORG_PATH" bash scripts/trust-gate.sh) >/dev/null 2>"$T/old.err"; orc=$?
    (cd "$N" && "$DEV" trust-gate) >/dev/null 2>"$T/new.err"; nrc=$?
    expect "trust-gate: both block a non-UTF-8 surface name (old=$orc new=$nrc)" $([ "$orc" = 1 ] && [ "$nrc" = 1 ] && grep -q 'not UTF-8' "$T/new.err"; echo $?) "$(cat "$T/old.err" "$T/new.err")"
else
    echo "skip trust-gate: this file system refuses a non-UTF-8 name"
fi

# ── planted problems ──
printf '\nParity: [gone](docs/nope.md) `scripts/nope.sh` `sos frobnicate` `sos gate nope` `crates/none/x.rs`, `harness-lite/scripts/env-guard.sh` `harness-lite/scripts/nope.sh`\nbefore\xe2\x80\xa8`scripts/after-ls.sh` and \x0c`docs/after-ff.md`; `.sos.toml` `docs/STATE.md` `foo.txt` `not a/path`\n' >> "$C/README.md"
printf '\nShortcut: `scripts/env-guard.sh` `templates/AGENTS.md` `hooks/pre-commit` `scripts/nope-hl.sh`\n' >> "$C/harness-lite/README.md"
printf '\nShortcut: `templates/AGENTS.md` `scripts/nope-roles.sh`\n' >> "$C/harness-lite/roles/worker.md"
printf 'CRLF\r\n`scripts/crlf-missing.sh`\r\nlone CR\r`scripts/cr-missing.sh`\r\n[x](./sos-dev-parity-gone.md)\r\n' > "$C/docs/sos-dev-parity.md"
printf '`scripts/plans-only.sh`\n' >> "$C/docs/plans/RUST_ALL_TOOLS_2026-11-04.md"
(cd "$C" && git add -A)
py_docs "$C" "$T/py2.txt"; prc=$?
(cd "$C" && "$DEV" docs-check "$SOS") > "$T/rs2.txt"; rrc=$?
same "docs-check: same report with planted problems" "$T/py2.txt" "$T/rs2.txt"
expect "docs-check: planted problems found (py=$prc rs=$rrc)" $([ "$prc" = 1 ] && [ "$rrc" = 1 ]; echo $?)
for want in 'broken link docs/nope.md' 'unknown command `sos frobnicate`' 'unknown gate `sos gate nope`' \
            'path not found `crates/none/x.rs`' 'path not found `harness-lite/scripts/nope.sh`' \
            'path not found `scripts/after-ls.sh`' 'path not found `docs/after-ff.md`' \
            'harness-lite/README.md:' 'harness-lite/roles/worker.md:' 'docs/sos-dev-parity.md:2: path not found `scripts/crlf-missing.sh`' \
            'docs/sos-dev-parity.md:4: path not found `scripts/cr-missing.sh`' 'docs/sos-dev-parity.md:5: broken link ./sos-dev-parity-gone.md'; do
    grep -qF "$want" "$T/rs2.txt"; expect "docs-check: reports $want" $?
done
for not in 'plans-only' '.sos.toml' 'STATE.md' 'foo.txt' 'not a/path' 'templates/AGENTS.md' 'env-guard.sh'; do
    ! grep -qF "$not" "$T/rs2.txt"; expect "docs-check: does not report $not" $?
done

# Hidden code point: both block it.
printf 'zero\xe2\x80\x8bwidth\n' >> "$C/README.md"
(cd "$C" && "$DEV" trust-gate) >/dev/null 2>"$T/new.err"; nrc=$?
expect "trust-gate (new): blocks U+200B" $([ "$nrc" = 1 ] && grep -q 'README.md:[0-9]*:5: U+200B zero width space' "$T/new.err"; echo $?) "$(cat "$T/new.err")"
if [ "$RG_OK" = 1 ]; then
    old_gate >/dev/null 2>"$T/old.err"; orc=$?
    expect "trust-gate (old, rg hidden): blocks U+200B" $([ "$orc" = 1 ] && grep -q 'hidden Unicode' "$T/old.err"; echo $?) "$(cat "$T/old.err")"
else
    echo "skip trust-gate (old): rg is in a system folder, cannot hide it"
fi
# A tag character, which the old script can only see without rg.
printf 'tag\xf3\xa0\x81\x81\n' >> "$C/docs/sos-dev-parity.md"
(cd "$C" && "$DEV" trust-gate) >/dev/null 2>"$T/new.err"
grep -q 'docs/sos-dev-parity.md:[0-9]*:4: U+E0041 tag character' "$T/new.err"; expect "trust-gate (new): blocks U+E0041" $? "$(cat "$T/new.err")"

echo "sos-dev parity: $pass passed, $fail failed"
[ "$fail" = 0 ]
