use crate::errors::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: StagingLogRepository — DIP abstraction
// (update_staging_status_tx is excluded — it takes &mut Transaction)
// ---------------------------------------------------------------------------
#[async_trait]
pub trait StagingLogRepository: Send + Sync {
    async fn create_staging_log(
        &self,
        workspace_id: Uuid,
        device_id: Option<String>,
        raw_user_id: &str,
        punch_time: DateTime<Utc>,
        raw_data: Option<serde_json::Value>,
        fingerprint: &str,
    ) -> Result<bool, AppError>;
    async fn get_pending_staging_logs(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<StagingLog>, AppError>;
    async fn update_staging_status(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        status: &str,
    ) -> Result<bool, AppError>;
    async fn count_pending_staging_logs(&self, workspace_id: Uuid) -> Result<i64, AppError>;
}

/// Staging log for unknown users (not yet mapped to employees)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StagingLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub device_id: Option<String>,
    pub raw_user_id: String,
    pub punch_time: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
    pub fingerprint: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// Create a staging log for unknown user.
/// Uses ON CONFLICT DO NOTHING to silently ignore duplicates.
pub async fn create_staging_log<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    workspace_id: Uuid,
    device_id: Option<String>,
    raw_user_id: &str,
    punch_time: DateTime<Utc>,
    raw_data: Option<serde_json::Value>,
    fingerprint: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO attendance_staging_logs
        (workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint)
        SELECT $1, $2, $3, $4, $5, $6
        WHERE NOT EXISTS (
            SELECT 1 FROM attendance_staging_logs 
            WHERE workspace_id = $1 AND fingerprint = $6
        )
        "#,
    )
    .bind(workspace_id)
    .bind(device_id)
    .bind(raw_user_id)
    .bind(punch_time)
    .bind(raw_data)
    .bind(fingerprint)
    .execute(executor)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Get pending staging logs for a workspace
pub async fn get_pending_staging_logs<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    workspace_id: Uuid,
    limit: i64,
) -> Result<Vec<StagingLog>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT id, workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint, status, created_at
        FROM attendance_staging_logs
        WHERE workspace_id = $1 AND status = 'pending'
        ORDER BY created_at ASC
        LIMIT $2
        "#,
    )
    .bind(workspace_id)
    .bind(limit)
    .fetch_all(executor)
    .await
}

/// Update staging log status with workspace_id filtering for multi-tenant safety.
/// Returns true if a row was actually updated.
pub async fn update_staging_status<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    id: Uuid,
    workspace_id: Uuid,
    status: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE attendance_staging_logs SET status = $1 WHERE id = $2 AND workspace_id = $3",
    )
    .bind(status)
    .bind(id)
    .bind(workspace_id)
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Update staging log status within an existing transaction.
pub async fn update_staging_status_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: Uuid,
    workspace_id: Uuid,
    status: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE attendance_staging_logs SET status = $1 WHERE id = $2 AND workspace_id = $3",
    )
    .bind(status)
    .bind(id)
    .bind(workspace_id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Count pending staging logs for a workspace
pub async fn count_pending_staging_logs<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    workspace_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM attendance_staging_logs WHERE workspace_id = $1 AND status = 'pending'",
    )
    .bind(workspace_id)
    .fetch_one(executor)
    .await?;
    Ok(row.0)
}

/// Lightweight direct claim of staged logs for an employee (no queue/Redis needed).
/// Finds all pending staging logs matching `employee_id_number`, inserts them into
/// `raw_attendance_logs`, and marks them as processed.
pub async fn claim_staged_logs_direct(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    employee_id_number: &str,
) -> Result<u32, sqlx::Error> {
    let staged_logs = sqlx::query_as::<_, StagingLog>(
        r#"
        SELECT id, workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint, status, created_at
        FROM attendance_staging_logs
        WHERE workspace_id = $1 AND raw_user_id = $2 AND status = 'pending'
        ORDER BY punch_time ASC
        "#,
    )
    .bind(workspace_id)
    .bind(employee_id_number)
    .fetch_all(pool)
    .await?;

    let mut claimed = 0u32;
    for staged in &staged_logs {
        // Insert into raw_attendance_logs
        let result = sqlx::query(
            r#"
            INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source, device_id, raw_data, punch, status, created_at)
            VALUES ($1, $2, $3, $4, 'device', $5, $6, -1, -1, $7)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(crate::utils::id::new_id())
        .bind(workspace_id)
        .bind(employee_id)
        .bind(staged.punch_time)
        .bind(&staged.device_id)
        .bind(&staged.raw_data)
        .bind(Utc::now())
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            claimed += 1;
        }

        // Mark as processed
        update_staging_status(pool, staged.id, workspace_id, "processed").await?;
    }

    Ok(claimed)
}
