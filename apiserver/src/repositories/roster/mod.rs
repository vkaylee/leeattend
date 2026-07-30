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
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employment_ids: Option<&Vec<Uuid>>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<EmployeeSchedule>, i64), AppError>;

    /// Segment-centric variant: resolves segments (via intervals) per day
    /// instead of monolithic schedules.  Returns [`EmployeeSegments`] whose
    /// `segments` vec may contain multiple entries for the same date when
    /// a shift has defined intervals or a split-shift pattern is active.
    ///
    /// Default implementation returns empty — override for real resolution.
    async fn get_workspace_segments(
        &self,
        _roster_cache: Option<&crate::cache::RosterCache>,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
        _employment_ids: Option<&Vec<Uuid>>,
        _page: Option<i64>,
        _limit: Option<i64>,
    ) -> Result<(Vec<crate::models::schedule::EmployeeSegments>, i64), AppError> {
        Ok((Vec::new(), 0))
    }

    async fn get_assignment_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<crate::models::AssignmentHistory>, i64), AppError>;

    async fn get_employment_rosters(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<ShiftRoster>, i64), AppError>;
    async fn get_employment_rosters_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<ShiftRoster>>, AppError>;

    async fn upsert_roster(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        roster: &ShiftRoster,
    ) -> Result<ShiftRoster, AppError>;

    async fn bulk_upsert_roster(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        items: Vec<crate::models::RosterItem>,
    ) -> Result<(), AppError>;

    /// Returns the effective shift ID, resolution traces, and the cached
    /// [`RosterData`] that was used during resolution.  Callers that also
    /// need interval configuration (start/end times, break info, grace
    /// periods) can use the returned [`RosterData`] directly instead of
    /// making a second `get_raw_roster_data` call.
    async fn get_effective_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<
        (
            Option<Uuid>,
            Vec<crate::models::ScheduleTrace>,
            std::sync::Arc<crate::cache::RosterData>,
        ),
        AppError,
    >;

    async fn delete_roster(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError>;

    async fn get_raw_roster_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<std::sync::Arc<crate::cache::RosterData>, AppError>;
}
