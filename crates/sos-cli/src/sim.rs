//! `sos sim …` — one simulator, used gently (wave 2.5; ports of tally/turnsigil `tools/sim-boot.sh`
//! and `tools/wait-settled.sh`, and lucilius `scripts/resolve-simulator-destination.sh`).
//!
//!   sos sim boot <udid> [--log F]        boot that one device if it is not booted, wait for boot status;
//!                                        never creates, clones or erases a device
//!   sos sim settle <label> [--log F]     wait until the 1-minute load settles before a UI-test phase;
//!                                        a timeout warns and exits 0 (the run goes on)
//!   sos sim destination [--name N]       `platform=iOS Simulator,id=<udid>` for the named available
//!                                        iOS device, else the first available iOS device
//!
//! The command check that stops a second simulator run before it starts stays `sos hook sim-guard`
//! (crate::guards::sim; a hook contract, so not moved).
//!
//! Every step of boot and settle is appended to the log (default build/uitest-settle.log), as the
//! scripts did. `xcrun` and `sysctl` are found on PATH, so tests put fakes there.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const DEFAULT_LOG: &str = "build/uitest-settle.log";

#[derive(Subcommand)]
pub enum SimCmd {
    /// Boot the one simulator (by UDID) if it is not booted, then wait for its boot status.
    /// Never creates, clones or erases a device. Exit 1 when simctl fails.
    Boot {
        udid: String,
        /// Log file (appended).
        #[arg(long, default_value = DEFAULT_LOG)]
        log: PathBuf,
    },
    /// Wait until the 1-minute load is <= --load in --run samples --interval seconds apart, or
    /// --max seconds pass. A timeout only warns: exit 0, the run goes on. Each option also reads
    /// the old script's variable (SETTLE_LOAD, SETTLE_MAX, SETTLE_INTERVAL, SETTLE_RUN).
    Settle {
        label: String,
        /// Log file (appended).
        #[arg(long, default_value = DEFAULT_LOG)]
        log: PathBuf,
        /// Load threshold (default 6.0: about twice a quiet 8 GB Mac's reading, below its 8 CPUs).
        #[arg(long)]
        load: Option<f64>,
        /// Give up after this many seconds (default 600: a boot's load decays over ~5 minutes).
        #[arg(long)]
        max: Option<u64>,
        /// Seconds between samples (default 20).
        #[arg(long)]
        interval: Option<u64>,
        /// Samples in a row that must be under the threshold (default 2: one dip does not count).
        #[arg(long)]
        run: Option<u32>,
    },
    /// Print `platform=iOS Simulator,id=<udid>` for the available iOS device called --name, else
    /// the first available iOS device. Exit 1 when there is none.
    Destination {
        /// The device name to prefer (the project's own simulator).
        #[arg(long)]
        name: Option<String>,
    },
}

pub fn run(cmd: SimCmd) -> Result<i32> {
    match cmd {
        SimCmd::Boot { udid, log } => boot(&udid, &log),
        SimCmd::Settle { label, log, load, max, interval, run } => {
            let o = SettleOpts {
                load: pick(load, "SETTLE_LOAD", 6.0)?,
                max: pick(max, "SETTLE_MAX", 600)?,
                interval: pick(interval, "SETTLE_INTERVAL", 20)?,
                run: pick(run, "SETTLE_RUN", 2)?,
            };
            settle(&label, &log, &o)
        }
        SimCmd::Destination { name } => destination(name.as_deref()),
    }
}

/// The flag, else the old script's environment variable, else the default.
fn pick<T: std::str::FromStr>(flag: Option<T>, var: &str, default: T) -> Result<T> {
    if let Some(v) = flag {
        return Ok(v);
    }
    match std::env::var(var) {
        Ok(s) if !s.is_empty() => s.trim().parse().map_err(|_| anyhow::anyhow!("{var}={s} is not a number")),
        _ => Ok(default),
    }
}

fn stamp() -> String {
    crate::role::local_time("%Y-%m-%dT%H:%M:%S").replacen('T', " ", 1)
}

/// Append one stamped line to the log, creating its folder.
fn log_line(log: &Path, line: &str) -> Result<()> {
    if let Some(dir) = log.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
    }
    let mut f = fs::OpenOptions::new().create(true).append(true).open(log)
        .with_context(|| format!("cannot open {}", log.display()))?;
    writeln!(f, "{} {line}", stamp()).with_context(|| format!("cannot write {}", log.display()))
}

/// `xcrun simctl <args>`: (success, stdout).
fn simctl(args: &[&str]) -> Result<(bool, String)> {
    let out = Command::new("xcrun").arg("simctl").args(args).output()
        .context("cannot run `xcrun simctl` (Xcode command line tools missing?)")?;
    Ok((out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned()))
}

fn boot(udid: &str, log: &Path) -> Result<i32> {
    let (ok, list) = simctl(&["list", "devices"])?;
    let tag = format!("({udid})");
    let booted = ok && list.lines().any(|l| l.contains(&tag) && l.contains("(Booted)"));
    if booted {
        log_line(log, &format!("[boot] {udid} already booted"))?;
    } else {
        log_line(log, &format!("[boot] booting {udid}"))?;
        if !Command::new("xcrun").args(["simctl", "boot", udid]).status().is_ok_and(|s| s.success()) {
            println!("⛔ simctl boot {udid} failed");
            return Ok(1);
        }
    }
    let waited = Command::new("xcrun").args(["simctl", "bootstatus", udid, "-b"])
        .stdout(std::process::Stdio::null()).status();
    if !waited.is_ok_and(|s| s.success()) {
        println!("⛔ simctl bootstatus {udid} failed");
        return Ok(1);
    }
    log_line(log, &format!("[boot] {udid} boot status complete"))?;
    Ok(0)
}

