use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{RawAttendanceLog, TimeEntry};
use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::shift_repository::ShiftRepository;
use crate::repositories::TimeEntryRepository;
use crate::services::pairing::types::{PairingProcessor, PairingServiceArgs, PairingShiftConfig};
use crate::services::SummaryRecalculator;

use super::types::OVERNIGHT_REPROCESS_CUTOFF_HOUR;

pub struct PairingService {
    pub(crate) time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub(crate) roster_repo: Arc<dyn RosterRepository>,
    pub(crate) employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
    pub(crate) raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    pub(crate) shift_repo: Arc<dyn ShiftRepository>,
    pub(crate) summary_service: Arc<dyn SummaryRecalculator>,
}

impl PairingService {
    pub fn new(args: PairingServiceArgs) -> Self {
        Self {
            time_entry_repo: args.time_entry_repo,
            roster_repo: args.roster_repo,
            employee_repo: args.employee_repo,
            raw_log_repo: args.raw_log_repo,
            shift_repo: args.shift_repo,
            summary_service: args.summary_service,
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn process_new_raw_log(&self, log: &RawAttendanceLog) -> Result<(), AppError> {
        // 1. Deduplication (exact match check)
        let is_duplicate = self
            .raw_log_repo
            .has_duplicate_raw_log(
                log.workspace_id(),
                log.employment_id(),
                log.id(),
                log.punch_time(),
            )
            .await?;

        if is_duplicate {
            info!(
                workspace_id = %log.workspace_id(),
                employment_id = %log.employment_id(),
                log_id = %log.id(),
                punch_time = %log.punch_time(),
                "Deduplicating log. Found another log with identical timestamp."
            );
            return Ok(());
        }

        // 2. Fetch Timezone (Two-step resolution)
        let auth_token = TenantAuthToken::system_override(log.workspace_id());

        // Step 2a: Get fallback timezone to determine tentative date
        let fallback_tz: Tz = self
            .employee_repo
            .get_employee_timezone_fallback(&auth_token, log.employment_id())
            .await?;

        let tentative_local_dt = log.punch_time().with_timezone(&fallback_tz);
        let tentative_date = tentative_local_dt.date_naive();

        // Step 2b: Get exact timezone considering secondment assignments for that date
        let tz: Tz = self
            .employee_repo
            .get_employee_timezone_at_date(&auth_token, log.employment_id(), tentative_date)
            .await?;

        // 3. Determine Local Date
        let local_dt = log.punch_time().with_timezone(&tz);
        let local_date = local_dt.date_naive();

        // 4. Reprocess previous day first if early in the morning (Overnight checkouts)
        if local_dt.hour() < OVERNIGHT_REPROCESS_CUTOFF_HOUR {
            let prev_date = local_date - Duration::days(1);
            info!(
                workspace_id = %log.workspace_id(),
                employment_id = %log.employment_id(),
                local_dt = %local_dt,
                prev_date = %prev_date,
                "Log is early in day, reprocessing previous day first for overnight checkouts"
            );
            self.reprocess_daily_attendance(log.workspace_id(), log.employment_id(), prev_date, tz)
                .await?;
        }

        // 5. Reprocess that day
        self.reprocess_daily_attendance(log.workspace_id(), log.employment_id(), local_date, tz)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn reprocess_daily_attendance(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError> {
        info!(
            workspace_id = %workspace_id,
            employment_id = %employment_id,
            date = %date,
            timezone = %tz,
            "Reprocessing daily attendance"
        );

        // Process the target day
        let shift_config = self
            .reprocess_single_day(workspace_id, employment_id, date, tz, None)
            .await?;

        // Propagate to subsequent days if overnight
        self.propagate_overnight_reprocess(workspace_id, employment_id, date, tz, shift_config)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn reprocess_attendance_range(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError> {
        super::service_range::reprocess_attendance_range(
            self,
            workspace_id,
            employment_id,
            start_date,
            end_date,
            tz,
        )
        .await
    }

    pub(crate) async fn reprocess_single_day(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
        tz: Tz,
        prefetched_adjacent: Option<&[TimeEntry]>,
    ) -> Result<Option<PairingShiftConfig>, AppError> {
        super::service_single::reprocess_single_day(
            self,
            workspace_id,
            employment_id,
            date,
            tz,
            prefetched_adjacent,
        )
        .await
    }

    pub(crate) async fn propagate_overnight_reprocess(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        tz: Tz,
        initial_shift_config: Option<PairingShiftConfig>,
    ) -> Result<(), AppError> {
        super::service_propagation::propagate_overnight_reprocess(
            self,
            workspace_id,
            employment_id,
            start_date,
            tz,
            initial_shift_config,
        )
        .await
    }

    pub(crate) async fn fetch_schedules_for_range(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<crate::models::EmployeeSchedule, AppError> {
        super::service_helpers::fetch_schedules_for_range(
            self,
            auth_token,
            employment_id,
            start_date,
            end_date,
        )
        .await
    }

    pub(crate) async fn fetch_logs_and_entries_for_range(
        &self,
        auth_token: &TenantAuthToken,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        tz: Tz,
    ) -> Result<(Vec<RawAttendanceLog>, Vec<TimeEntry>), AppError> {
        super::service_helpers::fetch_logs_and_entries_for_range(
            self,
            auth_token,
            workspace_id,
            employment_id,
            start_date,
            end_date,
            tz,
        )
        .await
    }

    pub(crate) async fn clear_auto_entries(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(), AppError> {
        super::service_helpers::clear_auto_entries(self, auth_token, employment_id, start, end)
            .await
    }

    pub(crate) async fn recalculate_summary_with_retry(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError> {
        super::service_helpers::recalculate_summary_with_retry(
            self,
            workspace_id,
            employment_id,
            date,
        )
        .await
    }

    pub(crate) async fn recalculate_range_summary_with_retry(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError> {
        super::service_helpers::recalculate_range_summary_with_retry(
            self,
            workspace_id,
            employment_id,
            start_date,
            end_date,
        )
        .await
    }
}

#[async_trait::async_trait]
impl PairingProcessor for PairingService {
    async fn process_new_raw_log(&self, log: &RawAttendanceLog) -> Result<(), AppError> {
        PairingService::process_new_raw_log(self, log).await
    }

    async fn reprocess_daily_attendance(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError> {
        PairingService::reprocess_daily_attendance(self, workspace_id, employment_id, date, tz)
            .await
    }

    async fn reprocess_attendance_range(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError> {
        PairingService::reprocess_attendance_range(
            self,
            workspace_id,
            employment_id,
            start_date,
            end_date,
            tz,
        )
        .await
    }
}
