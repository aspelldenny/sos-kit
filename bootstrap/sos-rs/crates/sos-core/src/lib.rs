//! sos-core — portable SOS Kit semantic core.
//!
//! Owns state.toml management / spec_hash / config schema. Host-NEUTRAL:
//! this crate declares ZERO dependency on any sos-adapter-*/sos-install/
//! sos-hooks/sos-cli crate (P077b dependency-direction gate, see
//! `tests/dep_direction.rs`). Adapters depend on this crate, never the
//! reverse.

pub mod state;
