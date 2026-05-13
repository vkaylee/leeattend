use crate::errors::AppError;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: AttendanceLogErrorRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait AttendanceLogErrorRepository: Send + Sync {
    async fn create_error_log(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
    ) -> Result<(), AppError>;
    async fn create_error_log_with_fingerprint(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
        fingerprint: &str,
    ) -> Result<bool, AppError>;
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
