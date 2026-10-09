//! P077d3 (OA-07) — tool-manifest oracle. NOT a Bash-parity fixture (`sos
//! tools status` is a NEW command, no Bash counterpart). Oracle = 3
//! hard-fail correctness fixtures against SYNTHETIC tools/manifests —
//! never depends on a real sister tool being installed on the CI/dev
//! machine (fake-PATH stub-script harness, collision-safe `TempFixture`,
//! mirrors `tests/install.rs`'s pattern for anchor #8 CHALLENGE
//! precedent).

use sos_install::tools::{self, Verdict};
use std::fs;
use std::path::{Path, PathBuf};

// --- Collision-safe TempFixture (replicated from tests/install.rs, which
// itself replicates crates/sos-cli/tests/parity.rs — no shared test-util
// crate exists today). ---
static TEMP_FIXTURE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct TempFixture(PathBuf);

impl TempFixture {
    fn new(name: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEMP_FIXTURE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "sos-install-tools-{name}-{}-{nanos}-{counter}",
            std::process::id()
        ));
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

/// Write an executable `#!/bin/sh` stub named `name` into `dir` that
/// prints `"<name> <version>"` on `--version` (matches the E1-confirmed
/// live format of every real sos-kit-authored tool: no `v` prefix, single
/// space, last-whitespace-token = version).
#[cfg(unix)]
fn write_stub_tool(dir: &Path, name: &str, version: &str) {
    use std::os::unix::fs::PermissionsExt;
    let script_path = dir.join(name);
    fs::write(
        &script_path,
        format!("#!/bin/sh\necho \"{name} {version}\"\n"),
    )
    .unwrap();
    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();
}

fn synthetic_manifest_toml(checksum_cell: &str) -> String {
    format!(
        r#"
[[tool]]
name = "fake-tool"
version = "1.2.0"
required = true

[tool.asset]
"aarch64-apple-darwin" = "fake-tool-aarch64-apple-darwin"

[tool.checksum]
"aarch64-apple-darwin" = "{checksum_cell}"

[[tool]]
name = "fake-optional"
version = "2.0.0"
required = false

[tool.asset]
"aarch64-apple-darwin" = "fake-optional-aarch64-apple-darwin"

[tool.checksum]
"aarch64-apple-darwin" = "{checksum_cell}"
"#
    )
}

// ── Fixture 1: manifest-pin-verify ──────────────────────────────────────

#[test]
fn manifest_pin_verify_round_trips_and_checks_pinned_fields() {
    let toml_str = synthetic_manifest_toml("TODO-sha256-fake-tool-P081");
    let manifest = tools::parse_manifest(&toml_str).expect("synthetic manifest must parse");
    assert_eq!(manifest.tools.len(), 2);

    let fake_tool = manifest.tools.iter().find(|t| t.name == "fake-tool").unwrap();
    assert_eq!(fake_tool.version, "1.2.0");
    assert!(fake_tool.required);
    assert_eq!(
        fake_tool.asset.get("aarch64-apple-darwin").unwrap(),
        "fake-tool-aarch64-apple-darwin"
    );
    assert_eq!(
        fake_tool.checksum.get("aarch64-apple-darwin").unwrap(),
        "TODO-sha256-fake-tool-P081"
    );

    let fake_optional = manifest.tools.iter().find(|t| t.name == "fake-optional").unwrap();
    assert!(!fake_optional.required);

    // Honest placeholder passes the schema-integrity check (E2 — no real
    // prebuilt hash to verify against yet, contains "TODO").
    tools::validate_manifest(&manifest).expect("TODO placeholder checksum must validate as honest");

    // A real-looking 64-hex checksum also validates.
    let real_hex = "a".repeat(64);
    let toml_real = synthetic_manifest_toml(&real_hex);
    let manifest_real = tools::parse_manifest(&toml_real).unwrap();
    tools::validate_manifest(&manifest_real).expect("64-hex checksum must validate");
}

#[test]
fn manifest_pin_verify_sabotaged_checksum_fails_loud() {
    // Sabotage: neither a TODO placeholder nor a real 64-hex digest —
    // garbage. Must fail LOUD (Err), not silently accept a corrupted
    // manifest (Task 6 requirement).
    let toml_str = synthetic_manifest_toml("garbage-value-clearly-invalid");
    let manifest = tools::parse_manifest(&toml_str).expect("still parses as TOML — sabotage is semantic, not syntactic");
    let result = tools::validate_manifest(&manifest);
    assert!(result.is_err(), "sabotaged checksum must fail validate_manifest loudly");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("fake-tool"), "error must name the offending tool");

    // Revert equivalent: an honest manifest (built fresh, not mutated
    // in-place) validates clean — proves the check isn't permanently wedged.
    let honest = tools::parse_manifest(&synthetic_manifest_toml("TODO-honest")).unwrap();
    tools::validate_manifest(&honest).expect("honest manifest must validate after sabotage test");
}

// ── Fixture 2: status-drift ─────────────────────────────────────────────

