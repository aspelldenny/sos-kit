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
//
// P078c — render-before-toolgate reorder (unblock P079 Codex dogfood):
// tool-manifest drift (OA-07) used to HARD-BLOCK render via
// `resolve_tools()?` running before `apply()`/`dry_run()`. That conflated
// two concerns: "can we write adapter files" (render, tool-version
// independent) vs "are sister tools workflow-ready" (tool-check). Default
// flow now is render-first, tool-check-after (report loud, don't abort);
// `--require-tools` opt-in restores the old fail-closed order for
// CI/production. Zero `sos-install` engine/tools change was needed — both
// concerns were already independently callable (`engine::apply`/
// `engine::dry_run` never depended on tool state; `tools::check_tools` /
// `tools::required_drift` / `tools::describe_failure` were already `pub`
// and non-erroring) — see `docs/discoveries/P078c.md`.

use anyhow::Result;
use sos_adapter_claude::ClaudeAdapter;
use sos_adapter_codex::CodexAdapter;
use sos_core::adapter::Adapter;
use sos_install::engine::{self, Decision};
use sos_install::tools;
use std::path::Path;

const OWNER: &str = "sos-adapter-claude";
const CODEX_OWNER: &str = "sos-adapter-codex";
const SOURCE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit 3 — installed-but-tools-not-ready (P078c CHỐT, distinct from exit
/// 0 = ready and exit 1 = render/apply actually failed). Reuses the
/// `std::process::exit` pattern already established in this codebase
/// (`commands/tools.rs:51`, `commands/launch.rs:34`) — no `main.rs`
/// plumbing needed, `?`/`Result` handling elsewhere is untouched.
const EXIT_TOOLS_NOT_READY: i32 = 3;

pub fn run(runtime: &str, dry_run: bool, require_tools: bool) -> Result<()> {
    match runtime {
        "auto" | "claude" => run_claude(dry_run, require_tools),
        "codex" => run_codex(dry_run, require_tools),
        other => {
            anyhow::bail!(
                "unknown --runtime '{other}' — expected one of: auto, claude, codex \
                 (comma-separated multi-runtime not yet wired, P077d2 scope = single adapter)"
            );
        }
    }
}

fn run_claude(dry_run: bool, require_tools: bool) -> Result<()> {
    run_adapter(&ClaudeAdapter, OWNER, "claude", dry_run, require_tools)
}

// P078b1 — Codex foundation. Mirrors `run_claude` 1:1: the install ENGINE
// (`sos_install::engine`) is driven THUẦN qua the `Adapter` trait (takes
// `&Plan`/`project_root` only, zero `Adapter`-typed parameter) — no engine
// change was needed to wire Codex (Worker CHALLENGE Turn 1, anchor #12).
// `CodexAdapter.plan()` renders the real 17-artifact set as of P078a/b
// (AGENTS.md, 4x `.codex/agents/*.toml`, `.codex/config.toml`,
// `.codex/hooks.json`, `.codex/rules/exec-policy.rules`, skills, ...) — NOT
// a stub anymore (the previous comment here claiming "still a b1 stub" was
// stale post P078a/b; fixed in P078c, see `docs/discoveries/P078c.md`).
fn run_codex(dry_run: bool, require_tools: bool) -> Result<()> {
    run_adapter(&CodexAdapter, CODEX_OWNER, "codex", dry_run, require_tools)
}

/// Shared render + tool-report flow for both runtimes (P078c) — the two
/// adapters only differ in concrete type + owner string, so the reorder
/// logic lives in ONE place, which also trivially satisfies the "symmetric
/// claude+codex" constraint (a single shared path IS the symmetry, no
/// duplication needed — Task 2 note in the phiếu).
fn run_adapter(
    adapter: &dyn Adapter,
    owner: &str,
    runtime_label: &str,
    dry_run: bool,
    require_tools: bool,
) -> Result<()> {
    let capabilities = adapter.detect();
    let plan = adapter.plan(&capabilities);
    let project_root = Path::new(".");

    // `--require-tools` (opt-in fail-fast, Task 3): restores the PRE-P078c
    // behavior verbatim — tool-check BEFORE any render/write, `?`-propagate
    // a required-tool failure as a hard install error (exit 1 via anyhow,
    // nothing written yet, nothing to roll back). Dry-run + require-tools:
    // still gate (no mutation happens either way in dry-run, but a CI
    // caller asking for `--require-tools --dry-run` wants the same
    // fail-closed signal it would get from a real run — Tầng-2 CHỐT, see
    // Discovery).
    if require_tools {
        engine::resolve_tools()?;
    }

    if dry_run {
        let decisions = engine::dry_run(project_root, &plan)?;
        println!("sos install --runtime {runtime_label} --dry-run (ZERO mutation):");
        if decisions.is_empty() {
            println!("  (empty plan — this adapter's plan() produced no operations)");
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

        // Show-both (Task 5): dry-run always reports tool-drift too, even
        // though nothing was mutated. Skipped if `--require-tools` already
        // ran the fail-fast check above.
        if !require_tools {
            report_tool_drift(dry_run);
        }
        return Ok(());
    }

    // P078f — `engine::apply()` now also arms Git hooks (core.hooksPath +
    // F09 hijack-guard) on success. `run_adapter` is the ONE shared
    // call-site for both `run_claude`/`run_codex` (Worker CHALLENGE Turn 1
    // anchor #2) — arming lives in the engine's core-path, so it runs
    // identically for `--runtime claude` and `--runtime codex` with no
    // per-runtime branch needed here (Task 3 = confirm-only, no-op).
    let report = engine::apply(project_root, &plan, owner, SOURCE_VERSION)?;
    println!("sos install --runtime {runtime_label}:");
    println!("  created:   {}", report.created.len());
    println!("  updated:   {}", report.updated.len());
    println!("  no-op:     {}", report.noop.len());
    println!("  conflicts: {}", report.conflicts.len());
    if !report.conflicts.is_empty() {
        println!("  (conflicted targets staged under .sos-install-incoming/ — merge by hand)");
    }

    // Default flow (Task 1/2): render already happened above regardless of
    // tool state. Tool-check now runs as a REPORT, not a gate — required
    // drift/missing → loud WARNING + exit 3, never silently swallowed
    // (OA-07 intent preserved: "don't silent-run on stale tools", not
    // "block render"). Skipped if `--require-tools` already gated above
    // (if we got here with require_tools=true, tools were already OK).
    if !require_tools {
        report_tool_drift(false);
    }
    Ok(())
}

/// Resolve tool status WITHOUT aborting (capture, don't `?`-propagate —
/// Task 1 step 2). Reuses the exact same core (`tools::check_tools` /
/// `tools::required_drift` / `tools::describe_failure`) that
/// `engine::resolve_tools()` / `sos tools status` already use — no new
/// message format invented. On required-tool drift/missing: print a loud
/// WARNING to stderr + exit 3 (installed-but-tools-not-ready). If the
/// manifest itself can't be parsed, that's reported the same way (loud,
/// non-fatal to the already-completed render) rather than silently
/// skipped.
fn report_tool_drift(is_dry_run: bool) {
    let manifest = match tools::embedded_manifest_parsed() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("⚠️  tool-manifest.toml could not be parsed — skipping tool-readiness check: {e}");
            return;
        }
    };
    let statuses = tools::check_tools(&manifest, None);
    if !tools::required_drift(&statuses) {
        return;
    }

    let action = if is_dry_run { "would be installed" } else { "installed" };
    eprintln!(
        "⚠️  tools not workflow-ready — adapter files {action} but the following required \
         sister-tool(s) failed the tool-manifest.toml pin check:"
    );
    for s in statuses
        .iter()
        .filter(|s| s.required && !matches!(s.verdict, tools::Verdict::Ok | tools::Verdict::Newer))
    {
        eprintln!("  {}", tools::describe_failure(s));
    }
    eprintln!("  → run 'sos tools status' / update tools before running the workflow");

    std::process::exit(EXIT_TOOLS_NOT_READY);
}
