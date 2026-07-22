// sos — 0→1 bootstrap (Rust port — phase 2; bash MVP at bin/sos.sh)
//
// Status: skeleton. Subcommands wired but most delegate to interactive flows
// that require Claude Code skills (/init, /apply, /forge). Rust port focuses
// on deterministic mechanics: state.toml management + spec_hash compute +
// launch checklist parser. LLM-driven phases stay as instructions printed
// to stdout, identical to bash MVP.

use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "sos", version, about = "SOS Kit 0→1 bootstrap")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Phase 0 — vision capture (Chủ nhà). Delegates to /init skill.
    Init,
    /// Phase 1 — pick stack + recipes (Chủ nhà → Kiến trúc sư).
    Blueprint,
    /// Phase 2 — lock P000-genesis.md with spec_hash (Kiến trúc sư).
    Contract,
    /// Phase 3 — apply 1 recipe (Thợ). Use --all for full Genesis recipe list.
    Apply {
        /// Recipe path: <category>/<name>, or --all
        recipe: String,
    },
    /// Forge new recipe (Kiến trúc sư).
    Recipe {
        #[command(subcommand)]
        action: RecipeCmd,
    },
    /// Phase N+1 — launch gate (hard block on incomplete checklist).
    Launch {
        /// Skip specific checklist items (audited). Repeatable.
        #[arg(long, value_delimiter = ',')]
        skip: Vec<String>,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Show .sos/state.toml summary.
    Status,
    /// Scan an EXISTING repo → draft AGENT_MAP with REAL surfaces (bug-for-bug
    /// parity with `bin/sos.sh` map — see P077c1; OA-02 fix is P077c5).
    Map {
        /// Target directory to scan (defaults to ".").
        #[arg(default_value = ".")]
        target: String,
    },
    /// Re-sync sos-kit spine into an adopted repo (KIT-LAG cure, P067) —
    /// bug-for-bug parity with `bin/sos.sh` sos_sync (see P077c2).
    Sync {
        /// Adopted repo dir to sync spine files into.
        target: String,
    },
    /// Bootstrap a NEW repo from sos-kit golden (greenfield) — bug-for-bug
    /// parity with `bin/sos.sh` sos_new (see P077c3).
    New {
        /// Target directory to bootstrap into.
        target: String,
        #[arg(long)]
        stack: Option<String>,
        /// Declared but unused, bug-for-bug (bin/sos.sh:368 parses `--pilot` and
        /// never reads the variable again).
        #[arg(long)]
        pilot: bool,
        #[arg(long)]
        force: bool,
    },
    /// Retrofit sos-kit spine into an EXISTING repo (brownfield) — ADDITIVE +
    /// NON-CLOBBER, bug-for-bug parity with `bin/sos.sh` sos_adopt (P077c4).
    Adopt {
        /// Existing repo dir to retrofit.
        dir: String,
        /// Declared but unused, bug-for-bug (bin/sos.sh:615-621 parses
        /// `--stack` and never reads the variable again in sos_adopt).
        #[arg(long)]
        stack: Option<String>,
    },
    /// P077d2 — install engine: transaction plan / dry-run / non-clobber /
    /// rollback / apply. NEW command, additive alongside `install.sh`
    /// (zero-touch); no Bash counterpart.
    Install {
        /// Target runtime adapter: auto | claude | codex.
        #[arg(long, default_value = "auto")]
        runtime: String,
        /// Compute + print the transaction plan only — ZERO filesystem
        /// mutation.
        #[arg(long)]
        dry_run: bool,
    },
    /// P077d3 (OA-07) — tool-manifest pin/status. NEW command, additive
    /// alongside `install.sh`; no Bash counterpart.
    Tools {
        #[command(subcommand)]
        action: ToolsCmd,
    },
}

#[derive(Subcommand)]
enum ToolsCmd {
    /// Report expected-vs-installed sister-tool version per
    /// `tool-manifest.toml`. Exit 1 if any REQUIRED tool is
    /// missing/older/unparseable; optional drift = warn only, exit 0.
    Status,
}

#[derive(Subcommand)]
enum RecipeCmd {
    /// Forge new recipe — invokes /forge skill.
    New { name: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Init => commands::init::run(),
        Cmd::Blueprint => commands::blueprint::run(),
        Cmd::Contract => commands::contract::run(),
        Cmd::Apply { recipe } => commands::apply::run(&recipe),
        Cmd::Recipe { action } => match action {
            RecipeCmd::New { name } => commands::recipe::new(&name),
        },
        Cmd::Launch { skip, reason } => commands::launch::run(skip, reason),
        Cmd::Status => commands::status::run(),
        Cmd::Map { target } => commands::map::run(&target),
        Cmd::Sync { target } => commands::sync::run(&target),
        Cmd::New { target, stack, pilot, force } => {
            commands::new::run(&target, stack.as_deref(), pilot, force)
        }
        Cmd::Adopt { dir, stack } => commands::adopt::run(&dir, stack.as_deref()),
        Cmd::Install { runtime, dry_run } => commands::install::run(&runtime, dry_run),
        Cmd::Tools { action } => match action {
            ToolsCmd::Status => commands::tools::run(),
        },
    }
}
