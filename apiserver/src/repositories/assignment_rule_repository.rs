use crate::errors::AppError;
#[allow(unused_imports)]
use crate::models::{
    AssignmentRule, AssignmentRuleConditions, CreateAssignmentRuleRequest,
    UpdateAssignmentRuleRequest,
};
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AssignmentRuleRepository: Send + Sync {
    async fn create_rule(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &CreateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, AppError>;
    async fn get_rules(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AssignmentRule>, AppError>;
    async fn get_active_rules(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AssignmentRule>, AppError>;
    async fn update_rule(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: &UpdateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, AppError>;
    async fn delete_rule(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError>;
}
