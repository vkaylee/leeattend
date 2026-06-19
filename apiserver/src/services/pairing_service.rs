use crate::errors::AppError;
use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use tracing::info;
use uuid::Uuid;

use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::repositories::shift_repository::ShiftRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;

use crate::models::RawAttendanceLog;
use crate::repositories::roster::RosterRepository;
use crate::repositories::AttendancePolicyRepository;
use crate::repositories::TimeEntryRepository;
use crate::services::pairing_algo;

pub struct PairingService {
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    policy_repo: Arc<dyn AttendancePolicyRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
    employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
    holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepository>,
    leave_request_repo:
        Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    daily_summary_repo: Arc<dyn crate::repositories::AttendanceDailySummaryRepository>,
    raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    shift_repo: Arc<dyn ShiftRepository>,
}

pub struct PairingServiceArgs {
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub policy_repo: Arc<dyn AttendancePolicyRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
    pub employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
    pub holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepository>,
    pub leave_request_repo:
        Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    pub daily_summary_repo: Arc<dyn crate::repositories::AttendanceDailySummaryRepository>,
    pub raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    pub shift_repo: Arc<dyn ShiftRepository>,
}

impl PairingService {
    pub fn new(args: PairingServiceArgs) -> Self {
        Self {
            time_entry_repo: args.time_entry_repo,
            roster_repo: args.roster_repo,
            policy_repo: args.policy_repo,
            workspace_repo: args.workspace_repo,
            employee_repo: args.employee_repo,
            employment_repo: args.employment_repo,
            holiday_repo: args.holiday_repo,
            leave_request_repo: args.leave_request_repo,
            daily_summary_repo: args.daily_summary_repo,
            raw_log_repo: args.raw_log_repo,
            shift_repo: args.shift_repo,
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
                "Deduplicating log {} for employment {}. Found another log with identical timestamp.",
                log.id(),
                log.employment_id()
            );
            return Ok(());
        }

