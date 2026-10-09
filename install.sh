#!/bin/sh
# sos-kit v3 installer: puts the `sos` binary on PATH. Nothing else is downloaded.
#
#   curl -fsSL https://raw.githubusercontent.com/aspelldenny/sos-kit/main/install.sh | sh
#
# Then, in each repo:  sos install  (vendors harness-lite/, starter files, git hooks).
#
# Fail-closed: a download or checksum failure aborts. The binary is verified against the
# .sha256 published with the same release.
#
# Env overrides:
#   SOS_VERSION  release tag to install   (default: the tag this script was published with)
#   SOS_BIN_DIR  install directory        (default ~/.local/bin)
#
# Requirements checked afterwards (not installed for you): git, gitleaks, python3.
# Developers can build instead: cargo install --path crates/sos-cli
set -eu

GH_OWNER="aspelldenny"
GH_REPO="sos-kit"
VERSION="${SOS_VERSION:-v0.3.3}"
BIN_DIR="${SOS_BIN_DIR:-$HOME/.local/bin}"

OS="$(uname -s)" ARCH="$(uname -m)"
case "$OS-$ARCH" in
  Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
  Linux-x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
  *)
    echo "✗ No prebuilt sos for $OS $ARCH." >&2
    echo "  → How to fix: install Rust, clone $GH_OWNER/$GH_REPO, run: cargo install --path crates/sos-cli" >&2
    exit 1 ;;
esac

if command -v sha256sum >/dev/null 2>&1; then SHA_CMD="sha256sum"
elif command -v shasum >/dev/null 2>&1; then SHA_CMD="shasum -a 256"
else echo "✗ Need sha256sum or shasum to verify the download." >&2; exit 1
fi

URL="https://github.com/$GH_OWNER/$GH_REPO/releases/download/$VERSION/sos-$TARGET"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
echo "▶ sos $VERSION ($TARGET)"
curl -fsSL --proto '=https' --connect-timeout 30 --max-time 300 -o "$TMP/sos" "$URL" \
  || { echo "✗ Download failed: $URL" >&2; exit 1; }
curl -fsSL --proto '=https' --connect-timeout 30 --max-time 60 -o "$TMP/sos.sha256" "$URL.sha256" \
  || { echo "✗ No checksum published at $URL.sha256 — refusing an unverified binary." >&2; exit 1; }
expected="$(cut -d' ' -f1 "$TMP/sos.sha256")"
actual="$($SHA_CMD "$TMP/sos" | cut -d' ' -f1)"
if [ "$expected" != "$actual" ]; then
  echo "✗ Checksum mismatch (expected $expected, got $actual). Aborting." >&2
  exit 1
fi
mkdir -p "$BIN_DIR"
chmod +x "$TMP/sos"
[ "$OS" = "Darwin" ] && xattr -d com.apple.quarantine "$TMP/sos" 2>/dev/null || true
if ! ver="$("$TMP/sos" --version 2>&1)"; then
  echo "✗ The downloaded binary does not run here: $ver" >&2
  exit 1
fi
# Stage next to the destination so the final rename is atomic (no half-written sos on PATH).
cp "$TMP/sos" "$BIN_DIR/.sos.new.$$" && mv -f "$BIN_DIR/.sos.new.$$" "$BIN_DIR/sos" || {
  rm -f "$BIN_DIR/.sos.new.$$"
  echo "✗ Could not write $BIN_DIR/sos (disk full or no permission?)." >&2
  exit 1
}
echo "  ✓ $BIN_DIR/sos ($ver)"

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "⚠ $BIN_DIR is not on PATH. Add to your shell profile: export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac
missing=""
for t in git gitleaks python3; do command -v "$t" >/dev/null 2>&1 || missing="$missing $t"; done
if [ -n "$missing" ]; then
  echo "⚠ Still needed:$missing   (macOS: brew install gitleaks; python3 ships with Xcode CLT)"
fi
echo "Next, inside a git repo:  sos install --dry-run   then   sos install"
