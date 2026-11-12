//! `sos gate ready` — run the project's end-to-end check (`.sos.toml [ready] cmd`) and record the
//! revision and result. Each slice can be green while the combined product is red (Payquill F35 and
//! F31, 23/10/2026), so the handover report quotes this line, not a claim. Not part of `sos gate all`:
//! it may be slow, and it runs at the handover boundary, not on every commit.
//!
//! The record is one plain line per run in `evidence/ready/runs.log`, so `scripts/status.sh` reads
//! it with `tail`: `PASS|FAIL <rev>[+dirty] <local time> <secs>s exit=<code> — <cmd>`. Exit 0 pass,
//! 1 fail or no command, 2 the command could not start or the record could not be written: a run
//! nothing recorded proves nothing.

use anyhow::{Context, Result};
use sos_gates::config::ReadyConfig;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub const LOG: &str = "evidence/ready/runs.log";

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let o = Command::new("git").current_dir(root).args(args).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn record(root: &Path, line: &str) -> Result<()> {
    let path = root.join(LOG);
    std::fs::create_dir_all(path.parent().unwrap_or(root)).with_context(|| format!("creating the folder of {LOG}"))?;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&path).with_context(|| format!("opening {LOG}"))?;
    writeln!(f, "{line}").with_context(|| format!("writing {LOG}"))
}

pub fn run(root: &Path, cfg: &ReadyConfig) -> i32 {
    let cmd = cfg.cmd.trim();
    if cmd.is_empty() {
        eprintln!("⛔ .sos.toml [ready] cmd is empty, so nothing proves the whole product works.\n  \
                   → How to fix: set `cmd` to what exercises the app end to end (e.g. \"make ready\"), `git add .sos.toml` (gates read what will be committed), then run this again.");
        return 1;
    }
    let rev = git(root, &["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "no-commit".into());
    // Its own log must not mark the next run dirty in an app whose .gitignore lacks the line yet.
    let rev = if tree_changed(root) { format!("{rev}+dirty") } else { rev };
    let ts = crate::role::local_time("%Y-%m-%dT%H:%M:%S");
    let start = std::time::Instant::now();
    // Git for Windows provides `sh`, as it does for hooks.
    let status = match Command::new("sh").args(["-c", cmd]).current_dir(root).status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("⛔ sos gate ready: could not start `sh` ({e}).\n  → How to fix: put a POSIX shell on PATH (Git for Windows ships one).");
            return 2;
        }
    };
    let (word, code) = (if status.success() { "PASS" } else { "FAIL" }, status.code().unwrap_or(-1));
    let line = format!("{word} {rev} {ts} {}s exit={code} — {}", start.elapsed().as_secs(), cmd.split_whitespace().collect::<Vec<_>>().join(" "));
    if let Err(e) = record(root, &line) {
        eprintln!("⛔ sos gate ready: {e:#}.\n  → How to fix: make {LOG} writable and run again; the result was: {line}");
        return 2;
    }
    eprintln!("sos gate ready: {line}");
    if status.success() {
        0
    } else {
        eprintln!("  → How to fix: fix what the command reported, then run `sos gate ready` again.");
        1
    }
}

/// What the banner says while nothing has ever passed. Same text as `scripts/status.sh`.
pub const NO_PASS: &str = "Note: `sos gate ready` has never passed here: get one journey working end to end (`sos gate skeleton`) before starting another.";

/// Whether any recorded run passed.
pub fn ever_passed(root: &Path) -> bool {
    std::fs::read_to_string(root.join(LOG)).is_ok_and(|l| l.lines().any(|l| l.starts_with("PASS ")))
}

/// `sos gate skeleton` — the skeleton stands: one journey runs end to end, proven by the last
/// recorded `sos gate ready`. Growing a second journey on a skeleton that does not run is how an
/// app ends up with every screen drawn and nothing working (plan
/// `docs/plans/QUY-TRINH-LAM-APP-NHAP-2026-11-12.md`, change 3). It reads the record and runs
/// nothing; it cannot know which journey the command walks, only that one was run and passed.
pub fn skeleton(root: &Path, cfg: &ReadyConfig) -> i32 {
    const HOW: &str = "finish the skeleton journey of the blueprint end to end on real data with a test that walks it, \
                       set `.sos.toml` [ready] cmd to run that test, and run `sos gate ready` until it passes. Do not start a second journey before that.";
    if cfg.cmd.trim().is_empty() {
        eprintln!("⛔ sos gate skeleton: .sos.toml [ready] cmd is empty, so nothing shows that one journey works end to end.\n  → How to fix: {HOW}");
        return 1;
    }
    let log = std::fs::read_to_string(root.join(LOG)).unwrap_or_default();
    match log.lines().rev().find(|l| !l.trim().is_empty()) {
        Some(last) => match stands(last, cfg.cmd.trim()) {
            Ok(()) => {
                eprintln!("sos gate skeleton: ok — {}", last_note(root).unwrap_or_else(|| last.to_string()));
                0
            }
            Err(why) => {
                eprintln!("⛔ sos gate skeleton: the last end-to-end record {why}: {last}\n  → How to fix: {HOW}");
                1
            }
        },
        None => {
            eprintln!("⛔ sos gate skeleton: `sos gate ready` has never been run here ({LOG} has no run).\n  → How to fix: {HOW}");
            1
        }
    }
}

