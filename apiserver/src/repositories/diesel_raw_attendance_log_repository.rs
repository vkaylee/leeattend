use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::attendance::{RawAttendanceLog, RawAttendanceLogWithEmployee};
use crate::models::diesel::attendance_log::RawAttendanceLogEntity;
use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::schema::{devices, employees, raw_attendance_logs};

pub struct DieselRawAttendanceLogRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselRawAttendanceLogRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl RawAttendanceLogRepository for DieselRawAttendanceLogRepository {
    async fn create_raw_log(
        &self,
        workspace_id: Uuid,
        log: &RawAttendanceLog,
    ) -> Result<bool, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = RawAttendanceLogEntity {
            id: log.id,
            workspace_id: log.workspace_id,
            employee_id: log.employee_id,
            punch_time: log.punch_time,
            source: log.source,
            device_id: log.device_id.clone(),
            raw_data: log.raw_data.clone(),
            created_at: Some(log.created_at),
            punch: log.punch,
            status: log.status,
        };

        let rows_affected = diesel::insert_into(raw_attendance_logs::table)
            .values(&entity)
            .on_conflict((
                raw_attendance_logs::workspace_id,
                raw_attendance_logs::employee_id,
                raw_attendance_logs::punch_time,
                raw_attendance_logs::punch,
                raw_attendance_logs::status,
            ))
            .do_nothing()
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows_affected > 0)
    }

    async fn get_raw_log_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<RawAttendanceLog>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let res: Option<RawAttendanceLogEntity> = raw_attendance_logs::table
            .filter(raw_attendance_logs::id.eq(id))
            .filter(raw_attendance_logs::workspace_id.eq(workspace_id))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(res.map(Into::into))
    }

    async fn get_latest_log(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<RawAttendanceLog>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let res: Option<RawAttendanceLogEntity> = raw_attendance_logs::table
            .filter(raw_attendance_logs::employee_id.eq(employee_id))
            .filter(raw_attendance_logs::workspace_id.eq(workspace_id))
            .order(raw_attendance_logs::punch_time.desc())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(res.map(Into::into))
    }

    async fn get_paginated_raw_logs(
        &self,
        workspace_id: Uuid,
        p_employee_id: Option<Uuid>,
        p_from: Option<DateTime<Utc>>,
        p_to: Option<DateTime<Utc>>,
        p_limit: i64,
        p_offset: i64,
    ) -> Result<Vec<RawAttendanceLogWithEmployee>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = raw_attendance_logs::table
            .inner_join(employees::table.on(raw_attendance_logs::employee_id.eq(employees::id)))
            .left_join(
                devices::table.on(diesel::dsl::sql::<diesel::sql_types::Bool>(
                    "raw_attendance_logs.device_id = devices.id::text",
                )),
            )
            .filter(raw_attendance_logs::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(emp_id) = p_employee_id {
            query = query.filter(raw_attendance_logs::employee_id.eq(emp_id));
        }

        if let Some(from) = p_from {
            query = query.filter(raw_attendance_logs::punch_time.ge(from));
        }

        if let Some(to) = p_to {
            query = query.filter(raw_attendance_logs::punch_time.le(to));
        }

        // Custom selection to match RawAttendanceLogWithEmployee
        let results = query
            .select((
                raw_attendance_logs::all_columns,
                employees::full_name,
                employees::employee_id_number,
                devices::name.nullable(),
            ))
            .order(raw_attendance_logs::punch_time.desc())
            .limit(p_limit)
            .offset(p_offset)
            .load::<(RawAttendanceLogEntity, String, String, Option<String>)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|(log, emp_name, emp_id_num, dev_name)| {
                let base: RawAttendanceLog = log.into();
                RawAttendanceLogWithEmployee {
                    id: base.id,
                    workspace_id: base.workspace_id,
                    employee_id: base.employee_id,
                    punch_time: base.punch_time,
                    source: base.source,
                    device_id: base.device_id,
                    raw_data: base.raw_data,
                    punch: base.punch,
                    status: base.status,
                    created_at: base.created_at,
                    employee_full_name: emp_name,
                    employee_id_number: emp_id_num,
                    device_name: dev_name,
                }
            })
            .collect())
    }

    async fn count_raw_logs(
        &self,
        workspace_id: Uuid,
        p_employee_id: Option<Uuid>,
        p_from: Option<DateTime<Utc>>,
        p_to: Option<DateTime<Utc>>,
    ) -> Result<i64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = raw_attendance_logs::table
            .filter(raw_attendance_logs::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(emp_id) = p_employee_id {
            query = query.filter(raw_attendance_logs::employee_id.eq(emp_id));
        }

        if let Some(from) = p_from {
            query = query.filter(raw_attendance_logs::punch_time.ge(from));
        }

        if let Some(to) = p_to {
            query = query.filter(raw_attendance_logs::punch_time.le(to));
        }

        let count = query
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(count)
    }

    async fn create_raw_logs_batch(
        &self,
        workspace_id: Uuid,
        logs: &[&RawAttendanceLog],
    ) -> Result<usize, AppError> {
        if logs.is_empty() {
            return Ok(0);
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities: Vec<RawAttendanceLogEntity> = logs
            .iter()
            .map(|log| RawAttendanceLogEntity {
                id: log.id,
                workspace_id: log.workspace_id,
                employee_id: log.employee_id,
                punch_time: log.punch_time,
                source: log.source,
                device_id: log.device_id.clone(),
                raw_data: log.raw_data.clone(),
                created_at: Some(log.created_at),
                punch: log.punch,
                status: log.status,
            })
            .collect();

        let rows_affected = diesel::insert_into(raw_attendance_logs::table)
            .values(&entities)
            .on_conflict((
                raw_attendance_logs::workspace_id,
                raw_attendance_logs::employee_id,
                raw_attendance_logs::punch_time,
                raw_attendance_logs::punch,
                raw_attendance_logs::status,
            ))
            .do_nothing()
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows_affected)
    }

    async fn has_duplicate_raw_log(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        log_id: Uuid,
        punch_time: DateTime<Utc>,
    ) -> Result<bool, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let duplicate_exists = diesel::select(diesel::dsl::exists(
            raw_attendance_logs::table
                .filter(raw_attendance_logs::workspace_id.eq(workspace_id))
                .filter(raw_attendance_logs::employee_id.eq(employee_id))
                .filter(raw_attendance_logs::id.ne(log_id))
                .filter(raw_attendance_logs::punch_time.eq(punch_time)),
        ))
        .get_result::<bool>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(duplicate_exists)
    }

    async fn get_employee_logs_in_time_range(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<RawAttendanceLog>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results = raw_attendance_logs::table
            .filter(raw_attendance_logs::workspace_id.eq(workspace_id))
            .filter(raw_attendance_logs::employee_id.eq(employee_id))
            .filter(raw_attendance_logs::punch_time.ge(start_time))
            .filter(raw_attendance_logs::punch_time.lt(end_time))
            .order(raw_attendance_logs::punch_time.asc())
            .load::<RawAttendanceLogEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
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
        let _repo = DieselRawAttendanceLogRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
