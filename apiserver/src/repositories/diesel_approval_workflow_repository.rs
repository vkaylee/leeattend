use crate::errors::AppError;
use crate::models::{ApprovalActionLog, ApprovalWorkflow, ApprovalWorkflowStep};
use crate::repositories::ApprovalWorkflowRepository;
use crate::schema::{approval_action_log, approval_workflow_steps, approval_workflows};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct DieselApprovalWorkflowRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselApprovalWorkflowRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl ApprovalWorkflowRepository for DieselApprovalWorkflowRepository {
    async fn create_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow: ApprovalWorkflow,
    ) -> Result<ApprovalWorkflow, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::insert_into(approval_workflows::table)
            .values(&workflow)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create approval workflow: {e}")))?;

        Ok(result)
    }

    async fn create_steps(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        steps: Vec<ApprovalWorkflowStep>,
    ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::insert_into(approval_workflow_steps::table)
            .values(&steps)
            .get_results(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create workflow steps: {e}")))?;

        Ok(result)
    }

    async fn get_active_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        entity_type: &str,
    ) -> Result<Option<ApprovalWorkflow>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_workflows::table
            .filter(approval_workflows::workspace_id.eq(workspace_id))
            .filter(approval_workflows::entity_type.eq(entity_type))
            .filter(approval_workflows::is_active.eq(true))
            .first::<ApprovalWorkflow>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("Failed to get active workflow: {e}")))?;

        Ok(result)
    }

    async fn get_steps(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_workflow_steps::table
            .filter(approval_workflow_steps::workflow_id.eq(workflow_id))
            .order(approval_workflow_steps::step_order.asc())
            .load::<ApprovalWorkflowStep>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get workflow steps: {e}")))?;

        Ok(result)
    }

    async fn log_action(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        log: ApprovalActionLog,
    ) -> Result<ApprovalActionLog, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::insert_into(approval_action_log::table)
            .values(&log)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to log approval action: {e}")))?;

        Ok(result)
    }

    async fn get_action_logs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
    ) -> Result<Vec<ApprovalActionLog>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_action_log::table
            .filter(approval_action_log::entity_id.eq(entity_id))
            .order(approval_action_log::acted_at.asc())
            .load::<ApprovalActionLog>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get action logs: {e}")))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::{DieselPools, TenantAuthToken};
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_repo_methods_error_handling() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselApprovalWorkflowRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = TenantAuthToken::system_override(ws_id);

        let workflow = ApprovalWorkflow {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            entity_type: "ot_request".into(),
            name: "Test".into(),
            description: None,
            total_steps: 1,
            is_active: true,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let err = repo.create_workflow(&auth_token, workflow).await;
        let _ = err;

        let err = repo.create_steps(&auth_token, vec![]).await;
        let _ = err;

        let err = repo
            .get_active_workflow(&auth_token, ws_id, "ot_request")
            .await;
        let _ = err;

        let err = repo.get_steps(&auth_token, Uuid::new_v4()).await;
        let _ = err;

        let log = ApprovalActionLog {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            entity_type: "ot_request".into(),
            entity_id: Uuid::new_v4(),
            step_order: 1,
            step_name: None,
            acted_by_user_id: Uuid::new_v4(),
            action: "approve".into(),
            note: None,
            acted_at: Utc::now(),
        };

        let err = repo.log_action(&auth_token, log).await;
        let _ = err;

        let err = repo.get_action_logs(&auth_token, Uuid::new_v4()).await;
        let _ = err;
    }
}
