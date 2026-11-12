//! `sos gate …` and `sos filter` — thin CLI over the sos-gates crate.

use clap::Subcommand;
use sos_gates::{blueprint, config, docs, features, ledger, local_secrets, repo, text, Report};
use anyhow::Context;
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
    /// Line caps on [docs].files; bytes and item length on [docs].sized.
    Docs,
    /// Relative links and anchors in [docs].links, against what will be committed.
    Links,
    /// Run [ready].cmd, the end-to-end check, and record revision and result in evidence/ready/.
    /// Not part of `all`: run it before handing work over.
    Ready,
    /// The blueprint ([blueprint].file) has journeys with concrete examples and one marked skeleton,
    /// a data model and the irreversible actions. Not part of `all`: run it before the blueprint review.
    Blueprint,
    /// The skeleton stands: [ready].cmd is set and its last recorded run passed. Not part of `all`:
    /// run it before briefing a second journey.
    Skeleton,
    /// FEATURES.json identity rules on the staged file vs HEAD.
    Features,
    /// Tokens in .git/config and agent config files; .env files ignored and untracked.
    LocalSecrets,
    /// Staged diff scanned by gitleaks.
    Secrets,
    /// No real .env file staged.
    EnvCommit,
    /// No two tracked paths differing only by letter case.
    Case,
    /// [git] protect_default_branch: no non-Markdown commit on the default branch.
    Branch,
    /// Everything the pre-commit runs: secrets, env-commit, case, branch, text --staged, docs, links, features.
    All,
    /// The app's git pre-commit: `all`, then the project's own pre-commit.local if there is one.
    PreCommit {
        /// The project's extra hook (default harness-lite/hooks/pre-commit.local).
        #[arg(long)]
        local: Option<PathBuf>,
        /// Passed on to the local hook.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// The app's git commit-msg: a fix commit must name what caught the defect and the stage that let
    /// it through (`Caught-by:`, `Escaped-from:`, [ledger]), then the project's own commit-msg.local if there is one.
    CommitMsg {
        /// The file git passes with the message.
        file: PathBuf,
        /// The project's extra hook (default harness-lite/hooks/commit-msg.local).
        #[arg(long)]
        local: Option<PathBuf>,
    },
    /// The app's git pre-push: `local-secrets`, then the project's own pre-push.local if there is
    /// one (it gets git's arguments and the ref list on stdin).
    PrePush {
        /// The project's extra hook (default harness-lite/hooks/pre-push.local).
        #[arg(long)]
        local: Option<PathBuf>,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
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
    let (first, name, local, args) = match which {
        GateCmd::PreCommit { local, args } => (GateCmd::All, "pre-commit", local, args),
        GateCmd::PrePush { local, args } => (GateCmd::LocalSecrets, "pre-push", local, args),
        GateCmd::CommitMsg { file, local } => {
            let arg = file.display().to_string();
            (GateCmd::CommitMsg { file, local: None }, "commit-msg", local, vec![arg])
        }
        other => return gates(other),
    };
    let rc = gates(first);
    if rc != 0 {
        return rc;
    }
    let local = local.unwrap_or_else(|| root().join(format!("harness-lite/hooks/{name}.local")));
    run_local(&local, &args)
}

/// The project's own hook after the gates. Present but unable to run is a failure, never a skip:
/// a check the project relies on must not vanish silently (Windows has no exec bit to look at,
/// so there the spawn itself decides).
fn run_local(path: &std::path::Path, args: &[String]) -> i32 {
    let shown = path.display().to_string().replace('\\', "/");
    if !path.exists() {
        return 0;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if std::fs::metadata(path).map(|m| m.permissions().mode() & 0o111 == 0).unwrap_or(true) {
            eprintln!("⛔ {shown} exists but is not executable.\n  → How to fix: chmod +x {shown} (or delete it if it is not meant to run).");
            return 1;
        }
    }
    let abs = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(path) };
    // Windows has no exec bit and no kernel #!: let Git's sh exec it, which reads the #! line the
    // way git does for hooks (a Python .local stays Python; review 1b point 7).
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", "exec \"$0\" \"$@\""]).arg(&abs);
        c
    } else {
        std::process::Command::new(&abs)
    };
    match cmd.args(args).status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("⛔ {shown} exists but could not run ({e}).\n  → How to fix: give it a #! line and make it runnable, or delete it.");
            1
        }
    }
}

fn gates(which: GateCmd) -> i32 {
    let root = root();
    let cfg = match config::load(&root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sos gate: {e:#}\n  → How to fix: correct .sos.toml (see `sos gate --help`)");
            return 2;
        }
    };
    if matches!(which, GateCmd::Ready) {
        return crate::ready::run(&root, &cfg.ready);
    }
    if matches!(which, GateCmd::Skeleton) {
        return crate::ready::skeleton(&root, &cfg.ready);
    }
    let reports: anyhow::Result<Vec<Report>> = (|| {
        Ok(match which {
            GateCmd::Text { files, staged } => vec![text::gate(&root, &cfg.text, &files, staged)?],
            GateCmd::Docs => vec![docs::gate(&root, &cfg.docs)],
            GateCmd::Blueprint => vec![blueprint::gate(&root, &cfg.blueprint)],
            GateCmd::Links => vec![crate::check_links::gate(&root, &cfg.docs)?],
            GateCmd::Features => vec![features::gate(&root, &cfg.features)?],
            GateCmd::LocalSecrets => vec![local_secrets::gate(&root)?],
            GateCmd::Secrets => vec![repo::secrets(&root)?],
            GateCmd::EnvCommit => vec![repo::env_commit(&root)?],
            GateCmd::Case => vec![repo::case_collision(&root)?],
            GateCmd::Branch => vec![repo::branch(&root, &cfg.git)?],
            GateCmd::All => vec![
                repo::secrets(&root)?,
                repo::env_commit(&root)?,
                repo::case_collision(&root)?,
                repo::branch(&root, &cfg.git)?,
                text::gate(&root, &cfg.text, &[], true)?,
                docs::gate(&root, &cfg.docs),
                crate::check_links::gate(&root, &cfg.docs)?,
                features::gate(&root, &cfg.features)?,
            ],
            GateCmd::CommitMsg { file, .. } => {
                let msg = std::fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
                vec![ledger::gate(&cfg.ledger, &msg)?]
            }
            GateCmd::Ready | GateCmd::Skeleton | GateCmd::PreCommit { .. } | GateCmd::PrePush { .. } => unreachable!("handled above"),
        })
    })();
    match reports {
        Err(e) => {
            eprintln!("sos gate: {e:#}\n  → How to fix: run it from inside the git repository; if the message names a file, make it readable UTF-8 text");
            2
        }
        Ok(rs) => {
            // Agents read hook output: when everything passes, say so in one line.
            if rs.iter().all(|r| r.ok() && r.warnings.is_empty()) {
                let skipped: Vec<&str> = rs.iter().filter(|r| r.skipped.is_some()).map(|r| r.gate).collect();
                let ran = rs.len() - skipped.len();
                if skipped.is_empty() {
                    eprintln!("sos gate: {ran} ok");
                } else {
                    eprintln!("sos gate: {ran} ok, not configured: {}", skipped.join(", "));
                }
                return 0;
            }
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
