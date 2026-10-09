# PHIẾU P040: Bootstrap stack detection (`sos init security`)

> **Loại:** Feature (new public CLI surface — `sos init security` subcommand + parser skeleton library)
> **Ưu tiên:** P1 (foundation cho wave 1 — P041 + P042 depend)
> **Tầng:** 1 (móng nhà — new public CLI subcommand, new file format `.sos-stack.toml`, new `scripts/parsers/` library namespace, downstream phiếu depend on its interface)
> **Ảnh hưởng:** `bin/sos.sh` (extend dispatcher + new subcommand handler), `scripts/parsers/` (NEW dir, 6 stub files), `templates/.sos-stack.toml.example` (NEW), `README.md` (mention subcommand), `docs/SETUP.md` (subcommand reference)
> **Dependency:** None (foundation phiếu — first of wave 1)

---

## Context

### Vấn đề hiện tại

Active sprint "Tarot port wave 1" cần security pipeline skeleton port từ Tarot dogfood. Trinh sát (P041) + Giám sát (P042) đều cần biết stack hiện tại (Node/Python/Rust/Go) để invoke parser phù hợp + query GHSA/advisory đúng ecosystem. Chưa có bootstrap step nào write stack metadata vào project; mỗi phiếu downstream sẽ phải tự re-detect → drift + duplicate logic.

Gap thực tế:
- `bin/sos.sh` đã có Phase 0 (`sos init` vision capture) nhưng KHÔNG có security init step. `sos blueprint` Phase 1 chỉ pick stack manual qua Architect prompt — không write machine-readable file.
- Tarot's `.claude/agents/advisory-watch.md` đọc lockfile path hardcoded; muốn port generic → cần convention "đọc `.sos-stack.toml` để biết lockfile ở đâu".
- 6 parser tarot có (pnpm-lock-v9, package-lock-v3, requirements-txt, pyproject-toml, cargo-lock, go-sum) — tarot-specific path + tarot-specific output schema. Port về sos-kit cần generic skeleton trước, P041 fill khi cần.

### Giải pháp

Extend `bin/sos.sh` với subcommand mới `sos init security`. Subcommand:
1. Detect stack qua manifest file existence (priority order: `package.json` → `pyproject.toml` → `requirements.txt` → `Cargo.toml` → `go.mod`). Multi-stack repo support: detect tất cả manifest, write 1 entry mỗi stack.
2. Detect lock file tương ứng (`pnpm-lock.yaml` v9 / `package-lock.json` v3 / `Cargo.lock` / `go.sum` — `requirements.txt` itself acts as lock for pip projects).
3. Write `.sos-stack.toml` (machine-readable, root of project) với schema gọn — đủ cho P041/P042 consume, không over-engineer.
4. Drop 6 parser skeleton stub files vào `scripts/parsers/`. Mỗi file: 1 function `parse(path: Path) -> list[dict]` return `[]` + TODO comment. P041 sẽ fill implementation từng cái khi cần.

**Schema `.sos-stack.toml` (Architect propose, Worker xác nhận khi EXECUTE):**

```toml
# .sos-stack.toml — written by `sos init security`. Generic stack metadata.
# Read by Trinh sát (advisory-watch) + Giám sát (boundary-check) subagents.
# Schema version 1. Bump if breaking.

schema_version = 1
detected_at = "2026-05-25T10:30:00Z"  # UTC ISO-8601
sos_kit_version = "P040"              # phiếu that introduced or last refreshed this file

[[stack]]
type = "node"               # one of: node | python | rust | go
manifest = "package.json"   # relative path from project root
lock_file = "pnpm-lock.yaml" # relative path; "" if no lock file detected
lock_format = "pnpm-v9"     # one of: pnpm-v9 | npm-v3 | requirements-txt | pyproject-toml | cargo-lock | go-sum | "" (no lock)
parser = "scripts/parsers/pnpm_lock_v9.py"  # which parser file to invoke; "" if no parser available

# Multi-stack repo example (delete if mono-stack):
# [[stack]]
# type = "python"
# manifest = "pyproject.toml"
# lock_file = "poetry.lock"  # NOTE: poetry.lock not in P040 scope — parser stub absent, lock_format = ""
# lock_format = "pyproject-toml"
# parser = "scripts/parsers/pyproject_toml.py"
```

**Detection heuristic (Worker implements in bash):**
- For each known manifest file, `test -f` at project root.
- If manifest found, derive lock_file via priority:
  - `package.json` → `pnpm-lock.yaml` (if exists, lock_format = `pnpm-v9`) ELSE `package-lock.json` (lock_format = `npm-v3`) ELSE `""` (no lock).
  - `pyproject.toml` → `""` (no canonical lock in P040 — Poetry/PDM/uv all differ; defer parser to P041). lock_format = `pyproject-toml` (manifest itself acts as version source).
  - `requirements.txt` → `requirements.txt` (lock_format = `requirements-txt`; the file is both manifest + lock for pip projects).
  - `Cargo.toml` → `Cargo.lock` (lock_format = `cargo-lock`).
  - `go.mod` → `go.sum` (lock_format = `go-sum`).
