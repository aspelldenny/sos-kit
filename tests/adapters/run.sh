#!/usr/bin/env bash
# Adapter tests: real hook payloads (captured live 2026-10-09 from Claude Code and Codex 0.162)
# plus mutations of them, run through adapters/<agent>/hook.py in a throwaway git repo.
# scripts/advise is replaced by a stub, so no model is called.
# Usage: tests/adapters/run.sh
set -uo pipefail
KIT=$(cd "$(dirname "$0")/../.." && pwd)
FIX="$KIT/tests/adapters/fixtures"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
cd "$T" && git init -q && git commit -q --allow-empty -m init
mkdir -p scripts adapters
cp "$KIT"/scripts/{env-guard.sh,test-watch.py,status.sh} scripts/
cp -R "$KIT"/adapters/claude "$KIT"/adapters/codex adapters/
cat > scripts/advise <<'PY'
#!/usr/bin/env python3
import json, os, sys
if "--config" in sys.argv:
    print(json.dumps({"backend": os.environ.get("STUB_BACKEND", "claude"), "after": 2, "max": 2})); sys.exit(0)
if os.environ.get("STUB_FAIL"):
    print("stub advisor down", file=sys.stderr); sys.exit(1)
print("STUB ADVICE agent=" + (sys.argv[sys.argv.index("--agent") + 1] if "--agent" in sys.argv else "?"))
PY
chmod +x scripts/advise
export CLAUDE_PROJECT_DIR="$T" ADVISE_AFTER=2 ADVISE_MAX=2
pass=0; fail=0

# line <file> <n> [python expression mutating d]
line() { python3 - "$FIX/$1" "$2" "${3:-}" <<'EOF'
import json, sys
d = json.loads(open(sys.argv[1]).read().splitlines()[int(sys.argv[2]) - 1])
if sys.argv[3]: exec(sys.argv[3])
print(json.dumps(d))
EOF
}
# check <name> <want-exit> <want-substring|-> <agent> <sub> <<< payload
check() {
    local name=$1 want=$2 grep_for=$3 agent=$4 sub=$5 out rc
    out=$(python3 "adapters/$agent/hook.py" "$sub" 2>&1); rc=$?
    local match=1
    if [ "$grep_for" = "-" ]; then [ -z "$out" ] && match=0; else grep -q -- "$grep_for" <<<"$out" && match=0; fi
    if [ "$rc" = "$want" ] && [ "$match" = 0 ]; then
        pass=$((pass + 1)); echo "ok   $name"
    else
        fail=$((fail + 1)); echo "FAIL $name (exit $rc, want $want / ${grep_for}): ${out:0:300}"
    fi
}
C=claude-live-2026-10-09.jsonl X=codex-live-2026-10-09.jsonl

