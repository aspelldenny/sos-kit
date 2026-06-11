#!/bin/sh
# sos-kit — "1 lệnh" installer (P064, Tier-3 distribution).
#
#   curl -fsSL https://raw.githubusercontent.com/aspelldenny/sos-kit/main/install.sh | sh
#
# What it does (idempotent):
#   1. Detect platform → one of the 3 ratified targets (mac-arm64 / linux-x64 / win-x64).
#   2. Download prebuilt binaries from each tool's latest GitHub Release → ~/.local/bin.
#      NO Rust toolchain required (cargo install is the DEV path — see templates/setup-dev.sh).
#   3. Clone the kit to ~/sos-kit (or leave an existing checkout untouched).
#   4. Put a `sos` launcher on PATH (wrapper script — symlinks break on Windows Git Bash).
#   5. Print the next step (`sos adopt .` / `sos new`).
#
# Fail-CLOSED: a failed binary download ABORTS the install (the kit's security gates
# need these binaries; a half-install that silently lacks them = the P059 class).
#
# Env overrides:
#   SOS_KIT_DIR  kit checkout location  (default ~/sos-kit)
#   SOS_BIN_DIR  binary install dir     (default ~/.local/bin)

set -eu

GH_OWNER="aspelldenny"
KIT_DIR="${SOS_KIT_DIR:-$HOME/sos-kit}"
BIN_DIR="${SOS_BIN_DIR:-$HOME/.local/bin}"
# Binary manifest — each name is a repo under $GH_OWNER with release assets
# named <bin>-<target-triple>[.exe] (contract: that repo's .github/workflows/release.yml).
# KNOWN GAP (explicit, Giám sát 2026-06-11 → BACKLOG [P071]): downloads are HTTPS-enforced
# but carry NO checksum/signature verification yet, and `releases/latest` is unpinned —
# trust anchor today = the GitHub account. .sha256 publishing + verify is the planned cure.
BINARIES="doctor claude-hooks"

# ── 1. Platform → target triple ─────────────────────────────────────────────
OS="$(uname -s)" ARCH="$(uname -m)" EXT=""
case "$OS-$ARCH" in
  Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
  Linux-x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
  MINGW*-x86_64|MSYS*-x86_64|CYGWIN*-x86_64)
                 TARGET="x86_64-pc-windows-msvc"; EXT=".exe" ;;
  *)
    echo "✗ Unsupported platform: $OS $ARCH" >&2
    echo "  Prebuilt targets: mac-arm64, linux-x64, win-x64 (Git Bash)." >&2
    echo "  Dev fallback (needs Rust): clone the tool repos + cargo install --path." >&2
    exit 1 ;;
esac
echo "▶ Platform: $OS $ARCH → $TARGET"

# ── 2. Prebuilt binaries ─────────────────────────────────────────────────────
mkdir -p "$BIN_DIR"
for bin in $BINARIES; do
  url="https://github.com/$GH_OWNER/$bin/releases/latest/download/${bin}-${TARGET}${EXT}"
  dest="$BIN_DIR/${bin}${EXT}"
  echo "▶ $bin ← $url"
  if curl -fSL --proto '=https' --connect-timeout 30 --max-time 300 --progress-bar -o "${dest}.tmp" "$url"; then
    mv "${dest}.tmp" "$dest"
    chmod +x "$dest"
    echo "  ✓ $dest ($("$dest" --version 2>/dev/null || echo 'installed'))"
  else
    rm -f "${dest}.tmp"
    echo "✗ Download FAILED for $bin — ABORTING (fail-closed: the kit's gates need it)." >&2
    echo "  Check: https://github.com/$GH_OWNER/$bin/releases" >&2
    exit 1
  fi
done

# ── 3. Kit checkout ──────────────────────────────────────────────────────────
if [ -d "$KIT_DIR/.git" ]; then
  echo "▶ Kit already at $KIT_DIR — left untouched (update: git -C \"$KIT_DIR\" pull)"
else
  echo "▶ Cloning sos-kit → $KIT_DIR"
  GIT_TERMINAL_PROMPT=0 git clone --depth 1 "https://github.com/$GH_OWNER/sos-kit" "$KIT_DIR"
fi

# ── 4. `sos` launcher on PATH ────────────────────────────────────────────────
# Wrapper, NOT a symlink: symlinks need developer mode on Windows; a 2-line sh
# wrapper works everywhere Git Bash does. sos.sh self-resolves SOS_KIT_DIR via
# its own path, so we export it explicitly for the wrapper-call case.
{
  printf '#!/bin/sh\n'
  printf 'SOS_KIT_DIR="%s" exec bash "%s/bin/sos.sh" "$@"\n' "$KIT_DIR" "$KIT_DIR"
} > "$BIN_DIR/sos"
chmod +x "$BIN_DIR/sos"
echo "▶ sos launcher → $BIN_DIR/sos"

# ── 5. PATH check + next steps ───────────────────────────────────────────────
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo ""
     echo "⚠ $BIN_DIR is not on your PATH. Add to your shell profile:"
     echo "    export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✓ sos-kit installed: $BINARIES + sos → $BIN_DIR"
echo ""
echo "Next — pick by repo state:"
echo "  existing repo:   cd <your-repo> && sos adopt ."
echo "  new repo:        sos new <dir> --stack <python|rust|ts>"
echo "  older-kit repo:  sos sync <your-repo>"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
