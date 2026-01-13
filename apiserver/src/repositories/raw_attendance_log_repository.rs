use sqlx::{query, query_as, query_scalar, PgPool};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::RawAttendanceLog;

pub async fn create_raw_log(
    pool: &PgPool,
    log: &RawAttendanceLog,
) -> Result<(), sqlx::Error> {
    query(
        "INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source, device_id, raw_data, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(log.id)
    .bind(log.workspace_id)
    .bind(log.employee_id)
    .bind(log.punch_time)
    .bind(&log.source)
    .bind(&log.device_id)
    .bind(&log.raw_data)
    .bind(log.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_latest_log(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Option<RawAttendanceLog>, sqlx::Error> {
    query_as::<_, RawAttendanceLog>(
        "SELECT * FROM raw_attendance_logs WHERE employee_id = $1 ORDER BY punch_time DESC LIMIT 1"
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
    let mut sql = "SELECT * FROM raw_attendance_logs WHERE workspace_id = $1".to_string();
    let mut param_idx = 2;

    if p_employee_id.is_some() {
        sql.push_str(&format!(" AND employee_id = ${}", param_idx));
        param_idx += 1;
    }
    if p_from.is_some() {
        sql.push_str(&format!(" AND punch_time >= ${}", param_idx));
        param_idx += 1;
    }
    if p_to.is_some() {
        sql.push_str(&format!(" AND punch_time <= ${}", param_idx));
        param_idx += 1;
    }

    sql.push_str(&format!(" ORDER BY punch_time DESC LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));

    let mut query = query_as::<_, RawAttendanceLog>(&sql).bind(workspace_id);
    
    if let Some(id) = p_employee_id { query = query.bind(id); }
    if let Some(f) = p_from { query = query.bind(f); }
    if let Some(t) = p_to { query = query.bind(t); }
    
    query.bind(p_limit).bind(p_offset).fetch_all(pool).await
}

pub async fn count_raw_logs(
    pool: &PgPool,
    workspace_id: Uuid,
    p_employee_id: Option<Uuid>,
    p_from: Option<DateTime<Utc>>,
    p_to: Option<DateTime<Utc>>,
) -> Result<i64, sqlx::Error> {
    let mut sql = "SELECT COUNT(*) FROM raw_attendance_logs WHERE workspace_id = $1".to_string();
    let mut param_idx = 2;

    if p_employee_id.is_some() {
        sql.push_str(&format!(" AND employee_id = ${}", param_idx));
        param_idx += 1;
    }
    if p_from.is_some() {
        sql.push_str(&format!(" AND punch_time >= ${}", param_idx));
        param_idx += 1;
    }
    if p_to.is_some() {
        sql.push_str(&format!(" AND punch_time <= ${}", param_idx));
    }

    let mut query = query_scalar::<_, i64>(&sql).bind(workspace_id);
    
    if let Some(id) = p_employee_id { query = query.bind(id); }
    if let Some(f) = p_from { query = query.bind(f); }
    if let Some(t) = p_to { query = query.bind(t); }
    
    query.fetch_one(pool).await
}
