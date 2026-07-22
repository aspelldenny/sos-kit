# Security Policy — SOS Kit

## What this kit auto-executes when you trust the folder

SOS Kit is a public distribution kit that **intentionally ships auto-exec surfaces** as its product. When you clone this repo and configure Claude Code to trust the folder (or run `scripts/install-hooks.sh`), the following surfaces become active:

| Surface | Trigger | What it does |
|---|---|---|
| `hooks/pre-commit` | `git commit` | Runs 8 pre-commit gates (type-check, docs-gate, BACKLOG, security, case-collision, no-code-on-default, env-block, trust-gate) |
| `hooks/pre-push` | `git push` | Pre-push checks |
| `scripts/*.sh` (13 scripts) | Various hooks, Claude Code PreToolUse/UserPromptSubmit | Guard scripts, security gate, session banner, etc. — each declared in `CLAUDE.md` scripts list |
| `.claude/settings.json` | Claude Code session start | Declares allowed Bash operations (permission allowlist) |
| `.mcp.json` | Claude Code session start | Declares MCP servers (local Rust binaries from related repos: ship, guard, vps, docs-gate) |
| `bin/sos.sh` | User invokes `sos <cmd>` | CLI entrypoint delegating to subcommands |
| `install.sh` | `curl <url> | sh` — primary installer | Sets up hooks, symlinks, and binary installations declared in this kit |
| `phieu/phieu.sh` | User sources in shell (`~/.zshrc`) | Shell functions: `phieu`, `phieu-init`, `phieu-done`, `phieu-list` |
| `templates/setup-dev.sh` | Contributor runs manually | Developer environment setup for contributors |

**Not auto-exec:** `agents/`, `skills/`, `docs/`, `phieu/*.md`, `configs/`, `recipes/` — these are read-only markdown/TOML that Claude loads as context but does not execute.

**Not shipped (per-machine, not git-tracked):** `.claude/settings.local.json` — this file is globally gitignored by design (it holds per-machine permission overrides). It is outside the baseline gate scope. Each adopter configures it locally; verify its contents manually if your threat model requires it.

---

## Invariants (the trust contract)

These are the properties the kit guarantees. Each is mechanically enforced:

**INV-TRUST-01: Auto-exec surface content integrity (P073)**
Every tracked auto-exec surface (see table above) has its sha256 committed in `.sos-trust-baseline`. The `scripts/trust-gate.sh` pre-commit gate fails if any surface's hash differs from baseline. A reviewed change is accepted only by running `scripts/trust-gate.sh rebaseline` after human review — making the diff visible in the PR before it reaches any adopter.

**INV-TRUST-02: No hidden Unicode in instruction files (P073)**
The `scripts/trust-gate.sh` gate scans all instruction/doc files Claude loads into context (`CLAUDE.md`, `agents/`, `skills/`, `phieu/`, `docs/`, `.claude/`, `README.md`, `SECURITY.md`) for hidden Unicode codepoints: U+FEFF BOM, U+200B/C/D zero-width, U+200E/F bidi marks, U+2060 word joiner, U+180E Mongolian vowel separator. Any hit fails the commit. These codepoints are the "Rules-File-Backdoor" prompt injection vector (invisible to human review, visible to the LLM).

**INV-TRUST-03: No runtime URL fetch by hooks/scripts**
Hooks and scripts operate on local files and git only. No hook makes an outbound HTTP request at commit time. MCP servers (declared in `.mcp.json`) are local Rust binaries installed from pinned releases — not dynamically fetched at runtime.

**INV-TRUST-04: Nothing is hidden from the user**
Every file in this repo is readable. The hook chain is documented in `hooks/pre-commit` header. The permission allowlist in `.claude/settings.json` is committed and gated by the baseline. No obfuscation, no eval of remote content.

