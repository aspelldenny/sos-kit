# sos — canonical Rust workspace (post-P077e cutover)

The `sos` 0→1 bootstrap tool. **Canonical since P077e**: this workspace IS the runtime source
for the 6 heavy `sos` subcommands (`new`/`adopt`/`sync`/`map`/`install`/`tools`) — `bin/sos.sh`
dispatches (`exec`) to the built binary here for those 6, and keeps only the 7 Claude-flavored
guidance subcommands (`init`/`blueprint`/`contract`/`apply`/`recipe`/`launch`/`status`) in Bash
until P078 renders them per-runtime. This workspace does **not** extract to a separate repo —
it stays part of sos-kit, rooted at the repo root (`Cargo.toml` + `crates/`), relocated from
the transitional `bootstrap/sos-rs/` path in P077f (pure move, no logic change).

## Status

**Skeleton.** Compiles, all subcommands wired. Deterministic logic implemented (state.toml mgmt, spec_hash compute, launch checklist parser). LLM-driven phases (init, blueprint, apply, forge) print instructions identical to bash MVP — they delegate to Claude Code skills (`/init`, `/apply`, `/forge`).

## Module layout (P077b)

Carved into a virtual workspace, 5 crates under `crates/`, one-way dependency
direction (adapter → core; core imports nothing sos-*):

```
crates/
├── sos-core            # semantic: state.toml mgmt, spec_hash, config schema.
│                       # ZERO dep on any adapter/install/hooks/cli crate.
├── sos-cli              # composition root — binary `sos`. Deps: sos-core +
│                       # sos-install + sos-adapter-claude + sos-hooks.
├── sos-install          # install ENGINE (transaction/dry-run/non-clobber/
│                       # rollback/manifest) — LIVE, P077d2. `tools.rs`
│                       # (tool-manifest pin/drift-check core) — LIVE, P077d3.
│                       # Deps: sos-core.
├── sos-adapter-claude   # Claude Code adapter skeleton (detect/plan/render/
│                       # verify/uninstall) — logic lands in P077d. Deps: sos-core.
├── sos-adapter-codex    # Codex CLI adapter — foundation LIVE (P078b1): detect()/
│                       # verify() real, render() deferred b2/b3. Deps: sos-core.
└── sos-hooks            # hooks framework skeleton — logic lands in P077d.
```

Dependency-direction gate (Tầng 1 — enforced two ways):
- **Compiler graph** — `sos-core/Cargo.toml` declares zero adapter/install/hooks/cli dep, so `use sos_adapter_*` etc. in core is a compile error.
- **Guard test** — `crates/sos-core/tests/dep_direction.rs` scans `sos-core/src/**` for forbidden tokens, catches regression the compiler alone would miss (e.g. a stray dep added to Cargo.toml).

