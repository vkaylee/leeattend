//! Employee Import Job Handler
//!
//! Processes bulk employee imports from Excel files as a background job.
//! Manages job tracking via the `attendance_import_jobs` table.

use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::models::ImportJobStatus;
use crate::queue::{Job, RedisPool};
use crate::repositories::import_job_repository::{ImportJobRepository, SqlxImportJobRepository};
use crate::services::attendance_import_service::AttendanceImportService;

use crate::worker::handlers::utils::ImportTimeoutGuard;

pub struct EmployeeImportHandler;

impl EmployeeImportHandler {
    /// Handle an employee import job.
    pub async fn handle(db: &PgPool, job: &Job, _redis: &RedisPool) -> Result<(), String> {
        let payload = job
            .employee_import_payload()
            .ok_or("Missing employee import payload")?;

        let import_repo = SqlxImportJobRepository::new(db.clone());

        // Update import job → Processing
        let _ = import_repo
            .update_job_progress(payload.import_job_id, 0, ImportJobStatus::Processing, None)
            .await;

        tracing::info!(
            workspace_id = %job.workspace_id,
            user_id = %payload.user_id,
            import_job_id = %payload.import_job_id,
            "Starting employee import processing"
        );

        let completed = Arc::new(AtomicBool::new(false));
        let _guard =
            ImportTimeoutGuard::new(db.clone(), payload.import_job_id, Arc::clone(&completed));

        let import_repo_arc = Arc::new(import_repo);
        let adj_repo = Arc::new(crate::repositories::attendance_adjustment_repository::SqlxAttendanceAdjustmentRepository::new(db.clone()));
        let employee_repo = Arc::new(crate::repositories::SqlxEmployeeRepository::new(db.clone()));
        let time_entry_repo = Arc::new(crate::repositories::SqlxTimeEntryRepository::new(
            db.clone(),
        ));
        let workspace_repo = Arc::new(crate::repositories::SqlxWorkspaceRepository::new(
            db.clone(),
        ));

        let service = AttendanceImportService::new(
            import_repo_arc,
            adj_repo,
            employee_repo,
            time_entry_repo,
            workspace_repo,
            db.clone(),
        );

        // Process the job
        // Note: process_job internally updates the DB to Completed or Failed.
        let process_result = service.process_job(payload.import_job_id).await;
        completed.store(true, Ordering::SeqCst);

        if process_result.is_ok() {
            tracing::info!(
                workspace_id = %job.workspace_id,
                import_job_id = %payload.import_job_id,
                "Employee import processing completed successfully"
            );
            Ok(())
        } else {
            let error_msg = format!("Employee import failed for job {}", payload.import_job_id);
            // process_job already updates the import_job_id to Failed in the DB.
            Err(error_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::queue::{EmployeeImportPayload, Job, JobPayload, TaskType};
    use uuid::Uuid;

    #[test]
    fn test_employee_import_payload_extraction() {
        let ws_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let import_job_id = Uuid::new_v4();

        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::EmployeeImport,
            ws_id,
            JobPayload::EmployeeImport(EmployeeImportPayload {
                import_job_id,
                user_id,
            }),
        );

        let payload = job
            .employee_import_payload()
            .expect("should extract payload");
        assert_eq!(payload.import_job_id, import_job_id);
        assert_eq!(payload.user_id, user_id);
        assert_eq!(job.workspace_id, ws_id);
    }

    #[test]
    fn test_wrong_payload_returns_none() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(crate::queue::LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2026-01-01 08:00:00".to_string(),
                device_id: None,
                raw_data: None,
                timezone: "UTC".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
            }),
        );

        assert!(job.employee_import_payload().is_none());
    }
}
