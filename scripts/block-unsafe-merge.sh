#!/usr/bin/env bash
# PreToolUse hook (Bash matcher) — B+3 FAIL-CLOSED SHIM [P064, Sếp-ratified 2026-06-09].
#
# The merge-gate logic lives in the `claude-hooks` Rust binary (subcommand
# `block-unsafe-merge`): blocks `gh pr merge <N>` / force-push unless the PR carries a
# /security-review APPROVE sentinel. This file is ONLY the deploy shim:
#
#   binary PRESENT → exec it (stdin JSON + env flow through untouched)
#   binary ABSENT  → exit 2 = BLOCK, LOUD.
#
# Why fail-closed (B+3, NOT bash-fallback A): this hook gates merges on security
# surfaces. Binary missing → exit 127 → harness would ALLOW = the gate silently opens
# (P059: the old bash gate already died silently-open on Windows once). A loud total
# block self-heals in one command — the bash fallback resurrects the rot the Rust port
# exists to kill. Doctrine: WORKFLOW_V2.2.md §B "guard bảo mật → fail-closed";
# decision trace: docs/BACKLOG.md [P064] RATIFIED DECISION.
#
# The 3 fail-open hooks (architect-guard / block-env-edit / session-banner) keep their
# bash here for now — for them, absent binary = allow = their correct default, so a
# shim adds nothing; direct binary wiring is the claude-hooks-side rollout (khi nguội).

# Trusted install locations FIRST (Giám sát 2026-06-11: bare-name exec = a PATH-prepended
# fake `claude-hooks` would be exec'd by the security gate itself). install.sh targets
# ~/.local/bin; cargo install targets ~/.cargo/bin. PATH lookup is the LAST resort only.
for cand in "$HOME/.local/bin/claude-hooks" "$HOME/.cargo/bin/claude-hooks"; do
  [ -x "$cand" ] && exec "$cand" block-unsafe-merge
done
command -v claude-hooks >/dev/null 2>&1 && exec claude-hooks block-unsafe-merge

echo "🚫 BLOCKED (fail-closed): \`claude-hooks\` binary not found — the merge security gate cannot run." >&2
echo "   Every Bash call is blocked until it is installed (1 command, no Rust needed):" >&2
echo "     curl -fsSL https://raw.githubusercontent.com/aspelldenny/sos-kit/main/install.sh | sh" >&2
echo "   Dev path: cargo install --path ~/claude-hooks" >&2
exit 2
