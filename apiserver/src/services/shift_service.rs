use chrono::{Local, NaiveDate};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Shift, UpdateShiftRequest};
use crate::repositories::shift_repository;

pub struct ShiftService {
    pool: PgPool,
}

impl ShiftService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn create_shift(
        &self,
        workspace_id: Uuid,
        req: &crate::models::CreateShiftRequest,
    ) -> Result<Shift, AppError> {
        shift_repository::create_shift(
            &self.pool,
            workspace_id,
            req,
            true, // is_active
            None, // parent_shift_id
        )
        .await
        .map_err(AppError::Sqlx)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_shifts(&self, workspace_id: Uuid) -> Result<Vec<Shift>, AppError> {
        shift_repository::get_workspace_shifts(&self.pool, workspace_id)
            .await
            .map_err(AppError::Sqlx)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_shift(
        &self,
        workspace_id: Uuid,
        shift_id: Uuid,
        req: UpdateShiftRequest,
    ) -> Result<Shift, AppError> {
        let pool = &self.pool;
        let old_shift = shift_repository::get_shift_by_id(pool, workspace_id, shift_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Shift not found".into()))?;

        let has_time_changes = old_shift.start_time != req.start_time
            || old_shift.end_time != req.end_time
            || old_shift.break_duration_minutes != req.break_duration_minutes
            || old_shift.break_start != req.break_start
            || old_shift.break_end != req.break_end
            || old_shift.allow_early_ot != req.allow_early_ot;

        if req.is_correction || !has_time_changes {
            // Direct update
            shift_repository::update_shift_direct(pool, workspace_id, shift_id, &req)
                .await
                .map_err(AppError::Sqlx)
        } else {
            // Create new version
            let effective_date = req
                .effective_date
                .unwrap_or_else(|| Local::now().date_naive());

            let mut tx = pool.begin().await.map_err(AppError::Sqlx)?;

            // 1. Archive old version
            shift_repository::archive_shift(&mut *tx, workspace_id, shift_id)
                .await
                .map_err(AppError::Sqlx)?;

            // 2. Create new version
            let new_shift = shift_repository::create_shift(
                &mut *tx,
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
                    early_punch_in_grace_period_minutes: req.early_punch_in_grace_period_minutes,
                    allow_early_ot: req.allow_early_ot,
                },
                true,
                Some(shift_id),
            )
            .await
            .map_err(AppError::Sqlx)?;

            // 3. Migrate assignments
            shift_repository::update_assignments_to_new_version_tx(
                &mut tx,
                shift_id,
                new_shift.id,
                effective_date,
            )
            .await
            .map_err(AppError::Sqlx)?;

            tx.commit().await.map_err(AppError::Sqlx)?;

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
        match shift_repository::assign_employee_shift(
            &self.pool,
            employee_id,
            shift_id,
            start_date,
            end_date,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(e)) if e.constraint() == Some("uq_employee_shift_start") => {
                Err(AppError::Conflict(
                    "Shift assignment already exists for this date".into(),
                ))
            }
            Err(e) => Err(AppError::Sqlx(e)),
        }
    }
}
