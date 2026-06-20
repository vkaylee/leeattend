use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use std::sync::Arc;

/// Hour threshold below which a log is considered "early in the day"
/// and may belong to an overnight shift from the previous day.
const OVERNIGHT_REPROCESS_CUTOFF_HOUR: u32 = 12;

/// Number of days to search before/after the target range for adjacent entries.
const ADJACENT_ENTRY_SEARCH_DAYS: i64 = 2;

/// Extra hours of padding when searching for raw logs around a date range.
const LOG_SEARCH_PADDING_HOURS: i64 = 12;

use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use tracing::info;
use uuid::Uuid;

use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::repositories::shift_repository::ShiftRepository;

use crate::models::{RawAttendanceLog, TimeEntry};
use crate::repositories::roster::RosterRepository;
use crate::repositories::TimeEntryRepository;

/// Trait abstracting the pairing service's public API for dependency injection.
///
/// Enables callers to depend on `Arc<dyn PairingProcessor>` instead of the
/// concrete `PairingService`, improving unit test isolation.
#[async_trait::async_trait]
pub trait PairingProcessor: Send + Sync {
    async fn process_new_raw_log(&self, log: &RawAttendanceLog) -> Result<(), AppError>;

    async fn reprocess_daily_attendance(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError>;

    async fn reprocess_attendance_range(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError>;
}
use crate::services::pairing_algo::{self, PairingShiftConfig};
use crate::services::SummaryRecalculator;

pub struct PairingService {
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
    raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    shift_repo: Arc<dyn ShiftRepository>,
    summary_service: Arc<dyn SummaryRecalculator>,
}

pub struct PairingServiceArgs {
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
    raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    shift_repo: Arc<dyn ShiftRepository>,
    summary_service: Arc<dyn SummaryRecalculator>,
}

impl PairingServiceArgs {
    pub fn new(
        time_entry_repo: Arc<dyn TimeEntryRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
        raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
        shift_repo: Arc<dyn ShiftRepository>,
        summary_service: Arc<dyn SummaryRecalculator>,
    ) -> Self {
        Self {
            time_entry_repo,
            roster_repo,
            employee_repo,
            raw_log_repo,
            shift_repo,
            summary_service,
        }
    }
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

    async fn fetch_schedules_for_range(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<crate::models::EmployeeSchedule, AppError> {
        let schedules = self
            .roster_repo
            .get_workspace_schedules(
                None,
                auth_token,
                start_date,
                end_date,
                Some(&vec![employment_id]),
            )
            .await?;

        schedules.into_iter().next().ok_or_else(|| {
            AppError::Internal(format!(
                "No schedule found for employee {employment_id} in range {start_date} to {end_date}"
            ))
        })
    }

    fn process_range_days(
        curr_date: NaiveDate,
        end_date: NaiveDate,
        emp_sched: &crate::models::EmployeeSchedule,
        tz: Tz,
        workspace_id: Uuid,
        employment_id: Uuid,
        raw_logs: &[RawAttendanceLog],
        adjacent_entries: &[TimeEntry],
    ) -> Result<Vec<TimeEntry>, AppError> {
        let mut new_entries_accumulated = Vec::new();
        let mut curr = curr_date;

        let schedule_map: std::collections::HashMap<NaiveDate, &crate::models::EffectiveSchedule> =
            emp_sched.schedules().iter().map(|s| (s.date(), s)).collect();

        while curr <= end_date {
            if let Some(sched) = schedule_map.get(&curr) {
                let day_new_entries = Self::process_single_day_in_range(
                    curr,
                    tz,
                    workspace_id,
                    employment_id,
                    sched,
                    raw_logs,
                    adjacent_entries,
                )?;
                new_entries_accumulated.extend(day_new_entries);
            }
            curr = curr.succ_opt().unwrap_or(curr);
        }
        Ok(new_entries_accumulated)
    }

    async fn save_range_entries(
        &self,
        auth_token: &TenantAuthToken,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        tz: Tz,
        new_entries: Vec<TimeEntry>,
    ) -> Result<(), AppError> {
        let range_start_local = crate::utils::tz::local_to_tz(&tz, start_date, 0, 0, 0)?;
        let range_end_local =
            crate::utils::tz::local_to_tz(&tz, end_date + Duration::days(1), 0, 0, 0)?;
        let range_start_utc = range_start_local.with_timezone(&Utc);
        let range_end_utc = range_end_local.with_timezone(&Utc);

        self.time_entry_repo
            .replace_time_entries(
                auth_token,
                employment_id,
                range_start_utc,
                range_end_utc,
                new_entries,
            )
            .await?;

        self.recalculate_range_summary_with_retry(
            workspace_id,
            employment_id,
            start_date,
            end_date,
        )
        .await?;

        Ok(())
    }

    async fn handle_overnight_tail(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        end_date: NaiveDate,
        emp_sched: &crate::models::EmployeeSchedule,
        tz: Tz,
    ) -> Result<(), AppError> {
        let schedule_map: std::collections::HashMap<NaiveDate, &crate::models::EffectiveSchedule> =
            emp_sched.schedules().iter().map(|s| (s.date(), s)).collect();

        let last_day_is_overnight =
            if let Some(sched) = schedule_map.get(&end_date) {
                if let (Some(st), Some(et)) = (sched.start_time(), sched.end_time()) {
                    et < st
                } else {
                    false
                }
            } else {
                false
            };

        if last_day_is_overnight {
            let next_date = end_date + Duration::days(1);
            info!(
                workspace_id = %workspace_id,
                employment_id = %employment_id,
                next_date = %next_date,
                "Last day of range is overnight, triggering next day reprocessing for self-healing"
            );
            let next_shift_config = self
                .reprocess_single_day(workspace_id, employment_id, next_date, tz, None)
                .await?;
            self.propagate_overnight_reprocess(
                workspace_id,
                employment_id,
                next_date,
                tz,
                next_shift_config,
            )
            .await?;
        }

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
        info!(
            workspace_id = %workspace_id,
            employment_id = %employment_id,
            start_date = %start_date,
            end_date = %end_date,
            "Reprocessing range attendance"
        );

        let auth_token = TenantAuthToken::system_override(workspace_id);

        let emp_sched = self
            .fetch_schedules_for_range(&auth_token, employment_id, start_date, end_date)
            .await?;

        let (raw_logs, adjacent_entries) = self
            .fetch_logs_and_entries_for_range(
                &auth_token,
                workspace_id,
                employment_id,
                start_date,
                end_date,
                tz,
            )
            .await?;

        let new_entries = Self::process_range_days(
            start_date,
            end_date,
            &emp_sched,
            tz,
            workspace_id,
            employment_id,
            &raw_logs,
            &adjacent_entries,
        )?;

        self.save_range_entries(
            &auth_token,
            workspace_id,
            employment_id,
            start_date,
            end_date,
            tz,
            new_entries,
        )
        .await?;

        self.handle_overnight_tail(workspace_id, employment_id, end_date, &emp_sched, tz)
            .await?;

        Ok(())
    }

    fn process_single_day_in_range(
        curr_date: NaiveDate,
        tz: Tz,
        workspace_id: Uuid,
        employment_id: Uuid,
        sched: &crate::models::schedule::EffectiveSchedule,
        raw_logs: &[RawAttendanceLog],
        adjacent_entries: &[TimeEntry],
    ) -> Result<Vec<TimeEntry>, AppError> {
        let context = crate::services::pairing_context::PairingContext::new(curr_date, tz);
        let shift_config = context.build_shift_config_from_schedule(sched);

        // Calculate fetch window for this specific day
        let (window_start_utc, window_end_utc) =
            context.calculate_fetch_window(shift_config.as_ref())?;

        // Exclude logs consumed by adjacent days (excluding current date)
        let consumed_log_ids =
            ConsumedLogTracker::extract_consumed_ids(adjacent_entries, curr_date, tz);

        let filtered_raw_logs: Vec<RawAttendanceLog> = raw_logs
            .iter()
            .filter(|log| {
                log.punch_time() >= window_start_utc && log.punch_time() <= window_end_utc
            })
            .filter(|log| !consumed_log_ids.contains(&log.id()))
            .cloned()
            .collect();

        let filtered_log_events = context.build_log_events(&filtered_raw_logs)?;

        if filtered_log_events.is_empty() {
            return Ok(Vec::new());
        }

        let strategy = pairing_algo::get_strategy();
        let sessions = strategy.pair(&filtered_log_events, shift_config.as_ref());

        let traces_val = serde_json::json!(sched.traces());
        context.build_time_entries(
            workspace_id,
            employment_id,
            sched.shift_id(),
            sessions,
            &traces_val,
        )
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

    async fn propagate_overnight_reprocess(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        tz: Tz,
        initial_shift_config: Option<PairingShiftConfig>,
    ) -> Result<(), AppError> {
        let Some(mut current_config) = initial_shift_config else {
            return Ok(());
        };

        if current_config.end_minutes() <= 1440 {
            return Ok(());
        }

        let lookahead_start = start_date + Duration::days(1);
        let lookahead_end = start_date + Duration::days(7);

        info!(
            workspace_id = %workspace_id,
            employment_id = %employment_id,
            start_date = %lookahead_start,
            end_date = %lookahead_end,
            "Propagating overnight shifts, batch fetching lookahead range"
        );

        let auth_token = TenantAuthToken::system_override(workspace_id);

        // 1. Batch fetch schedules
        let emp_sched = match self
            .fetch_schedules_for_range(&auth_token, employment_id, lookahead_start, lookahead_end)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "No schedules found during propagation lookahead, skipping propagation");
                return Ok(());
            }
        };

        // 2. Batch fetch logs and adjacent entries
        let (raw_logs, adjacent_entries) = self
            .fetch_logs_and_entries_for_range(
                &auth_token,
                workspace_id,
                employment_id,
                lookahead_start,
                lookahead_end,
                tz,
            )
            .await?;

        // 3. Process day-by-day in memory
        let mut new_entries_accumulated: Vec<TimeEntry> = Vec::new();
        let mut resolved_end_date = start_date;
        let mut current_date = lookahead_start;

        let schedule_map: std::collections::HashMap<NaiveDate, &crate::models::schedule::EffectiveSchedule> =
            emp_sched.schedules().iter().map(|s| (s.date(), s)).collect();

        while current_date <= lookahead_end {
            if current_config.end_minutes() <= 1440 {
                break;
            }

            if let Some(sched) = schedule_map.get(&current_date) {
                let (day_entries, next_shift_config) = Self::process_propagation_day(
                    current_date,
                    tz,
                    workspace_id,
                    employment_id,
                    sched,
                    &raw_logs,
                    &adjacent_entries,
                    &new_entries_accumulated,
                )?;

                new_entries_accumulated.extend(day_entries);
                resolved_end_date = current_date;

                if let Some(config) = next_shift_config {
                    current_config = config;
                } else {
                    break;
                }
            } else {
                break;
            }

            current_date += Duration::days(1);
        }

        // 4. If we actually reprocessed any dates, save the entries and update summaries in bulk
        if resolved_end_date > start_date {
            self.save_range_entries(
                &auth_token,
                workspace_id,
                employment_id,
                lookahead_start,
                resolved_end_date,
                tz,
                new_entries_accumulated,
            )
            .await?;
        }

        Ok(())
    }

    fn process_propagation_day(
        current_date: NaiveDate,
        tz: Tz,
        workspace_id: Uuid,
        employment_id: Uuid,
        sched: &crate::models::schedule::EffectiveSchedule,
        raw_logs: &[RawAttendanceLog],
        adjacent_entries: &[TimeEntry],
        new_entries_accumulated: &[TimeEntry],
    ) -> Result<(Vec<TimeEntry>, Option<PairingShiftConfig>), AppError> {
        let context = crate::services::pairing_context::PairingContext::new(current_date, tz);
        let next_shift_config = context.build_shift_config_from_schedule(sched);

        let (window_start_utc, window_end_utc) =
            context.calculate_fetch_window(next_shift_config.as_ref())?;

        let mut consumed_log_ids =
            ConsumedLogTracker::extract_consumed_ids(adjacent_entries, current_date, tz);

        for entry in new_entries_accumulated {
            if let Some(metadata) = entry.generation_metadata() {
                if let Ok(paired_meta) = serde_json::from_value::<
                    crate::services::pairing_algo::PairingMetadata,
                >(metadata.clone())
                {
                    for id in paired_meta.log_ids() {
                        consumed_log_ids.insert(*id);
                    }
                }
            }
        }

        let filtered_raw_logs: Vec<RawAttendanceLog> = raw_logs
            .iter()
            .filter(|log| {
                log.punch_time() >= window_start_utc && log.punch_time() <= window_end_utc
            })
            .filter(|log| !consumed_log_ids.contains(&log.id()))
            .cloned()
            .collect();

        let filtered_log_events = context.build_log_events(&filtered_raw_logs)?;

        let day_entries = if filtered_log_events.is_empty() {
            Vec::new()
        } else {
            let strategy = pairing_algo::get_strategy();
            let sessions = strategy.pair(&filtered_log_events, next_shift_config.as_ref());

            let traces_val = serde_json::json!(sched.traces());
            context.build_time_entries(
                workspace_id,
                employment_id,
                sched.shift_id(),
                sessions,
                &traces_val,
            )?
        };

        Ok((day_entries, next_shift_config))
    }

    async fn fetch_logs_and_entries_for_range(
        &self,
        auth_token: &TenantAuthToken,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        tz: Tz,
    ) -> Result<(Vec<RawAttendanceLog>, Vec<TimeEntry>), AppError> {
        let range_start_local = crate::utils::tz::local_to_tz(&tz, start_date, 0, 0, 0)?;
        let range_end_local =
            crate::utils::tz::local_to_tz(&tz, end_date + Duration::days(1), 0, 0, 0)?;

        let start_search_utc = (range_start_local - Duration::hours(LOG_SEARCH_PADDING_HOURS)).with_timezone(&Utc);
        let end_search_utc = (range_end_local + Duration::hours(LOG_SEARCH_PADDING_HOURS)).with_timezone(&Utc);

        let raw_logs = self
            .raw_log_repo
            .get_employment_logs_in_time_range(
                workspace_id,
                employment_id,
                start_search_utc,
                end_search_utc,
            )
            .await?;

        let search_start_utc = (range_start_local - Duration::days(ADJACENT_ENTRY_SEARCH_DAYS)).with_timezone(&Utc);
        let search_end_utc = (range_end_local + Duration::days(ADJACENT_ENTRY_SEARCH_DAYS)).with_timezone(&Utc);

        let adjacent_entries = self
            .time_entry_repo
            .get_employment_time_entries(
                auth_token,
                employment_id,
                search_start_utc,
                search_end_utc,
            )
            .await?;

        Ok((raw_logs, adjacent_entries))
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn reprocess_single_day(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
        tz: Tz,
        prefetched_adjacent: Option<&[TimeEntry]>,
    ) -> Result<Option<PairingShiftConfig>, AppError> {
        info!(
            workspace_id = %workspace_id,
            employment_id = %employment_id,
            date = %date,
            "Reprocessing single day"
        );

        let auth_token = TenantAuthToken::system_override(workspace_id);
        let context = crate::services::pairing_context::PairingContext::new(date, tz);

        // 1. Fetch effective schedule and config
        let (shift_id_opt, shift_config, traces) = self
            .fetch_effective_shift_config(&auth_token, employment_id, date, &context)
            .await?;

        // 2. Fetch raw logs
        let start_of_day_local = crate::utils::tz::local_to_tz(&tz, date, 0, 0, 0)?;
        let start_of_day_utc = start_of_day_local.with_timezone(&Utc);
        let next_day_utc = (start_of_day_local + Duration::days(1)).with_timezone(&Utc);

        let (window_start_utc, window_end_utc) =
            context.calculate_fetch_window(shift_config.as_ref())?;

        let logs = self
            .raw_log_repo
            .get_employment_logs_in_time_range(
                workspace_id,
                employment_id,
                window_start_utc,
                window_end_utc,
            )
            .await?;

        // 3. Exclude logs already consumed by adjacent days
        let consumed_log_ids = if let Some(adjacent) = prefetched_adjacent {
            ConsumedLogTracker::extract_consumed_ids(adjacent, date, tz)
        } else {
            let tracker =
                ConsumedLogTracker::new(&*self.time_entry_repo, &auth_token, employment_id, date, tz);
            tracker
                .get_consumed_log_ids(start_of_day_utc, next_day_utc)
                .await?
        };

        let filtered_raw_logs: Vec<RawAttendanceLog> = logs
            .into_iter()
            .filter(|log| !consumed_log_ids.contains(&log.id()))
            .collect();

        let filtered_log_events = context.build_log_events(&filtered_raw_logs)?;

        if filtered_log_events.is_empty() {
            self.clear_auto_entries(&auth_token, employment_id, start_of_day_utc, next_day_utc)
                .await?;

            self.recalculate_summary_with_retry(workspace_id, employment_id, date)
                .await?;

            return Ok(shift_config);
        }

        // 4. Run Algorithm
        let strategy = pairing_algo::get_strategy();
        let sessions = strategy.pair(&filtered_log_events, shift_config.as_ref());

        // 5. Update Database and Recalculate
        let new_entries = context.build_time_entries(
            workspace_id,
            employment_id,
            shift_id_opt,
            sessions,
            &traces,
        )?;

        self.save_paired_entries_and_update_summary(
            &auth_token,
            employment_id,
            date,
            start_of_day_utc,
            next_day_utc,
            new_entries,
        )
        .await?;

        Ok(shift_config)
    }

    async fn fetch_effective_shift_config(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        context: &crate::services::pairing_context::PairingContext,
    ) -> Result<(Option<Uuid>, Option<PairingShiftConfig>, serde_json::Value), AppError> {
        let (shift_id_opt, traces) = self
            .roster_repo
            .get_effective_shift(auth_token, employment_id, date)
            .await?;

        let shift = if let Some(shift_id) = shift_id_opt {
            self.shift_repo
                .get_shift_by_id(auth_token, shift_id)
                .await?
        } else {
            None
        };

        let shift_config = context.build_shift_config(shift.as_ref());
        Ok((shift_id_opt, shift_config, serde_json::json!(traces)))
    }

    async fn save_paired_entries_and_update_summary(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        start_of_day_utc: DateTime<Utc>,
        next_day_utc: DateTime<Utc>,
        new_entries: Vec<TimeEntry>,
    ) -> Result<(), AppError> {
        self.time_entry_repo
            .replace_time_entries(
                auth_token,
                employment_id,
                start_of_day_utc,
                next_day_utc,
                new_entries,
            )
            .await?;

        // Recalculate daily summary for OT workflow with retries
        self.recalculate_summary_with_retry(auth_token.workspace_id(), employment_id, date)
            .await?;
        Ok(())
    }

    async fn recalculate_summary_with_retry(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError> {
        let summary_service = &self.summary_service;
        retry_with_log(
            "recalculate_daily_summary",
            workspace_id,
            employment_id,
            &format!("date={date}"),
            3,
            100,
            || summary_service.recalculate_daily_summary(workspace_id, employment_id, date),
        )
        .await
    }

    async fn recalculate_range_summary_with_retry(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError> {
        let summary_service = &self.summary_service;
        retry_with_log(
            "recalculate_attendance_range",
            workspace_id,
            employment_id,
            &format!("range={start_date}..{end_date}"),
            3,
            100,
            || {
                summary_service.recalculate_attendance_range(
                    workspace_id,
                    employment_id,
                    start_date,
                    end_date,
                )
            },
        )
        .await
    }

    async fn clear_auto_entries(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(), AppError> {
        self.time_entry_repo
            .replace_time_entries(auth_token, employment_id, start, end, Vec::new())
            .await
    }
}

/// Execute an async operation with retries and structured logging.
async fn retry_with_log<F, Fut>(
    operation_name: &str,
    workspace_id: Uuid,
    employment_id: Uuid,
    context: &str,
    max_attempts: u32,
    delay_ms: u64,
    f: F,
) -> Result<(), AppError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<(), AppError>>,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if attempts >= max_attempts {
                    tracing::error!(
                        workspace_id = %workspace_id,
                        employment_id = %employment_id,
                        context = %context,
                        error = %e,
                        "Failed {} after {} attempts",
                        operation_name,
                        max_attempts,
                    );
                    return Err(e);
                } else {
                    tracing::warn!(
                        workspace_id = %workspace_id,
                        employment_id = %employment_id,
                        context = %context,
                        error = %e,
                        attempt = attempts,
                        "Temporary failure in {}, retrying...",
                        operation_name,
                    );
                    let base_delay = delay_ms * 2u64.pow(attempts - 1);
                    let jitter = rand::random::<u64>() % 50;
                    let delay = base_delay + jitter;
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }
}

struct ConsumedLogTracker<'a> {
    time_entry_repo: &'a dyn crate::repositories::TimeEntryRepository,
    auth_token: &'a TenantAuthToken,
    employment_id: Uuid,
    date: NaiveDate,
    tz: Tz,
}

impl<'a> ConsumedLogTracker<'a> {
    fn new(
        time_entry_repo: &'a dyn crate::repositories::TimeEntryRepository,
        auth_token: &'a TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        tz: Tz,
    ) -> Self {
        Self {
            time_entry_repo,
            auth_token,
            employment_id,
            date,
            tz,
        }
    }

    /// Extract consumed log IDs from pre-fetched adjacent entries.
    /// Shared between single-day reprocessing and range reprocessing.
    fn extract_consumed_ids(
        entries: &[TimeEntry],
        current_date: NaiveDate,
        tz: Tz,
    ) -> std::collections::HashSet<Uuid> {
        let mut consumed = std::collections::HashSet::new();
        for entry in entries {
            let entry_local = entry.punch_in_time().with_timezone(&tz);
            if entry_local.date_naive() == current_date {
                continue;
            }
            if entry_local.date_naive() == current_date + Duration::days(1)
                && entry_local.hour() < OVERNIGHT_REPROCESS_CUTOFF_HOUR
            {
                continue;
            }
            if let Some(metadata) = entry.generation_metadata() {
                if let Ok(paired_meta) = serde_json::from_value::<
                    crate::services::pairing_algo::PairingMetadata,
                >(metadata.clone())
                {
                    for id in paired_meta.log_ids() {
                        consumed.insert(*id);
                    }
                }
            }
        }
        consumed
    }

    async fn get_consumed_log_ids(
        &self,
        start_of_day_utc: DateTime<Utc>,
        next_day_utc: DateTime<Utc>,
    ) -> Result<std::collections::HashSet<Uuid>, AppError> {
        // Fetch adjacent time entries to check for already-paired logs
        let search_start = start_of_day_utc - Duration::days(ADJACENT_ENTRY_SEARCH_DAYS);
        let search_end = next_day_utc + Duration::days(ADJACENT_ENTRY_SEARCH_DAYS);
        let adjacent_entries = self
            .time_entry_repo
            .get_employment_time_entries(
                self.auth_token,
                self.employment_id,
                search_start,
                search_end,
            )
            .await?;

        Ok(Self::extract_consumed_ids(
            &adjacent_entries,
            self.date,
            self.tz,
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimeEntry;
    use chrono::{TimeZone, Utc};
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository;
    use crate::repositories::roster::MockRosterRepository;
    use crate::repositories::shift_repository::MockShiftRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;

    use crate::services::attendance_summary_service::MockSummaryRecalculator;

    #[tokio::test]
    async fn test_process_new_raw_log_duplicate() {
        let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
        mock_raw_repo
            .expect_has_duplicate_raw_log()
            .times(1)
            .returning(|_, _, _, _| Box::pin(async { Ok(true) }));

        let service = PairingService::new(PairingServiceArgs::new(
            Arc::new(MockTimeEntryRepository::new()),
            Arc::new(MockRosterRepository::new()),
            Arc::new(MockEmployeeRepository::new()),
            Arc::new(mock_raw_repo),
            Arc::new(MockShiftRepository::new()),
            Arc::new(MockSummaryRecalculator::new()),
        ));

        let log = crate::models::RawAttendanceLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now(),
            crate::models::enums::LogSource::Manual,
            None,
            Some(serde_json::json!({})),
            1,
            1,
            Utc::now(),
        );

        let result = service.process_new_raw_log(&log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clear_auto_entries() {
        let mut mock_time_entry_repo = MockTimeEntryRepository::new();
        mock_time_entry_repo
            .expect_replace_time_entries()
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let service = PairingService::new(PairingServiceArgs::new(
            Arc::new(mock_time_entry_repo),
            Arc::new(MockRosterRepository::new()),
            Arc::new(MockEmployeeRepository::new()),
            Arc::new(MockRawAttendanceLogRepository::new()),
            Arc::new(MockShiftRepository::new()),
            Arc::new(MockSummaryRecalculator::new()),
        ));

        let auth_token = TenantAuthToken::system_override(Uuid::new_v4());
        let result = service
            .clear_auto_entries(&auth_token, Uuid::new_v4(), Utc::now(), Utc::now())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reprocess_daily_attendance_no_logs() {
        let mut mock_roster_repo = MockRosterRepository::new();
        mock_roster_repo
            .expect_get_effective_shift()
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok((None, vec![])) }));

        let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
        mock_raw_repo
            .expect_get_employment_logs_in_time_range()
            .times(1)
            .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

        let mut mock_time_entry_repo = MockTimeEntryRepository::new();
        mock_time_entry_repo
            .expect_get_employment_time_entries()
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));
        mock_time_entry_repo
            .expect_replace_time_entries()
            .times(1) // Called by clear_auto_entries inside reprocess_daily_attendance
            .returning(|_, _, _, _, _| Ok(()));

        let mut mock_summary_service = MockSummaryRecalculator::new();
        mock_summary_service
            .expect_recalculate_daily_summary()
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = PairingService::new(PairingServiceArgs::new(
            Arc::new(mock_time_entry_repo),
            Arc::new(mock_roster_repo),
            Arc::new(MockEmployeeRepository::new()),
            Arc::new(mock_raw_repo),
            Arc::new(MockShiftRepository::new()),
            Arc::new(mock_summary_service),
        ));

        let result = service
            .reprocess_daily_attendance(
                Uuid::new_v4(),
                Uuid::new_v4(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                chrono_tz::UTC,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reprocess_daily_attendance_excludes_already_paired_logs() {
        let mut mock_roster_repo = MockRosterRepository::new();
        mock_roster_repo
            .expect_get_effective_shift()
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok((None, vec![])) }));

        let paired_log_id = Uuid::new_v4();
        let fresh_log_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        // 1. Mock time entry repo to return one adjacent entry with paired_log_id consumed
        let mut mock_time_entry_repo = MockTimeEntryRepository::new();
        let test_date = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();
        let log_time = Utc.with_ymd_and_hms(2026, 6, 19, 10, 0, 0).unwrap();
        let adjacent_time = log_time - chrono::Duration::days(1);
        let adjacent_entry = TimeEntry::new(
            Uuid::new_v4(),
            emp_id,
            ws_id,
            None,
            adjacent_time,
            None,
            None,
            false,
            "work".to_string(),
            None,
            Some(serde_json::json!({ "log_ids": vec![paired_log_id.to_string()] })),
            adjacent_time,
            adjacent_time,
        );
        mock_time_entry_repo
            .expect_get_employment_time_entries()
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![adjacent_entry.clone()]));

        // Expected replace time entries is called with the fresh log only
        mock_time_entry_repo
            .expect_replace_time_entries()
            .times(1)
            .returning(move |_, _, _, _, entries| {
                // Assert that the generated entry only corresponds to the fresh log
                assert_eq!(entries.len(), 1);
                let meta = entries[0].generation_metadata().unwrap();
                let log_ids = meta["log_ids"].as_array().unwrap();
                assert_eq!(log_ids.len(), 1);
                assert_eq!(log_ids[0].as_str().unwrap(), fresh_log_id.to_string());
                Ok(())
            });

        // 2. Mock raw log repo to return both logs
        let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
        let log1 = RawAttendanceLog::new(
            paired_log_id,
            ws_id,
            emp_id,
            log_time,
            crate::models::enums::LogSource::Manual,
            None,
            None,
            1,
            1,
            log_time,
        );
        let log2 = RawAttendanceLog::new(
            fresh_log_id,
            ws_id,
            emp_id,
            log_time + chrono::Duration::minutes(5),
            crate::models::enums::LogSource::Manual,
            None,
            None,
            1,
            1,
            log_time,
        );

        mock_raw_repo
            .expect_get_employment_logs_in_time_range()
            .times(1)
            .returning(move |_, _, _, _| {
                let l1 = log1.clone();
                let l2 = log2.clone();
                Box::pin(async move { Ok(vec![l1, l2]) })
            });

        let mut mock_summary_service = MockSummaryRecalculator::new();
        mock_summary_service
            .expect_recalculate_daily_summary()
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = PairingService::new(PairingServiceArgs::new(
            Arc::new(mock_time_entry_repo),
            Arc::new(mock_roster_repo),
            Arc::new(MockEmployeeRepository::new()),
            Arc::new(mock_raw_repo),
            Arc::new(MockShiftRepository::new()),
            Arc::new(mock_summary_service),
        ));

        let result = service
            .reprocess_daily_attendance(ws_id, emp_id, test_date, chrono_tz::UTC)
            .await;

        assert!(result.is_ok(), "Reprocess failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_reprocess_attendance_range_success() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2026, 6, 18).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();

        // 1. Mock Roster repo to return schedules for the range
        let mut mock_roster_repo = MockRosterRepository::new();
        let sched_emp_id = emp_id;
        mock_roster_repo
            .expect_get_workspace_schedules()
            .times(1)
            .returning(move |_, _, s_date, e_date, _| {
                let schedules = vec![
                    crate::models::EffectiveSchedule::new(
                        s_date,
                        None,
                        "none".to_string(),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        vec![],
                        false,
                        false,
                        None,
                    ),
                    crate::models::EffectiveSchedule::new(
                        e_date,
                        None,
                        "none".to_string(),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        vec![],
                        false,
                        false,
                        None,
                    ),
                ];
                Box::pin(async move {
                    Ok(vec![crate::models::EmployeeSchedule::new(
                        sched_emp_id,
                        sched_emp_id,
                        "EMP001".to_string(),
                        "Test Employee".to_string(),
                        schedules,
                    )])
                })
            });

        // 2. Mock raw log repo to return logs
        let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
        mock_raw_repo
            .expect_get_employment_logs_in_time_range()
            .times(1)
            .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

        // 3. Mock time entry repo to return empty adjacents and allow replacement
        let mut mock_time_entry_repo = MockTimeEntryRepository::new();
        mock_time_entry_repo
            .expect_get_employment_time_entries()
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));
        mock_time_entry_repo
            .expect_replace_time_entries()
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        // 4. Mock summary service to recalculate summaries in batch for the range
        let mut mock_summary_service = MockSummaryRecalculator::new();
        mock_summary_service
            .expect_recalculate_attendance_range()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = PairingService::new(PairingServiceArgs::new(
            Arc::new(mock_time_entry_repo),
            Arc::new(mock_roster_repo),
            Arc::new(MockEmployeeRepository::new()),
            Arc::new(mock_raw_repo),
            Arc::new(MockShiftRepository::new()),
            Arc::new(mock_summary_service),
        ));

        let result = service
            .reprocess_attendance_range(ws_id, emp_id, start_date, end_date, chrono_tz::UTC)
            .await;

        assert!(result.is_ok(), "Reprocess range failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_retry_with_log_returns_error_after_max_attempts() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let counter = std::sync::atomic::AtomicU32::new(0);

        let result = retry_with_log(
            "test_op",
            ws_id,
            emp_id,
            "test_context",
            3,
            1,
            || {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Box::pin(async { Err(AppError::Internal("test_err".to_string())) })
            },
        )
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_log_succeeds_on_second_attempt() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let counter = std::sync::atomic::AtomicU32::new(0);

        let result = retry_with_log(
            "test_op",
            ws_id,
            emp_id,
            "test_context",
            3,
            1,
            || {
                let attempts = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Box::pin(async move {
                    if attempts == 0 {
                        Err(AppError::Internal("test_err".to_string()))
                    } else {
                        Ok(())
                    }
                })
            },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
