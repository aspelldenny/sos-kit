#!/usr/bin/env bash
# `sos usage` against the old tools/agent-usage.py (copied to tests/usage/) on the part both read:
# one Claude project folder, tokens per model. One known, documented difference is applied to the
# oracle's side before comparing (docs/plans/RUST_ARCHITECTURE_2026-11-05.md §7): a sidechain line
# whose subagent file is missing is dropped by the oracle and kept by sos.
# Then a memory check: a large generated transcript must stay under 200 MB resident.
# Usage: tests/usage.sh [path/to/sos]
set -uo pipefail
KIT=$(cd "$(dirname "$0")/.." && pwd)
SOS=${1:-$KIT/target/debug/sos}
[ -x "$SOS" ] || SOS=$(command -v sos) || { echo "FAIL no sos binary (build first or pass its path)"; exit 1; }
PY=$(for p in python3 python; do "$p" -c '' >/dev/null 2>&1 && { echo "$p"; break; }; done)
export PYTHONUTF8=1
F=$KIT/tests/usage/fixtures
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
expect() { if [ "$2" = 0 ]; then pass=$((pass + 1)); echo "ok   $1"; else fail=$((fail + 1)); echo "FAIL $1: ${3:0:600}"; fi; }

"$PY" "$KIT/tests/usage/agent-usage.py" --projects-dir "$F/claude/-w-app" --since 2026-09-01 > "$T/oracle.txt"
"$SOS" usage --dir "$F/claude" --project /w/app --since 2026-09-01 --by model --json \
  --prices "$KIT/crates/sos-cli/src/usage/prices.toml" > "$T/sos.json"
out=$("$PY" - "$T/oracle.txt" "$T/sos.json" <<'PY'
import json, sys
from collections import defaultdict
F = ("calls", "input", "cache_write", "cache_read", "output")
oracle = defaultdict(lambda: [0] * 5)
for line in open(sys.argv[1], encoding="utf-8").read().splitlines()[1:]:
    cells = line.split()
    if cells[0] == "total":
        continue
    # role (may contain spaces: "main (Quản đốc)"), model, runs, calls, input, cache write, cache read, output
    nums = [int(c.replace(",", "")) for c in cells[-5:]]
    model = cells[-7]
    oracle[model] = [a + b for a, b in zip(oracle[model], nums)]
# Known differences, from the fixture (tests/usage/fixtures/claude/-w-app/S1.jsonl):
for model, d in (("claude-sonnet-5-5", [1, 7, 0, 0, 3]),):    # mside9, kept by sos
    oracle[model] = [a + b for a, b in zip(oracle[model], d)]
ours = defaultdict(lambda: [0] * 5)
for r in json.load(open(sys.argv[2], encoding="utf-8"))["rows"]:
    ours[r["model"]] = [a + r[f] for a, f in zip(ours[r["model"]], F)]
bad = [f"{m}: oracle {oracle[m]} sos {ours[m]}" for m in sorted(set(oracle) | set(ours)) if oracle[m] != ours[m]]
print("\n".join(bad) if bad else "same")
PY
)
[ "$out" = same ]; expect "per-model tokens match agent-usage.py on the shared part" $? "$out"

# Memory: ~120 MB of transcript, one 20 MB line over --max-line-bytes; RSS must stay < 200 MB.
"$PY" - "$T/big/-big" <<'PY'
import json, os, sys
d = sys.argv[1]; os.makedirs(d)
with open(os.path.join(d, "B.jsonl"), "w") as f:
    pad = "p" * 2000
    for i in range(60000):
        f.write(json.dumps({"type": "assistant", "sessionId": "B", "timestamp": "2026-11-01T00:00:00Z", "cwd": "/big",
            "message": {"id": "m%d" % i, "model": "claude-opus-5-5", "content": [{"type": "text", "text": pad}],
                        "usage": {"input_tokens": 1, "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0, "output_tokens": 1}}}) + "\n")
    f.write('{"type":"assistant","x":"' + "y" * (20 * 1024 * 1024) + '"}\n')
PY
rss_kb=""
if /usr/bin/time -v true >/dev/null 2>&1; then
  /usr/bin/time -v "$SOS" usage --dir "$T/big" --since 2026-10-01 --json --max-line-bytes 8388608 > "$T/big.json" 2> "$T/time.txt"
  rss_kb=$(sed -n 's/.*Maximum resident set size (kbytes): //p' "$T/time.txt")
elif /usr/bin/time -l true >/dev/null 2>&1; then
  /usr/bin/time -l "$SOS" usage --dir "$T/big" --since 2026-10-01 --json --max-line-bytes 8388608 > "$T/big.json" 2> "$T/time.txt"
  rss_kb=$(( $(awk '/maximum resident set size/ {print $1}' "$T/time.txt") / 1024 ))
fi
calls=$("$PY" -c 'import json,sys; d=json.load(open(sys.argv[1])); m=d["meta"]["agents"]["claude"]; print(m["calls"], m["skipped"]["oversize_lines"])' "$T/big.json")
[ "$calls" = "60000 1" ]; expect "large transcript: every call counted, the oversize line skipped" $? "$calls"
if [ -n "$rss_kb" ]; then
  [ "$rss_kb" -lt 204800 ]; expect "large transcript stays under 200 MB resident (${rss_kb} kB)" $?
else
  echo "skip memory check: no /usr/bin/time with -v or -l"
fi

echo "usage: $pass passed, $fail failed"
[ "$fail" = 0 ]
