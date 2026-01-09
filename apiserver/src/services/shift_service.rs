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
        effective_date: NaiveDate,
    ) -> Result<(), StatusCode> {
        shift_repository::assign_employee_shift(pool, employee_id, shift_id, effective_date)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
