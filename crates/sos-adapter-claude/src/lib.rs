//! P077d1 — Claude Code adapter contract (detect/plan/render/verify/uninstall).
//!
//! `ClaudeAdapter` implements the core-defined `Adapter` trait with STUB
//! bodies only — no filesystem mutation, no real rendering. Purpose:
//! prove the trait is implementable end-to-end and keep the
//! dependency-direction gate exercised (adapter -> core, one-way; see
//! `crates/sos-core/tests/dep_direction.rs`). Real Claude asset
//! detection/rendering/removal logic lands in P077d2.

use sos_core::adapter::{
    Adapter, Artifact, Asset, Capabilities, Findings, Plan, RemovalPlan,
};
use sos_core::manifest::ManagedManifest;

pub struct ClaudeAdapter;

impl Adapter for ClaudeAdapter {
    fn detect(&self) -> Capabilities {
        // Stub: real detection (scan for .claude/**, CLAUDE.md, etc.) is d2.
        Capabilities::default()
    }

    fn plan(&self, _capabilities: &Capabilities) -> Plan {
        // Stub: real managed-operation planning is d2.
        Plan::default()
    }

    fn render(&self, asset: &Asset, _capabilities: &Capabilities) -> Artifact {
        // Stub: real rendering into Claude-native artifact shape is d2.
        Artifact {
            target_path: asset.identity.clone(),
            content: asset.content.clone(),
        }
    }

    fn verify(&self) -> Findings {
        // Stub: real drift/health verification is d2.
        Findings::default()
    }

    fn uninstall(&self, _manifest: &ManagedManifest) -> RemovalPlan {
        // Stub: real safe-removal planning is d2.
        RemovalPlan::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_adapter_satisfies_adapter_trait_bound() {
        fn assert_adapter(_a: impl Adapter) {}
        assert_adapter(ClaudeAdapter);
    }
}
