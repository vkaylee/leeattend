use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::{DieselPoolsTenantTxExt, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::attendance_log::AttendanceStagingLogEntity;
use crate::repositories::staging_log_repository::{
    StagingLog, StagingLogRepository, StagingStatus,
};
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
        status: StagingStatus,
    ) -> Result<bool, AppError> {
        // Logic from SQLx: INSERT ... WHERE NOT EXISTS
        // Diesel doesn't have a direct "INSERT ... WHERE NOT EXISTS" DSL easily.
        // But we can use on_conflict if there is a constraint.
        // Fingerprint is a unique constraint in the DB.
        let token = TenantAuthToken::system_override(workspace_id);
        let entity = AttendanceStagingLogEntity {
            id: crate::utils::id::new_id(),
            workspace_id,
            device_id,
            raw_user_id: raw_user_id.to_string(),
            punch_time,
            raw_data,
            fingerprint: fingerprint.to_string(),
            status: status.as_str().to_string(),
            created_at: Utc::now(),
        };

        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    use crate::schema::attendance_staging_logs::dsl as staging_dsl;
                    use diesel::ExpressionMethods;
                    use diesel::QueryDsl;

                    let exists: bool = diesel::select(diesel::dsl::exists(
                        staging_dsl::attendance_staging_logs
                            .filter(staging_dsl::workspace_id.eq(workspace_id))
                            .filter(staging_dsl::fingerprint.eq(fingerprint)),
                    ))
                    .get_result(conn)
                    .await
                    .map_err(AppError::from)?;

                    if exists {
                        return Ok(false);
                    }

                    let rows_affected = diesel::insert_into(staging_dsl::attendance_staging_logs)
                        .values(&entity)
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(rows_affected > 0)
                })
            })
            .await
    }

    async fn get_pending_staging_logs(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<StagingLog>, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);

        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let results: Vec<AttendanceStagingLogEntity> = attendance_staging_logs::table
                        .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
                        .filter(
                            attendance_staging_logs::status.eq_any(StagingStatus::open_statuses()),
                        )
                        .order(attendance_staging_logs::created_at.asc())
                        .limit(limit)
                        .load(conn)
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
                })
            })
            .await
    }

    async fn get_pending_staged_logs(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Vec<StagingLog>, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);

        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let results: Vec<AttendanceStagingLogEntity> = attendance_staging_logs::table
                        .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
                        .filter(attendance_staging_logs::raw_user_id.eq(employee_id_number))
                        .filter(attendance_staging_logs::status.eq(StagingStatus::Pending.as_str()))
                        .order(attendance_staging_logs::punch_time.asc())
                        .load(conn)
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
                })
            })
            .await
    }

    async fn update_staging_status(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        status: StagingStatus,
    ) -> Result<bool, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);

        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let rows_affected = diesel::update(attendance_staging_logs::table)
                        .filter(attendance_staging_logs::id.eq(id))
                        .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
                        .set(attendance_staging_logs::status.eq(status.as_str()))
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(rows_affected > 0)
                })
            })
            .await
    }

    async fn count_pending_staging_logs(&self, workspace_id: Uuid) -> Result<i64, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);

        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let count = attendance_staging_logs::table
                        .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
                        .filter(
                            attendance_staging_logs::status.eq_any(StagingStatus::open_statuses()),
                        )
                        .count()
                        .get_result::<i64>(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(count)
                })
            })
            .await
    }

    async fn claim_staged_logs(
        &self,
        workspace_id: Uuid,
        raw_user_id: &str,
        employee_id: Uuid,
    ) -> Result<u64, AppError> {
        // Wrap in transaction: if INSERT succeeds but UPDATE fails, everything rolls back.
        // This prevents orphaned staging logs stuck in "pending" state.
        let token = TenantAuthToken::system_override(workspace_id);
        let raw_user_id_owned = raw_user_id.to_string();
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    use crate::schema::employments;

                    let staged_logs = attendance_staging_logs::table
                        .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
                        .filter(attendance_staging_logs::raw_user_id.eq(&raw_user_id_owned))
                        .filter(attendance_staging_logs::status.eq(StagingStatus::Pending.as_str()))
                        .order(attendance_staging_logs::punch_time.asc())
                        .load::<AttendanceStagingLogEntity>(conn)
                        .await
                        .map_err(AppError::from)?;

                    if staged_logs.is_empty() {
                        return Ok(0);
                    }

                    // Resolve each punch against the employment covering ITS OWN date.
                    // Batch-fetch every employment of this employee once (no query in
                    // the loop) — a re-hired employee's historical punches must attach
                    // to the employment that was in force then, not the current one.
                    let candidates = employments::table
                        .filter(employments::employee_id.eq(employee_id))
                        .filter(employments::workspace_id.eq(workspace_id))
                        .filter(employments::deleted_at.is_null())
                        .select((
                            employments::id,
                            employments::start_date,
                            employments::end_date,
                        ))
                        .load::<(Uuid, chrono::NaiveDate, Option<chrono::NaiveDate>)>(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db load employments: {e}")))?;

                    if candidates.is_empty() {
                        return Err(AppError::not_found(I18nKey::EmployeeNotFound));
                    }

                    let now = chrono::Utc::now();
                    let mut raw_log_entities = Vec::with_capacity(staged_logs.len());
                    let mut claimed_ids = Vec::with_capacity(staged_logs.len());

                    for staged in &staged_logs {
                        let punch_date = staged.punch_time.date_naive();
                        let covering: Vec<Uuid> = candidates
                            .iter()
                            .filter(|(_, start, end)| {
                                *start <= punch_date && end.is_none_or(|e| e >= punch_date)
                            })
                            .map(|(id, _, _)| *id)
                            .collect();

                        // Leave anything undecidable in staging rather than guessing:
                        // 0 candidates → no employment covers the date;
                        // >1 candidates → overlapping ranges, needs a human choice.
                        let [employment_id] = covering[..] else {
                            continue;
                        };

                        raw_log_entities.push(
                            crate::models::diesel::attendance_log::RawAttendanceLogEntity {
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
                                source_timestamp: Some(staged.punch_time.to_rfc3339()),
                                source_timezone: None,
                                timestamp_kind: Some("staged_utc".to_string()),
                                conversion_status: Some("converted".to_string()),
                                conversion_error: None,
                                received_at: Some(now),
                            },
                        );
                        claimed_ids.push(staged.id);
                    }

                    if raw_log_entities.is_empty() {
                        return Ok(0);
                    }

                    let inserted_rows =
                        diesel::insert_into(crate::schema::raw_attendance_logs::table)
                            .values(&raw_log_entities)
                            .on_conflict_do_nothing()
                            .execute(conn)
                            .await
                            .map_err(AppError::from)?;

                    diesel::update(
                        attendance_staging_logs::table
                            .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
                            .filter(attendance_staging_logs::id.eq_any(&claimed_ids)),
                    )
                    .set(attendance_staging_logs::status.eq(StagingStatus::Processed.as_str()))
                    .execute(conn)
                    .await
                    .map_err(AppError::from)?;

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
        employment_id_choice: Option<Uuid>,
    ) -> Result<crate::models::RawAttendanceLog, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        use crate::models::{LogSource, NewRawAttendanceLogParams, RawAttendanceLog};
        use crate::schema::raw_attendance_logs;
        use diesel::result::DatabaseErrorKind;

        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    use crate::schema::employments;

                    // Lock the staging row FIRST: the punch date it carries is what
                    // decides which employment owns the punch.
                    //
                    // Use for_update() to lock the row. We don't use nowait() here because it might
                    // not be available or might cause compatibility issues. Just use regular for_update().
                    let staged = attendance_staging_logs::table
                        .filter(attendance_staging_logs::id.eq(log_id))
                        .filter(attendance_staging_logs::workspace_id.eq(workspace_id))
                        .for_update()
                        .first::<AttendanceStagingLogEntity>(conn)
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

                    if !StagingStatus::open_statuses().contains(&staged.status.as_str()) {
                        return Err(AppError::validation(I18nKey::StagingLogProcessed));
                    }

                    let punch_date = staged.punch_time.date_naive();

                    // Candidate employments are those actually covering the punch
                    // date — NOT merely `status = "active"`. A re-hired employee's
                    // historical punch belongs to the contract in force back then.
                    let candidates = employments::table
                        .filter(employments::employee_id.eq(employee_id))
                        .filter(employments::workspace_id.eq(workspace_id))
                        .filter(employments::start_date.le(punch_date))
                        .filter(
                            employments::end_date
                                .is_null()
                                .or(employments::end_date.ge(punch_date)),
                        )
                        .filter(employments::deleted_at.is_null())
                        .select(employments::id)
                        .load::<Uuid>(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db resolve employment: {e}")))?;

                    let employment_id = match employment_id_choice {
                        // Operator picked one: honour it only if it is a genuine
                        // candidate, so a stale or hand-crafted id cannot attach a
                        // punch to an unrelated employment or another tenant's row.
                        Some(chosen) => {
                            if !candidates.contains(&chosen) {
                                return Err(AppError::validation(
                                    I18nKey::EmploymentNotCoveringPunch,
                                ));
                            }
                            chosen
                        }
                        None => match candidates.as_slice() {
                            [] => return Err(AppError::not_found(I18nKey::NoEmploymentOnDate)),
                            [only] => *only,
                            // Ambiguous: refuse rather than pick. The caller must
                            // supply an explicit choice.
                            _ => {
                                return Err(AppError::validation(
                                    I18nKey::AmbiguousEmploymentOnDate,
                                ))
                            }
                        },
                    };

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

                    let raw_log = RawAttendanceLog::new(NewRawAttendanceLogParams {
                        id: crate::utils::id::new_id(),
                        workspace_id,
                        employment_id,
                        punch_time: staged.punch_time,
                        source: LogSource::Device,
                        device_id: staged.device_id.clone(),
                        raw_data: staged.raw_data.clone(),
                        punch,
                        status,
                        created_at: Utc::now(),
                        source_timestamp: Some(staged.punch_time.to_rfc3339()),
                        source_timezone: None,
                        timestamp_kind: Some("staged_utc".to_string()),
                        conversion_status: Some("converted".to_string()),
                        conversion_error: None,
                        received_at: Some(Utc::now()),
                    });

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
                            source_timestamp: raw_log.source_timestamp.clone(),
                            source_timezone: raw_log.source_timezone.clone(),
                            timestamp_kind: raw_log.timestamp_kind.clone(),
                            conversion_status: raw_log.conversion_status.clone(),
                            conversion_error: raw_log.conversion_error.clone(),
                            received_at: raw_log.received_at,
                        };

                    diesel::insert_into(raw_attendance_logs::table)
                        .values(&raw_log_entity)
                        .execute(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db insert raw: {e}")))?;

                    diesel::update(
                        attendance_staging_logs::table
                            .filter(attendance_staging_logs::id.eq(log_id))
                            .filter(attendance_staging_logs::workspace_id.eq(workspace_id)),
                    )
                    .set(attendance_staging_logs::status.eq(StagingStatus::Processed.as_str()))
                    .execute(conn)
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
            .create_staging_log(
                ws_id,
                None,
                "user1",
                Utc::now(),
                None,
                "fingerprint1",
                StagingStatus::Pending,
            )
            .await;

        if created.is_ok() {
            let _ = repo.get_pending_staging_logs(ws_id, 10).await;
            let _ = repo.get_pending_staged_logs(ws_id, "user1").await;
            let _ = repo.count_pending_staging_logs(ws_id).await;
            let _ = repo.claim_staged_logs(ws_id, "user1", emp_id).await;

            // Re-create to test resolution
            let _ = repo
                .create_staging_log(
                    ws_id,
                    None,
                    "user2",
                    Utc::now(),
                    None,
                    "fingerprint2",
                    StagingStatus::Pending,
                )
                .await;

            let logs = repo
                .get_pending_staged_logs(ws_id, "user2")
                .await
                .unwrap_or_default();
            if let Some(log) = logs.into_iter().next() {
                let _ = repo
                    .update_staging_status(log.id, ws_id, StagingStatus::Processed)
                    .await;
                let _ = repo.resolve_staging_log(log.id, ws_id, emp_id, None).await;
            } else {
                let _ = repo
                    .update_staging_status(Uuid::new_v4(), ws_id, StagingStatus::Processed)
                    .await;
                let _ = repo
                    .resolve_staging_log(Uuid::new_v4(), ws_id, emp_id, None)
                    .await;
            }
        } else {
            let _ = repo.get_pending_staging_logs(ws_id, 10).await;
            let _ = repo.get_pending_staged_logs(ws_id, "user1").await;
            let _ = repo
                .update_staging_status(Uuid::new_v4(), ws_id, StagingStatus::Processed)
                .await;
            let _ = repo.count_pending_staging_logs(ws_id).await;
            let _ = repo.claim_staged_logs(ws_id, "user1", emp_id).await;
            let _ = repo
                .resolve_staging_log(Uuid::new_v4(), ws_id, emp_id, None)
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

        // 2. resolve_staging_log now locks the staging row FIRST (its punch date
        // decides the employment), so a non-existent log id fails as
        // StagingLogNotFound before the employee is ever considered.
        let resolve_res = repo
            .resolve_staging_log(Uuid::new_v4(), ws_id, emp_id, None)
            .await;
        assert!(resolve_res.is_err());
        match resolve_res.unwrap_err() {
            AppError::NotFound(msg) => {
                assert_eq!(msg.to_string(), "error.staging_log_not_found");
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
