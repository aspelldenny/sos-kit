# phieu.sh — Phiếu (ticket) workflow for SOS Kit
# Source from ~/.zshrc:  source /path/to/sos-kit/phieu/phieu.sh
#
# Commands:
#   phieu <slug>                      # auto-detect project from cwd, type=feat
#   phieu <type> <slug>               # auto-detect, explicit type
#   phieu <project> <slug>            # explicit project (from anywhere)
#   phieu <project> <type> <slug>     # explicit full
#   phieu-list [project]              # list worktrees (empty = all projects)
#   phieu-done [project] <P-slug>     # remove worktree
#   phieu-sync [project] <P-slug>     # rebase worktree on latest origin/main
#   phieu-init <project-path>         # onboard a new project (creates counter + wt dir)
#
# Ticket naming: <type>/P<NNN>-<slug>  where type ∈ feat|fix|chore|docs|infra

# --- Projects registry (populated by phieu-init) ---
# Users can also manually add: PHIEU_PROJECTS[name]="/path/to/project"
typeset -gA PHIEU_PROJECTS 2>/dev/null || declare -gA PHIEU_PROJECTS

# --- Helper: detect project from cwd ---
_phieu_detect_project() {
  local cwd="$PWD"
  for key in "${(@k)PHIEU_PROJECTS}"; do
    local root="${PHIEU_PROJECTS[$key]}"
    local wt="${root}-wt"
    if [[ "$cwd" == "$root" || "$cwd" == "$root"/* || "$cwd" == "$wt"/* ]]; then
      echo "$key"
      return 0
    fi
  done
  return 1
}

# --- Core: create phiếu + worktree ---
_phieu_impl() {
  local cmd="$1"
  local root="$2"
  local wt_parent="$3"
  shift 3

  local type slug
  if [ $# -eq 0 ]; then
    echo "Usage:"
    echo "  ${cmd} <slug>              # default type=feat"
    echo "  ${cmd} <type> <slug>       # type ∈ feat|fix|chore|docs|infra"
    return 1
  fi
  if [ $# -eq 1 ]; then
    type="feat"
    slug="$1"
  else
    type="$1"
    slug="$2"
  fi
  case "$type" in
    feat|fix|chore|docs|infra) ;;
    *)
      echo "❌ Invalid type: $type. Must be one of: feat, fix, chore, docs, infra"
      return 1
      ;;
  esac
  if ! [[ "$slug" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
    echo "❌ Slug invalid: '$slug'. Must be kebab-case (lowercase letters, digits, hyphens)"
    return 1
  fi

  local counter_file="${root}/.phieu-counter"
  [ ! -f "$counter_file" ] && echo "0" > "$counter_file"
  local n=$(cat "$counter_file")
  n=$((n + 1))
  local id=$(printf "P%03d" "$n")
  local branch="${type}/${id}-${slug}"
  local wt_name="${id}-${slug}"
  local wt_dir="${wt_parent}/${wt_name}"
  local ticket_file="docs/ticket/${id}-${slug}.md"

  echo "📝 Creating phiếu ${id} (project: $(basename "$root"))"
  echo "   Branch:   ${branch}"
  echo "   Worktree: ${wt_dir}"
  echo "   Ticket:   ${ticket_file}"
  echo ""

  cd "$root" || return 1
  echo "🔄 Fetching origin/main (base for new phiếu)..."
  git fetch origin main --quiet 2>/dev/null
  local base_ref="origin/main"
  if ! git rev-parse --verify "$base_ref" >/dev/null 2>&1; then
    echo "⚠️  origin/main not found, falling back to local main"
    base_ref="main"
  fi
  if ! git worktree add "$wt_dir" -b "$branch" "$base_ref"; then
    echo "❌ git worktree add failed — counter not incremented"
    return 1
  fi
  echo "$n" > "$counter_file"

  # Copy .env files if they exist (common for JS projects); silently skip if absent
  cp "${root}/.env" "${root}/.env.local" "$wt_dir/" 2>/dev/null

  cd "$wt_dir" || return 1

  # Create ticket from template — search in order: TICKET_TEMPLATE.md, TEMPLATE.md
  local template=""
  [ -f "docs/ticket/TICKET_TEMPLATE.md" ] && template="docs/ticket/TICKET_TEMPLATE.md"
  [ -z "$template" ] && [ -f "docs/ticket/TEMPLATE.md" ] && template="docs/ticket/TEMPLATE.md"
  if [ -n "$template" ]; then
    cp "$template" "${ticket_file}"
    sed -i.bak "1s|.*|# PHIẾU ${id}: ${slug}|" "${ticket_file}"
    rm -f "${ticket_file}.bak"
    echo "✅ Ticket created: ${ticket_file}"
  else
    echo "⚠️  No template found (docs/ticket/TICKET_TEMPLATE.md or TEMPLATE.md)"
    echo "    → ticket file NOT pre-created. Write it manually at ${ticket_file}"
  fi

  # Install dependencies based on detected stack
  if [ -f "pnpm-lock.yaml" ]; then
    echo "📦 pnpm install..."
    pnpm install || {
      echo "⚠️  pnpm install failed — fix it and run 'claude' in $wt_dir"
      return 1
    }
  elif [ -f "package-lock.json" ]; then
    echo "📦 npm install..."
    npm install || {
      echo "⚠️  npm install failed — fix it and run 'claude' in $wt_dir"
      return 1
    }
  elif [ -f "Cargo.toml" ]; then
    echo "📦 Rust project — skipping install (run cargo build/test when needed)"
  elif [ -f "requirements.txt" ] || [ -f "pyproject.toml" ]; then
    echo "📦 Python project — skipping install (activate venv when needed)"
  else
    echo "📦 Package manager not detected — skipping install"
  fi

  echo ""
  echo "🚀 Launching Claude Code in ${wt_dir}"
  if command -v claude >/dev/null 2>&1; then
    claude
  else
    echo "   (claude CLI not found — open your editor in ${wt_dir} manually)"
  fi
}

_phieu_done_impl() {
  local cmd="$1"
  local root="$2"
  local wt_parent="$3"
  shift 3
  if [ -z "$1" ]; then
    echo "Usage: ${cmd} <P042-slug>"
    echo "Example: ${cmd} P042-user-export"
    return 1
  fi
  local name="$1"
  cd "$root" || return 1
  git worktree remove "${wt_parent}/$name" && \
    echo "✅ Worktree removed: ${wt_parent}/$name (branch still exists)"
}

_phieu_list_impl() {
  local root="$1"
  local counter="${root}/.phieu-counter"
  echo "Active worktrees ($(basename "$root")):"
  cd "$root" && git worktree list
  local next=$(($(cat "$counter" 2>/dev/null || echo 0) + 1))
  printf "\nNext phiếu ID: P%03d\n" "$next"
}

# --- Main commands ---
phieu() {
  local project
  if [ $# -ge 1 ] && [[ -n "${PHIEU_PROJECTS[$1]}" ]]; then
    project="$1"
    shift
  fi
  [ -z "$project" ] && project=$(_phieu_detect_project)

  if [ -z "$project" ]; then
    echo "❌ Could not detect project from cwd '$PWD'."
    echo "Usage:"
    echo "  phieu <project> <slug>              # explicit (from anywhere)"
    echo "  cd ~/<project> && phieu <slug>      # auto-detect"
    if [ -n "${(k)PHIEU_PROJECTS}" ]; then
      echo "Registered projects: ${(@k)PHIEU_PROJECTS}"
    else
      echo "No projects registered. Run: phieu-init <project-path>"
    fi
    return 1
  fi

  if [ $# -eq 0 ]; then
    echo "Usage (current project: $project):"
    echo "  phieu <slug>              # default type=feat"
    echo "  phieu <type> <slug>       # type ∈ feat|fix|chore|docs|infra"
    echo "  phieu $project <slug>     # explicit from outside project"
    return 1
  fi

  local root="${PHIEU_PROJECTS[$project]}"
  _phieu_impl "phieu $project" "$root" "${root}-wt" "$@"
}

phieu-list() {
  local project
  if [ $# -ge 1 ] && [[ -n "${PHIEU_PROJECTS[$1]}" ]]; then
    project="$1"
  else
    project=$(_phieu_detect_project)
  fi

  if [ -z "$project" ]; then
    if [ -z "${(k)PHIEU_PROJECTS}" ]; then
      echo "No projects registered. Run: phieu-init <project-path>"
      return 0
    fi
    echo "📋 Registered projects:"
    for p in "${(@k)PHIEU_PROJECTS}"; do
      local root="${PHIEU_PROJECTS[$p]}"
      local next=$(($(cat "$root/.phieu-counter" 2>/dev/null || echo 0) + 1))
      local wt_count=$(cd "$root" 2>/dev/null && git worktree list 2>/dev/null | wc -l | tr -d ' ')
      printf "  %-20s  %d worktree(s), next ID: P%03d\n" "$p" "$wt_count" "$next"
    done
    echo ""
    echo "Run 'phieu-list <project>' for project detail."
    return 0
  fi

  _phieu_list_impl "${PHIEU_PROJECTS[$project]}"
}

phieu-done() {
  local project
  if [ $# -ge 1 ] && [[ -n "${PHIEU_PROJECTS[$1]}" ]]; then
    project="$1"
    shift
  fi
  [ -z "$project" ] && project=$(_phieu_detect_project)

  if [ -z "$project" ] || [ $# -eq 0 ]; then
    echo "Usage:"
    echo "  phieu-done <P042-slug>                # auto-detect project"
    echo "  phieu-done <project> <P042-slug>      # explicit"
    return 1
  fi

  local root="${PHIEU_PROJECTS[$project]}"
  _phieu_done_impl "phieu-done $project" "$root" "${root}-wt" "$@"
}

phieu-sync() {
  local project
  if [ $# -ge 1 ] && [[ -n "${PHIEU_PROJECTS[$1]}" ]]; then
    project="$1"
    shift
  fi
  [ -z "$project" ] && project=$(_phieu_detect_project)

  if [ -z "$project" ] || [ $# -eq 0 ]; then
    echo "Usage:"
    echo "  phieu-sync <P042-slug>                # auto-detect project"
    echo "  phieu-sync <project> <P042-slug>      # explicit"
    echo ""
    echo "What it does: rebase the worktree's branch onto latest origin/main"
    echo "              so your phiếu stays current with main's changes."
    return 1
  fi

  local name="$1"
  local root="${PHIEU_PROJECTS[$project]}"
  local wt_dir="${root}-wt/${name}"

  if [ ! -d "$wt_dir" ]; then
    echo "❌ Worktree not found: $wt_dir"
    return 1
  fi

  echo "🔄 Syncing worktree $wt_dir with origin/main..."
  cd "$wt_dir" || return 1

  # Fetch latest origin/main
  if ! git fetch origin main --quiet 2>/dev/null; then
    echo "⚠️  git fetch failed — check network"
    return 1
  fi

  # Check clean working tree (rebase needs clean)
  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "⚠️  Working tree has uncommitted changes. Commit or stash first."
    git status --short
    return 1
  fi

  # Attempt rebase
  if git rebase origin/main; then
    echo "✅ Rebased onto origin/main successfully."
    echo "   Run tests to confirm nothing broke, then continue."
  else
    echo ""
    echo "⚠️  Rebase hit conflict(s). NOT auto-resolving."
    echo ""
    echo "Options:"
    echo "  1. Resolve manually:"
    echo "     - Edit conflict files (git status shows which)"
    echo "     - git add <files>"
    echo "     - git rebase --continue"
    echo "  2. Abort and stay on old base:"
    echo "     - git rebase --abort"
    echo ""
    echo "You're now in ${wt_dir} — fix or abort from here."
    return 1
  fi
}

# --- Onboard a new project ---
phieu-init() {
  if [ -z "$1" ]; then
    echo "Usage: phieu-init <project-path>"
    echo "Example: phieu-init ~/my-project"
    return 1
  fi
  local path="${1%/}"
  path="${path/#\~/$HOME}"
  if [ ! -d "$path/.git" ]; then
    echo "❌ '$path' is not a git repo (.git/ not found)"
    return 1
  fi
  local name=$(basename "$path")
  if [[ -n "${PHIEU_PROJECTS[$name]}" ]]; then
    echo "⚠️  Project '$name' already registered at: ${PHIEU_PROJECTS[$name]}"
    return 1
  fi

  # Counter
  if [ ! -f "$path/.phieu-counter" ]; then
    echo "0" > "$path/.phieu-counter"
    echo "✓ Created $path/.phieu-counter = 0"
  fi
  # Worktree parent
  if [ ! -d "${path}-wt" ]; then
    mkdir -p "${path}-wt"
    echo "✓ Created ${path}-wt/"
  fi
  # Gitignore
  if [ -f "$path/.gitignore" ] && ! grep -q "^\.phieu-counter" "$path/.gitignore"; then
    printf "\n# Phiếu ID counter (local, per-machine)\n.phieu-counter\n" >> "$path/.gitignore"
    echo "✓ Added .phieu-counter to $path/.gitignore"
  fi
  # Register in current session + persist
  PHIEU_PROJECTS[$name]="$path"
  printf "\n# Added by phieu-init\nPHIEU_PROJECTS[%s]=\"%s\"\n" "$name" "$path" >> ~/.zshrc
  echo "✓ Appended to ~/.zshrc: PHIEU_PROJECTS[$name]=\"$path\""
  echo ""
  echo "🎉 Project '$name' is ready. Try:"
  echo "   phieu $name <slug>"
  echo "   phieu-list $name"
}
