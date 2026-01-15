use axum::http::StatusCode;
use chrono::{NaiveDate, NaiveTime, Local};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Shift, UpdateShiftRequest};
use crate::repositories::shift_repository;

pub struct ShiftService;

impl ShiftService {
    #[allow(clippy::too_many_arguments)]
    pub async fn create_shift(
        pool: &PgPool,
        workspace_id: Uuid,
        name: String,
        start_time: NaiveTime,
        end_time: NaiveTime,
        break_duration_minutes: i32,
        break_start: Option<NaiveTime>,
        break_end: Option<NaiveTime>,
        is_break_auto_deduct: bool,
        break_is_overtime: bool,
        early_punch_in_grace_period_minutes: i32,
        allow_early_ot: bool,
    ) -> Result<Shift, StatusCode> {
        shift_repository::create_shift(
            pool,
            workspace_id,
            name,
            start_time,
            end_time,
            break_duration_minutes,
            break_start,
            break_end,
            is_break_auto_deduct,
            break_is_overtime,
            early_punch_in_grace_period_minutes,
            allow_early_ot,
            true, // is_active
            None, // parent_shift_id
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get_workspace_shifts(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<Shift>, StatusCode> {
        shift_repository::get_workspace_shifts(pool, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn update_shift(
        pool: &PgPool,
        shift_id: Uuid,
        req: UpdateShiftRequest,
    ) -> Result<Shift, StatusCode> {
        let old_shift = shift_repository::get_shift_by_id(pool, shift_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let has_time_changes = old_shift.start_time != req.start_time 
            || old_shift.end_time != req.end_time
            || old_shift.break_duration_minutes != req.break_duration_minutes
            || old_shift.break_start != req.break_start
            || old_shift.break_end != req.break_end
            || old_shift.allow_early_ot != req.allow_early_ot;

        if req.is_correction || !has_time_changes {
            // Direct update
            shift_repository::update_shift_direct(
                pool,
                shift_id,
                req.name,
                req.start_time,
                req.end_time,
                req.break_duration_minutes,
                req.break_start,
                req.break_end,
                req.is_break_auto_deduct,
                req.break_is_overtime,
                req.early_punch_in_grace_period_minutes,
                req.allow_early_ot,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        } else {
            // Create new version
            let effective_date = req.effective_date.unwrap_or_else(|| Local::now().date_naive());
            
            let mut tx = pool.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // 1. Archive old version
            shift_repository::archive_shift(&mut *tx, shift_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // 2. Create new version
            let new_shift = shift_repository::create_shift(
                &mut *tx,
                old_shift.workspace_id,
                req.name,
                req.start_time,
                req.end_time,
                req.break_duration_minutes,
                req.break_start,
                req.break_end,
                req.is_break_auto_deduct,
                req.break_is_overtime,
                req.early_punch_in_grace_period_minutes,
                req.allow_early_ot,
                true,
                Some(shift_id),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // 3. Migrate assignments
            shift_repository::update_assignments_to_new_version_tx(
                &mut tx,
                shift_id,
                new_shift.id,
                effective_date,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            Ok(new_shift)
        }
    }

    pub async fn assign_shift(
        pool: &PgPool,
        employee_id: Uuid,
        shift_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), StatusCode> {
        match shift_repository::assign_employee_shift(pool, employee_id, shift_id, start_date, end_date)
            .await {
                Ok(_) => Ok(()),
                Err(sqlx::Error::Database(e)) if e.constraint() == Some("uq_employee_shift_start") => {
                    Err(StatusCode::CONFLICT)
                },
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
    }
}
