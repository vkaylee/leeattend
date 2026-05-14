use chrono::{Local, NaiveDate};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Shift, UpdateShiftRequest};
use crate::repositories::shift_repository::ShiftRepository;

pub struct ShiftService {
    repo: Arc<dyn ShiftRepository>,
}

impl ShiftService {
    pub fn new(repo: Arc<dyn ShiftRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn create_shift(
        &self,
        workspace_id: Uuid,
        req: &crate::models::CreateShiftRequest,
    ) -> Result<Shift, AppError> {
        self.repo
            .create_shift(
                workspace_id,
                req,
                true, // is_active
                None, // parent_shift_id
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_shifts(&self, workspace_id: Uuid) -> Result<Vec<Shift>, AppError> {
        self.repo.get_workspace_shifts(workspace_id).await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_shift(
        &self,
        workspace_id: Uuid,
        shift_id: Uuid,
        req: UpdateShiftRequest,
    ) -> Result<Shift, AppError> {
        let old_shift = self
            .repo
            .get_shift_by_id(workspace_id, shift_id)
            .await?
            .ok_or(AppError::NotFound("Shift not found".into()))?;

        let has_time_changes = old_shift.start_time != req.start_time
            || old_shift.end_time != req.end_time
            || old_shift.break_duration_minutes != req.break_duration_minutes
            || old_shift.break_start != req.break_start
            || old_shift.break_end != req.break_end
            || old_shift.allow_early_ot != req.allow_early_ot;

        if req.is_correction || !has_time_changes {
            // Direct update
            self.repo
                .update_shift_direct(workspace_id, shift_id, &req)
                .await
        } else {
            // Create new version
            let effective_date = req
                .effective_date
                .unwrap_or_else(|| Local::now().date_naive());

            // For complex versioning that involves multiple steps, we use the repo method
            // which handles its own transaction or logical grouping.
            // Note: Since we are in transition, some complex logic might still be SQLx-specific.

            // 1. Create new version & migrate assignments in one go if repo supports it,
            // or perform steps sequentially.

            // Since update_assignments_to_new_version is currently a separate method:
            let new_shift = self
                .repo
                .create_shift(
                    old_shift.workspace_id,
                    &crate::models::CreateShiftRequest {
                        name: req.name,
                        report_symbol: req.report_symbol,
                        start_time: req.start_time,
                        end_time: req.end_time,
                        break_duration_minutes: req.break_duration_minutes,
                        break_start: req.break_start,
                        break_end: req.break_end,
                        is_break_auto_deduct: req.is_break_auto_deduct,
                        break_is_overtime: req.break_is_overtime,
                        early_punch_in_grace_period_minutes: req
                            .early_punch_in_grace_period_minutes,
                        allow_early_ot: req.allow_early_ot,
                    },
                    true,
                    Some(shift_id),
                )
                .await?;

            // 2. Archive old version
            self.repo.archive_shift(workspace_id, shift_id).await?;

            // 3. Migrate assignments
            self.repo
                .update_assignments_to_new_version(shift_id, new_shift.id, effective_date)
                .await?;

            Ok(new_shift)
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn assign_shift(
        &self,
        employee_id: Uuid,
        shift_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), AppError> {
        match self
            .repo
            .assign_employee_shift(employee_id, shift_id, start_date, end_date)
            .await
        {
            Ok(_) => Ok(()),
            Err(AppError::Diesel(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                info,
            ))) if info.constraint_name() == Some("uq_employee_shift_start") => Err(
                AppError::Conflict("Shift assignment already exists for this date".into()),
            ),
            Err(e) => Err(e),
        }
    }
}
