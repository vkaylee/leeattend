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
pub mod feature_flag;
pub mod holiday;
pub mod ess;
pub mod leave;
pub mod overtime;
pub mod payroll;
pub mod schedule;
pub mod shift;
pub mod shift_swap;
pub mod time_tracking;
pub mod workspace;
pub mod employment_phase;
pub mod payroll_rule;
pub mod policy_template;
pub mod salary_base;

// Re-export everything to maintain backward compatibility with all consumers
pub use agent::*;
pub use attendance::*;
pub use auth::*;
pub use common::*;
pub use department::*;
pub use employee::*;
pub use enums::*;
pub use feature_flag::*;
pub use ess::*;
pub use holiday::*;
pub use leave::*;
pub use overtime::*;
pub use payroll::*;
pub use schedule::*;
pub use shift::*;
pub use shift_swap::*;
pub use time_tracking::*;
pub use workspace::*;
pub use employment_phase::*;
pub use payroll_rule::*;
pub use policy_template::*;
pub use salary_base::*;
