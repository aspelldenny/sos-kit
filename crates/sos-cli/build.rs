// Records the sos-kit commit the binary was built from (written to UPSTREAM.json on install).
use std::process::Command;

fn main() {
    let commit = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    let dirty = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    println!("cargo:rustc-env=SOS_KIT_COMMIT={commit}{}", if dirty { "-dirty" } else { "" });
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/index");
}
