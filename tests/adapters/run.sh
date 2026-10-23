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
if os.environ.get("STUB_DOWN"):
    print("claude failed: OAuth session expired. Fix: run `claude` and /login", file=sys.stderr); sys.exit(3)
if os.environ.get("STUB_FAIL"):
    print("stub advisor down", file=sys.stderr); sys.exit(1)
print("STUB ADVICE agent=" + (sys.argv[sys.argv.index("--agent") + 1] if "--agent" in sys.argv else "?"))
PY
chmod +x scripts/advise
export CLAUDE_PROJECT_DIR="$T" ADVISE_AFTER=2 ADVISE_MAX=2
pass=0; fail=0
# Windows: `python3` is often the Store alias that only fails; use the interpreter that runs.
PY=$(for p in python3 python; do "$p" -c '' >/dev/null 2>&1 && { echo "$p"; break; }; done)
export PYTHONUTF8=1  # Windows: the helpers below print Vietnamese into pipes
WIN=
case "$(uname -s)" in
MINGW*|MSYS*|CYGWIN*)  # Windows cannot exec a bash script; a Windows path with backslashes, quoted
    printf 'import os, sys\nopen(os.path.join(os.path.dirname(__file__), "notified"), "a").write(" ".join(sys.argv[1:]) + "\\n")\n' > "$T/notify.py"
    export SOS_NOTIFY_CMD="\"$(cygpath -w "$(command -v "$PY")")\" \"$(cygpath -w "$T/notify.py")\""; WIN=1 ;;
*)
    printf '#!/usr/bin/env bash\nprintf "%%s\\n" "$*" >> "$(dirname "$0")/notified"\n' > "$T/notify"
    chmod +x "$T/notify"; export SOS_NOTIFY_CMD="$T/notify" ;;
esac

