//! sos — SOS Kit v3 CLI.
//!
//! `sos install | update | check` vendor the harness into a repo and keep it healthy;
//! `sos gate …` and `sos filter` are the agent-neutral checks the git hooks call.
//! The v2 commands (new, adopt, map, sync, launch, …) are archived in archive/v2/.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod gate;
mod kit;

#[derive(Parser)]
#[command(name = "sos", version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("SOS_KIT_COMMIT"), ")"),
          about = "SOS Kit — harness for building with AI agents: install it in a repo and run its gates")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Install the harness into a git repo: harness-lite/ (kit-owned), starter project files
    /// (created only when missing), .gitignore lines, and git hooks.
    Install {
        /// Repository top level (default: current directory).
        #[arg(long, default_value = ".")]
        dir: PathBuf,
        /// Show the plan; write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Point core.hooksPath at harness-lite/hooks even if the repo already uses another hooks dir.
        #[arg(long)]
        force_hooks: bool,
    },
    /// Bring harness-lite/ to this sos version; locally edited files are kept (new version → .sos-new).
    Update {
        #[arg(long, default_value = ".")]
        dir: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Check an installed repo: kit files, hooks, required tools, .sos.toml, agent wiring.
    Check {
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },
    /// Run gates (what the git hooks call). Config: .sos.toml. Exit 0 ok, 1 blocked (including a
    /// missing required tool such as gitleaks), 2 invalid config or internal error.
    Gate {
        #[command(subcommand)]
        which: gate::GateCmd,
    },
    /// Strip banned wording, AI-vendor leaks and <thinking> blocks from stdin
    /// (runtime filter for LLM output; rules from .sos.toml [text]).
    Filter,
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Cmd::Install { dir, dry_run, force_hooks } => kit::install(&dir, &kit::Opts { dry_run, force_hooks }),
        Cmd::Update { dir, dry_run } => kit::update(&dir, &kit::Opts { dry_run, force_hooks: false }),
        Cmd::Check { dir } => kit::check(&dir),
        Cmd::Gate { which } => Ok(gate::run(which)),
        Cmd::Filter => Ok(gate::filter()),
    };
    match code {
        Ok(c) => std::process::exit(c),
        Err(e) => {
            eprintln!("sos: {e:#}");
            std::process::exit(2);
        }
    }
}
