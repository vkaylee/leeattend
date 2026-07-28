//! Report domain integration tests

mod api;
mod calculation;
mod cascade_snapshot_read;
mod export_api;
mod export_security;
mod export_smoke_test;
mod filtering;
mod legacy_fallback_read;
mod system;
mod websocket_auth;
mod worker_handler;
// mod streaming; // Skipping streaming tests for now as they require complex Redis setup and might be redundant with system tests for type checking
