use async_trait::async_trait;
use crate::models::{TimeEntry, TimeEntryWithEmployee};
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, query_scalar, PgPool};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: TimeEntryRepository — DIP abstraction over time entry data access
// ---------------------------------------------------------------------------
#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait TimeEntryRepository: Send + Sync {
    async fn create_time_entry(&self, entry: &TimeEntry) -> Result<(), sqlx::Error>;
    async fn get_time_entry_by_id(&self, workspace_id: Uuid, id: Uuid) -> Result<Option<TimeEntry>, sqlx::Error>;
    async fn get_active_time_entry(&self, employee_id: Uuid) -> Result<Option<TimeEntry>, sqlx::Error>;
    async fn get_latest_active_entry(&self, employee_id: Uuid) -> Result<Option<TimeEntry>, sqlx::Error>;
    async fn update_time_entry(&self, workspace_id: Uuid, entry: &TimeEntry) -> Result<(), sqlx::Error>;
    async fn delete_time_entry(&self, workspace_id: Uuid, id: Uuid) -> Result<(), sqlx::Error>;
    async fn get_employee_time_entries(&self, employee_id: Uuid, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<TimeEntry>, sqlx::Error>;
    async fn get_paginated_logs(&self, workspace_id: Uuid, p_employee_id: Option<Uuid>, p_from: Option<DateTime<Utc>>, p_to: Option<DateTime<Utc>>, p_limit: i64, p_offset: i64) -> Result<Vec<TimeEntryWithEmployee>, sqlx::Error>;
    async fn count_logs(&self, workspace_id: Uuid, p_employee_id: Option<Uuid>, p_from: Option<DateTime<Utc>>, p_to: Option<DateTime<Utc>>) -> Result<i64, sqlx::Error>;
    async fn get_workspace_time_entries_batch(&self, employee_ids: &[Uuid], from: DateTime<Utc>, to: DateTime<Utc>) -> Result<HashMap<Uuid, Vec<TimeEntry>>, sqlx::Error>;
}

pub struct SqlxTimeEntryRepository {
    pub pool: PgPool,
}

