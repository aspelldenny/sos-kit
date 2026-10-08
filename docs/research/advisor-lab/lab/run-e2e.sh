#!/usr/bin/env bash
# E2E: Sonnet worker fixes pay.py; hook calls the advisor after ADVISE_AFTER failing test runs.
# Usage: ./run-e2e.sh [advise_after=1] [backend=claude]
set -uo pipefail
cd "$(dirname "$0")"
git show baseline-bug:pay.py > pay.py && rm -rf .advise-state evidence/advice
python3 -m unittest -q test_pay >/dev/null 2>&1 && { echo "baseline is not failing — aborting"; exit 1; }
export ADVISE_AFTER="${1:-1}" ADVISE_BACKEND="${2:-claude}"
start=$(date +%s)
claude -p "Fix pay.py so that all tests in test_pay.py pass. Run python3 -m unittest test_pay after each change. Do not edit the tests. When done, report in 5 lines: root cause(s), fix, test result, and whether you received advisor guidance and what you did with it." \
  --model sonnet --allowedTools "Bash(python3:*)" "Edit" "Read" --output-format json < /dev/null > evidence/e2e.json 2>&1
echo "wall: $(( $(date +%s) - start ))s"
python3 -I -c "
import json;d=json.load(open('evidence/e2e.json'))
print('worker cost \$%.4f, turns %s, error %s' % (d.get('total_cost_usd') or 0, d.get('num_turns'), d.get('is_error')))
print(d.get('result','')[:1200])"
ls evidence/advice 2>/dev/null && head -3 evidence/advice/*.md
python3 -m unittest -q test_pay 2>&1 | tail -1
