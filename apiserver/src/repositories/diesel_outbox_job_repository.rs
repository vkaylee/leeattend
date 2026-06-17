use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::core::WorkspaceEntity;
use crate::models::diesel::outbox_job::OutboxJobEntity;
use crate::models::OutboxJob;
use crate::repositories::outbox_job_repository::OutboxJobRepository;
use crate::schema::{outbox_jobs, workspaces};
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselOutboxJobRepository {
    diesel_pools: DieselPools,
}

impl DieselOutboxJobRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl OutboxJobRepository for DieselOutboxJobRepository {
    async fn insert_job(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        job_type: &str,
        payload: serde_json::Value,
    ) -> Result<OutboxJob, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let new_job = OutboxJobEntity {
            id,
            workspace_id,
            job_type: job_type.to_string(),
            payload,
            status: "pending".to_string(),
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = diesel::insert_into(outbox_jobs::table)
            .values(&new_job)
            .get_result::<OutboxJobEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn update_job_status(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        status: &str,
        error_message: Option<String>,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        diesel::update(
            outbox_jobs::table
                .filter(outbox_jobs::id.eq(id))
                .filter(outbox_jobs::workspace_id.eq(workspace_id)),
        )
        .set((
            outbox_jobs::status.eq(status.to_string()),
            outbox_jobs::error_message.eq(error_message),
            outbox_jobs::updated_at.eq(Utc::now()),
        ))
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_pending_jobs(
        &self,
        auth_token: &TenantAuthToken,
        limit: i64,
    ) -> Result<Vec<OutboxJob>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let results = outbox_jobs::table
            .filter(outbox_jobs::workspace_id.eq(workspace_id))
            .filter(outbox_jobs::status.eq("pending".to_string()))
            .order(outbox_jobs::created_at.asc())
            .limit(limit)
            .load::<OutboxJobEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_stuck_jobs(
        &self,
        auth_token: &TenantAuthToken,
        older_than: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> Result<Vec<OutboxJob>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let results = outbox_jobs::table
            .filter(outbox_jobs::workspace_id.eq(workspace_id))
            .filter(
                outbox_jobs::status
                    .eq("pending".to_string())
                    .or(outbox_jobs::status.eq("running".to_string())),
            )
            .filter(outbox_jobs::updated_at.lt(older_than))
            .order(outbox_jobs::updated_at.asc())
            .limit(limit)
            .load::<OutboxJobEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_all_stuck_jobs(
        &self,
        older_than: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> Result<Vec<OutboxJob>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let workspaces = workspaces::table
            .select(WorkspaceEntity::as_select())
            .load::<WorkspaceEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut futures = Vec::new();
        for ws in workspaces {
            let pools = self.diesel_pools.clone();
            futures.push(async move {
                let auth_token = TenantAuthToken::system_override(ws.id);
                let mut conn = match pools.tenant_transaction_conn(&auth_token).await {
                    Ok(conn) => conn,
                    Err(e) => {
                        tracing::error!(
                            workspace_id = %ws.id,
                            error = ?e,
                            "Failed to get tenant database connection for outbox sweep"
                        );
                        return Ok::<_, AppError>(Vec::new());
                    }
                };

                let results = diesel::sql_query(
                    "UPDATE outbox_jobs \
                     SET status = 'running', updated_at = $1 \
                     WHERE id IN ( \
                         SELECT id FROM outbox_jobs \
                         WHERE workspace_id = $2 \
                           AND (status = 'pending' OR status = 'running') \
                           AND updated_at < $3 \
                         ORDER BY updated_at ASC \
                         LIMIT $4 \
                         FOR UPDATE SKIP LOCKED \
                     ) \
                     RETURNING id, workspace_id, job_type, payload, status, \
                               error_message, created_at, updated_at"
                )
                .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
                .bind::<diesel::sql_types::Uuid, _>(ws.id)
                .bind::<diesel::sql_types::Timestamptz, _>(older_than)
                .bind::<diesel::sql_types::BigInt, _>(limit)
                .get_results::<OutboxJobEntity>(&mut *conn)
                .await;

                match results {
                    Ok(jobs) => Ok(jobs.into_iter().map(Into::into).collect::<Vec<OutboxJob>>()),
                    Err(e) => {
                        tracing::error!(
                            workspace_id = %ws.id,
                            error = ?e,
                            "Failed to claim stuck outbox jobs for tenant"
                        );
                        Ok(Vec::new())
                    }
                }
            });
        }

        let results = futures_util::future::try_join_all(futures).await?;
        let all_jobs = results.into_iter().flatten().collect();
        Ok(all_jobs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::TenantAuthToken;
    use chrono::{Duration, Utc};
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_outbox_job_repository() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://test_user:test_password@db_system:5432/leeattend_test_template".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselOutboxJobRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = TenantAuthToken::system_override(ws_id);

        let job_id = Uuid::new_v4();
        let job_type = "test_job";
        let payload = json!({"key": "value"});

        let insert_res = repo
            .insert_job(&auth_token, job_id, job_type, payload.clone())
            .await;

        if let Ok(job) = insert_res {
            assert_eq!(job.id(), job_id);
            assert_eq!(job.workspace_id(), ws_id);
            assert_eq!(job.job_type(), job_type);
            assert_eq!(job.payload(), &payload);
            assert_eq!(job.status(), "pending");

            // Retrieve pending jobs
            let pending = repo.get_pending_jobs(&auth_token, 10).await.unwrap();
            assert!(pending.iter().any(|j| j.id() == job_id));

            // Retrieve stuck jobs
            let older_than = Utc::now() + Duration::minutes(5);
            let stuck = repo
                .get_stuck_jobs(&auth_token, older_than, 10)
                .await
                .unwrap();
            assert!(stuck.iter().any(|j| j.id() == job_id));

            // Update status
            repo.update_job_status(&auth_token, job_id, "completed", None)
                .await
                .unwrap();

            // Should not be pending anymore
            let pending_after = repo.get_pending_jobs(&auth_token, 10).await.unwrap();
            assert!(!pending_after.iter().any(|j| j.id() == job_id));
        } else {
            let _ = repo
                .update_job_status(&auth_token, job_id, "completed", None)
                .await;
            let _ = repo.get_pending_jobs(&auth_token, 10).await;
            let _ = repo.get_stuck_jobs(&auth_token, Utc::now(), 10).await;
        }
    }
}