        // 2. Fetch Timezone (Two-step resolution)
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(log.workspace_id());

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
                "Log is early in day ({}), reprocessing previous day {} for overnight checkouts",
                local_dt, prev_date
            );
            self.reprocess_daily_attendance(log.workspace_id(), log.employment_id(), prev_date, tz)
                .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn reprocess_daily_attendance(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError> {
        info!(
            "Reprocessing attendance for emp {} on date {}",
            employment_id, date
        );

        // 1. Fetch effective schedule
        let (shift_id_opt, traces) = self
            .roster_repo
            .get_effective_shift(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
                date,
            )
            .await?;

        let context = crate::services::pairing_context::PairingContext::new(date, tz);

        let shift = if let Some(shift_id) = shift_id_opt {
            self.shift_repo
                .get_shift_by_id(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    shift_id,
                )
                .await?
        } else {
            None
        };

        let shift_config = context.build_shift_config(shift.as_ref());

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

        // Fetch adjacent time entries (2 days before to 2 days after) to check for already-paired logs
        let search_start = start_of_day_utc - Duration::days(2);
        let search_end = next_day_utc + Duration::days(2);
        let adjacent_entries = self
            .time_entry_repo
            .get_employment_time_entries(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
                search_start,
                search_end,
            )
            .await?;

        // Filter out entries that belong to the current day (they will be replaced)
        // Extract all log IDs that were already consumed by entries of adjacent days
        let mut consumed_log_ids = std::collections::HashSet::new();
        for entry in adjacent_entries {
            let entry_local = entry.punch_in_time().with_timezone(&tz);
            if entry_local.date_naive() == date {
                continue;
            }

            if let Some(metadata) = entry.generation_metadata() {
                if let Some(log_ids_val) = metadata.get("log_ids") {
                    if let Some(arr) = log_ids_val.as_array() {
                        for val in arr {
                            if let Some(id_str) = val.as_str() {
                                if let Ok(id) = uuid::Uuid::parse_str(id_str) {
                                    consumed_log_ids.insert(id);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Exclude consumed raw logs
        let filtered_raw_logs: Vec<RawAttendanceLog> = logs
            .into_iter()
            .filter(|log| !consumed_log_ids.contains(&log.id()))
            .collect();

        // 3. Filter and Linearize Logs
        let filtered_log_events = context.build_log_events(&filtered_raw_logs)?;

        if filtered_log_events.is_empty() {
            self.clear_auto_entries(employment_id, workspace_id, start_of_day_utc, next_day_utc)
                .await?;

            let summary_svc = crate::services::AttendanceSummaryService::new(
                crate::services::attendance_summary_service::AttendanceSummaryServiceArgs {
                    time_entry_repo: Arc::clone(&self.time_entry_repo),
                    roster_repo: Arc::clone(&self.roster_repo),
                    policy_repo: Arc::clone(&self.policy_repo),
                    workspace_repo: Arc::clone(&self.workspace_repo),
                    employee_repo: Arc::clone(&self.employee_repo),
                    employment_repo: Arc::clone(&self.employment_repo),
                    holiday_repo: Arc::clone(&self.holiday_repo),
                    leave_request_repo: Arc::clone(&self.leave_request_repo),
                    daily_summary_repo: Arc::clone(&self.daily_summary_repo),
                },
            );
            let _ = summary_svc
                .recalculate_daily_summary(workspace_id, employment_id, date)
                .await;

            return Ok(());
        }

        // 4. Run Algorithm
        let strategy = pairing_algo::get_strategy();
        let sessions = strategy.pair(&filtered_log_events, shift_config.as_ref());

        // 5. Update Database
        let new_entries = context.build_time_entries(
            workspace_id,
            employment_id,
            shift_id_opt,
            sessions,
            &serde_json::json!(traces),
        )?;

        self.time_entry_repo
            .replace_time_entries(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
                start_of_day_utc,
                next_day_utc,
                new_entries,
            )
            .await?;

        // Recalculate daily summary for OT workflow
        let summary_svc = crate::services::AttendanceSummaryService::new(
            crate::services::attendance_summary_service::AttendanceSummaryServiceArgs {
                time_entry_repo: Arc::clone(&self.time_entry_repo),
                roster_repo: Arc::clone(&self.roster_repo),
                policy_repo: Arc::clone(&self.policy_repo),
                workspace_repo: Arc::clone(&self.workspace_repo),
                employee_repo: Arc::clone(&self.employee_repo),
                employment_repo: Arc::clone(&self.employment_repo),
                holiday_repo: Arc::clone(&self.holiday_repo),
                leave_request_repo: Arc::clone(&self.leave_request_repo),
                daily_summary_repo: Arc::clone(&self.daily_summary_repo),
            },
        );
        if let Err(e) = summary_svc
            .recalculate_daily_summary(workspace_id, employment_id, date)
            .await
        {
            tracing::error!(
                "Failed to recalculate attendance summary for emp {} on {}: {}",
                employment_id,
                date,
                e
            );
        }
        Ok(())
    }

    async fn clear_auto_entries(
        &self,
        employment_id: Uuid,
        workspace_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(), AppError> {
        self.time_entry_repo
            .replace_time_entries(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
                start,
                end,
                Vec::new(),
            )
            .await
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

    use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employment_repository::MockEmploymentRepository;
    use crate::repositories::holiday_repository::MockHolidayRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
    use crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository;
    use crate::repositories::roster::MockRosterRepository;
    use crate::repositories::shift_repository::MockShiftRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;

    fn mock_employment_repo() -> MockEmploymentRepository {
        let mut repo = MockEmploymentRepository::new();
        repo.expect_get_active_employment_on_date()
            .returning(|_, _, _| Ok(None));
        repo.expect_get_employment_by_id()
            .returning(|_, employment_id| {
                Ok(Some(crate::models::employment::Employment::new(
                    employment_id,
                    Uuid::new_v4(),
                    employment_id,
                    Uuid::new_v4(),
                    None,
                    None,
                    "active".to_string(),
                    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    None,
                    None,
                    "full_time".to_string(),
                    None,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                )))
            });
        repo
    }

    // We didn't add automock to AttendanceDailySummaryRepository yet, let's use a dummy struct for it.
    struct DummyDailySummaryRepo;
    #[async_trait::async_trait]
    impl crate::repositories::AttendanceDailySummaryRepository for DummyDailySummaryRepo {
        async fn get_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<crate::models::AttendanceDailySummary>, AppError> {
            Ok(None)
        }
        async fn get_summary_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<Option<crate::models::AttendanceDailySummary>, AppError> {
            Ok(None)
        }
        async fn upsert_calculated_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
            _a: i32,
            _s: i32,
            _n: crate::models::OtStatus,
            _h: bool,
            _hn: Option<String>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn claim_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
            _c: i32,
            _r: Option<String>,
        ) -> Result<usize, AppError> {
            Ok(0)
        }
        async fn approve_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
            _a: i32,
            _m: Option<String>,
            _s: crate::models::OtStatus,
        ) -> Result<usize, AppError> {
            Ok(0)
        }
        async fn upsert_paper_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
            _c: i32,
            _r: String,
            _u: Uuid,
            _p: Option<String>,
        ) -> Result<crate::models::AttendanceDailySummary, AppError> {
            Err(AppError::Internal("Not implemented".into()))
        }
        async fn get_batch_ot_statuses(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), String>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn get_summaries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Option<Uuid>,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<crate::models::AttendanceDailySummary>, AppError> {
            Ok(vec![])
        }
        async fn get_distinct_work_days_from_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<NaiveDate>, AppError> {
            Ok(vec![])
        }
    }

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
            policy_repo: Arc::new(MockAttendancePolicyRepository::new()),
            workspace_repo: Arc::new(MockWorkspaceRepository::new()),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(MockHolidayRepository::new()),
            leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
            daily_summary_repo: Arc::new(DummyDailySummaryRepo),
            raw_log_repo: Arc::new(mock_raw_repo),
            shift_repo: Arc::new(MockShiftRepository::new()),
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
            policy_repo: Arc::new(MockAttendancePolicyRepository::new()),
            workspace_repo: Arc::new(MockWorkspaceRepository::new()),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(MockHolidayRepository::new()),
            leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
            daily_summary_repo: Arc::new(DummyDailySummaryRepo),
            raw_log_repo: Arc::new(MockRawAttendanceLogRepository::new()),
            shift_repo: Arc::new(MockShiftRepository::new()),
        });

        let result = service
            .clear_auto_entries(Uuid::new_v4(), Uuid::new_v4(), Utc::now(), Utc::now())
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

        let mut mock_workspace_repo = MockWorkspaceRepository::new();
        mock_workspace_repo
            .expect_get_workspace_by_id()
            .times(1) // Called by AttendanceSummaryService::recalculate_daily_summary
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = PairingService::new(PairingServiceArgs {
            time_entry_repo: Arc::new(mock_time_entry_repo),
            roster_repo: Arc::new(mock_roster_repo),
            policy_repo: Arc::new(MockAttendancePolicyRepository::new()),
            workspace_repo: Arc::new(mock_workspace_repo),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(MockHolidayRepository::new()),
            leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
            daily_summary_repo: Arc::new(DummyDailySummaryRepo),
            raw_log_repo: Arc::new(mock_raw_repo),
            shift_repo: Arc::new(MockShiftRepository::new()),
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

        let mut mock_workspace_repo = MockWorkspaceRepository::new();
        mock_workspace_repo
            .expect_get_workspace_by_id()
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = PairingService::new(PairingServiceArgs {
            time_entry_repo: Arc::new(mock_time_entry_repo),
            roster_repo: Arc::new(mock_roster_repo),
            policy_repo: Arc::new(MockAttendancePolicyRepository::new()),
            workspace_repo: Arc::new(mock_workspace_repo),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(MockHolidayRepository::new()),
            leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
            daily_summary_repo: Arc::new(DummyDailySummaryRepo),
            raw_log_repo: Arc::new(mock_raw_repo),
            shift_repo: Arc::new(MockShiftRepository::new()),
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
