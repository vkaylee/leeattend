use crate::errors::AppResult;
use crate::models::{ApprovalActionLog, ApprovalWorkflow, ApprovalWorkflowStep};
use crate::repositories::ApprovalWorkflowRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct ApprovalWorkflowService {
    approval_workflow_repo: Arc<dyn ApprovalWorkflowRepository>,
}

impl ApprovalWorkflowService {
    pub fn new(approval_workflow_repo: Arc<dyn ApprovalWorkflowRepository>) -> Self {
        Self {
            approval_workflow_repo,
        }
    }

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
        self.approval_workflow_repo.log_action(auth_token, log).await
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::diesel_pool::TenantAuthToken;
    use async_trait::async_trait;
    use chrono::Utc;
    
    struct MockApprovalWorkflowRepository {
        pub create_workflow_called: std::sync::atomic::AtomicBool,
    }

    impl MockApprovalWorkflowRepository {
        fn new() -> Self {
            Self {
                create_workflow_called: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    #[async_trait]
    impl ApprovalWorkflowRepository for MockApprovalWorkflowRepository {
        async fn create_workflow(
            &self,
            _auth_token: &TenantAuthToken,
            workflow: ApprovalWorkflow,
        ) -> Result<ApprovalWorkflow, AppError> {
            self.create_workflow_called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(workflow)
        }

        async fn create_steps(
            &self,
            _auth_token: &TenantAuthToken,
            steps: Vec<ApprovalWorkflowStep>,
        ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
            Ok(steps)
        }

        async fn get_active_workflow(
            &self,
            _auth_token: &TenantAuthToken,
            _workspace_id: Uuid,
            _entity_type: &str,
        ) -> Result<Option<ApprovalWorkflow>, AppError> {
            Ok(None)
        }

        async fn get_steps(
            &self,
            _auth_token: &TenantAuthToken,
            _workflow_id: Uuid,
        ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
            Ok(vec![])
        }

        async fn log_action(
            &self,
            _auth_token: &TenantAuthToken,
            log: ApprovalActionLog,
        ) -> Result<ApprovalActionLog, AppError> {
            Ok(log)
        }

        async fn get_action_logs(
            &self,
            _auth_token: &TenantAuthToken,
            _entity_id: Uuid,
        ) -> Result<Vec<ApprovalActionLog>, AppError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_create_workflow_success() {
        let repo = Arc::new(MockApprovalWorkflowRepository::new());
        let service = ApprovalWorkflowService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let workflow = ApprovalWorkflow {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(), // deliberate mismatch
            entity_type: "overtime_request".to_string(),
            name: "Default OT Workflow".to_string(),
            description: None,
            total_steps: 1,
            is_active: true,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let step = ApprovalWorkflowStep {
            id: Uuid::new_v4(),
            workflow_id: Uuid::new_v4(),
            step_order: 1,
            step_name: "Manager Approval".to_string(),
            approver_type: "manager".to_string(),
            approver_config: serde_json::json!({}),
            action_required: "approve".to_string(),
            can_skip: false,
            created_at: Utc::now(),
        };

        let (created_wf, created_steps) = service.create_workflow(&token, workflow, vec![step]).await.unwrap();
        
        // Ensure workspace_id is enforced
        assert_eq!(created_wf.workspace_id, ws_id);
        
        // Ensure step has correct workflow_id
        assert_eq!(created_steps.len(), 1);
        assert_eq!(created_steps[0].workflow_id, created_wf.id);
        
        assert!(repo.create_workflow_called.load(std::sync::atomic::Ordering::SeqCst));
    }
}

