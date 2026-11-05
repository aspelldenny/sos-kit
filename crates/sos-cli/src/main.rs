//! sos — SOS Kit v3 CLI.
//!
//! `sos install | update | check` vendor the harness into a repo and keep it healthy;
//! `sos gate …` and `sos filter` are the agent-neutral checks the git hooks call.
//! The v2 commands (new, adopt, map, sync, launch, …) are archived in archive/v2/.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod advise;
mod gate;
mod guards;
mod hook;
mod kit;
mod role;
mod status;
mod usage;
mod watch;
mod wiring;

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
        /// Also rewrite the kit's own hook commands in .claude/settings.json and .codex/hooks.json
        /// (old forms → `sos hook`, missing kit events added); other entries are left alone.
        #[arg(long)]
        wiring: bool,
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
    /// Run one role (worker, reviewer, architect) on the model .sos.toml [roles] names for it:
    /// prompt on stdin with stdin closed, a wall-clock limit, the final message in one file.
    Role {
        /// worker | reviewer | architect
        role: String,
        /// The family that built the work (claude | codex); picks [roles.<role>.<from>] when set.
        #[arg(long)]
        from: String,
        /// File with the bounded brief.
        #[arg(long)]
        brief: PathBuf,
        /// Where the final message goes (default evidence/roles/<time>-<role>-<backend>.md).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Seconds before the run is stopped (default [roles] timeout_secs, else 1800).
        #[arg(long)]
        timeout: Option<u64>,
        /// Print the command; run nothing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Agent hooks: `sos hook claude|codex <session-start|pre-edit|pre-bash|post-bash>` (payload on
    /// stdin; exit 2 blocks), `rules --file F` (declarative rules), plus the agent-neutral
    /// `env-guard`, `sim-guard`, `status` and `rules --file F [--lint|--cases C]`.
    Hook {
        #[command(subcommand)]
        which: hook::HookCmd,
    },
    /// Token use by role × model from local Claude Code transcripts (and Codex rollouts), with a
    /// comparison price and suggestions. Read-only: nothing is sent, written or run.
    Usage(usage::Args),
    /// Ask a stronger model (the other family by default) for direction on the current work, from
    /// a snapshot: question, intent, latest failure, git state, relevant files. Exit 0 advice,
    /// 1 failed, 2 off or misconfigured, 3 unavailable (CLI not logged in). Settings: .sos.toml [advisor].
    Advise(advise::Args),
    /// Strip banned wording, AI-vendor leaks and <thinking> blocks from stdin
    /// (runtime filter for LLM output; rules from .sos.toml [text]).
    Filter,
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Cmd::Install { dir, dry_run, force_hooks } => kit::install(&dir, &kit::Opts { dry_run, force_hooks, wiring: false }),
        Cmd::Update { dir, dry_run, wiring } => kit::update(&dir, &kit::Opts { dry_run, force_hooks: false, wiring }),
        Cmd::Check { dir } => kit::check(&dir),
        Cmd::Gate { which } => Ok(gate::run(which)),
        Cmd::Role { role, from, brief, out, timeout, dry_run } =>
            role::run(&role::Args { role, from, brief, out, timeout, dry_run }),
        Cmd::Hook { which } => Ok(hook::run(which)),
        Cmd::Usage(a) => usage::run(a),
        Cmd::Advise(a) => advise::run(a),
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
