use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;
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
            .infra()
            .diesel()
            .api
            .get()
            .await
            .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

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

        let users: Vec<DebugUserRow> = diesel::sql_query(
            r"
            SELECT u.id, COALESCE(u.full_name, 'Unknown') as name, u.email, COALESCE(sr.name, 'user') as role
            FROM users u
            LEFT JOIN user_system_roles usr ON usr.user_id = u.id
            LEFT JOIN system_roles sr ON sr.id = usr.role_id
            LIMIT 20
            ",
        )
        .load(&mut *conn)
        .await
        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

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
        Err(AppError::not_found(I18nKey::NotFound))
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
            .infra()
            .diesel()
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
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
                        }
                    }
                    "payroll" => {
                        let queries = [
                            "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1))",
                            "DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                        ];
                        for q in queries {
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
                        }
                    }
                    "rosters" => {
                        let queries = [
                            "DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                        ];
                        for q in queries {
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
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
                            "DELETE FROM workspace_config_entries WHERE workspace_id = $1 AND scope_type = 'employee'",
                            "DELETE FROM employee_working_patterns WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_emergency_contacts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_documents WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_profiles WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_department_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employees WHERE workspace_id = $1",
                        ];
                        for q in queries {
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
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
                            "DELETE FROM workspace_config_entries WHERE workspace_id = $1",
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
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(ws_id)
                                .execute(conn)
                                .await
                                .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
                        }
                    }
                    "seed" => {
                        #[derive(diesel::QueryableByName)]
                        struct IdRow {
                            #[diesel(sql_type = diesel::sql_types::Uuid)]
                            id: Uuid,
                        }

                        let dept_eng: IdRow = diesel::sql_query("INSERT INTO departments (workspace_id, name, description) VALUES ($1, 'Engineering', 'Software Development') RETURNING id"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(ws_id)
                        .get_result(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        let dept_hr: IdRow = diesel::sql_query("INSERT INTO departments (workspace_id, name, description) VALUES ($1, 'Human Resources', 'People Operations') RETURNING id"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(ws_id)
                        .get_result(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        let _shift_morning: IdRow = diesel::sql_query("INSERT INTO shifts (workspace_id, name, start_time, end_time) VALUES ($1, 'Morning Shift', '08:00:00', '17:00:00') RETURNING id"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(ws_id)
                        .get_result(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        let loc_id = crate::utils::id::new_id();
                        diesel::sql_query(
                            "INSERT INTO work_locations (id, workspace_id, name, is_headquarters, timezone, status) \
                             VALUES ($1, $2, 'Headquarters', true, 'UTC', 'active') \
                             ON CONFLICT (workspace_id) WHERE is_headquarters = true DO NOTHING"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(loc_id)
                        .bind::<diesel::sql_types::Uuid, _>(ws_id)
                        .execute(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        #[derive(diesel::QueryableByName)]
                        struct LocIdRow {
                            #[diesel(sql_type = diesel::sql_types::Uuid)]
                            id: Uuid,
                        }

                        let loc: LocIdRow = diesel::sql_query(
                            "SELECT id FROM work_locations WHERE workspace_id = $1 AND is_headquarters = true LIMIT 1"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(ws_id)
                        .get_result(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        let employees = vec![
                            ("Alice Engineer", "EMP001", dept_eng.id),
                            ("Bob Developer", "EMP002", dept_eng.id),
                            ("Charlie Manager", "EMP003", dept_eng.id),
                            ("David HR", "EMP004", dept_hr.id),
                            ("Eve Recruiter", "EMP005", dept_hr.id),
                        ];

                        for (name, code, dept_id) in employees {
                            let emp_id = crate::utils::id::new_id();
                            diesel::sql_query(
                                "INSERT INTO employees (id, workspace_id, full_name, employee_id_number, original_hire_date, hire_date, is_active, primary_location_id) \
                                 VALUES ($1, $2, $3, $4, CURRENT_DATE, CURRENT_DATE, true, $5)"
                            )
                            .bind::<diesel::sql_types::Uuid, _>(emp_id)
                            .bind::<diesel::sql_types::Uuid, _>(ws_id)
                            .bind::<diesel::sql_types::Text, _>(name)
                            .bind::<diesel::sql_types::Text, _>(code)
                            .bind::<diesel::sql_types::Uuid, _>(loc.id)
                            .execute(conn)
                            .await
                            .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                            diesel::sql_query("INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, CURRENT_DATE)"
                            )
                            .bind::<diesel::sql_types::Uuid, _>(emp_id)
                            .bind::<diesel::sql_types::Uuid, _>(dept_id)
                            .execute(conn)
                            .await
                            .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
                        }
                    }
                    _ => return Err(AppError::validation(I18nKey::InvalidGroup)),
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
        Err::<(), AppError>(AppError::not_found(I18nKey::NotFound))
    }
}