**INV-TRUST-05: Install only does declared operations**
`install.sh` only performs: symlinking `hooks/` via `core.hooksPath`, copying skill files, and installing declared binary releases (ship, docs-gate, guard, vps, inv-gate, claude-hooks). Each release is checksum-verified (P071 — Leg 2 of the hardening sprint). The install script does not fetch or execute arbitrary remote code beyond the declared binary URLs.

**INV-TRUST-06: Release binaries are checksum-verified (P071)**
The `install.sh` verifies sha256 of each downloaded binary against a pinned checksum before execution. See `install.sh` for the verification block.

---

## Trust anchor

When you adopt SOS Kit, you are trusting:

1. **This GitHub repository** (`aspelldenny/sos-kit`) and the maintainer account that controls it. Compromising the maintainer's GitHub account is the primary trust boundary.
2. **The committed `.sos-trust-baseline`** — a sha256 snapshot of all auto-exec surfaces at the time of the last reviewed commit. Any change to an auto-exec surface after that point will fail the pre-commit gate for contributors — and will be visible as a diff in the PR for adopters reviewing before `git pull`.
3. **The binary releases** (ship, docs-gate, guard, vps, inv-gate, claude-hooks) from their respective GitHub repos. These are separated from sos-kit intentionally: a compromise of this repo does not compromise the binaries (separate signing + release pipelines).