/// Whether a line of the log is a pass of the command configured now: the shape `run` writes
/// (`PASS <rev> <time> <secs>s exit=0 — <cmd>`) and the same command. A pass of another command
/// says nothing about the one the app now calls its end-to-end check.
fn stands(line: &str, cmd: &str) -> Result<(), &'static str> {
    let Some((head, ran)) = line.split_once(" — ") else { return Err("is not a line `sos gate ready` writes") };
    let f: Vec<&str> = head.split(' ').collect();
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let rev = |s: &str| s == "no-commit" || (s.len() >= 4 && s.bytes().all(|b| b.is_ascii_hexdigit()));
    // 2026-11-12T10:00:00
    let time = |s: &str| s.len() == 19 && s.bytes().enumerate().all(|(i, b)| match i {
        4 | 7 => b == b'-',
        10 => b == b'T',
        13 | 16 => b == b':',
        _ => b.is_ascii_digit(),
    });
    let shaped = f.len() == 5
        && ["PASS", "FAIL"].contains(&f[0])
        && rev(f[1].strip_suffix("+dirty").unwrap_or(f[1]))
        && (time(f[2]) || digits(f[2])) // Unix seconds where `date` is not available
        && f[3].strip_suffix('s').is_some_and(digits)
        && f[4].strip_prefix("exit=").is_some_and(|c| digits(c.strip_prefix('-').unwrap_or(c)));
    if !shaped {
        return Err("is not a line `sos gate ready` writes");
    }
    if f[0] != "PASS" || f[4] != "exit=0" {
        return Err("did not pass");
    }
    if ran != cmd.split_whitespace().collect::<Vec<_>>().join(" ") {
        return Err("is for another command than `.sos.toml` [ready] cmd now names (run `sos gate ready` again)");
    }
    Ok(())
}

/// The tree is the record's own business: its log must not count as a change.
fn tree_changed(root: &Path) -> bool {
    git(root, &["status", "--porcelain", "--", ".", ":!evidence/ready"]).is_some_and(|s| !s.is_empty())
}

