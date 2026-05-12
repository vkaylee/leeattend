use async_trait::async_trait;
use chrono::Utc;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::import_job::ImportJobEntity;
use crate::models::enums::{ImportJobStatus, ImportTargetType};
use crate::models::AttendanceImportJob;
use crate::repositories::import_job_repository::ImportJobRepository;

pub struct DieselImportJobRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselImportJobRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl ImportJobRepository for DieselImportJobRepository {
    async fn create_job(&self, job: &AttendanceImportJob) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::sql_query(
            "INSERT INTO attendance_import_jobs (id, workspace_id, user_id, status, total_rows, processed_rows, file_path, error_log, target_type, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
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
        id: Uuid,
        processed_rows: i32,
        status: ImportJobStatus,
        error_log: Option<String>,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::sql_query(
            "UPDATE attendance_import_jobs SET processed_rows = $1, status = $2, error_log = $3, updated_at = $4 WHERE id = $5"
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

    async fn get_job_by_id(&self, id: Uuid) -> Result<Option<AttendanceImportJob>, AppError> {
        let row: Option<ImportJobEntity> = sqlx::query_as(
            "SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type::text, created_at, updated_at FROM attendance_import_jobs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.sqlx_pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(row.map(Into::into))
    }

    async fn get_latest_job(
        &self,
        workspace_id: Uuid,
        target_type: Option<ImportTargetType>,
    ) -> Result<Option<AttendanceImportJob>, AppError> {
        let row: Option<ImportJobEntity> = if let Some(tt) = target_type {
            sqlx::query_as(
                "SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type::text, created_at, updated_at FROM attendance_import_jobs WHERE workspace_id = $1 AND target_type = $2 ORDER BY created_at DESC LIMIT 1"
            )
            .bind(workspace_id)
            .bind(tt.to_string())
            .fetch_optional(&self.sqlx_pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type::text, created_at, updated_at FROM attendance_import_jobs WHERE workspace_id = $1 ORDER BY created_at DESC LIMIT 1"
            )
            .bind(workspace_id)
            .fetch_optional(&self.sqlx_pool)
            .await?
        };

        Ok(row.map(Into::into))
    }

    async fn get_pending_jobs(&self) -> Result<Vec<AttendanceImportJob>, AppError> {
        let rows: Vec<ImportJobEntity> = sqlx::query_as(
            "SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type::text, created_at, updated_at FROM attendance_import_jobs WHERE status = 'processing'"
        )
        .fetch_all(&self.sqlx_pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_job_total_rows(&self, id: Uuid, total_rows: i32) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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

impl crate::repositories::PoolBacked for DieselImportJobRepository {
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!("DieselImportJobRepository requires DieselPools. Use new() instead.");
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
