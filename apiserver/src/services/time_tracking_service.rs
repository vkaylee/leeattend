use std::sync::Arc;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{TimeEntry, UpdateTimeEntryRequest};
use crate::utils::i18n::I18nKey;

use crate::repositories::roster::RosterRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::AttendancePolicyRepository;

use crate::repositories::TimeEntryRepository;

pub struct TimeTrackingService {
    repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    policy_repo: Arc<dyn AttendancePolicyRepository>,
    condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
    employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
    employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepository>,
    leave_request_repo: Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    daily_summary_repo: Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
}

impl TimeTrackingService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repo: Arc<dyn TimeEntryRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        policy_repo: Arc<dyn AttendancePolicyRepository>,
        condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
        employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
        employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
        workspace_repo: Arc<dyn WorkspaceRepository>,
        holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepository>,
        leave_request_repo: Arc<
            dyn crate::repositories::leave_request_repository::LeaveRequestRepository,
        >,
        daily_summary_repo: Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
    ) -> Self {
        Self {
            repo,
            roster_repo,
            policy_repo,
            condition_type_repo,
            employee_repo,
            employment_repo,
            workspace_repo,
            holiday_repo,
            leave_request_repo,
            daily_summary_repo,
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn punch(
        &self,
        log_service: &crate::services::AttendanceLogService,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        note: Option<String>,
    ) -> Result<TimeEntry, AppError> {
        // Verify employment belongs to workspace
        let employment = self
            .employment_repo
            .get_employment_by_id(
                auth_token,
                employment_id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;

        if employment.status() != "active" {
            return Err(AppError::forbidden(I18nKey::EmploymentInactive));
        }

        let now = Utc::now();

        // Check current active entry to decide if this is conceptually IN or OUT
        // We do not reject based on this, we just use it for the raw log `punch` property
        // for backward compatibility with systems expecting 0=IN, 1=OUT
        let is_in = self
            .repo
            .get_latest_active_entry(
                auth_token,
                employment_id,
            )
            .await?
            .is_none();

        if is_in {
            // Get Workspace Timezone
            let workspace = self
                .workspace_repo
                .get_workspace_by_id(auth_token.workspace_id())
                .await?
                .ok_or(AppError::Internal("error.workspace_not_found".into()))?;

            let tz: chrono_tz::Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());
            let local_now = now.with_timezone(&tz);
            let today = local_now.date_naive();

            // 1. Identify effective shift
            let (shift_id, _) = self
                .roster_repo
                .get_effective_shift(
                    auth_token,
                    employment_id,
                    today,
                )
                .await?;

            if let Some(sid) = shift_id {
                let roster_data = self.roster_repo.get_raw_roster_data(auth_token.workspace_id()).await?;
                let interval = roster_data
                    .shift_intervals
                    .get(&sid)
                    .and_then(|intervals| intervals.first())
                    .ok_or(AppError::Internal("error.shift_not_found".into()))?;

                let start_time = interval.start_time;
                let grace_minutes =
                    u32::try_from(interval.early_punch_in_grace_period_minutes).unwrap_or(0);

                let shift_start = today.and_time(start_time);
                let earliest_punch =
                    shift_start - chrono::Duration::minutes(i64::from(grace_minutes));

                let rules = crate::services::PolicyResolverService::resolve_attendance_rules(
                    &*self.policy_repo,
                    &*self.employee_repo,
                    &*self.condition_type_repo,
                    employment_id,
                    auth_token,
                    today,
                )
                .await?;

                if rules.early_arrival_strategy() == crate::models::EarlyArrivalStrategy::Block
                    && local_now.naive_local() < earliest_punch
                {
                    return Err(AppError::validation(I18nKey::TimeTrackingTooEarly));
                }
            }
        }

        let raw_data = serde_json::json!({
            "punch": i32::from(!is_in),
            "status": 1, // 1 = Normal
            "note": note,
            "source": "direct_punch"
        });

        log_service
            .process_parsed_log(
                auth_token.workspace_id(),
                employment_id,
                now,
                crate::models::LogSource::Device,
                Some("ESS_WEB".to_string()),
                Some(raw_data),
            )
            .await?;

        let entry = self
            .repo
            .get_latest_entry_for_employment(
                auth_token,
                employment_id,
            )
            .await?
            .ok_or(AppError::Internal(
                "error.time_tracking_failed_fetch_punched".into(),
            ))?;

        Ok(entry)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>, AppError> {
        // Verify employment belongs to workspace
        let _ = self
            .employment_repo
            .get_employment_by_id(
                auth_token,
                employment_id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;

        self.repo
            .get_employment_time_entries(
                auth_token,
                employment_id,
                start_date,
                end_date,
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_time_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: UpdateTimeEntryRequest,
    ) -> Result<TimeEntry, AppError> {
        let mut entry = self
            .repo
            .get_time_entry_by_id(
                auth_token,
                id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::TimeEntryNotFound))?;

        let _ = self
            .employment_repo
            .get_employment_by_id(
                auth_token,
                entry.employment_id(),
            )
            .await?
            .ok_or_else(|| AppError::forbidden(I18nKey::TimeTrackingEditOtherWorkspace))?;

        if let Some(t) = req.punch_in_time() {
            entry.set_punch_in_time(t);
        }
        if let Some(t) = req.punch_out_time() {
            entry.set_punch_out_time(Some(t));
        }
        if let Some(et) = req.entry_type() {
            entry.set_entry_type(et.to_string());
        }
        if let Some(n) = req.note() {
            entry.set_note(Some(n.to_string()));
        }

        self.repo
            .update_time_entry(
                auth_token,
                &entry,
            )
            .await?;

        // Recalculate summary
        if let Ok(Some(ws)) = self.workspace_repo.get_workspace_by_id(auth_token.workspace_id()).await {
            let tz: Tz = crate::utils::tz::parse_workspace_tz(ws.timezone());
            let local_date = entry.punch_in_time().with_timezone(&tz).date_naive();
            let summary_svc = crate::services::AttendanceSummaryService::new(
                crate::services::attendance_summary_service::AttendanceSummaryServiceArgs {
                    time_entry_repo: Arc::clone(&self.repo),
                    roster_repo: Arc::clone(&self.roster_repo),
                    policy_repo: Arc::clone(&self.policy_repo),
                    condition_type_repo: Arc::clone(&self.condition_type_repo),
                    workspace_repo: Arc::clone(&self.workspace_repo),
                    employee_repo: Arc::clone(&self.employee_repo),
                    employment_repo: Arc::clone(&self.employment_repo),
                    holiday_repo: Arc::clone(&self.holiday_repo),
                    leave_request_repo: Arc::clone(&self.leave_request_repo),
                    daily_summary_repo: Arc::clone(&self.daily_summary_repo),
                },
            );
            let _ = summary_svc
                .recalculate_daily_summary(auth_token, entry.employment_id(), local_date)
                .await;
        }

        Ok(entry)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_time_entry(&self, auth_token: &crate::diesel_pool::TenantAuthToken, id: Uuid) -> Result<(), AppError> {
        let entry = self
            .repo
            .get_time_entry_by_id(
                auth_token,
                id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::TimeEntryNotFound))?;

        // Verify employment of this entry belongs to the workspace
        let _ = self
            .employment_repo
            .get_employment_by_id(
                auth_token,
                entry.employment_id(),
            )
            .await?
            .ok_or_else(|| AppError::forbidden(I18nKey::TimeTrackingEditOtherWorkspace))?;

        self.repo
            .delete_time_entry(
                auth_token,
                id,
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::TenantAuthToken;
    use crate::models::{Employment, TimeEntry, UpdateTimeEntryRequest};
    use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
    use crate::repositories::attendance_log_error_repository::MockAttendanceLogErrorRepository;
    use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;
    use crate::repositories::condition_type_repository::MockConditionTypeRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employment_repository::MockEmploymentRepository;
    use crate::repositories::holiday_repository::MockHolidayRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
    use crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository;
    use crate::repositories::roster::MockRosterRepository;
    use crate::repositories::staging_log_repository::MockStagingLogRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;
    use crate::services::{AttendanceLogService, AttendanceLogServiceDeps};
    use chrono::{NaiveDate, TimeZone, Utc};
    use uuid::Uuid;

    fn make_token(ws: Uuid) -> TenantAuthToken {
        TenantAuthToken::system_override(ws)
    }

    fn make_employment(ws: Uuid, status: &str) -> Employment {
        Employment::new(
            Uuid::new_v4(), ws, Uuid::new_v4(), Uuid::new_v4(),
            None, None, status.into(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            None, Some("Dev".into()), "full_time".into(), None,
            Utc::now(), Utc::now(), None, None,
        )
    }

    fn make_time_entry(ws: Uuid, emp_id: Uuid) -> TimeEntry {
        TimeEntry::new(
            Uuid::new_v4(), emp_id, ws, None, None,
            Utc.with_ymd_and_hms(2026, 7, 24, 8, 0, 0).unwrap(),
            None, None, false, "regular".into(), None, None,
            Utc::now(), Utc::now(),
        )
    }

    fn build_service(
        repo: MockTimeEntryRepository,
        employment_repo: MockEmploymentRepository,
        workspace_repo: MockWorkspaceRepository,
    ) -> TimeTrackingService {
        TimeTrackingService::new(
            Arc::new(repo),
            Arc::new(MockRosterRepository::new()),
            Arc::new(MockAttendancePolicyRepository::new()),
            Arc::new(MockConditionTypeRepository::new()),
            Arc::new(MockEmployeeRepository::new()),
            Arc::new(employment_repo),
            Arc::new(workspace_repo),
            Arc::new(MockHolidayRepository::new()),
            Arc::new(MockLeaveRequestRepository::new()),
            Arc::new(MockAttendanceDailySummaryRepository::new()),
        )
    }

    fn make_log_service() -> AttendanceLogService {
        AttendanceLogService::new(
            AttendanceLogServiceDeps {
                employee_repo: Arc::new(MockEmployeeRepository::new()),
                employment_repo: Arc::new(MockEmploymentRepository::new()),
                raw_log_repo: Arc::new(MockRawAttendanceLogRepository::new()),
                staging_repo: Arc::new(MockStagingLogRepository::new()),
                error_repo: Arc::new(MockAttendanceLogErrorRepository::new()),
                pairing_service: None,
            },
            None,
        )
    }

    // ── get_employee_history ──────────────────────────────────────────

    #[tokio::test]
    async fn get_employee_history_employment_not_found() {
        let ws = Uuid::new_v4();
        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo.expect_get_employment_by_id().returning(|_, _| Ok(None));
        let svc = build_service(MockTimeEntryRepository::new(), employment_repo, MockWorkspaceRepository::new());
        let result = svc.get_employee_history(&make_token(ws), Uuid::new_v4(), Utc::now(), Utc::now()).await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn get_employee_history_success() {
        let ws = Uuid::new_v4();
        let emp = make_employment(ws, "active");
        let emp_id = emp.id();
        let entry = make_time_entry(ws, emp_id);

        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo.expect_get_employment_by_id().returning(move |_, _| Ok(Some(emp.clone())));

        let mut repo = MockTimeEntryRepository::new();
        repo.expect_get_employment_time_entries().returning(move |_, _, _, _| Ok(vec![entry.clone()]));

        let svc = build_service(repo, employment_repo, MockWorkspaceRepository::new());
        let result = svc.get_employee_history(&make_token(ws), emp_id, Utc::now(), Utc::now()).await;
        assert_eq!(result.unwrap().len(), 1);
    }

    // ── update_time_entry ─────────────────────────────────────────────

    #[tokio::test]
    async fn update_time_entry_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockTimeEntryRepository::new();
        repo.expect_get_time_entry_by_id().returning(|_, _| Ok(None));
        let svc = build_service(repo, MockEmploymentRepository::new(), MockWorkspaceRepository::new());
        let result = svc.update_time_entry(&make_token(ws), Uuid::new_v4(), UpdateTimeEntryRequest::new(None, None, None, None, None)).await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn update_time_entry_cross_workspace_forbidden() {
        let ws = Uuid::new_v4();
        let entry = make_time_entry(ws, Uuid::new_v4());
        let entry_id = entry.id();

        let mut repo = MockTimeEntryRepository::new();
        repo.expect_get_time_entry_by_id().returning(move |_, _| Ok(Some(entry.clone())));

        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo.expect_get_employment_by_id().returning(|_, _| Ok(None));

        let svc = build_service(repo, employment_repo, MockWorkspaceRepository::new());
        let result = svc.update_time_entry(&make_token(ws), entry_id, UpdateTimeEntryRequest::new(None, None, None, None, None)).await;
        assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn update_time_entry_success() {
        let ws = Uuid::new_v4();
        let emp = make_employment(ws, "active");
        let emp_id = emp.id();
        let entry = make_time_entry(ws, emp_id);
        let entry_id = entry.id();

        let mut repo = MockTimeEntryRepository::new();
        repo.expect_get_time_entry_by_id().returning(move |_, _| Ok(Some(entry.clone())));
        repo.expect_update_time_entry().returning(|_, _| Ok(()));

        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo.expect_get_employment_by_id().returning(move |_, _| Ok(Some(emp.clone())));

        // workspace_repo returns Ok(None) to skip daily-summary recalculation
        let mut workspace_repo = MockWorkspaceRepository::new();
        workspace_repo.expect_get_workspace_by_id().returning(|_| Box::pin(async { Ok(None) }));

        let svc = build_service(repo, employment_repo, workspace_repo);
        let new_note = Some("note".into());
        let new_type = Some("overtime".into());
        let result = svc.update_time_entry(&make_token(ws), entry_id,
            UpdateTimeEntryRequest::new(None, None, None, new_type.clone(), new_note.clone())).await;
        let updated = result.unwrap();
        assert_eq!(updated.note().as_deref(), new_note.as_deref());
        assert_eq!(updated.entry_type(), "overtime");
    }

    // ── delete_time_entry ─────────────────────────────────────────────

    #[tokio::test]
    async fn delete_time_entry_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockTimeEntryRepository::new();
        repo.expect_get_time_entry_by_id().returning(|_, _| Ok(None));
        let svc = build_service(repo, MockEmploymentRepository::new(), MockWorkspaceRepository::new());
        let result = svc.delete_time_entry(&make_token(ws), Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn delete_time_entry_cross_workspace_forbidden() {
        let ws = Uuid::new_v4();
        let entry = make_time_entry(ws, Uuid::new_v4());
        let entry_id = entry.id();

        let mut repo = MockTimeEntryRepository::new();
        repo.expect_get_time_entry_by_id().returning(move |_, _| Ok(Some(entry.clone())));

        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo.expect_get_employment_by_id().returning(|_, _| Ok(None));

        let svc = build_service(repo, employment_repo, MockWorkspaceRepository::new());
        let result = svc.delete_time_entry(&make_token(ws), entry_id).await;
        assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn delete_time_entry_success() {
        let ws = Uuid::new_v4();
        let emp = make_employment(ws, "active");
        let entry = make_time_entry(ws, emp.id());
        let entry_id = entry.id();

        let mut repo = MockTimeEntryRepository::new();
        repo.expect_get_time_entry_by_id().returning(move |_, _| Ok(Some(entry.clone())));
        repo.expect_delete_time_entry().returning(|_, _| Ok(()));

        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo.expect_get_employment_by_id().returning(move |_, _| Ok(Some(emp.clone())));

        let svc = build_service(repo, employment_repo, MockWorkspaceRepository::new());
        assert!(svc.delete_time_entry(&make_token(ws), entry_id).await.is_ok());
    }

    // ── punch error paths (before log_service call) ────────────────────

    #[tokio::test]
    async fn punch_employment_not_found() {
        let ws = Uuid::new_v4();
        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo.expect_get_employment_by_id().returning(|_, _| Ok(None));
        let svc = build_service(MockTimeEntryRepository::new(), employment_repo, MockWorkspaceRepository::new());
        let result = svc.punch(&make_log_service(), &make_token(ws), Uuid::new_v4(), None).await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn punch_employment_inactive() {
        let ws = Uuid::new_v4();
        let emp = make_employment(ws, "inactive");
        let emp_id = emp.id();
        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo.expect_get_employment_by_id().returning(move |_, _| Ok(Some(emp.clone())));
        let svc = build_service(MockTimeEntryRepository::new(), employment_repo, MockWorkspaceRepository::new());
        let result = svc.punch(&make_log_service(), &make_token(ws), emp_id, None).await;
        assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
    }
}
