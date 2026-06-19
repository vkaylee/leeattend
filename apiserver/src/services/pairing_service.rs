use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use tracing::info;
use uuid::Uuid;

use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::repositories::shift_repository::ShiftRepository;

use crate::models::{RawAttendanceLog, TimeEntry};
use crate::repositories::roster::RosterRepository;
use crate::repositories::TimeEntryRepository;
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
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
    pub raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    pub shift_repo: Arc<dyn ShiftRepository>,
    pub summary_service: Arc<dyn SummaryRecalculator>,
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

        // 4. Reprocess that day
        self.reprocess_daily_attendance(log.workspace_id(), log.employment_id(), local_date, tz)
            .await?;

        // 5. Reprocess previous day if early in the morning (Overnight checkouts)
        if local_dt.hour() < 12 {
            let prev_date = local_date - Duration::days(1);
            info!(
                workspace_id = %log.workspace_id(),
                employment_id = %log.employment_id(),
                local_dt = %local_dt,
                prev_date = %prev_date,
                "Log is early in day, reprocessing previous day for overnight checkouts"
            );
            self.reprocess_daily_attendance(log.workspace_id(), log.employment_id(), prev_date, tz)
                .await?;
        }

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
            "Reprocessing daily attendance"
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
        let tracker =
            ConsumedLogTracker::new(&*self.time_entry_repo, &auth_token, employment_id, date, tz);
        let consumed_log_ids = tracker
            .get_consumed_log_ids(start_of_day_utc, next_day_utc)
            .await?;

        let filtered_raw_logs: Vec<RawAttendanceLog> = logs
            .into_iter()
            .filter(|log| !consumed_log_ids.contains(&log.id()))
            .collect();

        let filtered_log_events = context.build_log_events(&filtered_raw_logs)?;

        if filtered_log_events.is_empty() {
            self.clear_auto_entries(&auth_token, employment_id, start_of_day_utc, next_day_utc)
                .await?;

            self.recalculate_summary_with_retry(workspace_id, employment_id, date)
                .await;

            return Ok(());
        }

        // 4. Run Algorithm
        let strategy = pairing_algo::get_strategy();
        let shift_configs = if let Some(config) = shift_config {
            vec![config]
        } else {
            vec![]
        };
        let sessions = strategy.pair(&filtered_log_events, &shift_configs);

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
        .await
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
            .await;
        Ok(())
    }

    async fn recalculate_summary_with_retry(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
    ) {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        while attempts < MAX_ATTEMPTS {
            attempts += 1;
            match self
                .summary_service
                .recalculate_daily_summary(workspace_id, employment_id, date)
                .await
            {
                Ok(()) => break,
                Err(e) => {
                    if attempts == MAX_ATTEMPTS {
                        tracing::error!(
                            workspace_id = %workspace_id,
                            employment_id = %employment_id,
                            date = %date,
                            error = %e,
                            "Failed to recalculate attendance summary after {} attempts",
                            MAX_ATTEMPTS
                        );
                    } else {
                        tracing::warn!(
                            workspace_id = %workspace_id,
                            employment_id = %employment_id,
                            date = %date,
                            error = %e,
                            attempt = attempts,
                            "Temporary failure recalculating attendance summary, retrying..."
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }
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

    async fn get_consumed_log_ids(
        &self,
        start_of_day_utc: DateTime<Utc>,
        next_day_utc: DateTime<Utc>,
    ) -> Result<std::collections::HashSet<Uuid>, AppError> {
        // Fetch adjacent time entries (2 days before to 2 days after) to check for already-paired logs
        let search_start = start_of_day_utc - Duration::days(2);
        let search_end = next_day_utc + Duration::days(2);
        let adjacent_entries = self
            .time_entry_repo
            .get_employment_time_entries(
                self.auth_token,
                self.employment_id,
                search_start,
                search_end,
            )
            .await?;

        // Filter out entries that belong to the current day (they will be replaced)
        // Extract all log IDs that were already consumed by entries of adjacent days
        let mut consumed_log_ids = std::collections::HashSet::new();
        for entry in adjacent_entries {
            let entry_local = entry.punch_in_time().with_timezone(&self.tz);
            if entry_local.date_naive() == self.date {
                continue;
            }

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
        Ok(consumed_log_ids)
    }
}

// local_to_tz is now provided by crate::utils::tz::local_to_tz

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimeEntry;
    use chrono::Utc;
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

        let service = PairingService::new(PairingServiceArgs {
            time_entry_repo: Arc::new(MockTimeEntryRepository::new()),
            roster_repo: Arc::new(MockRosterRepository::new()),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            raw_log_repo: Arc::new(mock_raw_repo),
            shift_repo: Arc::new(MockShiftRepository::new()),
            summary_service: Arc::new(MockSummaryRecalculator::new()),
        });

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

        let service = PairingService::new(PairingServiceArgs {
            time_entry_repo: Arc::new(mock_time_entry_repo),
            roster_repo: Arc::new(MockRosterRepository::new()),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            raw_log_repo: Arc::new(MockRawAttendanceLogRepository::new()),
            shift_repo: Arc::new(MockShiftRepository::new()),
            summary_service: Arc::new(MockSummaryRecalculator::new()),
        });

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

        let service = PairingService::new(PairingServiceArgs {
            time_entry_repo: Arc::new(mock_time_entry_repo),
            roster_repo: Arc::new(mock_roster_repo),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            raw_log_repo: Arc::new(mock_raw_repo),
            shift_repo: Arc::new(MockShiftRepository::new()),
            summary_service: Arc::new(mock_summary_service),
        });

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
        let adjacent_time = Utc::now() - chrono::Duration::days(1);
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
        let log_time = Utc::now();
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

        let service = PairingService::new(PairingServiceArgs {
            time_entry_repo: Arc::new(mock_time_entry_repo),
            roster_repo: Arc::new(mock_roster_repo),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            raw_log_repo: Arc::new(mock_raw_repo),
            shift_repo: Arc::new(MockShiftRepository::new()),
            summary_service: Arc::new(mock_summary_service),
        });

        let result = service
            .reprocess_daily_attendance(
                ws_id,
                emp_id,
                NaiveDate::from_ymd_opt(2026, 6, 19).unwrap(),
                chrono_tz::UTC,
            )
            .await;

        assert!(result.is_ok(), "Reprocess failed: {:?}", result.err());
    }
}
