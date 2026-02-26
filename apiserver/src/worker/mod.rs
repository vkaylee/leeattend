//! Worker module for distributed job processing.
//!
//! Provides background workers that poll Redis queues and process jobs
//! with semaphore-controlled concurrency.

pub mod handlers;
pub mod runner;

pub use runner::{JobWorker, WorkerConfig};
