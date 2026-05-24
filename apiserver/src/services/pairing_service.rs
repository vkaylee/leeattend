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
use crate::services::pairing_algo::{self, PairingShiftConfig};

pub struct PairingService {
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    policy_repo: Arc<dyn AttendancePolicyRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
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
            .has_duplicate_raw_log(log.workspace_id, log.employee_id, log.id, log.punch_time)
            .await?;

        if is_duplicate {
            info!(
                "Deduplicating log {} for employee {}. Found another log with identical timestamp.",
                log.id, log.employee_id
            );
            return Ok(());
        }

        // 2. Fetch Workspace Config
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(log.workspace_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Workspace not found".into()))?;
        let timezone_str = workspace.timezone;
        let tz: Tz = crate::utils::tz::parse_workspace_tz(&timezone_str);

        // 3. Determine Local Date
        let local_dt = log.punch_time.with_timezone(&tz);
        let local_date = local_dt.date_naive();

        // 4. Reprocess that day
        self.reprocess_daily_attendance(log.workspace_id, log.employee_id, local_date, tz)
            .await?;

        // 5. Reprocess previous day if early in the morning (Overnight checkouts)
        if local_dt.hour() < 12 {
            let prev_date = local_date - Duration::days(1);
            info!(
                "Log is early in day ({}), reprocessing previous day {} for overnight checkouts",
                local_dt, prev_date
            );
            self.reprocess_daily_attendance(log.workspace_id, log.employee_id, prev_date, tz)
                .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn reprocess_daily_attendance(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError> {
        info!(
            "Reprocessing attendance for emp {} on date {}",
            employee_id, date
        );

        // 1. Fetch effective schedule
        let (shift_id_opt, traces) = self
            .roster_repo
            .get_effective_shift(&crate::diesel_pool::TenantAuthToken::system_override(workspace_id), employee_id, date)
            .await?;

        let mut shift_config = None;
        if let Some(shift_id) = shift_id_opt {
            let shift = self
                .shift_repo
                .get_shift_by_id(&crate::diesel_pool::TenantAuthToken::system_override(workspace_id), shift_id)
                .await?;

            if let Some(s) = shift {
                let start_m = s.start_time.hour() * 60 + s.start_time.minute();
                let mut end_m = s.end_time.hour() * 60 + s.end_time.minute();

                // Handle Overnight: if end < start, it's next day
                if s.end_time < s.start_time {
                    end_m += 1440;
                }

                let mut bs_m = s
                    .break_start
                    .map(|t: chrono::NaiveTime| t.hour() * 60 + t.minute());
                let mut be_m = s
                    .break_end
                    .map(|t: chrono::NaiveTime| t.hour() * 60 + t.minute());

                // Handle break overnight transition if needed
                if let (Some(bs), Some(be)) = (bs_m, be_m) {
                    if bs < start_m {
                        bs_m = Some(bs + 1440);
                    }
                    if be < bs {
                        be_m = Some(be + 1440);
                    }
                }

                shift_config = Some(PairingShiftConfig {
                    start_minutes: start_m as i32,
                    end_minutes: end_m as i32,
                    break_start_minutes: bs_m.map(|m| m as i32),
                    break_end_minutes: be_m.map(|m| m as i32),
                });
            }
        }

        // 2. Fetch raw logs
        let start_of_day_local = crate::utils::tz::local_to_tz(&tz, date, 0, 0, 0)?;
        let start_of_day_utc = start_of_day_local.with_timezone(&Utc);

        let (window_start_utc, window_end_utc) = if let Some(s) = &shift_config {
            const LOG_SEARCH_BUFFER_MINUTES: i64 = 360;
            let start_mins = s.start_minutes as i64 - LOG_SEARCH_BUFFER_MINUTES;
            // End of local day OR shift end (whichever is later) to handle:
            //   - Normal shifts: capture all overtime until midnight
            //   - Overnight shifts: capture logs past midnight (e.g. 22:00-06:00)
            let end_mins = std::cmp::max(1440, s.end_minutes as i64 + LOG_SEARCH_BUFFER_MINUTES);

            let s_utc = (start_of_day_local + Duration::minutes(start_mins)).with_timezone(&Utc);
            let e_utc = (start_of_day_local + Duration::minutes(end_mins)).with_timezone(&Utc);
            (s_utc, e_utc)
        } else {
            let s_utc = start_of_day_local.with_timezone(&Utc);
            let e_utc = s_utc + Duration::hours(24);
            (s_utc, e_utc)
        };

        let next_day_utc = (start_of_day_local + Duration::days(1)).with_timezone(&Utc);

        let logs = self
            .raw_log_repo
            .get_employee_logs_in_time_range(
                workspace_id,
                employee_id,
                window_start_utc,
                window_end_utc,
            )
            .await?;

        // 3. Filter and Linearize Logs
        let mut filtered_log_events = Vec::new();
        if !logs.is_empty() {
            let mut last_minute: Option<i64> = None;

            for log in logs {
                let current_minute = log.punch_time.timestamp() / 60;
                if last_minute == Some(current_minute) {
                    continue;
                }

                let log_local = log.punch_time.with_timezone(&tz);
                let duration_since_start = log_local.signed_duration_since(start_of_day_local);

                filtered_log_events.push(pairing_algo::LogEvent {
                    id: log.id,
                    minutes: duration_since_start.num_minutes() as i32,
                    raw_data: log.raw_data,
                });

                last_minute = Some(current_minute);
            }
        }

        if filtered_log_events.is_empty() {
            self.clear_auto_entries(employee_id, workspace_id, start_of_day_utc, next_day_utc)
                .await?;

            let summary_svc = crate::services::AttendanceSummaryService::new(
                crate::services::attendance_summary_service::AttendanceSummaryServiceArgs {
                    time_entry_repo: Arc::clone(&self.time_entry_repo),
                    roster_repo: Arc::clone(&self.roster_repo),
                    policy_repo: Arc::clone(&self.policy_repo),
                    workspace_repo: Arc::clone(&self.workspace_repo),
                    employee_repo: Arc::clone(&self.employee_repo),
                    holiday_repo: Arc::clone(&self.holiday_repo),
                    leave_request_repo: Arc::clone(&self.leave_request_repo),
                    daily_summary_repo: Arc::clone(&self.daily_summary_repo),
                },
            );
            let _ = summary_svc
                .recalculate_daily_summary(workspace_id, employee_id, date)
                .await;

            return Ok(());
        }

        // 4. Run Algorithm
        let strategy = pairing_algo::get_strategy();
        let sessions = strategy.pair(&filtered_log_events, shift_config.as_ref());

        // 5. Update Database
        let mut new_entries = Vec::new();
        for session in sessions {
            let start_dt_local = crate::utils::tz::local_to_tz(&tz, date, 0, 0, 0)?
                + Duration::minutes(session.start_minutes as i64);
            let start_dt_utc = start_dt_local.with_timezone(&Utc);

            if start_dt_local.date_naive() != date {
                continue;
            }

            let end_dt_utc = if let Some(m) = session.end_minutes {
                let end_dt_local = crate::utils::tz::local_to_tz(&tz, date, 0, 0, 0)?
                    + Duration::minutes(m as i64);
                Some(end_dt_local.with_timezone(&Utc))
            } else {
                None
            };

            let duration = end_dt_utc.map(|end| (end - start_dt_utc).num_minutes() as i32);

            let mut final_metadata = session.metadata.clone().unwrap_or(serde_json::json!({}));
            if let serde_json::Value::Object(ref mut map) = final_metadata {
                map.insert("schedule_traces".to_string(), serde_json::json!(traces));
            }

            new_entries.push(crate::models::TimeEntry {
                id: crate::utils::id::new_id(),
                workspace_id,
                employee_id,
                shift_id: shift_id_opt,
                punch_in_time: start_dt_utc,
                punch_out_time: end_dt_utc,
                duration_minutes: duration,
                is_break_overtime: session.is_break_overtime,
                entry_type: session.entry_type,
                note: session.note,
                generation_metadata: Some(final_metadata),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        self.time_entry_repo
            .replace_time_entries(&crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
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
                holiday_repo: Arc::clone(&self.holiday_repo),
                leave_request_repo: Arc::clone(&self.leave_request_repo),
                daily_summary_repo: Arc::clone(&self.daily_summary_repo),
            },
        );
        if let Err(e) = summary_svc
            .recalculate_daily_summary(workspace_id, employee_id, date)
            .await
        {
            tracing::error!(
                "Failed to recalculate attendance summary for emp {} on {}: {}",
                employee_id,
                date,
                e
            );
        }
        Ok(())
    }

    async fn clear_auto_entries(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(), AppError> {
        self.time_entry_repo
            .replace_time_entries(&crate::diesel_pool::TenantAuthToken::system_override(workspace_id), employee_id, start, end, Vec::new())
            .await
    }
}

// local_to_tz is now provided by crate::utils::tz::local_to_tz

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::holiday_repository::MockHolidayRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
    use crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository;
    use crate::repositories::roster::MockRosterRepository;
    use crate::repositories::shift_repository::MockShiftRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;
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
            holiday_repo: Arc::new(MockHolidayRepository::new()),
            leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
            daily_summary_repo: Arc::new(DummyDailySummaryRepo),
            raw_log_repo: Arc::new(mock_raw_repo),
            shift_repo: Arc::new(MockShiftRepository::new()),
        });

        let log = crate::models::RawAttendanceLog {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            punch_time: Utc::now(),
            source: crate::models::enums::LogSource::Manual,
            device_id: None,
            raw_data: Some(serde_json::json!({})),
            punch: 1,
            status: 1,
            created_at: Utc::now(),
        };

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
            .expect_get_employee_logs_in_time_range()
            .times(1)
            .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

        let mut mock_time_entry_repo = MockTimeEntryRepository::new();
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
}
