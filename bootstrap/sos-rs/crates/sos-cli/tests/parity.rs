// parity.rs — P077a parity-harness skeleton, flipped to a real gate by P077c1.
//
// Diffs the Rust `sos` binary's output for new/adopt/map/sync against the frozen
// Bash golden oracle (tests/golden/*.golden, see tests/README.md).
//
// P077a/b left this INFORMATIONAL (`HARD_FAIL: bool = false`, always PASS).
// P077c1 flips it to a per-command hard-fail SET: a command enters
// `PARITY_ENFORCED` once its Rust impl is proven to match the Bash oracle
// bug-for-bug. `map` is the first (and, at this phiếu, only) member.
// c2 (sync) / c3 (new) / c4 (adopt) add their own name to the slice as they land
// — no other harness rewrite needed.
//
// `map` is also the first command with a TWO-fixture oracle: the Bash
// `sos_map` writes its real work-product to a FILE (`<target>/docs/AGENT_MAP.yaml`)
// and only echoes a 1-line confirmation to stdout. `map.golden` freezes that
// stdout line; `map.agent_map.golden` (added in P077c1) freezes the file
// content. Both are diffed; a mismatch in EITHER hard-fails when the command
// is in `PARITY_ENFORCED` — otherwise the stdout-only oracle is blind to
// scan-correctness (the false-green class this harness exists to prevent).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Commands whose Rust impl has reached bug-for-bug parity with the Bash
/// oracle. Add a name here once its golden(s) are proven to match — the rest
/// of this harness (fixture build, normalize, assert) is already generic.
const PARITY_ENFORCED: &[&str] = &["map"];

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
    fs::read_to_string(golden_dir().join(name))
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

/// Same substitution rule as tests/golden/capture.sh normalize()'s target
/// abs-path -> <TARGET> line. (No <SOS_KIT_DIR>/<DATE> substitution needed
/// here — nothing this harness invokes echoes the kit dir or an embedded
/// date; kept generic anyway so future commands can extend it.)
fn normalize(raw: &str, target: &str) -> String {
    raw.replace(target, "<TARGET>")
}

/// A throwaway directory under the OS temp dir, removed on drop — no new
/// crate dependency needed (std::env::temp_dir + pid/nanos for uniqueness,
/// same idea as `mktemp -d` used by capture.sh).
struct TempFixture(PathBuf);

impl TempFixture {
    fn new(name: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("sos-parity-{name}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        TempFixture(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Build the same fixture `capture.sh` uses for `map` (routes/ + models/
/// files) so the harness diffs against the exact same input Bash was frozen
/// against.
fn build_map_fixture() -> (TempFixture, PathBuf) {
    let tmp = TempFixture::new("map");
    let target = tmp.path().join("map-fixture");
    fs::create_dir_all(target.join("src/routes")).unwrap();
    fs::create_dir_all(target.join("src/models")).unwrap();
    fs::write(target.join("src/routes/api.py"), "def h(): pass\n").unwrap();
    fs::write(target.join("src/models/user.py"), "class M: pass\n").unwrap();
    (tmp, target)
}

#[test]
fn parity_map_enforced() {
    assert!(
        PARITY_ENFORCED.contains(&"map"),
        "this test only makes sense while map is in PARITY_ENFORCED"
    );

    let (_tmp, target) = build_map_fixture();
    let target_str = target.to_string_lossy().into_owned();

    let stdout_actual = normalize(&run_rust(&["map", &target_str]), &target_str);
    let stdout_golden = read_golden("map.golden");
    assert_eq!(
        stdout_actual.trim(),
        stdout_golden.trim(),
        "map stdout mismatch vs map.golden (hard-fail — map is PARITY_ENFORCED)"
    );

    let file_path = target.join("docs/AGENT_MAP.yaml");
    let file_raw = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Rust map did not write {}: {e}", file_path.display()));
    let file_actual = normalize(&file_raw, &target_str);
    let file_golden = read_golden("map.agent_map.golden");
    assert_eq!(
        file_actual.trim(),
        file_golden.trim(),
        "map AGENT_MAP.yaml content mismatch vs map.agent_map.golden (hard-fail — map is PARITY_ENFORCED)"
    );
}

#[test]
fn parity_skeleton_informational() {
    let mut any_mismatch = false;

    for case in CASES {
        if PARITY_ENFORCED.contains(&case.name) {
            // Enforced commands have their own dedicated hard-fail test
            // (see parity_map_enforced) — this loop stays informational-only
            // for the commands that haven't reached parity yet.
            continue;
        }

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

    // Informational mode (P077a/b) for commands outside PARITY_ENFORCED:
    // always pass regardless of mismatch. Commands in PARITY_ENFORCED are
    // hard-failed by their own dedicated test above instead.
    let _ = any_mismatch;
}
