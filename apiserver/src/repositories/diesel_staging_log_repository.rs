use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::diesel::attendance_log::AttendanceStagingLogEntity;
use crate::repositories::staging_log_repository::{StagingLog, StagingLogRepository};
use crate::schema::attendance_staging_logs;

pub struct DieselStagingLogRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselStagingLogRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl StagingLogRepository for DieselStagingLogRepository {
    async fn create_staging_log(
        &self,
        workspace_id: Uuid,
        device_id: Option<String>,
        raw_user_id: &str,
        punch_time: DateTime<Utc>,
        raw_data: Option<serde_json::Value>,
        fingerprint: &str,
    ) -> Result<bool, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // Logic from SQLx: INSERT ... WHERE NOT EXISTS
        // Diesel doesn't have a direct "INSERT ... WHERE NOT EXISTS" DSL easily.
        // But we can use on_conflict if there is a constraint.
        // Fingerprint is a unique constraint in the DB.

        let entity = AttendanceStagingLogEntity {
            id: crate::utils::id::new_id(),
            workspace_id,
            device_id,
            raw_user_id: raw_user_id.to_string(),
            punch_time,
            raw_data,
            fingerprint: fingerprint.to_string(),
            status: "pending".to_string(),
            created_at: Utc::now(),
        };

        let rows_affected = diesel::insert_into(attendance_staging_logs::table)
            .values(&entity)
            .on_conflict(attendance_staging_logs::fingerprint)
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows_affected > 0)
    }

    async fn get_pending_staging_logs(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<StagingLog>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let results: Vec<AttendanceStagingLogEntity> = attendance_staging_logs::table
            .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
            .filter(attendance_staging_logs::status.eq("pending"))
            .order(attendance_staging_logs::created_at.asc())
            .limit(limit)
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|e| StagingLog {
                id: e.id,
                workspace_id: e.workspace_id,
                device_id: e.device_id,
                raw_user_id: e.raw_user_id,
                punch_time: e.punch_time,
                raw_data: e.raw_data,
                fingerprint: e.fingerprint,
                status: e.status,
                created_at: e.created_at,
            })
            .collect())
    }

    async fn update_staging_status(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        status: &str,
    ) -> Result<bool, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let rows_affected = diesel::update(attendance_staging_logs::table)
            .filter(attendance_staging_logs::id.eq(id))
            .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
            .set(attendance_staging_logs::status.eq(status))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows_affected > 0)
    }

    async fn count_pending_staging_logs(&self, workspace_id: Uuid) -> Result<i64, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let count = attendance_staging_logs::table
            .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
            .filter(attendance_staging_logs::status.eq("pending"))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(count)
    }
}

impl crate::repositories::PoolBacked for DieselStagingLogRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!(
            "DieselStagingLogRepository requires diesel_pools to be manually injected via new()"
        );
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
