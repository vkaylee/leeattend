use super::*;

/// Build PayrollContext values for a chunk of employments.
/// Returns a tuple per employment: `(employment_id, Result<context, error_result>)`.
/// Failures are pre-built `BatchCalculateResult` with `status: "error"`.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn build_payroll_contexts(
    chunk_employment_ids: &[Uuid],
    employment_to_employee: &HashMap<Uuid, Uuid>,
    employees: &mut HashMap<Uuid, crate::models::employee::Employee>,
    time_map: &mut HashMap<Uuid, Vec<crate::models::TimeEntry>>,
    roster_map: &mut HashMap<Uuid, Vec<crate::models::shift::ShiftRoster>>,
    conditions_map: &mut HashMap<Uuid, Vec<crate::models::WorkingCondition>>,
    policies_map: &mut HashMap<Uuid, crate::models::AttendancePolicyVersion>,
    emp_configs_map: &mut HashMap<Uuid, Vec<crate::models::EmployeePayrollConfig>>,
    dept_map: &mut HashMap<Uuid, Vec<crate::models::EmployeeDepartmentHistory>>,
    batch_dept_payroll_configs: &HashMap<Uuid, Vec<crate::models::DepartmentPayrollConfig>>,
    emp_to_location: &HashMap<Uuid, Uuid>,
    emp_to_employments: &HashMap<Uuid, Vec<crate::models::employment::Employment>>,
    salary_map: &mut HashMap<
        Uuid,
        Vec<(
            crate::models::EmployeeSalaryComponent,
            crate::models::SalaryComponentDefinition,
        )>,
    >,
    leave_breakdowns_map: &mut HashMap<
        Uuid,
        crate::repositories::leave_request_repository::LeaveBreakdown,
    >,
    unpaid_holidays_map: &mut HashMap<Uuid, i64>,
    rule_assignments_map: &HashMap<Uuid, Vec<RuleAssignmentWithTiers>>,
    month_end_policies_map: &mut HashMap<Uuid, crate::models::AttendancePolicyVersion>,
    ot_base_component_ids: &std::collections::HashSet<Uuid>,
    contracts_map: &mut HashMap<Uuid, Vec<crate::models::contract::EmployeeContract>>,
    assignments_map: &mut HashMap<
        Uuid,
        Vec<crate::models::employee_assignment::EmployeeAssignment>,
    >,
    d: &Arc<ChunkSharedData>,
) -> Vec<(
    Uuid,
    Result<PayrollContext, crate::models::BatchCalculateResult>,
)> {
    let mut results: Vec<(
        Uuid,
        Result<PayrollContext, crate::models::BatchCalculateResult>,
    )> = Vec::new();
    for employment_id in chunk_employment_ids {
        let emp_id = if let Some(id) = employment_to_employee.get(employment_id) {
            *id
        } else {
            results.push((
                *employment_id,
                Err(crate::models::BatchCalculateResult {
                    employment_id: *employment_id,
                    employee_id: Uuid::nil(),
                    status: "error".to_string(),
                    payslip: None,
                    error_message: Some("Employment has no associated employee record".into()),
                }),
            ));
            continue;
        };
        let Some(employee) = employees.remove(&emp_id) else {
            results.push((
                *employment_id,
                Err(crate::models::BatchCalculateResult {
                    employment_id: *employment_id,
                    employee_id: emp_id,
                    status: "error".to_string(),
                    payslip: None,
                    error_message: Some("Employee data not found in batch fetch results".into()),
                }),
            ));
            continue;
        };
        let daily_entries_map =
            build_daily_entries_map(time_map.remove(employment_id).unwrap_or_default(), &d.tz);

        let payroll_roster_map = roster_map
            .remove(employment_id)
            .unwrap_or_default()
            .into_iter()
            .map(|r| ((r.employment_id, r.date), r.shift_id))
            .collect();

        let working_conditions = conditions_map.remove(employment_id).unwrap_or_default();

        let attendance_policy = policies_map
            .remove(employment_id)
            .map_or_else(default_attendance_rules, |pv| pv.rules);

        let emp_payroll_configs = emp_configs_map.remove(employment_id).unwrap_or_default();

        let all_dept_history = dept_map.remove(&emp_id).unwrap_or_default();
        let mut emp_dept_payroll_configs = HashMap::new();
        for dh in &all_dept_history {
            if let Some(configs) = batch_dept_payroll_configs.get(&dh.department_id()) {
                emp_dept_payroll_configs.insert(dh.department_id(), configs.clone());
            }
        }

        let primary_location_id = emp_to_location
            .get(&emp_id)
            .copied()
            .unwrap_or_else(Uuid::new_v4);

        let salary_history = salary_map.remove(employment_id).unwrap_or_default();
        let ot_base_comp_ids = if ot_base_component_ids.is_empty() {
            salary_history.iter().map(|(_, scd)| scd.id).collect()
        } else {
            ot_base_component_ids.clone()
        };

        let month_end_attendance_policy = month_end_policies_map
            .remove(employment_id)
            .map_or_else(default_attendance_rules, |pv| pv.rules);

        let leave_breakdown = leave_breakdowns_map.remove(&emp_id).unwrap_or(
            crate::repositories::leave_request_repository::LeaveBreakdown {
                total_days: 0.0,
                unpaid_days: 0.0,
                sick_days: 0.0,
            },
        );

        let unpaid_holiday_days = unpaid_holidays_map.remove(&emp_id).unwrap_or(0);
        let rule_assignments_with_tiers = rule_assignments_map
            .get(employment_id)
            .cloned()
            .unwrap_or_default();

        results.push((
            *employment_id,
            Ok(PayrollContext {
                employee_id: emp_id,
                workspace: d.workspace.clone(),
                tz: d.tz,
                start_date: d.start_date,
                end_date: d.end_date,
                salary_history,
                all_dept_history,
                daily_entries_map,
                holidays_map: Arc::clone(&d.holidays_map),
                roster_data: Arc::clone(&d.roster_data),
                emp_id_number: employee.employee_id_number().to_string(),
                emp_tags: Some(employee.tags().to_vec()),
                payroll_roster_map,
                working_conditions,
                condition_type_catalog: Arc::clone(&d.condition_type_catalog),
                attendance_policy,
                emp_payroll_configs,
                ws_payroll_settings_history: Arc::clone(&d.ws_payroll_settings_history),
                department_payroll_configs: emp_dept_payroll_configs,
                overtime_data: Arc::clone(&d.overtime_data),
                ot_base_component_ids: ot_base_comp_ids,
                active_contracts: contracts_map.remove(&emp_id).unwrap_or_default(),
                active_assignments: assignments_map.remove(employment_id).unwrap_or_default(),
                active_employments: emp_to_employments.get(&emp_id).cloned().unwrap_or_default(),
                primary_location_id,
                month_end_payroll_settings: Arc::clone(&d.month_end_payroll_settings),
                month_end_attendance_policy,
                leave_breakdown,
                active_policies_with_tiers: Arc::clone(&d.active_policies_with_tiers),
                salary_base_groups: Arc::clone(&d.salary_base_groups),
                unpaid_holiday_days,
                rule_assignments_with_tiers,
            }),
        ));
    }
    results
}