impl SqlxTimeEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TimeEntryRepository for SqlxTimeEntryRepository {
    async fn create_time_entry(&self, entry: &TimeEntry) -> Result<(), sqlx::Error> {
        create_time_entry(&self.pool, entry).await
    }
    async fn get_time_entry_by_id(&self, workspace_id: Uuid, id: Uuid) -> Result<Option<TimeEntry>, sqlx::Error> {
        get_time_entry_by_id(&self.pool, workspace_id, id).await
    }
    async fn get_active_time_entry(&self, employee_id: Uuid) -> Result<Option<TimeEntry>, sqlx::Error> {
        get_active_time_entry(&self.pool, employee_id).await
    }
    async fn get_latest_active_entry(&self, employee_id: Uuid) -> Result<Option<TimeEntry>, sqlx::Error> {
        get_latest_active_entry(&self.pool, employee_id).await
    }
    async fn update_time_entry(&self, workspace_id: Uuid, entry: &TimeEntry) -> Result<(), sqlx::Error> {
        update_time_entry(&self.pool, workspace_id, entry).await
    }
    async fn delete_time_entry(&self, workspace_id: Uuid, id: Uuid) -> Result<(), sqlx::Error> {
        delete_time_entry(&self.pool, workspace_id, id).await
    }
    async fn get_employee_time_entries(&self, employee_id: Uuid, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<TimeEntry>, sqlx::Error> {
        get_employee_time_entries(&self.pool, employee_id, from, to).await
    }
    #[allow(clippy::too_many_arguments)]
    async fn get_paginated_logs(&self, workspace_id: Uuid, p_employee_id: Option<Uuid>, p_from: Option<DateTime<Utc>>, p_to: Option<DateTime<Utc>>, p_limit: i64, p_offset: i64) -> Result<Vec<TimeEntryWithEmployee>, sqlx::Error> {
        get_paginated_logs(&self.pool, workspace_id, p_employee_id, p_from, p_to, p_limit, p_offset).await
    }
    async fn count_logs(&self, workspace_id: Uuid, p_employee_id: Option<Uuid>, p_from: Option<DateTime<Utc>>, p_to: Option<DateTime<Utc>>) -> Result<i64, sqlx::Error> {
        count_logs(&self.pool, workspace_id, p_employee_id, p_from, p_to).await
    }
    async fn get_workspace_time_entries_batch(&self, employee_ids: &[Uuid], from: DateTime<Utc>, to: DateTime<Utc>) -> Result<HashMap<Uuid, Vec<TimeEntry>>, sqlx::Error> {
        get_workspace_time_entries_batch(&self.pool, employee_ids, from, to).await
    }
}


pub async fn create_time_entry(pool: &PgPool, entry: &TimeEntry) -> Result<(), sqlx::Error> {
    query(
        "INSERT INTO time_entries (id, employee_id, workspace_id, shift_id, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
    )
    .bind(entry.id)
    .bind(entry.employee_id)
    .bind(entry.workspace_id)
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
    workspace_id: Uuid,
    id: Uuid,
) -> Result<Option<TimeEntry>, sqlx::Error> {
    query_as::<_, TimeEntry>("SELECT * FROM time_entries WHERE id = $1 AND workspace_id = $2")
        .bind(id)
        .bind(workspace_id)
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

pub async fn update_time_entry(pool: &PgPool, workspace_id: Uuid, entry: &TimeEntry) -> Result<(), sqlx::Error> {
    query(
        "UPDATE time_entries SET punch_in_time = $1, punch_out_time = $2, duration_minutes = $3, is_break_overtime = $4, note = $5, updated_at = $6 WHERE id = $7 AND workspace_id = $8"
    )
    .bind(entry.punch_in_time)
    .bind(entry.punch_out_time)
    .bind(entry.duration_minutes)
    .bind(entry.is_break_overtime)
    .bind(&entry.note)
    .bind(entry.updated_at)
    .bind(entry.id)
    .bind(workspace_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_time_entry(pool: &PgPool, workspace_id: Uuid, id: Uuid) -> Result<(), sqlx::Error> {
    query("DELETE FROM time_entries WHERE id = $1 AND workspace_id = $2")
        .bind(id)
        .bind(workspace_id)
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
) -> Result<Vec<TimeEntryWithEmployee>, sqlx::Error> {
    let mut sql = r#"
        SELECT 
            te.*, 
            e.full_name as employee_full_name, 
            e.employee_id_number 
        FROM time_entries te
        JOIN employees e ON te.employee_id = e.id
        WHERE te.workspace_id = $1
    "#
    .to_string();
    let mut param_idx = 2;

    if p_employee_id.is_some() {
        sql.push_str(&format!(" AND te.employee_id = ${param_idx}"));
        param_idx += 1;
    }
    if p_from.is_some() {
        sql.push_str(&format!(" AND te.punch_in_time >= ${param_idx}"));
        param_idx += 1;
    }
    if p_to.is_some() {
        sql.push_str(&format!(" AND te.punch_in_time <= ${param_idx}"));
        param_idx += 1;
    }

    sql.push_str(&format!(
        " ORDER BY te.punch_in_time DESC LIMIT ${} OFFSET ${}",
        param_idx,
        param_idx + 1
    ));

    let mut query = query_as::<_, TimeEntryWithEmployee>(&sql).bind(workspace_id);

    if let Some(id) = p_employee_id {
        query = query.bind(id);
    }
    if let Some(f) = p_from {
        query = query.bind(f);
    }
    if let Some(t) = p_to {
        query = query.bind(t);
    }

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
                  JOIN employees e ON employee_id = e.id 
                  WHERE e.workspace_id = $1"
        .to_string();
    let mut param_idx = 2;

    if p_employee_id.is_some() {
        sql.push_str(&format!(" AND employee_id = ${param_idx}"));
        param_idx += 1;
    }
    if p_from.is_some() {
        sql.push_str(&format!(" AND punch_in_time >= ${param_idx}"));
        param_idx += 1;
    }
    if p_to.is_some() {
        sql.push_str(&format!(" AND punch_in_time <= ${param_idx}"));
        // NO param_idx += 1 here as it's the last one
    }

    let mut query = query_scalar::<_, i64>(&sql).bind(workspace_id);

    if let Some(id) = p_employee_id {
        query = query.bind(id);
    }
    if let Some(f) = p_from {
        query = query.bind(f);
    }
    if let Some(t) = p_to {
        query = query.bind(t);
    }

    query.fetch_one(pool).await
}

/// Batch fetch time entries for multiple employees in a single query
/// Returns HashMap keyed by employee_id for O(1) lookup
pub async fn get_workspace_time_entries_batch(
    pool: &PgPool,
    employee_ids: &[Uuid],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<HashMap<Uuid, Vec<TimeEntry>>, sqlx::Error> {
    if employee_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let entries = query_as::<_, TimeEntry>(
        r#"SELECT * FROM time_entries
           WHERE employee_id = ANY($1)
           AND punch_in_time >= $2 AND punch_in_time <= $3
           ORDER BY employee_id, punch_in_time DESC"#,
    )
    .bind(employee_ids)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    // Group by employee_id for O(1) lookup
    let mut result: HashMap<Uuid, Vec<TimeEntry>> = HashMap::new();
    for entry in entries {
        result.entry(entry.employee_id).or_default().push(entry);
    }

    Ok(result)
}
