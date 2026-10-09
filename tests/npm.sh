#!/usr/bin/env bash
# The npm package: version pins agree, and the `sos` wrapper installs the binary on first use
# when npm skipped the package's install script (npm 12 default).
#   tests/npm.sh          pins + wrapper logic with a stubbed setup (no network)
#   tests/npm.sh --real   also: pack, install with --ignore-scripts into an empty prefix, run
#                         `sos --version` (needs the GitHub release for the pinned tag)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail() { echo "FAIL: $*"; exit 1; }

# 1. Pins: package.json, Cargo, install.sh, postinstall tag and install.sh hash agree.
V=$(sed -n 's/^  "version": "\([0-9.]*\)".*/\1/p' "$ROOT/package.json")
[ "$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/crates/sos-cli/Cargo.toml")" = "$V" ] || fail "Cargo version != package.json $V"
grep -q "SOS_VERSION:-v$V}" "$ROOT/install.sh" || fail "install.sh does not pin v$V"
grep -q "^PIN_TAG=\"v$V\"" "$ROOT/scripts/npm-postinstall.sh" || fail "npm-postinstall.sh does not pin v$V"
grep -q "install.sh@v$V" "$ROOT/package.json" || fail "package.json description does not say v$V"
want=$(cut -d' ' -f1 "$ROOT/scripts/install-sh.sha256")
have=$( (command -v sha256sum >/dev/null && sha256sum || shasum -a 256) < "$ROOT/install.sh" | cut -d' ' -f1)
[ "$want" = "$have" ] || fail "scripts/install-sh.sha256 does not match install.sh (run: shasum -a 256 install.sh > scripts/install-sh.sha256)"

# 2. Wrapper logic. A package copy whose setup script is a stub that "installs" a fake sos.
P="$T/pkg"; mkdir -p "$P/bin" "$P/scripts" "$T/npmbin"
cp "$ROOT/package.json" "$P/"; cp "$ROOT/bin/sos-npm" "$P/bin/"
cat > "$P/scripts/npm-postinstall.sh" <<'EOF'
#!/bin/sh
echo "stub setup ran SOS_VERSION=${SOS_VERSION:-}"
[ "${STUB_FAIL:-}" = 1 ] && exit 1
mkdir -p "$SOS_BIN_DIR"
printf '#!/bin/sh\nif [ "$1" = --version ]; then echo "sos %s (stub)"; else echo "args:$*"; fi\n' "${STUB_VERSION}" > "$SOS_BIN_DIR/sos"
chmod +x "$SOS_BIN_DIR/sos"
EOF
ln -s "$P/bin/sos-npm" "$T/npmbin/sos"   # npm installs bins as symlinks
export SOS_BIN_DIR="$T/bin"
run() { "$T/npmbin/sos" "$@" 2>"$T/err"; }

STUB_VERSION=$V
export STUB_VERSION SOS_VERSION=v0.0.9   # a stray SOS_VERSION must not change what npm installs

if run gate all >/dev/null; then fail "missing binary: a gate must fail closed, not download"; fi
grep -q "stub setup" "$T/err" && fail "a gate ran setup (gates do not fetch)"
grep -q "sos --version" "$T/err" || fail "missing binary: gate gave no fix"
out=$(run install --dry-run) || fail "missing binary: wrapper failed: $(cat "$T/err")"
[ "$out" = "args:install --dry-run" ] || fail "missing binary: stdout was '$out' (setup output must go to stderr)"
grep -q "not installed yet" "$T/err" || fail "missing binary: no notice on stderr"
grep -q "SOS_VERSION=v$V" "$T/err" || fail "setup did not pin SOS_VERSION=v$V"

out=$(run check); grep -q "stub setup" "$T/err" && fail "current binary: setup ran again"
[ "$out" = "args:check" ] || fail "current binary: '$out'"

printf '#!/bin/sh\necho "sos 0.0.1 (old)"\n' > "$SOS_BIN_DIR/sos"
out=$(run --version) || fail "old binary: wrapper failed"
grep -q "updating the sos binary 0.0.1 → $V" "$T/err" || fail "old binary: no update notice"
[ "$out" = "sos $V (stub)" ] || fail "old binary: not updated ('$out')"

printf '#!/bin/sh\necho "sos 0.0.1 (old)"\n' > "$SOS_BIN_DIR/sos"
out=$(run gate all); grep -q "stub setup" "$T/err" && fail "old binary: a gate ran setup"
grep -q "older than the npm package" "$T/err" || fail "old binary: gate gave no note"
out=$(STUB_FAIL=1 run --version) || fail "old binary + failed update must still run the old binary"
[ "$out" = "sos 0.0.1 (old)" ] && grep -q "continuing with sos 0.0.1" "$T/err" || fail "old binary + failed update: '$out'"

printf '#!/bin/sh\necho "sos 99.0.0 (newer)"\n' > "$SOS_BIN_DIR/sos"
out=$(run --version); [ "$out" = "sos 99.0.0 (newer)" ] || fail "newer binary must be kept, got '$out'"
grep -q "stub setup" "$T/err" && fail "newer binary: setup ran"

rm -f "$SOS_BIN_DIR/sos"
if STUB_FAIL=1 run --version >/dev/null; then fail "failed setup must exit non-zero"; fi
grep -q "could not install the sos binary" "$T/err" || fail "failed setup: no explanation"
grep -q "retry when online" "$T/err" || fail "failed setup: no retry command"

# npm's bin directory used as SOS_BIN_DIR: the wrapper would call itself.
ln -sf "$P/bin/sos-npm" "$SOS_BIN_DIR/sos"
if run --version >/dev/null; then fail "wrapper == binary location must be refused"; fi
grep -q "is this npm wrapper" "$T/err" || fail "collision: no explanation"
rm -f "$SOS_BIN_DIR/sos"

# A release must be tagged with the version the package pins.
case "${GITHUB_REF_NAME:-}" in v*) [ "$GITHUB_REF_NAME" = "v$V" ] || fail "tag $GITHUB_REF_NAME != package version v$V";; esac

echo "npm wrapper ok (v$V)"
[ "${1:-}" = --real ] || exit 0

# 3. Real chain: the packed package, installed the way npm 12 does by default (no scripts).
cd "$T" && npm pack --silent "$ROOT" >/dev/null
npm install -g --prefix "$T/prefix" --ignore-scripts "$T"/sos-kit-*.tgz >/dev/null
export SOS_BIN_DIR="$T/realbin"
out=$("$T/prefix/bin/sos" --version 2>"$T/err") || { cat "$T/err"; fail "real install failed"; }
case "$out" in "sos $V "*) ;; *) fail "real install gave '$out', expected sos $V";; esac
echo "npm real install ok ($out)"
