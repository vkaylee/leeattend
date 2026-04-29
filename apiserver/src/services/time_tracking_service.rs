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
    pub async fn punch_in(
        &self,
        workspace_id: Uuid,
        req: PunchInRequest,
    ) -> Result<TimeEntry, AppError> {
        // Verify employee belongs to workspace
        let employee = employee::get_employee_by_id(&self.pool, req.employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        if !employee.is_active {
            return Err(AppError::Forbidden("Employee is inactive".into()));
        }

        // Get Workspace Timezone
        let workspace_repo = SqlxWorkspaceRepository {
            pool: self.pool.clone(),
        };
        let workspace = workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or(AppError::Internal("Workspace not found".into()))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(&workspace.timezone);

        // Check if already punched in
        if self
            .repo
            .get_latest_active_entry(workspace_id, req.employee_id)
            .await
            .map_err(AppError::Sqlx)?
            .is_some()
        {
            return Err(AppError::Conflict("Already punched in".into()));
        }

        let now = Utc::now();
        let local_now = now.with_timezone(&tz);
        let today = local_now.date_naive();

        // 1. Identify effective shift
        let (shift_id, _) = self
            .roster_repo
            .get_effective_shift(workspace_id, req.employee_id, today)
            .await
            .map_err(AppError::Sqlx)?;

        let mut final_is_break_overtime = req.is_break_overtime.unwrap_or(false);

        if let Some(sid) = shift_id {
            let shift = shift_repository::get_shift_by_id(&self.pool, workspace_id, sid)
                .await
                .map_err(AppError::Sqlx)?
                .ok_or(AppError::Internal("Shift not found".into()))?;

            let entry_type = req.entry_type.clone().unwrap_or_else(|| "work".to_string());
            if entry_type == "work" {
                let start_time = shift.start_time;
                let grace_minutes = shift.early_punch_in_grace_period_minutes as u32;

                let shift_start = today.and_time(start_time);
                let earliest_punch = shift_start - chrono::Duration::minutes(grace_minutes as i64);

                let rules =
                    crate::domain::policy::resolver::PolicyResolver::resolve_attendance_rules(
                        &self.pool,
                        req.employee_id,
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
            } else if entry_type == "break" && req.is_break_overtime.is_none() {
                final_is_break_overtime = shift.break_is_overtime;
            }
        }

        let entry = TimeEntry {
            id: Uuid::new_v4(),
            employee_id: req.employee_id,
            workspace_id,

            shift_id,
            punch_in_time: now,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: final_is_break_overtime,
            entry_type: req.entry_type.unwrap_or_else(|| "work".to_string()),
            note: req.note,
            generation_metadata: Some(serde_json::json!({ "source": "direct_punch" })),
            created_at: now,
            updated_at: now,
        };

        self.repo
            .create_time_entry(&entry)
            .await
            .map_err(AppError::Sqlx)?;

        Ok(entry)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn punch_out(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        req: PunchOutRequest,
    ) -> Result<TimeEntry, AppError> {
        // Verify employee belongs to workspace
        let _ = employee::get_employee_by_id(&self.pool, employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        let mut entry = self
            .repo
            .get_latest_active_entry(workspace_id, employee_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::Validation("Not punched in".into()))?;

        let punch_out_time = Utc::now();
        let duration = punch_out_time.signed_duration_since(entry.punch_in_time);

        entry.punch_out_time = Some(punch_out_time);
        entry.duration_minutes = Some(duration.num_minutes() as i32);
        if let Some(note) = req.note {
            entry.note = Some(
                format!("{}\n{}", entry.note.unwrap_or_default(), note)
                    .trim()
                    .to_string(),
            );
        }
        entry.updated_at = Utc::now();

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
                self.repo.clone(),
                self.roster_repo.clone(),
                self.pool.clone(),
            );
            let _ = summary_svc
                .recalculate_daily_summary(workspace_id, employee_id, local_date)
                .await;
        }

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
                self.repo.clone(),
                self.roster_repo.clone(),
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
