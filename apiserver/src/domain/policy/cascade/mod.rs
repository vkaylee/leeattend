//! Layered Explicit Policy System — cascade engine.
//!
//! Coexists with the legacy `resolver.rs` during Phase 2 → Phase 5 dual-run
//! ([[ADR-005]]). Callers still route through the old resolver; new callers
//! opt into the cascade engine via `evaluate()`.
//!
//! Layer order (bottom → top, per [[ADR-001]]):
//! L1 base → L2a day_type → L2b shift_template → L2c shift_version
//! → L2.5 employee_modifier → L3 exception.

pub mod base;
pub mod caps;
pub mod compiled;
pub mod context;
pub mod day_type;
pub mod engine;
pub mod merge;
pub mod partial;
pub mod time_slice;
pub mod trace;

pub use engine::evaluate;
