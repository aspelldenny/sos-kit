#!/usr/bin/env bash
# sos — 0→1 bootstrap (bash MVP — Rust port at bootstrap/sos-rs/)
#
# Source from your shell (or symlink to /usr/local/bin/sos):
#   source /path/to/sos-kit/bin/sos.sh
#   # or
#   ln -s /path/to/sos-kit/bin/sos.sh /usr/local/bin/sos
#
# Subcommands:
#   sos init        — Phase 0: vision capture (delegates to /init skill)
#   sos blueprint   — Phase 1: stack picker + recipe list
#   sos contract    — Phase 2: lock P000-genesis.md with spec_hash
#   sos apply NAME  — Phase 3: apply 1 recipe (delegates to /apply skill)
#   sos launch      — Phase N+1: gate against LAUNCH_CHECKLIST 100%
#   sos status      — show .sos/state.toml summary
#   sos help        — print this help

set -euo pipefail

SOS_KIT_DIR="${SOS_KIT_DIR:-$(dirname "$(realpath "${BASH_SOURCE[0]:-$0}")")/..}"

sos_help() {
  cat <<'EOF'
sos — 0→1 bootstrap for SOS Kit

Usage:
  sos new <dir> --stack <python|rust|ts>   Bootstrap a NEW (empty) repo from golden (freeze + skeleton + validate)
  sos adopt <dir> [--stack ...]            Retrofit spine into an EXISTING repo (additive + non-clobber + report)
  sos sync <dir>                           Re-sync spine into an ADOPTED repo (KIT-LAG cure: take-newer unmodified, flag customized)
  sos map [dir]                            Scan repo → draft AGENT_MAP with REAL surfaces (sound) + load_bearing/blast TODO (you)
  sos init                     Phase 0 — vision capture (Chủ nhà)
  sos init security            Bootstrap stack detection — write .sos-stack.toml (foundation for advisory-scan / security-review)
  sos blueprint                Phase 1 — pick stack + recipes (Chủ nhà → Kiến trúc sư)
  sos contract                 Phase 2 — lock P000-genesis.md (Kiến trúc sư)
  sos apply <category>/<name>  Phase 3 — apply 1 recipe (Thợ)
  sos apply --all              Apply all recipes from P000-genesis.md in order
  sos recipe new <category>/<name>   Forge new recipe (Kiến trúc sư)
  sos launch                   Phase N+1 — launch gate (Chủ nhà)
  sos status                   Show .sos/state.toml summary
  sos help                     This help

State: .sos/state.toml
Genesis phiếu: docs/ticket/P000-genesis.md
Recipe library: $SOS_KIT_DIR/recipes/

Env:
  SOS_KIT_DIR    Path to sos-kit checkout (default: dir of this script's parent).
  DOCTOR_BIN     Path to the `doctor` binary used by `sos new`'s verify-setup gate
                 (default: `doctor` on PATH; set this to point at a local build).

For deeper docs: cat $SOS_KIT_DIR/docs/GENESIS.md
EOF
}

sos_state_init() {
  mkdir -p .sos
  if [[ ! -f .sos/state.toml ]]; then
    cat > .sos/state.toml <<EOF
[state]
phase = "INIT"
created_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
last_updated = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# applied_recipes = []
# history = []
EOF
    echo "✓ Created .sos/state.toml"
  fi
}

sos_state_get_phase() {
  grep '^phase = ' .sos/state.toml 2>/dev/null | sed 's/phase = "\(.*\)"/\1/' || echo "INIT"
}

sos_state_set_phase() {
  local phase="$1"
  local ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if grep -q '^phase = ' .sos/state.toml; then
    # Cross-platform sed (Linux + BSD/macOS): write to temp then mv
    sed "s/^phase = \".*\"/phase = \"$phase\"/" .sos/state.toml > .sos/state.toml.tmp
    mv .sos/state.toml.tmp .sos/state.toml
    sed "s/^last_updated = \".*\"/last_updated = \"$ts\"/" .sos/state.toml > .sos/state.toml.tmp
    mv .sos/state.toml.tmp .sos/state.toml
  fi
  echo "✓ State → $phase"
}

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

sos_emit_map_stub() {
  # <out-path> — write an UNMAPPED draft AGENT_MAP (NO tarot content). For greenfield (nothing
  # to scan) or when scan finds no surfaces. Tells the reader to scan/fill, never lies.
  local out="$1"; mkdir -p "$(dirname "$out")"
  cat > "$out" <<'STUB'
# AGENT_MAP — UNMAPPED DRAFT. status: draft_unmapped.
# This repo has no mapped surfaces yet. Do NOT route phiếu by this file until filled.
#   Brownfield (existing code): run `sos map .` to scan-generate real surfaces.
#   Greenfield: add surfaces as you build them. configs/AGENT_MAP.example.yaml is a SHAPE
#     reference (schema), NOT content to copy — copying its tarot surfaces = "map nói dối".
version: 1
status: draft_unmapped
surfaces: {}
STUB
}

sos_map() {
  # sos map [target-dir] — scan an EXISTING repo → draft AGENT_MAP with REAL surfaces.
  # SOUND half (auto): real dirs/files of THIS repo, coarse-grouped by directory pattern.
  # PARTIAL half (human): load_bearing + blast left NEEDS_JUDGMENT. Does NOT copy the tarot
  # example (the bug that prompted this — sos adopt was dropping tarot CONTENT as a fake map).
  # Doctrine: docs/BOOTSTRAP_AUTOMATION_DRAFT.md §8 (scan-grounded; sound/partial, like docs-gate init).
  local target="${1:-.}"
  if [[ ! -d "$target" ]]; then echo "✗ $target is not a directory"; return 1; fi
  mkdir -p "$target/docs"
  local out="$target/docs/AGENT_MAP.yaml"
  local _t="$target"

  scan_files() {  # find-args... → newline list of relpaths (noise excluded, capped)
    { find "$_t" "$@" \
        -not -path '*/.git/*' -not -path '*/node_modules/*' -not -path '*/__pycache__/*' \
        -not -path '*/.sos-adopt-incoming/*' -not -path '*/migrations/versions/*' \
        -not -path '*/.venv/*' -not -path '*/venv/*' -not -path '*/dist/*' -not -path '*/build/*' \
        2>/dev/null | sed "s|^${_t}/||" | sort | head -25; } || true
  }

  local surfaces=""
  add_surface() {  # <name> <relpath-list>
    local name="$1" list="$2"
    [[ -z "$list" ]] && return 0
    surfaces="${surfaces}  ${name}:
    load_bearing: true                # NEEDS_JUDGMENT — confirm: true=architect reads deep, false=leaf. Default true = safe (over-read beats miss-blast).
    edit:
$(printf '%s\n' "$list" | sed 's|^|      - |')
    blast: \"TODO: what breaks if this changes — describe blast radius (you).\"
"
  }

  add_surface "routes_handlers" "$(scan_files -type f \( -path '*/routes/*' -o -path '*/handlers/*' -o -path '*/views/*' -o -path '*/controllers/*' -o -path '*/api/*' \) \( -name '*.py' -o -name '*.ts' -o -name '*.js' -o -name '*.rs' \))"
  add_surface "models_schema"   "$(scan_files -type f \( -path '*/models/*' -o -path '*/entities/*' -o -name 'schema.*' \) \( -name '*.py' -o -name '*.ts' -o -name '*.rs' \))"
  add_surface "services_logic"  "$(scan_files -type f \( -path '*/services/*' -o -path '*/lib/*' \) \( -name '*.py' -o -name '*.ts' -o -name '*.rs' \))"
  add_surface "migrations"      "$(scan_files -type d -name migrations)"
  add_surface "frontend"        "$(scan_files -type d \( -name templates -o -name components -o -name static \))"
  add_surface "config_runtime"  "$(scan_files -maxdepth 2 -type f \( -name 'config.py' -o -name 'settings.py' -o -name 'docker-compose*.yml' -o -name 'Dockerfile' -o -name '.flaskenv' \))"

  if [[ -z "$surfaces" ]]; then
    sos_emit_map_stub "$out"
    echo "  ⚠ sos map: no code surfaces detected → wrote draft_unmapped stub ($out). Fill by hand."
    return 0
  fi

  {
    cat <<'HEAD'
# AGENT_MAP — scan-generated DRAFT (sos map). status: draft_needs_review.
#
# SOUND half (auto): the surfaces + edit-paths below are REAL dirs/files in THIS repo, coarse-
#   grouped by directory pattern. This is NOT the tarot example (no "map nói dối").
# PARTIAL half (YOU): set load_bearing (true|false) + write blast for each — the scan can't
#   judge importance (product knowledge). Split coarse regions into semantic surfaces as you
#   refine (e.g. routes_handlers → auth / ratings / privacy).
#
# Validator: doctor validate-map --map docs/AGENT_MAP.yaml (every path must exist).
version: 1
status: draft_needs_review
generated_by: "sos map (scan)"

surfaces:
HEAD
    printf '%s' "$surfaces"
    cat <<'TAIL'

never_default_read:
  - docs/CHANGELOG.md
  - docs/DISCOVERIES.md
  - docs/BACKLOG.md
  - docs/ticket/archive/**
TAIL
  } > "$out"

  echo "  ✓ sos map: scanned $target → $out (draft_needs_review — set load_bearing + blast by hand)"
}

sos_new() {
  # sos new <target-dir> --stack <python|rust|ts> [--pilot]
  # Bootstrap a NEW repo from sos-kit golden (the scale foundation).
  # Doctrine: docs/BOOTSTRAP_AUTOMATION_DRAFT.md §7.
  #   1. Category A — FREEZE: copy proven spine verbatim (freeze-filtered).
  #   2. Category C — SKELETON: generate per-repo files with # TODO markers.
  #   3. Category B — DEFAULTS: sensible config, tunable.
  #   4. VALIDATOR: doctor verify-setup (wiring) + grep TODO (content to fill).
  # Done-when (acceptance test): a fresh spawn → verify-setup CONNECTED zero-hand-fix.
  local target="" stack="" pilot="false" force="false"
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --stack) stack="${2:-}"; shift 2 ;;
      --pilot) pilot="true"; shift ;;
      --force) force="true"; shift ;;
      --*) echo "✗ Unknown flag: $1"; return 1 ;;
      *) [[ -z "$target" ]] && target="$1"; shift ;;
    esac
  done

  if [[ -z "$target" ]]; then
    echo "Usage: sos new <target-dir> --stack <python|rust|ts> [--pilot]"
    return 1
  fi
  case "$stack" in
    python|rust|ts) ;;
    "") echo "✗ --stack required (python|rust|ts)"; return 1 ;;
    *)  echo "✗ Unknown stack: $stack (expected python|rust|ts)"; return 1 ;;
  esac

  local K="$SOS_KIT_DIR"
  if [[ ! -d "$K/.claude/agents" ]]; then
    echo "✗ SOS_KIT_DIR ($K) is not sos-kit (no .claude/agents). Set SOS_KIT_DIR."
    return 1
  fi
  # Guard: refuse a non-empty target unless --force (a command named `new` must not
  # silently overwrite an existing repo — that's `sos adopt`, future). Empty/new dir OK.
  if [[ -e "$target" && -n "$(ls -A "$target" 2>/dev/null)" && "$force" != "true" ]]; then
    echo "✗ $target exists and is non-empty — refusing to bootstrap into it."
    echo "  Pick an empty/new dir, or pass --force to overwrite into it."
    echo "  (Adopting an existing repo = future 'sos adopt', not 'sos new'.)"
    return 1
  fi

  echo "─────────────────────────────────────────"
  echo "sos new — bootstrap '$target' (stack: $stack)"
  echo "─────────────────────────────────────────"
  local name; name="$(basename "$target")"

  # ---- 1. Category A — FREEZE (copy proven spine verbatim) ----
  echo "[1/4] Category A — freeze (copy from golden)"
  mkdir -p "$target/.claude" "$target/docs/ticket/done" "$target/docs/security" \
           "$target/src" "$target/tests" "$target/hooks"
  # -L: dereference the drift-fix symlinks (.claude/agents/*.md → ../../agents/*.md) into
  # REAL files. A spawned repo has no top-level agents/ target, so a plain `cp -R` would
  # copy DANGLING symlinks → subagents unreadable → verify-setup J6 BROKEN (regression of 3334a3a).
  cp -RL "$K/.claude/agents"   "$target/.claude/agents"
  cp -RL "$K/.claude/commands" "$target/.claude/commands"
  cp    "$K/.claude/settings.json" "$target/.claude/settings.json"
  [[ -f "$K/agents/orchestrator.md" ]] && cp "$K/agents/orchestrator.md" "$target/.claude/agents/orchestrator.md"
  [[ -f "$K/templates/claude-settings.local.json" ]] && cp "$K/templates/claude-settings.local.json" "$target/.claude/settings.local.json"
  cp -R "$K/skills"    "$target/.claude/skills"   # Cat-A: 13 generic SOS role-workflows (/plan /verify /review /qa /ship /retro /init /idea /insight /route /decide /forge /apply)
  cp -R "$K/scripts"   "$target/scripts"
  cp -R "$K/phieu"     "$target/phieu"
  cp -R "$K/templates" "$target/templates"
  cp    "$K/hooks/pre-commit" "$target/hooks/pre-commit"
  cp    "$K/hooks/pre-push"   "$target/hooks/pre-push"
  chmod +x "$target/hooks/pre-commit" "$target/hooks/pre-push" 2>/dev/null || true
  # .gitignore — golden ships one (.DS_Store, .sos/, .sos-state/, build artifacts). Without it,
  # a spawned repo commits cruft on its first commit (dogfood finding: 4 .DS_Store leaked into ket).
  [[ -f "$K/.gitignore" ]] && cp "$K/.gitignore" "$target/.gitignore"
  # .mcp.json — wire ALL of OUR OWN Rust tools (Cat-A: in-control, no credential = the kit's
  # deterministic "hands"). Each exposes `serve` (stdio MCP). PATH-rel command, NOT a hardcoded
  # ~/.cargo path (Rule #3). External MCP (context7/supabase/canva/gdrive...) are Cat-C — the
  # repo declares its own; the kit does NOT ship them.
  # NOTE: guard/vps are deploy/server-scoped — inert on a local-only app (nothing to act on) but
  # harmless (serve idles until called). The repo enables them by adding config (.ship.toml,
  # ~/.vps.toml, guard config) when it gains a deploy/VPS target.
  cat > "$target/.mcp.json" <<'JSON'
{
  "mcpServers": {
    "doctor": {
      "_comment": "Mechanical gate — lane-check, validate-map, rotate-check, runtime-scan. Universal (zero-config). Install: cargo install --path ~/doctor.",
      "command": "doctor",
      "args": ["serve"]
    },
    "docs-gate": {
      "_comment": "Docs compliance — CHANGELOG/ARCHITECTURE/Discovery checks (same engine the pre-commit hook runs as CLI). Install: cargo install --path ~/docs-gate.",
      "command": "docs-gate",
      "args": ["serve"]
    },
    "ship": {
      "_comment": "Release workflow — test, commit, push, PR, canary, deploy. The commit/push/PR/check layer is universal (any repo); canary/deploy need a target. Install: cargo install --path ~/ship.",
      "command": "ship",
      "args": ["serve"]
    },
    "guard": {
      "_comment": "Pre-deploy infra gate — schema drift + env sync over SSH. Inert until the repo has a deploy/SSH target + config. Install: cargo install --path ~/guard.",
      "command": "guard",
      "args": ["serve"]
    },
    "vps": {
      "_comment": "VPS ops — docker status/logs/restart/stats. Inert until ~/.vps.toml names a host. Install: cargo install --path ~/vps.",
      "command": "vps",
      "args": ["serve"]
    }
  }
}
JSON
  # strip copy cruft (macOS .DS_Store, Python bytecode) so spawned repos stay clean
  find "$target" -name '.DS_Store' -delete 2>/dev/null || true
  find "$target" -name '*.pyc' -delete 2>/dev/null || true
  find "$target" -type d -name '__pycache__' -exec rm -rf {} + 2>/dev/null || true
  echo "  ✓ agents(deref) + commands + settings.json + skills(13) + scripts + phieu + templates + hooks/pre-commit + .gitignore + .mcp.json(doctor)"

  # ---- 2. Category C — SKELETON (+ # TODO markers) ----
  echo "[2/4] Category C — generate skeletons (+ # TODO)"
  # 2a. docs/security/INVARIANTS.md — generic 5 (universal) + empty INV-LOCAL TODO
  cp "$K/templates/INVARIANTS-template.md" "$target/docs/security/INVARIANTS.md"
  printf '\n## INV-LOCAL (project-specific — FILL THESE)\n\n# TODO: add at least one `## INV-LOCAL-NNN — <title>` for this product, or write `# No local INV` if none.\n' >> "$target/docs/security/INVARIANTS.md"
  # 2b. docs/AGENT_MAP.yaml — greenfield has no code to scan yet → UNMAPPED stub.
  #     (Do NOT copy the tarot example = "map nói dối". Run `sos map .` once code exists.)
  sos_emit_map_stub "$target/docs/AGENT_MAP.yaml"
  # 2c. docs/BACKLOG.md from template
  [[ -f "$K/templates/BACKLOG_template.md" ]] && cp "$K/templates/BACKLOG_template.md" "$target/docs/BACKLOG.md"
  # 2d. CLAUDE.md project skeleton
  cat > "$target/CLAUDE.md" <<EOF
# CLAUDE.md — $name

> Project context for Claude Code. Workflow doctrine: sos-kit (3 roles + Quản đốc orchestrator).
> **PRD / single source of truth: docs/PROJECT.md** — read it before writing phiếu or code.
>   (Create it via the /init skill, or drop your own PRD there. CLAUDE.md is a summary; PROJECT.md is canonical.)

## Project context

# TODO: fill stack, role (product/tool/util), and core constraints.
#       Bootstrap --stack was: $stack (a placeholder if your real stack isn't python/rust/ts — fix this line).

## Rules

Inherit sos-kit role separation: Chủ nhà / Kiến trúc sư / Thợ + Quản đốc.
EOF
  # 2e. stack manifest skeleton (feeds sos init security in step 3)
  case "$stack" in
    python) [[ -f "$target/pyproject.toml" ]] || printf '[project]\nname = "%s"\nversion = "0.1.0"\n# TODO: fill dependencies\n' "$name" > "$target/pyproject.toml" ;;
    rust)   [[ -f "$target/Cargo.toml" ]]     || printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2021"\n# TODO: fill dependencies\n' "$name" > "$target/Cargo.toml"
            # Ship a buildable target: an empty src/ makes `cargo check` (armed pre-commit) fail
            # "no targets in manifest" → blocks the bootstrap commit. A stub main.rs = valid crate.
            [[ -f "$target/src/main.rs" ]] || printf 'fn main() {}\n' > "$target/src/main.rs" ;;
    ts)     [[ -f "$target/package.json" ]]   || printf '{\n  "name": "%s",\n  "version": "0.0.0"\n}\n' "$name" > "$target/package.json" ;;
  esac
  # 2f. docs/ARCHITECTURE.md skeleton (docs-gate [architecture] target) so the
  #     bootstrapped repo passes its OWN docs-gate (gap fix: config enables it).
  cat > "$target/docs/ARCHITECTURE.md" <<EOF
# Architecture — $name

> # TODO: fill. docs-gate [architecture] points here (.docs-gate.toml).

## Overview

# TODO: what this repo is, one paragraph.

## Components

# TODO: main modules / surfaces.

## Data flow

# TODO: how data moves through the system.
EOF
  # 2g. CHANGELOG.md skeleton (root; docs-gate checks freshness + a date in the latest entry)
  [[ -f "$target/CHANGELOG.md" ]] || printf '# Changelog\n\nFormat loosely follows Keep a Changelog.\n\n## v0.0.0 — bootstrap via `sos new` — %s\n\n- Repo bootstrapped from sos-kit golden.\n' "$(date -u +%Y-%m-%d)" > "$target/CHANGELOG.md"
  echo "  ✓ INVARIANTS.md + AGENT_MAP.yaml + BACKLOG.md + CLAUDE.md + ARCHITECTURE.md + CHANGELOG.md + stack manifest"

  # ---- 3. Category B — DEFAULTS (tunable) ----
  echo "[3/4] Category B — defaults"
  if [[ ! -f "$target/.docs-gate.toml" ]]; then
    cat > "$target/.docs-gate.toml" <<'EOF'
# docs-gate config — bootstrap default (sos new). Mirrors sos-kit's working config.
# Paths are relative to docs_dir; CHANGELOG.md lives at repo root via "../".
docs_dir = "docs"
changelog = "../CHANGELOG.md"
changelog_max_age_days = 1
changelog_staged = false   # docs-gate v0.1.0 "../" normalization bug — age check still enforces freshness

rules = []
staleness = []
doc_structure = []
count_check = []
cross_doc = []

[architecture]
enabled = true
file = "ARCHITECTURE.md"   # resolves to docs/ARCHITECTURE.md (relative to docs_dir)
required_sections = 0      # flexible — skeleton ships; tighten when filled
required_non_empty = []

[ticket]
ticket_dir = "docs/ticket"
type_pattern = ""
valid_types = []
exclude_files = []
EOF
  fi
  ( cd "$target" && sos_init_security >/dev/null 2>&1 ) || true   # writes .sos-stack.toml
  echo "  ✓ .docs-gate.toml + .sos-stack.toml"

  # ---- 4. VALIDATOR — composite (wiring + content-to-fill) ----
  echo "[4/4] Validator"
  local doctor_bin="${DOCTOR_BIN:-doctor}"
  if command -v "$doctor_bin" >/dev/null 2>&1 || [[ -x "$doctor_bin" ]]; then
    local vs_out vs_rc
    set +e
    vs_out="$("$doctor_bin" verify-setup --repo "$target" 2>&1)"; vs_rc=$?
    set -e
    echo "$vs_out" | sed 's/^/    /'
    if [[ "$vs_rc" -eq 0 ]]; then echo "  ✓ verify-setup: CONNECTED"
    else echo "  ⚠ verify-setup: rc=$vs_rc — wiring gap above"; fi
  else
    echo "  ⏭ doctor not found — skip verify-setup (install: cargo install --path ~/doctor, or set DOCTOR_BIN=/path/to/doctor)"
  fi
  echo "  Category C placeholders to fill (# TODO):"
  grep -rl "# TODO" "$target/docs" "$target/CLAUDE.md" 2>/dev/null | sed "s|$target/|    - |" || echo "    (none found)"

  # ---- 5. Git + arm hooks (born-wired: gate LIVE for every future commit, not memory-dependent) ----
  echo "[+] Git init + arm hooks"
  if [[ ! -d "$target/.git" ]]; then
    # init + install-hooks BUT do NOT auto-commit: the bootstrap commit is the user's, and it
    # passes THROUGH the freshly-armed gate (proof the spawned repo gates its own work day 1).
    # NOTE: `git init -b main` needs git ≥ 2.28 — use `init` + `symbolic-ref` instead so the
    # branch is `main` on every git ever shipped (fleet hosts run older git: Ubuntu 20.04 = 2.25).
    ( cd "$target" && git init -q && git symbolic-ref HEAD refs/heads/main && bash scripts/install-hooks.sh 2>&1 | sed 's/^/    /' )
    echo "  ✓ git repo (branch main) + pre-commit/pre-push hooks armed"
  else
    # --force into a pre-existing .git/ — still arm hooks (mechanism, not a memory-dependent hint).
    # install-hooks.sh is idempotent (overwrite-on-purpose, scripts/install-hooks.sh:3).
    ( cd "$target" && bash scripts/install-hooks.sh 2>&1 | sed 's/^/    /' )
    echo "  ✓ pre-existing git repo — pre-commit/pre-push hooks armed"
  fi

  echo ""
  echo "✓ sos new done: $target"
  echo "  Next: fill # TODO (AGENT_MAP surfaces, INV-LOCAL, CLAUDE.md context) → git add -A && git commit."
}

sos_adopt() {
  # sos adopt <existing-repo-dir> [--stack <python|rust|ts>]
  # Retrofit the sos-kit spine into an EXISTING repo (brownfield half of the platform).
  # Opposite discipline to `sos new`: RESPECTS what's already there.
  #   ADDITIVE     — copy spine items only where the destination is ABSENT.
  #   NON-CLOBBER  — destination exists → stage sos-kit's version to .sos-adopt-incoming/<path>
  #                  + report for manual merge. NEVER overwrite the repo's own files.
  #   REPORT       — verify-setup verdict + DORMANT joints for 3-way triage (you decide).
  # Doctrine: docs/BOOTSTRAP_AUTOMATION_DRAFT.md (brownfield/retrofit half).
  local target="" stack=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --stack) stack="${2:-}"; shift 2 ;;
      --*) echo "✗ Unknown flag: $1"; return 1 ;;
      *) [[ -z "$target" ]] && target="$1"; shift ;;
    esac
  done
  if [[ -z "$target" ]]; then
    echo "Usage: sos adopt <existing-repo-dir> [--stack <python|rust|ts>]"; return 1
  fi
  if [[ ! -d "$target" ]]; then
    echo "✗ $target does not exist — for a NEW repo use: sos new $target --stack <X>"; return 1
  fi
  if [[ -z "$(ls -A "$target" 2>/dev/null)" ]]; then
    echo "✗ $target is empty (greenfield) — use: sos new $target --stack <X>"; return 1
  fi
  local K="$SOS_KIT_DIR"
  if [[ ! -d "$K/.claude/agents" ]]; then
    echo "✗ SOS_KIT_DIR ($K) is not sos-kit (no .claude/agents). Set SOS_KIT_DIR."; return 1
  fi

  echo "─────────────────────────────────────────"
  echo "sos adopt — retrofit spine into existing '$target' (ADDITIVE + NON-CLOBBER)"
  echo "─────────────────────────────────────────"
  # JA-08 (jarvis dogfood): dirty target → adopt's ~60 new files mix into the user's WIP in
  # git status. Warn, don't block (adopt only ADDS files — safe on a dirty tree).
  if [[ -d "$target/.git" && -n "$(git -C "$target" status --porcelain 2>/dev/null)" ]]; then
    echo "⚠ target has uncommitted changes — adopt only ADDS files (never edits yours),"
    echo "  but git status will mix adopt output with your WIP. Commit/stash first to keep"
    echo "  the adopt reviewable as its own commit. Proceeding."
  fi
  local incoming="$target/.sos-adopt-incoming"
  local added="" conflicts=""

  # adopt_item <src-rel> [dest-rel] — additive + non-clobber.
  #   plain file: copy if dest ABSENT; else stage to .sos-adopt-incoming/ + flag conflict.
  #   directory: PER-FILE — copy each absent file into the existing dir; stage only the
  #              files that genuinely collide (so a repo's own scripts/ keeps its files AND
  #              receives sos-kit's new ones; only true conflicts need manual merge).
  adopt_item() {
    local src="$K/$1"; local rel="${2:-$1}"
    [[ -e "$src" ]] || return 0
    if [[ -d "$src" ]]; then
      local fpath frel fdest kreal
      kreal="$(realpath "$K")"
      while IFS= read -r fpath; do
        # symlink-escape guard (Giám sát advisory on the find -L change): the deref'd target
        # must resolve INSIDE the kit tree — a future out-of-tree symlink (supply-chain or
        # contributor error) must never be copied into adopted repos. realpath, not string-
        # match on $fpath: find prints paths under $src, so the escape is only visible after
        # resolving the link target.
        [[ "$(realpath "$fpath")" == "$kreal"/* ]] || { echo "  ⚠ skip (symlink escapes kit tree): ${fpath#"$K"/}"; continue; }
        frel="${fpath#"$K"/}"; fdest="$target/$frel"
        if [[ -e "$fdest" ]]; then
          mkdir -p "$(dirname "$incoming/$frel")"; cp "$fpath" "$incoming/$frel"
          conflicts="${conflicts}    ~ ${frel}\n"
        else
          mkdir -p "$(dirname "$fdest")"; cp "$fpath" "$fdest"
          added="${added}    + ${frel}\n"
        fi
      # -L: deref symlinks — kit's .claude/agents/*.md are symlinks to ../../agents/ (drift-fix
      # 2026-06-01); plain -type f skips them → adopted repo gets NO spawnable agents (F-04
      # jarvis dogfood). cp without -P dereferences, so the dest receives a real file.
      done < <(find -L "$src" -type f -not -path '*/__pycache__/*' -not -name '*.pyc' -not -name '.DS_Store')
    else
      local dest="$target/$rel"
      if [[ -e "$dest" ]]; then
        mkdir -p "$(dirname "$incoming/$rel")"; cp "$src" "$incoming/$rel"
        conflicts="${conflicts}    ~ ${rel}\n"
      else
        mkdir -p "$(dirname "$dest")"; cp "$src" "$dest"
        added="${added}    + ${rel}\n"
      fi
    fi
  }

  echo "[1/4] Spine (copy-if-absent; existing → staged to .sos-adopt-incoming/)"
  adopt_item ".claude/agents"
  adopt_item ".claude/commands"
  adopt_item ".claude/settings.json"
  [[ -f "$K/agents/orchestrator.md" ]] && adopt_item "agents/orchestrator.md" ".claude/agents/orchestrator.md"
  adopt_item "scripts"
  adopt_item "phieu"
  adopt_item "templates"
  adopt_item "hooks/pre-commit"
  adopt_item "hooks/pre-push"
  adopt_item "docs/ORCHESTRATION.md"   # full orchestrator spec (P060: banner/handbook reference it; absent downstream = orchestrator runs "blind handbook")
  # skills → .claude/skills/ (Cat-A: 13 generic SOS role-workflows). ADDITIVE per-file — keep the
  # repo's own domain skills (Cat-C, e.g. its phase-gate/status); add the generic ones; stage true
  # name-collisions to .sos-adopt-incoming/. (adopt_item can't remap skills/→.claude/skills/.)
  if [[ -d "$K/skills" ]]; then
    local sfile srel sdest
    while IFS= read -r sfile; do
      srel=".claude/skills/${sfile#"$K"/skills/}"; sdest="$target/$srel"
      if [[ -e "$sdest" ]]; then
        mkdir -p "$(dirname "$incoming/$srel")"; cp "$sfile" "$incoming/$srel"
        conflicts="${conflicts}    ~ ${srel}\n"
      else
        mkdir -p "$(dirname "$sdest")"; cp "$sfile" "$sdest"
        added="${added}    + ${srel}\n"
      fi
    done < <(find "$K/skills" -type f -not -path '*/__pycache__/*' -not -name '*.pyc' -not -name '.DS_Store')
  fi
  # .mcp.json — wire OUR doctor gate (Cat-A, our tool). Create-if-absent; if the repo already has
  # one (its own external MCP = Cat-C), flag it — add the 'doctor' entry by hand, never clobber.
  if [[ ! -f "$target/.mcp.json" ]]; then
    cat > "$target/.mcp.json" <<'JSON'
{
  "mcpServers": {
    "doctor": {
      "_comment": "sos-kit mechanical gate. Our own Rust tool — wire via PATH. Install: cargo install --path ~/doctor.",
      "command": "doctor",
      "args": ["serve"]
    }
  }
}
JSON
    added="${added}    + .mcp.json (doctor gate wired — our tool, PATH-rel)\n"
  else
    # JA-05 (jarvis dogfood): hand-editing JSON is where new users drop the ball. jq-merge the
    # doctor entry ONLY if absent (`//=` never touches existing keys = non-clobber preserved).
    if command -v jq >/dev/null 2>&1; then
      if jq -e '.mcpServers.doctor' "$target/.mcp.json" >/dev/null 2>&1; then
        conflicts="${conflicts}    ~ .mcp.json (doctor entry already present — kept)\n"
      elif jq '.mcpServers.doctor //= {"_comment":"sos-kit mechanical gate. Our own Rust tool — wire via PATH. Install: cargo install --path ~/doctor.","command":"doctor","args":["serve"]}' \
             "$target/.mcp.json" > "$target/.mcp.json.tmp" 2>/dev/null; then
        mv "$target/.mcp.json.tmp" "$target/.mcp.json"
        added="${added}    + .mcp.json (doctor entry jq-merged — repo's own MCP servers kept)\n"
      else
        rm -f "$target/.mcp.json.tmp"
        conflicts="${conflicts}    ~ .mcp.json (exists, jq-merge failed — add the \"doctor\" server entry by hand)\n"
      fi
    else
      conflicts="${conflicts}    ~ .mcp.json (exists — add the \"doctor\" server entry by hand; repo's own MCP kept. Tip: install jq for auto-merge)\n"
    fi
  fi
  # JA-05b: .claude/settings.local.json — pre-approve the 5 marker Bash ops (INSTALL.md §2.5);
  # missing these = permission prompt on every architect/worker spawn. Copy-if-absent, else
  # jq-merge into permissions.allow (unique = idempotent).
  local slj="$target/.claude/settings.local.json"
  local marker_perms='["Bash(mkdir -p .sos-state)","Bash(touch .sos-state/architect-active)","Bash(rm -f .sos-state/architect-active)","Bash(touch .sos-state/worker-active)","Bash(rm -f .sos-state/worker-active)"]'
  if [[ ! -f "$slj" ]]; then
    mkdir -p "$target/.claude"
    cp "$K/templates/claude-settings.local.json" "$slj"
    added="${added}    + .claude/settings.local.json (5 marker perms — per-user, don't commit)\n"
  elif command -v jq >/dev/null 2>&1; then
    if jq -e --argjson p "$marker_perms" '(.permissions.allow // []) | contains($p)' "$slj" >/dev/null 2>&1; then
      conflicts="${conflicts}    ~ .claude/settings.local.json (5 marker perms already present — kept)\n"
    elif jq --argjson p "$marker_perms" '.permissions.allow = ((.permissions.allow // []) + $p | unique)' \
           "$slj" > "$slj.tmp" 2>/dev/null; then
      mv "$slj.tmp" "$slj"
      added="${added}    + .claude/settings.local.json (5 marker perms jq-merged — existing perms kept)\n"
    else
      rm -f "$slj.tmp"
      conflicts="${conflicts}    ~ .claude/settings.local.json (exists, jq-merge failed — add 5 marker perms by hand, INSTALL.md §2.5)\n"
    fi
  else
    conflicts="${conflicts}    ~ .claude/settings.local.json (exists — add 5 marker perms by hand, INSTALL.md §2.5. Tip: install jq for auto-merge)\n"
  fi
  echo "  done"

  echo "[2/4] Category C — generate ONLY if missing (never overwrite existing docs)"
  mkdir -p "$target/docs/security" "$target/docs/ticket/done"
  if [[ ! -f "$target/docs/security/INVARIANTS.md" ]]; then
    cp "$K/templates/INVARIANTS-template.md" "$target/docs/security/INVARIANTS.md"
    printf '\n## INV-LOCAL (project-specific — FILL THESE)\n\n# TODO: add `## INV-LOCAL-NNN — <title>`, or `# No local INV`.\n' >> "$target/docs/security/INVARIANTS.md"
    added="${added}    + docs/security/INVARIANTS.md\n"
  else conflicts="${conflicts}    ~ docs/security/INVARIANTS.md (exists — kept)\n"; fi
  if [[ ! -f "$target/docs/AGENT_MAP.yaml" ]]; then
    sos_map "$target" >/dev/null    # SCAN real surfaces (NOT copy tarot content); draft_needs_review
    added="${added}    + docs/AGENT_MAP.yaml (scanned real surfaces — set load_bearing + blast by hand)\n"
  else conflicts="${conflicts}    ~ docs/AGENT_MAP.yaml (exists — kept)\n"; fi
  if [[ ! -f "$target/.docs-gate.toml" ]]; then
    cat > "$target/.docs-gate.toml" <<'TOML'
docs_dir = "docs"
changelog = "../CHANGELOG.md"
changelog_max_age_days = 1
changelog_staged = false
rules = []
staleness = []
doc_structure = []
count_check = []
cross_doc = []

[architecture]
enabled = true
file = "ARCHITECTURE.md"
required_sections = 0
required_non_empty = []

[ticket]
ticket_dir = "docs/ticket"
type_pattern = ""
valid_types = []
exclude_files = []
TOML
    added="${added}    + .docs-gate.toml\n"
  else conflicts="${conflicts}    ~ .docs-gate.toml (exists — kept; verify ticket_dir + architecture)\n"; fi
  # Cat-C docs: GENERATE a skeleton when ABSENT (additive → repo passes its OWN pre-commit,
  # born-ready like sos new). NEVER overwrite an EXISTING one (that's the repo's content).
  # CLAUDE.md = product identity → report-only (never auto-generate over a brownfield).
  local _name; _name="$(basename "$(cd "$target" && pwd)")"
  if [[ -f "$target/CHANGELOG.md" ]]; then conflicts="${conflicts}    ~ CHANGELOG.md (exists — kept)\n"
  elif [[ -f "$target/docs/CHANGELOG.md" ]]; then
    # repo keeps its changelog under docs/ (its .docs-gate.toml points there) — do NOT
    # generate a competing root skeleton (F-05 jarvis dogfood: duplicate changelog noise)
    conflicts="${conflicts}    ~ docs/CHANGELOG.md (exists — repo's own location, kept; no root skeleton)\n"
  else
    printf '# Changelog\n\nFormat loosely follows Keep a Changelog.\n\n## v0.0.0 — sos adopt — %s\n\n- Spine retrofitted via `sos adopt`.\n' "$(date -u +%Y-%m-%d)" > "$target/CHANGELOG.md"
    added="${added}    + CHANGELOG.md (skeleton)\n"
  fi
  if [[ -f "$target/docs/ARCHITECTURE.md" ]]; then conflicts="${conflicts}    ~ docs/ARCHITECTURE.md (exists — kept)\n"
  else
    cat > "$target/docs/ARCHITECTURE.md" <<EOF
# Architecture — $_name

> # TODO: fill. docs-gate [architecture] points here.

## Overview

# TODO: what this repo is, one paragraph.

## Components

# TODO: main modules / surfaces.

## Data flow

# TODO: how data moves through the system.
EOF
    added="${added}    + docs/ARCHITECTURE.md (skeleton)\n"
  fi
  if [[ -f "$target/docs/BACKLOG.md" ]]; then conflicts="${conflicts}    ~ docs/BACKLOG.md (exists — kept)\n"
  elif [[ -f "$K/templates/BACKLOG_template.md" ]]; then
    cp "$K/templates/BACKLOG_template.md" "$target/docs/BACKLOG.md"
    added="${added}    + docs/BACKLOG.md (template — fill Active sprint)\n"
  fi
  [[ -f "$target/CLAUDE.md" ]] && conflicts="${conflicts}    ~ CLAUDE.md (exists — kept)\n" \
    || conflicts="${conflicts}    ! CLAUDE.md (MISSING — add a project CLAUDE.md by hand; adopt won't guess product identity)\n"

  # .gitignore — seed runtime markers + build-artifact dirs (F02 dogfood: RP01 left 167MB
  # target/ untracked → next `git add -A` would swallow it; the kit's own .sos-state markers
  # also had to be hand-ignored downstream). Append-if-missing per line (idempotent, non-clobber).
  local gi="$target/.gitignore"; touch "$gi"; local gi_added=0
  for pat in '.sos-state/' '.backup/' '.sos-adopt-incoming/' 'target/' '__pycache__/' '*.pyc' '*.egg-info/' 'node_modules/' 'dist/'; do
    grep -qxF "$pat" "$gi" 2>/dev/null || { printf '%s\n' "$pat" >> "$gi"; gi_added=$((gi_added+1)); }
  done
  [[ "$gi_added" -gt 0 ]] && added="${added}    + .gitignore ($gi_added build/runtime ignore lines)\n"

  # .phieu-counter — seed so `phieu <slug>` works out of the box (P061: phiếu lifecycle tooling
  # didn't bootstrap → phiếu made by hand + left untracked). Phiếu ARE tracked (audit trail, Sếp
  # 2026-06-09 = option b) — phieu/ is NOT gitignored above; Worker stages the phiếu file (worker.md).
  if [[ ! -f "$target/.phieu-counter" ]]; then
    echo "0" > "$target/.phieu-counter"
    added="${added}    + .phieu-counter (seed 0 — source phieu/phieu.sh to use \`phieu <slug>\`)\n"
  fi

  # ---- Born-wire (JA-03 + JA-04, jarvis dogfood): `sos new` arms its gates at birth; adopt
  # used to stop at copy+report → repo had the hooks but they never RAN (memory-dependent
  # manual step = the class that dies). install-hooks.sh carries the F09 hijack guard
  # (pre-existing hooksPath ≠ hooks → TTY confirm / non-TTY abort) so auto-arm is safe.
  echo "[3/4] Born-wire (arm hooks + stack detect)"
  if [[ -d "$target/.git" && -f "$target/scripts/install-hooks.sh" ]]; then
    local ih_rc
    set +e
    ( cd "$target" && bash scripts/install-hooks.sh ) 2>&1 | sed 's/^/    /'
    ih_rc=${PIPESTATUS[0]}
    set -e
    if [[ "$ih_rc" -eq 0 ]]; then
      added="${added}    + hooks ARMED (core.hooksPath → hooks/; pre-existing .git/hooks moved to .bak if any)\n"
    else
      conflicts="${conflicts}    ~ hooks NOT armed (F09 guard declined — existing hook setup detected; review then run: bash scripts/install-hooks.sh)\n"
    fi
  else
    conflicts="${conflicts}    ~ hooks NOT armed (no .git or scripts/install-hooks.sh missing)\n"
  fi
  local is_rc
  set +e
  ( cd "$target" && sos_init_security ) >/dev/null 2>&1
  is_rc=$?
  set -e
  case "$is_rc" in
    0) added="${added}    + .sos-stack.toml (stack detected — advisory-scan/security-review ready)\n" ;;
    1) conflicts="${conflicts}    ~ .sos-stack.toml (already exists — kept)\n" ;;
    *) conflicts="${conflicts}    ~ .sos-stack.toml (no stack manifest detected — see file comments, add [[stack]] by hand)\n" ;;
  esac

  echo "[4/4] Validator"
  local doctor_bin="${DOCTOR_BIN:-doctor}"
  if command -v "$doctor_bin" >/dev/null 2>&1 || [[ -x "$doctor_bin" ]]; then
    set +e
    "$doctor_bin" verify-setup --repo "$target" 2>&1 | sed 's/^/    /'
    local vs=${PIPESTATUS[0]}
    set -e
    if [[ "$vs" -eq 0 ]]; then
      echo "  ✓ verify-setup: CONNECTED"
    else
      echo "  ⚠ verify-setup DORMANT (rc=$vs) — triage EACH joint:"
      echo "      (A) wire genuinely not connected → fix the repo"
      echo "      (B) verify-setup doesn't know THIS repo's pattern → teach it / label brittle (NOT a repo bug)"
      echo "      (C) product gap → you decide the content"
    fi
    # adopt self-check: validate-map catches the map-lie class (map paths must exist).
    if [[ -f "$target/docs/AGENT_MAP.yaml" ]]; then
      set +e; ( cd "$target" && "$doctor_bin" validate-map --map docs/AGENT_MAP.yaml >/dev/null 2>&1 ); local vm=$?; set -e
      [[ "$vm" -eq 0 ]] && echo "  ✓ validate-map: AGENT_MAP paths resolve" \
        || echo "  ⚠ validate-map: map drift (rc=$vm) — run: doctor validate-map --map docs/AGENT_MAP.yaml"
    fi
  else
    echo "  ⏭ doctor not found (set DOCTOR_BIN=/path or cargo install --path ~/doctor)"
  fi

  echo ""
  echo "═══ sos adopt report ═══"
  printf "ADDED (copied — were absent):\n%b" "${added:-    (none)\n}"
  printf "REVIEW / merge by hand (existing-or-missing — adopt did NOT touch):\n%b" "${conflicts:-    (none)\n}"
  echo ""
  echo "Next: diff staged wiring  (git diff --no-index <existing> $incoming/<path>)  → merge by hand;"
  echo "      add MISSING docs; fill # TODO; then: doctor verify-setup --repo $target"
  # JA-09 (jarvis dogfood): the armed docs-gate WILL block the first commit if the repo's
  # changelog is stale (>max_age). Turn the first gate-fail into a guided step, not a wall.
  echo ""
  echo "Heads-up for your FIRST commit (hooks are now armed):"
  echo "  - docs-gate freshness: add a new CHANGELOG entry (e.g. \`## <ver> — sos-kit adopted — $(date -u +%Y-%m-%d)\`)"
  echo "  - fill docs/BACKLOG.md Active sprint (1-2 real items) — the session banner reads it"
  echo "  - restart Claude Code in the repo so agents/skills load"
}

sos_sync() {
  # sos sync <adopted-repo-dir>
  # KIT-LAG cure (P067): re-sync sos-kit spine into a repo adopted from an OLDER kit version.
  #   take-newer the spine files the repo HASN'T customized · flag genuinely-customized ones.
  # Provenance oracle = sos-kit's OWN git history (no manifest needed → works retroactively):
  #   downstream file's blob matches SOME historical blob of its canonical path → unmodified
  #     stale-canonical → safe overwrite (take-newer). Matches NONE → customized → stage for merge.
  # Repo-IDENTITY files (CLAUDE.md / BACKLOG / .docs-gate.toml / INVARIANTS / AGENT_MAP) NEVER touched.
  local target=""
  while [[ $# -gt 0 ]]; do case "$1" in --*) shift;; *) [[ -z "$target" ]] && target="$1"; shift;; esac; done
  [[ -z "$target" ]] && { echo "Usage: sos sync <adopted-repo-dir>"; return 1; }
  [[ -d "$target" ]] || { echo "✗ $target not found"; return 1; }
  local K="$SOS_KIT_DIR"
  [[ -d "$K/.git" ]] || { echo "✗ SOS_KIT_DIR ($K) has no .git — sync needs kit history as provenance oracle"; return 1; }
  [[ -d "$K/.claude/agents" ]] || { echo "✗ SOS_KIT_DIR ($K) is not sos-kit"; return 1; }

  local incoming="$target/.sos-sync-incoming"
  local added="" updated="" flagged="" current=0 added_n=0 updated_n=0 flagged_n=0

  _blob_in_history() {  # <canon_relpath> <blob_sha> -> 0 if dest matches any historical version
    local rel="$1" want="$2" c h
    while IFS= read -r c; do
      h=$(git -C "$K" rev-parse "$c:$rel" 2>/dev/null) || continue
      [[ "$h" == "$want" ]] && return 0
    done < <(git -C "$K" rev-list --all -- "$rel" 2>/dev/null)
    return 1
  }
  _sync_one() {  # <canon_relpath_in_kit> <dest_relpath_in_target>
    local canon="$1" destrel="$2" src="$K/$1" dest="$target/$2"
    [[ -f "$src" ]] || return 0
    if [[ ! -e "$dest" ]]; then
      mkdir -p "$(dirname "$dest")"; cp "$src" "$dest"; added="${added}    + ${destrel}"$'\n'; added_n=$((added_n+1)); return 0
    fi
    cmp -s "$src" "$dest" && { current=$((current+1)); return 0; }
    local blob; blob=$(git hash-object "$dest" 2>/dev/null)
    if _blob_in_history "$canon" "$blob"; then
      cp "$src" "$dest"; updated="${updated}    ^ ${destrel}"$'\n'; updated_n=$((updated_n+1))
    else
      mkdir -p "$(dirname "$incoming/$destrel")"; cp "$src" "$incoming/$destrel"
      flagged="${flagged}    ~ ${destrel}"$'\n'; flagged_n=$((flagged_n+1))
    fi
  }

  echo "sos sync — re-sync spine into '$target'  (take-newer unmodified, flag customized)"
  echo "  provenance oracle: $K git history"

  local f rel base root
  for root in scripts phieu templates; do
    [[ -d "$K/$root" ]] || continue
    while IFS= read -r f; do rel="${f#"$K"/}"; _sync_one "$rel" "$rel"; done \
      < <(find "$K/$root" -type f -not -path '*/__pycache__/*' -not -name '*.pyc' -not -name '.DS_Store')
  done
  for rel in hooks/pre-commit hooks/pre-push docs/ORCHESTRATION.md .claude/settings.json; do
    [[ -f "$K/$rel" ]] && _sync_one "$rel" "$rel"
  done
  for f in "$K"/agents/*.md; do
    [[ -f "$f" ]] || continue; base=$(basename "$f"); [[ "$base" == "README.md" ]] && continue
    _sync_one "agents/$base" ".claude/agents/$base"
  done
  for f in "$K"/.claude/commands/*; do
    [[ -f "$f" ]] || continue; base=$(basename "$f"); _sync_one ".claude/commands/$base" ".claude/commands/$base"
  done
  while IFS= read -r f; do rel="${f#"$K"/skills/}"; _sync_one "skills/$rel" ".claude/skills/$rel"; done \
    < <(find "$K/skills" -type f -name 'SKILL.md' 2>/dev/null)

  echo ""
  echo "=== sos sync report ==="
  echo "ADDED (absent -> copied): $added_n";        printf "%b" "${added:-    (none)\n}"
  echo "UPDATED (take-newer, unmodified stale): $updated_n"; printf "%b" "${updated:-    (none)\n}"
  echo "FLAGGED (repo-customized -> .sos-sync-incoming, merge by hand): $flagged_n"; printf "%b" "${flagged:-    (none)\n}"
  echo "ALREADY-CURRENT: $current"
  echo "Identity files (CLAUDE.md/BACKLOG/.docs-gate.toml/INVARIANTS/AGENT_MAP) untouched by design."
}

sos_blueprint() {
  if [[ ! -f docs/PROJECT.md ]]; then
    echo "✗ docs/PROJECT.md missing. Run 'sos init' first."
    return 1
  fi
  sos_state_init
  local phase
  phase="$(sos_state_get_phase)"
  if [[ "$phase" != "VISION_CAPTURED" && "$phase" != "INIT" ]]; then
    echo "⚠ State is '$phase' — blueprint expected after vision capture."
  fi

  cat <<'EOF'
─────────────────────────────────────────
Phase 1 — Blueprint (Stack + Recipes)
─────────────────────────────────────────

Open Claude Code and have Kiến trúc sư:

  1. Read docs/PROJECT.md + docs/SOUL.md (if exists)
  2. Pick tech stack appropriate to vision + constraints
  3. List recipes from $SOS_KIT_DIR/recipes/ in order of apply
  4. Flag any recipes that don't exist yet → forge before contract phase
  5. Write everything to docs/BLUEPRINT.md

Recipes available right now:
EOF
  if [[ -d "$SOS_KIT_DIR/recipes" ]]; then
    find "$SOS_KIT_DIR/recipes" -name '*.md' -not -name '_TEMPLATE.md' -not -name 'README.md' \
      | sed "s|$SOS_KIT_DIR/recipes/||" | sed 's|\.md$||' | sort | sed 's|^|  - |'
  fi
  echo ""
  echo "After BLUEPRINT.md ready → 'sos contract' to lock as P000-genesis.md."
  sos_state_set_phase "BLUEPRINT_DRAFTED"
}

sos_contract() {
  if [[ ! -f docs/BLUEPRINT.md ]]; then
    echo "✗ docs/BLUEPRINT.md missing. Run 'sos blueprint' first."
    return 1
  fi
  if [[ ! -f docs/ticket/P000-genesis.md ]]; then
    mkdir -p docs/ticket
    cp "$SOS_KIT_DIR/phieu/GENESIS_TEMPLATE.md" docs/ticket/P000-genesis.md
    echo "✓ Copied GENESIS_TEMPLATE.md → docs/ticket/P000-genesis.md"
  fi

  echo ""
  echo "Open Claude Code and have Kiến trúc sư fill docs/ticket/P000-genesis.md"
  echo "  - Vision Anchor (from PROJECT.md + SOUL.md)"
  echo "  - MVP Scope (Core features + Can ship without)"
  echo "  - Tech Commitments + Recipes to apply (from BLUEPRINT.md)"
  echo "  - Verification Anchors (project-specific invariants)"
  echo "  - Launch Checklist (copy from phieu/LAUNCH_CHECKLIST.md)"
  echo ""
  read -rp "When P000-genesis.md is ready, type 'lock' to compute spec_hash and lock: " confirm
  if [[ "$confirm" != "lock" ]]; then
    echo "Aborted. Re-run 'sos contract' when ready."
    return 1
  fi

  # Compute spec_hash on frozen sections (1, 2, 3)
  local hash
  if command -v sha256sum > /dev/null; then
    hash=$(awk '/^## 1\. Vision Anchor/,/^## 4\. Verification Anchors/' docs/ticket/P000-genesis.md | sha256sum | awk '{print $1}')
  else
    hash=$(awk '/^## 1\. Vision Anchor/,/^## 4\. Verification Anchors/' docs/ticket/P000-genesis.md | shasum -a 256 | awk '{print $1}')
  fi
  local ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  # Update P000 header
  sed "s|^> \*\*Spec Hash:\*\* .*|> **Spec Hash:** \`sha256:$hash\`|" docs/ticket/P000-genesis.md > docs/ticket/P000-genesis.md.tmp
  mv docs/ticket/P000-genesis.md.tmp docs/ticket/P000-genesis.md
  sed "s|^> \*\*Locked at:\*\* .*|> **Locked at:** \`$ts\`|" docs/ticket/P000-genesis.md > docs/ticket/P000-genesis.md.tmp
  mv docs/ticket/P000-genesis.md.tmp docs/ticket/P000-genesis.md

  # Update state
  cat >> .sos/state.toml <<EOF

[[history]]
event = "contract.lock"
spec_hash = "sha256:$hash"
timestamp = "$ts"
by = "Chủ nhà"
reason = "Genesis"
EOF
  sos_state_set_phase "LOCKED"

  echo "✓ P000-genesis.md locked"
  echo "  spec_hash: sha256:$hash"
  echo ""
  echo "Next: 'sos apply --all' to scaffold via recipes."
}

sos_apply() {
  if [[ "$#" -lt 1 ]]; then
    echo "Usage: sos apply <category>/<name>  |  sos apply --all"
    return 1
  fi
  local phase
  phase="$(sos_state_get_phase 2>/dev/null || echo INIT)"
  if [[ "$phase" != "LOCKED" && "$phase" != "SCAFFOLDED" && "$phase" != "ITERATING" ]]; then
    echo "✗ State is '$phase' — must run 'sos contract' first to lock P000."
    return 1
  fi

  if [[ "$1" == "--all" ]]; then
    echo "Reading recipe list from docs/ticket/P000-genesis.md..."
    local recipes
    recipes=$(awk '/^### Recipes to apply/,/^### Recipes thiếu/' docs/ticket/P000-genesis.md \
      | grep -E '^[0-9]+\.' | sed -E 's/^[0-9]+\. `?([^`]+)`?/\1/')
    if [[ -z "$recipes" ]]; then
      echo "✗ No recipes parsed from P000-genesis.md. Did Kiến trúc sư fill 'Recipes to apply'?"
      return 1
    fi
    echo "Recipes to apply (in order):"
    echo "$recipes" | sed 's/^/  - /'
    echo ""
    echo "Open Claude Code and invoke /apply per recipe in order."
    echo "Skill /apply will: generate sub-phiếu P000.N, run Task 0, execute, verify, commit."
    return 0
  fi

  local recipe="$1"
  if [[ ! -f "$SOS_KIT_DIR/recipes/$recipe.md" ]]; then
    echo "✗ Recipe not found: recipes/$recipe.md"
    echo "  Forge it first: sos recipe new $recipe"
    return 1
  fi

  echo "─────────────────────────────────────────"
  echo "Phase 3 — Apply: $recipe"
  echo "─────────────────────────────────────────"
  echo ""
  echo "Recipe: $SOS_KIT_DIR/recipes/$recipe.md"
  echo ""
  echo "Open Claude Code and invoke skill /apply with arg: $recipe"
  echo "Skill will:"
  echo "  1. Read recipe + verify Inputs satisfied"
  echo "  2. Generate sub-phiếu P000.N"
  echo "  3. Execute Steps (with plan mode if > 5 steps)"
  echo "  4. Run Verification anchors"
  echo "  5. Update state.toml + DISCOVERIES.md + commit"
  sos_state_set_phase "SCAFFOLDED"
}

sos_recipe() {
  if [[ "$#" -lt 2 || "$1" != "new" ]]; then
    echo "Usage: sos recipe new <category>/<name>"
    return 1
  fi
  local recipe="$2"
  local file="$SOS_KIT_DIR/recipes/$recipe.md"
  if [[ -f "$file" ]]; then
    echo "✗ Recipe exists: $file"
    echo "  To revise → invoke /forge skill with 'update' option."
    return 1
  fi
  echo "Open Claude Code and invoke skill /forge with arg: $recipe"
  echo "Skill /forge will: research official docs → write recipe → save to $file → commit."
}

sos_launch() {
  if [[ ! -f docs/ticket/P000-genesis.md ]]; then
    echo "✗ No P000-genesis.md. This isn't a Genesis-managed project."
    return 1
  fi

  echo "─────────────────────────────────────────"
  echo "Phase N+1 — Launch Gate"
  echo "─────────────────────────────────────────"
  echo ""
  echo "Checking LAUNCH_CHECKLIST in docs/ticket/P000-genesis.md..."
  local total
  local ticked
  total=$(awk '/^## 5\. Launch Checklist/,/^## 6\./' docs/ticket/P000-genesis.md | grep -cE '^- \[[x ]\]' || echo 0)
  ticked=$(awk '/^## 5\. Launch Checklist/,/^## 6\./' docs/ticket/P000-genesis.md | grep -cE '^- \[x\]' || echo 0)
  echo "  Ticked: $ticked / $total"
  if [[ "$total" -eq 0 ]]; then
    echo "✗ Could not parse Launch Checklist. Verify P000 structure."
    return 1
  fi
  if [[ "$ticked" -lt "$total" ]]; then
    echo ""
    echo "✗ HARD BLOCK — checklist incomplete ($ticked/$total)."
    echo ""
    echo "Untick items:"
    awk '/^## 5\. Launch Checklist/,/^## 6\./' docs/ticket/P000-genesis.md | grep '^- \[ \]' | head -20
    echo ""
    echo "Bypass with --skip <items> --reason \"...\" (audited). Not recommended."
    return 1
  fi

  echo "✓ Checklist 100%"
  echo "  Now run: guard check_all && ship canary"
  sos_state_set_phase "LAUNCHED"
  cat >> .sos/state.toml <<EOF

[[history]]
event = "launch"
timestamp = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
by = "Chủ nhà"
EOF
  echo ""
  echo "🎉 Launched. Don't forget docs/DISCOVERIES.md retro entry."
}

sos_status() {
  if [[ ! -f .sos/state.toml ]]; then
    echo "No .sos/state.toml — this isn't a Genesis-managed project (or run 'sos init')."
    return 1
  fi
  echo "─── .sos/state.toml ───"
  cat .sos/state.toml
}

# Dispatcher
sos() {
  local cmd="${1:-help}"
  shift || true
  case "$cmd" in
    new)         sos_new "$@" ;;
    adopt)       sos_adopt "$@" ;;
    sync)        sos_sync "$@" ;;
    map)         sos_map "$@" ;;
    init)        sos_init "$@" ;;
    blueprint)   sos_blueprint "$@" ;;
    contract)    sos_contract "$@" ;;
    apply)       sos_apply "$@" ;;
    recipe)      sos_recipe "$@" ;;
    launch)      sos_launch "$@" ;;
    status)      sos_status "$@" ;;
    help|--help|-h) sos_help ;;
    *) echo "Unknown command: $cmd"; sos_help; return 1 ;;
  esac
}

# If invoked as script (not sourced), call sos directly
if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]]; then
  sos "$@"
fi
