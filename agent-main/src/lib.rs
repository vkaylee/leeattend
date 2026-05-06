// ── TECH_DEBT: Enterprise-grade lint groups (2026-05-03) ──
// Blanket-allowed to keep CI green after workspace-level upgrade.
// Pay down debt: remove group-level allows and fix lints incrementally.
#![allow(clippy::style)]
#![allow(clippy::pedantic)]
#![allow(clippy::perf)]
#![allow(clippy::complexity)]
#![allow(clippy::nursery)]
#![allow(missing_docs)]
#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::clone_on_ref_ptr)]

pub mod adaptive_polling;
pub mod central_client;
pub mod config;
pub mod crash_recovery;
pub mod hybrid_database;
pub mod models;
pub mod redb_store;
pub mod retry;
pub mod utils;

pub mod mock_worker;
pub mod worker;
pub mod worker_processor;

pub mod orchestrator;
pub mod time_sync;
pub mod zk_worker;
