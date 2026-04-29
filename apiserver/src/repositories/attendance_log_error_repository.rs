use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: AttendanceLogErrorRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait AttendanceLogErrorRepository: super::PoolBacked + Send + Sync {
    async fn create_error_log(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
    ) -> Result<(), sqlx::Error>;
    async fn create_error_log_with_fingerprint(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
        fingerprint: &str,
    ) -> Result<bool, sqlx::Error>;
}

// ---------------------------------------------------------------------------
// SqlxAttendanceLogErrorRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxAttendanceLogErrorRepository {
    pub pool: PgPool,
}

impl SqlxAttendanceLogErrorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttendanceLogErrorRepository for SqlxAttendanceLogErrorRepository {
    async fn create_error_log(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
    ) -> Result<(), sqlx::Error> {
        create_error_log(
            &self.pool,
            workspace_id,
            agent_id,
            raw_content,
            error_message,
            source,
        )
        .await
    }
    async fn create_error_log_with_fingerprint(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
        fingerprint: &str,
    ) -> Result<bool, sqlx::Error> {
        create_error_log_with_fingerprint(
            &self.pool,
            workspace_id,
            agent_id,
            raw_content,
            error_message,
            source,
            fingerprint,
        )
        .await
    }
}

pub async fn create_error_log(
    pool: &PgPool,
    workspace_id: Uuid,
    agent_id: Option<String>,
    raw_content: Option<serde_json::Value>,
    error_message: &str,
    source: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO attendance_log_errors
        (workspace_id, agent_id, raw_content, error_message, source)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(workspace_id)
    .bind(agent_id)
    .bind(raw_content)
    .bind(error_message)
    .bind(source)
    .execute(pool)
    .await?;

    Ok(())
}

/// Create error log with fingerprint for deduplication.
/// Uses ON CONFLICT DO NOTHING to silently ignore duplicate errors.
pub async fn create_error_log_with_fingerprint(
    pool: &PgPool,
    workspace_id: Uuid,
    agent_id: Option<String>,
    raw_content: Option<serde_json::Value>,
    error_message: &str,
    source: &str,
    fingerprint: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO attendance_log_errors
        (workspace_id, agent_id, raw_content, error_message, source, fingerprint)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (workspace_id, fingerprint) WHERE fingerprint IS NOT NULL DO NOTHING
        "#,
    )
    .bind(workspace_id)
    .bind(agent_id)
    .bind(raw_content)
    .bind(error_message)
    .bind(source)
    .bind(fingerprint)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

impl super::PoolBacked for SqlxAttendanceLogErrorRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
