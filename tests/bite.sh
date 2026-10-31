#!/usr/bin/env bash
# scripts/bite on a throwaway Python repo whose change has one good test, one tautology and one
# unchecked branch: each must show up where it belongs. Usage: tests/bite.sh
set -uo pipefail
KIT=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
expect() { if [ "$2" = 0 ]; then pass=$((pass + 1)); echo "ok   $1"; else fail=$((fail + 1)); echo "FAIL $1: ${3:0:400}"; fi; }
g() { git -c user.name=t -c user.email=t@t "$@"; }
# Windows: `python3` is often the Store alias that only fails; use the interpreter that runs.
PY=$(for p in python3 python; do "$p" -c '' >/dev/null 2>&1 && { echo "$p"; break; }; done)
export PYTHONUTF8=1

cd "$T" && g init -q -b main && mkdir tests
printf 'def clamp(x):\n    return x\n' > calc.py
printf 'import unittest\nimport calc\n\n\nclass Old(unittest.TestCase):\n    def test_identity(self):\n        self.assertEqual(calc.clamp(3), 3)\n' > tests/test_calc.py
touch tests/__init__.py
g add -A && g commit -qm base
cat > calc.py <<'PY'
LIMIT = 10


def clamp(x, lo=0, hi=LIMIT):
    """Keep x in [lo, hi].

    x == lo is allowed: a docstring is not code.
    """
    if x < lo:  # below
        return lo
    if x > hi:
        return hi
    return x  # never x + 1: a comment is not code
PY
cat > tests/test_calc.py <<'PY'
import unittest
import calc


class Old(unittest.TestCase):
    def test_identity(self):
        self.assertEqual(calc.clamp(3), 3)


class Clamp(unittest.TestCase):
    def test_low_end(self):
        self.assertEqual(calc.clamp(-1), 0)

    def test_tautology(self):
        x = 3
        self.assertEqual(x, 3)
PY
g add -A && g commit -qm change
B="$PY $KIT/scripts/bite --test '$PY -m unittest discover -s tests -t . -v' --src '\.py$' --tests '^tests/' --base main~1"

o=$(eval "$B" 2>&1); rc=$?
expect "bite: measured" "$([ $rc = 0 ]; echo $?)" "rc=$rc $o"
grep -q "passes on old code: Clamp.test_tautology" <<<"$o"; expect "tautology passes on the old code" $? "$o"
! grep -q "passes on old code: Clamp.test_low_end" <<<"$o"; expect "the real test fails on the old code" $? "$o"
! grep -q "Old.test_identity" <<<"$o"; expect "untouched test not listed" $? "$o"
grep -q "survived: calc.py:1  LIMIT = 10  ->  LIMIT = 11" <<<"$o"; expect "unchecked default (hi=LIMIT) survives" $? "$o"
! grep -qE "survived: calc.py:(9|11)" <<<"$o"; expect "checked branches killed" $? "$o"
at=$("$PY" -c "import json,glob;print(sorted(m['at'] for m in json.load(open(sorted(glob.glob('evidence/bite/*.json'))[-1]))['mutants']))")
[ "$at" = "['calc.py:1', 'calc.py:11', 'calc.py:4', 'calc.py:9']" ]; expect "mutants on code lines only (not the docstring)" $? "$at"
[ -z "$(git status --porcelain --untracked-files=no)" ] && [ "$(git worktree list | wc -l | tr -d ' ')" = 1 ]
expect "checkout untouched, worktree removed" $? "$(git status --short; git worktree list)"

o=$("$PY" "$KIT/scripts/bite" 2>&1); rc=$?
[ $rc = 2 ] && grep -q "\[bite\]" <<<"$o"; expect "not configured -> exit 2 with the fix" $? "rc=$rc $o"
printf "[bite]\ntest = '$PY -m unittest discover -s tests -t . -v'\nsrc = '\\\\.py\$'\ntests = '^tests/'\nmutants = 1\n" > .sos.toml
o=$("$PY" "$KIT/scripts/bite" --base main~1 2>&1); rc=$?
[ $rc = 0 ] && grep -q "/1 mutants killed" <<<"$o"; expect ".sos.toml [bite] is read" $? "rc=$rc $o"
printf 'import unittest\n\n\nclass Red(unittest.TestCase):\n    def test_red(self):\n        self.fail("red")\n' > tests/test_red.py
g add -A && g commit -qm red
o=$(eval "$B~1" 2>&1); rc=$?
[ $rc = 1 ] && grep -q "red at HEAD" <<<"$o"; expect "red HEAD -> not measured" $? "rc=$rc $o"

o=$("$PY" "$KIT/tests/bite_parsers.py" "$KIT/scripts/bite" 2>&1)
[ "$o" = ok ]; expect "parsers: unittest (docstring), pytest, XCTest formats, test bounds and classes" $? "$o"

