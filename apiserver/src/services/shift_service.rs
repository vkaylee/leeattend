use sqlx::PgPool;
use uuid::Uuid;
use chrono::{NaiveTime, NaiveDate};
use axum::http::StatusCode;

use crate::repositories::shift_repository;
use crate::models::Shift;

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
    ) -> Result<Shift, StatusCode> {
        shift_repository::create_shift(pool, workspace_id, name, start_time, end_time, break_duration_minutes, break_is_overtime)
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
