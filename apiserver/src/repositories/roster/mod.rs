//! Roster repository — split into domain sub-modules.
//!
//! - [`shift_resolution`]: Pure shift resolution logic (no DB)

pub mod shift_resolution;

use crate::errors::AppError;
use crate::models::{EmployeeSchedule, ShiftRoster};
use async_trait::async_trait;
use chrono::NaiveDate;

use uuid::Uuid;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait RosterRepository: Send + Sync {
    async fn get_workspace_schedules(
        &self,
        roster_cache: Option<&crate::cache::RosterCache>,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employee_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<EmployeeSchedule>, AppError>;

    async fn get_assignment_history(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<crate::models::AssignmentHistory>, AppError>;

    async fn get_employee_rosters(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ShiftRoster>, AppError>;

    async fn upsert_roster(&self, roster: &ShiftRoster) -> Result<ShiftRoster, AppError>;

    async fn bulk_upsert_roster(
        &self,
        employee_id: Uuid,
        items: Vec<crate::models::RosterItem>,
    ) -> Result<(), AppError>;

    async fn get_effective_shift(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<(Option<Uuid>, Vec<crate::models::ScheduleTrace>), AppError>;

    async fn delete_roster(&self, employee_id: Uuid, date: NaiveDate) -> Result<(), AppError>;
}
