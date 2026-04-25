# sos — Rust port (skeleton)

Phase 2 of the `sos` 0→1 bootstrap tool. Bash MVP at `bin/sos.sh` is the canonical executable today.

## Status

**Skeleton.** Compiles, all subcommands wired. Deterministic logic implemented (state.toml mgmt, spec_hash compute, launch checklist parser). LLM-driven phases (init, blueprint, apply, forge) print instructions identical to bash MVP — they delegate to Claude Code skills (`/init`, `/apply`, `/forge`).

## Build

```bash
cd bootstrap/sos-rs
cargo build --release
# binary: target/release/sos
cargo install --path .
# now `sos` is on PATH
```

## Usage

Identical to bash MVP — see `cat ../../bin/sos.sh` or `sos help`.

## Why both Rust and bash

- **Bash MVP** ships immediately, easy to iterate on while the design churns.
- **Rust port** matches DNA of `ship`/`guard`/`vps`/`docs-gate` (4 sister tools), gets cargo-installable, faster startup, type-safe state machine.
- When Rust port reaches feature parity + battle-tested, deprecate bash and move to its own repo `github.com/aspelldenny/sos`.

## TODO before parity

- [ ] `sos init` — interactive 3-question wizard (currently delegates to /init skill)
- [ ] `sos blueprint` — interactive stack picker (currently lists recipes only)
- [ ] `sos apply <name>` — direct recipe execution without /apply skill (full Rust port of skill workflow)
- [ ] Cross-platform testing (Linux + macOS — Windows TBD)
- [ ] MCP server mode (like ship serve / guard serve / vps serve)
