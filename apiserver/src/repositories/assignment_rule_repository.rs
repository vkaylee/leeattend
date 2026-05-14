use crate::errors::AppError;
#[allow(unused_imports)]
use crate::models::{
    AssignmentRule, AssignmentRuleConditions, CreateAssignmentRuleRequest,
    UpdateAssignmentRuleRequest,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AssignmentRuleRepository: Send + Sync {
    async fn create_rule(
        &self,
        workspace_id: Uuid,
        req: &CreateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, AppError>;
    async fn get_rules(&self, workspace_id: Uuid) -> Result<Vec<AssignmentRule>, AppError>;
    async fn get_active_rules(&self, workspace_id: Uuid) -> Result<Vec<AssignmentRule>, AppError>;
    async fn update_rule(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: &UpdateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, AppError>;
    async fn delete_rule(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError>;
}
