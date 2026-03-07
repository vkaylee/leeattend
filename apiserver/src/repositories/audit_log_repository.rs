use crate::domain::audit::AuditLog;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait AuditLogRepository: super::PoolBacked + Send + Sync {
    async fn create_log(&self, config: &AuditLog) -> Result<AuditLog, sqlx::Error>;
    
    // Retrieves audit logs for a workspace, ordered by created_at DESC
    async fn find_by_workspace(
        &self,
        workspace_id: Uuid,
        limit: i64,
        offset: i64,
        action: Option<String>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
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
        action: Option<String>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<AuditLog>, i64), sqlx::Error> {
        let mut count_qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new("SELECT count(*) FROM audit_logs WHERE workspace_id = ");
        count_qb.push_bind(workspace_id);

        if let Some(ref a) = action {
            count_qb.push(" AND action = ");
            count_qb.push_bind(a);
        }
        if let Some(ref sd) = start_date {
            count_qb.push(" AND created_at >= ");
            count_qb.push_bind(sd);
        }
        if let Some(ref ed) = end_date {
            count_qb.push(" AND created_at <= ");
            count_qb.push_bind(ed);
        }

        let count: Option<i64> = count_qb.build_query_scalar().fetch_one(&self.pool).await?;
        let count = count.unwrap_or(0);

        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new("SELECT id, workspace_id, actor_id, target_user_id, target_resource_id, action, details, ip_address, user_agent, created_at FROM audit_logs WHERE workspace_id = ");
        qb.push_bind(workspace_id);

        if let Some(ref a) = action {
            qb.push(" AND action = ");
            qb.push_bind(a);
        }
        if let Some(ref sd) = start_date {
            qb.push(" AND created_at >= ");
            qb.push_bind(sd);
        }
        if let Some(ref ed) = end_date {
            qb.push(" AND created_at <= ");
            qb.push_bind(ed);
        }

        qb.push(" ORDER BY created_at DESC LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let logs = qb.build_query_as::<AuditLog>().fetch_all(&self.pool).await?;

        Ok((logs, count))
    }
}

impl super::PoolBacked for SqlxAuditLogRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self { Self { pool } }
    fn pool(&self) -> &sqlx::PgPool { &self.pool }
}
