// P077b — dependency-direction gate (guard test layer).
//
// sos-core must stay host-neutral: it owns semantic (state/policy/roles/
// workflow/config schema) and MUST NOT import any adapter/install/hooks/cli
// crate. Two-layer defense:
//   (a) compiler graph — sos-core/Cargo.toml declares ZERO dep on those
//       crates, so `use sos_adapter_*` etc. would be a compile error
//       (structural primary, see docs/discoveries/P077b.md negative-test).
//   (b) this guard test — catches regression where the compiler stays
//       quiet because someone forgot to add the source usage but DID add
//       the dependency to Cargo.toml (or vice versa drifted).
//
// Scans crates/sos-core/src/**/*.rs for forbidden import tokens.

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_TOKENS: &[&str] = &["sos_adapter", "sos_install", "sos_hooks", "sos_cli"];

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read_dir src") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(path);
        }
    }
}

#[test]
fn sos_core_stays_host_neutral() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);
    assert!(!files.is_empty(), "expected sos-core/src to contain .rs files");

    let mut violations = Vec::new();
    for file in &files {
        let content = fs::read_to_string(file).expect("read src file");
        for token in FORBIDDEN_TOKENS {
            if content.contains(token) {
                violations.push(format!("{}: forbidden token `{token}`", file.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "sos-core must not reference adapter/install/hooks/cli crates:\n{}",
        violations.join("\n")
    );
}
