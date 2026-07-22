//! P077d — Claude Code adapter contract (detect/plan/render/verify/uninstall).
//! Skeleton only; logic lands in P077d. Exists here to wire the real
//! dependency graph (adapter -> core, one-way) for the P077b gate.

// Proves adapter->core is allowed (compiles clean): sos-core is a direct dep.
#[allow(unused_imports)]
use sos_core::state as _;
