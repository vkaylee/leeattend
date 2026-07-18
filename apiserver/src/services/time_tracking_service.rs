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
        workspace_id: Uuid,
        employment_id: Uuid,
        note: Option<String>,
    ) -> Result<TimeEntry, AppError> {
        // Verify employment belongs to workspace
        let employment = self
            .employment_repo
            .get_employment_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
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
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
            )
            .await?
            .is_none();

        if is_in {
            // Get Workspace Timezone
            let workspace = self
                .workspace_repo
                .get_workspace_by_id(workspace_id)
                .await?
                .ok_or(AppError::Internal("error.workspace_not_found".into()))?;

            let tz: chrono_tz::Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());
            let local_now = now.with_timezone(&tz);
            let today = local_now.date_naive();

            // 1. Identify effective shift
            let (shift_id, _) = self
                .roster_repo
                .get_effective_shift(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    employment_id,
                    today,
                )
                .await?;

            if let Some(sid) = shift_id {
                let roster_data = self.roster_repo.get_raw_roster_data(workspace_id).await?;
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
                    workspace_id,
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
                workspace_id,
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
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
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
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>, AppError> {
        // Verify employment belongs to workspace
        let _ = self
            .employment_repo
            .get_employment_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;

        self.repo
            .get_employment_time_entries(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
                start_date,
                end_date,
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_time_entry(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: UpdateTimeEntryRequest,
    ) -> Result<TimeEntry, AppError> {
        let mut entry = self
            .repo
            .get_time_entry_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::TimeEntryNotFound))?;

        let _ = self
            .employment_repo
            .get_employment_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
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
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                &entry,
            )
            .await?;

        // Recalculate summary
        if let Ok(Some(ws)) = self.workspace_repo.get_workspace_by_id(workspace_id).await {
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
                .recalculate_daily_summary(workspace_id, entry.employment_id(), local_date)
                .await;
        }

        Ok(entry)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_time_entry(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let entry = self
            .repo
            .get_time_entry_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::TimeEntryNotFound))?;

        // Verify employment of this entry belongs to the workspace
        let _ = self
            .employment_repo
            .get_employment_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                entry.employment_id(),
            )
            .await?
            .ok_or_else(|| AppError::forbidden(I18nKey::TimeTrackingEditOtherWorkspace))?;

        self.repo
            .delete_time_entry(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                id,
            )
            .await?;

        Ok(())
    }
}
