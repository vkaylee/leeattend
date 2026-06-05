use super::ApprovalWorkflowService;
use crate::errors::{AppError, AppResult};
use crate::models::{
    ApprovalActionLog, ApprovalWorkflow, ApprovalWorkflowInstance, ApprovalWorkflowStep,
};
use uuid::Uuid;

impl ApprovalWorkflowService {
    pub async fn create_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        mut workflow: ApprovalWorkflow,
        steps: Vec<ApprovalWorkflowStep>,
    ) -> AppResult<(ApprovalWorkflow, Vec<ApprovalWorkflowStep>)> {
        workflow.workspace_id = auth_token.workspace_id();

        self.approval_workflow_repo
            .create_workflow_transactional(auth_token, workflow, steps)
            .await
    }

    pub async fn get_active_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_type: &str,
    ) -> AppResult<Option<ApprovalWorkflow>> {
        self.approval_workflow_repo
            .get_active_workflow(auth_token, auth_token.workspace_id(), entity_type)
            .await
    }

    pub async fn list_workflows(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        entity_type: Option<String>,
        page: i64,
        limit: i64,
    ) -> AppResult<(Vec<serde_json::Value>, i64)> {
        let (workflows, total) = self
            .approval_workflow_repo
            .list_workflows(auth_token, workspace_id, entity_type, page, limit)
            .await?;
        if workflows.is_empty() {
            return Ok((vec![], 0));
        }

        let workflow_ids: Vec<Uuid> = workflows.iter().map(|w| w.id).collect();
        let all_steps = self
            .approval_workflow_repo
            .get_steps_for_workflows(auth_token, workflow_ids)
            .await?;

        let mut result = Vec::new();
        for wf in workflows {
            let mut wf_val =
                serde_json::to_value(&wf).map_err(|e| AppError::Internal(e.to_string()))?;
            let steps: Vec<&ApprovalWorkflowStep> = all_steps
                .iter()
                .filter(|s| s.workflow_id == wf.id)
                .collect();
            if let serde_json::Value::Object(ref mut map) = wf_val {
                map.insert(
                    "steps".to_string(),
                    serde_json::to_value(&steps).map_err(|e| AppError::Internal(e.to_string()))?,
                );
            }
            result.push(wf_val);
        }

        Ok((result, total))
    }

    pub async fn get_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> AppResult<Option<(ApprovalWorkflow, Vec<ApprovalWorkflowStep>)>> {
        if let Some(wf) = self
            .approval_workflow_repo
            .get_workflow_by_id(auth_token, workflow_id)
            .await?
        {
            let steps = self
                .approval_workflow_repo
                .get_steps(auth_token, workflow_id)
                .await?;
            Ok(Some((wf, steps)))
        } else {
            Ok(None)
        }
    }

    pub async fn update_workflow_definition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        mut workflow: ApprovalWorkflow,
        steps: Vec<ApprovalWorkflowStep>,
    ) -> AppResult<(ApprovalWorkflow, Vec<ApprovalWorkflowStep>)> {
        workflow.workspace_id = auth_token.workspace_id();
        workflow.total_steps = steps.len() as i32;

        self.approval_workflow_repo
            .update_workflow_definition_transactional(auth_token, workflow, steps)
            .await
    }

    pub async fn delete_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> AppResult<()> {
        self.approval_workflow_repo
            .delete_workflow(auth_token, workflow_id)
            .await
    }

    pub async fn get_steps(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> AppResult<Vec<ApprovalWorkflowStep>> {
        self.approval_workflow_repo
            .get_steps(auth_token, workflow_id)
            .await
    }

    pub async fn log_action(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        mut log: ApprovalActionLog,
    ) -> AppResult<ApprovalActionLog> {
        log.workspace_id = auth_token.workspace_id();
        self.approval_workflow_repo
            .log_action(auth_token, log)
            .await
    }

    pub async fn get_action_logs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
    ) -> AppResult<Vec<ApprovalActionLog>> {
        self.approval_workflow_repo
            .get_action_logs(auth_token, entity_id)
            .await
    }

    pub async fn get_workflow_instance_by_entity(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_type: &str,
        entity_id: Uuid,
    ) -> AppResult<Option<ApprovalWorkflowInstance>> {
        self.approval_workflow_repo
            .get_instance_by_entity(auth_token, entity_type, entity_id)
            .await
    }

    pub async fn get_workflow_instance_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance_id: Uuid,
    ) -> AppResult<Option<ApprovalWorkflowInstance>> {
        self.approval_workflow_repo
            .get_instance_by_id(auth_token, instance_id)
            .await
    }
}
