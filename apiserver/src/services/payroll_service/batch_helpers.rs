use super::*;

impl PayrollService {
    pub(super) async fn fetch_overtime_data(
        &self,
        workspace_id: Uuid,
    ) -> crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
        match self
            .overtime_policy_repo
            .get_workspace_overtime_data(workspace_id)
            .await
        {
            Ok(data) => data,
            Err(e) => {
                tracing::error!(workspace_id = %workspace_id, error = %e, "Failed to fetch overtime data — proceeding empty");
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                    policies: std::collections::HashMap::new(),
                    assignments: Vec::new(),
                }
            }
        }
    }

    pub(super) async fn fetch_holidays(
        &self,
        token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
        workspace_id: Uuid,
    ) -> HashMap<NaiveDate, crate::models::Holiday> {
        match self
            .holiday_repo
            .get_holidays_in_range(token, start_date, end_date)
            .await
        {
            Ok(map) => map,
            Err(e) => {
                tracing::error!(workspace_id = %workspace_id, error = %e, "Failed to fetch holidays — proceeding empty");
                std::collections::HashMap::new()
            }
        }
    }

    pub(super) async fn fetch_incentive_policies_with_tiers(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<IncentivePolicyWithTiers> {
        let active_policies = match self
            .payroll_repo
            .get_active_incentive_policies_for_period(workspace_id, start_date, end_date)
            .await
        {
            Ok(policies) => policies,
            Err(e) => {
                tracing::error!(workspace_id = %workspace_id, error = %e, "Failed to fetch incentive policies");
                return Vec::new();
            }
        };
        let mut result = Vec::new();
        for policy in active_policies {
            if let Ok(tiers) = self
                .payroll_repo
                .get_policy_tiers(workspace_id, policy.id())
                .await
            {
                result.push(IncentivePolicyWithTiers { policy, tiers });
            }
        }
        result
    }

    pub(super) async fn resolve_ot_base_component_ids(
        &self,
        workspace_id: Uuid,
        salary_base_groups: &[crate::models::SalaryBaseGroup],
    ) -> std::collections::HashSet<Uuid> {
        let ot_group_id = salary_base_groups
            .iter()
            .find(|g| g.slug == "ot_base")
            .map(|g| g.id);
        let mut ids = std::collections::HashSet::new();
        if let Some(group_id) = ot_group_id {
            if let Ok(members) = self.salary_base_repo.get_group_members(workspace_id).await {
                for member in members {
                    if member.base_group_id == group_id {
                        ids.insert(member.component_definition_id);
                    }
                }
            }
        }
        ids
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn build_rule_assignments_map(
        &self,
        workspace_id: Uuid,
        chunk_employment_ids: &[Uuid],
        employment_to_employee: &HashMap<Uuid, Uuid>,
        employee_departments: &HashMap<Uuid, Option<Uuid>>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> HashMap<Uuid, Vec<RuleAssignmentWithTiers>> {
        let resolved_assignments_map = self
            .payroll_repo
            .get_resolved_rule_assignments_batch(
                workspace_id,
                &employee_departments.keys().copied().collect::<Vec<_>>(),
                employee_departments,
                start_date,
                end_date,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    employee_count = employee_departments.len(),
                    error = %e,
                    "build_rule_assignments_map: get_resolved_rule_assignments_batch failed — returning empty"
                );
                HashMap::new()
            });

        let mut rule_def_ids: Vec<Uuid> = resolved_assignments_map
            .values()
            .flat_map(|assignments| assignments.iter().map(|a| a.rule_definition_id))
            .collect();
        rule_def_ids.sort();
        rule_def_ids.dedup();

        let rule_tiers_map = self
            .payroll_repo
            .get_payroll_rule_tiers_batch(workspace_id, &rule_def_ids)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    rule_def_count = rule_def_ids.len(),
                    error = %e,
                    "build_rule_assignments_map: get_payroll_rule_tiers_batch failed — returning empty"
                );
                HashMap::new()
            });

        let mut rule_assignments_map = HashMap::new();
        for employment_id in chunk_employment_ids {
            if let Some(emp_id) = employment_to_employee.get(employment_id) {
                if let Some(assignments) = resolved_assignments_map.get(emp_id) {
                    for assignment in assignments {
                        let tiers = rule_tiers_map
                            .get(&assignment.rule_definition_id)
                            .cloned()
                            .unwrap_or_default();
                        rule_assignments_map
                            .entry(*employment_id)
                            .or_insert_with(Vec::new)
                            .push(RuleAssignmentWithTiers {
                                assignment: assignment.clone(),
                                tiers,
                            });
                    }
                }
            }
        }
        rule_assignments_map
    }

    pub(super) fn clone_service(&self) -> Self {
        Self {
            payroll_repo: Arc::clone(&self.payroll_repo),
            department_repo: Arc::clone(&self.department_repo),
            time_entry_repo: Arc::clone(&self.time_entry_repo),
            workspace_repo: Arc::clone(&self.workspace_repo),
            holiday_repo: Arc::clone(&self.holiday_repo),
            roster_repo: Arc::clone(&self.roster_repo),
            leave_repo: Arc::clone(&self.leave_repo),
            policy_repo: Arc::clone(&self.policy_repo),
            condition_type_repo: Arc::clone(&self.condition_type_repo),
            employee_repo: Arc::clone(&self.employee_repo),
            overtime_policy_repo: Arc::clone(&self.overtime_policy_repo),
            salary_base_repo: Arc::clone(&self.salary_base_repo),
            roster_provider: Arc::clone(&self.roster_provider),
            employment_repo: Arc::clone(&self.employment_repo),
        }
    }
}
