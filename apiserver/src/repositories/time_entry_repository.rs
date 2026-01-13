use sqlx::{query, query_as, query_scalar, PgPool};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::TimeEntry;

pub async fn create_time_entry(
    pool: &PgPool,
    entry: &TimeEntry,
) -> Result<(), sqlx::Error> {
    query(
        "INSERT INTO time_entries (id, employee_id, shift_id, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(entry.id)
    .bind(entry.employee_id)
    .bind(entry.shift_id)
    .bind(entry.punch_in_time)
    .bind(entry.punch_out_time)
    .bind(entry.duration_minutes)
    .bind(entry.is_break_overtime)
    .bind(&entry.entry_type)
    .bind(&entry.note)
    .bind(entry.created_at)
    .bind(entry.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_time_entry_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<TimeEntry>, sqlx::Error> {
    query_as::<_, TimeEntry>(
        "SELECT * FROM time_entries WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_active_time_entry(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Option<TimeEntry>, sqlx::Error> {
    query_as::<_, TimeEntry>(
        "SELECT * FROM time_entries WHERE employee_id = $1 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1"
    )
    .bind(employee_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_latest_active_entry(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Option<TimeEntry>, sqlx::Error> {
    query_as::<_, TimeEntry>(
        "SELECT * FROM time_entries WHERE employee_id = $1 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1"
    )
    .bind(employee_id)
    .fetch_optional(pool)
    .await
}

pub async fn update_time_entry(
    pool: &PgPool,
    entry: &TimeEntry,
) -> Result<(), sqlx::Error> {
    query(
        "UPDATE time_entries SET punch_in_time = $1, punch_out_time = $2, duration_minutes = $3, is_break_overtime = $4, note = $5, updated_at = $6 WHERE id = $7"
    )
    .bind(entry.punch_in_time)
    .bind(entry.punch_out_time)
    .bind(entry.duration_minutes)
    .bind(entry.is_break_overtime)
    .bind(&entry.note)
    .bind(entry.updated_at)
    .bind(entry.id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_time_entry(
    pool: &PgPool,
    id: Uuid,
) -> Result<(), sqlx::Error> {
    query(
        "DELETE FROM time_entries WHERE id = $1"
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_employee_time_entries(
    pool: &PgPool,
    employee_id: Uuid,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<TimeEntry>, sqlx::Error> {
    query_as::<_, TimeEntry>(
        "SELECT * FROM time_entries WHERE employee_id = $1 AND punch_in_time >= $2 AND punch_in_time <= $3 ORDER BY punch_in_time DESC"
    )
    .bind(employee_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

pub async fn get_paginated_logs(
    pool: &PgPool,
    workspace_id: Uuid,
    p_employee_id: Option<Uuid>,
    p_from: Option<DateTime<Utc>>,
    p_to: Option<DateTime<Utc>>,
    p_limit: i64,
    p_offset: i64,
) -> Result<Vec<TimeEntry>, sqlx::Error> {
    let mut sql = "SELECT te.* FROM time_entries te 
                  JOIN employees e ON te.employee_id = e.id 
                  WHERE e.workspace_id = $1".to_string();
    let mut param_idx = 2;

    if p_employee_id.is_some() {
        sql.push_str(&format!(" AND te.employee_id = ${}", param_idx));
        param_idx += 1;
    }
    if p_from.is_some() {
        sql.push_str(&format!(" AND te.punch_in_time >= ${}", param_idx));
        param_idx += 1;
    }
    if p_to.is_some() {
        sql.push_str(&format!(" AND te.punch_in_time <= ${}", param_idx));
        param_idx += 1;
    }

    sql.push_str(&format!(" ORDER BY te.punch_in_time DESC LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));

    let mut query = query_as::<_, TimeEntry>(&sql).bind(workspace_id);
    
    if let Some(id) = p_employee_id { query = query.bind(id); }
    if let Some(f) = p_from { query = query.bind(f); }
    if let Some(t) = p_to { query = query.bind(t); }
    
    query.bind(p_limit).bind(p_offset).fetch_all(pool).await
}

pub async fn count_logs(
    pool: &PgPool,
    workspace_id: Uuid,
    p_employee_id: Option<Uuid>,
    p_from: Option<DateTime<Utc>>,
    p_to: Option<DateTime<Utc>>,
) -> Result<i64, sqlx::Error> {
    let mut sql = "SELECT COUNT(*) FROM time_entries te 
                  JOIN employees e ON te.employee_id = e.id 
                  WHERE e.workspace_id = $1".to_string();
    let mut param_idx = 2;

    if p_employee_id.is_some() {
        sql.push_str(&format!(" AND te.employee_id = ${}", param_idx));
        param_idx += 1;
    }
    if p_from.is_some() {
        sql.push_str(&format!(" AND te.punch_in_time >= ${}", param_idx));
        param_idx += 1;
    }
    if p_to.is_some() {
        sql.push_str(&format!(" AND te.punch_in_time <= ${}", param_idx));
        // NO param_idx += 1 here as it's the last one
    }

    let mut query = query_scalar::<_, i64>(&sql).bind(workspace_id);
    
    if let Some(id) = p_employee_id { query = query.bind(id); }
    if let Some(f) = p_from { query = query.bind(f); }
    if let Some(t) = p_to { query = query.bind(t); }
    
    query.fetch_one(pool).await
}