struct SettleOpts {
    load: f64,
    max: u64,
    interval: u64,
    run: u32,
}

/// The 1-minute load from `sysctl -n vm.loadavg` (`{ 1.23 1.10 0.98 }` on macOS, where the
/// simulator lives). None when it cannot be read: there is no other source, so an unreadable load
/// never passes for a low one.
fn load1() -> Option<String> {
    let out = Command::new("sysctl").args(["-n", "vm.loadavg"]).output().ok().filter(|o| o.status.success())?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let v = text.split_whitespace().nth(1)?;
    // Only a finite, non-negative number is a load: `-inf` or `nan` would pass for a low one.
    v.parse::<f64>().ok().filter(|l| l.is_finite() && *l >= 0.0).map(|_| v.to_string())
}

fn settle(label: &str, log: &Path, o: &SettleOpts) -> Result<i32> {
    if !(o.load.is_finite()) || o.run == 0 {
        bail!("settle: the load must be a number and --run at least 1");
    }
    let start = Instant::now();
    let mut run = 0u32;
    log_line(log, &format!("[{label}] start: settle when 1-min load <= {} in {} samples {}s apart, max {}s",
        o.load, o.run, o.interval, o.max))?;
    loop {
        let elapsed = start.elapsed().as_secs();
        let load = load1();
        // An unreadable load is not a low one: it does not count towards settling.
        let under = load.as_deref().and_then(|l| l.parse::<f64>().ok()).is_some_and(|l| l <= o.load);
        run = if under { run + 1 } else { 0 };
        let shown = load.unwrap_or_else(|| "?".into());
        log_line(log, &format!("[{label}] +{elapsed}s load1={shown} under={run}/{}", o.run))?;
        if run >= o.run {
            log_line(log, &format!("[{label}] settled after {elapsed}s (load1={shown})"))?;
            println!("settle [{label}]: load {shown} after {elapsed}s");
            return Ok(0);
        }
        if elapsed >= o.max {
            log_line(log, &format!("[{label}] WARNING: not settled after {elapsed}s (load1={shown} > {}); continuing", o.load))?;
            println!("⚠️  settle [{label}]: load still {shown} after {elapsed}s (threshold {}); continuing (see {})",
                o.load, log.display());
            return Ok(0);
        }
        std::thread::sleep(Duration::from_secs(o.interval));
    }
}

/// One device line of `simctl list devices available`: `    <name> (<udid>) (<state>)`.
fn device_line(line: &str) -> Option<(&str, &str)> {
    let body = line.strip_prefix("    ")?;
    if body.starts_with(' ') {
        return None;
    }
    // The UDID is the last parenthesised group before the state: a name may hold parentheses.
    let state_at = body.rfind(" (")?;
    let head = &body[..state_at];
    let udid_at = head.rfind(" (")?;
    let udid = head[udid_at + 2..].strip_suffix(')')?;
    let is_udid = udid.len() == 36 && udid.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    is_udid.then(|| (&head[..udid_at], udid))
}

/// The iOS devices of `simctl list devices available`, in order: (name, udid).
fn ios_devices(list: &str) -> Vec<(String, String)> {
    let mut in_ios = false;
    let mut out = Vec::new();
    for line in list.lines() {
        if let Some(rest) = line.strip_prefix("-- ") {
            in_ios = rest.starts_with("iOS ");
            continue;
        }
        if in_ios {
            if let Some((name, udid)) = device_line(line) {
                out.push((name.to_string(), udid.to_string()));
            }
        }
    }
    out
}

fn destination(name: Option<&str>) -> Result<i32> {
    let (ok, list) = simctl(&["list", "devices", "available"])?;
    if !ok {
        eprintln!("`xcrun simctl list devices available` failed.");
        return Ok(1);
    }
    let devices = ios_devices(&list);
    let pick = name.and_then(|n| devices.iter().find(|(d, _)| d == n)).or(devices.first());
    match pick {
        Some((_, udid)) => {
            println!("platform=iOS Simulator,id={udid}");
            Ok(0)
        }
        None => {
            eprintln!("No available iOS Simulator found. Create one, or pass the destination yourself.");
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "== Devices ==\n\
        -- iOS 26.0 --\n    \
        iPhone 17 (2) (11111111-2222-3333-4444-555555555555) (Shutdown)\n    \
        Lucilius P004 Delivery Oracle 2 (AAAAAAAA-2222-3333-4444-555555555555) (Shutdown)\n    \
        Lucilius P004 Delivery Oracle (BBBBBBBB-2222-3333-4444-555555555555) (Booted)\n\
        -- watchOS 26.0 --\n    \
        Watch (CCCCCCCC-2222-3333-4444-555555555555) (Shutdown)\n";

    #[test]
    fn reads_ios_devices_only() {
        let d = ios_devices(LIST);
        assert_eq!(d.len(), 3);
        assert_eq!(d[0], ("iPhone 17 (2)".into(), "11111111-2222-3333-4444-555555555555".into()));
        assert_eq!(d[2].0, "Lucilius P004 Delivery Oracle");
    }
}
