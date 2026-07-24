use super::*;

impl PayrollService {
    /// Process a single chunk of employment IDs for batch payroll calculation.
    #[allow(clippy::too_many_lines)]
    pub(super) async fn process_payroll_chunk(
        &self,
        chunk_employment_ids: Vec<Uuid>,
        d: &Arc<ChunkSharedData>,
        workspace_id: Uuid,
    ) -> Vec<crate::models::BatchCalculateResult> {
        const MAX_CONCURRENT_PAYROLL: usize = 10;

        let effective_time = d
            .start_date
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc();

        // Resolve employments → employee mapping
        let employments_history = self
            .employment_repo
            .get_employments_by_ids(&d.token, &chunk_employment_ids)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    chunk_size = chunk_employment_ids.len(),
                    error = %e,
                    "Batch payroll: get_employments_by_ids failed — chunk will have per-item errors"
                );
                Vec::new()
            });

        let mut employment_to_employee: HashMap<Uuid, Uuid> = HashMap::new();
        let mut employee_ids_set = std::collections::BTreeSet::new();
        for emp_ment in &employments_history {
            employment_to_employee.insert(emp_ment.id(), emp_ment.employee_id());
            employee_ids_set.insert(emp_ment.employee_id());
        }
        let chunk_employee_ids: Vec<Uuid> = employee_ids_set.into_iter().collect();

        // Batch fetch all chunk-level data
        let salary_histories = self
            .payroll_repo
            .get_salary_components_history_batch(
                workspace_id,
                &chunk_employment_ids,
                d.start_date,
                d.end_date,
            )
            .await;

        let dept_histories = self
            .department_repo
            .get_employee_department_history_batch(&d.token, &chunk_employee_ids)
            .await;

        let time_entries = self
            .time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                &d.token,
                &chunk_employment_ids,
                d.start_dt,
                d.end_dt,
            )
            .await;

        let employees_res = self
            .employee_repo
            .get_employees_by_ids(&chunk_employee_ids, &d.token)
            .await;

        let rosters = self
            .roster_repo
            .get_employment_rosters_batch(&d.token, &chunk_employment_ids, d.start_date, d.end_date)
            .await;

        let conditions_res = self
            .employee_repo
            .get_working_conditions_batch_by_employments(
                &d.token,
                &chunk_employment_ids,
                d.start_date,
                d.end_date,
            )
            .await;

        let policies_res = self
            .policy_repo
            .get_employment_effective_policies_batch(
                &d.token,
                &chunk_employment_ids,
                effective_time,
            )
            .await;

        let contracts_res = self
            .contract_repo
            .list_contracts_batch(&d.token, &chunk_employee_ids)
            .await;

        let assignments_res = self
            .employee_repo
            .list_assignments_batch(&d.token, &chunk_employment_ids)
            .await;

        let end_effective_time = d
            .end_date
            .and_hms_opt(23, 59, 59)
            .unwrap_or_default()
            .and_utc();
        let month_end_policies_res = self
            .policy_repo
            .get_employment_effective_policies_batch(
                &d.token,
                &chunk_employment_ids,
                end_effective_time,
            )
            .await;

        let employees = match employees_res {
            Ok(emps) => emps
                .into_iter()
                .map(|e| (e.id(), e))
                .collect::<HashMap<_, _>>(),
            Err(e) => {
                tracing::error!(
                    workspace_id = %workspace_id,
                    chunk_size = chunk_employment_ids.len(),
                    error = %e,
                    "Batch payroll: get_employees_by_ids failed — marking all chunk employments as errors"
                );
                let mut error_results = Vec::with_capacity(chunk_employment_ids.len());
                for &employment_id in &chunk_employment_ids {
                    let emp_id = employment_to_employee
                        .get(&employment_id)
                        .copied()
                        .unwrap_or(Uuid::nil());
                    error_results.push(crate::models::BatchCalculateResult {
                        employment_id,
                        employee_id: emp_id,
                        status: "error".to_string(),
                        payslip: None,
                        error_message: Some(format!("Failed to fetch employee data: {e}")),
                    });
                }
                return error_results;
            }
        };

        // Ensure every employment_id in the chunk has a mapping.
        // If employment lookup failed for some IDs, create error results.
        let mut pre_errors = Vec::new();
        for &employment_id in &chunk_employment_ids {
            if !employment_to_employee.contains_key(&employment_id) {
                pre_errors.push(crate::models::BatchCalculateResult {
                    employment_id,
                    employee_id: Uuid::nil(),
                    status: "error".to_string(),
                    payslip: None,
                    error_message: Some("Employment not found in workspace".to_string()),
                });
            }
        }
        // Remove employment_ids with errors from further processing
        let chunk_employment_ids: Vec<Uuid> = chunk_employment_ids
            .into_iter()
            .filter(|id| employment_to_employee.contains_key(id))
            .collect();

        let salary_map = salary_histories.unwrap_or_default();
        let dept_map = dept_histories.unwrap_or_default();

        let mut time_map = HashMap::new();
        if let Ok(entries) = time_entries {
            for (employment_id, list) in entries {
                time_map.insert(employment_id, list);
            }
        }

        let mut roster_map = HashMap::new();
        if let Ok(r) = rosters {
            for (employment_id, list) in r {
                roster_map.insert(employment_id, list);
            }
        }

        let mut conditions_map = HashMap::new();
        if let Ok(c) = conditions_res {
            for (employment_id, list) in c {
                conditions_map.insert(employment_id, list);
            }
        }

        let policies_map = policies_res.unwrap_or_default();
        let month_end_policies_map = month_end_policies_res.unwrap_or_default();
        let contracts_map = contracts_res.unwrap_or_default();
        let assignments_map = assignments_res.unwrap_or_default();

        let batch_dept_ids: Vec<Uuid> = dept_map
            .values()
            .flatten()
            .map(crate::models::EmployeeDepartmentHistory::department_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        let batch_dept_payroll_configs = self
            .department_repo
            .get_department_payroll_configs_batch(&d.token, &batch_dept_ids)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    dept_count = batch_dept_ids.len(),
                    error = %e,
                    "Batch payroll: get_department_payroll_configs_batch failed — continuing without department configs"
                );
                HashMap::new()
            });

        let emp_configs_map: HashMap<Uuid, Vec<crate::models::EmployeePayrollConfig>> = self
            .employee_repo
            .get_employee_payroll_configs_batch(&d.token, &chunk_employment_ids)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    chunk_size = chunk_employment_ids.len(),
                    error = %e,
                    "Batch payroll: get_employee_payroll_configs_batch failed — continuing without employee configs"
                );
                HashMap::new()
            });

        let mut emp_to_location = HashMap::new();
        let mut emp_to_employments = HashMap::new();
        for emp_ment in &employments_history {
            if emp_ment.status() == "active"
                && emp_ment.start_date() <= d.start_date
                && emp_ment.end_date().is_none_or(|end| end >= d.start_date)
            {
                if let Some(loc_id) = emp_ment.primary_location_id() {
                    emp_to_location.insert(emp_ment.employee_id(), loc_id);
                }
            }
            emp_to_employments
                .entry(emp_ment.employee_id())
                .or_insert_with(Vec::new)
                .push(emp_ment.clone());
        }

        let ot_base_component_ids = self
            .resolve_ot_base_component_ids(workspace_id, &d.salary_base_groups)
            .await;

        let leave_breakdowns_map = self
            .leave_repo
            .get_approved_leave_breakdowns_batch(
                &chunk_employee_ids,
                d.start_date,
                d.end_date,
                &d.token,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    employee_count = chunk_employee_ids.len(),
                    error = %e,
                    "Batch payroll: get_approved_leave_breakdowns_batch failed — continuing without leave data"
                );
                HashMap::new()
            });

        let unpaid_holidays_map = self
            .leave_repo
            .get_unpaid_holiday_days_without_leave_batch(
                &d.token,
                &chunk_employee_ids,
                d.start_date,
                d.end_date,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    employee_count = chunk_employee_ids.len(),
                    error = %e,
                    "Batch payroll: get_unpaid_holiday_days_without_leave_batch failed — continuing without unpaid holiday data"
                );
                HashMap::new()
            });

        // Map employee_id -> last_department_id
        let mut employee_departments = HashMap::new();
        for employment_id in &chunk_employment_ids {
            if let Some(emp_id) = employment_to_employee.get(employment_id) {
                let all_dept_history = dept_map.get(emp_id).cloned().unwrap_or_default();
                let last_department_id = all_dept_history
                    .iter()
                    .filter(|dh| dh.effective_date() <= d.end_date)
                    .max_by_key(|dh| dh.effective_date())
                    .map(crate::models::EmployeeDepartmentHistory::department_id);
                employee_departments.insert(*emp_id, last_department_id);
            }
        }

        let rule_assignments_map = self
            .build_rule_assignments_map(
                workspace_id,
                &chunk_employment_ids,
                &employment_to_employee,
                &employee_departments,
                d.start_date,
                d.end_date,
            )
            .await;

        // Build PayrollContexts — split into successes and pre-built error results
        let context_results = build_payroll_contexts(
            &chunk_employment_ids,
            &employment_to_employee,
            &emp_to_location,
            &emp_to_employments,
            &batch_dept_payroll_configs,
            &rule_assignments_map,
            &ot_base_component_ids,
            &mut BatchMaps {
                employees,
                time_map,
                roster_map,
                conditions_map,
                policies_map,
                emp_configs_map,
                dept_map,
                salary_map,
                leave_breakdowns_map,
                unpaid_holidays_map,
                month_end_policies_map,
                contracts_map,
                assignments_map,
            },
            d,
        );

        let mut results: Vec<crate::models::BatchCalculateResult> =
            Vec::with_capacity(chunk_employment_ids.len() + pre_errors.len());
        results.extend(pre_errors);
        let mut ok_contexts: Vec<(Uuid, PayrollContext)> = Vec::new();

        for (employment_id, result) in context_results {
            match result {
                Ok(ctx) => ok_contexts.push((employment_id, ctx)),
                Err(error_result) => results.push(error_result),
            }
        }

        // Run concurrent calculations for successfully built contexts
        let calculated: Vec<crate::models::BatchCalculateResult> = stream::iter(ok_contexts)
            .map(|(employment_id, ctx)| {
                let emp_id = ctx.employee_id;
                let svc = self.clone_service();
                async move {
                    match svc.calculate_payslip_with_context(ctx).await {
                        Ok(payslip) => crate::models::BatchCalculateResult {
                            employment_id,
                            employee_id: emp_id,
                            status: "success".to_string(),
                            payslip: Some(payslip),
                            error_message: None,
                        },
                        Err(status) => crate::models::BatchCalculateResult {
                            employment_id,
                            employee_id: emp_id,
                            status: "error".to_string(),
                            payslip: None,
                            error_message: Some(format!("Calculation error: {status}")),
                        },
                    }
                }
            })
            .buffer_unordered(MAX_CONCURRENT_PAYROLL)
            .collect()
            .await;

        results.extend(calculated);
        results
    }
}