- `parser` field = `scripts/parsers/<lock_format_underscored>.py` if parser stub exists, else `""`. (Filenames use underscores per Constraint #8 below — Python import compatibility for P041.)
- If no manifest found at root → write `.sos-stack.toml` with `schema_version = 1`, `detected_at`, `sos_kit_version`, ZERO `[[stack]]` entries + comment block explaining "no stack detected; add `[[stack]]` manually or run from a project with one of: package.json / pyproject.toml / requirements.txt / Cargo.toml / go.mod".

**Parser stub interface (Worker writes 6 files identical structure):**

```python
# scripts/parsers/<format>.py
# Parse <format> lock file into a generic dep list.
# Skeleton stub introduced in P040. Implementation deferred to P041 (advisory-watch consumer).
"""
Generic interface contract:
    parse(path: Path) -> list[dict]

Each dict in the returned list MUST have these keys:
    name: str          # package name (e.g. "react", "django", "serde")
    version: str       # resolved/pinned version (e.g. "18.2.0", "4.2.1")
    ecosystem: str     # one of: "npm" | "pypi" | "crates" | "go"
    source: str        # "direct" | "transitive" (best-effort; "transitive" OK if unsure)

Optional keys (parser may add, consumer may ignore):
    license: str | None
    integrity: str | None  # hash if available
"""
from pathlib import Path


def parse(path: Path) -> list[dict]:
    # TODO(P041): implement <format> parsing. See tarot's `.claude/agents/advisory-watch.md`
    # for reference parser semantics (strip tarot-specific paths when porting).
    _ = path  # silence unused-arg lint until P041 implements
    return []


if __name__ == "__main__":
    import sys
    if len(sys.argv) != 2:
        print(f"Usage: python {sys.argv[0]} <path-to-lock-file>", file=sys.stderr)
        sys.exit(1)
    deps = parse(Path(sys.argv[1]))
    print(f"Parsed {len(deps)} deps from {sys.argv[1]} (stub returns empty list — P040 skeleton)")
```

6 stub files (identical structure, different `<format>` in header comment + TODO ref) — **filenames use underscores** for Python module import compatibility (P041 will `import` these):
- `scripts/parsers/pnpm_lock_v9.py`
- `scripts/parsers/package_lock_v3.py`
- `scripts/parsers/requirements_txt.py`
- `scripts/parsers/pyproject_toml.py`
- `scripts/parsers/cargo_lock.py`
- `scripts/parsers/go_sum.py`

### Scope

- CHỈ sửa / tạo:
  - `bin/sos.sh` — extend `sos_init` dispatcher to handle `sos init security` subcommand; add `sos_init_security()` function.
  - `scripts/parsers/` (NEW dir) — 6 stub `.py` files.
  - `templates/.sos-stack.toml.example` (NEW) — schema example for users who want to inspect format or hand-author.
  - `README.md` — 1-2 sentence mention of `sos init security` in install/onboarding flow (where appropriate — Worker locates).
  - `docs/SETUP.md` — add subcommand reference in Quick Start section.
- KHÔNG sửa:
  - Existing `sos_init()` function logic (Phase 0 vision capture) — `sos init security` is a NEW branch, not a rewrite.
  - Any `agents/*.md` — orchestrator + worker + architect contracts unchanged.
  - `phieu/TICKET_TEMPLATE.md` — phiếu format unchanged.
  - `hooks/pre-commit` — no commit gating on `.sos-stack.toml` in P040.
  - Implement parser logic — all 6 files return `[]` stub. P041 fills.
  - GHSA query / advisory fetch — P041 scope.
  - INVARIANTS schema — P042 scope.
  - `.claude/commands/*.md` slash command files — P041/P042 scope.
  - `bootstrap/sos-rs/` (Rust port) — separate track, P033 in Next sprint.

---

## Task 0 — Verification Anchors

> Architect humility note: Architect Read `bin/sos.sh` end-to-end + globbed `scripts/parsers/` (empty) + globbed `templates/` (has 2 files). All `[verified]` anchors come from Architect Read in this DRAFT session. `[needs Worker verify]` items defer to Worker grep at EXECUTE because either (a) Architect cannot run bash to confirm runtime semantics, or (b) line numbers may drift between DRAFT and EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `bin/sos.sh` exists; dispatcher `sos()` at line ~319 switches on `$1` with cases `init / blueprint / contract / apply / recipe / launch / status / help` | `grep -n "^sos()" bin/sos.sh` + `sed -n '319,333p' bin/sos.sh` | ✅ [verified] — Architect Read full file 2026-05-25; dispatcher at line 319-333, cases 322-330 |
| 2 | `sos_init()` function at line ~78 currently only handles Phase 0 vision capture; does NOT accept args (uses `"$@"` but doesn't case-split) | `sed -n '78,100p' bin/sos.sh` | ✅ [verified] — Architect Read lines 78-100; function checks `docs/PROJECT.md` existence then echoes instructions, ignores args |
| 3 | `scripts/parsers/` directory does NOT exist at HEAD | `ls scripts/parsers/ 2>/dev/null` (empty or error) + `Glob("scripts/parsers/*")` | ✅ [verified] — Architect Glob `scripts/parsers/*` 2026-05-25 returned no files; only `scripts/architect-guard.sh`, `session-start-banner.sh`, `sync-personal-agents.sh` exist |
| 4 | `templates/` directory exists with current contents: `BACKLOG_template.md`, `claude-settings.local.json` (2 files) | `ls templates/` | ✅ [verified] — Architect Glob 2026-05-25 |
| 5 | No `.sos-stack.toml` anywhere in repo (greenfield, no prior schema to migrate) | `find . -name ".sos-stack.toml" -not -path "./node_modules/*"` | ✅ [verified] — Architect Glob `**/.sos-stack.toml` 2026-05-25 returned 0 files |
| 6 | `scripts/architect-guard.sh` blocks `.py` reads when marker active — implication: Architect could NOT have read parser stub examples even if they existed; Worker needs to author skeleton from spec in this phiếu, not from a reference file | `sed -n '58p' scripts/architect-guard.sh` (line listing `*.py` in BLOCKED case) | ✅ [verified] — Architect Read `scripts/architect-guard.sh` 2026-05-25; line 58 lists `*.py` in source-code BLOCKED case |
| 7 | `docs/SETUP.md` Quick Start has sections: 1. Install Rust tools, 2. Skills, 3. Phiếu, 4. Setup project, 5. Pre-commit hook, 6. Canary. `sos init security` reference fits in section 4 (project setup) or as a new sub-step | `grep -n "^### " docs/SETUP.md` | ✅ [verified] — Architect Read 2026-05-25, lines 5, 34, 59, 78, 129, 138 (sections 1-6) |
| 8 | `README.md` mentions `sos init` somewhere (onboarding flow); P040 adds `sos init security` as a sibling subcommand mention | `grep -n "sos init\|sos.sh" README.md` | ⚠️ [needs Worker verify] — Architect did NOT Read README.md in this DRAFT (cap on context). Worker greps to find the right insertion point. If README has a CLI subcommand table, add row; if only prose mention, add 1 sentence. Worker self-decides exact placement (Tầng 2 sentence-level wording), but the FACT that `sos init security` is now a valid subcommand must appear |
| 9 | Tarot's tarot-specific paths in tarot's `extract-pnpm-versions.py` (referenced in BACKLOG P040 spec) — Architect has NO access to tarot repo; cannot peek at tarot's existing parser implementations | (cannot verify from sos-kit alone) | ⚠️ [Architect cannot verify] — P040 ships GENERIC stubs (empty return). P041 (which already cites tarot port) will fill implementation. Worker DOES NOT need to consult tarot for P040 — the stubs are intentionally empty |
| 10 | Bash idiom for cross-platform UTC timestamp: `date -u +%Y-%m-%dT%H:%M:%SZ` works on macOS BSD `date` AND Linux GNU `date` (already used in `bin/sos.sh:51, 67, 170` etc.) | `grep -n "date -u" bin/sos.sh` | ✅ [verified] — Architect Read bin/sos.sh 2026-05-25, lines 51 + 67 + 170 + 277 + 302 all use the same idiom |
| 11 | The 5 manifest filenames are spelled correctly: `package.json`, `pyproject.toml`, `requirements.txt`, `Cargo.toml`, `go.mod` (case-sensitive on Linux) | (well-known) | ✅ [verified by ecosystem convention] |
| 12 | TOML multi-array syntax `[[stack]]` is the canonical way to express list-of-tables (TOML 1.0 spec); chosen over `stack = [{...}]` for readability when rendered | (well-known TOML feature) | ✅ [verified by TOML spec convention] |

**Summary:** 10 ✅ verified + 2 ⚠️ Worker spot-check (Anchor #8 README placement, Anchor #9 acknowledged cross-repo limit). No ❌. Worker proceeds with EXECUTE after CHALLENGE round.

### Pre-phiếu snapshot (Worker auto first-step)

> Worker EXECUTE FIRST ACTION (before any code edit, before Task 0 grep verification): take a rollback point so failed mid-execute can revert.

```bash
# Run from project root (worktree root for phiếu workflow):
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/ — auto-cleaned on phieu-done"
```

If the phiếu hits ❌ mid-execute: `cp .backup/${PHIEU_ID}/settings.local.json .claude/` and `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (within phiếu worktree only).

---

## Debate Log

> Tầng 1 phiếu — Worker MUST CHALLENGE before EXECUTE (per ORCHESTRATION.md Hard rule #7 / P036). New public CLI surface, new file schema, downstream phiếu (P041 + P042) will consume → mismatch here costs all 3 phiếu.

**Phiếu version:** V2 (Sếp-decided: parser filenames = underscores, not dashes)

### Turn 1 — Worker Challenge

*(Worker fills this when invoked in CHALLENGE mode. If no objections, write "Worker accepted V1 — no challenges. Ready for Chủ nhà approval." and skip to Final consensus.)*

**Anchor verification (recap from Task 0):**
- Anchor #N: ✅/⚠️/❌ + 1-line summary if ⚠️/❌

**Objections (Tầng 1 only — phiếu cần sửa):**
- [O1.1] Phiếu specifies DASHES (`pnpm-lock-v9.py` ×6) but Sếp constraint = UNDERSCORES. Architect punted to "Worker decision" but this is Tầng 1 schema/path contract (P041/P042 will hardcode `parser = ...` paths + Python `import`). Phiếu must resolve before EXECUTE.

**Proposed alternatives** (Worker recommends 1):
- A. Underscores (Worker lean — Python import compatibility for P041 agent script)

**Status:** ✅ ADDRESSED by Sếp out-of-band (2026-05-25)

### Turn 1 — Architect Response (phiếu V2)

- [O1.1] → **ACCEPT** — Sếp decision final: parser filenames use **underscores**, not dashes. Reason: P041 is a Python agent script that will `import` parser functions; dashes cause `SyntaxError` in `import` statements. BACKLOG dashes were descriptor cosmetic, not spec contract. Phiếu updated throughout: Task 3 file list (6 files), bash code block in Task 1 `Thêm 2` (6 `parser = ...` lines), "Files cần sửa" table (6 rows), schema example in Giải pháp section + `templates/.sos-stack.toml.example`. The "Worker decision" punt in Task 3 Validate has been replaced with a definitive DECIDED note. Added Constraint #8 making the underscore rule explicit and load-bearing for P041.

**Status:** ✅ RESPONDED — phiếu bumped to V2. Ready for Worker EXECUTE.

### Final consensus
- Phiếu version: V2
- Total turns: 1
- Approved by Chủ nhà: 2026-05-25 (underscore decision) — code execution may begin

---

## Nhiệm vụ

> Worker order: Task 0 (snapshot) → Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Nghiệm thu. Tasks 1+2 are independent; Tasks 3+4+5 depend on Task 1's subcommand existing so dry-run nghiệm thu has something to run.

### Task 1: Extend `bin/sos.sh` — new subcommand `sos init security`

**File:** `bin/sos.sh`

**Tìm 1** (the `sos_init()` function, currently around line 78-100, starts with `sos_init() {`):

The function as-is treats `sos init` as a single command (Phase 0 vision capture). We need to make it dispatch on the first arg — `security` triggers new flow, anything else (including no arg) keeps current Phase 0 behavior.

**Thay bằng 1** — refactor the dispatcher portion of `sos_init()`:

```bash
sos_init() {
  # Subcommand dispatch: `sos init security` → new bootstrap; `sos init` (no args) → Phase 0 vision capture (legacy)
  local subcmd="${1:-}"
  if [[ "$subcmd" == "security" ]]; then
    shift
    sos_init_security "$@"
    return $?
  fi

  # Legacy Phase 0 vision capture (unchanged behavior)
  if [[ -f docs/PROJECT.md ]]; then
    echo "⚠ docs/PROJECT.md already exists. This project is past phase 0."
    echo "  Use '/insight' skill to refine, or 'sos status' to see current phase."
    return 1
  fi

  sos_state_init
  echo "─────────────────────────────────────────"
  echo "Phase 0 — Vision Capture"
  echo "─────────────────────────────────────────"
  echo ""
  echo "Open Claude Code in this directory and run skill /init."
  echo ""
  echo "The /init skill will:"
  echo "  1. Ask 3 questions (project type, persona, pitch)"
  echo "  2. Generate docs/PROJECT.md, docs/SOUL.md (if persona), docs/CHARACTER.md (if persona)"
  echo "  3. Initialize phiếu workflow (.phieu-counter, docs/ticket/, docs/DISCOVERIES.md)"
  echo "  4. Copy phiếu/GENESIS_TEMPLATE.md → docs/ticket/P000-genesis.md (draft)"
  echo "  5. Update .sos/state.toml → phase = VISION_CAPTURED"
  echo ""
  echo "After /init done: 'sos blueprint' to continue Phase 1."
}
```

**Lưu ý 1:** Preserve the existing Phase 0 echo block byte-for-byte (it's documented in INSTALL.md + skill `/init`). Only ADD the subcmd dispatch at the top.

**Tìm 2** — locate where to add new function `sos_init_security()`. Insertion point: AFTER `sos_init()` closing brace, BEFORE `sos_blueprint()` (which starts ~line 102 currently).

**Thêm 2** — new function:

```bash
sos_init_security() {
  # Detect stack(s) by manifest file existence, write .sos-stack.toml.
  # Foundation for advisory-scan (P041) + security-review (P042).

  if [[ -f .sos-stack.toml ]]; then
    echo "⚠ .sos-stack.toml already exists at project root."
    echo "  Inspect with: cat .sos-stack.toml"
    echo "  To refresh: delete and re-run 'sos init security'."
    return 1
  fi

  local ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  local stacks_found=0
  local tmpfile
  tmpfile="$(mktemp)"

  # Header
  cat > "$tmpfile" <<EOF
# .sos-stack.toml — written by 'sos init security' on $ts
# Generic stack metadata. Consumed by advisory-scan (P041) + security-review (P042).
# Schema version 1. Bump if breaking.

schema_version = 1
detected_at = "$ts"
sos_kit_version = "P040"

EOF

  # Node — package.json
  if [[ -f package.json ]]; then
    local node_lock=""
    local node_format=""
    local node_parser=""
    if [[ -f pnpm-lock.yaml ]]; then
      node_lock="pnpm-lock.yaml"
      node_format="pnpm-v9"
      node_parser="scripts/parsers/pnpm_lock_v9.py"
    elif [[ -f package-lock.json ]]; then
      node_lock="package-lock.json"
      node_format="npm-v3"
      node_parser="scripts/parsers/package_lock_v3.py"
    fi
    cat >> "$tmpfile" <<EOF
[[stack]]
type = "node"
manifest = "package.json"
lock_file = "$node_lock"
lock_format = "$node_format"
parser = "$node_parser"

EOF
    stacks_found=$((stacks_found + 1))
  fi

  # Python — pyproject.toml (preferred) OR requirements.txt
  if [[ -f pyproject.toml ]]; then
    cat >> "$tmpfile" <<EOF
[[stack]]
type = "python"
manifest = "pyproject.toml"
lock_file = ""
lock_format = "pyproject-toml"
parser = "scripts/parsers/pyproject_toml.py"

EOF
    stacks_found=$((stacks_found + 1))
  fi
  if [[ -f requirements.txt ]]; then
    cat >> "$tmpfile" <<EOF
[[stack]]
type = "python"
manifest = "requirements.txt"
lock_file = "requirements.txt"
lock_format = "requirements-txt"
parser = "scripts/parsers/requirements_txt.py"

EOF
    stacks_found=$((stacks_found + 1))
  fi

  # Rust — Cargo.toml
  if [[ -f Cargo.toml ]]; then
    local rust_lock=""
    if [[ -f Cargo.lock ]]; then
      rust_lock="Cargo.lock"
    fi
    cat >> "$tmpfile" <<EOF
[[stack]]
type = "rust"
manifest = "Cargo.toml"
lock_file = "$rust_lock"
lock_format = "cargo-lock"
parser = "scripts/parsers/cargo_lock.py"

EOF
    stacks_found=$((stacks_found + 1))
  fi

  # Go — go.mod
  if [[ -f go.mod ]]; then
    local go_lock=""
    if [[ -f go.sum ]]; then
      go_lock="go.sum"
    fi
    cat >> "$tmpfile" <<EOF
[[stack]]
type = "go"
manifest = "go.mod"
lock_file = "$go_lock"
lock_format = "go-sum"
parser = "scripts/parsers/go_sum.py"

EOF
    stacks_found=$((stacks_found + 1))
  fi

  # No stack found — write empty .sos-stack.toml with hint
  if [[ "$stacks_found" -eq 0 ]]; then
    cat >> "$tmpfile" <<'EOF'
# No stack manifest detected at project root.
# Expected one of: package.json / pyproject.toml / requirements.txt / Cargo.toml / go.mod
# Add [[stack]] entries manually if your project layout is non-standard, or
# re-run 'sos init security' from a directory containing one of the above.
EOF
  fi

  mv "$tmpfile" .sos-stack.toml

  echo "✓ .sos-stack.toml written (${stacks_found} stack(s) detected)"
  if [[ "$stacks_found" -gt 0 ]]; then
    echo ""
    echo "Next:"
    echo "  - Inspect: cat .sos-stack.toml"
    echo "  - Run advisory scan (when P041 ships): /advisory-scan"
    echo "  - Run boundary check (when P042 ships): /security-review <PR>"
  else
    echo ""
    echo "⚠ No stack detected. See .sos-stack.toml comments for next steps."
    return 2
  fi
}
```

**Lưu ý 2:**
- Uses `mktemp` + `mv` for atomic write (avoid partial-write on Ctrl-C). Pattern matches `sos_state_set_phase()` at line 70 which does similar tmp+mv.
- Echoes match existing `sos.sh` voice: terse, `✓` for success, `⚠` for warning, `─` separator only for major phase transitions.
- Return codes: 0 = success with stacks, 1 = file already exists, 2 = no stack detected (still wrote empty file with hint). Worker may pick different codes if existing `bin/sos.sh` convention differs — Tầng 2 self-decide.
- DO NOT auto-create `scripts/parsers/` here. That's Task 3. If user runs `sos init security` before parsers exist, `parser = "scripts/parsers/<format>.py"` is still written as a reference path; P041 will create real parsers when needed. The TOML is just metadata.
- **Parser path strings MUST use underscores** (`pnpm_lock_v9.py`, etc.) — see Constraint #8. The 6 path literals above are fixed; do not "normalize" them back to dashes.

**Tìm 3** — `sos_help()` function (line 22-43). Help text lists subcommands.

**Thay bằng 3** — insert `sos init security` mention in the help text. After line 27 (`sos init`):

```bash
  sos init                     Phase 0 — vision capture (Chủ nhà)
  sos init security            Bootstrap stack detection — write .sos-stack.toml (foundation for advisory-scan / security-review)
  sos blueprint                Phase 1 — pick stack + recipes (Chủ nhà → Kiến trúc sư)
```

(Insert the new line; keep all other help text unchanged. Indentation matches existing 2-space style.)

**Validate (Task 1):**
- `bash -n bin/sos.sh` (syntax check) exits 0.
- `source bin/sos.sh && type sos_init_security` shows function definition.
- In a scratch dir with NO manifest: `sos init security` → exits 2, creates `.sos-stack.toml` with header + no `[[stack]]` blocks + hint comment.
- In a scratch dir with `package.json` + `pnpm-lock.yaml`: `sos init security` → exits 0, `.sos-stack.toml` contains 1 `[[stack]]` with `type = "node"`, `lock_format = "pnpm-v9"`, `parser = "scripts/parsers/pnpm_lock_v9.py"`.
- Re-run in same dir: exits 1 with "already exists" message.

---

### Task 2: `.sos-stack.toml.example` template

**File:** `templates/.sos-stack.toml.example` (NEW)

**Thêm (full file content):**

```toml
# .sos-stack.toml — example for inspection or manual authoring.
# Auto-generated by `sos init security` in a real project.
# Consumers: advisory-scan (P041), security-review (P042) subagents.

schema_version = 1
detected_at = "2026-05-25T10:30:00Z"
sos_kit_version = "P040"

# Mono-stack Node + pnpm example
[[stack]]
type = "node"               # one of: node | python | rust | go
manifest = "package.json"   # relative path from project root
lock_file = "pnpm-lock.yaml"
lock_format = "pnpm-v9"     # one of: pnpm-v9 | npm-v3 | requirements-txt | pyproject-toml | cargo-lock | go-sum | ""
parser = "scripts/parsers/pnpm_lock_v9.py"  # invoked by P041; "" if no parser available

# Multi-stack example (uncomment + adapt for monorepo):
# [[stack]]
# type = "python"
# manifest = "pyproject.toml"
# lock_file = ""              # P040 does not pin Poetry/PDM/uv lock — defer to P041
# lock_format = "pyproject-toml"
# parser = "scripts/parsers/pyproject_toml.py"

# Rust example:
# [[stack]]
# type = "rust"
# manifest = "Cargo.toml"
# lock_file = "Cargo.lock"
# lock_format = "cargo-lock"
# parser = "scripts/parsers/cargo_lock.py"

# Go example:
# [[stack]]
# type = "go"
# manifest = "go.mod"
# lock_file = "go.sum"
# lock_format = "go-sum"
# parser = "scripts/parsers/go_sum.py"
```

**Lưu ý:**
- Filename uses `.example` suffix to make it readable in repo (not gitignored) and clearly distinct from real `.sos-stack.toml` in user projects.
- Not auto-copied by `sos init security` — `bin/sos.sh` writes the real file fresh. This is a docs artifact only.

**Validate (Task 2):**
- File created at `templates/.sos-stack.toml.example`.
- `python -c "import tomllib; tomllib.loads(open('templates/.sos-stack.toml.example').read())"` parses cleanly (note: commented `[[stack]]` blocks are TOML comments, only the one uncommented `[[stack]]` is parsed) — exits 0. (If Python <3.11, use `tomli` package or skip — Worker self-decide.)

---

### Task 3: 6 parser skeleton stubs in `scripts/parsers/`

**Files (NEW)** — filenames use **underscores** (Python import compatibility, per Constraint #8 — DECIDED, not optional):
1. `scripts/parsers/pnpm_lock_v9.py`
2. `scripts/parsers/package_lock_v3.py`
3. `scripts/parsers/requirements_txt.py`
4. `scripts/parsers/pyproject_toml.py`
5. `scripts/parsers/cargo_lock.py`
6. `scripts/parsers/go_sum.py`

**Thêm (template — apply to each file with `<format>` replaced):**

```python
# scripts/parsers/<format>.py
# Parse <format> lock/manifest file into a generic dep list.
# Skeleton stub introduced in P040. Implementation deferred to P041 (advisory-watch consumer).
"""
Generic interface contract (P040):

    parse(path: Path) -> list[dict]

Each dict in the returned list MUST have these keys:
    name: str          # package name (e.g. "react", "django", "serde")
    version: str       # resolved/pinned version (e.g. "18.2.0", "4.2.1")
    ecosystem: str     # one of: "npm" | "pypi" | "crates" | "go"
    source: str        # "direct" | "transitive" (best-effort; "transitive" OK if unsure)

Optional keys (parser may add, consumer may ignore):
    license: str | None
    integrity: str | None  # hash if available

P040 ships an empty-list stub. P041 fills implementation.
"""
from pathlib import Path


def parse(path: Path) -> list[dict]:
    # TODO(P041): implement <format> parsing.
    # Reference: tarot's `.claude/agents/advisory-watch.md` documents the parser
    # contract; port logic without tarot-specific path assumptions.
    _ = path  # silence unused-arg lint until P041 implements
    return []


if __name__ == "__main__":
    import sys
    if len(sys.argv) != 2:
        print(f"Usage: python {sys.argv[0]} <path-to-lock-file>", file=sys.stderr)
        sys.exit(1)
    deps = parse(Path(sys.argv[1]))
    print(f"Parsed {len(deps)} deps from {sys.argv[1]} (stub returns empty list — P040 skeleton)")
```

**Per-file `<format>` substitutions (header comment + TODO ref):**

| File | `<format>` |
|------|-----------|
| `pnpm_lock_v9.py` | `pnpm-lock.yaml v9 (YAML)` |
| `package_lock_v3.py` | `package-lock.json v3 (npm JSON)` |
| `requirements_txt.py` | `pip requirements.txt` |
| `pyproject_toml.py` | `Python pyproject.toml (PEP 621 / Poetry)` |
| `cargo_lock.py` | `Cargo.lock (Rust TOML)` |
| `go_sum.py` | `go.sum (Go modules)` |

**Lưu ý:**
- All 6 files identical structure except for the 2 `<format>` literal substitutions. Worker may write a helper script to generate them OR write each by hand — Tầng 2 self-decide.
- `_ = path` line is intentional. P040 stubs MUST NOT raise / error; they must return `[]` so downstream consumers don't crash before P041 lands.
- DO NOT add `__init__.py` to `scripts/parsers/` — keep flat, each file standalone, P041 may add packaging later if needed.
- Python version target: 3.10+ (`list[dict]` syntax). If sos-kit users on 3.9 → P041 backports. Architect chooses 3.10+ because tarot already on Python 3.11+; matching ecosystem assumption.

**Validate (Task 3):**
- All 6 files exist at correct path with **underscore** filenames (`pnpm_lock_v9.py`, etc. — NOT dashes).
- For each file: `python -c "from scripts.parsers.<underscore_name> import parse; from pathlib import Path; assert parse(Path('/dev/null')) == []"` exits 0. (Example: `from scripts.parsers.pnpm_lock_v9 import parse`.) **DECIDED (Sếp, 2026-05-25):** underscores, not dashes — Python `import` syntax rejects dashes; P041 will be a Python agent script that imports these parsers. Filenames in `bin/sos.sh` `parser = "scripts/parsers/<name>.py"` strings must match (already done in Task 1 spec).
- Linter `python -m py_compile scripts/parsers/*.py` exits 0 for all 6.

---

### Task 4: `docs/SETUP.md` — add `sos init security` reference

**File:** `docs/SETUP.md`

**Tìm** (section "### 4. Setup each project you want SOS Kit to run on" — currently around line 78-125, ends before "### 5. Install pre-commit hook" at line 129):

The section currently has steps 4a through 4g. Add a new step 4h after 4g for security init.

**Thay bằng / Thêm** — insert before the closing paragraph "After these steps, your project is ready..." (line ~127):

```markdown
# 4h. Initialize security pipeline metadata (P040+)
# Detects stack (Node/Python/Rust/Go) via manifest files, writes .sos-stack.toml.
# Required before /advisory-scan (P041) or /security-review (P042) — those subagents
# read .sos-stack.toml to know which parser + which ecosystem to query.
sos init security
```

**Lưu ý:**
- Inserted as step 4h to match the existing 4a-4g numbering. Don't renumber 5+ (pre-commit + canary keep their numbers).
- One-line bash + 3-line comment matches existing 4a-4g style (each has a comment then 1-2 commands).
- If Worker thinks heading restructure (Step 7 standalone "Security init") reads cleaner, that's Tầng 2 doc structure — Worker self-decides, but minimal-diff (4h) is recommended.

**Validate (Task 4):**
- `grep -c "sos init security" docs/SETUP.md` → ≥1.
- Visual scan: section 4 ends cleanly, no broken markdown.

---

### Task 5: `README.md` — mention `sos init security` in onboarding

**File:** `README.md`

**Tìm** — Worker greps `grep -n "sos init\|sos.sh\|sos blueprint" README.md` to find where the CLI subcommands are introduced. Likely candidates:
- A "Quick Start" section
- A subcommand list / table
- Onboarding flow description

**Thay bằng / Thêm:**
- If README has a subcommand TABLE: add 1 row for `sos init security` with brief description "Detect stack + write `.sos-stack.toml` (foundation for security pipeline)".
- If README only has PROSE: add 1 sentence after the closest `sos init` mention, e.g.:
  > After `sos init`, optionally run `sos init security` to bootstrap stack detection for the advisory-scan + security-review subagents (introduced in P040; consumed by P041 + P042).

**Lưu ý:**
- Tầng 2 doc wording — Worker self-decide exact phrasing. The REQUIREMENT is that `sos init security` exists as a discoverable subcommand in README (otherwise users won't know to run it).
- If README structure changed significantly since Architect's last glance → escalate format question to Sếp via Discovery (not blocking — Worker can pick best-fit insertion).

**Validate (Task 5):**
- `grep -c "sos init security" README.md` → ≥1.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `bin/sos.sh` | Task 1: extend `sos_init()` dispatcher + new `sos_init_security()` function (~80 LOC) + 1 line in `sos_help()` |
| `templates/.sos-stack.toml.example` | NEW — Task 2: example schema for inspection/hand-authoring |
| `scripts/parsers/pnpm_lock_v9.py` | NEW — Task 3: stub returning `[]` |
| `scripts/parsers/package_lock_v3.py` | NEW — Task 3: stub returning `[]` |
| `scripts/parsers/requirements_txt.py` | NEW — Task 3: stub returning `[]` |
| `scripts/parsers/pyproject_toml.py` | NEW — Task 3: stub returning `[]` |
| `scripts/parsers/cargo_lock.py` | NEW — Task 3: stub returning `[]` |
| `scripts/parsers/go_sum.py` | NEW — Task 3: stub returning `[]` |
| `docs/SETUP.md` | Task 4: add step 4h in section "Setup each project" |
| `README.md` | Task 5: mention `sos init security` in onboarding/subcommand area |

(10 files total — 1 modified, 9 new.)

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md` | No edits. State machine + role contracts unchanged. |
| `phieu/TICKET_TEMPLATE.md` | No edits. Phiếu format unchanged. |
| `hooks/pre-commit` | No edits. P040 does not gate commits on `.sos-stack.toml`. |
| `scripts/architect-guard.sh` | No edits. Continues blocking `.py` reads when marker active — that means the 6 new parser files in `scripts/parsers/` will be UNREADABLE by Architect after P040 ships. This is correct: Architect should specify parser interface in phiếu (P041), Worker reads + edits Python files. |
| `bootstrap/sos-rs/` | No edits. Rust port is P033 (Next sprint), not wave 1. |
| `.claude/commands/*.md` | No new slash commands in P040. P041 adds `/advisory-scan`, P042 adds `/security-review`. |
| `configs/*.toml` (existing ship/docs-gate configs) | No edits. P040 introduces a NEW separate file `.sos-stack.toml` that lives at project root, not in `configs/`. |

---

## Luật chơi (Constraints)

1. **Tier locked at 1 (móng nhà).** Must complete CHALLENGE round before EXECUTE (P036 Hard rule #7). If CHALLENGE surfaces a Tầng 2 sub-issue (e.g., exact help-text wording) Worker self-decides and logs to Discovery — only Tầng 1 issues (schema shape, subcommand contract, manifest-detection priority order, parser interface) need ACCEPT/DEFEND from Architect in RESPOND.
2. **Schema is the contract.** `.sos-stack.toml` keys `schema_version` / `detected_at` / `sos_kit_version` / `[[stack]]` with `type` / `manifest` / `lock_file` / `lock_format` / `parser` are FIXED in P040. P041 + P042 will read these exact keys. Renaming a key = breaking change requiring schema_version bump + Sếp approval.
3. **Parser stubs return `[]` only.** P040 MUST NOT ship parser implementations even if Worker is tempted ("just for pnpm because it's easy"). Reason: parser implementation requires advisory-format contract (P041 owns) — implementing partially in P040 creates two sources of truth.
4. **No new dependencies.** `bin/sos.sh` uses only POSIX bash + `date -u` + `mktemp` + `mv` (all already used). Python stubs use only stdlib (`pathlib`). NO `pip install` / `npm install` introduced. If Worker discovers a parser stub needs a third-party lib (e.g. `tomli` for Python <3.11) → defer that to P041, NOT P040.
5. **Idempotency:** `sos init security` checks for `.sos-stack.toml` existence and refuses to overwrite (return 1). Refresh requires manual delete. This protects against accidental clobber of hand-edited multi-stack TOMLs.
6. **No auto-mkdir for `scripts/parsers/`.** `bin/sos.sh` does NOT create the parser directory; it only writes the path as metadata. The directory is created by Task 3 (Worker editing this phiếu). Reason: if a user installs sos-kit fresh and runs `sos init security`, the parser dir+stubs are already in the installed kit — no runtime mkdir needed.
7. **`scripts/architect-guard.sh` continues to block Architect's `.py` reads.** This is intentional. The 6 stubs are simple enough that Architect specs in this phiếu (with exact code in `Thêm` block) — Worker writes. Future stub edits (P041) follow same pattern: Architect specs interface in phiếu, Worker implements.
8. **Parser filenames MUST use underscores** (`pnpm_lock_v9.py`, `package_lock_v3.py`, `requirements_txt.py`, `pyproject_toml.py`, `cargo_lock.py`, `go_sum.py`) — Python module import compatibility for P041 agent script (dashes are invalid in `import` statements). DECIDED by Sếp 2026-05-25; not Worker-discretionary. `bin/sos.sh` `parser = "scripts/parsers/<name>.py"` literals + `templates/.sos-stack.toml.example` paths must match exactly.

---

## Nghiệm thu

### Automated
- [ ] `bash -n bin/sos.sh` exits 0 (syntax check).
- [ ] `source bin/sos.sh && type sos_init_security` shows the function definition.
- [ ] All 6 `scripts/parsers/*.py` pass `python -m py_compile`.
- [ ] `templates/.sos-stack.toml.example` parses as valid TOML (Python `tomllib` or any TOML parser exits 0).
- [ ] `git diff --stat` shows ≤10 files changed/added (matches "Files cần sửa" exactly).

### Manual Testing (dry-run)
- [ ] **Mono-stack Node test.** In a scratch dir with `package.json` + `pnpm-lock.yaml`:
  ```bash
  mkdir -p /tmp/sos-p040-test-node && cd /tmp/sos-p040-test-node
  echo '{"name":"t"}' > package.json && touch pnpm-lock.yaml
  source ~/sos-kit/bin/sos.sh
  sos init security
  cat .sos-stack.toml
  ```
  Expect: file exists, `[[stack]]` block with `type = "node"`, `lock_format = "pnpm-v9"`, `parser = "scripts/parsers/pnpm_lock_v9.py"`, exit 0.
- [ ] **Mono-stack Python (requirements.txt) test.** Same pattern with `requirements.txt`:
  ```bash
  mkdir -p /tmp/sos-p040-test-py && cd /tmp/sos-p040-test-py
  touch requirements.txt
  source ~/sos-kit/bin/sos.sh
  sos init security
  ```
  Expect: `[[stack]]` with `type = "python"`, `lock_format = "requirements-txt"`, `parser = "scripts/parsers/requirements_txt.py"`.
- [ ] **No-stack test.** Empty dir → `sos init security` writes header-only `.sos-stack.toml` with no `[[stack]]`, exits 2, prints "no stack detected" hint.
- [ ] **Multi-stack test.** Dir with `package.json` + `Cargo.toml`: writes 2 `[[stack]]` blocks (Node + Rust), exits 0, count message says "2 stack(s)".
- [ ] **Idempotency test.** Run `sos init security` twice in same dir: 2nd run exits 1 with "already exists" message; file unchanged (`stat .sos-stack.toml` mtime same as after first run).
- [ ] **Parser stub invocation test.** `python scripts/parsers/pnpm_lock_v9.py /dev/null` exits 0, prints "Parsed 0 deps from /dev/null (stub returns empty list — P040 skeleton)".

### Regression
- [ ] `sos init` (no args) still triggers Phase 0 vision capture — echoes same legacy message about `/init` skill. `grep -c "Phase 0 — Vision Capture" bin/sos.sh` ≥1 and runtime output unchanged from pre-P040.
- [ ] `sos blueprint`, `sos contract`, `sos apply`, `sos recipe`, `sos launch`, `sos status`, `sos help` — all behave exactly as before. No subcommand other than `init` changed.
- [ ] `sos help` output now includes `sos init security` line (positive regression).
- [ ] `scripts/architect-guard.sh` unchanged. `git diff scripts/architect-guard.sh` → empty.
- [ ] No `.claude/settings.json` or `.claude/settings.local.json` edits required for P040. (Note for Worker: if Bash ops like `mkdir -p .sos-state` or generic file writes trigger permission prompts during dry-run testing in scratch dirs, that's environment-specific, NOT a P040 regression.)

### Docs Gate
- [ ] `CHANGELOG.md` — new entry at top: "P040: bootstrap stack detection — `sos init security` subcommand auto-detects Node/Python/Rust/Go via manifest files, writes `.sos-stack.toml` schema. Adds 6 parser skeleton stubs at `scripts/parsers/` (all return `[]`; P041 fills implementations). Foundation for advisory-scan (P041) + security-review (P042)."
- [ ] `docs/SETUP.md` — step 4h added (Task 4).
- [ ] `README.md` — `sos init security` mentioned in onboarding/subcommand area (Task 5).
- [ ] `docs/BACKLOG.md` — P040 row moved from Active sprint to "Recently shipped" (Sếp/orchestrator handles, not Worker).

### Discovery Report
- [ ] Write to `docs/discoveries/P040.md` (per-phiếu file, P038 pattern):
  - **Schema final shape.** Did Worker keep all keys (`schema_version`, `detected_at`, `sos_kit_version`, `[[stack]]` with 5 keys) as specified? Any adjustment during EXECUTE? If yes, why + downstream P041/P042 impact note.
  - **Parser filename convention.** DECIDED V2: underscores. Worker confirm all 6 files landed as `pnpm_lock_v9.py` / `package_lock_v3.py` / `requirements_txt.py` / `pyproject_toml.py` / `cargo_lock.py` / `go_sum.py` AND that `bin/sos.sh` `parser = ...` strings + `templates/.sos-stack.toml.example` paths all match. If any drift, flag for P041 consumer.
  - **Anchor #8 (README) resolution.** Where exactly did `sos init security` land in README? Table row or prose sentence? Quote the inserted text.
  - **Detection priority order at EXECUTE.** Did the order `package.json → pyproject.toml → requirements.txt → Cargo.toml → go.mod` feel right, or did multi-stack edge case (e.g. Python project with BOTH `pyproject.toml` AND `requirements.txt`) expose ambiguity? If Worker chose to detect both as 2 separate `[[stack]]` entries (current spec) vs merging — document the call.
  - **Idempotency check feeling.** Exit 1 on existing `.sos-stack.toml` — too strict (should refresh-with-confirm flag exist later)? Signal for P041/P042 calibration.
  - **CHALLENGE round value.** Was the CHALLENGE round (Worker → Architect) for this Tầng 1 phiếu valuable? Did it catch anything? Honest signal for P036 retrospective. (Note: V1→V2 bump was triggered by exactly this round catching the dash/underscore punt.)
  - **Total time + tokens.** Architect estimate: half-day. Actual? Tier-1 dogfood data point.
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
