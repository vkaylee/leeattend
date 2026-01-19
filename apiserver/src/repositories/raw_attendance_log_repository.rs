use crate::models::RawAttendanceLog;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

pub async fn create_raw_log(pool: &PgPool, log: &RawAttendanceLog) -> Result<(), sqlx::Error> {
    query(
        "INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source, device_id, raw_data, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(log.id)
    .bind(log.workspace_id)
    .bind(log.employee_id)
    .bind(log.punch_time)
    .bind(log.source)
    .bind(&log.device_id)
    .bind(&log.raw_data)
    .bind(log.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_raw_log_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<RawAttendanceLog>, sqlx::Error> {
    query_as::<_, RawAttendanceLog>("SELECT * FROM raw_attendance_logs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_latest_log(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Option<RawAttendanceLog>, sqlx::Error> {
    query_as::<_, RawAttendanceLog>(
        "SELECT * FROM raw_attendance_logs WHERE employee_id = $1 ORDER BY punch_time DESC LIMIT 1",
    )
    .bind(employee_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_paginated_raw_logs(
    pool: &PgPool,
    workspace_id: Uuid,
    p_employee_id: Option<Uuid>,
    p_from: Option<DateTime<Utc>>,
    p_to: Option<DateTime<Utc>>,
    p_limit: i64,
    p_offset: i64,
) -> Result<Vec<RawAttendanceLog>, sqlx::Error> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> =
        sqlx::QueryBuilder::new("SELECT * FROM raw_attendance_logs WHERE workspace_id = ");
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

    query_builder.push(" ORDER BY punch_time DESC LIMIT ");
    query_builder.push_bind(p_limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(p_offset);

    query_builder
        .build_query_as::<RawAttendanceLog>()
        .fetch_all(pool)
        .await
}

pub async fn count_raw_logs(
    pool: &PgPool,
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
        .fetch_one(pool)
        .await
        .map(|r| r.0)
}
