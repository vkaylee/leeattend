use chrono::{DateTime, Utc};
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

use crate::models::TimeEntry;

pub async fn create_time_entry(pool: &PgPool, entry: &TimeEntry) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO time_entries (id, employee_id, shift_id, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        entry.id,
        entry.employee_id,
        entry.shift_id,
        entry.punch_in_time,
        entry.punch_out_time,
        entry.duration_minutes,
        entry.is_break_overtime,
        entry.entry_type,
        entry.note,
        entry.created_at,
        entry.updated_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_time_entry_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<TimeEntry>, sqlx::Error> {
    query_as!(
        TimeEntry,
        r#"SELECT id, employee_id, shift_id, punch_in_time as "punch_in_time!", punch_out_time, duration_minutes, is_break_overtime, entry_type, note, created_at as "created_at!", updated_at as "updated_at!" FROM time_entries WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_latest_active_entry(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Option<TimeEntry>, sqlx::Error> {
    query_as!(
        TimeEntry,
        r#"SELECT id, employee_id, shift_id, punch_in_time as "punch_in_time!", punch_out_time, duration_minutes, is_break_overtime, entry_type, note, created_at as "created_at!", updated_at as "updated_at!" FROM time_entries WHERE employee_id = $1 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1"#,
        employee_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_time_entry(pool: &PgPool, entry: &TimeEntry) -> Result<(), sqlx::Error> {
    query!(
        "UPDATE time_entries SET
            shift_id = $1,
            punch_in_time = $2,
            punch_out_time = $3,
            duration_minutes = $4,
            is_break_overtime = $5,
            entry_type = $6,
            note = $7,
            updated_at = $8
        WHERE id = $9",
        entry.shift_id,
        entry.punch_in_time,
        entry.punch_out_time,
        entry.duration_minutes,
        entry.is_break_overtime,
        entry.entry_type,
        entry.note,
        Utc::now(),
        entry.id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_employee_time_entries(
    pool: &PgPool,
    employee_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Vec<TimeEntry>, sqlx::Error> {
    query_as!(
        TimeEntry,
        r#"SELECT id, employee_id, shift_id, punch_in_time as "punch_in_time!", punch_out_time, duration_minutes, is_break_overtime, entry_type, note, created_at as "created_at!", updated_at as "updated_at!" FROM time_entries WHERE employee_id = $1 AND punch_in_time >= $2 AND punch_in_time <= $3 ORDER BY punch_in_time DESC"#,
        employee_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_time_entry(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = query!("DELETE FROM time_entries WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}