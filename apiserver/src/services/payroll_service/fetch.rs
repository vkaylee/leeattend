use super::*;
use crate::utils::tz::local_to_utc;

impl PayrollService {
    /// Phase 1: Fetch all data needed for payroll calculation.
    /// Public to allow access from decomposed structs.
    #[allow(clippy::too_many_lines)]
    pub(super) async fn fetch_context(
        &self,
        employee_id: Uuid,
        auth_token: &TenantAuthToken,
        year: i32,
        month: u32,
    ) -> Result<PayrollContext, AppError> {
        let workspace_id = auth_token.workspace_id();
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                AppError::Internal(format!("Workspace fetch error: {e}"))
            })?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        let start_date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
        let next_month_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
        let end_date = next_month_date
            .pred_opt()
            .ok_or_else(|| AppError::Internal("Date calculation error".into()))?;

        let all_dept_history = self
            .department_repo
            .get_employee_department_history(auth_token, employee_id)
            .await?;

        let employee = self
            .employee_repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;

        let active_emp_id = employee
            .active_employment_id()
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

        let salary_history = self
            .payroll_repo
            .get_salary_components_history(workspace_id, active_emp_id, start_date, end_date)
            .await?;

        let emp_id_number = employee.employee_id_number().to_string();
        let emp_tags = Some(employee.tags().to_vec());

        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0).map_err(AppError::Internal)?;
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59).map_err(AppError::Internal)?;

        let time_entries = self
            .time_entry_repo
            .get_employment_time_entries(auth_token, active_emp_id, start_dt, end_dt)
            .await?;

        let holidays_map = self
            .holiday_repo
            .get_holidays_in_range(auth_token, start_date, end_date)
            .await?;

        let roster_data = self.roster_provider.get_roster_data(workspace_id).await?;

        let (rosters, _) = self
            .roster_repo
            .get_employment_rosters(auth_token, active_emp_id, start_date, end_date, None, None)
            .await?;
        let payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = rosters
            .into_iter()
            .map(|r| ((r.employment_id, r.date), r.shift_id))
            .collect();

        let mut daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> =
            HashMap::new();
        for entry in time_entries {
            let local_date = entry.punch_in_time().with_timezone(&tz).date_naive();
            daily_entries_map.entry(local_date).or_default().push(entry);
        }

        let (working_conditions, _) = self
            .employee_repo
            .get_working_conditions(auth_token, active_emp_id, 1, 3000)
            .await?;
        let condition_type_catalog = PolicyResolverService::load_condition_type_catalog(
            self.condition_type_repo.as_ref(),
            auth_token,
        )
        .await?;

        let effective_time = start_date
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc();
        let policy_version = self
            .policy_repo
            .get_employment_effective_policy(auth_token, active_emp_id, effective_time)
            .await?;

        let attendance_policy = if let Some(pv) = policy_version {
            pv.rules
        } else {
            crate::models::AttendanceRules::default()
        };

        let emp_payroll_configs = self
            .employee_repo
            .get_employee_payroll_configs(auth_token, active_emp_id)
            .await?;

        let ws_payroll_settings_history = self
            .payroll_repo
            .get_payroll_settings_history(workspace_id)
            .await?;

        let department_ids: Vec<Uuid> = all_dept_history
            .iter()
            .map(crate::models::EmployeeDepartmentHistory::department_id)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let department_payroll_configs = self
            .department_repo
            .get_department_payroll_configs_batch(auth_token, &department_ids)
            .await?;

        let overtime_data = self
            .overtime_policy_repo
            .get_workspace_overtime_data(workspace_id)
            .await?;

        let active_assignments = self
            .employee_repo
            .list_employee_assignments(auth_token, active_emp_id)
            .await?;

        let active_employment = self
            .employment_repo
            .get_active_employment_on_date(auth_token, employee_id, start_date)
            .await?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

        let primary_location_id = active_employment.primary_location_id().ok_or_else(|| {
            AppError::not_found_args(
                crate::utils::i18n::I18nKey::NotFound,
                vec![("details", "primary_location_not_found".to_string())],
            )
        })?;

        let active_employments = self
            .employment_repo
            .list_employee_employments(auth_token, employee_id)
            .await?;

        let groups = self
            .salary_base_repo
            .get_groups(workspace_id)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    error = %e,
                    "fetch_context: get_groups failed (ot_base lookup) — returning empty"
                );
                Vec::new()
            });
        let ot_group_id = groups.iter().find(|g| g.slug == "ot_base").map(|g| g.id);

        let mut ot_base_component_ids = std::collections::HashSet::new();
        if let Some(group_id) = ot_group_id {
            if let Ok(members) = self.salary_base_repo.get_group_members(workspace_id).await {
                for member in members {
                    if member.base_group_id == group_id {
                        ot_base_component_ids.insert(member.component_definition_id);
                    }
                }
            }
        } else {
            // Fallback for tests or unseeded workspaces: treat all components as part of OT base
            for (_, scd) in &salary_history {
                ot_base_component_ids.insert(scd.id);
            }
        }

        // Fetch month end payroll settings
        let month_end_payroll_settings = self
            .payroll_repo
            .get_payroll_settings_at(workspace_id, end_date)
            .await?;

        // Resolve month end attendance policy
        let month_end_attendance_policy = PolicyResolverService::resolve_attendance_policy(
            &*self.policy_repo,
            employee_id,
            auth_token,
            end_date,
        )
        .await?;

        // Fetch leave breakdown
        let leave_breakdown = self
            .leave_repo
            .get_approved_leave_breakdown(employee_id, start_date, end_date, auth_token)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to fetch leave breakdown for employee {}: {}",
                    employee_id,
                    e
                );
                AppError::Internal(format!("Leave breakdown error (incentives): {e}"))
            })?;

        // Fetch active incentive policies with tiers
        let active_policies = self
            .payroll_repo
            .get_active_incentive_policies_for_period(workspace_id, start_date, end_date)
            .await?;

        let policy_ids: Vec<Uuid> = active_policies
            .iter()
            .map(crate::models::PayrollIncentivePolicy::id)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let policy_tiers_map = self
            .payroll_repo
            .get_policy_tiers_batch(workspace_id, &policy_ids)
            .await?;

        let active_policies_with_tiers: Vec<IncentivePolicyWithTiers> = active_policies
            .into_iter()
            .map(|policy| {
                let tiers = policy_tiers_map
                    .get(&policy.id())
                    .cloned()
                    .unwrap_or_default();
                IncentivePolicyWithTiers { policy, tiers }
            })
            .collect();

        // Fetch salary base groups
        let salary_base_groups = self
            .salary_base_repo
            .get_groups(workspace_id)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    employee_id = %employee_id,
                    error = %e,
                    "fetch_context: get_groups failed (salary base) — returning empty"
                );
                Vec::new()
            });

        // Fetch unpaid holiday days
        let unpaid_holiday_days: i64 = self
            .leave_repo
            .get_unpaid_holiday_days_without_leave(auth_token, employee_id, start_date, end_date)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    employee_id = %employee_id,
                    error = %e,
                    "fetch_context: get_unpaid_holiday_days_without_leave failed — defaulting to 0"
                );
                0
            });

        // Fetch last department id
        let last_department_id = all_dept_history
            .iter()
            .filter(|d| d.effective_date() <= end_date)
            .max_by_key(|d| d.effective_date())
            .map(crate::models::EmployeeDepartmentHistory::department_id);

        // Fetch rule assignments with tiers
        let rule_assignments = self
            .payroll_repo
            .get_resolved_rule_assignments(
                workspace_id,
                employee_id,
                last_department_id,
                start_date,
                end_date,
            )
            .await?;

        let rule_def_ids: Vec<Uuid> = rule_assignments
            .iter()
            .map(|a| a.rule_definition_id)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let rule_tiers_map = self
            .payroll_repo
            .get_payroll_rule_tiers_batch(workspace_id, &rule_def_ids)
            .await?;

        let rule_assignments_with_tiers: Vec<RuleAssignmentWithTiers> = rule_assignments
            .into_iter()
            .map(|assignment| {
                let tiers = rule_tiers_map
                    .get(&assignment.rule_definition_id)
                    .cloned()
                    .unwrap_or_default();
                RuleAssignmentWithTiers { assignment, tiers }
            })
            .collect();

        // Fetch daily summaries for all active employments in the pay period.
        // These are the single source of truth for standard/OT unit computation.
        let employment_ids: Vec<Uuid> = active_employments
            .iter()
            .map(crate::models::Employment::id)
            .collect();
        let daily_summaries = self
            .daily_summary_repo
            .get_summaries_by_employments_batch(auth_token, &employment_ids, start_date, end_date)
            .await?;

        Ok(PayrollContext {
            employee_id,
            workspace,
            tz,
            start_date,
            end_date,
            salary_history,
            all_dept_history,
            daily_entries_map,
            holidays_map: Arc::new(holidays_map),
            roster_data,
            emp_id_number,
            emp_tags,
            payroll_roster_map,
            working_conditions,
            condition_type_catalog: Arc::new(condition_type_catalog),
            attendance_policy,
            emp_payroll_configs,
            ws_payroll_settings_history: Arc::new(ws_payroll_settings_history),
            department_payroll_configs,
            overtime_data: Arc::new(overtime_data),
            ot_base_component_ids,
            active_assignments,
            active_employments,
            primary_location_id,
            month_end_payroll_settings: Arc::new(month_end_payroll_settings),
            month_end_attendance_policy,
            leave_breakdown,
            active_policies_with_tiers: Arc::new(active_policies_with_tiers),
            salary_base_groups: Arc::new(salary_base_groups),
            unpaid_holiday_days,
            rule_assignments_with_tiers,
            daily_summaries,
        })
    }
}
