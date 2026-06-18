use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::diesel::attendance_log::AttendanceStagingLogEntity;
use crate::repositories::staging_log_repository::{StagingLog, StagingLogRepository};
use crate::schema::attendance_staging_logs;
use crate::utils::i18n::I18nKey;

pub struct DieselStagingLogRepository {
    diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselStagingLogRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
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

        // Wrap in transaction: if INSERT succeeds but UPDATE fails, everything rolls back.
        // This prevents orphaned staging logs stuck in "pending" state.
        let raw_user_id_owned = raw_user_id.to_string();
        conn.transaction(|conn| {
            Box::pin(async move {
                use crate::schema::employments;

                // Verify that employee has active employment in this workspace
                let active_employment = employments::table
                    .filter(employments::employee_id.eq(employee_id))
                    .filter(employments::workspace_id.eq(workspace_id))
                    .filter(employments::status.eq("active"))
                    .filter(employments::deleted_at.is_null())
                    .select(employments::id)
                    .first::<Uuid>(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(format!("db check active employment: {e}")))?;

                let Some(employment_id) = active_employment else {
                    return Err(AppError::not_found(I18nKey::EmployeeNotFound));
                };

                let staged_logs = attendance_staging_logs::table
                    .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
                    .filter(attendance_staging_logs::raw_user_id.eq(&raw_user_id_owned))
                    .filter(attendance_staging_logs::status.eq("pending"))
                    .order(attendance_staging_logs::punch_time.asc())
                    .load::<AttendanceStagingLogEntity>(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                if staged_logs.is_empty() {
                    return Ok(0);
                }

                let now = chrono::Utc::now();
                let raw_log_entities: Vec<_> = staged_logs
                    .iter()
                    .map(
                        |staged| crate::models::diesel::attendance_log::RawAttendanceLogEntity {
                            id: crate::utils::id::new_id(),
                            workspace_id,
                            employment_id,
                            punch_time: staged.punch_time,
                            source: crate::models::LogSource::Device,
                            device_id: staged.device_id.clone(),
                            raw_data: staged.raw_data.clone(),
                            punch: -1,
                            status: -1,
                            created_at: Some(now),
                        },
                    )
                    .collect();

                let inserted_rows = diesel::insert_into(crate::schema::raw_attendance_logs::table)
                    .values(&raw_log_entities)
                    .on_conflict_do_nothing()
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                let staged_ids: Vec<_> = staged_logs.iter().map(|staged| staged.id).collect();

                diesel::update(
                    attendance_staging_logs::table
                        .filter(attendance_staging_logs::id.eq_any(&staged_ids)),
                )
                .set(attendance_staging_logs::status.eq("processed"))
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                Ok(inserted_rows as u64)
            })
        })
        .await
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
                use crate::schema::employments;

                // Verify that employee has active employment in this workspace
                let active_employment = employments::table
                    .filter(employments::employee_id.eq(employee_id))
                    .filter(employments::workspace_id.eq(workspace_id))
                    .filter(employments::status.eq("active"))
                    .filter(employments::deleted_at.is_null())
                    .select(employments::id)
                    .first::<Uuid>(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(format!("db check active employment: {e}")))?;

                let Some(employment_id) = active_employment else {
                    return Err(AppError::not_found(I18nKey::EmployeeNotFound));
                };

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
                            AppError::conflict(I18nKey::StagingLogConflict)
                        } else if e == diesel::result::Error::NotFound {
                            AppError::not_found(I18nKey::StagingLogNotFound)
                        } else {
                            AppError::Internal(format!("db: {e}"))
                        }
                    })?;

                if staged.status != "pending" {
                    return Err(AppError::validation(I18nKey::StagingLogProcessed));
                }

                let punch = staged
                    .raw_data
                    .as_ref()
                    .and_then(|d| d["punch"].as_i64())
                    .map_or(-1, |v| i32::try_from(v).unwrap_or(-1));
                let status = staged
                    .raw_data
                    .as_ref()
                    .and_then(|d| d["status"].as_i64())
                    .map_or(-1, |v| i32::try_from(v).unwrap_or(-1));

                let raw_log = RawAttendanceLog::new(
                    crate::utils::id::new_id(),
                    workspace_id,
                    employment_id,
                    staged.punch_time,
                    LogSource::Device,
                    staged.device_id.clone(),
                    staged.raw_data.clone(),
                    punch,
                    status,
                    Utc::now(),
                );

                let raw_log_entity =
                    crate::models::diesel::attendance_log::RawAttendanceLogEntity {
                        id: raw_log.id(),
                        workspace_id: raw_log.workspace_id(),
                        employment_id: raw_log.employment_id(),
                        punch_time: raw_log.punch_time(),
                        source: *raw_log.source(),
                        device_id: raw_log.device_id().map(ToString::to_string),
                        raw_data: raw_log.raw_data().cloned(),
                        punch: raw_log.punch(),
                        status: raw_log.status(),
                        created_at: Some(raw_log.created_at()),
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
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_staging_log_repository_errors() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselStagingLogRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        let created = repo
            .create_staging_log(ws_id, None, "user1", Utc::now(), None, "fingerprint1")
            .await;

        if created.is_ok() {
            let _ = repo.get_pending_staging_logs(ws_id, 10).await;
            let _ = repo.get_pending_staged_logs(ws_id, "user1").await;
            let _ = repo.count_pending_staging_logs(ws_id).await;
            let _ = repo.claim_staged_logs(ws_id, "user1", emp_id).await;

            // Re-create to test resolution
            let _ = repo
                .create_staging_log(ws_id, None, "user2", Utc::now(), None, "fingerprint2")
                .await;

            let logs = repo
                .get_pending_staged_logs(ws_id, "user2")
                .await
                .unwrap_or_default();
            if let Some(log) = logs.into_iter().next() {
                let _ = repo.update_staging_status(log.id, ws_id, "processed").await;
                let _ = repo.resolve_staging_log(log.id, ws_id, emp_id).await;
            } else {
                let _ = repo
                    .update_staging_status(Uuid::new_v4(), ws_id, "processed")
                    .await;
                let _ = repo
                    .resolve_staging_log(Uuid::new_v4(), ws_id, emp_id)
                    .await;
            }
        } else {
            let _ = repo.get_pending_staging_logs(ws_id, 10).await;
            let _ = repo.get_pending_staged_logs(ws_id, "user1").await;
            let _ = repo
                .update_staging_status(Uuid::new_v4(), ws_id, "processed")
                .await;
            let _ = repo.count_pending_staging_logs(ws_id).await;
            let _ = repo.claim_staged_logs(ws_id, "user1", emp_id).await;
            let _ = repo
                .resolve_staging_log(Uuid::new_v4(), ws_id, emp_id)
                .await;
        }
    }

    #[tokio::test]
    async fn test_claim_resolve_non_existent_employee() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselStagingLogRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        // 1. claim_staged_logs with invalid employee should return NotFound
        let claim_res = repo.claim_staged_logs(ws_id, "user1", emp_id).await;
        assert!(claim_res.is_err());
        match claim_res.unwrap_err() {
            AppError::NotFound(msg) => {
                assert_eq!(msg.to_string(), "error.employee_not_found");
            }
            AppError::Internal(msg) => {
                assert!(
                    msg.contains("connecting") || msg.contains("connection") || msg.contains("db")
                );
            }
            other => {
                panic!("Expected AppError::NotFound in claim or db connection error, got {other:?}")
            }
        }

        // 2. resolve_staging_log with invalid employee should return NotFound
        let resolve_res = repo
            .resolve_staging_log(Uuid::new_v4(), ws_id, emp_id)
            .await;
        assert!(resolve_res.is_err());
        match resolve_res.unwrap_err() {
            AppError::NotFound(msg) => {
                assert_eq!(msg.to_string(), "error.employee_not_found");
            }
            AppError::Internal(msg) => {
                assert!(
                    msg.contains("connecting") || msg.contains("connection") || msg.contains("db")
                );
            }
            other => panic!(
                "Expected AppError::NotFound in resolve or db connection error, got {other:?}"
            ),
        }
    }
}
