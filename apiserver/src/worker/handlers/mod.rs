//! Job handlers for different task types.

pub mod accrual;
pub mod attendance;
pub mod bulk_termination;
pub mod claim_staged_logs;
pub mod employee_import;
pub mod expiring_contracts;
pub mod finalize_import;
pub mod housekeeping;
pub mod leave_import;
pub mod log_ingestion;
pub mod merge;
pub mod payroll_report;
pub mod reprocess_attendance;
pub mod sync_hire_date;
pub mod utils;

pub use accrual::AccrualHandler;
pub use attendance::AttendanceHandler;
pub use bulk_termination::BulkTerminationHandler;
pub use claim_staged_logs::ClaimStagedLogsHandler;
pub use employee_import::{EmployeeImportChunkHandler, EmployeeImportHandler};
pub use expiring_contracts::ExpiringContractsHandler;
pub use finalize_import::FinalizeImportHandler;
pub use housekeeping::NightlyHousekeepingHandler;
pub use leave_import::{LeaveImportChunkHandler, LeaveImportHandler};
pub use log_ingestion::LogIngestionHandler;
pub use merge::MergeHandler;
pub use payroll_report::PayrollReportHandler;
pub use reprocess_attendance::ReprocessAttendanceHandler;
pub use sync_hire_date::SyncHireDateHandler;
