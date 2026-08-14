//! Integration test modules organized by domain.
//! Updated: 2026-03-10 — roster `repository_direct` tests added
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]
#![allow(missing_docs)]
#![allow(warnings)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

pub mod common;

// Domain modules - uncomment as migrated
pub mod agent;
pub mod approval;
pub mod approval_workflow_tests;
pub mod assignment_rule;
pub mod attendance;
pub mod audit;
pub mod auth;
pub mod condition_type;
pub mod dashboard_metrics;
pub mod department;
pub mod employee;
pub mod ess;
pub mod feature_flags;
pub mod holiday;
pub mod leave;
pub mod lifecycle;
pub mod misc;
pub mod monitor;
pub mod notifications;
pub mod overtime;
pub mod payroll;
pub mod rbac;
pub mod report;
pub mod roster;
pub mod shift;
pub mod shift_swap;
pub mod system;
pub mod tenant_provisioner_tests;
pub mod work_segment;
pub mod work_segments_api;
pub mod workflow_callback;
pub mod workspace;
pub mod workspace_config;
