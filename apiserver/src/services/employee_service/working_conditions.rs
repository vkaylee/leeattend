use super::EmployeeService;
use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use crate::models::{CreateWorkingConditionRequest, WorkingCondition};
use crate::utils::id::new_id;
use uuid::Uuid;

impl EmployeeService {
    #[tracing::instrument(skip_all, err)]
    pub async fn get_working_conditions(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkingCondition>, i64), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let emp = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::not_found(I18nKey::EmployeeNotFound))?;

        let active_employment_id = emp
            .active_employment_id()
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

        self.repo
            .get_working_conditions(auth_token, active_employment_id, page, limit)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn create_working_condition(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: CreateWorkingConditionRequest,
    ) -> Result<WorkingCondition, AppError> {
        let _workspace_id = auth_token.workspace_id();
        let emp = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::not_found(I18nKey::EmployeeNotFound))?;

        let active_employment_id = emp
            .active_employment_id()
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

        let condition = WorkingCondition::new(
            new_id(),
            active_employment_id,
            req.type_,
            req.minute_allowance,
            req.is_paid,
            req.start_date,
            req.end_date,
        );

        self.repo
            .create_working_condition(auth_token, &condition)
            .await?;

        Ok(condition)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_working_condition(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        condition_id: Uuid,
    ) -> Result<(), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let emp = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::not_found(I18nKey::EmployeeNotFound))?;

        let active_employment_id = emp
            .active_employment_id()
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

        let deleted = self
            .repo
            .delete_working_condition(auth_token, condition_id, active_employment_id)
            .await?;

        if !deleted {
            return Err(AppError::not_found(I18nKey::NotFound));
        }
        Ok(())
    }
}
