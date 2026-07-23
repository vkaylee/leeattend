use crate::diesel_pool::DieselPools;
use crate::errors::{AppError, AppResult};
use crate::repositories::debug_repository::{DebugRepository, DebugUserInfo};
use crate::utils::i18n::I18nKey;
use async_trait::async_trait;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

pub struct DieselDebugRepository {
    diesel_pools: DieselPools,
}

impl DieselDebugRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
#[allow(clippy::too_many_lines)]
impl DebugRepository for DieselDebugRepository {
    async fn get_debug_users(&self) -> AppResult<Vec<DebugUserInfo>> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

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
            .map(|u| DebugUserInfo {
                id: u.id,
                name: u.name,
                email: u.email,
                role: u.role,
            })
            .collect();

        Ok(mapped)
    }

    async fn clear_debug_db(&self, workspace_id: Uuid, group: &str) -> AppResult<()> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(&crate::diesel_pool::TenantAuthToken::system_override(
                workspace_id,
            ))
            .await?;

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                match group {
                    "attendance" => {
                        let queries = [
                            "DELETE FROM attendance_adjustment_requests WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM time_entries WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM raw_attendance_logs WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM attendance_import_jobs WHERE workspace_id = $1",
                        ];
                        for q in queries {
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                                .execute(conn)
                                .await
                                .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
                        }
                    }
                    "payroll" => {
                        let queries = [
                            "DELETE FROM payslip_incentives WHERE payslip_id IN (SELECT id FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1))",
                            "DELETE FROM payslip_rule_evaluations WHERE payslip_id IN (SELECT id FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1))",
                            "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1))",
                            "DELETE FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                        ];
                        for q in queries {
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                                .execute(conn)
                                .await
                                .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
                        }
                    }
                    "rosters" => {
                        let queries = [
                            "DELETE FROM shift_rosters WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                        ];
                        for q in queries {
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                                .execute(conn)
                                .await
                                .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
                        }
                    }
                    "employees" => {
                        let queries = [
                            "DELETE FROM attendance_adjustment_requests WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM time_entries WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM raw_attendance_logs WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM shift_rosters WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM work_segments WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employee_shifts WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM payslip_incentives WHERE payslip_id IN (SELECT id FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1))",
                            "DELETE FROM payslip_rule_evaluations WHERE payslip_id IN (SELECT id FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1))",
                            "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1))",
                            "DELETE FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM leave_accrual_logs WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM leave_requests WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM leave_balances WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employee_salary_components WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM workspace_config_entries WHERE workspace_id = $1 AND scope_type = 'employee'",
                            "DELETE FROM employee_working_patterns WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employee_emergency_contacts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_documents WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_profiles WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_assignments WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employee_contracts WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employments WHERE workspace_id = $1",
                            "DELETE FROM employees WHERE workspace_id = $1",
                        ];
                        for q in queries {
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                                .execute(conn)
                                .await
                                .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;
                        }
                    }
                    "full_reset" => {
                        let queries = [
                            "DELETE FROM attendance_adjustment_requests WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM time_entries WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM raw_attendance_logs WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM attendance_import_jobs WHERE workspace_id = $1",
                            "DELETE FROM shift_rosters WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM work_segments WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employee_shifts WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employee_working_patterns WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM payslip_incentives WHERE payslip_id IN (SELECT id FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1))",
                            "DELETE FROM payslip_rule_evaluations WHERE payslip_id IN (SELECT id FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1))",
                            "DELETE FROM payslip_segments WHERE payslip_id IN (SELECT id FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1))",
                            "DELETE FROM payslips WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employee_salary_components WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM workspace_config_entries WHERE workspace_id = $1",
                            "DELETE FROM leave_accrual_logs WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM leave_requests WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM leave_balances WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM leave_policy_assignments WHERE workspace_id = $1",
                            "DELETE FROM employee_emergency_contacts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_documents WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_profiles WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
                            "DELETE FROM employee_assignments WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employee_contracts WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = $1)",
                            "DELETE FROM employments WHERE workspace_id = $1",
                            "DELETE FROM employees WHERE workspace_id = $1",
                            "DELETE FROM assignment_rules WHERE workspace_id = $1",
                            "DELETE FROM working_pattern_details WHERE pattern_id IN (SELECT id FROM working_patterns WHERE workspace_id = $1)",
                            "DELETE FROM working_patterns WHERE workspace_id = $1",
                            "DELETE FROM leave_policy_definitions WHERE workspace_id = $1",
                            "DELETE FROM salary_component_definitions WHERE workspace_id = $1",
                            "UPDATE shift_versions SET superseded_by = NULL WHERE template_id IN (SELECT id FROM shift_templates WHERE workspace_id = $1)",
                            "DELETE FROM shift_audit_log WHERE template_id IN (SELECT id FROM shift_templates WHERE workspace_id = $1)",
                            "DELETE FROM shift_default_intervals WHERE version_id IN (SELECT id FROM shift_versions WHERE template_id IN (SELECT id FROM shift_templates WHERE workspace_id = $1))",
                            "DELETE FROM shift_versions WHERE workspace_id = $1",
                            "DELETE FROM shift_templates WHERE workspace_id = $1",
                            "DELETE FROM departments WHERE workspace_id = $1",
                        ];

                        for q in queries {
                            diesel::sql_query(q)
                                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
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
                        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                        .get_result(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        let dept_hr: IdRow = diesel::sql_query("INSERT INTO departments (workspace_id, name, description) VALUES ($1, 'Human Resources', 'People Operations') RETURNING id"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                        .get_result(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        let now = chrono::Utc::now();
                        let today = now.date_naive();
                        let shift_morning_tpl: IdRow = diesel::sql_query("INSERT INTO shift_templates (workspace_id, name, created_at) VALUES ($1, 'Morning Shift', $2) RETURNING id"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                        .bind::<diesel::sql_types::Timestamptz, _>(now)
                        .get_result(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        let shift_morning_ver: IdRow = diesel::sql_query("INSERT INTO shift_versions (template_id, workspace_id, version_number, name, effective_from, change_type, created_by, created_at) VALUES ($1, $2, 1, 'Morning Shift', $3, 'create', '00000000-0000-0000-0000-000000000000', $4) RETURNING id"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(shift_morning_tpl.id)
                        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                        .bind::<diesel::sql_types::Date, _>(today)
                        .bind::<diesel::sql_types::Timestamptz, _>(now)
                        .get_result(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        diesel::sql_query("UPDATE shift_templates SET current_version_id = $1 WHERE id = $2"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(shift_morning_ver.id)
                        .bind::<diesel::sql_types::Uuid, _>(shift_morning_tpl.id)
                        .execute(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        diesel::sql_query("INSERT INTO shift_default_intervals (version_id, sort_order, start_time, end_time) VALUES ($1, 0, '08:00:00', '17:00:00')"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(shift_morning_ver.id)
                        .execute(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        let loc_id = crate::utils::id::new_id();
                        diesel::sql_query(
                            "INSERT INTO work_locations (id, workspace_id, name, is_headquarters, timezone, status) \
                             VALUES ($1, $2, 'Headquarters', true, 'UTC', 'active') \
                             ON CONFLICT (workspace_id) WHERE is_headquarters = true DO NOTHING"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(loc_id)
                        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
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
                        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                        .get_result(conn)
                        .await
                        .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                        #[derive(diesel::QueryableByName)]
                        struct EmployerIdRow {
                            #[diesel(sql_type = diesel::sql_types::Uuid)]
                            id: Uuid,
                        }

                        let employer: EmployerIdRow = if let Ok(row) = diesel::sql_query(
                            "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1"
                        )
                        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                        .get_result(conn)
                        .await {
                            row
                        } else {
                            let new_employer_id = crate::utils::id::new_id();
                            diesel::sql_query(
                                "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address) \
                                 VALUES ($1, $2, 'Seeded Employer', 'SEED-TAX-1', 'Seeded Address') \
                                 RETURNING id"
                            )
                            .bind::<diesel::sql_types::Uuid, _>(new_employer_id)
                            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                            .get_result(conn)
                            .await
                            .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?
                        };

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
                                "INSERT INTO employees (id, workspace_id, full_name, employee_id_number, hire_date, original_hire_date) \
                                 VALUES ($1, $2, $3, $4, CURRENT_DATE, CURRENT_DATE)"
                            )
                            .bind::<diesel::sql_types::Uuid, _>(emp_id)
                            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                            .bind::<diesel::sql_types::Text, _>(name)
                            .bind::<diesel::sql_types::Text, _>(code)
                            .execute(conn)
                            .await
                            .map_err(|_e| AppError::service_unavailable(I18nKey::ServiceUnavailable))?;

                            let employment_id = crate::utils::id::new_id();
                            diesel::sql_query(
                                "INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, department_id, status, start_date) \
                                 VALUES ($1, $2, $3, $4, $5, $6, 'active', CURRENT_DATE)"
                            )
                            .bind::<diesel::sql_types::Uuid, _>(employment_id)
                            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                            .bind::<diesel::sql_types::Uuid, _>(emp_id)
                            .bind::<diesel::sql_types::Uuid, _>(employer.id)
                            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(loc.id))
                            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(dept_id))
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

        Ok(())
    }
}