# A mutant that breaks the build is not a kill; base == head and an outside cwd are refused.
mkdir "$T/b2" && cd "$T/b2" && g init -q -b main
printf 'let xs = Array<Int>()\n' > app.swift
cat > run.sh <<'SH'
#!/usr/bin/env bash
if grep -q 'ready = false' app.swift; then echo "app.swift:2: error: expected type"; exit 1; fi
if grep -q 'm = 8' app.swift; then echo "Test Case '-[M.C testA]' passed (0.0 seconds)"; echo "error: Other.swift does not load"; exit 1; fi
if grep -q 'n = 4' app.swift; then echo "Test Case '-[M.C testA]' failed (0.0 seconds)"; exit 1; fi
echo "Test Case '-[M.C testA]' passed (0.0 seconds)"
SH
chmod +x run.sh && g add -A && g commit -qm base
printf 'let xs = Array<Int>()\nlet ready = true\nlet n = 3\n++ i == 3\nlet m = 7\n' > app.swift
g add -A && g commit -qm change
o=$("$PY" "$KIT/scripts/bite" --test ./run.sh --src 'app\.swift$' --tests 'Tests/' --base main~1 2>&1); rc=$?
[ $rc = 0 ] && grep -q "1/2 mutants killed" <<<"$o" && grep -q "not counted (no test ran, build broke?): app.swift:2" <<<"$o" \
  && grep -q "not counted (no test ran, build broke?): app.swift:5" <<<"$o"
expect "build-breaking mutant not counted as a kill, nor a red run where no test failed" $? "rc=$rc $o"
grep -q "survived: app.swift:4  ++ i == 3  ->  ++ i != 3" <<<"$o"; expect "an added line starting with ++ is read" $? "$o"
o=$("$PY" "$KIT/scripts/bite" --test ./run.sh --src x --tests y --base HEAD 2>&1); rc=$?
[ $rc = 1 ] && grep -q "same commit" <<<"$o"; expect "base == head -> not measurable" $? "rc=$rc $o"
o=$("$PY" "$KIT/scripts/bite" --test ./run.sh --src x --tests y --cwd /tmp 2>&1); rc=$?
[ $rc = 2 ]; expect "absolute cwd refused" $? "rc=$rc $o"
o=$("$PY" "$KIT/scripts/bite" --test 'xcodebuild test -scheme X' --src x --tests y --base main~1 2>&1); rc=$?
[ $rc = 2 ] && grep -q "parallel-testing-enabled NO" <<<"$o"; expect "xcodebuild test that would clone the simulator refused by sim-guard" $? "rc=$rc $o"
printf '#!/usr/bin/env bash\nsleep 20\n' > slow.sh && chmod +x slow.sh && g add -A && g commit -qm slow
case "$(uname -s)" in
MINGW*|MSYS*|CYGWIN*)  # Windows has no SIGTERM to catch: bash's kill on a native process is TerminateProcess
    echo "skip SIGTERM removes the worktree (Windows: no POSIX signals)" ;;
*)
    "$PY" "$KIT/scripts/bite" --test ./slow.sh --src app --tests y --base main~2 >/dev/null 2>&1 & bp=$!
    sleep 2; kill -TERM $bp; wait $bp 2>/dev/null
    [ "$(git worktree list | wc -l | tr -d ' ')" = 1 ]; expect "SIGTERM removes the worktree" $? "$(git worktree list)" ;;
esac

# node --test: per-test verdicts by title, and --audit on the tests that already exist.
mkdir "$T/js" && cd "$T/js" && g init -q -b main && mkdir test
printf 'export const clamp = (x) => x;\n' > clamp.mjs
printf "import test from 'node:test';\nimport assert from 'node:assert';\nimport { clamp } from '../clamp.mjs';\ntest('identity', () => assert.equal(clamp(3), 3));\n" > test/clamp.test.mjs
g add -A && g commit -qm base
cat > clamp.mjs <<'JS'
// x == lo is allowed: a comment is not code
const HI = 10;
export const clamp = (x, lo = 0) => {
  if (x < lo) return lo;
  if (x > HI) return HI;
  return x;
};
JS
cat > test/clamp.test.mjs <<'JS'
import test from 'node:test';
import assert from 'node:assert';
import { clamp } from '../clamp.mjs';
test('identity', () => assert.equal(clamp(3), 3));
test('low end', () => assert.equal(clamp(-1), 0));
test('low end again', () => assert.equal(clamp(-5), 0));
test('tautology', () => {
  const x = 3;
  assert.equal(x, 3);
});
JS
g add -A && g commit -qm change
J="$PY $KIT/scripts/bite --test 'node --test --test-reporter=spec test/*.test.mjs' --src '\.mjs\$' --tests '^test/'"
o=$(eval "$J --base main~1" 2>&1); rc=$?
[ $rc = 0 ] && grep -q "passes on old code: .tautology" <<<"$o" && ! grep -q "passes on old code: .low end" <<<"$o"
expect "node --test: the tautology passes on the old code, the real tests do not" $? "rc=$rc $o"
grep -q "survived: clamp.mjs:2  const HI = 10;  ->  const HI = 11;" <<<"$o" && grep -q "3/4 mutants killed" <<<"$o"
expect "node --test: unchecked limit survives, checked lines killed, comment not mutated" $? "$o"
o=$(eval "$J --audit" 2>&1); rc=$?
[ $rc = 0 ] && grep -q "killed none: .tautology" <<<"$o" && [ "$(grep -c "^  killed none" <<<"$o")" = 1 ] \
  && [ "$("$PY" -c "import json,glob;print(json.load(open(sorted(glob.glob('evidence/bite/*.json'))[-1]))['covered_by'])")" = "{'.low end again': '.low end'}" ]
