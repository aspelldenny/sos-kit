#!/bin/sh
# scripts/npm-postinstall.sh — npm postinstall for @aspelldenny/sos-kit (P081b, Stage 2).
#
# THIN: this script does NOT fork any fetch/verify logic. It only:
#   1. downloads install.sh from a PINNED TAG (not `main` — supply-chain),
#   2. verifies its sha256 against a hash shipped in the npm package,
#   3. runs it — fail-CLOSED on any mismatch or download failure.
#
# install.sh itself does the real work (10 sister tools + sos-bin + wrapper),
# unchanged and un-forked. Single source of truth = install.sh.
#
# Honors `npm install --ignore-scripts`: npm simply never invokes this file in
# that case, so there is nothing to special-case here — the fallback message
# below is what a user sees if they run this script BY HAND after skipping it.

set -eu

GH_OWNER="aspelldenny"
GH_REPO="sos-kit"
PIN_TAG="v0.1.0"
# npm installs `bin` entries as SYMLINKS (bin/sos-kit-setup -> ../lib/node_modules/.../
# scripts/npm-postinstall.sh) — dirname "$0" alone resolves to the symlink's own
# directory, not the real file's, so it must be dereferenced first.
_resolve_self() {
  _f="$0"
  while [ -L "$_f" ]; do
    _link="$(readlink "$_f")"
    case "$_link" in
      /*) _f="$_link" ;;
      *)  _f="$(dirname -- "$_f")/$_link" ;;
    esac
  done
  printf '%s\n' "$_f"
}
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$(_resolve_self)")" && pwd)"
PINNED_SHA_FILE="$SCRIPT_DIR/install-sh.sha256"

echo "▶ sos-kit: fetching full toolset via install.sh@${PIN_TAG} (pinned tag, sha256-verified)"

if command -v sha256sum >/dev/null 2>&1; then
  SHA_CMD="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
  SHA_CMD="shasum -a 256"
else
  echo "✗ No sha256 tool (need sha256sum or shasum) — cannot verify install.sh. ABORTING." >&2
  echo "  Run manually once you have one: sh $SCRIPT_DIR/npm-postinstall.sh" >&2
  exit 1
fi

if [ ! -f "$PINNED_SHA_FILE" ]; then
  echo "✗ Missing pinned hash file: $PINNED_SHA_FILE — package is corrupt. ABORTING." >&2
  exit 1
fi
expected="$(cut -d' ' -f1 "$PINNED_SHA_FILE")"

TMP_INSTALL="$(mktemp)"
trap 'rm -f "$TMP_INSTALL"' EXIT

INSTALL_URL="https://raw.githubusercontent.com/${GH_OWNER}/${GH_REPO}/${PIN_TAG}/install.sh"
echo "▶ install.sh ← $INSTALL_URL"
if ! curl -fsSL --proto '=https' --connect-timeout 30 --max-time 60 -o "$TMP_INSTALL" "$INSTALL_URL"; then
  echo "✗ Download FAILED for install.sh@${PIN_TAG} — ABORTING (fail-closed)." >&2
  echo "  Run manually once network is available: npx --package=@aspelldenny/sos-kit sos-kit-setup" >&2
  echo "  or: sh $SCRIPT_DIR/npm-postinstall.sh" >&2
  exit 1
fi

actual="$($SHA_CMD "$TMP_INSTALL" | cut -d' ' -f1)"
if [ "$expected" != "$actual" ]; then
  echo "✗ CHECKSUM MISMATCH for install.sh@${PIN_TAG} — expected $expected, got $actual." >&2
  echo "✗ ABORTING (fail-CLOSED) — refusing to run an unverified script." >&2
  echo "  This could mean the pinned tag moved, a corrupt download, or tampering." >&2
  echo "  Report: https://github.com/${GH_OWNER}/${GH_REPO}/issues" >&2
  exit 1
fi
echo "  ✓ install.sh sha256 verified"

echo "▶ running install.sh (SOS_KIT_DIR=${SOS_KIT_DIR:-<default>} SOS_BIN_DIR=${SOS_BIN_DIR:-<default>})"
if ! sh "$TMP_INSTALL"; then
  echo "✗ install.sh FAILED — sos-kit install incomplete." >&2
  echo "  Retry manually: sh $SCRIPT_DIR/npm-postinstall.sh" >&2
  echo "  or fallback setup: npx --package=@aspelldenny/sos-kit sos-kit-setup" >&2
  exit 1
fi

echo "✓ sos-kit: postinstall complete (npm bin/sos-npm now delegates to installed wrapper)"
