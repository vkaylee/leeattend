use crate::domain::audit::AuditLog;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn create_log(&self, config: &AuditLog) -> Result<AuditLog, sqlx::Error>;
    
    // Retrieves audit logs for a workspace, ordered by created_at DESC
    async fn find_by_workspace(
        &self,
        workspace_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AuditLog>, i64), sqlx::Error>;
}

pub struct SqlxAuditLogRepository {
    pub pool: PgPool,
}

#[async_trait]
impl AuditLogRepository for SqlxAuditLogRepository {
    async fn create_log(&self, log: &AuditLog) -> Result<AuditLog, sqlx::Error> {
        let rec = sqlx::query_as!(
            AuditLog,
            r#"
            INSERT INTO audit_logs (id, workspace_id, actor_id, target_user_id, target_resource_id, action, details, ip_address, user_agent, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, workspace_id, actor_id, target_user_id, target_resource_id, action, details, ip_address, user_agent, created_at
            "#,
            log.id,
            log.workspace_id,
            log.actor_id,
            log.target_user_id,
            log.target_resource_id,
            log.action,
            log.details,
            log.ip_address,
            log.user_agent,
            log.created_at
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_by_workspace(
        &self,
        workspace_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AuditLog>, i64), sqlx::Error> {
        let count_query = sqlx::query_scalar!(
            r#"
            SELECT count(*) FROM audit_logs
            WHERE workspace_id = $1
            "#,
            workspace_id
        );
        let count = count_query.fetch_one(&self.pool).await?.unwrap_or(0);

        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, workspace_id, actor_id, target_user_id, target_resource_id, action, details, ip_address, user_agent, created_at
            FROM audit_logs
            WHERE workspace_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            workspace_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((logs, count))
    }
}
