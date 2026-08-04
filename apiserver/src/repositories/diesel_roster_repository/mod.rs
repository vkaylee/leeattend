use crate::cache::RosterCache;
use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::{AssignmentHistory, EmployeeSchedule, ShiftRoster};
use crate::repositories::roster::RosterRepository;
use async_trait::async_trait;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod roster;
pub mod schedules;
pub mod segments;
pub mod shifts;

#[cfg(test)]
mod tests;

pub struct DieselRosterRepository {
    diesel_pools: DieselPools,
    #[allow(dead_code)]
    roster_cache: Option<Arc<RosterCache>>,
}

impl DieselRosterRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self {
            diesel_pools,
            roster_cache: None,
        }
    }

    /// Inject a [`RosterCache`] for use by [`get_effective_shift`].
    /// When set, shift resolution reads from the cache instead of
    /// querying individual tables (P2–P5).
    #[must_use]
    pub fn with_roster_cache(mut self, cache: Arc<RosterCache>) -> Self {
        self.roster_cache = Some(cache);
        self
    }
}

pub(super) fn effective_employment_range(
    report_start: NaiveDate,
    report_end: NaiveDate,
    employment_start: NaiveDate,
    employment_end: Option<NaiveDate>,
) -> Option<(NaiveDate, NaiveDate)> {
    let start = report_start.max(employment_start);
    let end = employment_end.map_or(report_end, |e| report_end.min(e));
    if start <= end {
        Some((start, end))
    } else {
        None
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl RosterRepository for DieselRosterRepository {
    async fn get_workspace_schedules(
        &self,
        roster_cache: Option<&RosterCache>,
        auth_token: &TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employment_ids: Option<&Vec<Uuid>>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<EmployeeSchedule>, i64), AppError> {
        self.get_workspace_schedules(
            roster_cache,
            auth_token,
            start_date,
            end_date,
            employment_ids,
            page,
            limit,
        )
        .await
    }

    async fn get_workspace_segments(
        &self,
        roster_cache: Option<&RosterCache>,
        auth_token: &TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employment_ids: Option<&Vec<Uuid>>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<crate::models::schedule::EmployeeSegments>, i64), AppError> {
        self.get_workspace_segments(
            roster_cache,
            auth_token,
            start_date,
            end_date,
            employment_ids,
            page,
            limit,
        )
        .await
    }

    async fn get_assignment_history(
        &self,
        auth_token: &TenantAuthToken,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<AssignmentHistory>, i64), AppError> {
        self.get_assignment_history(auth_token, page, limit).await
    }

    async fn get_employment_rosters(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<ShiftRoster>, i64), AppError> {
        self.get_employment_rosters(auth_token, employment_id, start_date, end_date, page, limit)
            .await
    }

    async fn get_employment_rosters_batch(
        &self,
        auth_token: &TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<Uuid, Vec<ShiftRoster>>, AppError> {
        self.get_employment_rosters_batch(auth_token, employment_ids, start_date, end_date)
            .await
    }

    async fn upsert_roster(
        &self,
        auth_token: &TenantAuthToken,
        roster: &ShiftRoster,
    ) -> Result<ShiftRoster, AppError> {
        self.upsert_roster(auth_token, roster).await
    }

    async fn bulk_upsert_roster(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        items: Vec<crate::models::RosterItem>,
    ) -> Result<(), AppError> {
        self.bulk_upsert_roster(auth_token, employment_id, items)
            .await
    }

    async fn get_effective_shift(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<
        (
            Option<Uuid>,
            Vec<crate::models::ScheduleTrace>,
            std::sync::Arc<crate::cache::RosterData>,
        ),
        AppError,
    > {
        self.get_effective_shift(auth_token, employment_id, date)
            .await
    }

    async fn delete_roster(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError> {
        self.delete_roster(auth_token, employment_id, date).await
    }

    async fn get_raw_roster_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<Arc<crate::cache::RosterData>, AppError> {
        self.get_raw_roster_data(workspace_id).await
    }
}
