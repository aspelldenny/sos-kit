#!/usr/bin/env bash
# scripts/bite on a throwaway Python repo whose change has one good test, one tautology and one
# unchecked branch: each must show up where it belongs. Usage: tests/bite.sh
set -uo pipefail
KIT=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
expect() { if [ "$2" = 0 ]; then pass=$((pass + 1)); echo "ok   $1"; else fail=$((fail + 1)); echo "FAIL $1: ${3:0:400}"; fi; }
g() { git -c user.name=t -c user.email=t@t "$@"; }

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
B="python3 $KIT/scripts/bite --test 'python3 -m unittest discover -s tests -t . -v' --src '\.py$' --tests '^tests/' --base main~1"

o=$(eval "$B" 2>&1); rc=$?
expect "bite: measured" "$([ $rc = 0 ]; echo $?)" "rc=$rc $o"
grep -q "passes on old code: Clamp.test_tautology" <<<"$o"; expect "tautology passes on the old code" $? "$o"
! grep -q "passes on old code: Clamp.test_low_end" <<<"$o"; expect "the real test fails on the old code" $? "$o"
! grep -q "Old.test_identity" <<<"$o"; expect "untouched test not listed" $? "$o"
grep -q "survived: calc.py:1  LIMIT = 10  ->  LIMIT = 11" <<<"$o"; expect "unchecked default (hi=LIMIT) survives" $? "$o"
! grep -qE "survived: calc.py:(9|11)" <<<"$o"; expect "checked branches killed" $? "$o"
at=$(python3 -c "import json,glob;print(sorted(m['at'] for m in json.load(open(sorted(glob.glob('evidence/bite/*.json'))[-1]))['mutants']))")
[ "$at" = "['calc.py:1', 'calc.py:11', 'calc.py:4', 'calc.py:9']" ]; expect "mutants on code lines only (not the docstring)" $? "$at"
[ -z "$(git status --porcelain --untracked-files=no)" ] && [ "$(git worktree list | wc -l | tr -d ' ')" = 1 ]
expect "checkout untouched, worktree removed" $? "$(git status --short; git worktree list)"

o=$(python3 "$KIT/scripts/bite" 2>&1); rc=$?
[ $rc = 2 ] && grep -q "\[bite\]" <<<"$o"; expect "not configured -> exit 2 with the fix" $? "rc=$rc $o"
printf "[bite]\ntest = 'python3 -m unittest discover -s tests -t . -v'\nsrc = '\\\\.py\$'\ntests = '^tests/'\nmutants = 1\n" > .sos.toml
o=$(python3 "$KIT/scripts/bite" --base main~1 2>&1); rc=$?
[ $rc = 0 ] && grep -q "/1 mutants killed" <<<"$o"; expect ".sos.toml [bite] is read" $? "rc=$rc $o"
printf 'import unittest\n\n\nclass Red(unittest.TestCase):\n    def test_red(self):\n        self.fail("red")\n' > tests/test_red.py
g add -A && g commit -qm red
o=$(eval "$B~1" 2>&1); rc=$?
[ $rc = 1 ] && grep -q "red at HEAD" <<<"$o"; expect "red HEAD -> not measured" $? "rc=$rc $o"

echo ""; echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
