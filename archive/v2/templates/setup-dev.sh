#!/usr/bin/env bash
# setup-dev.sh — golden per-machine DEV bootstrap (sos-kit P065, generalized from tarot P344).
#
# AUDIENCE: developers who hack on the kit's Rust tools (builds from local checkouts).
# Kit USERS don't need Rust — use the 1-command installer instead:
#   curl -fsSL https://raw.githubusercontent.com/aspelldenny/sos-kit/main/install.sh | sh
#
# What it does (run once per machine, idempotent):
#   (a) check Rust toolchain (warn + exit if missing — never auto-install for the user)
#   (b) cargo install the tool set from local checkouts (edit TOOLS for your repo)
#   (c) arm git hooks via scripts/install-hooks.sh (F09-guarded)
#
# Per-repo customization: edit TOOLS. Keep entries = repo dir names under $SOS_TOOLS_DIR.
# Optional heavy deps: advisory-cron register stays OPT-IN (uncomment below) — auto-registering
# a daily CVE scan on every thin repo is noise (BACKLOG §B.2 trigger-wiring residual).

set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel 2>/dev/null || git rev-parse --show-toplevel)"
SOS_TOOLS_DIR="${SOS_TOOLS_DIR:-$HOME}"

# ── Per-repo tool list (EDIT ME) ─────────────────────────────────────────────
TOOLS=(claude-hooks doctor docs-gate)

# ── (a) Rust check ───────────────────────────────────────────────────────────
if ! command -v cargo >/dev/null 2>&1; then
  echo "❌ Rust toolchain not found. Install it first, then re-run:"
  echo "   https://rustup.rs  (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh)"
  echo "   (No Rust and just USING the kit? Run the prebuilt installer instead — see header.)"
  exit 1
fi
echo "✓ Rust: $(cargo --version)"

# ── (b) Install tools from local checkouts ──────────────────────────────────
for tool in "${TOOLS[@]}"; do
  src="$SOS_TOOLS_DIR/$tool"
  if [[ ! -d "$src" ]]; then
    echo "❌ $tool repo not found at $src"
    echo "   Clone it, or set SOS_TOOLS_DIR to the parent dir of your tool checkouts."
    exit 1
  fi
  echo "▶ cargo install --path $src"
  cargo install --path "$src" --quiet
  echo "  ✓ $tool: $(command -v "$tool") — $("$tool" --version 2>/dev/null || echo 'version unknown')"
done

# ── (c) Arm git hooks (F09-guarded — protects a pre-existing hook setup) ─────
if [[ -f "$REPO_ROOT/scripts/install-hooks.sh" ]]; then
  bash "$REPO_ROOT/scripts/install-hooks.sh"
else
  echo "⚠ scripts/install-hooks.sh not found — hooks NOT armed (did sos adopt run here?)"
fi

# ── (d) Optional opt-ins (uncomment per repo) ────────────────────────────────
# advisory-cron register --repo "$REPO_ROOT" --daily 09:00   # daily CVE scan (deps-heavy repos)

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✓ setup-dev complete: ${TOOLS[*]} on PATH + hooks armed."
echo "  Hook deploy doctrine [P064 B+3]: block-unsafe-merge = fail-closed shim"
echo "  (binary absent → Bash BLOCKED LOUD until installed); fail-open hooks"
echo "  (architect-guard / block-env-edit / session-banner) default-allow when absent."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
