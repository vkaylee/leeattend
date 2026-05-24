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
    async fn create_job(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        job: &AttendanceImportJob,
    ) -> Result<(), AppError>;
    async fn update_job_progress(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        processed_rows: i32,
        status: ImportJobStatus,
        error_log: Option<String>,
    ) -> Result<(), AppError>;
    async fn get_job_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<AttendanceImportJob>, AppError>;
    async fn get_latest_job(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        target_type: Option<ImportTargetType>,
    ) -> Result<Option<AttendanceImportJob>, AppError>;
    async fn get_pending_jobs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AttendanceImportJob>, AppError>;
    async fn update_job_total_rows(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        total_rows: i32,
    ) -> Result<(), AppError>;
}
