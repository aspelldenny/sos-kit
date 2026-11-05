//! `sos sim boot|settle|destination` against a fake `xcrun` and `sysctl` on PATH (wave 2.5).
//! The fakes record every call in calls.log and answer from files the test writes, so each case
//! states what simctl says and checks what sos did with it.
#![cfg(unix)]
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const LIST: &str = "== Devices ==
-- iOS 26.0 --
    iPhone 17 (2) (11111111-2222-3333-4444-555555555555) (Shutdown)
    Lucilius P004 Delivery Oracle 2 (AAAAAAAA-2222-3333-4444-555555555555) (Shutdown)
    Lucilius P004 Delivery Oracle (BBBBBBBB-2222-3333-4444-555555555555) (Booted)
-- watchOS 26.0 --
    Watch (CCCCCCCC-2222-3333-4444-555555555555) (Shutdown)
";

struct Fake {
    dir: tempfile::TempDir,
}

impl Fake {
    /// A bin/ with `xcrun` and `sysctl`. xcrun: `simctl list …` prints list.txt (exit list.rc),
    /// `simctl boot` exits boot.rc, `simctl bootstatus` exits bootstatus.rc. sysctl prints the
    /// next line of loads.txt (the last one again when they run out); an empty line fails.
    fn new() -> Fake {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        fs::create_dir(&bin).unwrap();
        let d = dir.path().display();
        let xcrun = format!(r#"#!/bin/sh
echo "xcrun $*" >> "{d}/calls.log"
case "$2" in
  list) cat "{d}/list.txt" 2>/dev/null; exit $(cat "{d}/list.rc" 2>/dev/null || echo 0) ;;
  boot) exit $(cat "{d}/boot.rc" 2>/dev/null || echo 0) ;;
  bootstatus) echo "Device booted"; exit $(cat "{d}/bootstatus.rc" 2>/dev/null || echo 0) ;;