# ── Claude Code ──────────────────────────────────────────────
check "claude: Write notes.txt allowed"        0 -          claude pre-edit <<<"$(line $C 3)"
check "claude: Write .env blocked"             2 "How to fix" claude pre-edit <<<"$(line $C 3 'd["tool_input"]["file_path"]="/REPO/.env"')"
check "claude: Write .env.local blocked"       2 "BLOCKED"  claude pre-edit <<<"$(line $C 3 'd["tool_input"]["file_path"]="/REPO/app/.env.local"')"
check "claude: Write .env.example allowed"     0 -          claude pre-edit <<<"$(line $C 3 'd["tool_input"]["file_path"]="/REPO/.env.example"')"
check "claude: NotebookEdit .env blocked"      2 "BLOCKED"  claude pre-edit <<<"$(line $C 3 'd["tool_input"]={"notebook_path":"/REPO/.env"}')"
check "claude: unreadable payload blocked"     2 "BLOCKED"  claude pre-edit <<<"not json"
check "claude: piped fail #1 silent"           0 -          claude post-bash <<<"$(line $C 1)"
check "claude: failure event #2 -> advice"    0 '"hookEventName": "PostToolUseFailure"' claude post-bash <<<"$(line $C 2)"
check "claude: passing run resets count"     0 -          claude post-bash <<<"$(line $C 1 'd["tool_response"]["stdout"]="3 passed"')"
check "claude: pass reset -> fail #1 silent"   0 -          claude post-bash <<<"$(line $C 1)"
check "claude: fail #2 -> advice (PostToolUse)" 0 '"hookEventName": "PostToolUse"' claude post-bash <<<"$(line $C 1)"
check "claude: fail #3 silent"                 0 -          claude post-bash <<<"$(line $C 1)"
check "claude: fail #4 -> 2nd advice"          0 "STUB ADVICE" claude post-bash <<<"$(line $C 1)"
check "claude: fail #5 silent"                 0 -          claude post-bash <<<"$(line $C 1)"
check "claude: fail #6 -> limit, stop+report"  0 "ADVISOR LIMIT" claude post-bash <<<"$(line $C 1)"
check "claude: fail #7 quiet after limit"    0 -          claude post-bash <<<"$(line $C 1)"
check "claude: fail #8 quiet after limit"    0 -          claude post-bash <<<"$(line $C 1)"
check "claude: subagent has its own count"     0 -          claude post-bash <<<"$(line $C 1 'd["agent_id"]="w1"')"
check "claude: non-test command ignored"       0 -          claude post-bash <<<"$(line $C 2 'd["tool_input"]["command"]="ls"')"
SOS_ADVISOR=off check "claude: SOS_ADVISOR=off" 0 -        claude post-bash <<<"$(line $C 2 'd["session_id"]="s2"')"
check "claude: k2 fail #1 silent"              0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k2"')"
check "claude: 'cat pytest.ini' does not reset" 0 -         claude post-bash <<<"$(line $C 1 'd["session_id"]="k2"; d["tool_input"]["command"]="cat pytest.ini"; d["tool_response"]["stdout"]="[pytest]"')"
check "claude: k2 fail #2 -> advice"           0 "STUB ADVICE" claude post-bash <<<"$(line $C 1 'd["session_id"]="k2"')"
check "claude: 'failures=0' is a pass #1"      0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k3"; d["tool_response"]["stdout"]="Ran 4 tests\nOK (failures=0)"')"
check "claude: 'failures=0' is a pass #2"      0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k3"; d["tool_response"]["stdout"]="Ran 4 tests\nOK (failures=0)"')"
check "claude: wrapped test cmd counts #1"     0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k4"; d["tool_input"]["command"]="cd app && CI=1 timeout 60 python3 -m pytest -q"')"
check "claude: wrapped test cmd counts #2"     0 "STUB ADVICE" claude post-bash <<<"$(line $C 1 'd["session_id"]="k4"; d["tool_input"]["command"]="cd app && CI=1 timeout 60 python3 -m pytest -q"')"
check "claude: .Env.Local (case) blocked"      2 "BLOCKED"  claude pre-edit <<<"$(line $C 3 'd["tool_input"]["file_path"]="/REPO/.Env.Local"')"
check "claude: k5 fail #1 silent"              0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k5"')"
check "claude: quoted ';pytest' not a test"    0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k5"; d["tool_input"]["command"]="echo \"hello; pytest\" && printf \"a|pytest\""; d["tool_response"]["stdout"]="hello; pytest"')"
check "claude: k5 fail #2 -> advice"           0 "STUB ADVICE" claude post-bash <<<"$(line $C 1 'd["session_id"]="k5"')"
check "claude: k6 pytest fail #1"              0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k6"')"
check "claude: k6 other suite passes, no reset" 0 -         claude post-bash <<<"$(line $C 1 'd["session_id"]="k6"; d["tool_input"]["command"]="cargo test"; d["tool_response"]["stdout"]="test result: ok. 3 passed"')"
check "claude: k6 pytest fail #2 -> advice"    0 "agent=claude" claude post-bash <<<"$(line $C 1 'd["session_id"]="k6"')"
STUB_BACKEND=off check "claude: backend off -> silent" 0 - claude post-bash <<<"$(line $C 1 'd["session_id"]="k7"')"
STUB_BACKEND=off check "claude: backend off #2 silent" 0 - claude post-bash <<<"$(line $C 1 'd["session_id"]="k7"')"
check "claude: k8 fail #1"                     0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k8"')"
STUB_FAIL=1 check "claude: advisor down is visible" 0 "advisor unavailable" claude post-bash <<<"$(line $C 1 'd["session_id"]="k8"')"
check "claude: session-start prints status"    0 "branch:"  claude session-start <<<"{}"

