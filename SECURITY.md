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

**Approval-transition deadlock fix + actor-check (P078e Task 1, 2026-07-23).** P079 round-2
live-dogfood found the #5 exemption above only covered *create* — once the state file existed
(post-bootstrap, `version=V1, approved_version=empty`), the very first legitimate approval write
(`version=V2, approved_version=V2`) was BLOCKed by the version-match check, a circular deadlock
(the write that grants approval is itself gated on approval already existing). Fixed: the
state-file-alone exemption now covers create **or** update, gated on a new **actor-check** —
`.sos-state/worker-active` and `.sos-state/architect-active` must both be absent (mirrors
`orchestrator-guard.sh`'s `[ -f ".sos-state/worker-active" ] && continue` pattern) — so only the
owner/orchestrator main-thread can exercise the exemption; a worker/architect subagent turn gets
no special treatment and still falls through to the ordinary version-match BLOCK. The multi-path
all-path check (#6 above) is unchanged and still blocks any patch bundling `ticket-state.env`
with a second path. Verified via mock-payload tests (round-2 repro ALLOW, worker/architect-marker
self-approve BLOCK, pre-approval code-edit regression still BLOCK, bundle still BLOCK) plus a
negative-test: removing the actor-check flips the self-approve-BLOCK test to ALLOW
(`crates/sos-adapter-codex/src/lib.rs`, `docs/discoveries/P078e.md`).

**Codex caveat — actor-check is defense-in-depth, not SOUND, on Codex.** The actor-check depends
on the same `worker-active`/`architect-active` markers that gap-#6/P078d2b below documents as
**not reliably set inside custom-role Codex subagents**. On Claude the check is correct and
complete. On Codex, a worker subagent's `apply_patch` may not even route through this in-session
hook — so approval-record integrity on Codex still rests on **human-review-at-the-git-commit
boundary**, not this gate. `sos install` rendering + arming real Git hooks by default (closing
that boundary gap) is tracked separately as **P078f + P078g — SHIPPED (2026-07-23)**, see below and
`docs/adapters/P079-ROUND2-FINDINGS-2026-07-23.md` / `docs/adapters/P079-ROUND3-FINDINGS-2026-07-23.md`.

**Actor-check advance-block + path-normalize (P078h, 2026-07-23) — closes the P078e self-approve
hole.** P079 round-3 live-dogfood (`docs/adapters/P079-ROUND3-FINDINGS-2026-07-23.md` §B4) found
that P078e's actor-check only stopped *removing* the state-file-alone exemption for a marked
actor — it still fell through to the ordinary version-match check below, which compares only
**pre-patch** state. Starting from an already-approved `version=V3, approved_version=V3`, a
worker with `.sos-state/worker-active` present could write `version=V4, approved_version=V4` and
the version-match check saw `V3 == V3` (already approved) and let it through: a self-approve
**advance** hole, not a self-approve **create** hole. Fixed: a marked actor
(`.sos-state/worker-active` **or** `.sos-state/architect-active` present) is now BLOCKed outright
for **any** write that touches `.sos-state/ticket-state.env` — create, update, or advance — with
NO fall-through to version-match. Version-match is no longer an escape hatch for a marked actor
under any circumstance; only the main-thread (both markers absent) reaches the version-match
checks at all. A second, independent fix in the same phiếu normalizes a repo-root-absolute
`apply_patch` path to its relative form (stripping a literal `$REPO_ROOT/` prefix, computed via
the same `git rev-parse --show-toplevel || pwd` REPO_ROOT already used elsewhere in the guard)
**before** both the state-file-allow compare and the multi-path all-path check consume it — P079
round-3's "Additional usability observation" found the previous exact-relative-string compare
false-blocked a legitimate main-thread approval write whose patch happened to carry an absolute
path. The normalize step is deliberately conservative: it is a plain string-prefix strip, not a
`realpath`/symlink resolution, so any path that ends up still containing `..` after the strip (or
was never under `REPO_ROOT` to begin with) is left as-is and therefore can never equal the exact
state-file string — an unresolved/foreign-looking path always falls through to fail-CLOSED rather
than being trusted. Verified via mock-payload tests (worker/architect-marker advance-of-approved
BLOCK, marked-actor create BLOCK, main-thread relative-path no-regress ALLOW, absolute-path
normalize ALLOW on main-thread / BLOCK when marked, bundle-with-absolute-path no-regress BLOCK)
plus two negative-tests: reverting the early-BLOCK flips the advance-of-approved test from
BLOCK back to ALLOW; reverting the normalize step flips the absolute-path main-thread test from
ALLOW back to false-BLOCK (`crates/sos-adapter-codex/src/lib.rs`, `docs/discoveries/P078h.md`).
**Codex caveat unchanged from P078e:** this fix strengthens the guard's LOGIC, it does not change
WHETHER the guard fires inside a real Codex custom-role subagent — that gap (`worker-active`/
`architect-active` markers absent on Codex 0.145.0, upstream `openai/codex#21753`) is still open
and out of scope here. The advance-block fix is SOUND on Claude (in-subagent hooks fire reliably)
and for the manual-marker Codex repro path; on a real Codex custom-role subagent spawn it remains
best-effort/inactive, same as the rest of the actor-check — the Git backstop (P078f/P078g arming)
and human-review-at-commit remain the real boundary on Codex.

**In-subagent role-envelope enforcement — MISSING, not PARTIAL (P078d2b, upstream
`openai/codex#21753`).** All 5 guards above are `PreToolUse`/`UserPromptSubmit` hooks that fire
on the **main thread**. On Codex 0.145.0, they do **not** fire inside a spawned **custom-role**
subagent (`architect`/`worker`/`advisory-watch`/`boundary-check`) — only `agent_type="default"`
dispatches `SubagentStart`/`SubagentStop` and (by extension) in-subagent `PreToolUse`; custom-role
spawns get nothing (probe: `docs/adapters/SUBAGENT-HOOK-PROBE-2026-07-22.md`). This is
**dogfood-confirmed, not theoretical**: `docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md`
#4 — a spawned architect's forbidden `apply_patch` against Rust `src/` succeeded in-subagent, and
the `architect-active` marker was never created. **Do not trust an architect/worker subagent to
police its own envelope on Codex; trust the Git gate.** The real backstops are: (1) main-thread
`PreToolUse` guards (dogfood-confirmed to fire — `orchestrator-guard.sh`/`block-env-edit.sh`
blocked real attempts on the main thread), (2) universal Git pre-commit/pre-push (agent-agnostic —
fires at the commit/push boundary regardless of runtime), (3) `AGENTS.md` guidance (prose only).
The `SubagentStart`/`SubagentStop` marker hooks in `.codex/hooks.json` are retained best-effort
(they still fire for DEFAULT subagents and would activate for custom roles if upstream `#21753`
is fixed) but are explicitly NOT relied upon for this or any other capability claim — see
`CodexAdapter::verify()` Finding #6 and `adapters/codex/CAPABILITY.md` §6.

## `sos install`: Git hooks are RENDERED + armed-by-default (P078f + P078g, 2026-07-23)

`sos install --runtime {claude,codex}` (the Rust `sos-install` engine,
`crates/sos-install/src/engine.rs`) now **renders the real hook scripts AND
activates** the Git hook backstop as part of a successful install — it no
longer just points `core.hooksPath` at a directory and hopes something is
there.

**Gap history (read chronologically — the boundary went through 2 fixes):**
- **P078f (2026-07-23, `d35f462`)** added the arming step (`core.hooksPath` +
  F09 hijack-guard) but assumed `hooks/pre-commit`/`hooks/pre-push` were
  already on disk from adapter rendering. They weren't — **neither runtime's
  adapter `plan()` ever rendered git hook scripts** (P079 round-3, Test A2/A3/A4
  FAIL). Result: `core.hooksPath` got armed pointing at an EMPTY `hooks/` dir —
  looked active, blocked nothing. A real `.env` commit (A3) and a real
  code-on-default commit (A4) both went through.
- **P078g (2026-07-23, this fix)** closes that gap: `render_embedded_hooks()`
  now writes `hooks/pre-commit` + `hooks/pre-push` from **compile-time
  `include_str!`** of the kit-source scripts (baked into the `sos-install`
  binary — `sos install` has no `SOS_KIT_DIR`, it runs standalone in an
  arbitrary target repo, unlike `new`/`adopt`/`sync`), run in `apply()`'s
  success path immediately BEFORE `arm_git_hooks()`. And `arm_git_hooks()`
  itself is hardened to **never arm empty**: a new `hooks_present_and_executable()`
  check runs right before `git config --local core.hooksPath hooks` — if
  either script is absent/non-executable, arming is refused (`Err`, loud
  stderr) instead of silently pointing at nothing.

**What `apply()` does on success (engine-native, not a shell-out to
`scripts/install-hooks.sh` — Windows Git Bash may be absent):**
0. **(P078g) Render** `hooks/pre-commit` + `hooks/pre-push` from embedded kit
   source into the target repo. Non-clobber: a pre-existing hook file with
   DIFFERENT content (adopter customized it) is left untouched (warn only).
1. `chmod +x hooks/pre-commit` (+ `hooks/pre-push`) — Git silently ignores
   non-executable hooks.
2. **F09 hijack-guard:** if `git config --local core.hooksPath` is already set to
   something OTHER than `hooks` (the adopter has their own hook chain), a TTY session
   prompts `[y/N]` (default N); a non-TTY session (CI, piped install) **ABORTs** the
   install rather than silently overriding it. This never fires on our own prior
   value — repeat installs are idempotent.
2.5. **(P078g) Never-arm-empty guard:** if `hooks/pre-commit`/`hooks/pre-push`
   are absent or non-executable at this point, arming is refused loudly
   instead of proceeding.
3. `git config --local core.hooksPath hooks`.
4. Any pre-existing `.git/hooks/{pre-commit,pre-push}` is **renamed** (never deleted)
   to `*.pre-hookspath.bak` — an escape hatch for the adopter to recover their prior
   hook.

A target that isn't a git repository at all → warn-skip, install still succeeds
(non-git installs, e.g. scaffolding-only use, are not blocked).

**Symmetric for both runtimes:** `crates/sos-cli/src/commands/install.rs`'s
`run_adapter()` is the single shared call-site for `--runtime claude` and
`--runtime codex` — both call the same `engine::apply()`, so render+arm is
identical for either adapter; there is no runtime-specific branch that could
silently skip it for one of them. Known asymmetry, pre-existing and
out-of-scope for P078g: `ClaudeAdapter::plan()` is a total stub (renders 0
assets today) — this is unrelated to hook arming, which happens at the
engine level regardless of adapter plan content.

**Verified:** `crates/sos-install/tests/install.rs` runs the full render+arm
flow against a real temp `git init` repo with a **fresh-no-seed** `Plan`
(zero hook-related ops — P078f's own 5 tests seeded hook content into the
synthetic `Plan`, which is exactly why they didn't catch the P078g gap).
Assertions: rendered content byte-matches kit source, chmod, `core.hooksPath`
armed only AFTER files exist, a **real `git commit` of a real `.env`** is
genuinely blocked (the armed hook chain runs `scripts/block-env-commit.sh`
for real, not just "file exists" — a clean commit is the negative control),
refuse-when-absent (`arm_git_hooks` called with nothing rendered → `Err`,
`core.hooksPath` stays unset), render non-clobber (pre-existing customized
`hooks/pre-commit` preserved verbatim), plus all 5 P078f tests still green
(`.bak` rename, non-clobber abort, non-git warn-skip, idempotent re-install).
See `docs/discoveries/P078g.md` and `docs/discoveries/P078f.md`.

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
