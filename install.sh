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
# Full kit toolset (Sếp 2026-06-11: "chạy 1 phát là ok hết"): doctor (validator) +
# claude-hooks (session hooks, B+3 gate) + docs-gate (pre-commit [2/7] — absent = gate
# silently skips, P006) + ship/guard/vps (release/deploy/ops, wired in .mcp.json) +
# doc-rotate (docs cap/rotate) + advisory-inbox (CVE inbox CLI, /advisory-scan dep) +
# advisory-cron (daily scan scheduler — register is per-repo OPT-IN, binary just lands) +
# inv-gate (mechanical security INV gate — v0.1.0 public 2026-06-11; pre-commit [4/7] uses it
# binary-first, kills the python3 dep in the gate chain — harvest action inv-gate IG-06/dogfood).
# Download integrity: fetch-verify now enforced (P071-Stage2). Each binary download
# fetches its companion .sha256, recomputes the hash of the .tmp file with $SHA_CMD, and
# string-compares the first field. Mismatch → ABORT (poisoned/corrupt asset). Required-bin
# with missing .sha256 → ABORT (Stage 1 must have published it). Trust anchor: GitHub
# account + sha256 checksum. Version pinning (releases/latest→pinned-tag) = follow-up [P-pin].
BINARIES="doctor claude-hooks docs-gate ship advisory-inbox inv-gate"
# OPTIONAL — download failure = WARN + continue, NOT abort (fail-closed stays reserved
# for the gate binaries above). Two reasons a tool sits here:
#   guard / vps / doc-rotate: repos PRIVATE (real VPS IP/port in history — scrub before
#     publish). Functionally optional anyway (guard/vps inert without a deploy/SSH
#     target; doc-rotate = docs hygiene). Promote to BINARIES when published.
#   advisory-cron: repo PUBLIC (2026-06-11, history-scanned clean) but macOS+Linux only
#     by design (Phase 3 compile_error!) → stays optional so Windows installs don't abort.
OPTIONAL_BINARIES="guard vps doc-rotate advisory-cron"

# ── 1. Platform → target triple ─────────────────────────────────────────────
OS="$(uname -s)" ARCH="$(uname -m)" EXT=""
case "$OS-$ARCH" in
  Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
  Linux-x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
  MINGW*-x86_64|MSYS*-x86_64|CYGWIN*-x86_64)
                 TARGET="x86_64-pc-windows-msvc"; EXT=".exe" ;;
  # Intel Mac: no x86_64-apple-darwin build (kit ships 3 platform-arch only, IG-10). Run the
  # arm64 binary under Rosetta 2 — it's transparent once Rosetta is installed.
  Darwin-x86_64) TARGET="aarch64-apple-darwin"
    echo "ℹ Intel Mac — using arm64 binary via Rosetta 2 (install: softwareupdate --install-rosetta)." >&2 ;;
  *)
    echo "✗ Unsupported platform: $OS $ARCH" >&2
    echo "  Supported (3 platform-arch — IG-10 scope): mac (arm64 native / Intel via Rosetta)," >&2
    echo "  linux-x64, win-x64 (Git Bash). Linux-ARM64 not built yet — see BACKLOG." >&2
    echo "  Dev fallback (needs Rust): clone the tool repos + cargo install --path." >&2
    exit 1 ;;
esac
echo "▶ Platform: $OS $ARCH → $TARGET"

# ── Hash tool probe ──────────────────────────────────────────────────────────
# Prefers sha256sum (Linux-native; macOS also has /sbin/sha256sum — Darwin 1.0).
# Falls back to shasum -a 256 (macOS standard tool, also present on Git Bash).
# $SHA_CMD is intentionally a plain string (not an array) so "shasum -a 256"
# word-splits correctly in POSIX sh at the call site in fetch_bin().
if command -v sha256sum >/dev/null 2>&1; then
  SHA_CMD="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
  SHA_CMD="shasum -a 256"
else
  echo "✗ No sha256 tool (need sha256sum or shasum) — cannot verify downloads. ABORTING." >&2
  exit 1
fi

# ── 2. Prebuilt binaries ─────────────────────────────────────────────────────
mkdir -p "$BIN_DIR"
fetch_bin() {  # <bin> <required|optional> → 0 ok | 1 failed
  bin="$1"
  url="https://github.com/$GH_OWNER/$bin/releases/latest/download/${bin}-${TARGET}${EXT}"
  dest="$BIN_DIR/${bin}${EXT}"
  echo "▶ $bin ← $url"
  if curl -fSL --proto '=https' --connect-timeout 30 --max-time 300 --progress-bar -o "${dest}.tmp" "$url"; then
    # ── Integrity verify: fetch .sha256, recompute, compare first field ──────
    # Use recompute+string-compare (NOT sha256sum -c / shasum -c) because -c
    # matches by the filename recorded in the .sha256 (CI wrote <bin>-<target>)
    # but we downloaded to <dest>.tmp — filenames differ across the .tmp rename.
    sha_url="${url}.sha256"
    if curl -fSL --proto '=https' --connect-timeout 30 --max-time 60 -o "${dest}.sha256.tmp" "$sha_url" 2>/dev/null; then
      expected=$(cut -d' ' -f1 "${dest}.sha256.tmp")
      actual=$($SHA_CMD "${dest}.tmp" | cut -d' ' -f1)
      rm -f "${dest}.sha256.tmp"
      if [ "$expected" != "$actual" ]; then
        echo "✗ CHECKSUM MISMATCH for $bin — expected $expected, got $actual. ABORTING." >&2
        rm -f "${dest}.tmp"
        return 1
      fi
      echo "  ✓ sha256 verified"
    else
      rm -f "${dest}.sha256.tmp"
      if [ "$2" = required ]; then
        echo "✗ No .sha256 published for required bin $bin — cannot verify. ABORTING (Stage 1 incomplete?)." >&2
        rm -f "${dest}.tmp"
        return 1
      fi
      echo "  ⚠ no .sha256 for optional $bin — skipping verify (install unverified)."
    fi
    # ── End verify ───────────────────────────────────────────────────────────
    mv "${dest}.tmp" "$dest"
    chmod +x "$dest"
    # macOS Gatekeeper: curl-downloaded binaries get a com.apple.quarantine xattr → first run
    # is blocked ("cannot be opened"). Strip it so the binary runs (IG-11 inv-gate dogfood).
    [ "$OS" = "Darwin" ] && xattr -d com.apple.quarantine "$dest" 2>/dev/null || true
    echo "  ✓ $dest ($("$dest" --version 2>/dev/null || echo 'installed'))"
    return 0
  fi
  rm -f "${dest}.tmp"
  return 1
}
for bin in $BINARIES; do
  if ! fetch_bin "$bin" required; then
    echo "✗ Download FAILED for $bin — ABORTING (fail-closed: the kit's gates need it)." >&2
    echo "  Check: https://github.com/$GH_OWNER/$bin/releases" >&2
    exit 1
  fi
done
for bin in $OPTIONAL_BINARIES; do
  if ! fetch_bin "$bin" optional; then
    echo "  ⚠ $bin unavailable — skipped (optional: repo not public yet, or no build for $TARGET)."
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
echo "✓ sos-kit installed: $BINARIES + optional($OPTIONAL_BINARIES) + sos → $BIN_DIR"
echo ""
echo "Next — pick by repo state:"
echo "  existing repo:   cd <your-repo> && sos adopt ."
echo "  new repo:        sos new <dir> --stack <python|rust|ts>"
echo "  older-kit repo:  sos sync <your-repo>"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
