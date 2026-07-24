//! Job Types and Metadata
//!
//! Defines the core job structures for the distributed worker system.
//!
//! Each [`Job`] carries a [`JobPayload`] enum that encapsulates type-specific data:
//! - `AttendanceReport` / `PayrollReport` → [`ReportPayload`] (employee_ids, date_range, config)
//! - `MergeResults` → [`MergePayload`] (date_range, config — no employee_ids)
//!
//! Use [`Job::report_payload()`] and [`Job::merge_payload()`] for safe extraction.

use chrono::{DateTime, Utc};
use uuid::Uuid;

mod types;
pub use types::*;

#[cfg(test)]
mod tests;

// Re-export items the test module's `use super::*;` expects from the
// previously-monolithic file (migration scaffold — same pattern as leave_service/).
// NOTE: Uuid is already imported at module level for production use (line 12),
// so it's visible to tests via `use super::*;` without a separate re-export.
#[cfg(test)]
pub(super) use chrono::NaiveDate;
#[cfg(test)]
pub(super) use serde_json;

/// A single worker job.
/// Generic job envelope with type-specific payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Job {
    /// Unique identifier for this job
    pub job_id: Uuid,
    /// Parent job ID (coordinator job)
    pub parent_job_id: Uuid,
    /// Batch index (0-based)
    pub batch_index: u32,
    /// Type of processing task
    pub task_type: TaskType,
    /// Workspace this job belongs to
    pub workspace_id: Uuid,
    /// Type-specific payload data
    pub payload: JobPayload,
    /// Job priority for queue routing
    #[serde(default)]
    pub priority: JobPriority,
    /// Job creation timestamp
    pub created_at: DateTime<Utc>,
    /// Current retry count
    pub retry_count: u32,
    /// Maximum retries before failure
    pub max_retries: u32,
    /// Context tracing (e.g. Request ID) for propagation
    pub trace_context: Option<String>,
}

impl Job {
    /// Create a new job with default retry settings and Normal priority.
    pub fn new(
        parent_job_id: Uuid,
        batch_index: u32,
        task_type: TaskType,
        workspace_id: Uuid,
        payload: JobPayload,
    ) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            parent_job_id,
            batch_index,
            task_type,
            workspace_id,
            payload,
            priority: JobPriority::default(),
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
            trace_context: crate::utils::logging::get_trace_context(),
        }
    }

    /// Create a new job with explicit priority.
    pub fn with_priority(
        parent_job_id: Uuid,
        batch_index: u32,
        task_type: TaskType,
        workspace_id: Uuid,
        payload: JobPayload,
        priority: JobPriority,
    ) -> Self {
        let mut job = Self::new(parent_job_id, batch_index, task_type, workspace_id, payload);
        job.priority = priority;
        job
    }

    /// Increment retry count and return whether retries are exhausted.
    pub fn increment_retry(&mut self) -> bool {
        self.retry_count += 1;
        self.retry_count > self.max_retries
    }

    /// Helper: extract report payload (attendance/payroll).
    pub fn report_payload(&self) -> Option<&ReportPayload> {
        match &self.payload {
            JobPayload::AttendanceReport(p) | JobPayload::PayrollReport(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract merge payload.
    pub fn merge_payload(&self) -> Option<&MergePayload> {
        match &self.payload {
            JobPayload::MergeResults(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract accrual payload.
    pub fn accrual_payload(&self) -> Option<&AccrualPayload> {
        match &self.payload {
            JobPayload::LeaveAccrual(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract log ingestion payload.
    pub fn log_ingestion_payload(&self) -> Option<&LogIngestionPayload> {
        match &self.payload {
            JobPayload::LogIngestion(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract reprocess payload.
    pub fn reprocess_payload(&self) -> Option<&ReprocessPayload> {
        match &self.payload {
            JobPayload::ReprocessAttendance(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract claim staged logs payload.
    pub fn claim_staged_logs_payload(&self) -> Option<&ClaimStagedLogsPayload> {
        match &self.payload {
            JobPayload::ClaimStagedLogs(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract leave import payload.
    pub fn leave_import_payload(&self) -> Option<&LeaveImportPayload> {
        match &self.payload {
            JobPayload::LeaveImport(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract housekeeping payload.
    pub fn housekeeping_payload(&self) -> Option<&HousekeepingPayload> {
        match &self.payload {
            JobPayload::NightlyHousekeeping(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract employee import payload.
    pub fn employee_import_payload(&self) -> Option<&EmployeeImportPayload> {
        match &self.payload {
            JobPayload::EmployeeImport(p) => Some(p),
            _ => None,
        }
    }

    pub fn employee_import_chunk_payload(&self) -> Option<&EmployeeImportChunkPayload> {
        match &self.payload {
            JobPayload::EmployeeImportChunk(p) => Some(p),
            _ => None,
        }
    }

    pub fn leave_import_chunk_payload(&self) -> Option<&LeaveImportChunkPayload> {
        match &self.payload {
            JobPayload::LeaveImportChunk(p) => Some(p),
            _ => None,
        }
    }

    pub fn finalize_import_payload(&self) -> Option<&FinalizeImportPayload> {
        match &self.payload {
            JobPayload::FinalizeImport(p) => Some(p),
            _ => None,
        }
    }

    pub fn expiring_contracts_payload(&self) -> Option<&ExpiringContractsPayload> {
        match &self.payload {
            JobPayload::ExpiringContractsCheck(p) => Some(p),
            _ => None,
        }
    }

    pub fn sync_hire_date_payload(&self) -> Option<&SyncHireDatePayload> {
        match &self.payload {
            JobPayload::SyncEmployeeHireDate(p) => Some(p),
            _ => None,
        }
    }

    pub fn bulk_termination_payload(&self) -> Option<&BulkTerminationPayload> {
        match &self.payload {
            JobPayload::BulkTermination(p) => Some(p),
            _ => None,
        }
    }
}
