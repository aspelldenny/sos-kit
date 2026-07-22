// commands/install.rs — P077d2: wire `sos install --runtime <r>` NEW
// command, additive alongside `install.sh` (bin/sos.sh + install.sh remain
// ZERO-touch — this is a brand-new command, not a port of an existing
// Bash flow; oracle = install correctness fixtures in `sos-install`, no
// Bash counterpart — see `docs/plans/P077d-decomposition.md:9`).
//
// This command is the COMPOSITION ROOT for the install engine: it
// constructs a concrete `Adapter` (currently only `ClaudeAdapter`, still
// stub per d1 — real Claude asset rendering is deferred, NOT part of d2)
// and feeds it to `sos_install::engine`, which is driven THUẦN qua the
// `Adapter` trait and holds zero host-specific knowledge.

use anyhow::{bail, Result};
use sos_adapter_claude::ClaudeAdapter;
use sos_core::adapter::Adapter;
use sos_install::engine::{self, Decision};
use std::path::Path;

const OWNER: &str = "sos-adapter-claude";
const SOURCE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run(runtime: &str, dry_run: bool) -> Result<()> {
    match runtime {
        "auto" | "claude" => run_claude(dry_run),
        "codex" => {
            bail!("codex adapter not yet available (P078) — use --runtime claude for now");
        }
        other => {
            bail!(
                "unknown --runtime '{other}' — expected one of: auto, claude, codex \
                 (comma-separated multi-runtime not yet wired, P077d2 scope = single adapter)"
            );
        }
    }
}

fn run_claude(dry_run: bool) -> Result<()> {
    let adapter = ClaudeAdapter;
    let capabilities = adapter.detect();
    let plan = adapter.plan(&capabilities);

    // Step 5 (P077d3, OA-07) — tool-manifest resolve+verify, called at the
    // right transaction position, BEFORE any filesystem mutation. `?`
    // propagates a required-tool failure as a hard install error (fail
    // rõ); nothing has been written yet at this point, so there is
    // nothing to roll back.
    engine::resolve_tools()?;

    let project_root = Path::new(".");

    if dry_run {
        let decisions = engine::dry_run(project_root, &plan)?;
        println!("sos install --runtime claude --dry-run (ZERO mutation):");
        if decisions.is_empty() {
            println!("  (empty plan — ClaudeAdapter.plan() is still a d1 stub; real render lands after d2)");
        }
        for d in &decisions {
            let verb = match d.decision {
                Decision::Create => "would-CREATE",
                Decision::Update => "would-UPDATE",
                Decision::NoOp => "no-op (unchanged)",
                Decision::Conflict => "would-CONFLICT (non-clobber, staged to .sos-install-incoming/)",
            };
            println!("  {verb}: {}", d.target_path);
        }
        return Ok(());
    }

    let report = engine::apply(project_root, &plan, OWNER, SOURCE_VERSION)?;
    println!("sos install --runtime claude:");
    println!("  created:   {}", report.created.len());
    println!("  updated:   {}", report.updated.len());
    println!("  no-op:     {}", report.noop.len());
    println!("  conflicts: {}", report.conflicts.len());
    if !report.conflicts.is_empty() {
        println!("  (conflicted targets staged under .sos-install-incoming/ — merge by hand)");
    }
    Ok(())
}
