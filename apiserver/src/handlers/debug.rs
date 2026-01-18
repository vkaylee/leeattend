use axum::{extract::{State, Path}, Json, response::IntoResponse};
use std::sync::Arc;
use crate::AppState;
use serde::{Serialize, Deserialize};
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
) -> impl IntoResponse {
    #[cfg(debug_assertions)]
    {
        let users = sqlx::query_as::<_, DebugUser>(
            r#"SELECT id, COALESCE(full_name, 'Unknown') as name, email, role FROM users LIMIT 20"#
        )
        .fetch_all(&state.db)
        .await;

        match users {
            Ok(u) => Json(u).into_response(),
            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }

    #[cfg(not(debug_assertions))]
    {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn clear_debug_db(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ClearDbQuery>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    #[cfg(debug_assertions)]
    {
        let mut conn = state.db.acquire().await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        match payload.group.as_str() {
            "attendance" => {
                sqlx::query("DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)")
                    .bind(ws_id)
                    .execute(&mut *conn)
                    .await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                
                sqlx::query("DELETE FROM raw_attendance_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)")
                    .bind(ws_id)
                    .execute(&mut *conn)
                    .await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
            "payroll" => {
                sqlx::query("DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1))")
                    .bind(ws_id)
                    .execute(&mut *conn)
                    .await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                
                sqlx::query("DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)")
                    .bind(ws_id)
                    .execute(&mut *conn)
                    .await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
            "rosters" => {
                sqlx::query("DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)")
                    .bind(ws_id)
                    .execute(&mut *conn)
                    .await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
            "full_reset" => {
                let mut tx = state.db.begin().await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                
                let queries = [
                    // 1. Attendance & Operations
                    "DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM raw_attendance_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM import_jobs WHERE workspace_id = $1",
                    
                    // 2. Schedules & Rosters
                    "DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_shifts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_working_patterns WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    
                    // 3. Payroll & Components
                    "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1))",
                    "DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_salary_components WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_payroll_configs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM department_payroll_configs WHERE department_id IN (SELECT id FROM departments WHERE workspace_id = $1)",
                    
                    // 4. Leaves & Accruals
                    "DELETE FROM leave_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM leave_balances WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM leave_accrual_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    
                    // 5. Employee History (Periods & Departments)
                    "DELETE FROM employee_department_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employment_periods WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                ];
                
                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error executing {}: {}", q, e)))?;
                }
                
                tx.commit().await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
            _ => return Err((axum::http::StatusCode::BAD_REQUEST, "Invalid group".to_string())),
        };

        Ok((axum::http::StatusCode::OK, "Cleared"))
    }

    #[cfg(not(debug_assertions))]
    {
        Err((axum::http::StatusCode::NOT_FOUND, "Not found".to_string()))
    }
}
