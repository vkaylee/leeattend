use std::sync::Arc;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{PunchInRequest, PunchOutRequest, TimeEntry, UpdateTimeEntryRequest};
use crate::repositories::employee;

use crate::repositories::roster::RosterRepository;
use crate::repositories::shift_repository;
use crate::repositories::workspace_repository::{SqlxWorkspaceRepository, WorkspaceRepository};
use crate::repositories::TimeEntryRepository;

pub struct TimeTrackingService {
    repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    /// Kept for operations not yet abstracted
    pool: PgPool,
}

impl TimeTrackingService {
    pub fn new(
        repo: Arc<dyn TimeEntryRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        pool: PgPool,
    ) -> Self {
        Self {
            repo,
            roster_repo,
            pool,
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn punch(
        &self,
        log_service: &crate::services::AttendanceLogService,
        workspace_id: Uuid,
        employee_id: Uuid,
        note: Option<String>,
    ) -> Result<TimeEntry, AppError> {
        // Verify employee belongs to workspace
        let employee = employee::get_employee_by_id(&self.pool, employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        if !employee.is_active {
            return Err(AppError::Forbidden("Employee is inactive".into()));
        }

        let now = Utc::now();

        // Check current active entry to decide if this is conceptually IN or OUT
        // We do not reject based on this, we just use it for the raw log `punch` property
        // for backward compatibility with systems expecting 0=IN, 1=OUT
        let is_in = self
            .repo
            .get_latest_active_entry(workspace_id, employee_id)
            .await
            .map_err(AppError::Sqlx)?
            .is_none();

        if is_in {
            // Get Workspace Timezone
            let workspace_repo =
                crate::repositories::workspace_repository::SqlxWorkspaceRepository {
                    pool: self.pool.clone(),
                };
            let workspace = workspace_repo
                .get_workspace_by_id(workspace_id)
                .await
                .map_err(AppError::Internal)?
                .ok_or(AppError::Internal("Workspace not found".into()))?;

            let tz: chrono_tz::Tz = crate::utils::tz::parse_workspace_tz(&workspace.timezone);
            let local_now = now.with_timezone(&tz);
            let today = local_now.date_naive();

            // 1. Identify effective shift
            let (shift_id, _) = self
                .roster_repo
                .get_effective_shift(workspace_id, employee_id, today)
                .await
                .map_err(AppError::Sqlx)?;

            if let Some(sid) = shift_id {
                let shift = crate::repositories::shift_repository::get_shift_by_id(
                    &self.pool,
                    workspace_id,
                    sid,
                )
                .await
                .map_err(AppError::Sqlx)?
                .ok_or(AppError::Internal("Shift not found".into()))?;

                let start_time = shift.start_time;
                let grace_minutes = shift.early_punch_in_grace_period_minutes as u32;

                let shift_start = today.and_time(start_time);
                let earliest_punch = shift_start - chrono::Duration::minutes(grace_minutes as i64);

                let rules =
                    crate::domain::policy::resolver::PolicyResolver::resolve_attendance_rules(
                        &self.pool,
                        employee_id,
                        workspace_id,
                        today,
                    )
                    .await
                    .map_err(AppError::Sqlx)?;

                if rules.early_arrival_strategy == crate::models::EarlyArrivalStrategy::Block
                    && local_now.naive_local() < earliest_punch
                {
                    return Err(AppError::Validation("Too early to punch in".into()));
                }
            }
        }

        let raw_data = serde_json::json!({
            "punch": (!is_in) as i32,
            "status": 1, // 1 = Normal
            "note": note,
            "source": "direct_punch"
        });

        log_service
            .process_parsed_log(
                workspace_id,
                employee_id,
                now,
                crate::models::LogSource::Device,
                Some("ESS_WEB".to_string()),
                Some(raw_data),
            )
            .await
            .map_err(AppError::Internal)?;

        let entry = sqlx::query_as::<_, TimeEntry>(
            "SELECT * FROM time_entries WHERE employee_id = $1 AND workspace_id = $2 ORDER BY punch_in_time DESC LIMIT 1"
        )
        .bind(employee_id)
        .bind(workspace_id)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Sqlx)?;

        Ok(entry)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>, AppError> {
        // Verify employee belongs to workspace
        let _ = employee::get_employee_by_id(&self.pool, employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        self.repo
            .get_employee_time_entries(workspace_id, employee_id, start_date, end_date)
            .await
            .map_err(AppError::Sqlx)
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
            .get_time_entry_by_id(workspace_id, id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Time entry not found".into()))?;

        // Verify employee of this entry belongs to the workspace
        let _ = employee::get_employee_by_id(&self.pool, entry.employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::Forbidden(
                "Employee does not belong to workspace".into(),
            ))?;

        if let Some(t) = req.punch_in_time {
            entry.punch_in_time = t;
        }
        if let Some(t) = req.punch_out_time {
            entry.punch_out_time = Some(t);
        }
        if let Some(et) = req.entry_type {
            entry.entry_type = et;
        }
        if let Some(n) = req.note {
            entry.note = Some(n);
        }

        if let Some(po) = entry.punch_out_time {
            let duration = po.signed_duration_since(entry.punch_in_time);
            entry.duration_minutes = Some(duration.num_minutes() as i32);
        }

        self.repo
            .update_time_entry(workspace_id, &entry)
            .await
            .map_err(AppError::Sqlx)?;

        // Recalculate summary
        let workspace_repo = SqlxWorkspaceRepository {
            pool: self.pool.clone(),
        };
        if let Ok(Some(ws)) = workspace_repo.get_workspace_by_id(workspace_id).await {
            let tz: Tz = crate::utils::tz::parse_workspace_tz(&ws.timezone);
            let local_date = entry.punch_in_time.with_timezone(&tz).date_naive();
            let summary_svc = crate::services::AttendanceSummaryService::new(
                Arc::clone(&self.repo),
                Arc::clone(&self.roster_repo),
                self.pool.clone(),
            );
            let _ = summary_svc
                .recalculate_daily_summary(workspace_id, entry.employee_id, local_date)
                .await;
        }

        Ok(entry)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_time_entry(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let entry = self
            .repo
            .get_time_entry_by_id(workspace_id, id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Time entry not found".into()))?;

        // Verify employee of this entry belongs to the workspace
        let _ = employee::get_employee_by_id(&self.pool, entry.employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::Forbidden(
                "Employee does not belong to workspace".into(),
            ))?;

        self.repo
            .delete_time_entry(workspace_id, id)
            .await
            .map_err(AppError::Sqlx)?;

        Ok(())
    }
}
