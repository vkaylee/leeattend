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
