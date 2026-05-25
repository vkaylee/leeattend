use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::diesel::attendance_log::AttendanceStagingLogEntity;
use crate::repositories::staging_log_repository::{StagingLog, StagingLogRepository};
use crate::schema::attendance_staging_logs;

pub struct DieselStagingLogRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselStagingLogRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
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
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

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

        use crate::schema::attendance_staging_logs::dsl as staging_dsl;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        let exists: bool = diesel::select(diesel::dsl::exists(
            staging_dsl::attendance_staging_logs
                .filter(staging_dsl::workspace_id.eq(workspace_id))
                .filter(staging_dsl::fingerprint.eq(fingerprint)),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if exists {
            return Ok(false);
        }

        let rows_affected = diesel::insert_into(staging_dsl::attendance_staging_logs)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows_affected > 0)
    }

    async fn get_pending_staging_logs(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<StagingLog>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results: Vec<AttendanceStagingLogEntity> = attendance_staging_logs::table
            .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
            .filter(attendance_staging_logs::status.eq("pending"))
            .order(attendance_staging_logs::created_at.asc())
            .limit(limit)
            .load(&mut *conn)
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

    async fn get_pending_staged_logs(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Vec<StagingLog>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results: Vec<AttendanceStagingLogEntity> = attendance_staging_logs::table
            .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
            .filter(attendance_staging_logs::raw_user_id.eq(employee_id_number))
            .filter(attendance_staging_logs::status.eq("pending"))
            .order(attendance_staging_logs::punch_time.asc())
            .load(&mut *conn)
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
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let rows_affected = diesel::update(attendance_staging_logs::table)
            .filter(attendance_staging_logs::id.eq(id))
            .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
            .set(attendance_staging_logs::status.eq(status))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows_affected > 0)
    }

    async fn count_pending_staging_logs(&self, workspace_id: Uuid) -> Result<i64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let count = attendance_staging_logs::table
            .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
            .filter(attendance_staging_logs::status.eq("pending"))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(count)
    }

    async fn claim_staged_logs(
        &self,
        workspace_id: Uuid,
        raw_user_id: &str,
        employee_id: Uuid,
    ) -> Result<u64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let staged_logs = attendance_staging_logs::table
            .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
            .filter(attendance_staging_logs::raw_user_id.eq(raw_user_id))
            .filter(attendance_staging_logs::status.eq("pending"))
            .order(attendance_staging_logs::punch_time.asc())
            .load::<AttendanceStagingLogEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut claimed = 0u64;
        for staged in staged_logs {
            let id = crate::utils::id::new_id();
            let raw_log_entity =
                crate::models::diesel::attendance_log::RawAttendanceLogEntity {
                    id,
                    workspace_id,
                    employee_id,
                    punch_time: staged.punch_time,
                    source: crate::models::LogSource::Device,
                    device_id: staged.device_id.clone(),
                    raw_data: staged.raw_data.clone(),
                    punch: -1,
                    status: -1,
                    created_at: Some(chrono::Utc::now()),
                };

            let rows = diesel::insert_into(crate::schema::raw_attendance_logs::table)
                .values(&raw_log_entity)
                .on_conflict_do_nothing()
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

            if rows > 0 {
                claimed += 1;
            }

            diesel::update(
                attendance_staging_logs::table.filter(attendance_staging_logs::id.eq(staged.id)),
            )
            .set(attendance_staging_logs::status.eq("processed"))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;
        }

        Ok(claimed)
    }

    async fn resolve_staging_log(
        &self,
        log_id: Uuid,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<crate::models::RawAttendanceLog, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::models::{LogSource, RawAttendanceLog};
        use crate::schema::raw_attendance_logs;
        use diesel::result::DatabaseErrorKind;

        conn.transaction(|conn| {
            Box::pin(async move {
                // Use for_update() to lock the row. We don't use nowait() here because it might
                // not be available or might cause compatibility issues. Just use regular for_update().
                let staged = attendance_staging_logs::table
                    .filter(attendance_staging_logs::id.eq(log_id))
                    .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
                    .for_update()
                    .first::<AttendanceStagingLogEntity>(&mut *conn)
                    .await
                    .map_err(|e| {
                        if let diesel::result::Error::DatabaseError(
                            DatabaseErrorKind::SerializationFailure,
                            _,
                        ) = e
                        {
                            AppError::Conflict(
                                "Log is being processed by another request".to_string(),
                            )
                        } else if e == diesel::result::Error::NotFound {
                            AppError::NotFound("Staging log not found".to_string())
                        } else {
                            AppError::Internal(format!("db: {e}"))
                        }
                    })?;

                if staged.status != "pending" {
                    return Err(AppError::Validation("Log already processed".to_string()));
                }

                let punch = staged
                    .raw_data
                    .as_ref()
                    .and_then(|d| d["punch"].as_i64())
                    .map(|v| v as i32)
                    .unwrap_or(-1);
                let status = staged
                    .raw_data
                    .as_ref()
                    .and_then(|d| d["status"].as_i64())
                    .map(|v| v as i32)
                    .unwrap_or(-1);

                let raw_log = RawAttendanceLog {
                    id: crate::utils::id::new_id(),
                    workspace_id,
                    employee_id,
                    punch_time: staged.punch_time,
                    source: LogSource::Device,
                    device_id: staged.device_id.clone(),
                    raw_data: staged.raw_data.clone(),
                    punch,
                    status,
                    created_at: Utc::now(),
                };

                let raw_log_entity =
                    crate::models::diesel::attendance_log::RawAttendanceLogEntity {
                        id: raw_log.id,
                        workspace_id: raw_log.workspace_id,
                        employee_id: raw_log.employee_id,
                        punch_time: raw_log.punch_time,
                        source: raw_log.source,
                        device_id: raw_log.device_id.clone(),
                        raw_data: raw_log.raw_data.clone(),
                        punch: raw_log.punch,
                        status: raw_log.status,
                        created_at: Some(raw_log.created_at),
                    };

                diesel::insert_into(raw_attendance_logs::table)
                    .values(&raw_log_entity)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db insert raw: {e}")))?;

                diesel::update(
                    attendance_staging_logs::table
                        .filter(attendance_staging_logs::id.eq(log_id))
                        .filter(attendance_staging_logs::workspace_id.eq(workspace_id)),
                )
                .set(attendance_staging_logs::status.eq("processed"))
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db update staging: {e}")))?;

                Ok(raw_log)
            })
        })
        .await
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
        let _repo = DieselStagingLogRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
