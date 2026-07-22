//! P077d — install framework (transaction plan / dry-run / non-clobber /
//! rollback / sync / manifest).
//!
//! P077d2: `engine` module implements the install ENGINE (transaction
//! plan/apply/rollback/non-clobber/dry-run), driven purely through the
//! `sos-core::adapter::Adapter` trait's `Plan` output — this crate holds
//! ZERO host-specific (Claude/Codex) knowledge.

pub mod engine;
pub mod tools;