#[test]
#[cfg(unix)]
fn status_drift_detects_older_installed_required_tool() {
    let fixture = TempFixture::new("status-drift");
    write_stub_tool(fixture.path(), "fake-tool", "1.0.0"); // older than pinned 1.2.0
    write_stub_tool(fixture.path(), "fake-optional", "1.0.0"); // older than pinned 2.0.0 (optional)

    let manifest = tools::parse_manifest(&synthetic_manifest_toml("TODO-sha256-fake-tool-P081")).unwrap();
    let path_override = fixture.path().to_string_lossy().to_string();
    let statuses = tools::check_tools(&manifest, Some(&path_override));

    let fake_tool = statuses.iter().find(|s| s.name == "fake-tool").unwrap();
    assert_eq!(fake_tool.verdict, Verdict::Drift, "installed 1.0.0 < pinned 1.2.0 must be Drift");
    assert_eq!(fake_tool.found.as_deref(), Some("1.0.0"));

    let fake_optional = statuses.iter().find(|s| s.name == "fake-optional").unwrap();
    assert_eq!(fake_optional.verdict, Verdict::Drift, "optional tool still gets a real verdict computed");

    assert!(
        tools::required_drift(&statuses),
        "required-tool drift must flip required_drift() true"
    );
}

#[test]
#[cfg(unix)]
fn status_drift_missing_required_tool_flips_required_drift_missing_optional_does_not() {
    let fixture = TempFixture::new("status-missing");
    // Neither stub written — both tools absent from the fake-PATH dir.

    let manifest = tools::parse_manifest(&synthetic_manifest_toml("TODO-sha256-fake-tool-P081")).unwrap();
    let path_override = fixture.path().to_string_lossy().to_string();
    let statuses = tools::check_tools(&manifest, Some(&path_override));

    let fake_tool = statuses.iter().find(|s| s.name == "fake-tool").unwrap();
    assert_eq!(fake_tool.verdict, Verdict::Missing);
    assert!(fake_tool.found.is_none());
    assert!(tools::required_drift(&statuses), "required Missing must flip required_drift() true");

    // Now only the OPTIONAL tool is missing (required present+matching) —
    // required_drift() must stay false (warn-only for optional).
    let fixture2 = TempFixture::new("status-optional-missing");
    write_stub_tool(fixture2.path(), "fake-tool", "1.2.0"); // exact match
    let path_override2 = fixture2.path().to_string_lossy().to_string();
    let statuses2 = tools::check_tools(&manifest, Some(&path_override2));
    let fake_tool2 = statuses2.iter().find(|s| s.name == "fake-tool").unwrap();
    assert_eq!(fake_tool2.verdict, Verdict::Ok);
    let fake_optional2 = statuses2.iter().find(|s| s.name == "fake-optional").unwrap();
    assert_eq!(fake_optional2.verdict, Verdict::Missing);
    assert!(
        !tools::required_drift(&statuses2),
        "optional Missing alone must NOT flip required_drift() (warn-only, exit 0)"
    );
}

// ── Fixture 3: doctor / step-5 fail-clear ───────────────────────────────

#[test]
#[cfg(unix)]
fn gate_required_fails_loud_on_required_missing_with_tool_expected_found_message() {
    let fixture = TempFixture::new("gate-missing");
    // fake-tool absent -> Missing -> gate_required must Err.

    let manifest = tools::parse_manifest(&synthetic_manifest_toml("TODO-sha256-fake-tool-P081")).unwrap();
    let path_override = fixture.path().to_string_lossy().to_string();
    let result = tools::gate_required(&manifest, Some(&path_override));

    assert!(result.is_err(), "required tool Missing must fail the gate");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("fake-tool"), "message must name the tool");
    assert!(msg.contains("expected 1.2.0"), "message must show expected version");
    assert!(msg.contains("MISSING"), "message must show found=MISSING");
}

#[test]
#[cfg(unix)]
fn gate_required_fails_loud_on_required_older_and_passes_when_pinned() {
    // Older required -> fail.
    let fixture_old = TempFixture::new("gate-older");
    write_stub_tool(fixture_old.path(), "fake-tool", "1.0.0");
    let manifest = tools::parse_manifest(&synthetic_manifest_toml("TODO-sha256-fake-tool-P081")).unwrap();
    let path_old = fixture_old.path().to_string_lossy().to_string();
    let result_old = tools::gate_required(&manifest, Some(&path_old));
    assert!(result_old.is_err(), "required tool older than pinned must fail the gate");
    let msg = result_old.unwrap_err().to_string();
    assert!(msg.contains("fake-tool"));
    assert!(msg.contains("expected 1.2.0"));
    assert!(msg.contains("found 1.0.0"));

    // Exact-pinned required + optional missing -> gate passes (Ok).
    let fixture_ok = TempFixture::new("gate-ok");
    write_stub_tool(fixture_ok.path(), "fake-tool", "1.2.0");
    let path_ok = fixture_ok.path().to_string_lossy().to_string();
    let result_ok = tools::gate_required(&manifest, Some(&path_ok));
    assert!(
        result_ok.is_ok(),
        "required tool exactly at pin + optional missing must pass gate_required (fail-closed reserved for required)"
    );
}
