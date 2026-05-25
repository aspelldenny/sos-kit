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
