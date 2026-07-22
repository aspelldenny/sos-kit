// P077d1 — compile-level proof that a plain in-crate type can satisfy the
// `Adapter` trait bound using only core's runtime-neutral types. This is a
// SHAPE test only (does the trait compile + is it implementable), not a
// behavior test — real adapter logic lives in sos-adapter-claude (and
// later a Codex adapter, P078), exercised outside this crate.

use sos_core::adapter::{
    Adapter, Artifact, Asset, Capabilities, Findings, Plan, RemovalPlan,
};
use sos_core::manifest::ManagedManifest;

struct NoopAdapter;

impl Adapter for NoopAdapter {
    fn detect(&self) -> Capabilities {
        Capabilities::default()
    }

    fn plan(&self, _capabilities: &Capabilities) -> Plan {
        Plan::default()
    }

    fn render(&self, asset: &Asset, _capabilities: &Capabilities) -> Artifact {
        Artifact {
            target_path: asset.identity.clone(),
            content: asset.content.clone(),
        }
    }

    fn verify(&self) -> Findings {
        Findings::default()
    }

    fn uninstall(&self, _manifest: &ManagedManifest) -> RemovalPlan {
        RemovalPlan::default()
    }
}

fn assert_adapter(_a: impl Adapter) {}

#[test]
fn noop_adapter_satisfies_adapter_trait_bound() {
    assert_adapter(NoopAdapter);
}