esac
exit 64
"#);
        let sysctl = format!(r#"#!/bin/sh
echo "sysctl $*" >> "{d}/calls.log"
n=$(cat "{d}/n" 2>/dev/null || echo 0); n=$((n + 1)); echo $n > "{d}/n"
total=$(wc -l < "{d}/loads.txt")
[ "$n" -gt "$total" ] && n=$total
l=$(sed -n "${{n}}p" "{d}/loads.txt")
[ -z "$l" ] && exit 1
echo "{{ $l 1.00 1.00 }}"
"#);
        for (name, body) in [("xcrun", xcrun), ("sysctl", sysctl)] {
            let p = bin.join(name);
            fs::write(&p, body).unwrap();
            fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Fake { dir }
    }
    fn path(&self) -> &Path {
        self.dir.path()
    }
    fn put(&self, name: &str, body: &str) {
        fs::write(self.path().join(name), body).unwrap();
    }
    fn calls(&self) -> String {
        fs::read_to_string(self.path().join("calls.log")).unwrap_or_default()
    }
    fn log(&self) -> String {
        fs::read_to_string(self.path().join("settle.log")).unwrap_or_default()
    }
    /// (exit code, stdout, stderr) of `sos sim <args>` with only the fakes and /bin, /usr/bin on PATH.
    fn sos(&self, args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
        let path = format!("{}:/usr/bin:/bin", self.path().join("bin").display());
        let mut c = Command::new(env!("CARGO_BIN_EXE_sos"));
        c.arg("sim").args(args).current_dir(self.path()).env("PATH", path);
        for v in ["SETTLE_LOAD", "SETTLE_MAX", "SETTLE_INTERVAL", "SETTLE_RUN"] {
            c.env_remove(v);
        }
        for (k, v) in env {
            c.env(k, v);
        }
        let o = c.output().unwrap();
        (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
    }
}

fn log_arg(f: &Fake) -> PathBuf {
    f.path().join("settle.log")
}

// ── boot ──

#[test]
fn boot_skips_a_booted_device_and_waits_for_status() {
    let f = Fake::new();
    f.put("list.txt", LIST);
    let log = log_arg(&f);
    let (rc, _, err) = f.sos(&["boot", "BBBBBBBB-2222-3333-4444-555555555555", "--log", log.to_str().unwrap()], &[]);
    assert_eq!(rc, 0, "{err}");
    let calls = f.calls();
    assert!(!calls.contains("simctl boot "), "{calls}");
    assert!(calls.contains("xcrun simctl bootstatus BBBBBBBB-2222-3333-4444-555555555555 -b"), "{calls}");
    assert!(f.log().contains("already booted"), "{}", f.log());
}

#[test]
fn boot_boots_only_the_named_device() {
    let f = Fake::new();
    f.put("list.txt", LIST);
    let log = log_arg(&f);
    let (rc, _, err) = f.sos(&["boot", "AAAAAAAA-2222-3333-4444-555555555555", "--log", log.to_str().unwrap()], &[]);
    assert_eq!(rc, 0, "{err}");
    let calls = f.calls();
    assert_eq!(calls.matches("simctl boot ").count(), 1, "{calls}");
    assert!(calls.contains("xcrun simctl boot AAAAAAAA-2222-3333-4444-555555555555\n"), "{calls}");
    // Never creates, clones, erases or shuts down anything.
    for verb in ["create", "clone", "erase", "shutdown", "delete"] {
        assert!(!calls.contains(&format!("simctl {verb}")), "{verb}: {calls}");
    }
    let log = f.log();
    assert!(log.contains("booting AAAAAAAA") && log.contains("boot status complete"), "{log}");
}

#[test]
fn boot_reports_simctl_failures() {
    let f = Fake::new();
    f.put("list.txt", LIST);
    f.put("boot.rc", "149");
    let log = log_arg(&f);
    let (rc, out, _) = f.sos(&["boot", "AAAAAAAA-2222-3333-4444-555555555555", "--log", log.to_str().unwrap()], &[]);
    assert_eq!(rc, 1);
    assert!(out.contains("⛔ simctl boot AAAAAAAA-2222-3333-4444-555555555555 failed"), "{out}");
    assert!(!f.calls().contains("bootstatus"));

    f.put("boot.rc", "0");
    f.put("bootstatus.rc", "1");
    let (rc, out, _) = f.sos(&["boot", "AAAAAAAA-2222-3333-4444-555555555555", "--log", log.to_str().unwrap()], &[]);
    assert_eq!(rc, 1);
    assert!(out.contains("⛔ simctl bootstatus"), "{out}");
}

#[test]
fn boot_still_boots_when_the_list_fails() {
    // A failed `simctl list` proves nothing about the state, even if its output says Booted.
    let f = Fake::new();
    f.put("list.txt", LIST);
    f.put("list.rc", "1");
    let log = log_arg(&f);
    let (rc, _, err) = f.sos(&["boot", "BBBBBBBB-2222-3333-4444-555555555555", "--log", log.to_str().unwrap()], &[]);
    assert_eq!(rc, 0, "{err}");
    let calls = f.calls();
    let boot = calls.find("xcrun simctl boot BBBBBBBB-2222-3333-4444-555555555555\n").expect(&calls);
    let status = calls.find("xcrun simctl bootstatus BBBBBBBB").expect(&calls);
    assert!(boot < status, "{calls}");
}

#[test]
fn boot_log_defaults_to_build_folder() {
    let f = Fake::new();
    f.put("list.txt", LIST);
    let (rc, _, err) = f.sos(&["boot", "BBBBBBBB-2222-3333-4444-555555555555"], &[]);
    assert_eq!(rc, 0, "{err}");
    let log = fs::read_to_string(f.path().join("build/uitest-settle.log")).unwrap();
    assert!(log.contains("[boot] BBBBBBBB-2222-3333-4444-555555555555 already booted"), "{log}");
}

// ── settle ──

#[test]
fn settle_waits_for_a_run_of_low_samples() {
    let f = Fake::new();
    // A dip (2.0) between high readings does not count; two lows in a row do.
    f.put("loads.txt", "9.5\n2.0\n8.0\n3.0\n2.5\n");
    let log = log_arg(&f);
    let (rc, out, err) = f.sos(&["settle", "uitest", "--log", log.to_str().unwrap(), "--interval", "0"], &[]);
    assert_eq!(rc, 0, "{err}");
    assert!(out.starts_with("settle [uitest]: load 2.5 after "), "{out}");
    assert_eq!(f.calls().matches("sysctl -n vm.loadavg").count(), 5);
    let log = f.log();
    assert!(log.contains("[uitest] +0s load1=9.5 under=0/2"), "{log}");
    assert!(log.contains("load1=2.0 under=1/2") && log.contains("load1=8.0 under=0/2"), "{log}");
    assert!(log.contains("[uitest] settled after"), "{log}");
}

#[test]
fn settle_options_come_from_flags_then_env() {
    let f = Fake::new();
    f.put("loads.txt", "4.0\n");
    let log = log_arg(&f);
    // Threshold 3.0 from the environment: 4.0 is too high, and a 0 s limit gives up at once.
    let (rc, out, _) = f.sos(&["settle", "env", "--log", log.to_str().unwrap()],
        &[("SETTLE_LOAD", "3.0"), ("SETTLE_MAX", "0"), ("SETTLE_INTERVAL", "0")]);
    assert_eq!(rc, 0);
    assert!(out.contains("⚠️  settle [env]: load still 4.0 after 0s (threshold 3); continuing"), "{out}");
    // The flag wins over the environment.
    let (rc, out, _) = f.sos(&["settle", "flag", "--log", log.to_str().unwrap(), "--load", "5", "--run", "1"],
        &[("SETTLE_LOAD", "3.0"), ("SETTLE_MAX", "0"), ("SETTLE_INTERVAL", "0")]);
    assert_eq!(rc, 0);
    assert!(out.contains("settle [flag]: load 4.0 after 0s"), "{out}");
    // A bad number is an error, not a silent default.
    let (rc, _, err) = f.sos(&["settle", "bad", "--log", log.to_str().unwrap()], &[("SETTLE_LOAD", "high")]);
    assert_eq!(rc, 2);
    assert!(err.contains("SETTLE_LOAD=high is not a number"), "{err}");
}

#[test]
fn settle_unreadable_load_never_counts_as_settled() {
    let f = Fake::new();
    f.put("loads.txt", "\n");
    let log = log_arg(&f);
    let (rc, out, _) = f.sos(&["settle", "blind", "--log", log.to_str().unwrap(), "--max", "0", "--interval", "0", "--run", "1"], &[]);
    assert_eq!(rc, 0);
    assert!(out.contains("⚠️  settle [blind]: load still ?"), "{out}");
    assert!(f.log().contains("load1=? under=0/1"), "{}", f.log());
    // sysctl succeeds but the number is not a load.
    for bad in ["-inf", "nan", "-1.0", "abc"] {
        let f = Fake::new();
        f.put("loads.txt", &format!("{bad}\n"));
        let log = log_arg(&f);
        let (rc, out, _) = f.sos(&["settle", "probe", "--log", log.to_str().unwrap(), "--max", "0", "--interval", "0", "--run", "1"], &[]);
        assert_eq!(rc, 0);
        assert!(out.contains("⚠️  settle [probe]: load still ?"), "{bad}: {out}");
        assert!(f.log().contains("load1=? under=0/1"), "{bad}: {}", f.log());
    }
}

// ── destination ──

#[test]
fn destination_prefers_the_exact_name() {
    let f = Fake::new();
    f.put("list.txt", LIST);
    let (rc, out, err) = f.sos(&["destination", "--name", "Lucilius P004 Delivery Oracle"], &[]);
    assert_eq!(rc, 0, "{err}");
    // Not the "… Oracle 2" listed before it, which the old prefix match picked.
    assert_eq!(out, "platform=iOS Simulator,id=BBBBBBBB-2222-3333-4444-555555555555\n");
    assert!(f.calls().contains("xcrun simctl list devices available"));
}

#[test]
fn destination_falls_back_to_the_first_ios_device() {
    let f = Fake::new();
    f.put("list.txt", LIST);
    for args in [&["destination", "--name", "No Such Phone"][..], &["destination"][..]] {
        let (rc, out, _) = f.sos(args, &[]);
        assert_eq!(rc, 0);
        // The name holds its own parentheses; the UDID is still found.
        assert_eq!(out, "platform=iOS Simulator,id=11111111-2222-3333-4444-555555555555\n");
    }
}

#[test]
fn destination_without_ios_devices_fails() {
    let f = Fake::new();
    f.put("list.txt", "== Devices ==\n-- watchOS 26.0 --\n    Watch (CCCCCCCC-2222-3333-4444-555555555555) (Shutdown)\n");
    let (rc, out, err) = f.sos(&["destination"], &[]);
    assert_eq!((rc, out.as_str()), (1, ""));
    assert!(err.contains("No available iOS Simulator"), "{err}");

    f.put("list.rc", "1");
    let (rc, _, err) = f.sos(&["destination"], &[]);
    assert_eq!(rc, 1);
    assert!(err.contains("failed"), "{err}");
}
