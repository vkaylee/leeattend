use crate::errors::AppError;
use crate::models::{OvertimePolicy, OvertimeRule};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: OvertimePolicyRepositoryTrait — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait OvertimePolicyRepositoryTrait: Send + Sync {
    async fn create_policy_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<OvertimePolicy, AppError>;
    async fn get_policy_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<OvertimePolicy>, AppError>;
    async fn get_policy_with_rules(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<(OvertimePolicy, Vec<OvertimeRule>)>, AppError>;
    async fn add_rule(
        &self,
        workspace_id: Uuid,
        rule: &OvertimeRule,
    ) -> Result<OvertimeRule, AppError>;
    async fn resolve_policy_for_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Uuid>, AppError>;
    async fn create_policy_assignment(
        &self,
        workspace_id: Uuid,
        assignee_type: &str,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
    ) -> Result<(), AppError>;

    async fn get_workspace_overtime_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<WorkspaceOvertimeData, AppError>;
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceOvertimeData {
    pub policies: std::collections::HashMap<
        Uuid,
        (
            crate::models::OvertimePolicy,
            Vec<crate::models::OvertimeRule>,
        ),
    >,
    pub assignments: Vec<crate::models::OvertimePolicyAssignment>,
}

impl WorkspaceOvertimeData {
    pub fn resolve_policy_for_employee(
        &self,
        employee_id: Uuid,
        date: NaiveDate,
        all_dept_history: &[crate::models::EmployeeDepartmentHistory],
    ) -> Option<Uuid> {
        // 1. Employee Override
        let emp_assignment = self
            .assignments
            .iter()
            .filter(|a| {
                a.assignee_type() == "employee"
                    && a.assignee_id() == Some(employee_id)
                    && a.effective_from() <= date
                    && a.effective_to().is_none_or(|to| to >= date)
            })
            .max_by_key(|a| a.created_at());
        if let Some(a) = emp_assignment {
            return Some(a.policy_id());
        }

        // 2. Department Override
        let dept_assignment = all_dept_history
            .iter()
            .filter(|h| h.effective_date() <= date)
            .filter_map(|h| {
                self.assignments
                    .iter()
                    .filter(|a| {
                        a.assignee_type() == "department"
                            && a.assignee_id() == Some(h.department_id())
                            && a.effective_from() <= date
                            && a.effective_to().is_none_or(|to| to >= date)
                    })
                    .max_by_key(|a| a.created_at())
                    .map(|a| (h.effective_date(), a.created_at(), a.policy_id()))
            })
            .max_by_key(|(h_eff, a_created, _)| (*h_eff, *a_created));

        if let Some((_, _, policy_id)) = dept_assignment {
            return Some(policy_id);
        }

        // 3. Workspace Default
        let ws_assignment = self
            .assignments
            .iter()
            .filter(|a| {
                a.assignee_type() == "workspace"
                    && a.effective_from() <= date
                    && a.effective_to().is_none_or(|to| to >= date)
            })
            .max_by_key(|a| a.created_at());
        if let Some(a) = ws_assignment {
            return Some(a.policy_id());
        }

        None
    }
}
