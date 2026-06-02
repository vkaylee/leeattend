use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::outbox_job::OutboxJobEntity;
use crate::models::OutboxJob;
use crate::repositories::outbox_job_repository::OutboxJobRepository;
use crate::schema::outbox_jobs;
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
}
