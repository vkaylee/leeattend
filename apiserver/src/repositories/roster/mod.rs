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

use crate::models::{EmployeeSchedule, ShiftRoster};
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait RosterRepository: super::PoolBacked + Send + Sync {
    async fn get_workspace_schedules(
        &self,
        roster_cache: Option<&crate::cache::RosterCache>,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employee_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<EmployeeSchedule>, sqlx::Error>;

    async fn get_assignment_history(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<crate::models::AssignmentHistory>, sqlx::Error>;

    async fn get_employee_rosters(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ShiftRoster>, sqlx::Error>;

    async fn upsert_roster(&self, roster: &ShiftRoster) -> Result<ShiftRoster, sqlx::Error>;

    async fn bulk_upsert_roster(
        &self,
        employee_id: Uuid,
        items: Vec<crate::models::RosterItem>,
    ) -> Result<(), sqlx::Error>;

    async fn get_effective_shift(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<(Option<Uuid>, Vec<crate::models::ScheduleTrace>), sqlx::Error>;

    async fn delete_roster(&self, employee_id: Uuid, date: NaiveDate) -> Result<(), sqlx::Error>;
}

pub struct SqlxRosterRepository {
    pub pool: PgPool,
}

impl SqlxRosterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RosterRepository for SqlxRosterRepository {
    async fn get_workspace_schedules(
        &self,
        roster_cache: Option<&crate::cache::RosterCache>,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employee_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<EmployeeSchedule>, sqlx::Error> {
        schedule_queries::get_workspace_schedules(
            &self.pool,
            roster_cache,
            workspace_id,
            start_date,
            end_date,
            employee_ids,
        )
        .await
    }

    async fn get_assignment_history(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<crate::models::AssignmentHistory>, sqlx::Error> {
        roster_crud::get_assignment_history(&self.pool, workspace_id, limit).await
    }

    async fn get_employee_rosters(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ShiftRoster>, sqlx::Error> {
        roster_crud::get_employee_rosters(&self.pool, employee_id, start_date, end_date).await
    }

    async fn upsert_roster(&self, roster: &ShiftRoster) -> Result<ShiftRoster, sqlx::Error> {
        roster_crud::upsert_roster(&self.pool, roster).await
    }

    async fn bulk_upsert_roster(
        &self,
        employee_id: Uuid,
        items: Vec<crate::models::RosterItem>,
    ) -> Result<(), sqlx::Error> {
        roster_crud::bulk_upsert_roster(&self.pool, employee_id, items).await
    }

    async fn get_effective_shift(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<(Option<Uuid>, Vec<crate::models::ScheduleTrace>), sqlx::Error> {
        schedule_queries::get_effective_shift(&self.pool, workspace_id, employee_id, date).await
    }

    async fn delete_roster(&self, employee_id: Uuid, date: NaiveDate) -> Result<(), sqlx::Error> {
        roster_crud::delete_roster(&self.pool, employee_id, date).await
    }
}

impl super::PoolBacked for SqlxRosterRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
