use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{EmployeePhase, EmploymentPhaseDefinition};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EmploymentPhaseRepository: Send + Sync {
    async fn create_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<String>,
        phase_type: &str,
        rule_config: &serde_json::Value,
        display_order: i32,
    ) -> Result<EmploymentPhaseDefinition, AppError>;

    async fn get_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<EmploymentPhaseDefinition>, AppError>;

    async fn assign_phase(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        phase_definition_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        override_config: Option<serde_json::Value>,
    ) -> Result<EmployeePhase, AppError>;

    async fn update_phase_status(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        phase_id: Uuid,
        status: &str,
        evaluation_notes: Option<String>,
        evaluated_by: Uuid,
    ) -> Result<EmployeePhase, AppError>;

    async fn get_employment_phases(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
    ) -> Result<Vec<EmployeePhase>, AppError>;
}