# ── Codex ────────────────────────────────────────────────────
check "codex: real apply_patch allowed"        0 -          codex pre-edit <<<"$(line $X 4)"
check "codex: Bash pre-tool ignored"           0 -          codex pre-edit <<<"$(line $X 2)"
check "codex: 2nd file .env blocked"           2 "How to fix" codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="*** Begin Patch\n*** Add File: /REPO/notes.txt\n+hi\n*** Update File: /REPO/.env\n@@\n-A=1\n+A=2\n*** End Patch"')"
check "codex: Move to .env blocked"            2 "BLOCKED"  codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="*** Begin Patch\n*** Update File: cfg.txt\n*** Move to: .env.production\n*** End Patch"')"
check "codex: Delete .env.example allowed"     0 -          codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="*** Begin Patch\n*** Delete File: .env.example\n*** End Patch"')"
check "codex: indented .env header blocked"    2 "BLOCKED"  codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="*** Begin Patch\n*** Add File: notes.txt\n+hi\n *** Add File: .env\n+TOKEN=x\n*** End Patch"')"
check "codex: tab-indented .env header blocked" 2 "BLOCKED" codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="*** Begin Patch\n\t*** Update File: .env\n*** End Patch"')"
check "codex: NBSP-indented .env blocked"     2 "BLOCKED"  codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="*** Begin Patch\n*** Add File: notes.txt\n+ok\n\u00a0*** Add File: .env\n+T=x\n*** End Patch"')"
check "codex: CR/VT/EM-space prefix blocked"   2 "BLOCKED"  codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="*** Begin Patch\n*** Add File: a.txt\n+ok\n\r\u000b\u2003*** Update File: .env\n*** End Patch"')"
check "codex: .ENV (case) blocked"             2 "BLOCKED"  codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="*** Begin Patch\n*** Add File: .ENV\n+T=x\n*** End Patch"')"
check "codex: U+2028 inside path blocked"     2 "BLOCKED"  codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="*** Begin Patch\n*** Update File: dir\u2028/.env\n@@\n-A=1\n+A=2\n*** End Patch"')"
check "codex: unparseable patch blocked"       2 "could not find" codex pre-edit <<<"$(line $X 4 'd["tool_input"]["command"]="garbage"')"
for n in 1 2 3; do
check "codex: P078b3 fixture line $n allowed"  0 -          codex pre-edit <<<"$(sed -n "${n}p" "$FIX/codex-p078b3-apply-patch.jsonl")"
done
check "codex: piped fail #1 silent"            0 -          codex post-bash <<<"$(line $X 3)"
check "codex: piped fail #2 -> advice"         0 "agent=codex" codex post-bash <<<"$(line $X 3)"
check "codex: exit-code-only fail #1 silent"   0 -          codex post-bash <<<"$(line $X 3 'd["session_id"]="s3"; d["tool_response"]="Exit code: 1\nOutput:\nboom"')"
check "codex: exit-code-only fail #2 advice"   0 "STUB ADVICE"          codex post-bash <<<"$(line $X 3 'd["session_id"]="s3"; d["tool_response"]="Exit code: 1\nOutput:\nboom"')"
check "codex: session-start gives context"     0 "additionalContext" codex session-start <<<"$(line $X 1)"

# ── advise: the prompt carries intent (real scripts/advise, fake claude CLI) ──
A=$(mktemp -d); mkdir -p "$A/bin" "$A/repo"
cat > "$A/bin/claude" <<'SH'
#!/usr/bin/env bash
cat > "$ADVISE_PROMPT_OUT"
echo '{"result":"VERDICT: continue","is_error":false}'
SH
chmod +x "$A/bin/claude"
( cd "$A/repo" && git init -q && git commit -q --allow-empty -m init )
# prompt <name> <want-substring>: run advise in $A/repo and grep the prompt it sent
prompt() {
    local name=$1 want=$2
    (cd "$A/repo" && PATH="$A/bin:$PATH" ADVISE_PROMPT_OUT="$A/prompt" python3 "$KIT/scripts/advise" --backend claude "why red?" >/dev/null 2>&1)
    if grep -q -- "$want" "$A/prompt" 2>/dev/null; then pass=$((pass + 1)); echo "ok   $name"
    else fail=$((fail + 1)); echo "FAIL $name (want $want): $(head -c 300 "$A/prompt" 2>/dev/null)"; fi
}
prompt "advise: no intent -> says so"          "none found"
mkdir -p "$A/repo/docs"
printf '# Backlog\n\nintro\n\n## Now\n\n- [ ] export splits shifts at Sunday midnight\n\n## Later\n\n- [ ] dark mode\n' > "$A/repo/docs/BACKLOG.md"
prompt "advise: backlog current section sent"  "export splits shifts"
if grep -q "dark mode" "$A/prompt"; then fail=$((fail + 1)); echo "FAIL advise: later section leaked"; else pass=$((pass + 1)); echo "ok   advise: later section not sent"; fi
printf '{"features":[{"id":"F01","title":"Totals match payslip","passes":false},{"id":"F02","title":"Old done thing","passes":true}]}' > "$A/repo/docs/FEATURES.json"
prompt "advise: open feature sent"             "Totals match payslip"
if grep -q "Old done thing" "$A/prompt"; then fail=$((fail + 1)); echo "FAIL advise: passing feature sent"; else pass=$((pass + 1)); echo "ok   advise: passing feature not sent"; fi
printf 'Brief: refunds must never go negative\n' > "$A/brief.md"
ADVISE_BRIEF="$A/brief.md" prompt "advise: ADVISE_BRIEF wins" "refunds must never go negative"
rm -rf "$A"

echo ""; echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
