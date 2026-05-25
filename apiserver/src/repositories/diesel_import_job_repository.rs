use async_trait::async_trait;
use chrono::Utc;
use diesel::OptionalExtension;
use diesel::{ExpressionMethods, QueryDsl};
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
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let entity = ImportJobEntity {
            id: job.id,
            workspace_id: job.workspace_id,
            user_id: job.user_id,
            status: job.status,
            total_rows: job.total_rows,
            processed_rows: job.processed_rows,
            file_path: job.file_path.clone(),
            error_log: job.error_log.clone(),
            target_type: job.target_type.to_string(),
            created_at: job.created_at,
            updated_at: job.updated_at,
        };

        use crate::schema::attendance_import_jobs;
        diesel::insert_into(attendance_import_jobs::table)
            .values(&entity)
            .execute(&mut *conn)
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
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use crate::schema::attendance_import_jobs;
        diesel::update(attendance_import_jobs::table)
            .filter(attendance_import_jobs::id.eq(id))
            .filter(attendance_import_jobs::workspace_id.eq(auth_token.workspace_id()))
            .set((
                attendance_import_jobs::processed_rows.eq(processed_rows),
                attendance_import_jobs::status.eq(status),
                attendance_import_jobs::error_log.eq(error_log),
                attendance_import_jobs::updated_at.eq(Utc::now()),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_job_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<AttendanceImportJob>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use crate::schema::attendance_import_jobs;
        let row: Option<ImportJobEntity> = attendance_import_jobs::table
            .filter(attendance_import_jobs::id.eq(id))
            .filter(attendance_import_jobs::workspace_id.eq(auth_token.workspace_id()))
            .first(&mut *conn)
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
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::attendance_import_jobs;

        let mut query = attendance_import_jobs::table
            .filter(attendance_import_jobs::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(tt) = target_type {
            query = query.filter(attendance_import_jobs::target_type.eq(tt.to_string()));
        }

        let row: Option<ImportJobEntity> = query
            .order(attendance_import_jobs::created_at.desc())
            .first(&mut *conn)
            .await
            .optional()?;

        Ok(row.map(Into::into))
    }

    async fn get_pending_jobs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AttendanceImportJob>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use crate::schema::attendance_import_jobs;
        let rows: Vec<ImportJobEntity> = attendance_import_jobs::table
            .filter(attendance_import_jobs::workspace_id.eq(auth_token.workspace_id()))
            .filter(
                attendance_import_jobs::status
                    .eq(crate::models::enums::ImportJobStatus::Processing),
            )
            .load(&mut *conn)
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
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use crate::schema::attendance_import_jobs;
        diesel::update(attendance_import_jobs::table)
            .filter(attendance_import_jobs::id.eq(id))
            .filter(attendance_import_jobs::workspace_id.eq(auth_token.workspace_id()))
            .set((
                attendance_import_jobs::total_rows.eq(total_rows),
                attendance_import_jobs::updated_at.eq(Utc::now()),
            ))
            .execute(&mut *conn)
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
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselImportJobRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
