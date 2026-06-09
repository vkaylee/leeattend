use crate::models::{
    BulkAssignResponse, EmployeeWorkingPattern, WorkingPattern, WorkingPatternDetail,
};
use chrono::NaiveDate;
use uuid::Uuid;

use crate::errors::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait WorkingPatternRepositoryTrait: Send + Sync {
    async fn create_pattern_with_details(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        pattern_type: &str,
        cycle_length_days: Option<i32>,
        details: Vec<(i32, Option<Uuid>)>, // (schedule_day, shift_id)
    ) -> Result<WorkingPattern, AppError>;

    #[allow(clippy::too_many_arguments)]
    async fn update_pattern_with_details(
        &self,
        workspace_id: Uuid,
        pattern_id: Uuid,
        name: &str,
        description: Option<&str>,
        pattern_type: &str,
        cycle_length_days: Option<i32>,
        details: Vec<(i32, Option<Uuid>)>, // (schedule_day, shift_id)
    ) -> Result<WorkingPattern, AppError>;

    async fn get_patterns(&self, workspace_id: Uuid) -> Result<Vec<WorkingPattern>, AppError>;

    async fn get_pattern_details(
        &self,
        workspace_id: Uuid,
        pattern_id: Uuid,
    ) -> Result<Vec<WorkingPatternDetail>, AppError>;

    async fn assign_pattern(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        pattern_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        anchor_date: Option<NaiveDate>,
    ) -> Result<EmployeeWorkingPattern, AppError>;

    async fn get_employment_pattern_history(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
    ) -> Result<Vec<EmployeeWorkingPattern>, AppError>;

    async fn bulk_assign_pattern(
        &self,
        workspace_id: Uuid,
        employment_ids: &[Uuid],
        pattern_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        anchor_date: Option<NaiveDate>,
    ) -> Result<BulkAssignResponse, AppError>;

    async fn delete_pattern(&self, workspace_id: Uuid, pattern_id: Uuid) -> Result<(), AppError>;
}
