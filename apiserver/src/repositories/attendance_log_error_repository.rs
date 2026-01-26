use sqlx::PgPool;
use uuid::Uuid;

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
        "#
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
