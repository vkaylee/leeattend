// ==============================================================================
// models/mod.rs — Domain-split model modules
// ==============================================================================
// This file re-exports all types from domain-specific sub-modules so that all
// existing `use crate::models::SomeType` imports continue to work unchanged.
// ==============================================================================

pub mod agent;
pub mod attendance;
pub mod auth;
pub mod common;
pub mod department;
pub mod employee;
pub mod enums;
pub mod ess;
pub mod leave;
pub mod overtime;
pub mod payroll;
pub mod schedule;
pub mod shift;
pub mod time_tracking;
pub mod workspace;

// Re-export everything to maintain backward compatibility with all consumers
pub use agent::*;
pub use attendance::*;
pub use auth::*;
pub use common::*;
pub use department::*;
pub use employee::*;
pub use enums::*;
pub use ess::*;
pub use leave::*;
pub use overtime::*;
pub use payroll::*;
pub use schedule::*;
pub use shift::*;
pub use time_tracking::*;
pub use workspace::*;
