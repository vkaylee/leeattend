use async_trait::async_trait;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::diesel::attendance_log::AttendanceLogErrorEntity;
use crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository;
use crate::schema::attendance_log_errors;

pub struct DieselAttendanceLogErrorRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselAttendanceLogErrorRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl AttendanceLogErrorRepository for DieselAttendanceLogErrorRepository {
    async fn create_error_log(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = AttendanceLogErrorEntity {
            id: crate::utils::id::new_id(),
            workspace_id,
            agent_id,
            raw_content,
            error_message: error_message.to_string(),
            source: source.to_string(),
            created_at: Some(chrono::Utc::now()),
            fingerprint: None,
        };

        diesel::insert_into(attendance_log_errors::table)
            .values(&entity)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn create_error_log_with_fingerprint(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
        fingerprint: &str,
    ) -> Result<bool, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = AttendanceLogErrorEntity {
            id: crate::utils::id::new_id(),
            workspace_id,
            agent_id,
            raw_content,
            error_message: error_message.to_string(),
            source: source.to_string(),
            created_at: Some(chrono::Utc::now()),
            fingerprint: Some(fingerprint.to_string()),
        };

        let rows_affected = diesel::insert_into(attendance_log_errors::table)
            .values(&entity)
            .on_conflict((
                attendance_log_errors::workspace_id,
                attendance_log_errors::fingerprint,
            ))
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows_affected > 0)
    }
}

impl crate::repositories::PoolBacked for DieselAttendanceLogErrorRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!("DieselAttendanceLogErrorRepository requires diesel_pools to be manually injected via new()");
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