/// The last recorded run in `root`, with a note when it may not speak for what is here now: a PASS on
/// the same commit proves nothing about edits since, and one made on a dirty tree cannot be matched
/// to the tree now. Same text as `scripts/status.sh`.
pub fn last_note(root: &Path) -> Option<String> {
    let log = std::fs::read_to_string(root.join(LOG)).ok()?;
    let line = log.lines().rev().find(|l| !l.trim().is_empty())?;
    let logged = line.split(' ').nth(1).unwrap_or("");
    let rev = logged.trim_end_matches("+dirty");
    let head = git(root, &["rev-parse", "--short", "HEAD"]).unwrap_or_default();
    Some(if rev != head {
        format!("Ready: {line} (HEAD is now {head})")
    } else if rev != logged || tree_changed(root) {
        format!("Ready: {line} (the tree may differ from that run: rerun before handover)")
    } else {
        format!("Ready: {line}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAY_DIFFER: &str = "(the tree may differ from that run: rerun before handover)";

    fn repo() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for a in [&["init", "-q"][..], &["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "x"][..]] {
            assert!(Command::new("git").current_dir(d.path()).args(a).status().unwrap().success());
        }
        d
    }

    #[test]
    fn records_revision_and_result_of_each_run() {
        let d = repo();
        let cfg = |c: &str| ReadyConfig { cmd: c.into() };
        assert_eq!(run(d.path(), &cfg("  ")), 1);
        assert!(!d.path().join(LOG).exists());
        assert_eq!(last_note(d.path()), None);

        assert_eq!(run(d.path(), &cfg("exit 3")), 1);
        assert_eq!(run(d.path(), &cfg("pwd >/dev/null\ntrue")), 0);
        let log = std::fs::read_to_string(d.path().join(LOG)).unwrap();
        let head = git(d.path(), &["rev-parse", "--short", "HEAD"]).unwrap();
        let lines: Vec<&str> = log.lines().collect();
        assert!(lines[0].starts_with(&format!("FAIL {head} ")) && lines[0].contains(" exit=3 — exit 3"), "{log}");
        assert!(lines[1].starts_with(&format!("PASS {head} ")) && lines[1].ends_with("— pwd >/dev/null true"), "{log}");
        assert_eq!(last_note(d.path()).unwrap(), format!("Ready: {}", lines[1]));

        // Edited after a clean PASS: same commit, but the record no longer speaks for the tree.
        std::fs::write(d.path().join("later.txt"), "x").unwrap();
        assert!(last_note(d.path()).unwrap().ends_with(MAY_DIFFER), "{:?}", last_note(d.path()));
        std::fs::remove_file(d.path().join("later.txt")).unwrap();

        // A later commit: the record no longer speaks for HEAD, and the note says so.
        assert!(Command::new("git").current_dir(d.path())
            .args(["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "y"]).status().unwrap().success());
        let now = git(d.path(), &["rev-parse", "--short", "HEAD"]).unwrap();
        assert!(last_note(d.path()).unwrap().ends_with(&format!("(HEAD is now {now})")));
    }

    #[test]
    fn skeleton_stands_only_when_the_last_recorded_run_passed() {
        let d = repo();
        let cfg = |c: &str| ReadyConfig { cmd: c.into() };
        // No command, then a command never run: both block.
        assert_eq!(skeleton(d.path(), &cfg("")), 1);
        assert_eq!(skeleton(d.path(), &cfg("true")), 1);
        assert!(!ever_passed(d.path()));
        // A failed run blocks; a pass opens it; a later failure closes it again.
        assert_eq!(run(d.path(), &cfg("false")), 1);
        assert_eq!(skeleton(d.path(), &cfg("false")), 1);
        assert!(!ever_passed(d.path()));
        assert_eq!(run(d.path(), &cfg("true")), 0);
        assert_eq!(skeleton(d.path(), &cfg("true")), 0);
        assert!(ever_passed(d.path()));
        assert_eq!(run(d.path(), &cfg("false")), 1);
        assert_eq!(skeleton(d.path(), &cfg("true")), 1);
        // The record still holds the command: an emptied [ready] cmd blocks whatever the log says.
        assert_eq!(run(d.path(), &cfg("true")), 0);
        assert_eq!(skeleton(d.path(), &cfg(" ")), 1);
        // A pass of another command, or a line `sos gate ready` did not write, proves nothing.
        assert_eq!(skeleton(d.path(), &cfg("true && true")), 1);
        assert_eq!(skeleton(d.path(), &cfg("true")), 0);
        std::fs::write(d.path().join(LOG), "PASS garbage\n").unwrap();
        assert_eq!(skeleton(d.path(), &cfg("true")), 1);
        for bad in ["PASS garbage garbage garbage exit=0 — true", "PASS abc1234 yesterday 1s exit=0 — true",
                    "PASS abc1234 2026-11-12T10:00:00 fast exit=0 — true", "PASS not-a-rev 2026-11-12T10:00:00 1s exit=0 — true"] {
            std::fs::write(d.path().join(LOG), format!("{bad}\n")).unwrap();
            assert_eq!(skeleton(d.path(), &cfg("true")), 1, "{bad}");
        }
        std::fs::write(d.path().join(LOG), "FAIL abc1234 2026-11-12T10:00:00 1s exit=1 — true\nPASS abc1234+dirty 1794477600 1s exit=0 — true\n").unwrap();
        assert_eq!(skeleton(d.path(), &cfg("true")), 0);
        std::fs::write(d.path().join(LOG), "PASS abc1234 2026-11-12T10:00:00 1s exit=1 — true\n").unwrap();
        assert_eq!(skeleton(d.path(), &cfg("true")), 1);
    }

    #[test]
    fn a_dirty_tree_is_named_and_a_run_that_cannot_be_recorded_fails() {
        let d = repo();
        std::fs::write(d.path().join("x.txt"), "x").unwrap();
        assert_eq!(run(d.path(), &ReadyConfig { cmd: "true".into() }), 0);
        let head = git(d.path(), &["rev-parse", "--short", "HEAD"]).unwrap();
        assert!(std::fs::read_to_string(d.path().join(LOG)).unwrap().starts_with(&format!("PASS {head}+dirty ")));
        // Run on a dirty tree, then read on a dirty or a clean one: neither can be matched to that run.
        assert!(last_note(d.path()).unwrap().ends_with(MAY_DIFFER));
        std::fs::remove_file(d.path().join("x.txt")).unwrap();
        assert!(last_note(d.path()).unwrap().ends_with(MAY_DIFFER));
        // `evidence/ready` is a file: the check passes but leaves no record, so the gate says no.
        let e = repo();
        std::fs::create_dir(e.path().join("evidence")).unwrap();
        std::fs::write(e.path().join("evidence/ready"), "").unwrap();
        assert_eq!(run(e.path(), &ReadyConfig { cmd: "true".into() }), 2);
    }
}
