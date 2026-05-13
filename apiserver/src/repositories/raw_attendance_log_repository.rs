use crate::errors::AppError;
use crate::models::{RawAttendanceLog, RawAttendanceLogWithEmployee};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, QueryBuilder};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: RawAttendanceLogRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait RawAttendanceLogRepository: Send + Sync {
    async fn create_raw_log(&self, log: &RawAttendanceLog) -> Result<bool, AppError>;
    async fn get_raw_log_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<RawAttendanceLog>, AppError>;
    async fn get_latest_log(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<RawAttendanceLog>, AppError>;
    async fn get_paginated_raw_logs(
        &self,
        workspace_id: Uuid,
        p_employee_id: Option<Uuid>,
        p_from: Option<DateTime<Utc>>,
        p_to: Option<DateTime<Utc>>,
        p_limit: i64,
        p_offset: i64,
    ) -> Result<Vec<RawAttendanceLogWithEmployee>, AppError>;
    async fn count_raw_logs(
        &self,
        workspace_id: Uuid,
        p_employee_id: Option<Uuid>,
        p_from: Option<DateTime<Utc>>,
        p_to: Option<DateTime<Utc>>,
    ) -> Result<i64, AppError>;
    async fn create_raw_logs_batch(&self, logs: &[&RawAttendanceLog]) -> Result<usize, AppError>;
    async fn has_duplicate_raw_log(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        log_id: Uuid,
        punch_time: DateTime<Utc>,
    ) -> Result<bool, AppError>;
    async fn get_employee_logs_in_time_range(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<RawAttendanceLog>, AppError>;
}

pub async fn create_raw_log<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    log: &RawAttendanceLog,
) -> Result<bool, sqlx::Error> {
    let result = query(
        "INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source, device_id, raw_data, created_at, punch, status) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (workspace_id, employee_id, punch_time, punch, status) DO NOTHING"
    )
    .bind(log.id)
    .bind(log.workspace_id)
    .bind(log.employee_id)
    .bind(log.punch_time)
    .bind(log.source)
    .bind(&log.device_id)
    .bind(&log.raw_data)
    .bind(log.created_at)
    .bind(log.punch)
    .bind(log.status)
    .execute(executor)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_raw_log_by_id<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    workspace_id: Uuid,
    id: Uuid,
) -> Result<Option<RawAttendanceLog>, sqlx::Error> {
    query_as::<_, RawAttendanceLog>(
        "SELECT * FROM raw_attendance_logs WHERE id = $1 AND workspace_id = $2",
    )
    .bind(id)
    .bind(workspace_id)
    .fetch_optional(executor)
    .await
}

pub async fn get_latest_log<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    workspace_id: Uuid,
    employee_id: Uuid,
) -> Result<Option<RawAttendanceLog>, sqlx::Error> {
    query_as::<_, RawAttendanceLog>(
        "SELECT * FROM raw_attendance_logs WHERE employee_id = $1 AND workspace_id = $2 ORDER BY punch_time DESC LIMIT 1",
    )
    .bind(employee_id)
    .bind(workspace_id)
    .fetch_optional(executor)
    .await
}

pub async fn get_paginated_raw_logs<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    workspace_id: Uuid,
    p_employee_id: Option<Uuid>,
    p_from: Option<DateTime<Utc>>,
    p_to: Option<DateTime<Utc>>,
    p_limit: i64,
    p_offset: i64,
) -> Result<Vec<RawAttendanceLogWithEmployee>, sqlx::Error> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        r#"
            SELECT 
                rl.*, 
                e.full_name as employee_full_name, 
                e.employee_id_number,
                d.name as device_name
            FROM raw_attendance_logs rl
            JOIN employees e ON rl.employee_id = e.id
            LEFT JOIN devices d ON rl.device_id = d.id::text
            WHERE rl.workspace_id = 
        "#,
    );
    query_builder.push_bind(workspace_id);

    if let Some(emp_id) = p_employee_id {
        query_builder.push(" AND rl.employee_id = ");
        query_builder.push_bind(emp_id);
    }

    if let Some(from) = p_from {
        query_builder.push(" AND rl.punch_time >= ");
        query_builder.push_bind(from);
    }

    if let Some(to) = p_to {
        query_builder.push(" AND rl.punch_time <= ");
        query_builder.push_bind(to);
    }

    query_builder.push(" ORDER BY rl.punch_time DESC LIMIT ");
    query_builder.push_bind(p_limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(p_offset);

    query_builder
        .build_query_as::<RawAttendanceLogWithEmployee>()
        .fetch_all(executor)
        .await
}

pub async fn count_raw_logs<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    workspace_id: Uuid,
    p_employee_id: Option<Uuid>,
    p_from: Option<DateTime<Utc>>,
    p_to: Option<DateTime<Utc>>,
) -> Result<i64, sqlx::Error> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> =
        sqlx::QueryBuilder::new("SELECT COUNT(*) FROM raw_attendance_logs WHERE workspace_id = ");
    query_builder.push_bind(workspace_id);

    if let Some(emp_id) = p_employee_id {
        query_builder.push(" AND employee_id = ");
        query_builder.push_bind(emp_id);
    }

    if let Some(from) = p_from {
        query_builder.push(" AND punch_time >= ");
        query_builder.push_bind(from);
    }

    if let Some(to) = p_to {
        query_builder.push(" AND punch_time <= ");
        query_builder.push_bind(to);
    }

    query_builder
        .build_query_as::<(i64,)>()
        .fetch_one(executor)
        .await
        .map(|r| r.0)
}

/// Batch insert multiple raw attendance logs
/// Uses ON CONFLICT DO NOTHING to skip duplicates
/// Returns count of actually inserted rows
pub async fn create_raw_logs_batch<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    logs: &[&RawAttendanceLog],
) -> Result<usize, sqlx::Error> {
    if logs.is_empty() {
        return Ok(0);
    }

    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source, device_id, raw_data, created_at, punch, status) ",
    );

    query_builder.push_values(logs.iter(), |mut b, log| {
        b.push_bind(log.id)
            .push_bind(log.workspace_id)
            .push_bind(log.employee_id)
            .push_bind(log.punch_time)
            .push_bind(log.source)
            .push_bind(&log.device_id)
            .push_bind(&log.raw_data)
            .push_bind(log.created_at)
            .push_bind(log.punch)
            .push_bind(log.status);
    });

    query_builder
        .push(" ON CONFLICT (workspace_id, employee_id, punch_time, punch, status) DO NOTHING");

    let result = query_builder.build().execute(executor).await?;

    Ok(result.rows_affected() as usize)
}
