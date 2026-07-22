// parity.rs — P077a parity-harness skeleton.
//
// Diffs the Rust `sos` binary's output for new/adopt/map/sync against the frozen
// Bash golden oracle (tests/golden/*.golden, see tests/README.md). Rust does not
// implement these subcommands yet (P077c's job) — this harness stays INFORMATIONAL:
// it always PASSES, but prints "not yet parity" per command so a human (or CI log)
// can see the gap without the build going red.
//
// P077c flips HARD_FAIL to true once Rust reaches parity — no other rewrite needed.

use std::path::PathBuf;
use std::process::Command;

/// Single switch P077c flips to make this harness a real gate.
const HARD_FAIL: bool = false;

struct Case {
    name: &'static str,
    args: &'static [&'static str],
    golden: &'static str,
}

const CASES: &[Case] = &[
    Case { name: "new",   args: &["new", "/tmp/parity-fixture-new", "--stack", "python"], golden: "new.golden" },
    Case { name: "adopt", args: &["adopt", "/tmp/parity-fixture-adopt", "--stack", "python"], golden: "adopt.golden" },
    Case { name: "map",   args: &["map", "/tmp/parity-fixture-map"], golden: "map.golden" },
    Case { name: "sync",  args: &["sync", "/tmp/parity-fixture-sync"], golden: "sync.golden" },
];

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn read_golden(name: &str) -> String {
    std::fs::read_to_string(golden_dir().join(name))
        .unwrap_or_else(|e| panic!("missing golden fixture {name}: {e}"))
}

fn run_rust(args: &[&str]) -> String {
    let bin = env!("CARGO_BIN_EXE_sos");
    let out = Command::new(bin)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to exec {bin}: {e}"));
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    combined
}

#[test]
fn parity_skeleton_informational() {
    let mut any_mismatch = false;

    for case in CASES {
        let golden = read_golden(case.golden);
        let actual = run_rust(case.args);

        if actual.trim() == golden.trim() {
            println!("P077a: {} — PARITY (Rust output matches Bash golden)", case.name);
        } else {
            any_mismatch = true;
            eprintln!(
                "P077a: {} not yet parity — Rust command unimplemented/differs from Bash golden ({})",
                case.name, case.golden
            );
        }
    }

    if HARD_FAIL {
        assert!(!any_mismatch, "parity harness: mismatches found (hard-fail mode, see stderr above)");
    }
    // Informational mode (P077a/b): always pass regardless of mismatch.
}
