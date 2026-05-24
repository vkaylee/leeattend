#![allow(unused_variables)]
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

#[derive(Serialize)]
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
        let mut conn = state
            .infra
            .diesel
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        use diesel_async::RunQueryDsl;

        #[derive(diesel::QueryableByName)]
        struct DebugUserRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            email: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            role: String,
        }

        let users: Vec<DebugUserRow> = crate::system_crud_query!(
            &mut conn,
            r#"SELECT id, COALESCE(full_name, 'Unknown') as name, email, role FROM users LIMIT 20"#,
        )
        .load(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let mapped = users
            .into_iter()
            .map(|u| DebugUser {
                id: u.id,
                name: u.name,
                email: u.email,
                role: u.role,
            })
            .collect();

        Ok(Json(mapped))
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
        let mut conn = state
            .infra
            .diesel
            .tenant_transaction_conn(&crate::diesel_pool::TenantAuthToken::system_override(ws_id))
            .await?;

        use diesel_async::AsyncConnection;
        use diesel_async::RunQueryDsl;

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                match payload.group.as_str() {
                    "attendance" => {
                        let queries = [
                            "DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM raw_attendance_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM attendance_import_jobs WHERE workspace_id = $1",
                        ];
                        for q in queries {
                            crate::system_crud_query!(conn, q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|e| AppError::Internal(e.to_string()))?;
                        }
                    }
                    "payroll" => {
                        let queries = [
                            "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1))",
                            "DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                        ];
                        for q in queries {
                            crate::system_crud_query!(conn, q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|e| AppError::Internal(e.to_string()))?;
                        }
                    }
                    "rosters" => {
                        let queries = [
                            "DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                        ];
                        for q in queries {
                            crate::system_crud_query!(conn, q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|e| AppError::Internal(e.to_string()))?;
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
                            "DELETE FROM employees WHERE workspace_id = $1",
                        ];
                        for q in queries {
                            crate::system_crud_query!(conn, q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|e| AppError::Internal(e.to_string()))?;
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
                            "DELETE FROM employees WHERE workspace_id = $1",
                            "DELETE FROM assignment_rules WHERE workspace_id = $1",
                            "DELETE FROM working_patterns WHERE workspace_id = $1",
                            "DELETE FROM leave_policy_definitions WHERE workspace_id = $1",
                            "DELETE FROM salary_component_definitions WHERE workspace_id = $1",
                            "DELETE FROM shifts WHERE workspace_id = $1",
                            "DELETE FROM departments WHERE workspace_id = $1",
                        ];

                        for q in queries {
                            crate::system_crud_query!(conn, q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|e| AppError::Internal(e.to_string()))?;
                        }
                    }
                    "seed" => {
                        #[derive(diesel::QueryableByName)]
                        struct IdRow {
                            #[diesel(sql_type = diesel::sql_types::Uuid)]
                            id: Uuid,
                        }

                        let dept_eng: IdRow = crate::system_crud_query!(conn, 
                            "INSERT INTO departments (workspace_id, name, description) VALUES ($1, 'Engineering', 'Software Development') RETURNING id"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(ws_id)
                        .get_result(conn)
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;

                        let dept_hr: IdRow = crate::system_crud_query!(conn, 
                            "INSERT INTO departments (workspace_id, name, description) VALUES ($1, 'Human Resources', 'People Operations') RETURNING id"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(ws_id)
                        .get_result(conn)
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;

                        let _shift_morning: IdRow = crate::system_crud_query!(conn, 
                            "INSERT INTO shifts (workspace_id, name, start_time, end_time) VALUES ($1, 'Morning Shift', '08:00:00', '17:00:00') RETURNING id"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(ws_id)
                        .get_result(conn)
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;

                        let employees = vec![
                            ("Alice Engineer", "alice@example.com", "EMP001", dept_eng.id),
                            ("Bob Developer", "bob@example.com", "EMP002", dept_eng.id),
                            ("Charlie Manager", "charlie@example.com", "EMP003", dept_eng.id),
                            ("David HR", "david@example.com", "EMP004", dept_hr.id),
                            ("Eve Recruiter", "eve@example.com", "EMP005", dept_hr.id),
                        ];

                        for (name, email, code, dept_id) in employees {
                            let emp_id = crate::utils::id::new_id();
                            crate::system_crud_query!(conn, 
                                "INSERT INTO employees (id, workspace_id, full_name, email, employee_id_number, original_hire_date, hire_date, is_active) VALUES ($1, $2, $3, $4, $5, CURRENT_DATE, CURRENT_DATE, true)"
                            )
                            .bind::<diesel::sql_types::Uuid, _>(emp_id)
                            .bind::<diesel::sql_types::Uuid, _>(ws_id)
                            .bind::<diesel::sql_types::Text, _>(name)
                            .bind::<diesel::sql_types::Text, _>(email)
                            .bind::<diesel::sql_types::Text, _>(code)
                            .execute(conn)
                            .await
                            .map_err(|e| AppError::Internal(e.to_string()))?;

                            crate::system_crud_query!(conn, 
                                "INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, CURRENT_DATE)"
                            )
                            .bind::<diesel::sql_types::Uuid, _>(emp_id)
                            .bind::<diesel::sql_types::Uuid, _>(dept_id)
                            .execute(conn)
                            .await
                            .map_err(|e| AppError::Internal(e.to_string()))?;
                        }
                    }
                    _ => return Err(AppError::Validation("Invalid group".to_string())),
                }

                Ok(())
            })
        })
        .await?;

        Ok((StatusCode::OK, "Cleared"))
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = (ws_id, state, payload);
        Err::<(), AppError>(AppError::NotFound("Not found".to_string()))
    }
}
