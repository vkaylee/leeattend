//! Leave Import Job Handler
//!
//! Processes bulk leave request imports from Excel files as a background job.
//! Manages job tracking via the `attendance_import_jobs` table (target_type = "leave_import").

use crate::models::ImportJobStatus;
use crate::queue::{Job, RedisPool};
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::import_job_repository::SqlxImportJobRepository;
use crate::services::leave_import_service::LeaveImportService;
use sqlx::PgPool;

pub struct LeaveImportHandler;

impl LeaveImportHandler {
    /// Handle a leave import job.
    /// Delegates to LeaveImportService for Excel parsing and submission.
    pub async fn handle(db: &PgPool, job: &Job, _redis: &RedisPool) -> Result<(), String> {
        let payload = job
            .leave_import_payload()
            .ok_or("Missing leave import payload")?;

        let import_repo = SqlxImportJobRepository::new(db.clone());

        // Update import job → Processing
        let _ = import_repo
            .update_job_progress(payload.import_job_id, 0, ImportJobStatus::Processing, None)
            .await;

        tracing::info!(
            workspace_id = %job.workspace_id,
            file_path = %payload.file_path,
            auto_approve = payload.auto_approve,
            import_job_id = %payload.import_job_id,
            "Starting leave import processing"
        );

        let service = LeaveImportService::new(db.clone());

        if service
            .process_job(
                payload.import_job_id,
                job.workspace_id,
                payload.user_id,
                &payload.file_path,
                payload.auto_approve,
            )
            .await
            .is_ok()
        {
            tracing::info!(
                workspace_id = %job.workspace_id,
                import_job_id = %payload.import_job_id,
                "Leave import processing completed"
            );
            Ok(())
        } else {
            let error_msg = format!("Leave import failed for job {}", payload.import_job_id);
            Err(error_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::queue::{Job, JobPayload, LeaveImportPayload, TaskType};
    use uuid::Uuid;

    #[test]
    fn test_leave_import_payload_extraction() {
        let ws_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let import_job_id = Uuid::new_v4();

        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::LeaveImport,
            ws_id,
            JobPayload::LeaveImport(LeaveImportPayload {
                import_job_id,
                user_id,
                file_path: "/tmp/test.xlsx".to_string(),
                auto_approve: true,
            }),
        );

        let payload = job.leave_import_payload().expect("should extract payload");
        assert_eq!(payload.import_job_id, import_job_id);
        assert_eq!(job.workspace_id, ws_id);
        assert_eq!(payload.user_id, user_id);
        assert_eq!(payload.file_path, "/tmp/test.xlsx");
        assert!(payload.auto_approve);
    }

    #[test]
    fn test_leave_import_payload_auto_approve_false() {
        let ws_id = Uuid::new_v4();
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::LeaveImport,
            ws_id,
            JobPayload::LeaveImport(LeaveImportPayload {
                import_job_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                file_path: "/tmp/leaves.csv".to_string(),
                auto_approve: false,
            }),
        );

        let payload = job.leave_import_payload().expect("should extract");
        assert!(!payload.auto_approve);
        assert_eq!(payload.file_path, "/tmp/leaves.csv");
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

        assert!(job.leave_import_payload().is_none());
    }

    #[test]
    fn test_workspace_id_matches_job() {
        let ws_id = Uuid::new_v4();
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::LeaveImport,
            ws_id,
            JobPayload::LeaveImport(LeaveImportPayload {
                import_job_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                file_path: "/tmp/test.xlsx".to_string(),
                auto_approve: false,
            }),
        );

        assert_eq!(job.workspace_id, ws_id);
        let payload = job.leave_import_payload().unwrap();
        assert_eq!(payload.user_id, payload.user_id);
    }
}