**Deviation from target (`docs/PORTABILITY_ARCHITECTURE.md` lines 24-38)** — status:
1. ~~Workspace root stays at `bootstrap/sos-rs/` (target: repo-root)~~ — **RESOLVED P077f**: workspace now lives at repo-root, layout matches target.
2. ~~`sos-adapter-codex` not created~~ — **RESOLVED P078b1**: crate created, `detect()`/`verify()` live (foundation subsection below); real artifact rendering (`AGENTS.md`/`.codex/**`) still deferred to P078b2/b3.
3. `sos-hooks` is still an empty skeleton (out of P077d2 scope); `sos-adapter-claude` implements the `Adapter` contract (stub bodies only — `plan()`/`render()` do not touch `.claude/**` for real, that's deferred past d2); `sos-install` now has a LIVE install engine (see below) — the remaining gap is real Claude asset rendering, not the engine.

### Adapter contract + managed-manifest schema (P077d1)

`sos-core` now carries the foundation abstraction that `sos-install` (d2) and every runtime adapter (`sos-adapter-claude`, and a future `sos-adapter-codex`, P078) build on — **zero install engine, zero filesystem mutation**:

- `crates/sos-core/src/adapter.rs` — the `Adapter` trait, 5 methods (`detect`/`plan`/`render`/`verify`/`uninstall`) per `docs/PORTABILITY_ARCHITECTURE.md:63-69`, plus runtime-neutral placeholder types (`Capabilities`, `Plan`/`ManagedOperation`, `Asset`/`Artifact`, `Findings`/`Finding`, `RemovalPlan`/`RemovalStep`). No host token (`.claude`, `CLAUDE_*`, Codex) — enforced by the dep-direction guard's forbidden-token scan.
- `crates/sos-core/src/manifest.rs` — `ManagedManifest` struct, 6 fields per `core/ASSETS.md:57-64` (`owner`, `source_version`, `source_identity`, `target_path`, `content_hash`, `rollback_ref: Option<String>`), serde-derived, TOML round-trip tested (`sos-core` already depends on `toml` for `state.toml`).
- `crates/sos-adapter-claude/src/lib.rs` — `ClaudeAdapter` implements `Adapter` with **stub bodies only** (no fs mutation, no real rendering) — proves the trait is implementable end-to-end while keeping the dependency-direction gate exercised (adapter → core, one-way).
- New tests: `crates/sos-core/tests/adapter_trait_shape.rs` (compile-level trait-bound proof), `crates/sos-core/src/manifest.rs` inline round-trip tests, `sos-adapter-claude`'s own trait-bound test. `crates/sos-core/tests/dep_direction.rs` re-verified green after the addition.
- Install engine (transaction plan / dry-run / non-clobber / rollback / apply) and tool-manifest pinning (OA-07) are **out of scope** here — P077d2/P077d3.

### Install engine (P077d2)

`sos-install::engine` (`crates/sos-install/src/engine.rs`) implements the install transaction, driven **thuần qua the `Adapter` trait** — zero host-specific (`.claude`/`CLAUDE_*`/Codex) knowledge, same dep-direction discipline as `sos-adapter-claude`:

- **Narrow d1 amendment** (Worker CHALLENGE Turn 1 O1.1, Architect ACCEPT Alt A): `ManagedOperation` (`sos-core/src/adapter.rs`) gained a `content: String` field — without it, `plan()` gave the engine paths+descriptions only, no bytes to write/hash/record. `Adapter`'s 5 methods and `ManagedManifest`'s 6 fields are unchanged.
- **Non-clobber discrimination is 4-way** (`Decision::Create`/`NoOp`/`Update`/`Conflict`): absent → CREATE; on-disk already == desired bytes → NoOp (true idempotence, zero mutation, unlike naively re-writing identical bytes every run); on-disk differs from desired but its hash matches the manifest's recorded `content_hash` (unmodified since last install) → UPDATE allowed; differs + no hash match / no record (user-customized) → CONFLICT, staged to `.sos-install-incoming/<path>` (mirrors `.sos-adopt-incoming`, `commands/adopt.rs`), original left byte-untouched.
- **Rollback** = snapshot-before-mutate: CREATE ops delete-on-fail, UPDATE ops restore the exact prior bytes captured before overwrite; `.sos-manifest.toml` is only written after the WHOLE transaction succeeds (never partially committed).
- **Manifest artifact:** `.sos-manifest.toml` at project root, `[[managed]]` array-of-tables of `ManagedManifest` entries.
- **Step 5 seam:** `resolve_tools()` is a stub no-op — `// SEAM P077d3` — P078/OA-07 fills it; d2 asserts nothing about its result.
- **Oracle:** `crates/sos-install/tests/install.rs`, a deterministic `MockAdapter` + 5 hard-fail correctness fixtures (additive / non-clobber×2 / rollback / idempotence / dry-run) — **not** a Bash-parity fixture (`sos install` has no Bash counterpart).
- **Wired command:** `sos install --runtime <auto|claude|codex> [--dry-run]` (`crates/sos-cli/src/commands/install.rs`) — `claude` constructs `ClaudeAdapter` (still d1-stub, so plan/render stay minimal/empty — real Claude asset rendering is deferred past d2); `codex` errors clearly ("not yet available, P078"). Fully additive alongside `install.sh`/`bin/sos.sh` (zero-touch, verified `git diff` empty).

### tool-manifest (P077d3, OA-07)

Fills d2's step-5 `resolve_tools()` seam (was a `Vec::new()` no-op stub) with a real pin+drift-check, verify-ONLY (no download/atomic-upgrade/auto-fetch of the external binaries — that's P081 future):

