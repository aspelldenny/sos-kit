//! P077d1 — runtime-adapter contract (foundation abstraction).
//!
//! Defines the 5-method contract every runtime adapter (Claude, Codex, ...)
//! must implement (`docs/PORTABILITY_ARCHITECTURE.md:63-69`):
//!
//! ```text
//! detect() → capabilities
//! plan(core, project) → managed file/tool/hook operations
//! render(asset, capabilities) → runtime-native artifact
//! verify(project) → findings
//! uninstall(manifest) → safe removal plan
//! ```
//!
//! Host-NEUTRAL: every type here is a plain, adapter-agnostic placeholder.
//! ZERO host token (`.claude`, `CLAUDE_*`, Codex, or a forbidden sos
//! crate-name import — see `tests/dep_direction.rs`'s token list) may
//! appear in this module — enforced by the dep-direction guard which
//! scans `sos-core/src/**` for those forbidden crate-name tokens, plus
//! manual review per `core/POLICY.md` "Portable core" (zero runtime
//! token).
//!
//! Engine bodies (real detection/planning/rendering/removal logic) land in
//! P077d2. This module only carves the SHAPE — signatures and return types
//! — so `sos-adapter-claude` (and later a Codex adapter, P078) can
//! implement one shared contract without refactor-churn.

use crate::manifest::ManagedManifest;

/// What the target project/environment currently supports — output of
/// `detect()`. Runtime-neutral: a generic bag of named boolean/feature
/// flags, not a Claude- or Codex-specific struct.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Capabilities {
    /// Named feature flags detected in the target project (e.g. "hooks",
    /// "mcp", "skills") — adapter decides what names mean, core stays
    /// agnostic to their semantics.
    pub features: Vec<String>,
}

/// A single managed operation an adapter's `plan()` wants to perform —
/// e.g. write a file, register a hook, wire a tool. Kept minimal; the
/// concrete operation vocabulary (and any fs mutation) is d2's job.
#[derive(Debug, Clone, PartialEq)]
pub struct ManagedOperation {
    pub description: String,
    pub target_path: String,
}

/// Output of `plan()` — an ordered list of managed operations, not yet
/// applied. No fs mutation happens by constructing a `Plan`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Plan {
    pub operations: Vec<ManagedOperation>,
}

/// A source-of-truth asset (recipe, skill, doc fragment, ...) that
/// `render()` turns into a runtime-native artifact. Placeholder — real
/// asset sourcing is d2/d3.
#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    pub identity: String,
    pub content: String,
}

/// Runtime-native rendering of an `Asset` for a specific adapter's
/// target format (e.g. Claude agent frontmatter, Codex config). Kept as
/// a plain string payload at this layer — adapters interpret it.
#[derive(Debug, Clone, PartialEq)]
pub struct Artifact {
    pub target_path: String,
    pub content: String,
}

/// One verification finding — e.g. drift between an installed artifact
/// and its expected content-hash. Output of `verify()`.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub target_path: String,
    pub message: String,
}

/// Aggregate output of `verify()`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Findings {
    pub items: Vec<Finding>,
}

/// A single planned removal — e.g. delete a generated file only if its
/// content-hash still matches the manifest's recorded hash (non-clobber
/// on removal too). Real removal execution is d2.
#[derive(Debug, Clone, PartialEq)]
pub struct RemovalStep {
    pub target_path: String,
    pub safe: bool,
}

/// Output of `uninstall()` — an ordered, not-yet-applied removal plan.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RemovalPlan {
    pub steps: Vec<RemovalStep>,
}

/// The adapter contract. Every runtime adapter (Claude, Codex, ...)
/// implements this trait against core's runtime-neutral types only.
/// `sos-core` never implements or invokes a concrete adapter — adapters
/// depend on core, never the reverse (dependency-direction gate,
/// `tests/dep_direction.rs`).
pub trait Adapter {
    /// Detect what the target project currently supports/has installed.
    fn detect(&self) -> Capabilities;

    /// Given detected capabilities, produce the managed operations this
    /// adapter would perform (no mutation — a plan only).
    fn plan(&self, capabilities: &Capabilities) -> Plan;

    /// Render a portable asset into this adapter's runtime-native
    /// artifact, given detected capabilities.
    fn render(&self, asset: &Asset, capabilities: &Capabilities) -> Artifact;

    /// Verify installed state against expectations (drift/health check).
    fn verify(&self) -> Findings;

    /// Given a managed manifest, produce a safe removal plan (no
    /// mutation — a plan only).
    fn uninstall(&self, manifest: &ManagedManifest) -> RemovalPlan;
}
