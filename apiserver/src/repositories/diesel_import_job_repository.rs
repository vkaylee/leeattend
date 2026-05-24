use async_trait::async_trait;
use chrono::Utc;
use diesel::OptionalExtension;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::import_job::ImportJobEntity;
use crate::models::enums::{ImportJobStatus, ImportTargetType};
use crate::models::AttendanceImportJob;
use crate::repositories::import_job_repository::ImportJobRepository;

pub struct DieselImportJobRepository {
    pub diesel_pools: DieselPools,
}

impl DieselImportJobRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl ImportJobRepository for DieselImportJobRepository {
    async fn create_job(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        job: &AttendanceImportJob,
    ) -> Result<(), AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;

        diesel::sql_query(
            "INSERT INTO attendance_import_jobs (id, workspace_id, user_id, status, total_rows, processed_rows, file_path, error_log, target_type, created_at, updated_at) VALUES ($1, $2, $3, $4::import_job_status_type, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind::<diesel::sql_types::Uuid, _>(job.id)
        .bind::<diesel::sql_types::Uuid, _>(job.workspace_id)
        .bind::<diesel::sql_types::Uuid, _>(job.user_id)
        .bind::<diesel::sql_types::Text, _>(job.status.to_string())
        .bind::<diesel::sql_types::Int4, _>(job.total_rows)
        .bind::<diesel::sql_types::Int4, _>(job.processed_rows)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&job.file_path)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&job.error_log)
        .bind::<diesel::sql_types::Text, _>(job.target_type.to_string())
        .bind::<diesel::sql_types::Timestamptz, _>(job.created_at)
        .bind::<diesel::sql_types::Timestamptz, _>(job.updated_at)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn update_job_progress(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        processed_rows: i32,
        status: ImportJobStatus,
        error_log: Option<String>,
    ) -> Result<(), AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;

        diesel::sql_query(
            "UPDATE attendance_import_jobs SET processed_rows = $1, status = $2::import_job_status_type, error_log = $3, updated_at = $4 WHERE id = $5"
        )
        .bind::<diesel::sql_types::Int4, _>(processed_rows)
        .bind::<diesel::sql_types::Text, _>(status.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&error_log)
        .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_job_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<AttendanceImportJob>, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;

        let row: Option<ImportJobEntity> = diesel::sql_query(
            "SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type::text, created_at, updated_at FROM attendance_import_jobs WHERE id = $1"
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .get_result(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(row.map(Into::into))
    }

    async fn get_latest_job(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        target_type: Option<ImportTargetType>,
    ) -> Result<Option<AttendanceImportJob>, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();

        let row: Option<ImportJobEntity> = if let Some(tt) = target_type {
            diesel::sql_query(
                "SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type::text, created_at, updated_at FROM attendance_import_jobs WHERE workspace_id = $1 AND target_type = $2 ORDER BY created_at DESC LIMIT 1"
            )
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Text, _>(tt.to_string())
            .get_result(&mut conn)
            .await
            .optional()?
        } else {
            diesel::sql_query(
                "SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type::text, created_at, updated_at FROM attendance_import_jobs WHERE workspace_id = $1 ORDER BY created_at DESC LIMIT 1"
            )
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .get_result(&mut conn)
            .await
            .optional()?
        };

        Ok(row.map(Into::into))
    }

    async fn get_pending_jobs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AttendanceImportJob>, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;

        let rows: Vec<ImportJobEntity> = diesel::sql_query(
            "SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type::text, created_at, updated_at FROM attendance_import_jobs WHERE status = 'processing'"
        )
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_job_total_rows(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        total_rows: i32,
    ) -> Result<(), AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;

        diesel::sql_query(
            "UPDATE attendance_import_jobs SET total_rows = $1, updated_at = $2 WHERE id = $3",
        )
        .bind::<diesel::sql_types::Int4, _>(total_rows)
        .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/leeattend_test_db".to_string());
        
        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselImportJobRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
