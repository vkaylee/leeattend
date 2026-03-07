use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: StagingLogRepository — DIP abstraction
// (update_staging_status_tx is excluded — it takes &mut Transaction)
// ---------------------------------------------------------------------------
#[async_trait]
pub trait StagingLogRepository: super::PoolBacked + Send + Sync {
    async fn create_staging_log(&self, workspace_id: Uuid, device_id: Option<String>, raw_user_id: &str, punch_time: DateTime<Utc>, raw_data: Option<serde_json::Value>, fingerprint: &str) -> Result<bool, sqlx::Error>;
    async fn get_pending_staging_logs(&self, workspace_id: Uuid, limit: i64) -> Result<Vec<StagingLog>, sqlx::Error>;
    async fn update_staging_status(&self, id: Uuid, workspace_id: Uuid, status: &str) -> Result<bool, sqlx::Error>;
    async fn count_pending_staging_logs(&self, workspace_id: Uuid) -> Result<i64, sqlx::Error>;
}

// ---------------------------------------------------------------------------
// SqlxStagingLogRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxStagingLogRepository {
    pub pool: PgPool,
}

impl SqlxStagingLogRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl StagingLogRepository for SqlxStagingLogRepository {
    async fn create_staging_log(&self, workspace_id: Uuid, device_id: Option<String>, raw_user_id: &str, punch_time: DateTime<Utc>, raw_data: Option<serde_json::Value>, fingerprint: &str) -> Result<bool, sqlx::Error> {
        create_staging_log(&self.pool, workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint).await
    }
    async fn get_pending_staging_logs(&self, workspace_id: Uuid, limit: i64) -> Result<Vec<StagingLog>, sqlx::Error> {
        get_pending_staging_logs(&self.pool, workspace_id, limit).await
    }
    async fn update_staging_status(&self, id: Uuid, workspace_id: Uuid, status: &str) -> Result<bool, sqlx::Error> {
        update_staging_status(&self.pool, id, workspace_id, status).await
    }
    async fn count_pending_staging_logs(&self, workspace_id: Uuid) -> Result<i64, sqlx::Error> {
        count_pending_staging_logs(&self.pool, workspace_id).await
    }
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

/// Update staging log status with workspace_id filtering for multi-tenant safety.
/// Returns true if a row was actually updated.
pub async fn update_staging_status(
    pool: &PgPool,
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
    .execute(pool)
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

impl super::PoolBacked for SqlxStagingLogRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self { Self { pool } }
    fn pool(&self) -> &sqlx::PgPool { &self.pool }
}
