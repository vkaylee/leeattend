use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ClearWorkspaceDataQuery {
    pub group: String,
}

pub async fn clear_workspace_data(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ClearWorkspaceDataQuery>,
) -> AppResult<StatusCode> {
    let mut tx = state.infra.db.api().begin().await?;

    match query.group.as_str() {
        "attendance" => {
            sqlx::query("DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM raw_attendance_logs WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM attendance_import_jobs WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
        }
        "schedules" => {
            sqlx::query("DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM employee_shifts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
        }
        "leave" => {
            sqlx::query("DELETE FROM leave_accrual_logs WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM leave_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM leave_balances WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
        }
        "payroll" => {
            sqlx::query("DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM employee_salary_components WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM salary_component_definitions WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
        }
        "full_reset" => {
            sqlx::query("DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM raw_attendance_logs WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM attendance_import_jobs WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM employee_shifts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM leave_accrual_logs WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM leave_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM leave_balances WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM employee_salary_components WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM salary_component_definitions WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM employee_department_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await?;
            sqlx::query("DELETE FROM leave_policy_assignments WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM leave_policy_definitions WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM employees WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM shifts WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM departments WHERE workspace_id = $1")
                .bind(ws_id)
                .execute(&mut *tx)
                .await?;
        }
        _ => return Err(AppError::Validation("Invalid group".to_string())),
    };

    tx.commit().await?;
    Ok(StatusCode::OK)
}
