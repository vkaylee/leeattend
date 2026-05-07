use crate::errors::{AppError, AppResult};
use crate::AppState;
#[allow(unused_imports)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
pub struct DebugUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct ClearDbQuery {
    pub group: String,
}

pub async fn get_debug_users(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<DebugUser>>> {
    #[cfg(debug_assertions)]
    {
        let users = sqlx::query_as::<_, DebugUser>(
            r#"SELECT id, COALESCE(full_name, 'Unknown') as name, email, role FROM users LIMIT 20"#,
        )
        .fetch_all(state.infra.db.api())
        .await?;

        Ok(Json(users))
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = state; // Suppress unused warning in release mode
        Err(AppError::NotFound("Not found".to_string()))
    }
}

#[allow(clippy::too_many_lines)]
pub async fn clear_debug_db(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ClearDbQuery>,
) -> AppResult<impl IntoResponse> {
    #[cfg(debug_assertions)]
    {
        let mut tx = state.infra.db.api().begin().await?;

        match payload.group.as_str() {
            "attendance" => {
                let queries = [
                    "DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM raw_attendance_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM attendance_import_jobs WHERE workspace_id = $1",
                ];
                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await?;
                }
            }
            "payroll" => {
                let queries = [
                    "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1))",
                    "DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                ];
                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await?;
                }
            }
            "rosters" => {
                let queries = [
                    "DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                ];
                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await?;
                }
            }
            "employees" => {
                let queries = [
                    "DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM raw_attendance_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_shifts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1))",
                    "DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM leave_accrual_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM leave_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM leave_balances WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_salary_components WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_payroll_configs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_working_patterns WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_emergency_contacts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_documents WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_profiles WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_department_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employment_periods WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employees WHERE workspace_id = $1",
                ];
                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await?;
                }
            }
            "full_reset" => {
                let queries = [
                    "DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM raw_attendance_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM attendance_import_jobs WHERE workspace_id = $1",
                    "DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_shifts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_working_patterns WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1))",
                    "DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_salary_components WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_payroll_configs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM department_payroll_configs WHERE department_id IN (SELECT id FROM departments WHERE workspace_id = $1)",
                    "DELETE FROM leave_accrual_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM leave_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM leave_balances WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM leave_policy_assignments WHERE workspace_id = $1",
                    "DELETE FROM employee_emergency_contacts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_documents WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_profiles WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_department_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employment_periods WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employees WHERE workspace_id = $1",
                    "DELETE FROM assignment_rules WHERE workspace_id = $1",
                    "DELETE FROM working_patterns WHERE workspace_id = $1",
                    "DELETE FROM leave_policy_definitions WHERE workspace_id = $1",
                    "DELETE FROM salary_component_definitions WHERE workspace_id = $1",
                    "DELETE FROM shifts WHERE workspace_id = $1",
                    "DELETE FROM departments WHERE workspace_id = $1",
                ];

                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await?;
                }
            }
            "seed" => {
                let dept_eng = sqlx::query_scalar::<_, Uuid>(
                    "INSERT INTO departments (workspace_id, name, description) VALUES ($1, 'Engineering', 'Software Development') RETURNING id"
                 )
                 .bind(ws_id)
                 .fetch_one(&mut *tx)
                 .await?;

                let dept_hr = sqlx::query_scalar::<_, Uuid>(
                    "INSERT INTO departments (workspace_id, name, description) VALUES ($1, 'Human Resources', 'People Operations') RETURNING id"
                 )
                 .bind(ws_id)
                 .fetch_one(&mut *tx)
                 .await?;

                let _shift_morning = sqlx::query_scalar::<_, Uuid>(
                    "INSERT INTO shifts (workspace_id, name, start_time, end_time) VALUES ($1, 'Morning Shift', '08:00:00', '17:00:00') RETURNING id"
                 )
                 .bind(ws_id)
                 .fetch_one(&mut *tx)
                 .await?;

                let employees = vec![
                    ("Alice Engineer", "alice@example.com", "EMP001", dept_eng),
                    ("Bob Developer", "bob@example.com", "EMP002", dept_eng),
                    ("Charlie Manager", "charlie@example.com", "EMP003", dept_eng),
                    ("David HR", "david@example.com", "EMP004", dept_hr),
                    ("Eve Recruiter", "eve@example.com", "EMP005", dept_hr),
                ];

                for (name, email, code, dept_id) in employees {
                    let emp_id = crate::utils::id::new_id();
                    sqlx::query(
                         "INSERT INTO employees (id, workspace_id, full_name, email, employee_id_number, is_active) VALUES ($1, $2, $3, $4, $5, true)"
                     )
                     .bind(emp_id)
                     .bind(ws_id)
                     .bind(name)
                     .bind(email)
                     .bind(code)
                     .execute(&mut *tx)
                     .await?;

                    sqlx::query(
                         "INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, CURRENT_DATE)"
                     )
                     .bind(emp_id)
                     .bind(dept_id)
                     .execute(&mut *tx)
                     .await?;

                    sqlx::query(
                        "INSERT INTO employment_periods (employee_id, hire_date, type) VALUES ($1, CURRENT_DATE, 'FullTime')"
                     )
                     .bind(emp_id)
                     .execute(&mut *tx)
                     .await?;
                }
            }
            _ => return Err(AppError::Validation("Invalid group".to_string())),
        }

        tx.commit().await?;
        Ok((StatusCode::OK, "Cleared"))
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = (ws_id, state, payload);
        Err::<(), AppError>(AppError::NotFound("Not found".to_string()))
    }
}