# line <file> <n> [python expression mutating d]
line() { "$PY" - "$FIX/$1" "$2" "${3:-}" <<'EOF'
import json, sys
d = json.loads(open(sys.argv[1]).read().splitlines()[int(sys.argv[2]) - 1])
if sys.argv[3]: exec(sys.argv[3])
print(json.dumps(d))
EOF
}
# check <name> <want-exit> <want-substring|-> <agent> <sub> <<< payload
check() {
    local name=$1 want=$2 grep_for=$3 agent=$4 sub=$5 out rc
    out=$("$PY" "adapters/$agent/hook.py" "$sub" 2>&1); rc=$?
    # Windows: the Codex adapter blocks with deny JSON and exit 0 (`powershell -Command` turns exit 2 into 1).
    if [ "$agent/$sub/$want" = codex/pre-edit/2 ] && [ -n "$WIN" ]; then
        want=0; grep -q '"permissionDecision": "deny"' <<<"$out" || rc="$rc, no deny JSON"
    fi
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
PY3=$(command -v "$PY"); mkdir -p "$T/win"
printf '#!/usr/bin/env bash\necho "Python was not found" >&2; exit 49\n' > "$T/win/python3"; chmod +x "$T/win/python3"
pl=$(line $C 1 'd["session_id"]="kwin"')  # built first: the here-string would see the stub PATH
for n in 1 2; do o=$(PATH="$T/win:$PATH" "$PY3" adapters/claude/hook.py post-bash <<<"$pl" 2>&1); done
grep -q "STUB ADVICE" <<<"$o" && pass=$((pass + 1)) && echo "ok   windows: advice without a working python3" || { fail=$((fail + 1)); echo "FAIL windows: advice without a working python3: ${o:0:300}"; }
# The project's own test command (.sos.toml [advisor] tests), e.g. POD-agent's `node scripts/check.mjs`.
printf '[advisor]\ntests = ["node scripts/check.mjs", '"'"'make e2e'"'"']\n\n[state]\nfile = "x"\n' > .sos.toml
own() { line $C 1 "d[\"session_id\"]=\"$1\"; d[\"tool_input\"][\"command\"]=\"$2\"; d[\"tool_response\"][\"stdout\"]=\"⛔ check đỏ (1 lỗi)\"; d[\"hook_event_name\"]=\"PostToolUseFailure\"; d[\"error\"]=\"Exit code 1\""; }
check "own: 'node scripts/check.mjs' fail #1"    0 -           claude post-bash <<<"$(own o1 'node scripts/check.mjs')"
check "own: fail #2 (with a flag) -> advice"     0 "STUB ADVICE" claude post-bash <<<"$(own o1 'node scripts/check.mjs --full')"
check "own: 'node scripts/check.mjsx' not a run" 0 -           claude post-bash <<<"$(own o2 'node scripts/check.mjsx')"
check "own: second fail of it still not a run"   0 -           claude post-bash <<<"$(own o2 'node scripts/check.mjsx')"
for c in 'make e2e' 'cd web && make e2e'; do o=$("$PY" -c '
import runpy, sys
sys.modules["tomllib"] = None  # Python < 3.11: no tomllib, the [advisor] table is read by hand
sys.argv = ["test-watch.py", "--key", "o3", "--cmd", sys.argv[1], "--exit", "1", "--agent", "claude"]
runpy.run_path("scripts/test-watch.py", run_name="__main__")' "$c" <<<"FAILED" 2>&1); done
grep -q "STUB ADVICE" <<<"$o" && pass=$((pass + 1)) && echo "ok   own: read without tomllib" || { fail=$((fail + 1)); echo "FAIL own: read without tomllib: ${o:0:300}"; }
check "own: a passing run resets the streak #1"  0 -   claude post-bash <<<"$(own o4 'node scripts/check.mjs')"
check "own: passing run"                        0 -   claude post-bash <<<"$(line $C 1 'd["session_id"]="o4"; d["tool_input"]["command"]="node scripts/check.mjs"; d["tool_response"]["stdout"]="✅ check xanh"')"
check "own: after the reset, fail #1 silent"    0 -   claude post-bash <<<"$(own o4 'node scripts/check.mjs')"
o=$("$PY" "$KIT/tests/adapters/test_watch_units.py" "$KIT/scripts/test-watch.py" 2>&1)
[ "$o" = ok ] && pass=$((pass + 1)) && echo "ok   test-watch units: TOML reader, own-command match" || { fail=$((fail + 1)); echo "FAIL test-watch units: $o"; }
rm .sos.toml
check "claude: k6 pytest fail #1"              0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k6"')"
check "claude: k6 other suite passes, no reset" 0 -         claude post-bash <<<"$(line $C 1 'd["session_id"]="k6"; d["tool_input"]["command"]="cargo test"; d["tool_response"]["stdout"]="test result: ok. 3 passed"')"
check "claude: k6 pytest fail #2 -> advice"    0 "agent=claude" claude post-bash <<<"$(line $C 1 'd["session_id"]="k6"')"
STUB_BACKEND=off check "claude: backend off -> silent" 0 - claude post-bash <<<"$(line $C 1 'd["session_id"]="k7"')"
STUB_BACKEND=off check "claude: backend off #2 silent" 0 - claude post-bash <<<"$(line $C 1 'd["session_id"]="k7"')"
check "claude: k8 fail #1"                     0 -          claude post-bash <<<"$(line $C 1 'd["session_id"]="k8"')"
STUB_FAIL=1 check "claude: advisor down is visible" 0 "advisor unavailable" claude post-bash <<<"$(line $C 1 'd["session_id"]="k8"')"
K9='d["session_id"]="k9"'
check "claude: k9 fail #1"                     0 -          claude post-bash <<<"$(line $C 1 "$K9")"
STUB_DOWN=1 check "claude: advisor logged out -> told once" 0 "do not leave it for the end" claude post-bash <<<"$(line $C 1 "$K9")"
if grep -q "advisor cannot run: .*OAuth session expired" "$T/notified" 2>/dev/null; then pass=$((pass + 1)); echo "ok   claude: Chủ nhà notified directly"
else fail=$((fail + 1)); echo "FAIL claude: no desktop notification: $(cat "$T/notified" 2>/dev/null)"; fi
rm -f "$T/notified"
STUB_DOWN=1 check "claude: still down -> quiet period"   0 -   claude post-bash <<<"$(line $C 1 "$K9")"
[ ! -e "$T/notified" ]; r=$?; if [ $r = 0 ]; then pass=$((pass + 1)); echo "ok   claude: one notification per quiet period"; else fail=$((fail + 1)); echo "FAIL claude: notified again"; fi
rm -rf .advise-state/unavailable
check "claude: kn fail #1 silent" 0 - claude post-bash <<<"$(line $C 1 'd["session_id"]="kn"')"
SOS_NOTIFY_CMD='broken "quote' STUB_DOWN=1 check "claude: a bad notify command still tells the worker" 0 "do not leave it for the end" claude post-bash <<<"$(line $C 1 'd["session_id"]="kn"')"
ADVISE_UNAVAILABLE_QUIET=0 check "claude: login fixed -> advice at once" 0 "STUB ADVICE" claude post-bash <<<"$(line $C 1 "$K9")"
check "claude: k9 fail silent"                 0 -          claude post-bash <<<"$(line $C 1 "$K9")"
check "claude: k9 2nd advice (down call not counted)" 0 "STUB ADVICE" claude post-bash <<<"$(line $C 1 "$K9")"
XA='d["session_id"]="x1"; d["tool_input"]["command"]="xcodebuild -scheme T -only-testing:TTests/A test > build/a.log 2>&1"; d["tool_response"]["stdout"]="** TEST FAILED **"'
XB='d["session_id"]="x1"; d["tool_input"]["command"]="xcodebuild -scheme T -only-testing:TTests/B test -resultBundlePath r.xcresult | tail -5"; d["tool_response"]["stdout"]="** TEST FAILED **"'
check "claude: xcodebuild test fail #1"        0 -          claude post-bash <<<"$(line $C 1 "$XA")"
check "claude: other filter/log, same streak"  0 "STUB ADVICE" claude post-bash <<<"$(line $C 1 "$XB")"
HD='d["session_id"]="x2"; d["tool_input"]["command"]="python3 - <<PY\nimport subprocess\nPY"; d["tool_response"]["stdout"]="Testing failed:\n** TEST FAILED **"'
check "claude: heredoc w/ xcode verdict #1"    0 -          claude post-bash <<<"$(line $C 1 "$HD")"
check "claude: heredoc w/ xcode verdict #2"    0 "STUB ADVICE" claude post-bash <<<"$(line $C 1 "$HD")"
BF='d["session_id"]="x3"; d["tool_input"]["command"]="xcodebuild -scheme T build -quiet"; d["tool_response"]["stdout"]="** BUILD FAILED **"'
check "claude: build fail #1"                  0 -          claude post-bash <<<"$(line $C 1 "$BF")"
check "claude: build fail #2 -> advice"        0 "STUB ADVICE" claude post-bash <<<"$(line $C 1 "$BF")"
TL='d["session_id"]="x4"; d["tool_input"]["command"]="tail -40 build/a.log"; d["tool_response"]["stdout"]="** TEST FAILED **"'
check "claude: tail of a failed log #1 silent" 0 -          claude post-bash <<<"$(line $C 1 "$TL")"
check "claude: tail of a failed log #2 silent" 0 -          claude post-bash <<<"$(line $C 1 "$TL")"
XL='d["session_id"]="x5"; d["tool_input"]["command"]="xcodebuild -list"; d["tool_response"]["stdout"]="error"; d["tool_response"]["interrupted"]=False'
check "claude: xcodebuild -list ignored #1"    0 -          claude post-bash <<<"$(line $C 1 "$XL")"
check "claude: xcodebuild -list ignored #2"    0 -          claude post-bash <<<"$(line $C 1 "$XL")"
check "claude: session-start prints status"    0 "branch:"  claude session-start <<<"{}"
mkdir -p docs && { echo "## Now"; for i in $(seq 90); do echo "- [ ] item $i"; done; } > docs/BACKLOG.md
seq 210 > AGENTS.md
check "claude: status notes oversized state"   0 "section is 91 lines" claude session-start <<<"{}"
check "claude: status notes oversized entry"   0 "are 210 lines" claude session-start <<<"{}"
rm -rf docs AGENTS.md
printf '[roles.reviewer.claude]   # Claude built it\nbackend = "codex"\nmodel = "m"\n\n[roles.reviewer.codex]\nbackend = "claude"\n' > .sos.toml
check "claude: status names the cross-family reviewer" 0 "claude-built work → codex; codex-built work → claude" claude session-start <<<"{}"
printf '[roles.reviewer]\nbackend = "claude"\n' > .sos.toml
out=$("$PY" adapters/claude/hook.py session-start <<<"{}" 2>&1)
if grep -q "Người soát" <<<"$out"; then fail=$((fail + 1)); echo "FAIL status: same-family reviewer named"; else pass=$((pass + 1)); echo "ok   status: quiet for a same-family reviewer"; fi
rm -f .sos.toml

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
# fakecli <name> <<< script: a stub CLI first on PATH. Windows resolves CLIs through PATHEXT
# (scripts/advise uses shutil.which), so an extensionless stub alone is skipped and the REAL
# claude/codex gets called (23/10/2026); a <name>.cmd wrapper runs the stub instead.
fakecli() {
    cat > "$A/bin/$1"; chmod +x "$A/bin/$1"
    case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*)
        printf '@"%s" "%%~dp0%s" %%*
' "$(cygpath -w "$(command -v bash)")" "$1" > "$A/bin/$1.cmd" ;;
    esac
}
fakecli claude <<'SH'
#!/usr/bin/env bash
cat > "$ADVISE_PROMPT_OUT"
echo '{"result":"VERDICT: continue","is_error":false}'
SH
( cd "$A/repo" && git init -q && git commit -q --allow-empty -m init )
# prompt <name> <want-substring>: run advise in $A/repo and grep the prompt it sent
prompt() {
    local name=$1 want=$2
    (cd "$A/repo" && PATH="$A/bin:$PATH" ADVISE_PROMPT_OUT="$A/prompt" "$PY" "$KIT/scripts/advise" --backend claude "why red?" >/dev/null 2>&1)
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
printf '[state]\nfile = "docs/PROGRESS.md"\n' > "$A/repo/.sos.toml"
printf '# P\n\n## Current\n\nP5 refund fence open\n' > "$A/repo/docs/PROGRESS.md"
prompt "advise: [state] file is the intent"   "P5 refund fence open"
if grep -q "export splits shifts" "$A/prompt"; then fail=$((fail + 1)); echo "FAIL advise: BACKLOG used despite [state]"; else pass=$((pass + 1)); echo "ok   advise: BACKLOG not used when [state] names another"; fi
printf '# P\n\n## Hiện tại\n\n%s\n' "$("$PY" -c 'print("Kiểm tra hành trình hoàn tiền. " * 200)')" > "$A/repo/docs/PROGRESS.md"
prompt "advise: long Vietnamese state doc ok"  "Kiểm tra hành trình"
printf 'Brief: refunds must never go negative\n' > "$A/brief.md"
ADVISE_BRIEF="$A/brief.md" prompt "advise: ADVISE_BRIEF wins" "refunds must never go negative"
# A lost login: exit 3, the fix logged, the session-start status says it until a good call.
fakecli claude <<'SH'
#!/usr/bin/env bash
cat > /dev/null
if [ -n "${CLAUDE_LOGGED_OUT:-}" ]; then
  echo '{"result":"Failed to authenticate: OAuth session expired and could not be refreshed","is_error":true}'; exit 1
fi
echo '{"result":"OK","is_error":false}'
SH
expect() { if [ "$2" = 0 ]; then pass=$((pass + 1)); echo "ok   $1"; else fail=$((fail + 1)); echo "FAIL $1: ${3:0:300}"; fi; }
o=$(cd "$A/repo" && PATH="$A/bin:$PATH" CLAUDE_LOGGED_OUT=1 "$PY" "$KIT/scripts/advise" --backend claude "why red?" 2>&1); rc=$?
[ "$rc" = 3 ] && grep -q "/login" <<<"$o"; expect "advise: lost login -> exit 3 with the fix" $? "rc=$rc $o"
o=$(cd "$A/repo" && bash "$KIT/scripts/status.sh"); grep -q "advisor's last call failed" <<<"$o"; expect "status: names the broken advisor" $? "$o"
# Windows: `python3` on PATH is the Store alias that only fails; the adapter passes its own interpreter.
PY3=$(command -v "$PY"); mkdir -p "$A/win"
printf '#!/usr/bin/env bash\necho "Python was not found" >&2; exit 49\n' > "$A/win/python3"; chmod +x "$A/win/python3"
o=$(cd "$A/repo" && PATH="$A/win:$PATH" CLAUDE_PROJECT_DIR="$A/repo" "$PY3" "$KIT/adapters/claude/hook.py" session-start <<<"{}" 2>&1)
grep -q "advisor's last call failed" <<<"$o"; expect "windows: session-start status runs without a working python3" $? "$o"
o=$(cd "$A/repo" && PATH="$A/bin:$PATH" "$PY" "$KIT/scripts/advise" --backend claude --check 2>&1); rc=$?
[ "$rc" = 0 ] && grep -q "answers" <<<"$o"; expect "advise --check: answers after login" $? "rc=$rc $o"
o=$(cd "$A/repo" && bash "$KIT/scripts/status.sh"); ! grep -q "advisor" <<<"$o"; expect "status: quiet once the advisor answers" $? "$o"
o=$(cd "$A/repo" && PATH="$A/bin:$PATH" CLAUDE_LOGGED_OUT=1 "$PY" "$KIT/scripts/advise" --backend claude --check 2>&1); rc=$?
[ "$rc" = 3 ]; expect "advise --check: logged out -> exit 3" $? "rc=$rc $o"
mkdir -p "$A/repo/.advise-state" && touch "$A/repo/.advise-state/unavailable"
(cd "$A/repo" && PATH="$A/bin:$PATH" "$PY" "$KIT/scripts/advise" --backend claude --check >/dev/null 2>&1)
[ ! -e "$A/repo/.advise-state/unavailable" ]; expect "advise --check: success lifts the quiet period" $? ""
# codex echoes the prompt on stderr: login words in the user's failure must not read as a lost login
fakecli codex <<'SH'
#!/usr/bin/env bash
p=$(cat); printf 'user\n%s\nERROR: 400 model not supported\n' "$p" >&2; exit 1
SH
printf 'AssertionError: OAuth refresh got 401 unauthorized at /login\n' > "$A/err.txt"
o=$(cd "$A/repo" && PATH="$A/bin:$PATH" "$PY" "$KIT/scripts/advise" --backend codex --error-file "$A/err.txt" "why red?" 2>&1); rc=$?
[ "$rc" = 1 ] && ! grep -q "codex login" <<<"$o"; expect "advise: codex echo of auth words is not a lost login" $? "rc=$rc ${o: -200}"
rm -rf "$A"

echo ""; echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
