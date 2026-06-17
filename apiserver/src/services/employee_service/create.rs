use super::EmployeeService;
use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use crate::models::{CreateEmployeeRequest, Employee};
use crate::utils::id::new_id;
use chrono::Utc;

impl EmployeeService {
    #[tracing::instrument(skip_all, err)]
    pub async fn create_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: CreateEmployeeRequest,
    ) -> Result<Employee, AppError> {
        let workspace_id = auth_token.workspace_id();
        // Optional: Check if user_id exists if provided
        if let Some(user_id) = req.user_id {
            self.user_repo
                .get_user_by_id(user_id)
                .await?
                .ok_or(AppError::validation(I18nKey::UserNotFound))?;

            // Enforce 1-to-1 User/Employee link in this workspace
            if self
                .repo
                .get_employee_id_by_user_id(auth_token, user_id)
                .await?
                .is_some()
            {
                return Err(AppError::validation(I18nKey::UserAlreadyLinked));
            }
        }

        let employee = Employee::new(
            new_id(),
            workspace_id,
            req.user_id,
            req.full_name,
            req.employee_id_number.into_inner(),
            req.tags.unwrap_or_default(),
            Utc::now(),
            Utc::now(),
            None,
        );

        // Use repository to create employee and set recruitment source transactionally
        self.repo
            .create_employee_transactional(
                auth_token,
                &employee,
                req.hire_date,
                req.recruitment_source_id,
                req.employer_id,
                req.primary_location_id,
                req.department_id,
            )
            .await?;

        // Claim staged logs asynchronously (best-effort, non-fatal if fails)
        if let Err(e) = self
            .staging_repo
            .claim_staged_logs(workspace_id, employee.employee_id_number(), employee.id())
            .await
        {
            tracing::warn!(
                "Failed to claim staged logs for newly created employee {}: {}",
                employee.id(),
                e
            );
        }

        // Trigger onboarding checklist if lifecycle service is registered
        if let Some(ref lifecycle_service) = self.lifecycle_service {
            if let Ok(employments) = self
                .employment_repo
                .list_employee_employments(auth_token, employee.id())
                .await
            {
                if let Some(emp) = employments.first() {
                    if let Err(e) = lifecycle_service
                        .auto_trigger_onboarding(workspace_id, emp.id(), None)
                        .await
                    {
                        tracing::warn!(
                            "Failed to auto-trigger onboarding checklist for employee {}: {:?}",
                            employee.id(),
                            e
                        );
                    }
                }
            }
        }

        Ok(employee)
    }
}
