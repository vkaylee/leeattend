use axum::http::StatusCode;
use chrono::{NaiveDate, NaiveTime};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Shift;
use crate::repositories::shift_repository;

pub struct ShiftService;

impl ShiftService {
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
