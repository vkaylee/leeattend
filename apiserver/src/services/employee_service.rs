use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    CreateEmployeeRequest, CreateWorkingConditionRequest, Employee, EmploymentPeriod,
    RehireEmployeeRequest, UpdateEmployeeRequest, WorkingCondition,
};
use crate::repositories::{employee, user_repository, EmployeeRepository};
use crate::utils::normalize_employee_id;

pub struct EmployeeService {
    repo: Arc<dyn EmployeeRepository>,
    /// Kept for operations not yet abstracted (transactions, user_repository, etc.)
    pool: PgPool,
}

impl EmployeeService {
    pub fn new(repo: Arc<dyn EmployeeRepository>, pool: PgPool) -> Self {
        Self { repo, pool }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_working_conditions(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        let emp = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        self.repo
            .get_working_conditions(workspace_id, emp.id)
            .await
            .map_err(AppError::Sqlx)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn create_working_condition(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        req: CreateWorkingConditionRequest,
    ) -> Result<WorkingCondition, AppError> {
        let emp = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        let condition = WorkingCondition {
            id: Uuid::new_v4(),
            employee_id: emp.id,
            type_: req.type_,
            minute_allowance: req.minute_allowance,
            is_paid: req.is_paid,
            start_date: req.start_date,
            end_date: req.end_date,
        };

        self.repo
            .create_working_condition(&condition)
            .await
            .map_err(AppError::Sqlx)?;

        Ok(condition)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_working_condition(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        condition_id: Uuid,
    ) -> Result<(), AppError> {
        let emp = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        let deleted = self
            .repo
            .delete_working_condition(condition_id, emp.id)
            .await
            .map_err(AppError::Sqlx)?;

        if !deleted {
            return Err(AppError::NotFound("Working condition not found".into()));
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn create_employee(
        &self,
        workspace_id: Uuid,
        req: CreateEmployeeRequest,
    ) -> Result<Employee, AppError> {
        // Optional: Check if user_id exists if provided
        if let Some(user_id) = req.user_id {
            user_repository::get_user_by_id(&self.pool, user_id)
                .await?
                .ok_or(AppError::Validation("User not found".into()))?;
        }

        let employee = Employee {
            id: Uuid::new_v4(),
            workspace_id,
            user_id: req.user_id,
            full_name: req.full_name,
            employee_id_number: normalize_employee_id(&req.employee_id_number),
            hire_date: req.hire_date,
            termination_date: None,
            tags: req.tags.unwrap_or_default(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            department_name: None,
        };

        // Transaction-based creation still uses pool directly
        let mut tx = self.pool.begin().await.map_err(AppError::Sqlx)?;
        employee::create_employee_tx(&mut tx, &employee)
            .await
            .map_err(AppError::Sqlx)?;
        tx.commit().await.map_err(AppError::Sqlx)?;

        // Set recruitment source if provided
        if let Some(source_id) = req.recruitment_source_id {
            self.repo
                .set_employee_recruitment_source(employee.id, Some(source_id))
                .await
                .map_err(AppError::Sqlx)?;
        }

        // Claim staged logs directly for the new employee
        if let Err(e) = crate::repositories::staging_log_repository::claim_staged_logs_direct(
            &self.pool,
            workspace_id,
            employee.id,
            &employee.employee_id_number,
        )
        .await
        {
            tracing::warn!(
                "Failed to claim staged logs for new employee {}: {}",
                employee.id,
                e
            );
        }

        Ok(employee)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn rehire_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        req: RehireEmployeeRequest,
    ) -> Result<Employee, AppError> {
        let employee = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        if employee.is_active {
            return Err(AppError::Validation("Employee is already active".into()));
        }

        if let Some(term_date) = employee.termination_date {
            if req.hire_date <= term_date {
                return Err(AppError::Validation(
                    "Rehire date must be after termination date".into(),
                ));
            }
        }

        let tx = self.pool.begin().await.map_err(AppError::Sqlx)?;

        let update_req = UpdateEmployeeRequest {
            user_id: None,
            full_name: None,
            employee_id_number: None,
            hire_date: None,
            termination_date: None,
            tags: None,
            is_active: Some(true),
            max_concurrent_workers: None,
            recruitment_source_id: None,
        };

        self.repo
            .update_employee(employee_id, workspace_id, &update_req)
            .await
            .map_err(AppError::Sqlx)?;

        let period = EmploymentPeriod {
            id: Uuid::new_v4(),
            employee_id,
            hire_date: req.hire_date,
            termination_date: None,
            is_rehire: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo
            .create_employment_period(&period)
            .await
            .map_err(AppError::Sqlx)?;

        tx.commit().await.map_err(AppError::Sqlx)?;

        self.repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Employee not found after rehire".into()))
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employment_history(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<EmploymentPeriod>, AppError> {
        let _ = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        self.repo
            .get_employment_history(workspace_id, employee_id)
            .await
            .map_err(AppError::Sqlx)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Employee, AppError> {
        self.repo
            .get_employee_by_id(id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_all_employees(
        &self,
        workspace_id: Uuid,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        self.repo
            .get_all_employees(workspace_id, options)
            .await
            .map_err(AppError::Sqlx)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: UpdateEmployeeRequest,
    ) -> Result<Employee, AppError> {
        // Optional: Check if user_id exists if provided
        if let Some(user_id) = req.user_id {
            user_repository::get_user_by_id(&self.pool, user_id)
                .await?
                .ok_or(AppError::Validation("User not found".into()))?;
        }

        let employee = self
            .repo
            .update_employee(id, workspace_id, &req)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        // Update recruitment source if provided
        if req.recruitment_source_id.is_some() {
            self.repo
                .set_employee_recruitment_source(id, req.recruitment_source_id)
                .await
                .map_err(AppError::Sqlx)?;
        }

        // If employee_id_number was updated, claim staged logs directly
        if let Some(new_id_number) = req.employee_id_number {
            let normalized = normalize_employee_id(&new_id_number);
            if let Err(e) = crate::repositories::staging_log_repository::claim_staged_logs_direct(
                &self.pool,
                workspace_id,
                id,
                &normalized,
            )
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

    // ── Recruitment Source Service Methods ──────────────────────────

    #[tracing::instrument(skip_all, err)]
    pub async fn create_recruitment_source(
        &self,
        workspace_id: Uuid,
        req: crate::models::CreateRecruitmentSourceRequest,
    ) -> Result<crate::models::RecruitmentSource, AppError> {
        let source = crate::models::RecruitmentSource {
            id: Uuid::new_v4(),
            workspace_id,
            source_type: req.source_type,
            name: req.name,
            referrer_employee_id: req.referrer_employee_id,
            referral_bonus_amount: req.referral_bonus_amount,
            referral_bonus_paid: false,
            agency_contact_name: req.agency_contact_name,
            agency_contact_email: req.agency_contact_email,
            agency_fee_amount: req.agency_fee_amount,
            agency_fee_paid: false,
            notes: req.notes,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo
            .create_recruitment_source(&source)
            .await
            .map_err(AppError::Sqlx)?;
        Ok(source)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_recruitment_sources(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::RecruitmentSource>, AppError> {
        self.repo
            .get_recruitment_sources(workspace_id)
            .await
            .map_err(AppError::Sqlx)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<crate::models::RecruitmentSource, AppError> {
        self.repo
            .update_recruitment_source(id, workspace_id, &req)
            .await?
            .ok_or(AppError::NotFound("Recruitment source not found".into()))
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<(), AppError> {
        let rows = self
            .repo
            .delete_recruitment_source(id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?;
        if rows == 0 {
            return Err(AppError::NotFound("Recruitment source not found".into()));
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_recruitment_source_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::RecruitmentSourceBreakdown>, AppError> {
        self.repo
            .get_recruitment_source_breakdown(workspace_id)
            .await
            .map_err(AppError::Sqlx)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_employee(&self, id: Uuid, workspace_id: Uuid) -> Result<(), AppError> {
        let rows_affected = self
            .repo
            .delete_employee(id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?;

        if rows_affected == 0 {
            Err(AppError::NotFound("Employee not found".into()))
        } else {
            Ok(())
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_department_history(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeDepartmentHistory>, AppError> {
        let _ = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        crate::repositories::department_repository::get_employee_department_history(
            &self.pool,
            workspace_id,
            employee_id,
        )
        .await
        .map_err(AppError::Sqlx)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn transfer_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        req: crate::models::TransferEmployeeRequest,
    ) -> Result<crate::models::EmployeeDepartmentHistory, AppError> {
        let _ = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        crate::repositories::department_repository::transfer_employee(
            &self.pool,
            employee_id,
            req.department_id,
            req.effective_date,
            req.notes,
        )
        .await
        .map_err(AppError::Sqlx)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_turnover_analytics(
        &self,
        workspace_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<crate::models::TurnoverAnalytics, AppError> {
        self.repo
            .get_turnover_analytics(workspace_id, start_date, end_date)
            .await
            .map_err(AppError::Sqlx)
    }
}
