use crate::errors::AppError;
use crate::models::{enums::ImportTargetType, AttendanceImportJob, ImportJobStatus};
use async_trait::async_trait;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: ImportJobRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ImportJobRepository: Send + Sync {
    async fn create_job(&self, job: &AttendanceImportJob) -> Result<(), AppError>;
    async fn update_job_progress(
        &self,
        id: Uuid,
        processed_rows: i32,
        status: ImportJobStatus,
        error_log: Option<String>,
    ) -> Result<(), AppError>;
    async fn get_job_by_id(&self, id: Uuid) -> Result<Option<AttendanceImportJob>, AppError>;
    async fn get_latest_job(
        &self,
        workspace_id: Uuid,
        target_type: Option<ImportTargetType>,
    ) -> Result<Option<AttendanceImportJob>, AppError>;
    async fn get_pending_jobs(&self) -> Result<Vec<AttendanceImportJob>, AppError>;
    async fn update_job_total_rows(&self, id: Uuid, total_rows: i32) -> Result<(), AppError>;
}
