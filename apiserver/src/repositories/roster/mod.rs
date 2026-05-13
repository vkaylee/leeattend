//! Roster repository — split into domain sub-modules.
//!
//! - [`shift_resolution`]: Pure shift resolution logic (no DB)
//! - [`schedule_queries`]: DB-backed schedule grid + effective shift queries
//! - [`roster_crud`]: Roster CRUD operations + assignment history

pub mod roster_crud;
pub mod schedule_queries;
pub mod shift_resolution;

// Re-export free functions for backward compatibility
pub use roster_crud::{
    bulk_upsert_roster, delete_roster, get_assignment_history, get_employee_rosters, upsert_roster,
};
pub use schedule_queries::{get_effective_shift, get_workspace_schedules};
pub use shift_resolution::resolve_effective_shift_from_data;

use crate::errors::AppError;
use crate::models::{EmployeeSchedule, ShiftRoster};
use async_trait::async_trait;
use chrono::NaiveDate;

use uuid::Uuid;

#[async_trait]
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
