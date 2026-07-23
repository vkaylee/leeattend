//! Worker module for distributed job processing.
//!
//! Provides background workers that poll Redis queues and process jobs
//! with semaphore-controlled concurrency.

pub mod deps;
pub mod file_scan_scheduler;
pub mod handlers;
pub mod outbox_sweep;
pub mod pairing_scheduler;
pub mod runner;
pub mod scheduler;

pub use deps::WorkerDeps;
pub use file_scan_scheduler::FileScanScheduler;
pub use outbox_sweep::OutboxSweepScheduler;
pub use pairing_scheduler::{PairingNotifier, PairingScheduler, pairing_channel};
pub use runner::{JobWorker, WorkerConfig};
pub use scheduler::HousekeepingScheduler;

#[cfg(test)]
pub mod test_utils;
