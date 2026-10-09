//! `sos gate …` and `sos filter` — thin CLI over the sos-gates crate.

use clap::Subcommand;
use sos_gates::{config, docs, features, local_secrets, text, Report};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum GateCmd {
    /// Wording rules on files, or on staged files matching [text].files.
    Text {
        /// Files to check (working tree). Ignored with --staged.
        files: Vec<String>,
        #[arg(long)]
        staged: bool,
    },
    /// Line caps on [docs].files.
    Docs,
    /// FEATURES.json identity rules on the staged file vs HEAD.
    Features,
    /// Tokens in .git/config and agent config files; .env files ignored and untracked.
    LocalSecrets,
    /// text --staged, docs and features (what the pre-commit runs).
    All,
}

fn root() -> PathBuf {
    std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Exit code: 0 ok, 1 a gate blocked, 2 config or runtime error.
pub fn run(which: GateCmd) -> i32 {
    let root = root();
    let cfg = match config::load(&root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sos gate: {e:#}\n  → How to fix: correct .sos.toml (see `sos gate --help`)");
            return 2;
        }
    };
    let reports: anyhow::Result<Vec<Report>> = (|| {
        Ok(match which {
            GateCmd::Text { files, staged } => vec![text::gate(&root, &cfg.text, &files, staged)?],
            GateCmd::Docs => vec![docs::gate(&root, &cfg.docs)],
            GateCmd::Features => vec![features::gate(&root, &cfg.features)?],
            GateCmd::LocalSecrets => vec![local_secrets::gate(&root)?],
            GateCmd::All => vec![
                text::gate(&root, &cfg.text, &[], true)?,
                docs::gate(&root, &cfg.docs),
                features::gate(&root, &cfg.features)?,
            ],
        })
    })();
    match reports {
        Err(e) => {
            eprintln!("sos gate: {e:#}");
            2
        }
        Ok(rs) => {
            for r in &rs {
                eprint!("{}", r.render());
            }
            if rs.iter().all(Report::ok) { 0 } else { 1 }
        }
    }
}

pub fn filter() -> i32 {
    let root = root();
    let rules = match config::load(&root).and_then(|c| text::Rules::from_config(&c.text)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("sos filter: {e:#}");
            return 2;
        }
    };
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("sos filter: stdin is not UTF-8 text");
        return 2;
    }
    let (out, v) = text::filter(&input, &rules);
    let _ = std::io::stdout().write_all(out.as_bytes());
    if !v.is_empty() {
        eprintln!("sos filter: stripped {} violation(s)", v.len());
    }
    0
}
