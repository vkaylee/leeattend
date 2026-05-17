use crate::errors::AppError;
use crate::models::{OvertimePolicy, OvertimeRule};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: OvertimePolicyRepositoryTrait — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait OvertimePolicyRepositoryTrait: Send + Sync {
    async fn create_policy_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<OvertimePolicy, AppError>;
    async fn get_policy_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<OvertimePolicy>, AppError>;
    async fn get_policy_with_rules(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<(OvertimePolicy, Vec<OvertimeRule>)>, AppError>;
    async fn add_rule(&self, workspace_id: Uuid, rule: &OvertimeRule) -> Result<OvertimeRule, AppError>;
    async fn resolve_policy_for_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Uuid>, AppError>;
    async fn create_policy_assignment(
        &self,
        workspace_id: Uuid,
        assignee_type: &str,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
    ) -> Result<(), AppError>;
}
