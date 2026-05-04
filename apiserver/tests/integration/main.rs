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
pub mod attendance;
pub mod auth;
pub mod department;
pub mod employee;
pub mod holiday;
pub mod leave;
pub mod payroll;
pub mod rbac;
pub mod report;
pub mod roster;
pub mod shift_swap;
pub mod workspace;
// pub mod workspace;
pub mod ess;
pub mod feature_flags;
pub mod misc;
pub mod monitor;
pub mod notifications;
pub mod shift;
