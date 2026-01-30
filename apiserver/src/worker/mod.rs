//! Worker module for distributed report processing.
//!
//! Provides background workers that poll Redis queues and process report jobs
//! with semaphore-controlled concurrency.

pub mod handlers;
pub mod runner;

pub use runner::{ReportWorker, WorkerConfig};
