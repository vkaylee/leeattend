use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

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
pub async fn create_staging_log(
    pool: &PgPool,
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
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (workspace_id, fingerprint) DO NOTHING
        "#,
    )
    .bind(workspace_id)
    .bind(device_id)
    .bind(raw_user_id)
    .bind(punch_time)
    .bind(raw_data)
    .bind(fingerprint)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Get pending staging logs for a workspace
pub async fn get_pending_staging_logs(
    pool: &PgPool,
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
    .fetch_all(pool)
    .await
}

/// Update staging log status (e.g., 'processed', 'dismissed')
pub async fn update_staging_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE attendance_staging_logs SET status = $1 WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Count pending staging logs for a workspace
pub async fn count_pending_staging_logs(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM attendance_staging_logs WHERE workspace_id = $1 AND status = 'pending'",
    )
    .bind(workspace_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