**What protects you at the content layer (this phiếu's contribution):**
- `scripts/trust-gate.sh` runs at every `git commit` for kit contributors → any auto-exec surface modification requires a conscious `rebaseline` step before it can be committed.
- For adopters: review the `.sos-trust-baseline` diff in any PR that touches auto-exec surfaces. A PR that modifies `hooks/pre-commit` without updating `.sos-trust-baseline` is suspicious.

**What does NOT protect you:**
- A compromised maintainer GitHub account that creates a malicious PR + approves it via a second account. Mitigation: watch releases + the baseline diff in PRs.
- Untracked files you add locally that match auto-exec patterns but were never `git add`ed before rebaseline — these are silently missed (the git ls-files gotcha). Run `scripts/trust-gate.sh rebaseline` after `git add`ing any new auto-exec file.

---

## Rebaseline workflow

When a legitimate change is made to an auto-exec surface:

```bash
# 1. Edit the surface (e.g. hooks/pre-commit)
# 2. Review your changes
# 3. Stage all changes including the surface
git add <changed-surface-file>
# 4. Regenerate the baseline (note: new files must be `git add`ed FIRST)
scripts/trust-gate.sh rebaseline
# 5. Stage the updated baseline
git add .sos-trust-baseline
# 6. Commit — trust-gate will now pass
git commit -m "..."
```

The baseline diff in `git diff .sos-trust-baseline` shows exactly which surfaces changed — this is the human-readable audit trail in the PR.

---

## Codex adapter enforcement (rendered to target, PARTIAL)

P078b3 rewrote 5 `PreToolUse`/`UserPromptSubmit` guard scripts (`scripts/codex/*`) plus
`.codex/hooks.json` and `.codex/rules/exec-policy.rules`, crate-embedded in
`crates/sos-adapter-codex/src/templates.rs`. These are **rendered to a target project** via
`sos install --runtime codex` — they are NOT part of this repo's own auto-exec surface and are
NOT covered by `.sos-trust-baseline` (that baseline only tracks bytes that actually exist and
auto-run inside `sos-kit` itself; `.codex/`/`scripts/codex/` never land in this repo's tree,
per Decision 6, `docs/ticket/P078b3-codex-enforcement.md`).

Honest 3-surface statement (kept consistent across `verify()`, `adapters/codex/CAPABILITY.md`,
and this section): the rendered guards are **bypassable** — Codex only runs project hooks for
TRUSTED repos, non-managed hooks need explicit `/hooks` trust, and a user can disable hooks
entirely. **Git/CI backstops (branch protection, PR review-trigger map) are retained as the
real security boundary** for every project that adopts the Codex adapter — the hook layer is
fast-feedback only, never claimed as unbypassable. The guards also invert Claude's fail-open
default for unparseable input: an `apply_patch` payload whose patch body doesn't match Codex's
`*** Add/Update/Delete/Move File:` marker BLOCKs (fail-CLOSED), because an unparsed Codex patch
could otherwise write anywhere silently.

**Multi-path bypass hole — CLOSED (P078d2a, 2026-07-22).** P079 live-dogfood found all 5 guards
above extracted only the FIRST `*** ... File: <path>` marker from a multi-file `apply_patch`
(`| head -n1`) — a patch listing an allowed path FIRST and a forbidden path (`.env`, `src/**`,
`.sos-state/ticket-state.env`) SECOND would exit ALLOW on the first match and never inspect the
rest, silently letting the second write through. Fixed: every guard with an allow-list now
extracts EVERY path in the patch and **BLOCKs if ANY path violates** the rule (ALLOW requires
every path to be exempt — the previous "first-path-exempt → allow-all" semantic is gone).
Verified via a negative-test: reverting to `head -n1` flips the bypass-fixture tests from
BLOCK(exit 2) back to ALLOW(exit 0) (`crates/sos-adapter-codex/src/lib.rs`,
`docs/discoveries/P078d2a.md`).

**Approval-gate self-bootstrap exemption (P078d2a #5) — coupled with the fix above.**
`scripts/codex/approval-gate.sh` now allows a patch that touches ONLY
`.sos-state/ticket-state.env` (and no other path) through when the state file doesn't exist yet,
so a fresh install isn't permanently deadlocked (previously: BLOCK on every non-ticket patch when
the state file was missing, including the patch that would create it). **This exemption is safe
only because the multi-path fix above lands in the same patch** — a patch bundling
`ticket-state.env` with a second path is caught by the all-path check and falls through to the
normal fail-CLOSED BLOCK, not the bootstrap ALLOW. A rendered skeleton state file
(`.sos-state/ticket-state.env`, empty `version=`/`approved_version=`) is now also emitted at
install time so most fresh installs never reach the missing-file branch at all; its non-clobber
safety reuses the pre-existing `sos-install::engine` checksum/manifest logic — no engine change.

## Install: tool-version drift (OA-07) is workflow-safety, not a trust boundary (P078c)

`sos install` reorders render vs. tool-manifest check (P078c): adapter files are written first,
tool-version drift is reported after (loud WARNING + exit 3) instead of blocking the write, with
`--require-tools` as an opt-in fail-closed escape hatch for CI/production. **N/A explicit for
this repo's threat model:** OA-07 (sister-tool version drift — `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md`)
is a 🟠 workflow-safety concern (repo contract may assume tool behavior a stale local binary
doesn't have yet) — it does not touch an auto-exec surface, a trust anchor, or the checksum
verification chain described above. Rendering adapter files is pure filesystem write of
crate-embedded content (no remote fetch, no code execution) regardless of tool-drift state;
nothing here weakens `.sos-trust-baseline`, the checksum-verified binary installs, or the
Codex-adapter guard fail-CLOSED behavior documented above.

## Reporting a vulnerability

If you discover a security issue in SOS Kit:

- **Preferred:** Open a [GitHub Security Advisory](https://github.com/aspelldenny/sos-kit/security/advisories/new) (private, then disclosed after fix).
- **Alternative:** Email the maintainer directly (see GitHub profile).

Please do not open a public issue for security vulnerabilities before a fix is available.

---

## Future work

- **Porting the trust gate to `claude-hooks` binary (P012):** The current gate is a bash script in this repo. Porting it to the `claude-hooks` Rust binary for stronger tamper-resistance is tracked in BACKLOG line 249. This is deliberately out of scope for P073.
- **GitHub Tier-1 hardening:** Secret scanning, push protection, branch ruleset, and fork-PR approval are already enabled on this repo (separate from this phiếu's content-integrity layer).
