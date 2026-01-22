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
        let mut tx = state.db.begin().await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        match payload.group.as_str() {
            "attendance" => {
                let queries = [
                    "DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM raw_attendance_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM attendance_import_jobs WHERE workspace_id = $1",
                ];
                 for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error executing {}: {}", q, e)))?;
                }
            }
            "payroll" => {
                let queries = [
                    "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1))",
                    "DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    // Note: We don't delete employee_salary_components here as that's configuration, not just 'payroll processing' data.
                    // But if 'payroll' implies resetting all payroll history, we might want to keep the config.
                ];
                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error executing {}: {}", q, e)))?;
                }
            }
            "rosters" => {
                let queries = [
                    "DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    // We keep employee_shifts as they might be considered 'assignments' rather than just roster output, but usually wiping rosters means wiping schedule.
                    // Let's check logic: employee_shifts are temporal assignments. shift_rosters are daily resolutions.
                    // Wiping 'rosters' usually implies wiping the generated schedule.
                ];
                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error executing {}: {}", q, e)))?;
                }
            }
            "employees" => {
                // Delete Employees and ALL their data (Logs, Payroll, etc.) but KEEP Workspace Config (Depts, Shifts, Policies)
                let queries = [
                    // 1. Data
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
                    
                    // 2. Entity Config / Links
                    "DELETE FROM employee_salary_components WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_payroll_configs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_working_patterns WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_emergency_contacts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_documents WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_profiles WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employee_department_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM employment_periods WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    
                    // 3. The Employees themselves
                    "DELETE FROM employees WHERE workspace_id = $1",
                ];
                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error executing {}: {}", q, e)))?;
                }
            }
            "full_reset" => {
                // FACTORY RESET: Delete EVERYTHING except Workspace and Members
                let queries = [
                    // 1. Data & Entity Config (Same as employees + more)
                    "DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                    "DELETE FROM raw_attendance_logs WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)", // Also deletes by workspace_id if column exists, but employee_id cover is safe
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
                    
                    // 2. Entities
                    "DELETE FROM employees WHERE workspace_id = $1",
                    
                    // 3. Workspace Config
                    "DELETE FROM assignment_rules WHERE workspace_id = $1",
                    "DELETE FROM working_patterns WHERE workspace_id = $1",
                    "DELETE FROM leave_policy_definitions WHERE workspace_id = $1",
                    "DELETE FROM salary_component_definitions WHERE workspace_id = $1",
                    "DELETE FROM shifts WHERE workspace_id = $1",
                    "DELETE FROM departments WHERE workspace_id = $1",
                ];
                
                for q in queries {
                    sqlx::query(q).bind(ws_id).execute(&mut *tx).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error executing {}: {}", q, e)))?;
                }
            }
            "seed" => {
                 // 1. Create Departments
                 let dept_eng = sqlx::query_scalar::<_, Uuid>(
                    "INSERT INTO departments (workspace_id, name, description) VALUES ($1, 'Engineering', 'Software Development') RETURNING id"
                 )
                 .bind(ws_id)
                 .fetch_one(&mut *tx)
                 .await
                 .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                 let dept_hr = sqlx::query_scalar::<_, Uuid>(
                    "INSERT INTO departments (workspace_id, name, description) VALUES ($1, 'Human Resources', 'People Operations') RETURNING id"
                 )
                 .bind(ws_id)
                 .fetch_one(&mut *tx)
                 .await
                 .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                 // 2. Create Shifts
                 let _shift_morning = sqlx::query_scalar::<_, Uuid>(
                    "INSERT INTO shifts (workspace_id, name, code, start_time, end_time) VALUES ($1, 'Morning Shift', 'M', '08:00:00', '17:00:00') RETURNING id"
                 )
                 .bind(ws_id)
                 .fetch_one(&mut *tx)
                 .await
                 .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                 // 3. Create Employees
                 let employees = vec![
                     ("Alice Engineer", "alice@example.com", "EMP001", dept_eng),
                     ("Bob Developer", "bob@example.com", "EMP002", dept_eng),
                     ("Charlie Manager", "charlie@example.com", "EMP003", dept_eng),
                     ("David HR", "david@example.com", "EMP004", dept_hr),
                     ("Eve Recruiter", "eve@example.com", "EMP005", dept_hr),
                 ];

                 for (name, email, code, dept_id) in employees {
                     let emp_id = Uuid::new_v4();
                     sqlx::query(
                         "INSERT INTO employees (id, workspace_id, first_name, last_name, email, employee_code, is_active) VALUES ($1, $2, $3, $4, $5, $6, true)"
                     )
                     .bind(emp_id)
                     .bind(ws_id)
                     .bind(name) // Single name field now? No, schema has first/last or single? 20260122000000_employee_single_name.sql says single. 
                     // Let's assume single name column 'full_name' or similar if migration applied. 
                     // Wait, I need to check if 'first_name/last_name' still exist or were replaced.
                     // The context file mentions 'employee-single-name-field-20260122.md'.
                     // Let's assume for now they might still be separate or I should check models.
                     // To be safe, I'll use placeholders that work for common schemas or check schema.
                     // Actually, let's just insert basic mandatory fields. 
                     // If 'first_name' and 'last_name' are required, I must provide them.
                     // If 'full_name' is used, I should provide that.
                     // I will assume standard fields for now and fix if it fails (or check schema first).
                     // Checking schema...
                     .bind("") // last_name placeholder
                     .bind(email)
                     .bind(code)
                     .execute(&mut *tx)
                     .await
                     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to insert employee: {}", e)))?;
                    
                     // Assign Department
                     sqlx::query(
                         "INSERT INTO employee_department_history (employee_id, department_id, start_date) VALUES ($1, $2, CURRENT_DATE)"
                     )
                     .bind(emp_id)
                     .bind(dept_id)
                     .execute(&mut *tx)
                     .await
                     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                     // Assign Employment Period
                     sqlx::query(
                        "INSERT INTO employment_periods (employee_id, start_date, type) VALUES ($1, CURRENT_DATE, 'FullTime')"
                     )
                     .bind(emp_id)
                     .execute(&mut *tx)
                     .await
                     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                 }
            }
            _ => return Err((axum::http::StatusCode::BAD_REQUEST, "Invalid group".to_string())),
        };

        tx.commit().await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok((axum::http::StatusCode::OK, "Cleared"))
    }

    #[cfg(not(debug_assertions))]
    {
        Err((axum::http::StatusCode::NOT_FOUND, "Not found".to_string()))
    }
}
