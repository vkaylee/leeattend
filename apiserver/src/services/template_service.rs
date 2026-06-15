use crate::errors::AppError;
use crate::repositories::{
    EmployeeContractRepository, EmployeeRepository, EmploymentRepository, WorkspaceRepository,
};
use regex::Regex;
use std::collections::HashMap;
use uuid::Uuid;

pub struct TemplateService;

impl TemplateService {
    /// Takes a template string containing variables like `{{EMPLOYEE_NAME}}`
    /// and a HashMap of variable names to their values, and returns the
    /// filled template string.
    pub fn fill_template(
        template: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, AppError> {
        // Regex to match {{VAR_NAME}}
        let re = Regex::new(r"\{\{([A-Z0-9_]+)\}\}").map_err(|e| {
            AppError::Internal(format!("Failed to compile regex for templates: {e}"))
        })?;

        let result = re.replace_all(template, |caps: &regex::Captures| {
            let key = &caps[1];
            if let Some(val) = variables.get(key) {
                val.clone()
            } else {
                // If variable not found, leave the placeholder intact
                caps[0].to_string()
            }
        });

        Ok(result.into_owned())
    }

    /// Fetches necessary employee data and uses `fill_template` to generate the document.
    pub async fn generate_employee_document(
        workspace_id: Uuid,
        employee_id: Uuid,
        workspace_repo: &dyn WorkspaceRepository,
        employee_repo: &dyn EmployeeRepository,
        employment_repo: &dyn EmploymentRepository,
        contract_repo: &dyn EmployeeContractRepository,
        template_content: &str,
        variables_override: Option<HashMap<String, String>>,
    ) -> Result<String, AppError> {
        let mut variables = HashMap::new();

        // 1. Fetch Workspace
        if let Ok(Some(workspace)) = workspace_repo.get_workspace_by_id(workspace_id).await {
            variables.insert("COMPANY_NAME".to_string(), workspace.name().to_string());
        } else {
            variables.insert("COMPANY_NAME".to_string(), "LeeAttend Corp".to_string());
        }

        // 2. Fetch Employee
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);
        if let Ok(Some(employee)) = employee_repo
            .get_employee_by_id(employee_id, &auth_token)
            .await
        {
            variables.insert(
                "EMPLOYEE_NAME".to_string(),
                employee.full_name().to_string(),
            );
            variables.insert(
                "EMPLOYEE_ID_NUMBER".to_string(),
                employee.employee_id_number().to_string(),
            );
        }

        // 3. Fetch Employment (Active on today or fallback to first employment in list)
        let today = chrono::Utc::now().date_naive();
        let employment = if let Ok(Some(emp)) = employment_repo
            .get_active_employment_on_date(&auth_token, employee_id, today)
            .await
        {
            Some(emp)
        } else if let Ok(employments) = employment_repo
            .list_employee_employments(&auth_token, employee_id)
            .await
        {
            employments.into_iter().next()
        } else {
            None
        };

        if let Some(emp) = employment {
            variables.insert(
                "HIRE_DATE".to_string(),
                emp.start_date().format("%d/%m/%Y").to_string(),
            );
            if let Some(title) = emp.job_title() {
                variables.insert("JOB_TITLE".to_string(), title.to_string());
            }

            // 4. Fetch Contract (Active/First contract for this employment)
            if let Ok(contracts) = contract_repo
                .list_employee_contracts(&auth_token, emp.id())
                .await
            {
                if let Some(contract) = contracts.first() {
                    if let Some(num) = contract.contract_number() {
                        variables.insert("CONTRACT_NUMBER".to_string(), num.to_string());
                    }
                }
            }
        }

        // Default fallback if not already populated
        variables
            .entry("DOCUMENT_DATE".to_string())
            .or_insert_with(|| chrono::Utc::now().format("%d/%m/%Y").to_string());

        // Merge overrides
        if let Some(overrides) = variables_override {
            for (k, v) in overrides {
                variables.insert(k, v);
            }
        }

        Self::fill_template(template_content, &variables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_template() {
        let template = "Hello {{FIRST_NAME}} {{LAST_NAME}}, your salary is {{SALARY}}.";
        let mut vars = HashMap::new();
        vars.insert("FIRST_NAME".to_string(), "John".to_string());
        vars.insert("LAST_NAME".to_string(), "Doe".to_string());
        vars.insert("SALARY".to_string(), "1000".to_string());

        let result = TemplateService::fill_template(template, &vars).unwrap();
        assert_eq!(result, "Hello John Doe, your salary is 1000.");
    }

    #[test]
    fn test_missing_variable_keeps_placeholder() {
        let template = "Hello {{FIRST_NAME}}, your ID is {{MISSING_ID}}.";
        let mut vars = HashMap::new();
        vars.insert("FIRST_NAME".to_string(), "Jane".to_string());

        let result = TemplateService::fill_template(template, &vars).unwrap();
        assert_eq!(result, "Hello Jane, your ID is {{MISSING_ID}}.");
    }

    #[tokio::test]
    async fn test_generate_employee_document() {
        use crate::models::enums::{ContractStatus, ContractType};
        use crate::models::{
            contract::EmployeeContract, employment::Employment, Employee, Workspace,
        };
        use crate::repositories::employee::MockEmployeeRepository;
        use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
        use crate::repositories::employment_repository::MockEmploymentRepository;
        use crate::repositories::workspace_repository::MockWorkspaceRepository;
        use chrono::Utc;
        use rust_decimal::Decimal;

        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let emp_title = "Tech Lead".to_string();

        let mut workspace_repo = MockWorkspaceRepository::new();
        workspace_repo
            .expect_get_workspace_by_id()
            .with(mockall::predicate::eq(ws_id))
            .returning(move |_| {
                Box::pin(async move {
                    Ok(Some(Workspace::new(
                        ws_id,
                        "LeeAttend Corp Test".to_string(),
                        "dd/MM/yyyy".to_string(),
                        "HH:mm".to_string(),
                        "Asia/Ho_Chi_Minh".to_string(),
                        "#ff0000".to_string(),
                        "vi".to_string(),
                        None,
                        None,
                        "none".to_string(),
                        Decimal::new(15, 1),
                        Utc::now(),
                        None,
                    )))
                })
            });

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .with(mockall::predicate::eq(emp_id), mockall::predicate::always())
            .returning(move |_, _| {
                Ok(Some(Employee::new(
                    emp_id,
                    ws_id,
                    None,
                    "John Doe".to_string(),
                    "EMP001".to_string(),
                    vec![],
                    Utc::now(),
                    Utc::now(),
                    None,
                )))
            });

        let mut employment_repo = MockEmploymentRepository::new();
        let emp_title_clone = emp_title.clone();
        employment_repo
            .expect_get_active_employment_on_date()
            .returning(move |_, _, _| {
                let emp_title_clone = emp_title_clone.clone();
                Ok(Some(Employment::new(
                    Uuid::new_v4(),
                    ws_id,
                    emp_id,
                    Uuid::new_v4(),
                    None,
                    None,
                    "active".to_string(),
                    chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    None,
                    Some(emp_title_clone),
                    "primary".to_string(),
                    None,
                    Utc::now(),
                    Utc::now(),
                )))
            });

        let mut contract_repo = MockEmployeeContractRepository::new();
        contract_repo
            .expect_list_employee_contracts()
            .returning(move |_, _| {
                Ok(vec![EmployeeContract::new(
                    Uuid::new_v4(),
                    ws_id,
                    Uuid::new_v4(),
                    Some("CONT-123".to_string()),
                    ContractType::FixedTerm,
                    ContractStatus::Active,
                    chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    None,
                    None,
                    None,
                    Utc::now(),
                    Utc::now(),
                )])
            });

        let template = "Welcome to {{COMPANY_NAME}}, {{EMPLOYEE_NAME}} (ID: {{EMPLOYEE_ID_NUMBER}}). Your title is {{JOB_TITLE}} and your contract is {{CONTRACT_NUMBER}}.";
        let result = TemplateService::generate_employee_document(
            ws_id,
            emp_id,
            &workspace_repo,
            &employee_repo,
            &employment_repo,
            &contract_repo,
            template,
            None,
        )
        .await
        .unwrap();

        assert_eq!(
            result,
            "Welcome to LeeAttend Corp Test, John Doe (ID: EMP001). Your title is Tech Lead and your contract is CONT-123."
        );
    }

    #[tokio::test]
    async fn test_generate_employee_document_fallbacks() {
        use crate::repositories::employee::MockEmployeeRepository;
        use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
        use crate::repositories::employment_repository::MockEmploymentRepository;
        use crate::repositories::workspace_repository::MockWorkspaceRepository;

        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        // 1. Workspace get returns None (or Err) -> COMPANY_NAME fallback is "LeeAttend Corp"
        let mut workspace_repo = MockWorkspaceRepository::new();
        workspace_repo
            .expect_get_workspace_by_id()
            .returning(|_| Box::pin(async move { Ok(None) }));

        // 2. Employee get returns None -> no variables set
        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .returning(|_, _| Ok(None));

        // 3. Employment active check returns None -> list check returns empty -> no variables set
        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo
            .expect_get_active_employment_on_date()
            .returning(|_, _, _| Ok(None));
        employment_repo
            .expect_list_employee_employments()
            .returning(|_, _| Ok(vec![]));

        let contract_repo = MockEmployeeContractRepository::new();

        let template =
            "Company: {{COMPANY_NAME}} | Employee: {{EMPLOYEE_NAME}} | Date: {{DOCUMENT_DATE}}";
        let result = TemplateService::generate_employee_document(
            ws_id,
            emp_id,
            &workspace_repo,
            &employee_repo,
            &employment_repo,
            &contract_repo,
            template,
            None,
        )
        .await
        .unwrap();

        assert!(result.contains("Company: LeeAttend Corp"));
        assert!(result.contains("Employee: {{EMPLOYEE_NAME}}"));
    }
}
