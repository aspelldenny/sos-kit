# Changelog

All notable changes to sos-kit. Format loosely follows Keep a Changelog. Versions are wave-based, not date-based.

**Older entries (P078b3 and earlier — v2.3 wave start through v2.1/v2.0/v1) archived to `docs/archive/CHANGELOG_pre-P078c.md`** on 2026-07-23 to keep this file under the 40k doc-size threshold.

## v2.3 forge (in progress) — Phiếu path + sentinel + agents-drift cure + portability architecture — 2026-07-22

**Recipe Tier-0 harvest — nextauth + payos SDK rewrite (2026-07-23):**
- Added `recipes/auth/nextauth-google-credentials.md` — NextAuth v4 Google OAuth +
  Credentials (JWT strategy), mined from tarot `src/lib/auth.ts` (verified @cd16a86).
  Real-code gotcha caught vs Architect's draft: without a DB adapter, `jwt()`'s `user`
  param carries the OAuth provider's id (Google `sub`) on first sign-in, not the app's
  DB id — `token.sub` must be re-resolved by email lookup and overwritten. Also fixed
  `token.uid` → `token.sub` (NextAuth's real field).
- Rewrote `recipes/payment/payos-vn.md` — raw-HMAC → official `@payos/node@2.0.5` SDK.
  Real-code corrections vs draft: `new PayOS({clientId,apiKey,checksumKey})` is an
  **object-arg constructor**, not positional args; bad-signature webhook returns
  **400** (non-retriable), not 401; VIP grant is a separate `Subscription` model
  (tier/status/currentPeriodStart/currentPeriodEnd), not a `vipUntil` column on `User`;
  dropped an unverified `data.reference` field the draft assumed (tarot only ever
  reads `.orderCode`). Both recipes carry a `## Forge verification` section with
  anchors run live against `~/tarot` @cd16a86.
- Updated `recipes/README.md` recipe index + root `README.md` recipes file tree.

**[P081b] Distribution — Stage 2 npm wrapper (2026-07-23):**
- Added `package.json` (`sos-kit`, `0.1.0` — synced to `crates/sos-cli`
  Cargo.toml + tag `v0.1.0`): thin package, `bin.sos` → `bin/sos-npm`,
  `bin.sos-kit-setup` → `scripts/npm-postinstall.sh` (manual fallback), `os:
  [darwin, linux]` (no Windows), `files` whitelist (no forked install logic
  shipped as JS).
- Added `scripts/npm-postinstall.sh`: downloads `install.sh` from PINNED tag
  `v0.1.0` (not `main`), verifies its sha256 against `scripts/install-sh.sha256`
  (shipped in the package, computed from `git show v0.1.0:install.sh`) —
  fail-CLOSED on mismatch/download failure, then runs it. `install.sh` itself
  unchanged — single source of truth for the 10 sister tools + `sos-bin` +
  wrapper.
- Added `bin/sos-npm`: thin delegate to the `$BIN_DIR/sos` wrapper install.sh
  writes; missing (postinstall skipped via `--ignore-scripts`, or failed) →
  prints setup guidance + exits non-zero instead of half-dispatching.
- `.sos-trust-baseline` rebaselined (+`bin/sos-npm`; `scripts/npm-postinstall.sh`
  covered by existing `scripts/*.sh` glob); `scripts/trust-gate.sh`
  `SURFACE_GLOBS` gained `bin/sos-npm`.
- Nghiệm thu (isolated prefix, zero touch to real machine): `npm pack` →
  install rc=0, 10 tools + `sos-bin` + wrapper present, `sos tools status`
  rc=0, `sos-bin --version` → `0.1.0`, checksum-tamper → abort exit 1,
  `--ignore-scripts` → clean no-op + `sos-kit-setup` fallback verified
  end-to-end. Docs: `INSTALL.md`, `README.md`, `SECURITY.md` (new npm
  threat-model subsection). **NOT published** — `npm publish --access public`
  is a manual Sếp/Quản đốc step (BACKLOG Park), same discipline as the
  `v0.1.0` tag push. Full report: `docs/discoveries/P081b.md`.

**[P081] Distribution — Stage 1 release pipeline + checksum + curl|sh (2026-07-23):**
- Added `.github/workflows/release.yml`: tag `v*` → build `sos` binary for
  `aarch64-apple-darwin` (tested target) + `x86_64-unknown-linux-gnu`
  (build-only, NOT dogfood-tested) → GitHub Release (draft→publish) with
  `.sha256` companion per asset. Windows dropped from Stage 1 matrix.
- Release tag axis clarified: binary version (`sos-cli` Cargo.toml,
  `v0.1.0` first release) is a SEPARATE axis from doctrine version ("v2.3
  forge" above) — do not conflate.
- `tool-manifest.toml`: FULL checksum fill — all 10 sister tools' real
  sha256 fetched from their published GitHub Release asset digests at the
  pinned version. Only exception: `advisory-cron`'s Windows triple (no
  asset published — `compile_error!` on Windows by design), kept as TODO
  + comment.
- `install.sh`: route A — fetches prebuilt `sos-<triple>` + `.sha256`
  companion into sidecar `$BIN_DIR/sos-bin` (fail-CLOSED on
  missing/mismatch), wrapper exports `SOS_RUST_BIN` with `:=` default
  (user-set env still wins). `bin/sos.sh` dispatch contract untouched.
- Docs: `SECURITY.md` (distribution auto-exec surface), `.sos-trust-baseline`
  rebaselined, `INSTALL.md` (curl\|sh command + prose 9→10 tools, added
  `inv-gate`), `docs/SETUP.md`, `README.md`.
- Stage 2 (npm/pnpm wrapper + native plugins) PARKED in BACKLOG — gated on
  Stage 1 running for real ≥1 release.
- Full report: `docs/discoveries/P081.md`.

**[P080] Dual-runtime brownfield dogfood — ROUND-2 PASS, P080 DONE, P081 UNGATED (2026-07-23):**
- Re-ran D1 on a pristine fixture post P080x merge (`1821dca`): missing
  `block-env-commit.sh` + real `.env` staged → BLOCKED exit 1; missing
  `no-code-on-default.sh` + product code staged → BLOCKED exit 1; negative
  control (both guards present, clean commit, feature branch) → exit 0. Gap
  closed, no new regression. Smoke-reran A2 (dual render) + B1 (brownfield
  non-clobber) — unchanged, no regression.
- **Task 3 (cross-runtime state, real `codex exec` 0.145.0) — PASS.** C1
  (main-thread `apply_patch` write) → hooks fire, patch completed. C2
  (`.sos-state/worker-active` marker present, Codex attempts advance) →
  `PreToolUse Blocked`, state unchanged (correct). C3 (no marker, main-thread
  advance) → completed, Claude-side re-read agrees. 3 operational caveats
  logged (sandbox default read-only needs `--sandbox workspace-write`; hook
  enforcement is trust-gated and fails SILENTLY on untrusted repos — must
  verify the `"hook: PreToolUse ..."` line before trusting a verdict;
  `trusted_hash` is content-based, clonable across same-template fixtures).
- E2 (Linux) formally **DEFERRED per Sếp decision** — not a gap, an
  intentional scope choice (macOS first, Windows-Linux dual-runtime later).
- **P081 (distribution) UNGATED.**
- Full report (round-2 section): `docs/adapters/P080-FINDINGS-2026-07-23.md`.

**[P080x] FIX (SECURITY) — Dev `[8/8]` hook `hooks/pre-commit`: `[6/8]` no-code-on-default + `[7/8]` block-env now fail-CLOSED on missing guard script (closes P080 round-1 gap D1, 2026-07-23):**
- **Gap closed:** P078i's fail-CLOSED fix only shipped in the `sos install` backstop hook (`crates/sos-install/src/templates/backstop-pre-commit.sh`); the dev `[8/8]` hook that `sos new` copies (`hooks/pre-commit`) still had the old fail-OPEN else-branches for these 2 phases — live-verified `exit 0` when the guard was deleted and a real `.env` was committed.
- **Fix:** `hooks/pre-commit` `[6/8]`/`[7/8]` else-branches now `red` + `FAIL_COUNT++` on missing guard script (commit blocked) instead of `⏭ skip` (commit silently allowed). Scoped to exactly the 2 invariants P078i's backstop covers; `[1-5]`/`[8]` untouched (deferred, see phase-decision table in phiếu). Phase count `[8/8]` unchanged.
- Verified via pristine fixture (no seeding beyond guards + hook): missing-guard `.env` commit BLOCKED exit 1, missing-guard code-on-default commit BLOCKED exit 1, both-guards clean commit exit 0 (negative control), both-guards `.env` commit still BLOCKED exit 1 (regression), 2-directional revert/restore test.
- Docs: `SECURITY.md` (new section), `docs/SETUP.md` `[6/8]`/`[7/8]` descriptions, `CLAUDE.md` scripts list.
- Full report: `docs/discoveries/P080x.md`.

**[P080] Dual-runtime brownfield dogfood — FAIL, 1 gap found (2026-07-23):**
- Ran the `[Thợ-local]` half of the P080 test matrix (Nhóm A fresh-dual incl.
  A2-reverse + A5-uninstall, Nhóm B brownfield, Nhóm D regression incl.
  sync/map dual smoke) on fresh git fixtures (NOT the sos-kit checkout).
  Nhóm C (cross-runtime state, real `codex exec`) and E2 (Linux) stay
  `[Sếp+Codex]` PENDING per phiếu scope.
- **Dual render co-exists cleanly** both orders (Claude `sos new`/`sos adopt`
  + Codex `sos install --runtime codex`): non-clobber holds, `core.hooksPath`
  arms exactly once, no cross-runtime file destruction, `sos sync`/`sos map`
  smoke-clean on a dual-installed repo.
- **D1 FAIL (HIGH):** the dev `[8/8]` `hooks/pre-commit` (rendered only by
  `sos new`, Claude path) is fail-OPEN on `[6/8]` no-code-on-default and
  `[7/8]` block-env-commit when their delegated scripts are missing — the
  exact "round-4 A3/A4" bug the Codex-side backstop-minimal hook (P078i) was
  supposed to have fixed. The fix only landed in the backstop hook, never
  ported back to the dev hook. Proved live: remove
  `scripts/block-env-commit.sh` + rebaseline trust-gate → a real `.env`
  secret commits cleanly (exit 0). Recommend `P080x-hook-fail-open-parity`
  (Tầng 1, Debate) to port the fail-closed pattern into the dev hook.
- **A5 N/A (not FAIL):** `uninstall()`/`RemovalPlan` is an honest empty stub
  on both adapters, with no `sos uninstall` CLI subcommand wired at all —
  same known-gap class as `ClaudeAdapter::plan()` (anchor #5).
- A4 caveat (not a FAIL): on a brand-new zero-commit repo,
  `no-code-on-default.sh`'s `refs/heads/main` fallback resolution can't see
  the not-yet-created branch ref → warns+allows on the very first commit.
  Narrow window, pre-existing, worth a note in the same gap ticket.
- **P081 distribution stays gated** until P080x closes and this round re-runs
  green.
- Full report: `docs/adapters/P080-FINDINGS-2026-07-23.md`. Discovery:
  `docs/discoveries/P080.md`.

**[P078j] FIX (SECURITY) — Codex adapter: guard path-matching upgraded from lexical strip to symlink-safe canonicalize — closes P079 round-4 B4 for real (2026-07-23):**
- **Gap closed (P079 round-4 §B4, `docs/adapters/P079-ROUND4-FINDINGS-2026-07-23.md` `:149-191`, "New/remaining gaps" #2):** P078h's `apply_patch` path-normalize was a plain literal `$REPO_ROOT/` string-prefix strip, not filesystem-equivalent. On macOS `/tmp` is a symlink to `/private/tmp`; `git rev-parse --show-toplevel` returns the canonical `/private/tmp/...` form, so a patch path spelled `/tmp/...` never shared a literal prefix with `REPO_ROOT` even though it names the identical file. This broke BOTH directions: a marked actor's forbidden self-advance (`V3/V3` → `V4/V4`) LEAKED through the alias (early-BLOCK's path compare never matched `STATE_FILE`), and a legitimate main-thread approval write via the same alias was false-BLOCKed (never won the state-file-alone exemption).
- **Fix:** `crates/sos-adapter-codex/src/templates.rs` (`guard_approval_gate_sh()`) now canonicalizes BOTH `REPO_ROOT` (`cd "$REPO_ROOT" && pwd -P`, already `cd`'d there) AND every absolute candidate patch path BEFORE the prefix-strip/compare — candidate canonicalize resolves the DIRNAME only (`cd "$(dirname PATH)" && pwd -P`, re-appending the basename), which stays correct for a file that does not yet exist on disk (bootstrap-create case, P078d2a #5/B1) and is portable to macOS BSD `pwd`/`cd` (no GNU `readlink -f`/`realpath -m` assumed). One normalize site, same two consumers as P078h (state-file-alone exemption + d2a multi-path all-path check). Does NOT alter the P078h early-BLOCK/exemption decision logic — only the path forms fed into it. Conservative fail-closed preserved: an unresolvable dirname or a path canonicalizing outside `REPO_ROOT` is never granted the exemption.
- **`|| true` fix for `set -e`:** the dirname-resolve `cd ... && pwd -P` command substitution is a plain assignment statement — under `set -euo pipefail`, a failed `cd` (dirname doesn't exist) would otherwise abort the whole script; appended `|| true` so it degrades to an empty `RESOLVED_DIR` (fail-closed fallthrough) instead.
- Tests (`crates/sos-adapter-codex/src/lib.rs`, 4 new + reuse of 2 pre-existing P078h absolute-path tests for canonical/relative no-regress): a `run_guard_symlink()` fixture harness (real dir + separate symlink alias, guard's cwd bound to the real dir, payload built via the alias) reproduces round-4 B4 directly without depending on any OS-specific mount — marked-actor advance via alias → BLOCK, main-thread approval via alias → ALLOW, bundle-via-alias (marked-actor oracle) → BLOCK (multi-path no-regress), path-trick-outside-`REPO_ROOT` → fail-closed (never exempted). **Two-directional negative-test performed and reverted:** stashing the Task-1 canonicalize change flips the marked-actor-advance test BLOCK→ALLOW (leak reproduces) AND the main-thread-approval test ALLOW→BLOCK (false-block reproduces) simultaneously — confirmed red on both, then restored (fix re-verified green, 64/64 pass, ×20 flake-clean).
- Docs: `SECURITY.md` (new "Path-matching upgraded to symlink-safe canonicalize" section), `adapters/codex/CAPABILITY.md` (P078j addendum under the P078h entry), `docs/BACKLOG.md`.
- Full report: `docs/discoveries/P078j.md`.

**[P078i] FIX (SECURITY) — `sos install` now renders a self-contained, fail-CLOSED backstop `hooks/pre-commit` (not the dev [8/8] hook) — closes P079 round-4 A3/A4 for real (2026-07-23):**
- **Gap closed (P079 round-4 `docs/adapters/P079-ROUND4-FINDINGS-2026-07-23.md`, Test A3/A4 FAIL, "structural-oracle-gap lần 3"):** P078g (below) correctly rendered + armed `hooks/pre-commit`/`hooks/pre-push`, but the CONTENT it rendered was the kit's own dev `[8/8]` hook — built for `sos new` dev checkouts, delegating every phase to `scripts/*.sh` + `docs-gate`, and **fail-OPEN** when a delegated script is absent (`echo "... missing"` then falls through, no `exit`). A fresh `sos install` into a brownfield repo never renders that `scripts/` tree — both required probes (`.env` block, no-code-on-default) silently passed on real commits (`ENV_COMMITTED=yes`, `CODE_COMMITTED=yes`).
- **Fix — route (a), minimal purpose-built backstop calling embedded verbatim guards (Debate Log Turn 1 recommendation, EXECUTE confirmed feasible):** new template `crates/sos-install/src/templates/backstop-pre-commit.sh` (embedded `EMBEDDED_BACKSTOP_PRE_COMMIT`) enforces ONLY 2 invariants — `.env*` block + no-code-on-default — no docs-gate/trust-gate/type-check phases. **fail-CLOSED, the opposite of the round-4 bug:** each phase checks its delegated guard exists first; if absent, `exit 1` with a `BLOCKED:` message — never "missing → allowed". `render_embedded_hooks()` (`crates/sos-install/src/engine.rs`) now co-renders the backstop hook AND its 2 delegated guard scripts (`scripts/block-env-commit.sh`, `scripts/no-code-on-default.sh` — embedded verbatim, single-source with the kit's own copies, `EMBEDDED_BLOCK_ENV`/`EMBEDDED_NO_CODE_DEFAULT`) in the same render pass, so a fresh install's output is a fully closed dependency set; all 4 rendered artifacts get chmod +x immediately after write.
- **`sos new` (dev projects) untouched:** still copies the real dev `[8/8]` `hooks/pre-commit` + full `scripts/` tree at runtime via `crates/sos-cli/src/commands/new.rs:312`'s `copy_tree` — a code path that never calls `engine::apply()`/`render_embedded_hooks()` (confirmed zero overlap, Worker CHALLENGE Turn 1). `hooks/pre-push` unaffected (self-contained advisory-only, no delegated scripts, no closure risk — kept as-is for both `sos new` and `sos install`).
- **Structural-oracle-gap lần 3, closed with a dependency-closure assertion:** P078f's fixture seeded hook content into the synthetic `Plan` (missed "install never renders the hook"); P078g's fixture seeded `scripts/block-env-commit.sh` directly (missed "install renders a hook referencing a script install itself never writes"). New test `pristine_install_renders_closed_dependency_set_and_blocks_real_env_and_code_commits` runs against a git repo with **ZERO seeding** (no `hooks/`, no `scripts/` beforehand), then greps the rendered hook for every `scripts/*.sh` reference and asserts each exists among that same install run's own output — closing the recursive gap mechanically.
- Tests (`crates/sos-install/tests/install.rs`, 4 new + 1 updated): dependency-closure pristine oracle (real `.env` block, real code-on-default block, closure assertion, clean-commit negative control), fail-closed-when-guard-absent (delete a just-rendered guard, confirm next `.env` commit is STILL blocked), 2 template-content assertions (references only its 2 deps, never docs-gate/trust-gate/inv-gate/python/install-hooks.sh). `cargo test -p sos-install`: 19/19 pass; `cargo test --workspace` green; `cargo build --release` clean.
- Docs: `SECURITY.md` (new "self-contained backstop" section, gap-history count 2→3), `adapters/codex/CAPABILITY.md` (gap #4 correction — round-4 found the P078g "genuinely blocked" claim was conditional, now unconditionally true), `docs/BACKLOG.md`.
- Full report: `docs/discoveries/P078i.md`.

**[P078g] FIX (SECURITY) — `sos install` now RENDERS real `hooks/pre-commit`+`pre-push` before arming; never arms an empty `hooksPath` (2026-07-23):**
- **Gap closed (P079 round-3 `docs/adapters/P079-ROUND3-FINDINGS-2026-07-23.md`, Test A2/A3/A4 FAIL):** P078f (`d35f462`) armed `core.hooksPath=hooks` but neither runtime's adapter `plan()` ever rendered `hooks/pre-commit`/`hooks/pre-push` to disk — the backstop "looked armed" (Test A1 PASS) but Git pointed at an empty dir, so a real `.env` commit (A3) and a real code-on-default commit (A4) both went through unblocked. False-security, worse than no backstop.
- **Route (Worker CHALLENGE Turn 1, Anchor #4 corrected the phiếu's `new.rs`-copy assumption):** `sos install` has NO `SOS_KIT_DIR` — it runs standalone in an arbitrary target repo, unlike `new`/`adopt`/`sync`. Fix uses **compile-time `include_str!`** of `hooks/pre-commit` + `hooks/pre-push` baked into the `sos-install` crate (`crates/sos-install/src/engine.rs`, `EMBEDDED_PRE_COMMIT`/`EMBEDDED_PRE_PUSH`), rendered by a new `render_embedded_hooks()` inside `engine::apply()`'s success path, immediately BEFORE `arm_git_hooks()`. Symmetric for both runtimes by construction (single engine call-site, no per-runtime branch). Non-clobber: a pre-existing hook file with DIFFERENT content is left untouched (warn only), extending F09's non-clobber philosophy to the render step.
- **Harden `arm_git_hooks()` — never arm-empty (Constraint 2):** new `hooks_present_and_executable()` check runs right before `git config --local core.hooksPath hooks` — if either hook script is absent/non-executable, arming is refused (`Err`, loud stderr), `core.hooksPath` is left unset. `arm_git_hooks` made `pub` so this guard is directly testable in isolation from the render step.
- **Tests (`crates/sos-install/tests/install.rs`, 5 new, fresh-no-seed fixtures — fixes P078f's own oracle gap where all 5 hook-arming tests seeded hook content into the synthetic `Plan`):** render-fresh (byte-match kit source + chmod + hooksPath armed AFTER render), **real end-to-end `.env` block** (genuine `git commit` runs the armed `hooks/pre-commit`, which runs the real `scripts/block-env-commit.sh` seeded into the fixture — blocked; clean commit passes as negative control), refuse-when-absent (`arm_git_hooks` called with no rendered hooks → `Err`, hooksPath stays unset), render non-clobber (pre-existing customized `hooks/pre-commit` preserved verbatim). `cargo test -p sos-install`: 15/15 pass (10 pre-existing + 5 new), `cargo check --workspace` clean.
- **Known asymmetry, pre-existing, out-of-scope:** `ClaudeAdapter::plan()` is a total stub (0 rendered assets) — `--runtime claude` renders nothing today regardless of this fix. Not touched here (P078g scope = engine-level hook arming only).
- Docs: `SECURITY.md`, `adapters/codex/CAPABILITY.md`, `docs/BACKLOG.md`.
- Full report: `docs/discoveries/P078g.md`.

**[P078h] FIX (SECURITY) — Codex adapter: actor-check advance-block closes P078e self-approve hole + patch-path normalize (2026-07-23):**
- **Gap #3 (P079 round-3 §B4, `docs/adapters/P079-ROUND3-FINDINGS-2026-07-23.md`):** P078e's actor-check stopped a marked actor (worker/architect subagent) from exercising the state-file-alone exemption, but still fell through to the ordinary version-match check below, which compares pre-patch state only. Starting from an already-approved `version=V3, approved_version=V3`, a worker with `.sos-state/worker-active` present could still write `version=V4, approved_version=V4` because the fall-through check saw `V3 == V3` and let it through — a self-approve **advance** hole, distinct from the create/first-approval hole P078e closed.
- **Fix:** a marked actor is now BLOCKed outright for ANY write touching `.sos-state/ticket-state.env` — create, update, or advance — with NO fall-through to version-match. Version-match is reached only when both markers are absent (main-thread). Main-thread exemption (P078e deadlock fix) unchanged and unregressed.
- **Gap #4 (P079 round-3 "Additional usability observation"):** the state-file-allow exemption compared the extracted patch path to the exact relative string `.sos-state/ticket-state.env`; a live Codex attempt that emitted a repo-root-absolute patch path false-blocked, prompt-shape dependent.
- **Fix:** normalize every extracted patch path (strip a literal `$REPO_ROOT/` prefix, reusing the guard's existing `REPO_ROOT` variable) BEFORE both the state-file-allow compare and the multi-path all-path check consume it — one normalize site, both consumers. Conservative: no symlink/`..` resolution — a path that doesn't cleanly strip to relative (still contains `..`, or was never under `REPO_ROOT`) is left as-is and can never win the exemption, so it falls through to fail-CLOSED.
- Tests (`crates/sos-adapter-codex/src/lib.rs`, 8 new): worker/architect advance-of-approved BLOCK (the gap #3 core oracle), worker create BLOCK, main-thread relative-path no-regress ALLOW, absolute-path normalize ALLOW on main-thread / BLOCK when marked, bundle-with-absolute-path no-regress BLOCK. **Negative-tests performed and reverted:** reverting the early-BLOCK flips the advance-of-approved test BLOCK→ALLOW; reverting the normalize step flips the absolute-path main-thread test ALLOW→false-BLOCK — both confirmed red, then restored.
- Oracle: `cargo check --workspace` clean, `sos-adapter-codex` 60/60 pass, ×20 = 0 flaky, dep-direction green (`sos-core` carries zero adapter import).
- **Additive, untouched:** `crates/sos-install/**`, `crates/sos-cli/**`, `sos-core`, `sos-adapter-claude`, d2a multi-path guard core, SubagentStart marker (d2b), `scripts/orchestrator-guard.sh` — only `crates/sos-adapter-codex/src/{templates.rs,lib.rs}` + docs touched. Independent of/parallel with **P078g** (install arm-hooks, gap #1).
- **Codex caveat unchanged:** strengthens guard LOGIC, does not change whether the guard fires inside a real Codex custom-role subagent spawn (`worker-active`/`architect-active` markers still absent on Codex 0.145.0, upstream `openai/codex#21753`). SOUND on Claude + manual-marker Codex repro; best-effort/inactive on a real Codex custom-role subagent spawn — Git backstop (P078f/P078g) + human-review-at-commit remain the real boundary there.
- Docs: `adapters/codex/CAPABILITY.md` §4 (P078h paragraph), `SECURITY.md` (P078h paragraph before the in-subagent-enforcement section). Trust-gate: `SECURITY.md` unicode-scanned ASCII-clean (`trust-gate.sh check` exit 0); no rebaseline (no `SURFACE_GLOBS` file touched — `crates/**` is Rust source, not a tracked runtime surface).
- Full report: `docs/discoveries/P078h.md`.

**[P078f] FIX (SECURITY) — `sos install` now arms Git hooks by default (`core.hooksPath` + F09 hijack-guard) (2026-07-23):**
- **Gap closed (P079 round-2 `docs/adapters/P079-ROUND2-FINDINGS-2026-07-23.md`, split off P078e per P085 decomposition heuristic):** `sos install` used to render `hooks/{pre-commit,pre-push}` to disk but never activate them — `git config --local core.hooksPath` stayed unset, so the tracked Git boundary never fired unless an adopter separately ran `scripts/install-hooks.sh`. This rendered the "honest-MISSING" story in `adapters/codex/CAPABILITY.md` (Git backstop as the real enforcement layer on Codex) untrue in practice: the declared backstop was off by default.
- **Fix (`crates/sos-install/src/engine.rs`, new `arm_git_hooks()` + helpers):** engine-native port (NOT a shell-out to `install-hooks.sh` — P059/P072 Windows-portability precedent) of the script's 4 steps, run once on the success path of `apply()`, after the manifest is committed: (1) `chmod +x hooks/pre-commit`/`hooks/pre-push` (`#[cfg(unix)]`, no-op on Windows); (2) F09 hijack-guard — if `core.hooksPath` is already set to something other than `hooks`, TTY prompts `[y/N]` (default N), non-TTY **ABORTs** (fail-closed, never silently clobbers an adopter's own hook chain); (3) `git config --local core.hooksPath hooks`; (4) rename (never delete) any stale `.git/hooks/{pre-commit,pre-push}` to `*.pre-hookspath.bak`. Non-git-repo target → warn-skip, install still succeeds. Idempotent: a prior run's own `hooks` value never re-trips the guard.
- **Symmetric by construction:** `crates/sos-cli/src/commands/install.rs`'s `run_adapter()` is the one shared call-site for both `--runtime claude` and `--runtime codex` → both paths through the same `engine::apply()`, no per-runtime branch needed (Task 3 = confirm + comment only).
- Tests (`crates/sos-install/tests/install.rs`, 5 new, real `git init` temp-repo fixtures): hooksPath set + chmod, stale-hook renamed to `.bak` with content preserved, non-clobber abort on foreign hooksPath (non-interactive), non-git-dir warn-skip success, idempotent second run. `cargo test --workspace` green (11/11 in `sos-install`'s `install.rs`, full workspace unaffected).
- Docs: `SECURITY.md` (install-time hook-arming boundary), `adapters/codex/CAPABILITY.md` (Git backstop now armed-by-default, not "off after install"), `docs/plans/P078d-decomposition.md` (P078f marked DONE), `docs/BACKLOG.md`.
- Full report: `docs/discoveries/P078f.md`.

**[P078e Task 1] FIX (SECURITY) — Codex adapter: approval-transition deadlock closed + self-approve actor-check (2026-07-23):**
- **Gap #1 (P079 round-2, `docs/adapters/P079-ROUND2-FINDINGS-2026-07-23.md`):** bootstrap creates state `version=V1, approved_version=empty`; the first legitimate approval write (`version=V2, approved_version=V2`) was BLOCKed by the version-match check because the state file already existed — circular deadlock (approving requires the write, the write requires already-approved).
- **Fix:** `scripts/codex/approval-gate.sh`'s state-file-alone exemption (P078d2a #5, previously create-when-missing only) now also covers **update** — any patch touching ONLY `.sos-state/ticket-state.env` is allowed through whether the file is being created or updated.
- **Actor-check (NEW, the security-critical half of this fix):** the exemption fires ONLY when neither `.sos-state/worker-active` nor `.sos-state/architect-active` is present (mirrors `orchestrator-guard.sh`'s `[ -f ".sos-state/worker-active" ] && continue` pattern) — so only the owner/orchestrator main-thread can exercise it. A worker/architect subagent turn gets no special treatment and still falls through to the ordinary version-match BLOCK, preventing the agent under review from self-approving its own ticket version. The pre-existing all-path check (P078d2a #6) is unchanged — a patch bundling `ticket-state.env` with a second path still BLOCKs.
- **Codex caveat (honest, not SOUND on Codex):** the actor-check depends on the same markers that P078d2b/gap-#6 documents as unreliable inside custom-role Codex subagents (`openai/codex#21753`). Correct+complete on Claude; defense-in-depth only on Codex — real backstop stays human-review-at-the-git-commit boundary. Gap #2 (`sos install` arming Git hooks by default) split out as **P078f** per the phiếu's own pre-declared decomposition threshold (security-arming in `install-hooks.sh` beyond a bare `git config` call) — not yet shipped.
- Tests (`crates/sos-adapter-codex/src/lib.rs`): round-2 repro ALLOW (V1→V2, no marker), self-approve BLOCK ×2 (`worker-active` set, `architect-active` set), pre-approval code-edit regression BLOCK, multi-path bundle BLOCK preserved even with no marker. **Negative-test performed and reverted:** commenting out the actor-check flips the self-approve test from BLOCK(exit 2) to ALLOW(exit 0) — confirmed the check actually bites, then restored.
- Oracle: `cargo build/test --workspace` green, `sos-adapter-codex` 53/53 pass, ×20 = 0 flaky, dep-direction green.
- **Additive, untouched:** P078d2a multi-path all-path guard logic, P078d1 3 startup render-fns, `crates/sos-install/src/engine.rs`, `crates/sos-cli/src/commands/install.rs`, `sos-core`/`sos-adapter-claude` — zero diff (`git diff --stat` = only `templates.rs` + `lib.rs` test module for this crate).
- Docs: `adapters/codex/CAPABILITY.md` §4, `SECURITY.md` "Codex adapter enforcement" section — both updated with actor-check + Codex caveat. Trust-gate: SECURITY.md is unicode-scanned (ASCII clean) but not in the auto-exec surface baseline diff (no `SURFACE_GLOBS` file touched) — no rebaseline needed.
- Full report: `docs/discoveries/P078e.md`.

**[P085] docs: codify phiếu-decomposition heuristic in ORCHESTRATION.md — 5 split signals (incompatible oracles / external-input blocker / security-surface / delivery clarity / lane budget) + keep-whole caveat. Guidance not gate (§0.1); lane budget stays the only mechanical gate. Pointer mirrored in agents/orchestrator.md.**

**[P078d2b] DOCS + DECLARATION (HONESTY) — Codex adapter: in-subagent role-envelope enforcement declared MISSING (upstream openai/codex#21753); SubagentStart-marker deprecated best-effort; P078d DONE (2026-07-22):**
- **Root cause (probe-confirmed, `docs/adapters/SUBAGENT-HOOK-PROBE-2026-07-22.md`):** Codex 0.145.0 does NOT dispatch `SubagentStart`/`SubagentStop` for CUSTOM-role spawns (`architect`/`worker`/`advisory-watch`/`boundary-check`) — only `agent_type="default"` fires. Session metadata carries `agent_role`, but the hook never runs. Upstream regression/gap: `github.com/openai/codex/issues/21753`. Dogfood-confirmed (`docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md` #4): a spawned architect's forbidden `apply_patch` against Rust `src/` succeeded IN-SUBAGENT; the `architect-active` marker was never created.
- **`CodexAdapter::verify()` (`crates/sos-adapter-codex/src/lib.rs`):** added Finding #6 — "in-subagent role-envelope enforcement (architect/worker)" — `FindingStatus::Missing`, cites `#21753` and both probe/dogfood evidence files. 5 → 6 Findings; count-oracle test renamed/updated (`verify_reports_exactly_six_gaps_none_sound`) plus a new dedicated assertion (`verify_declares_in_subagent_enforcement_missing`). Findings #1/#3/#5 (PARTIAL) NOT downgraded — 1 clause added to each clarifying "enforced on the MAIN THREAD only, NOT inside spawned custom-role subagents".
- **SubagentStart/Stop marker (`templates.rs` `hooks_json()`) — GIỮ (kept), deprecated best-effort:** render output byte-identical (Rust-comment-only change, zero JSON bytes touched — `hooks_json_is_valid_json_with_expected_events` + `hooks_json_top_level_keys_are_description_and_hooks_only` still pass unmodified). A loud comment above the raw-string literal documents that the marker fires for DEFAULT subagents only and is not relied upon for any capability claim; retained (not removed) so it self-activates if upstream `#21753` is ever fixed.
- **`AGENTS.md` render (`agents_md()`):** one new orchestrator-facing bullet — in-subagent guards do not enforce on Codex; the orchestrator (main thread) owns boundary review of subagent-authored diffs before merge; Git pre-commit/pre-push is the real backstop; optional best-effort main-thread `touch`/`rm` of the marker around delegation.
- **`adapters/codex/CAPABILITY.md`:** new §6 "In-subagent role-envelope enforcement (custom-role subagents) — MISSING" — Claude-vs-Codex containment-layer table + the 3 real backstops (main-thread `PreToolUse`, guards dogfood-confirmed to fire; universal Git pre-commit/pre-push, agent-agnostic; `AGENTS.md` guidance, prose-only) + `#21753` citation.
- **`SECURITY.md`:** new threat-model paragraph in the existing "Codex adapter enforcement" section — "do not trust an architect/worker subagent to police its own envelope on Codex; trust the Git gate" — same 3-backstop declaration, cites `verify()` Finding #6 + `CAPABILITY.md` §6.
- **`adapters/codex/MAPPING.md`:** "5 `Finding`s" → "6 `Finding`s", notes #6 = Missing.
- **NOT a behavioral fix:** `openai/codex#21753` is upstream, out of scope — no hack/workaround of custom-role hook dispatch attempted.
- **Additive verified:** `git diff` on d2a guards (#5/#6 multi-path bypass fix + approval bootstrap exemption), `crates/sos-install/src/engine.rs`, `crates/sos-core`, `bin/sos.sh`, `install.sh` = empty. `cargo build/test --workspace` green (48/48 `sos-adapter-codex` tests, incl. all d1/d2a regressions), ×20 = 0 flaky, dep-direction guard green (0 hits in `sos-core`).
- **P078d = DONE (d1 startup-schema + d2a multi-path-guard + d2b MISSING-declaration).** Codex adapter build is honest-complete; remaining P078 work is behavioral re-dogfood (P079 round 2, owner-run) and P085 decompose-heuristic.
- Full report: `docs/discoveries/P078d2b.md`.

**[P078d2a] FIX (SECURITY) — Codex adapter: multi-path guard bypass hole CLOSED + approval bootstrap deadlock + spawn caveat (2026-07-22):**
- **#6 multi-path bypass (SECURITY HOLE, CLOSED):** all 4 guards with an allow-list (`architect-guard.sh`, `orchestrator-guard.sh`, `block-env-edit.sh`, `approval-gate.sh`, `crates/sos-adapter-codex/src/templates.rs`) used to extract only the FIRST `*** (Add|Update|Delete|Move) File: <path>` marker from a multi-file `apply_patch` (`| head -n1`). A patch with an allowed path FIRST and a forbidden path (`.env`, `src/**`, `.sos-state/ticket-state.env`) SECOND would exit ALLOW on the first match, never inspecting the rest. Fix: extract EVERY path (`grep -oE` already emits one line per match; removing `head -n1` is sufficient), loop over all of them, **BLOCK if ANY path violates** the guard's rule (was "first-path-exempt → allow-all", now "all-path-exempt or BLOCK"). Loop implemented as a herestring `while read` (bash-3.2-compatible population, macOS ships bash 3.2 — no `readarray`/`mapfile`) into an array, then a plain `for` over it — deliberately NOT `... | while read; do ...; exit 2; done`, which would run in a piped subshell and silently swallow the `exit 2`, keeping the hole open while still printing "BLOCKED" to stderr.
- **#5 approval bootstrap deadlock (fixed, coupled hard with #6):** `approval-gate.sh` used to BLOCK every non-ticket `apply_patch` when `.sos-state/ticket-state.env` was missing — including the patch that would create that very file, a chicken-egg deadlock at fresh install. Fix, two layers: (a) a narrow self-bootstrap exemption — allow ONLY when the state file is missing AND the patch touches `.sos-state/ticket-state.env` and NO other path; (b) a rendered skeleton state file (`STATE_SKELETON` Asset, `.sos-state/ticket-state.env`, empty `version=`/`approved_version=`) emitted at install time so most fresh installs never even reach the missing-file branch. The skeleton's non-clobber safety reuses the ALREADY-generic `sos-install::engine` checksum/manifest `Decision::Conflict` logic — zero engine code changed. **This exemption is safe ONLY because #6's all-path check lands in the same patch:** a patch bundling `ticket-state.env` with a second path (`.env`, `src/**`) is caught by #6 and falls through to fail-CLOSED BLOCK, not the bootstrap ALLOW.
- **#7 spawn caveat:** `AGENTS.md` orchestrator guidance now documents the observed P079 first-spawn failure — a full-history forked agent inherits the parent's `agent_type`; spawn `architect`/`worker`/`advisory-watch`/`boundary-check` by omitting `agent_type` or without a full-history fork.
- `all_assets()` extends 17 → 18 (`STATE_SKELETON`, `.sos-state/ticket-state.env`).
- Tests (`crates/sos-adapter-codex/src/lib.rs`): 5 new multi-path bypass BLOCK assertions (one per allow-list guard, ticket-allowed-path-first + forbidden-path-second), 1 multi-path no-regress ALLOW (all paths ticket-exempt), 2 bootstrap tests (state-file-alone → ALLOW; state-file + second path → BLOCK). **Negative-test performed and reverted**: re-adding `| head -n1` at all 4 sites flips exactly these 6 tests from BLOCK(exit 2) to ALLOW(exit 0) — confirmed the fix (and the bash-subshell-safe loop construction) actually closes the hole, not a silent no-op; reverted back to the fix immediately after.
- Oracle: `cargo build/test --workspace` green, `sos-adapter-codex` 47/47 pass, ×20 = 0 flaky, dep-direction green, `bash -n` clean on all 5 guards.
- **Additive, untouched:** `#4` SubagentStart/Stop marker lifecycle (`templates.rs:315-317` hooks wiring) — deferred to **P078d2b** pending a live SubagentStart probe. `sos-install`/`sos-core`/`sos-adapter-claude`/`bin/sos.sh`/`install.sh` — zero diff. `docs-gate`/`dep-direction` checked clean.
- Full report: `docs/discoveries/P078d2a.md`.

**[P078d1] FIX — Codex adapter startup-schema fixes: 3 STARTUP-BLOCKERS from P079 live-dogfood (2026-07-22):**
- `crates/sos-adapter-codex/src/templates.rs`, 3 content-fn format fixes, all confirmed against real Codex 0.145.0 error messages (`docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md`):
  - `config_toml()` — `sandbox_mode`/`approval_policy` now emit BEFORE the first table header (`[mcp_servers.doctor]`), not merely before `[agents]` (Worker CHALLENGE caught that the original fix-wording would still nest under `[mcp_servers.doctor]`). Fixes `invalid type string "workspace-write", expected struct AgentRoleToml`.
  - `rules_exec_policy()` — `pattern` now a token LIST (`pattern = ["git", "push", "--force"]`), was a bare string. Fixes `pattern doesn't match, expected list, actual string`.
  - `hooks_json()` — dropped unsupported top-level `_provenance`/`_partial_note` fields, folded into `description` (Codex hooks schema only accepts `description`/`hooks`). Fixes `unknown field _provenance, expected description or hooks`.
- 3 new schema-shape tests added (`crates/sos-adapter-codex/src/lib.rs`) using real parsers (`toml`, `serde_json`) for config.toml/hooks.json, and a documented structural-string oracle for `.rules` (Starlark syntax — `toml`/`serde_json` cannot parse it; adding a Starlark parser crate for a test-only need was rejected as Tầng-1 overkill). All 3 negative-tested (revert fix → test FAILS, restore → PASS).
- **Honest limit (b2/b3-gap lesson):** the prior oracle (generic valid-TOML/valid-JSON) PASSED while these 3 blockers were live — schema-shape assert is still a hand-coded approximation of Codex 0.145.0's real deserializer, not a substitute for a live-Codex run.
- `cargo build/test --workspace` green, `sos-adapter-codex` lib tests ×20 = 0 flaky, dep-direction green. Live render re-smoke (`sos install --runtime codex`) confirms all 3 fixes in the actual written files.
- Additive: only `templates.rs` (3 content-fn) + test module. Enforcement content-fns (marker lifecycle, approval-gate, guard path-parsing) untouched — deferred to **P078d2**. Full report: `docs/discoveries/P078d1.md`.

**[P078c] FIX — install: render-before-toolgate reorder, unblock P079 Codex dogfood (2026-07-22):**
- `sos install --runtime <codex|claude>` used to run `resolve_tools()?` (OA-07 tool-manifest check) BEFORE `apply()`/`dry_run()`, with `?` hard-aborting on any required-tool drift/missing — so a machine with sister-tool drift (this dev machine, live: `doctor` 0.1.1 < pinned 0.1.3, `inv-gate` MISSING) could not render/write ANY adapter file, blocking P079's Codex dogfood over an unrelated concern.
- Default flow now: render (`apply()`/`dry_run()`) runs first, unconditionally; tool-check runs AFTER as a report, not a gate — required drift/missing prints a loud stderr WARNING (tool name + expected + found + `sos tools status` pointer, reusing `tools::describe_failure()`'s exact message, no new format invented) and exits **3** (distinct "installed, tools-not-ready"), never silently swallowed. Render/apply failure stays exit 1 (unchanged). All-tools-ready stays exit 0.
- New opt-in `--require-tools` flag restores the pre-P078c fail-closed order verbatim (tool-check before apply, abort exit 1, zero render) for CI/production callers that want the strongest OA-07 guarantee.
- `--dry-run` shows both the transaction plan AND the tool-drift warning (zero mutation either way); `--require-tools --dry-run` still gates before printing the plan (CI wants the same fail-closed signal even in dry-run).
- **Zero `sos-install` engine/tools change** — `engine::apply()`/`engine::dry_run()` never took tool status as input (already independently callable, per the engine's own doc comment) and `tools::check_tools()`/`tools::required_drift()`/`tools::describe_failure()` were already `pub` and non-`Result` — `install.rs` only changed WHICH functions it calls and WHEN, no new engine/tools code.
- Two near-duplicate functions (`run_claude()`/`run_codex()`) collapsed into one shared `run_adapter(&dyn Adapter, ...)` — symmetric claude+codex behavior is now structural (one code path), not maintained-by-hand.
- Stale comment fixed: `install.rs` still claimed `CodexAdapter.plan()` was "a b1 stub" — false since P078a/b shipped the real 17-artifact render (`plan_renders_seventeen_artifacts` test).
- Oracle: `cargo build/test --workspace` green, ×20 = 0 flaky, clippy 0 new warnings. Live smoke on this drift machine: `install --runtime codex` → 17 real files written (`AGENTS.md`, `.codex/**`, `scripts/codex/**`, `.agents/skills/**`) + loud drift warning + exit 3; `--require-tools` on the same machine → exit 1, zero files written; `--runtime claude` symmetric; `sos tools status` regression-verified still exit 1 on drift, unchanged. Additive: `bin/sos.sh`/`install.sh`/`crates/sos-install/` diffs stay empty.
- OA-07 preserved, not weakened: drift is always surfaced loud and non-zero; only the "block render" behavior changed to "render + report" for the default path, with `--require-tools` as the opt-in full fail-closed escape hatch.
- **P079 Codex dogfood is now unblocked** — a drifted local tool version no longer prevents adapter files from being written.

