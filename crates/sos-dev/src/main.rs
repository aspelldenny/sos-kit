//! `sos-dev` — SOS Kit's own dev tooling. Kit only: not in the release, never installed into apps.
//!
//!   sos-dev trust-gate [check|rebaseline|list]   content-integrity gate (was scripts/trust-gate.sh)
//!   sos-dev docs-check [SOS] [--root DIR]        docs name only real paths and commands (was tests/docs-check.py)
//!   sos-dev install-hooks                        core.hooksPath → hooks/ (was scripts/install-hooks.sh)

mod docs;
mod git;
mod hooks;
mod trust;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "sos-dev", about = "SOS Kit's own dev tooling (kit repo only; never shipped to apps)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Copy, ValueEnum)]
enum TrustMode {
    /// Baseline diff + hidden-Unicode scan (default).
    Check,
    /// Regenerate .sos-trust-baseline from the tracked surfaces.
    Rebaseline,
    /// Print the tracked surfaces the baseline covers.
    List,
}

#[derive(Subcommand)]
enum Cmd {
    /// Auto-exec surfaces match .sos-trust-baseline; no hidden Unicode in instruction files.
    TrustGate {
        #[arg(value_enum, default_value = "check")]
        mode: TrustMode,
    },
    /// Every repo path and `sos …` command named in the live Markdown exists.
    DocsCheck {
        /// The sos binary whose --help lists the commands (default <root>/target/debug/sos).
        sos: Option<PathBuf>,
        /// Repo root (default: the git top level of the current folder).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Point core.hooksPath at the tracked hooks/ folder.
    InstallHooks,
}

fn toplevel() -> anyhow::Result<PathBuf> {
    git::toplevel(&std::env::current_dir()?)
}

fn run(cli: Cli) -> anyhow::Result<u8> {
    match cli.cmd {
        Cmd::TrustGate { mode } => {
            let root = toplevel()?;
            match mode {
                TrustMode::Rebaseline => trust::rebaseline(&root).map(|_| 0),
                TrustMode::List => trust::list(&root).map(|_| 0),
                TrustMode::Check => {
                    let base = trust::check_baseline(&root)?;
                    let uni = trust::check_unicode(&root)?;
                    Ok(if base && uni { 0 } else { 1 })
                }
            }
        }
        Cmd::DocsCheck { sos, root } => {
            let root = match root {
                Some(r) => r,
                None => toplevel()?,
            };
            let sos = sos.unwrap_or_else(|| docs::default_sos(&root));
            Ok(if docs::run(&root, &sos)? { 0 } else { 1 })
        }
        Cmd::InstallHooks => hooks::run(&toplevel()?).map(|c| c as u8),
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("ERROR: sos-dev: {e:#}");
            ExitCode::from(1)
        }
    }
}
