use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::id::new_id;

use crate::errors::AppError;
use crate::models::{
    CreateEmployeeRequest, CreateWorkingConditionRequest, Employee, EmploymentPeriod,
    RehireEmployeeRequest, UpdateEmployeeRequest, WorkingCondition,
};
use crate::repositories::{employee, user_repository, EmployeeRepository};

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
            id: new_id(),
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
            id: new_id(),
            workspace_id,
            user_id: req.user_id,
            full_name: req.full_name,
            employee_id_number: req.employee_id_number.into_inner(),
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
            id: new_id(),
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
            let normalized = new_id_number.into_inner();
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
            id: new_id(),
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

#[cfg(test)]
#[allow(clippy::unimplemented)]
mod tests {
    use super::*;
    use crate::models::{
        EmployeeFilterOptions, RecruitmentSource, RecruitmentSourceBreakdown, TurnoverAnalytics,
        TurnoverByDepartment, TurnoverBySource, TurnoverTenureBucket,
    };
    use crate::repositories::employee::EmployeeBasicInfo;
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};
    use rust_decimal::Decimal;
    use std::sync::Mutex;
    use uuid::Uuid;

    // ── Test Helpers ─────────────────────────────────────────────────

    fn make_employee(ws: Uuid) -> Employee {
        Employee {
            id: Uuid::new_v4(),
            workspace_id: ws,
            user_id: None,
            full_name: "Test Employee".into(),
            employee_id_number: "EMP001".into(),
            hire_date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            termination_date: None,
            tags: vec![],
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            department_name: None,
        }
    }

    fn make_working_condition(emp_id: Uuid) -> WorkingCondition {
        WorkingCondition {
            id: Uuid::new_v4(),
            employee_id: emp_id,
            type_: "late_allowance".into(),
            minute_allowance: 15,
            is_paid: true,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            end_date: None,
        }
    }

    // ── Mock Repository ──────────────────────────────────────────────

    /// Configurable mock — set the inner Option fields to control what each
    /// method returns. Defaults to "not found" / empty for all.
    struct MockEmployeeRepo {
        employee: Mutex<Option<Employee>>,
        employees: Mutex<Vec<Employee>>,
        total: Mutex<i64>,
        working_conditions: Mutex<Vec<WorkingCondition>>,
        delete_wc_result: Mutex<bool>,
        delete_emp_rows: Mutex<u64>,
        employment_history: Mutex<Vec<EmploymentPeriod>>,
        updated_employee: Mutex<Option<Employee>>,
        recruitment_sources: Mutex<Vec<RecruitmentSource>>,
        delete_rs_rows: Mutex<u64>,
        turnover: Mutex<Option<TurnoverAnalytics>>,
        updated_recruitment_source: Mutex<Option<RecruitmentSource>>,
        recruitment_source_breakdown: Mutex<Vec<RecruitmentSourceBreakdown>>,
    }

    impl MockEmployeeRepo {
        fn empty() -> Self {
            Self {
                employee: Mutex::new(None),
                employees: Mutex::new(vec![]),
                total: Mutex::new(0),
                working_conditions: Mutex::new(vec![]),
                delete_wc_result: Mutex::new(false),
                delete_emp_rows: Mutex::new(0),
                employment_history: Mutex::new(vec![]),
                updated_employee: Mutex::new(None),
                recruitment_sources: Mutex::new(vec![]),
                delete_rs_rows: Mutex::new(0),
                turnover: Mutex::new(None),
                updated_recruitment_source: Mutex::new(None),
                recruitment_source_breakdown: Mutex::new(vec![]),
            }
        }

        fn with_employee(emp: Employee) -> Self {
            let mut mock = Self::empty();
            mock.employee = Mutex::new(Some(emp));
            mock
        }
    }

    impl crate::repositories::PoolBacked for MockEmployeeRepo {
        fn from_pool(_pool: sqlx::PgPool) -> Self
        where
            Self: Sized,
        {
            Self::empty()
        }
        fn pool(&self) -> &sqlx::PgPool {
            unimplemented!("Mock does not provide a real pool")
        }
    }

    #[async_trait]
    impl EmployeeRepository for MockEmployeeRepo {
        async fn find_by_number(
            &self,
            _ws: Uuid,
            _num: &str,
        ) -> Result<Option<Employee>, sqlx::Error> {
            Ok(self.employee.lock().expect("lock").clone())
        }

        async fn get_employee_by_id(
            &self,
            _id: Uuid,
            _ws: Uuid,
        ) -> Result<Option<Employee>, sqlx::Error> {
            Ok(self.employee.lock().expect("lock").clone())
        }

        async fn get_employee_by_id_number(
            &self,
            _num: &str,
            _ws: Uuid,
        ) -> Result<Option<Employee>, sqlx::Error> {
            Ok(self.employee.lock().expect("lock").clone())
        }

        async fn get_all_employees(
            &self,
            _ws: Uuid,
            _opts: EmployeeFilterOptions,
        ) -> Result<(Vec<Employee>, i64), sqlx::Error> {
            let emps = self.employees.lock().expect("lock").clone();
            let total = *self.total.lock().expect("lock");
            Ok((emps, total))
        }

        async fn get_active_employees(&self, _ws: Uuid) -> Result<Vec<Employee>, sqlx::Error> {
            Ok(self.employees.lock().expect("lock").clone())
        }

        async fn get_active_employee_ids(
            &self,
            _ws: Uuid,
            _filter: Option<&Vec<Uuid>>,
        ) -> Result<Vec<Uuid>, sqlx::Error> {
            Ok(vec![])
        }

        async fn get_paginated_employee_ids(
            &self,
            _ws: Uuid,
            _args: crate::models::EmployeePaginationArgs<'_>,
        ) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error> {
            Ok((vec![], 0))
        }

        async fn update_employee(
            &self,
            _id: Uuid,
            _ws: Uuid,
            _req: &crate::models::UpdateEmployeeRequest,
        ) -> Result<Option<Employee>, sqlx::Error> {
            Ok(self.updated_employee.lock().expect("lock").clone())
        }

        async fn delete_employee(&self, _id: Uuid, _ws: Uuid) -> Result<u64, sqlx::Error> {
            Ok(*self.delete_emp_rows.lock().expect("lock"))
        }

        async fn get_working_conditions(
            &self,
            _ws: Uuid,
            _emp: Uuid,
        ) -> Result<Vec<WorkingCondition>, sqlx::Error> {
            Ok(self.working_conditions.lock().expect("lock").clone())
        }

        async fn create_working_condition(
            &self,
            _cond: &WorkingCondition,
        ) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn delete_working_condition(
            &self,
            _id: Uuid,
            _emp: Uuid,
        ) -> Result<bool, sqlx::Error> {
            Ok(*self.delete_wc_result.lock().expect("lock"))
        }

        async fn get_employment_history(
            &self,
            _ws: Uuid,
            _emp: Uuid,
        ) -> Result<Vec<EmploymentPeriod>, sqlx::Error> {
            Ok(self.employment_history.lock().expect("lock").clone())
        }

        async fn create_employment_period(
            &self,
            _period: &EmploymentPeriod,
        ) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn get_existing_employee_ids_by_numbers(
            &self,
            _ws: Uuid,
            _nums: &[String],
        ) -> Result<Vec<String>, sqlx::Error> {
            Ok(vec![])
        }

        async fn upsert_employee_payroll_config(
            &self,
            employee_id: Uuid,
            std_working_days: Decimal,
            hours_per_day: Decimal,
            effective_date: NaiveDate,
        ) -> Result<crate::models::EmployeePayrollConfig, sqlx::Error> {
            Ok(crate::models::EmployeePayrollConfig {
                id: Uuid::new_v4(),
                employee_id,
                std_working_days,
                hours_per_day,
                effective_date,
                created_at: Utc::now(),
            })
        }

        async fn get_employee_payroll_config_at(
            &self,
            _emp: Uuid,
            _date: NaiveDate,
        ) -> Result<Option<crate::models::EmployeePayrollConfig>, sqlx::Error> {
            Ok(None)
        }

        async fn set_employee_recruitment_source(
            &self,
            _emp: Uuid,
            _src: Option<Uuid>,
        ) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn create_recruitment_source(
            &self,
            _source: &RecruitmentSource,
        ) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn get_recruitment_sources(
            &self,
            _ws: Uuid,
        ) -> Result<Vec<RecruitmentSource>, sqlx::Error> {
            Ok(self.recruitment_sources.lock().expect("lock").clone())
        }

        async fn get_recruitment_source_by_id(
            &self,
            _id: Uuid,
            _ws: Uuid,
        ) -> Result<Option<RecruitmentSource>, sqlx::Error> {
            Ok(None)
        }

        async fn update_recruitment_source(
            &self,
            _id: Uuid,
            _ws: Uuid,
            _req: &crate::models::UpdateRecruitmentSourceRequest,
        ) -> Result<Option<RecruitmentSource>, sqlx::Error> {
            Ok(self
                .updated_recruitment_source
                .lock()
                .expect("lock")
                .clone())
        }

        async fn delete_recruitment_source(
            &self,
            _id: Uuid,
            _ws: Uuid,
        ) -> Result<u64, sqlx::Error> {
            Ok(*self.delete_rs_rows.lock().expect("lock"))
        }

        async fn get_recruitment_source_breakdown(
            &self,
            _ws: Uuid,
        ) -> Result<Vec<RecruitmentSourceBreakdown>, sqlx::Error> {
            Ok(self
                .recruitment_source_breakdown
                .lock()
                .expect("lock")
                .clone())
        }

        async fn get_turnover_analytics(
            &self,
            _ws: Uuid,
            start_date: NaiveDate,
            end_date: NaiveDate,
        ) -> Result<TurnoverAnalytics, sqlx::Error> {
            Ok(self
                .turnover
                .lock()
                .expect("lock")
                .clone()
                .unwrap_or(TurnoverAnalytics {
                    tenure_distribution: vec![],
                    by_source: vec![],
                    by_department: vec![],
                    total_terminated: 0,
                    period_start: start_date.to_string(),
                    period_end: end_date.to_string(),
                }))
        }

        async fn get_managed_departments(
            &self,
            _ws: Uuid,
            _user: Uuid,
        ) -> Result<Vec<Uuid>, sqlx::Error> {
            Ok(vec![])
        }

        async fn get_employee_id_by_user_id(
            &self,
            _ws: Uuid,
            _user: Uuid,
        ) -> Result<Option<Uuid>, sqlx::Error> {
            Ok(None)
        }
    }

    /// Helper: create a service backed by the mock (pool is unused in mock-only paths).
    fn make_service(mock: MockEmployeeRepo) -> EmployeeService {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://mock:mock@localhost/mock")
            .expect("lazy pool creation");
        EmployeeService::new(Arc::new(mock), pool)
    }

    // ── Tests: get_employee_by_id ────────────────────────────────────

    #[tokio::test]
    async fn get_employee_by_id_found() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let emp_id = emp.id;
        let svc = make_service(MockEmployeeRepo::with_employee(emp.clone()));

        let result = svc.get_employee_by_id(emp_id, ws).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("ok").id, emp_id);
    }

    #[tokio::test]
    async fn get_employee_by_id_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc.get_employee_by_id(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: get_all_employees ─────────────────────────────────────

    #[tokio::test]
    async fn get_all_employees_returns_list_and_count() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let mock = MockEmployeeRepo::empty();
        *mock.employees.lock().expect("lock") = vec![emp.clone()];
        *mock.total.lock().expect("lock") = 1;

        let svc = make_service(mock);
        let (list, total) = svc
            .get_all_employees(ws, EmployeeFilterOptions::default())
            .await
            .expect("ok");

        assert_eq!(list.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(list[0].full_name, "Test Employee");
    }

    #[tokio::test]
    async fn get_all_employees_empty() {
        let svc = make_service(MockEmployeeRepo::empty());
        let (list, total) = svc
            .get_all_employees(Uuid::new_v4(), EmployeeFilterOptions::default())
            .await
            .expect("ok");

        assert!(list.is_empty());
        assert_eq!(total, 0);
    }

    // ── Tests: get_working_conditions ────────────────────────────────

    #[tokio::test]
    async fn get_working_conditions_returns_conditions() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let cond = make_working_condition(emp.id);

        let mock = MockEmployeeRepo::with_employee(emp.clone());
        *mock.working_conditions.lock().expect("lock") = vec![cond.clone()];

        let svc = make_service(mock);
        let result = svc.get_working_conditions(emp.id, ws).await.expect("ok");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].type_, "late_allowance");
        assert_eq!(result[0].minute_allowance, 15);
    }

    #[tokio::test]
    async fn get_working_conditions_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .get_working_conditions(Uuid::new_v4(), Uuid::new_v4())
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: create_working_condition ──────────────────────────────

    #[tokio::test]
    async fn create_working_condition_success() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let svc = make_service(MockEmployeeRepo::with_employee(emp.clone()));

        let req = CreateWorkingConditionRequest {
            type_: "early_leave".into(),
            minute_allowance: 30,
            is_paid: false,
            start_date: NaiveDate::from_ymd_opt(2026, 6, 1).expect("valid date"),
            end_date: None,
        };

        let result = svc.create_working_condition(emp.id, ws, req).await;
        assert!(result.is_ok());
        let wc = result.expect("ok");
        assert_eq!(wc.type_, "early_leave");
        assert_eq!(wc.minute_allowance, 30);
        assert!(!wc.is_paid);
        assert_eq!(wc.employee_id, emp.id);
    }

    // ── Tests: delete_working_condition ──────────────────────────────

    #[tokio::test]
    async fn delete_working_condition_success() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let mock = MockEmployeeRepo::with_employee(emp.clone());
        *mock.delete_wc_result.lock().expect("lock") = true;

        let svc = make_service(mock);
        let result = svc
            .delete_working_condition(emp.id, ws, Uuid::new_v4())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_working_condition_not_found() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let mock = MockEmployeeRepo::with_employee(emp.clone());
        // delete_wc_result defaults to false

        let svc = make_service(mock);
        let result = svc
            .delete_working_condition(emp.id, ws, Uuid::new_v4())
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn delete_working_condition_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .delete_working_condition(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4())
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: delete_employee ───────────────────────────────────────

    #[tokio::test]
    async fn delete_employee_success() {
        let mock = MockEmployeeRepo::empty();
        *mock.delete_emp_rows.lock().expect("lock") = 1;
        let svc = make_service(mock);

        let result = svc.delete_employee(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc.delete_employee(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: get_employment_history ────────────────────────────────

    #[tokio::test]
    async fn get_employment_history_returns_periods() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let period = EmploymentPeriod {
            id: Uuid::new_v4(),
            employee_id: emp.id,
            hire_date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            termination_date: None,
            is_rehire: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mock = MockEmployeeRepo::with_employee(emp.clone());
        *mock.employment_history.lock().expect("lock") = vec![period];

        let svc = make_service(mock);
        let result = svc.get_employment_history(emp.id, ws).await.expect("ok");
        assert_eq!(result.len(), 1);
        assert!(!result[0].is_rehire);
    }

    #[tokio::test]
    async fn get_employment_history_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .get_employment_history(Uuid::new_v4(), Uuid::new_v4())
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: get_recruitment_sources ───────────────────────────────

    #[tokio::test]
    async fn get_recruitment_sources_returns_list() {
        let ws = Uuid::new_v4();
        let source = RecruitmentSource {
            id: Uuid::new_v4(),
            workspace_id: ws,
            source_type: crate::models::enums::RecruitmentSourceType::Direct,
            name: "LinkedIn".into(),
            referrer_employee_id: None,
            referral_bonus_amount: None,
            referral_bonus_paid: false,
            agency_contact_name: None,
            agency_contact_email: None,
            agency_fee_amount: None,
            agency_fee_paid: false,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mock = MockEmployeeRepo::empty();
        *mock.recruitment_sources.lock().expect("lock") = vec![source];

        let svc = make_service(mock);
        let result = svc.get_recruitment_sources(ws).await.expect("ok");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "LinkedIn");
    }

    // ── Tests: delete_recruitment_source ─────────────────────────────

    #[tokio::test]
    async fn delete_recruitment_source_success() {
        let mock = MockEmployeeRepo::empty();
        *mock.delete_rs_rows.lock().expect("lock") = 1;
        let svc = make_service(mock);

        let result = svc
            .delete_recruitment_source(Uuid::new_v4(), Uuid::new_v4())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_recruitment_source_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .delete_recruitment_source(Uuid::new_v4(), Uuid::new_v4())
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: update_recruitment_source ─────────────────────────────

    #[tokio::test]
    async fn update_recruitment_source_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = crate::models::UpdateRecruitmentSourceRequest {
            name: Some("Updated".into()),
            referrer_employee_id: None,
            referral_bonus_amount: None,
            referral_bonus_paid: None,
            agency_contact_name: None,
            agency_contact_email: None,
            agency_fee_amount: None,
            agency_fee_paid: None,
            notes: None,
        };
        let result = svc
            .update_recruitment_source(Uuid::new_v4(), Uuid::new_v4(), req)
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: get_turnover_analytics ────────────────────────────────

    #[tokio::test]
    async fn get_turnover_analytics_returns_data() {
        let ws = Uuid::new_v4();
        let analytics = TurnoverAnalytics {
            tenure_distribution: vec![TurnoverTenureBucket {
                bucket: "0-6 months".into(),
                count: 5,
            }],
            by_source: vec![TurnoverBySource {
                source_type: "direct".into(),
                source_name: "Walk-in".into(),
                count: 3,
            }],
            by_department: vec![TurnoverByDepartment {
                department_name: "Engineering".into(),
                count: 2,
            }],
            total_terminated: 5,
            period_start: "2026-01-01".into(),
            period_end: "2026-06-30".into(),
        };

        let mock = MockEmployeeRepo::empty();
        *mock.turnover.lock().expect("lock") = Some(analytics);

        let svc = make_service(mock);
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date");
        let end = NaiveDate::from_ymd_opt(2026, 6, 30).expect("valid date");

        let result = svc
            .get_turnover_analytics(ws, start, end)
            .await
            .expect("ok");
        assert_eq!(result.total_terminated, 5);
        assert_eq!(result.tenure_distribution.len(), 1);
        assert_eq!(result.by_source.len(), 1);
        assert_eq!(result.by_department[0].department_name, "Engineering");
    }
    // ── Additional Tests for Coverage ──────────────────────────────

    #[tokio::test]
    async fn create_working_condition_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = CreateWorkingConditionRequest {
            type_: "early_leave".into(),
            minute_allowance: 30,
            is_paid: false,
            start_date: NaiveDate::from_ymd_opt(2026, 6, 1).expect("valid date"),
            end_date: None,
        };
        let result = svc
            .create_working_condition(Uuid::new_v4(), Uuid::new_v4(), req)
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn create_recruitment_source_success() {
        let ws = Uuid::new_v4();
        let svc = make_service(MockEmployeeRepo::empty());
        let req = crate::models::CreateRecruitmentSourceRequest {
            source_type: crate::models::enums::RecruitmentSourceType::Direct,
            name: "Indeed".into(),
            referrer_employee_id: None,
            referral_bonus_amount: None,
            agency_contact_name: None,
            agency_contact_email: None,
            agency_fee_amount: None,
            notes: None,
        };
        let result = svc.create_recruitment_source(ws, req).await;
        assert!(result.is_ok());
        let src = result.unwrap();
        assert_eq!(src.name, "Indeed");
        assert_eq!(src.workspace_id, ws);
    }

    #[tokio::test]
    async fn update_recruitment_source_success() {
        let ws = Uuid::new_v4();
        let id = Uuid::new_v4();
        let mock = MockEmployeeRepo::empty();
        let updated = RecruitmentSource {
            id,
            workspace_id: ws,
            source_type: crate::models::enums::RecruitmentSourceType::Direct,
            name: "Updated Name".into(),
            referrer_employee_id: None,
            referral_bonus_amount: None,
            referral_bonus_paid: false,
            agency_contact_name: None,
            agency_contact_email: None,
            agency_fee_amount: None,
            agency_fee_paid: false,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        *mock.updated_recruitment_source.lock().expect("lock") = Some(updated.clone());
        let svc = make_service(mock);
        let req = crate::models::UpdateRecruitmentSourceRequest {
            name: Some("Updated Name".into()),
            referrer_employee_id: None,
            referral_bonus_amount: None,
            referral_bonus_paid: None,
            agency_contact_name: None,
            agency_contact_email: None,
            agency_fee_amount: None,
            agency_fee_paid: None,
            notes: None,
        };
        let result = svc.update_recruitment_source(id, ws, req).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Updated Name");
    }

    #[tokio::test]
    async fn get_recruitment_source_breakdown_returns_data() {
        let ws = Uuid::new_v4();
        let mock = MockEmployeeRepo::empty();
        let breakdown = RecruitmentSourceBreakdown {
            source_type: "direct".into(),
            source_name: "Indeed".into(),
            count: 5,
        };
        *mock.recruitment_source_breakdown.lock().expect("lock") = vec![breakdown];
        let svc = make_service(mock);
        let result = svc.get_recruitment_source_breakdown(ws).await;
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].source_name, "Indeed");
    }

    #[tokio::test]
    async fn rehire_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = crate::models::RehireEmployeeRequest {
            hire_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        };
        let result = svc
            .rehire_employee(Uuid::new_v4(), Uuid::new_v4(), req)
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn rehire_employee_already_active() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws); // active by default
        let svc = make_service(MockEmployeeRepo::with_employee(emp.clone()));
        let req = crate::models::RehireEmployeeRequest {
            hire_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        };
        let result = svc.rehire_employee(emp.id, ws, req).await;
        assert!(
            matches!(result.unwrap_err(), AppError::Validation(e) if e == "Employee is already active")
        );
    }

    #[tokio::test]
    async fn rehire_employee_invalid_date() {
        let ws = Uuid::new_v4();
        let mut emp = make_employee(ws);
        emp.is_active = false;
        emp.termination_date = Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap());
        let svc = make_service(MockEmployeeRepo::with_employee(emp.clone()));
        let req = crate::models::RehireEmployeeRequest {
            hire_date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), // Before termination
        };
        let result = svc.rehire_employee(emp.id, ws, req).await;
        assert!(
            matches!(result.unwrap_err(), AppError::Validation(e) if e == "Rehire date must be after termination date")
        );
    }

    #[tokio::test]
    async fn update_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = crate::models::UpdateEmployeeRequest {
            user_id: None,
            full_name: None,
            employee_id_number: None,
            hire_date: None,
            termination_date: None,
            tags: None,
            is_active: None,
            max_concurrent_workers: None,
            recruitment_source_id: None,
        };
        let result = svc
            .update_employee(Uuid::new_v4(), Uuid::new_v4(), req)
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn update_employee_success() {
        let ws = Uuid::new_v4();
        let mock = MockEmployeeRepo::empty();
        let updated = make_employee(ws);
        let updated_id = updated.id;
        *mock.updated_employee.lock().unwrap() = Some(updated.clone());
        let svc = make_service(mock);
        let req = crate::models::UpdateEmployeeRequest {
            user_id: None,
            full_name: Some("Updated Name".into()),
            employee_id_number: None,
            hire_date: None,
            termination_date: None,
            tags: None,
            is_active: None,
            max_concurrent_workers: None,
            recruitment_source_id: None, // No source ID, no id_number update
        };
        let result = svc.update_employee(updated_id, ws, req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_department_history_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .get_department_history(Uuid::new_v4(), Uuid::new_v4())
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn transfer_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = crate::models::TransferEmployeeRequest {
            department_id: Uuid::new_v4(),
            effective_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            notes: None,
        };
        let result = svc
            .transfer_employee(Uuid::new_v4(), Uuid::new_v4(), req)
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }
}
