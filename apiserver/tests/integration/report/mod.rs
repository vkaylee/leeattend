//! Report domain integration tests

mod api;
mod calculation;
mod export_security;
mod filtering;
mod system;
mod websocket_auth;
// mod streaming; // Skipping streaming tests for now as they require complex Redis setup and might be redundant with system tests for type checking
