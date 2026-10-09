//! P077d1 — managed-manifest schema (foundation abstraction).
//!
//! Every generated/managed artifact (installed registration, hook stub,
//! rendered adapter file, ...) is recorded with a `ManagedManifest` entry
//! per `core/ASSETS.md:57-64` ("Generated assets") — 6 semantic fields:
//! owner, source version, source identity, target path, installed content
//! hash, and an optional rollback reference (set only when a mutation has
//! occurred).
//!
//! Data-only: no install/apply/rollback logic lives here (that's P077d2).
//! Serialized as TOML — `sos-core` already depends on the `toml` crate for
//! `state.toml` (see `state.rs`); this keeps one serialization format
//! across all core-owned on-disk schemas.

use serde::{Deserialize, Serialize};

/// One record of a managed/generated artifact — e.g. a rendered Claude
/// agent file, a hook stub, an installed registration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedManifest {
    /// The owning integration or portable component (e.g.
    /// "sos-adapter-claude", "sos-core") responsible for this artifact.
    pub owner: String,

    /// The source product version the artifact was rendered/installed
    /// from (e.g. a sos-kit release tag or crate version).
    pub source_version: String,

    /// Identity of the portable source this artifact was derived from
    /// (e.g. a recipe name, skill id, or doc fragment identity).
    pub source_identity: String,

    /// Where the artifact was installed, relative to project root.
    pub target_path: String,

    /// sha256 (or equivalent) hash of the artifact's installed content,
    /// used to detect drift and to gate non-clobber apply/removal.
    pub content_hash: String,

    /// Previous-state or rollback reference — set only when a mutation
    /// has occurred (`core/ASSETS.md:64`). `None` for a fresh install
    /// that has never been overwritten.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_ref: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_manifest_toml_round_trip() {
        let original = ManagedManifest {
            owner: "sos-adapter-claude".to_string(),
            source_version: "0.1.0".to_string(),
            source_identity: "agents/worker.md".to_string(),
            target_path: ".claude/agents/worker.md".to_string(),
            content_hash: "deadbeef".to_string(),
            rollback_ref: Some("prev-deadbeef".to_string()),
        };

        let toml_str = toml::to_string_pretty(&original).expect("serialize manifest to TOML");
        let round_tripped: ManagedManifest =
            toml::from_str(&toml_str).expect("deserialize manifest from TOML");

        assert_eq!(original, round_tripped);
    }

    #[test]
    fn managed_manifest_rollback_ref_none_round_trips() {
        let original = ManagedManifest {
            owner: "sos-adapter-claude".to_string(),
            source_version: "0.1.0".to_string(),
            source_identity: "agents/worker.md".to_string(),
            target_path: ".claude/agents/worker.md".to_string(),
            content_hash: "deadbeef".to_string(),
            rollback_ref: None,
        };

        let toml_str = toml::to_string_pretty(&original).expect("serialize manifest to TOML");
        let round_tripped: ManagedManifest =
            toml::from_str(&toml_str).expect("deserialize manifest from TOML");

        assert_eq!(original, round_tripped);
    }
}
