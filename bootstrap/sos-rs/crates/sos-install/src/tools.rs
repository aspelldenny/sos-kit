//! P077d3 (OA-07) — tool-manifest core: parse the kit-root
//! `tool-manifest.toml` pin, resolve installed sister-tool versions, and
//! compute a per-tool drift verdict. ONE core (`check_tools` /
//! `gate_required`), consumed by THREE surfaces: `sos tools status`
//! (`crates/sos-cli/src/commands/tools.rs`), the install-engine step-5 seam
//! (`resolve_tools()` in `engine.rs`), and a future doctor-fold IF `sos
//! doctor` ever becomes a real subcommand (today it does not — every
//! `doctor` reference in `sos-cli` is `Command::new("doctor")` shelling to
//! the EXTERNAL binary, so fail-clear surfaces via the other two paths
//! only — see `docs/discoveries/P077d3.md`).
//!
//! d3 GIỚI HẠN (scope limit): verify-only. No download / atomic-upgrade /
//! rollback of the external binaries themselves — that's P081 future.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::process::Command;

/// `tool-manifest.toml` embedded at compile time (kit root, 5 levels up
/// from this file — `crates/sos-install/src/tools.rs` ->
/// `bootstrap/sos-rs/crates/sos-install` -> kit root). Embedding (not an
/// env-var/flag lookup) is deliberate: the compiled `sos` binary is used
/// INSIDE arbitrary target projects that do not necessarily have a sos-kit
/// checkout available at runtime, so the pin must travel WITH the binary,
/// not be resolved relative to wherever it happens to run
/// (`[needs Worker verify]` anchor resolved this way — see Discovery).
const EMBEDDED_TOOL_MANIFEST: &str =
    include_str!("../../../../../tool-manifest.toml");

/// One pinned sister-tool entry — mirrors `tool-manifest.toml`'s `[[tool]]`
/// array-of-tables shape exactly (Task 1 CHỐT schema).
#[derive(Debug, Clone, Deserialize)]
pub struct ToolPin {
    pub name: String,
    pub version: String,
    pub required: bool,
    #[serde(default)]
    pub asset: BTreeMap<String, String>,
    #[serde(default)]
    pub checksum: BTreeMap<String, String>,
}

/// Whole-file shape: `tool-manifest.toml` is `[[tool]]` at the top level,
/// so the wrapper struct's field is renamed `tool` (serde name) but kept
/// as `tools` (Rust name) for readability at call sites.
#[derive(Debug, Clone, Deserialize)]
pub struct ToolManifest {
    #[serde(rename = "tool")]
    pub tools: Vec<ToolPin>,
}

/// Parse a `tool-manifest.toml`-shaped TOML string. Pure — no I/O beyond
/// the string already in hand.
pub fn parse_manifest(content: &str) -> Result<ToolManifest> {
    toml::from_str(content).context("parse tool-manifest.toml")
}

