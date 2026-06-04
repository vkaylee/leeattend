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

        let created_workflow = self
            .approval_workflow_repo
            .create_workflow(auth_token, workflow)
            .await?;

        let mut steps_to_create = steps;
        for step in &mut steps_to_create {
            step.workflow_id = created_workflow.id;
        }

        let created_steps = self
            .approval_workflow_repo
            .create_steps(auth_token, steps_to_create)
            .await?;

        Ok((created_workflow, created_steps))
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
    ) -> AppResult<Vec<serde_json::Value>> {
        let workflows = self
            .approval_workflow_repo
            .list_workflows(auth_token, workspace_id)
            .await?;
        if workflows.is_empty() {
            return Ok(vec![]);
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

        Ok(result)
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

        let updated_wf = self
            .approval_workflow_repo
            .update_workflow(auth_token, workflow)
            .await?;

        // Delete old steps
        self.approval_workflow_repo
            .delete_steps_by_workflow_id(auth_token, updated_wf.id)
            .await?;

        // Create new steps
        let mut steps_to_create = steps;
        for step in &mut steps_to_create {
            step.workflow_id = updated_wf.id;
        }

        let created_steps = self
            .approval_workflow_repo
            .create_steps(auth_token, steps_to_create)
            .await?;

        Ok((updated_wf, created_steps))
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