expect "--audit: tests that kill nothing and duplicate proofs are listed" $? "rc=$rc $o"
mkdir -p node_modules/dep node_modules/.cache && printf 'export const one = 1;\n' > node_modules/dep/index.mjs && echo node_modules > .gitignore
printf "import test from 'node:test';\nimport assert from 'node:assert';\nimport { mkdirSync, writeFileSync } from 'node:fs';\nimport { one } from 'dep/index.mjs';\ntest('dep', () => { mkdirSync('node_modules/.cache', { recursive: true }); writeFileSync('node_modules/.cache/run', 'x'); assert.equal(one, 1); });\n" > test/dep.test.mjs
g add -A && g commit -qm dep
o=$(eval "$J --base main~1" 2>&1); rc=$?
[ $rc = 0 ] && [ -f node_modules/dep/index.mjs ] && [ ! -e node_modules/.cache/run ] && [ "$(git worktree list | wc -l | tr -d ' ')" = 1 ]
expect "node_modules reach the worktree, its cache stays there, the checkout's survive removal" $? "rc=$rc $o $(ls -aR node_modules)"
g worktree add -q "$T/js-wt" HEAD && o=$(cd "$T/js-wt" && eval "$J --base main~1" 2>&1); rc=$?
[ $rc = 0 ] && [ -f node_modules/dep/index.mjs ]; expect "from a worktree without node_modules, the main checkout's are used" $? "rc=$rc $o"
g worktree remove --force "$T/js-wt"  # bite wrote its evidence there
# A cwd under a linked package is the checkout's own folder: nothing is created or run there (review 23/10/2026).
o=$(eval "$J --base main~1 --cwd node_modules/dep" 2>&1); rc=$?
[ $rc = 1 ] && grep -q "resolves outside the throwaway worktree" <<<"$o" && [ -f node_modules/dep/index.mjs ] \
  && [ ! -e node_modules/dep/node_modules ] && [ "$(git worktree list | wc -l | tr -d ' ')" = 1 ]
expect "cwd through a dependency link -> not run, the checkout untouched" $? "rc=$rc $o"
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*)  # `\repo` is rooted on Windows: wt / cwd would be the drive's \repo
    o=$(eval "$J --base main~1 --cwd '\\js'" 2>&1); rc=$?
    [ $rc = 2 ] && grep -q "cwd must be a path inside the repo" <<<"$o"; expect "rooted Windows cwd refused" $? "rc=$rc $o" ;;
esac
# A tracked symlink in the source is not mutated: writing it would change the file it points to.
printf 'export const outside = 1 < 2;\n' > "$T/outside.mjs" && ln -s "$T/outside.mjs" linked.mjs && g add -A && g commit -qm link
if [ -L linked.mjs ]; then
o=$(eval "$J --audit" 2>&1); rc=$?
[ $rc = 0 ] && ! grep -q "linked.mjs" "$(ls -t evidence/bite/*.json | head -1)" && [ "$(cat "$T/outside.mjs")" = "export const outside = 1 < 2;" ]
expect "--audit leaves tracked symlinks and their targets alone" $? "rc=$rc $o"
else echo "skip --audit leaves tracked symlinks alone (ln -s made a copy: Windows without symlink rights)"; fi
o=$("$PY" "$KIT/scripts/bite" --test 'node --test --test-reporter=dot test/*.test.mjs' --src '\.mjs$' --tests '^test/' --audit 2>&1); rc=$?
[ $rc = 1 ] && grep -q "no per-test verdicts" <<<"$o"; expect "--audit without per-test verdicts -> not measured" $? "rc=$rc $o"

echo ""; echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
