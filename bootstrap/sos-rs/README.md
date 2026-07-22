# sos — Rust port (skeleton)

Phase 2 of the `sos` 0→1 bootstrap tool. Bash MVP at `bin/sos.sh` is the canonical executable today.

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
├── sos-install          # install framework skeleton (transaction/dry-run/
│                       # rollback/manifest) — logic lands in P077d.
├── sos-adapter-claude   # Claude Code adapter skeleton (detect/plan/render/
│                       # verify/uninstall) — logic lands in P077d. Deps: sos-core.
└── sos-hooks            # hooks framework skeleton — logic lands in P077d.
```

Dependency-direction gate (Tầng 1 — enforced two ways):
- **Compiler graph** — `sos-core/Cargo.toml` declares zero adapter/install/hooks/cli dep, so `use sos_adapter_*` etc. in core is a compile error.
- **Guard test** — `crates/sos-core/tests/dep_direction.rs` scans `sos-core/src/**` for forbidden tokens, catches regression the compiler alone would miss (e.g. a stray dep added to Cargo.toml).

**Deviation from target (`docs/PORTABILITY_ARCHITECTURE.md` lines 24-38)** — tracked, not yet fixed:
1. Workspace root stays at `bootstrap/sos-rs/` (target: repo-root) — relocation is P077e.
2. `sos-adapter-codex` not created (target lists it) — that's P078, out of scope here.
3. `sos-install` / `sos-adapter-claude` / `sos-hooks` are empty skeletons — real logic lands in P077d.

## Build

```bash
cd bootstrap/sos-rs
cargo build --workspace --release
# binary: target/release/sos (produced by the sos-cli crate)
cargo install --path crates/sos-cli
# now `sos` is on PATH
```

## Usage

Identical to bash MVP — see `cat ../../bin/sos.sh` or `sos help`.

## Why both Rust and bash

- **Bash MVP** ships immediately, easy to iterate on while the design churns.
- **Rust port** matches DNA of `ship`/`guard`/`vps`/`docs-gate` (4 sister tools), gets cargo-installable, faster startup, type-safe state machine.
- **Ownership (updated P077a):** the Rust workspace is being lifted INTO `sos-kit` itself (`docs/PORTABILITY_ARCHITECTURE.md`, `docs/plans/P077-decomposition.md`) — it does NOT move to a separate repo. `bin/sos.sh` stays canonical through P077a–P077d (Rust is developed alongside it, verified against a frozen Bash golden oracle — see `crates/sos-cli/tests/README.md`). Only P077e (the decomposition's final, founder-confirmed sub-phiếu) cuts over the Rust binary to canonical and updates the repo's "not a runtime binary source" contract in root `CLAUDE.md`.

## Command parity status (P077c)

`bin/sos.sh` is canonical; `crates/sos-cli/tests/parity.rs` diffs the Rust binary against a
frozen oracle (`tests/golden/*.golden`, see `tests/golden/capture.sh` + `tests/README.md`). A
command enters `PARITY_ENFORCED` (hard-fail on mismatch) once its Rust impl is proven against
that oracle. **`sync`/`new` are Bash-PARITY oracles** (Rust proven bug-for-bug identical to
Bash). **`map`/`adopt` became CORRECTNESS oracles as of P077c5** (Rust intentionally DIVERGES
from Bash to fix OA-02 — `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md:93-131`; `bin/sos.sh`
stays unchanged/buggy by design, Rust-only fix, cutover to Bash deferred to P077e). See
`tests/README.md` "correctness oracle vs parity oracle" section.

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
