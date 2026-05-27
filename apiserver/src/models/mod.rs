// ==============================================================================
// models/mod.rs — Domain-split model modules
// ==============================================================================
// This file re-exports all types from domain-specific sub-modules so that all
// existing `use crate::models::SomeType` imports continue to work unchanged.
// ==============================================================================

pub mod agent;
pub mod approval_workflow;
pub mod attendance;
pub mod auth;
pub mod common;
pub mod contract;
pub mod department;
pub mod diesel;
pub mod employee;
pub mod employee_assignment;
pub mod employment_phase;
pub mod enums;
pub mod ess;
pub mod feature_flag;
pub mod holiday;
pub mod leave;
pub mod notification;
pub mod overtime;
pub mod overtime_request;
pub mod payroll;
pub mod payroll_rule;
pub mod policy_template;
pub mod salary_base;
pub mod schedule;
pub mod shift;
pub mod shift_swap;
pub mod system_db_cluster;
pub mod time_tracking;
pub mod workspace;
pub mod workspace_config;
pub mod work_location;
// Re-export everything to maintain backward compatibility with all consumers
pub use agent::*;
pub use approval_workflow::*;
pub use attendance::*;
pub use auth::*;
pub use common::*;
pub use contract::*;
pub use department::*;
pub use employee::*;
pub use employee_assignment::*;
pub use employment_phase::*;
pub use enums::*;
pub use ess::*;
pub use feature_flag::*;
pub use holiday::*;
pub use leave::*;
pub use notification::*;
pub use overtime::*;
pub use overtime_request::*;
pub use payroll::*;
pub use payroll_rule::*;
pub use policy_template::*;
pub use salary_base::*;
pub use schedule::*;
pub use shift::*;
pub use shift_swap::*;
pub use time_tracking::*;
pub use workspace::*;
pub use workspace_config::*;
pub use work_location::*;
