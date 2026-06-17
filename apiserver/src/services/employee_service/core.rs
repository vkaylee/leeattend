use super::EmployeeService;
use crate::errors::AppError;
use crate::models::{Employee, UpdateEmployeeRequest};
use uuid::Uuid;

impl EmployeeService {
    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Employee, AppError> {
        let _workspace_id = auth_token.workspace_id();
        self.repo
            .get_employee_by_id(id, auth_token)
            .await?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_all_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        let _workspace_id = auth_token.workspace_id();
        self.repo.get_all_employees(auth_token, options).await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: UpdateEmployeeRequest,
    ) -> Result<Employee, AppError> {
        let workspace_id = auth_token.workspace_id();
        // Optional: Check if user_id exists if provided, and check duplicate
        if let Some(Some(user_id)) = req.user_id {
            self.user_repo
                .get_user_by_id(user_id)
                .await?
                .ok_or(AppError::validation(crate::utils::i18n::I18nKey::UserNotFound))?;

            // Enforce 1-to-1 User/Employee link in this workspace
            if let Some(existing_emp_id) = self
                .repo
                .get_employee_id_by_user_id(auth_token, user_id)
                .await?
            {
                if existing_emp_id != id && req.force_transfer != Some(true) {
                    let existing_emp = self
                        .repo
                        .get_employee_by_id(existing_emp_id, auth_token)
                        .await?
                        .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;

                    return Err(AppError::validation_args(
                        crate::utils::i18n::I18nKey::UserAlreadyLinked,
                        [("name", existing_emp.full_name())],
                    ));
                }
            }
        }

        let employee = self
            .repo
            .update_employee_transactional(id, auth_token, &req)
            .await?
            .ok_or(AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;

        // If employee_id_number was updated, claim staged logs directly
        if let Some(new_id_number) = req.employee_id_number {
            let normalized = new_id_number.into_inner();
            if let Err(e) = self
                .staging_repo
                .claim_staged_logs(workspace_id, &normalized, id)
                .await
            {
                tracing::warn!(
                    "Failed to claim staged logs for updated employee {}: {}",
                    id,
                    e
                );
            }
        }

        Ok(employee)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<(), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let rows_affected = self.repo.delete_employee(id, auth_token).await?;

        if rows_affected == 0 {
            Err(AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))
        } else {
            Ok(())
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_department_history(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<crate::models::EmployeeDepartmentHistory>, AppError> {
        let _workspace_id = auth_token.workspace_id();
        let _ = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;

        self.dept_repo
            .get_employee_department_history(auth_token, employee_id)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn transfer_employee(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: crate::models::TransferEmployeeRequest,
    ) -> Result<crate::models::EmployeeDepartmentHistory, AppError> {
        let _workspace_id = auth_token.workspace_id();
        let _ = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;

        self.dept_repo
            .transfer_employee(
                auth_token,
                employee_id,
                req.department_id,
                req.effective_date,
                req.notes,
            )
            .await
    }
}