- **`tool-manifest.toml`** (kit ROOT, sibling of `.sos-trust-baseline` — committed config, NOT generated, NOT `.sos-manifest.toml`) — `[[tool]]` array-of-tables, one entry per sister tool: `name`/`version`/`required` + `[tool.asset]`/`[tool.checksum]` keyed by the 3 target-triple `install.sh` resolves (`aarch64-apple-darwin`/`x86_64-unknown-linux-gnu`/`x86_64-pc-windows-msvc`). 6 `required = true` (`doctor`/`claude-hooks`/`docs-gate`/`ship`/`advisory-inbox`/`inv-gate`, matches `install.sh:40` `BINARIES`) + 4 `required = false` (`guard`/`vps`/`doc-rotate`/`advisory-cron`, matches `install.sh:48` `OPTIONAL_BINARIES`). Versions filled from each sister tool's own source `Cargo.toml` (live-checked 2026-07-22), NOT `releases/latest` (the OA-07 bug). Checksums are honest `TODO-sha256-<tool>-P081` placeholders — no prebuilt asset has been hashed against this pin shape yet (E2); real checksum-fill is P081.
- **`crates/sos-install/src/tools.rs`** (NEW) — `ToolManifest`/`ToolPin` (serde TOML deserialize) + `check_tools()` (resolves each tool's installed `<name> --version` — E1-confirmed uniform `"<name> <x.y.z>"` format across all 9 live-tested sos-kit tools, no `v` prefix — against the pin; hand-rolled dotted-tuple version compare, no `semver` crate anywhere in the workspace) + `Verdict` (`Ok`/`Newer`/`Drift`/`Missing`/`Unparseable`) + `required_drift()`/`gate_required()` (fail-closed core, shared by BOTH surfaces below) + `validate_manifest()` (schema-integrity: every checksum cell must be a real-looking 64-hex or a `TODO` placeholder — catches a corrupted manifest even though d3 can't verify real binary bytes). Embedded at compile time via `include_str!` (not an env-var/path lookup — the compiled `sos` binary runs inside arbitrary target projects that don't necessarily have a sos-kit checkout, so the pin must travel WITH the binary).
- **`sos tools status`** (NEW, `crates/sos-cli/src/commands/tools.rs`) — table report (tool/required/expected/installed/verdict), exit 1 if any REQUIRED tool is Drift/Missing/Unparseable, exit 0 otherwise (optional drift = warn line only, never flips exit).
- **Step 5 filled** (`sos_install::engine::resolve_tools()`) — now reads the embedded manifest and calls `gate_required()` at the SAME call position d2 wired (`commands/install.rs`, before `decide_targets()`/`apply()`); a required-tool failure aborts the install with `?` BEFORE any filesystem mutation happens — trivially satisfies "fail rõ + rollback" (nothing was written yet).
- **No `sos doctor` subcommand exists** in `sos-cli` today — every `doctor` reference is `Command::new("doctor")` shelling to the EXTERNAL binary (`adopt.rs`/`new.rs` `verify-setup`). Fail-clear therefore surfaces via `sos tools status` + the step-5 install gate only — no subcommand was added that could collide with the real `doctor` binary's identity.
- **Oracle:** `crates/sos-install/tests/tools.rs` — 3 fixture groups (manifest-pin-verify incl. sabotage-checksum-fails-loud, status-drift, doctor/step-5 fail-clear), fake-PATH stub-script harness (child-process-scoped `PATH` override, not global `env::set_var`, so parallel `#[test]` threads never race) — entirely synthetic, zero dependency on any real sister tool being installed.
- **Live smoke evidence** (this dev machine, unmodified): `sos tools status` / `sos install --dry-run` both reproduce OA-07's exact finding — `doctor 0.1.1` vs pinned `0.1.3`, `inv-gate` MISSING, exit 1 — proving the gate genuinely fires on real drift, not just synthetic fixtures.

### Codex adapter foundation (P078b1)

`crates/sos-adapter-codex` (NEW, deps CHỈ `sos-core` — adapter→core one-way) — `CodexAdapter` implements the core `Adapter` trait. Decomposed from a single "Codex adapter" phiếu into 3 (foundation / declarative render / enforcement) — repo precedent (P077 a–f, P077d1–d3) favors decompose for large Rust builds with different risk profiles:

- **`detect()`** — STRUCTURAL: static Codex CLI 0.145.0 facts (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:32` — hooks/multi_agent stable, sandbox_mode read-only available, `apply_patch` model, shell-based reads) plus a best-effort `codex --version` probe. The probe is **fail-safe**: `codex` absent on PATH never panics or errors — `detect()` falls back to the static-only `Capabilities`, because the b1 structural oracle runs on a machine WITHOUT Codex installed (behavioral verification with a real Codex CLI is P079).
- **`verify()`** — the machine surface of the **PARTIAL-declaration mechanism**: returns `Findings` with exactly 5 items, each carrying a `FindingStatus` (`Sound`/`Partial`/`Missing`, new — see below) — never `Sound` for a known gap (`core/ROLES.md` separation-invariant #5: capability absence must be explicit, an integration cannot simulate success with prose). The 5 gaps: per-role built-in tool allowlist (PARTIAL — enforced via PreToolUse-hook + prose instead of Claude's structural removal), repo-distributed slash commands (MISSING — replacement = repo skill), skill-level `allowed-tools` (PARTIAL — not mechanical), native ticket-version approval (MISSING — build via persisted marker + guard, P078b3), architect Read/Glob path interception (PARTIAL — Codex reads via shell, restriction must inspect command text; enforcement additionally not unbypassable — retain Git/CI backstops). Human-readable frozen twin: `adapters/codex/CAPABILITY.md` (seeded from this function).
- **`plan()`/`render()`/`uninstall()`** — minimal-honest stubs (empty `Plan`/passthrough `Artifact`/empty `RemovalPlan`) — no artifact bytes are produced at b1; real rendering into `AGENTS.md`/`.codex/agents/*.toml`/`.agents/skills/*/SKILL.md`/`.codex/config.toml` is P078b2, the `.codex/hooks.json`/`.codex/rules/*.rules`/guard-script enforcement layer is P078b3.
- **`FindingStatus` enum (additive, `sos-core/src/adapter.rs`)** — `Finding` previously had no oracle-strength field at all (only `target_path`/`message`); this is a **pure addition**, vocab matches `core/POLICY.md` "Oracle-first claims" (SOUND/PARTIAL/MISSING). `ClaudeAdapter` needed **no compile-fix** — it never constructs a `Finding`, only `Findings::default()`.
- **Wired `sos-cli`** — `install --runtime codex` no longer bails "not yet available (P078)"; it constructs `CodexAdapter` and drives `sos_install::engine` exactly like the `claude` branch (engine takes `&Plan`/`project_root` only, zero `Adapter`-typed parameter — confirmed no engine change needed).
- **Declarative docs** — `adapters/codex/{README,MAPPING,CAPABILITY}.md` (NEW), mirroring `adapters/claude/`'s declarative-boundary pattern (P076).
- **Oracle: STRUCTURAL only** (Decision 5, `docs/ticket/P078b1-codex-adapter-foundation.md`) — `cargo build/test --workspace` green ×20 = 0 flaky; dep-direction guard green (new crate + enum introduce zero forbidden token into `sos-core/src/**`); `install --runtime codex --dry-run` reaches the identical engine code path as `--runtime claude` (the removed bail string is gone; full exit-0 is blocked on this dev machine only by an unrelated, pre-existing tool-manifest pin drift that identically affects `--runtime claude`, not a regression from this ticket). **Behavioral** verification (Codex CLI actually executing the rendered output correctly) is P079, out of scope here.

## Build

```bash
cd <repo-root>
cargo build --workspace --release
# binary: target/release/sos (produced by the sos-cli crate)
cargo install --path crates/sos-cli
# now `sos` is on PATH
```

## Usage

Identical to bash MVP — see `cat ../bin/sos.sh` or `sos help`.

## Why both Rust and bash

- **Bash MVP** ships immediately, easy to iterate on while the design churns.
- **Rust port** matches DNA of `ship`/`guard`/`vps`/`docs-gate` (4 sister tools), gets cargo-installable, faster startup, type-safe state machine.
- **Ownership (updated P077e — CUTOVER LIVE):** the Rust workspace lives INSIDE `sos-kit` itself (`docs/PORTABILITY_ARCHITECTURE.md`, `docs/plans/P077-decomposition.md`) and does NOT move to a separate repo. Through P077a–P077d, `bin/sos.sh`'s Bash implementation stayed canonical while Rust was developed alongside it, verified against a frozen Bash golden oracle (`crates/sos-cli/tests/README.md`). **As of P077e, this binary is canonical** for the 6 heavy subcommands (`new`/`adopt`/`sync`/`map`/`install`/`tools`) — `bin/sos.sh` execs it; the old Bash `sos_new`/`sos_adopt`/`sos_sync`/`sos_map` functions are retained DORMANT (rollback safety only, not called by the dispatcher). Root `CLAUDE.md`'s "not a runtime binary source" contract has been flipped to reflect this repo as the runtime monorepo for the `sos` binary.

## Command parity status (P077c, cutover-updated P077e)

This Rust binary is canonical (post-P077e); `crates/sos-cli/tests/parity.rs` diffs it against a
frozen oracle (`tests/golden/*.golden`, see `tests/golden/capture.sh` + `tests/README.md`). A
command enters `PARITY_ENFORCED` (hard-fail on mismatch) once its Rust impl is proven against
that oracle. **`sync`/`new` are Bash-PARITY oracles** (Rust proven bug-for-bug identical to the
now-dormant Bash). **`map`/`adopt` became CORRECTNESS oracles as of P077c5** (Rust intentionally
DIVERGES from the old Bash to fix OA-02 — `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md:93-131`;
the dormant Bash `sos_map`/`sos_adopt` stay unchanged/buggy by design — users now get the fix
live via the P077e dispatch cutover). See `tests/README.md` "correctness oracle vs parity oracle"
section.

| Command | Status |
|---|---|
| `map` | **Correctness (hard-fail, stdout + file)** — P077c1 shipped Bash-parity; **P077c5 flipped to CORRECTNESS and fixed OA-02**: (1) stack-aware generic source surface (Rust `src/**`, Python/Node/Go/Swift, monorepo-aware — was previously missing entirely), (2) static exclude-list (`templates/`, `phieu/`, `scripts/`, `hooks/`, `.claude/`) so kit-managed assets adopt just copied are never scanned as product surfaces, (3) 3-verdict `status` field (`PATH_VALID` / `COVERAGE_UNKNOWN` / `COVERAGE_REVIEWED`) replacing the single `draft_needs_review` string — fresh scans always emit `coverage_unknown`, never routing authority. Diffs both the 1-line stdout confirmation (`map.golden`) AND the real work-product it writes, `<target>/docs/AGENT_MAP.yaml` (`map.agent_map.golden`) — both now hand-authored/frozen against the CORRECTED Rust output, not Bash. 3 dedicated acceptance fixtures (`oa02_*` tests in `parity.rs`) hard-fail on regression. |
| `sync` | **Parity (hard-fail, stdout + file-tree)** — P077c2. Diffs the stdout report (`sync.golden`, re-froze against a synthetic self-contained fake-kit — no real sos-kit HEAD dependency) AND a sorted post-sync file-tree manifest (`sync.tree.golden`, new — `<verb> <relpath> <sha256>` for every ADDED/UPDATED/INCOMING path). Provenance oracle (`_blob_in_history`) replicated via `git` shell-out (no git2 dep). Traversal NOT sorted (matches Bash's unsorted `find` — see `tests/README.md`). |
| `new` | **Parity (hard-fail, stdout + tree-shape + gen-content)** — P077c3. Diffs 3 fixtures: `new.golden` (stdout, re-froze against a synthetic minimal fake-kit + `DOCTOR_BIN=/nonexistent/doctor` — the PREVIOUS golden had accidentally captured the CONNECTED verify-setup path, a host artifact not a design choice), `new.tree.golden` (sorted path-shape manifest, no content, excludes `.git/` — proves every copied+generated file/dir lands at the right path), `new.gen.golden` (content-hash manifest for GENERATED-authored files ONLY — copied kit assets are tree-only, never hashed, to avoid coupling this fixture to unrelated kit-content changes). Synthetic fake-kit is simpler than `sync`'s (plain directory tree, no git history needed — `new` only copies verbatim). |
| `adopt` | **Parity + one correctness exception (hard-fail, stdout + tree-shape + gen-content + preservation)** — P077c4 shipped Bash-parity for all 4 layers; **P077c5 flips ONLY the `docs/AGENT_MAP.yaml` line of the gen-content assert to correctness** (map-within-adopt re-invokes the now-fixed `sos map` binary — zero code change needed in `adopt.rs`, the exclude-list applies regardless of caller). stdout/tree-shape/preservation stay pure Bash-parity, unchanged. Diffs stdout (`adopt.golden`), `adopt.tree.golden` (sorted path-shape, excludes `.git*`, INCLUDES `.sos-adopt-incoming/**`), `adopt.gen.golden` (content-hash, GENERATED-authored only — `docs/AGENT_MAP.yaml`'s hash line hand-verified against corrected Rust, others still Bash-captured), and an in-test **preservation-assert** (NOT a golden — universal non-clobber property: every seeded pre-existing file's sha256 unchanged before/after, every `.sos-adopt-incoming/<path>` byte-matches its kit source). Non-clobber `added`/`conflicts` list order = raw traversal order (not sorted), same class as `sync`. |

## TODO before parity

- [ ] `sos init` — interactive 3-question wizard (currently delegates to /init skill)
- [ ] `sos blueprint` — interactive stack picker (currently lists recipes only)
- [ ] `sos apply <name>` — direct recipe execution without /apply skill (full Rust port of skill workflow)
- [ ] Cross-platform testing (Linux + macOS — Windows TBD)
- [ ] MCP server mode (like ship serve / guard serve / vps serve)