/// A checksum cell is "honest" if it's either the documented placeholder
/// (contains "TODO" — E2, no prebuilt asset hashed against this manifest
/// shape yet) or a real-looking sha256 hex digest (64 hex chars). Anything
/// else (garbage, truncated, wrong charset) is a malformed manifest —
/// fail loud rather than silently accept it (Task 6 "sabotage checksum ->
/// fail loud").
pub fn checksum_looks_valid(value: &str) -> bool {
    value.to_uppercase().contains("TODO")
        || (value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Schema-integrity check beyond bare TOML-parseability: every checksum
/// cell must be either a real-looking sha256 or the documented TODO
/// placeholder. This is the honest limit of "checksum verify" d3 can do
/// without a real prebuilt asset to hash (E2) — it catches a corrupted/
/// garbage manifest, it does NOT verify an installed binary's bytes
/// against a real digest (no download happens in d3, see module doc).
pub fn validate_manifest(manifest: &ToolManifest) -> Result<()> {
    for t in &manifest.tools {
        if t.name.trim().is_empty() {
            bail!("tool-manifest.toml: a [[tool]] entry has an empty name");
        }
        for (platform, csum) in &t.checksum {
            if !checksum_looks_valid(csum) {
                bail!(
                    "tool-manifest.toml: {} / {} has a malformed checksum value: {csum:?} \
                     (expected 64-hex sha256 or a TODO placeholder)",
                    t.name,
                    platform
                );
            }
        }
    }
    Ok(())
}

/// Load + parse + validate the EMBEDDED (compiled-in) tool-manifest — the
/// production entry point used by both `sos tools status` and the
/// install-engine step-5 seam.
pub fn embedded_manifest_parsed() -> Result<ToolManifest> {
    let manifest = parse_manifest(EMBEDDED_TOOL_MANIFEST)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

/// Per-tool drift verdict, one of 5 (Task 2 spec).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Installed version == pinned version.
    Ok,
    /// Installed version > pinned version (newer than the pin — allowed,
    /// warn-optional only, never fails required).
    Newer,
    /// Installed version < pinned version (older — the OA-07 case).
    Drift,
    /// Tool not found on PATH at all.
    Missing,
    /// Installed but its `--version` output couldn't be parsed as a
    /// dotted numeric version AND didn't exact-string-match the pin (E1).
    Unparseable,
}

/// Resolved status for one manifest tool entry.
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub name: String,
    pub required: bool,
    pub expected: String,
    /// `None` only when `Verdict::Missing`.
    pub found: Option<String>,
    pub verdict: Verdict,
}

/// Parse a dotted numeric version ("0.1.3" -> [0,1,3]). Returns `None` on
/// any non-numeric segment. This is a deliberately tiny hand-rolled
/// comparator (anchor #8: no `semver` crate anywhere in the workspace) —
/// `Vec<u64>`'s derived `Ord` gives correct dotted-version ordering for
/// same-shaped x.y.z strings without adding a new dependency.
fn parse_version_tuple(v: &str) -> Option<Vec<u64>> {
    v.split('.').map(|p| p.parse::<u64>().ok()).collect()
}

fn compare_versions(installed: &str, expected: &str) -> Verdict {
    if installed == expected {
        return Verdict::Ok;
    }
    match (parse_version_tuple(installed), parse_version_tuple(expected)) {
        (Some(i), Some(e)) => {
            if i > e {
                Verdict::Newer
            } else {
                Verdict::Drift
            }
        }
        // Can't parse one side numerically but strings differ -> honest
        // fallback is exact-string-mismatch = Drift (no `semver` crate to
        // fall back on, anchor #8), UNLESS the installed string genuinely
        // doesn't look version-shaped at all (E1) -> Unparseable.
        _ => {
            if parse_version_tuple(installed).is_none() {
                Verdict::Unparseable
            } else {
                Verdict::Drift
            }
        }
    }
}

/// Run `<name> --version` and parse the LAST whitespace-separated token as
/// the version string (E1-confirmed uniform format across all 9
/// live-tested sos-kit-authored tools: `"<name> <x.y.z>"`, no `v` prefix).
///
/// `path_override`, when `Some`, replaces PATH for JUST this child-process
/// invocation (not the calling test/process's own env) — this is what
/// lets tests point at a fake-PATH fixture dir deterministically, without
/// mutating global process state shared across parallel `#[test]`
/// threads (a real `env::set_var("PATH", ..)` in one test would race any
/// concurrently-running test in the same binary).
fn resolve_installed_version(name: &str, path_override: Option<&str>) -> Option<String> {
    let mut cmd = Command::new(name);
    cmd.arg("--version");
    if let Some(path) = path_override {
        cmd.env("PATH", path);
    }
    let output = cmd.output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = if !stdout.trim().is_empty() {
        stdout.to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.split_whitespace().last().map(|s| s.to_string())
}

/// Resolve every tool in `manifest` against the current environment. Pure
/// data-in/data-out aside from the `<tool> --version` subprocess calls
/// (side-effect I/O, no filesystem mutation).
pub fn check_tools(manifest: &ToolManifest, path_override: Option<&str>) -> Vec<ToolStatus> {
    manifest
        .tools
        .iter()
        .map(|t| match resolve_installed_version(&t.name, path_override) {
            None => ToolStatus {
                name: t.name.clone(),
                required: t.required,
                expected: t.version.clone(),
                found: None,
                verdict: Verdict::Missing,
            },
            Some(found) => {
                let verdict = compare_versions(&found, &t.version);
                ToolStatus {
                    name: t.name.clone(),
                    required: t.required,
                    expected: t.version.clone(),
                    found: Some(found),
                    verdict,
                }
            }
        })
        .collect()
}

/// True iff any REQUIRED tool's verdict is `Drift` / `Missing` /
/// `Unparseable` — the fail-closed condition. Optional tools never flip
/// this regardless of verdict (warn-only, Task 3 exit-code CHỐT).
pub fn required_drift(statuses: &[ToolStatus]) -> bool {
    statuses.iter().any(|s| {
        s.required
            && matches!(s.verdict, Verdict::Drift | Verdict::Missing | Verdict::Unparseable)
    })
}

/// Human-readable one-liner for a failing REQUIRED tool, shared by both
/// `sos tools status`'s stderr lines and the install step-5 bail message
/// (Task 4 message-shape CHỐT: "tool + expected + found").
pub fn describe_failure(status: &ToolStatus) -> String {
    let found = status.found.clone().unwrap_or_else(|| "MISSING".to_string());
    let verb = match status.verdict {
        Verdict::Missing => "MISSING",
        Verdict::Drift => "DRIFT (older than pinned)",
        Verdict::Unparseable => "UNPARSEABLE version output",
        Verdict::Newer | Verdict::Ok => "unexpected-ok", // unreachable via required_drift callers
    };
    format!(
        "{}: required, {verb} (expected {}, found {found})",
        status.name, status.expected
    )
}

/// Core fail-clear gate shared by BOTH the install-engine step-5 seam
/// (`engine::resolve_tools()`) and, if ever needed, a future doctor-fold —
/// resolves `manifest`, and returns `Err` with a message naming every
/// failing REQUIRED tool if `required_drift` fires. `Ok(statuses)`
/// otherwise (callers may still want the full status list, e.g. for a
/// verbose print).
pub fn gate_required(manifest: &ToolManifest, path_override: Option<&str>) -> Result<Vec<ToolStatus>> {
    let statuses = check_tools(manifest, path_override);
    if required_drift(&statuses) {
        let lines: Vec<String> = statuses
            .iter()
            .filter(|s| {
                s.required && matches!(s.verdict, Verdict::Drift | Verdict::Missing | Verdict::Unparseable)
            })
            .map(describe_failure)
            .collect();
        bail!(
            "required tool(s) failed the tool-manifest.toml pin check:\n  {}",
            lines.join("\n  ")
        );
    }
    Ok(statuses)
}
