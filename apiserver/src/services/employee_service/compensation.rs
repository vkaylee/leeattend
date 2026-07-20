use super::EmployeeService;
use crate::errors::AppError;
use crate::models::{
    CompensationChangeDetails, CompensationEmploymentContext, CompensationTimelineEvent,
    SalaryComponentWithDefinition,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

impl EmployeeService {
    #[allow(clippy::too_many_lines)]
    #[tracing::instrument(skip_all, err)]
    pub async fn get_compensation_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        filter_employment_id: Option<Uuid>,
    ) -> Result<Vec<CompensationTimelineEvent>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let payroll_repo = self.payroll_repo.as_ref().ok_or_else(|| {
            AppError::Internal(
                "Payroll repository is not configured on EmployeeService".to_string(),
            )
        })?;
        let employer_repo = self.employer_repo.as_ref().ok_or_else(|| {
            AppError::Internal(
                "Employer repository is not configured on EmployeeService".to_string(),
            )
        })?;

        // 1. Fetch all employments for this employee
        let mut employments = self
            .employment_repo
            .list_employee_employments(auth_token, employee_id)
            .await?;

        // 2. Filter employments if requested
        if let Some(target_id) = filter_employment_id {
            employments.retain(|e| e.id() == target_id);
        }

        if employments.is_empty() {
            return Ok(Vec::new());
        }

        let employment_ids: Vec<Uuid> = employments
            .iter()
            .map(crate::models::employment::Employment::id)
            .collect();

        // 3. Fetch all salary component changes for these employments
        let components_history = payroll_repo
            .get_salary_components_history_for_employments(workspace_id, &employment_ids)
            .await?;

        // 4. Resolve employer names with a single batch query (no N+1)
        let mut employer_ids: Vec<Uuid> = employments
            .iter()
            .map(crate::models::employment::Employment::employer_id)
            .collect();
        employer_ids.sort();
        employer_ids.dedup();

        let employers = employer_repo
            .get_employers_by_ids(auth_token, &employer_ids)
            .await?;
        let employer_names: std::collections::HashMap<Uuid, String> = employers
            .into_iter()
            .map(|e| (e.id, e.name))
            .collect();

        // 5. Resolve department names with a single batch query (no N+1)
        let mut dept_ids: Vec<Uuid> = employments
            .iter()
            .filter_map(crate::models::employment::Employment::department_id)
            .collect();
        dept_ids.sort();
        dept_ids.dedup();

        let departments = self
            .dept_repo
            .get_departments_by_ids(auth_token, &dept_ids)
            .await?;
        let department_names: std::collections::HashMap<Uuid, String> = departments
            .into_iter()
            .map(|d| (d.id(), d.name().to_string()))
            .collect();

        // Map employments by ID for quick lookup
        let employments_map: std::collections::HashMap<
            Uuid,
            &crate::models::employment::Employment,
        > = employments.iter().map(|e| (e.id(), e)).collect();

        // Identify distinct chronological keys (employment_id, effective_date)
        let mut timeline_keys: Vec<(Uuid, NaiveDate)> = components_history
            .iter()
            .map(|(esc, _)| (esc.employment_id, esc.effective_date))
            .collect();
        timeline_keys.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        timeline_keys.dedup();

        let mut active_components: std::collections::HashMap<
            Uuid,
            std::collections::HashMap<
                Uuid,
                (
                    crate::models::EmployeeSalaryComponent,
                    crate::models::SalaryComponentDefinition,
                ),
            >,
        > = std::collections::HashMap::new();

        let mut timeline_events = Vec::new();
        let mut previous_total: Option<Decimal> = None;
        let mut previous_employment_id: Option<Uuid> = None;

        for (emp_id, eff_date) in timeline_keys {
            let Some(emp) = employments_map.get(&emp_id) else {
                continue;
            };

            let emp_components = active_components.entry(emp_id).or_default();

            for (esc, scd) in &components_history {
                if esc.employment_id == emp_id && esc.effective_date == eff_date {
                    emp_components.insert(esc.component_definition_id, (esc.clone(), scd.clone()));
                }
            }

            let mut event_components: Vec<SalaryComponentWithDefinition> = emp_components
                .values()
                .map(|(esc, scd)| SalaryComponentWithDefinition {
                    component: esc.clone(),
                    definition: scd.clone(),
                })
                .collect();

            event_components.sort_by(|a, b| a.definition.code.cmp(&b.definition.code));

            let total_compensation: Decimal =
                event_components.iter().map(|c| c.component.amount).sum();

            let change_type = if previous_total.is_none() {
                "initial".to_string()
            } else if Some(emp_id) != previous_employment_id {
                "transfer".to_string()
            } else {
                "adjustment".to_string()
            };

            let delta = match previous_total {
                Some(prev) => total_compensation - prev,
                None => total_compensation,
            };

            let employer_name = employer_names
                .get(&emp.employer_id())
                .cloned()
                .unwrap_or_else(|| "Unknown Employer".to_string());
            let department_name = emp
                .department_id()
                .and_then(|id| department_names.get(&id).cloned());

            let event = CompensationTimelineEvent {
                effective_date: eff_date,
                employment: CompensationEmploymentContext {
                    id: emp_id,
                    employer_name,
                    department_name,
                    job_title: emp.job_title().map(std::string::ToString::to_string),
                },
                components: event_components,
                total_compensation,
                change_type,
                change_details: CompensationChangeDetails {
                    previous_total,
                    delta,
                },
            };

            timeline_events.push(event);

            previous_total = Some(total_compensation);
            previous_employment_id = Some(emp_id);
        }

        // Return newest first
        timeline_events.reverse();

        Ok(timeline_events)
    }
}
