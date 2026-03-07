use std::sync::Arc;

use axum::http::StatusCode;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateEmployeeRequest, CreateWorkingConditionRequest, Employee, EmploymentPeriod,
    RehireEmployeeRequest, UpdateEmployeeRequest, WorkingCondition,
};
use crate::repositories::{employee_repository, user_repository, EmployeeRepository};
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

    pub async fn get_working_conditions(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, StatusCode> {
        let emp = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        self.repo
            .get_working_conditions(emp.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn create_working_condition(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        req: CreateWorkingConditionRequest,
    ) -> Result<WorkingCondition, StatusCode> {
        let emp = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

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
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(condition)
    }

    pub async fn delete_working_condition(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        condition_id: Uuid,
    ) -> Result<(), StatusCode> {
        let emp = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let deleted = self
            .repo
            .delete_working_condition(condition_id, emp.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if !deleted {
            return Err(StatusCode::NOT_FOUND);
        }
        Ok(())
    }

    pub async fn create_employee(
        &self,
        workspace_id: Uuid,
        req: CreateEmployeeRequest,
    ) -> Result<Employee, StatusCode> {
        // Optional: Check if user_id exists if provided
        if let Some(user_id) = req.user_id {
            user_repository::get_user_by_id(&self.pool, user_id)
                .await
                .map_err(|e| {
                    eprintln!("Error checking user_id for new employee: {e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .ok_or(StatusCode::BAD_REQUEST)?;
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
        };

        // Transaction-based creation still uses pool directly
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        employee_repository::create_employee_tx(&mut tx, &employee)
            .await
            .map_err(|e| {
                eprintln!("Error creating employee: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        tx.commit()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Set recruitment source if provided
        if let Some(source_id) = req.recruitment_source_id {
            self.repo
                .set_employee_recruitment_source(employee.id, Some(source_id))
                .await
                .map_err(|e| {
                    eprintln!("Error setting recruitment source: {e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }

        // Auto-claim any staged logs for this employee_id_number
        if let Err(e) = {
            let te_repo = std::sync::Arc::new(crate::repositories::SqlxTimeEntryRepository::new(self.pool.clone()));
            let ps = std::sync::Arc::new(crate::services::pairing_service::PairingService::new(te_repo, std::sync::Arc::new(crate::repositories::roster_repository::SqlxRosterRepository::new(self.pool.clone())), self.pool.clone()));
            let emp_repo: std::sync::Arc<dyn crate::repositories::EmployeeRepository> = std::sync::Arc::new(crate::repositories::SqlxEmployeeRepository::new(self.pool.clone()));
            let rl_repo: std::sync::Arc<dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository> = std::sync::Arc::new(crate::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(self.pool.clone()));
            let st_repo: std::sync::Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository> = std::sync::Arc::new(crate::repositories::staging_log_repository::SqlxStagingLogRepository::new(self.pool.clone()));
            let er_repo: std::sync::Arc<dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> = std::sync::Arc::new(crate::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(self.pool.clone()));
            let (log_svc, _h) = crate::services::AttendanceLogService::new(self.pool.clone(), ps, 1, 1, 1000, emp_repo, rl_repo, st_repo, er_repo);
            log_svc.claim_staged_logs(workspace_id, employee.id, &employee.employee_id_number).await
        } {
            tracing::warn!(
                "Failed to claim staged logs for new employee {}: {}",
                employee.id,
                e
            );
        }

        Ok(employee)
    }

    pub async fn rehire_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        req: RehireEmployeeRequest,
    ) -> Result<Employee, StatusCode> {
        let employee = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        if employee.is_active {
            return Err(StatusCode::BAD_REQUEST);
        }

        if let Some(term_date) = employee.termination_date {
            if req.hire_date <= term_date {
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        let tx = self
            .pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        self.repo
            .update_employee(
                employee_id,
                workspace_id,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(true),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        tx.commit()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        self.repo
            .get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)
    }

    pub async fn get_employment_history(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<EmploymentPeriod>, StatusCode> {
        let _ = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        self.repo
            .get_employment_history(employee_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get_employee_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Employee, StatusCode> {
        self.repo
            .get_employee_by_id(id, workspace_id)
            .await
            .map_err(|e| {
                eprintln!("Error getting employee: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)
    }

    pub async fn get_all_employees(
        &self,
        workspace_id: Uuid,
        search_query: Option<String>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Employee>, i64), StatusCode> {
        self.repo
            .get_all_employees(workspace_id, search_query, page, limit)
            .await
            .map_err(|e| {
                eprintln!("Error getting all employees: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    pub async fn update_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: UpdateEmployeeRequest,
    ) -> Result<Employee, StatusCode> {
        // Optional: Check if user_id exists if provided
        if let Some(user_id) = req.user_id {
            user_repository::get_user_by_id(&self.pool, user_id)
                .await
                .map_err(|e| {
                    eprintln!("Error checking user_id for update employee: {e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .ok_or(StatusCode::BAD_REQUEST)?;
        }

        let employee = self
            .repo
            .update_employee(
                id,
                workspace_id,
                req.user_id,
                req.full_name,
                req.employee_id_number
                    .clone()
                    .map(|s| normalize_employee_id(&s)),
                req.hire_date,
                req.termination_date,
                req.tags,
                req.is_active,
            )
            .await
            .map_err(|e| {
                eprintln!("Error updating employee: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Update recruitment source if provided
        if req.recruitment_source_id.is_some() {
            self.repo
                .set_employee_recruitment_source(id, req.recruitment_source_id)
                .await
                .map_err(|e| {
                    eprintln!("Error setting recruitment source: {e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }

        // If employee_id_number was updated, try to claim staged logs
        if let Some(new_id_number) = req.employee_id_number {
            let normalized = normalize_employee_id(&new_id_number);
            if let Err(e) = {
                let te_repo = std::sync::Arc::new(crate::repositories::SqlxTimeEntryRepository::new(self.pool.clone()));
                let ps = std::sync::Arc::new(crate::services::pairing_service::PairingService::new(te_repo, std::sync::Arc::new(crate::repositories::roster_repository::SqlxRosterRepository::new(self.pool.clone())), self.pool.clone()));
                let emp_repo: std::sync::Arc<dyn crate::repositories::EmployeeRepository> = std::sync::Arc::new(crate::repositories::SqlxEmployeeRepository::new(self.pool.clone()));
                let rl_repo: std::sync::Arc<dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository> = std::sync::Arc::new(crate::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(self.pool.clone()));
                let st_repo: std::sync::Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository> = std::sync::Arc::new(crate::repositories::staging_log_repository::SqlxStagingLogRepository::new(self.pool.clone()));
                let er_repo: std::sync::Arc<dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> = std::sync::Arc::new(crate::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(self.pool.clone()));
                let (log_svc, _h) = crate::services::AttendanceLogService::new(self.pool.clone(), ps, 1, 1, 1000, emp_repo, rl_repo, st_repo, er_repo);
                log_svc.claim_staged_logs(workspace_id, id, &normalized).await
            } {
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

    pub async fn create_recruitment_source(
        &self,
        workspace_id: Uuid,
        req: crate::models::CreateRecruitmentSourceRequest,
    ) -> Result<crate::models::RecruitmentSource, StatusCode> {
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
            .map_err(|e| {
                eprintln!("Error creating recruitment source: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        Ok(source)
    }

    pub async fn get_recruitment_sources(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::RecruitmentSource>, StatusCode> {
        self.repo
            .get_recruitment_sources(workspace_id)
            .await
            .map_err(|e| {
                eprintln!("Error fetching recruitment sources: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    pub async fn update_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<crate::models::RecruitmentSource, StatusCode> {
        self.repo
            .update_recruitment_source(
                id,
                workspace_id,
                req.name,
                req.referrer_employee_id,
                req.referral_bonus_amount,
                req.referral_bonus_paid,
                req.agency_contact_name,
                req.agency_contact_email,
                req.agency_fee_amount,
                req.agency_fee_paid,
                req.notes,
            )
            .await
            .map_err(|e| {
                eprintln!("Error updating recruitment source: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)
    }

    pub async fn delete_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<(), StatusCode> {
        let rows = self
            .repo
            .delete_recruitment_source(id, workspace_id)
            .await
            .map_err(|e| {
                eprintln!("Error deleting recruitment source: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        if rows == 0 {
            return Err(StatusCode::NOT_FOUND);
        }
        Ok(())
    }

    pub async fn get_recruitment_source_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::RecruitmentSourceBreakdown>, StatusCode> {
        self.repo
            .get_recruitment_source_breakdown(workspace_id)
            .await
            .map_err(|e| {
                eprintln!("Error fetching recruitment source breakdown: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    pub async fn delete_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<(), StatusCode> {
        let rows_affected = self
            .repo
            .delete_employee(id, workspace_id)
            .await
            .map_err(|e| {
                eprintln!("Error deleting employee: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if rows_affected == 0 {
            Err(StatusCode::NOT_FOUND)
        } else {
            Ok(())
        }
    }

    pub async fn get_department_history(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeDepartmentHistory>, StatusCode> {
        let _ = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        crate::repositories::department_repository::get_employee_department_history(
            &self.pool,
            employee_id,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn transfer_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        req: crate::models::TransferEmployeeRequest,
    ) -> Result<crate::models::EmployeeDepartmentHistory, StatusCode> {
        let _ = self
            .repo
            .get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        crate::repositories::department_repository::transfer_employee(
            &self.pool,
            employee_id,
            req.department_id,
            req.effective_date,
            req.notes,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get_turnover_analytics(
        &self,
        workspace_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<crate::models::TurnoverAnalytics, StatusCode> {
        self.repo
            .get_turnover_analytics(workspace_id, start_date, end_date)
            .await
            .map_err(|e| {
                eprintln!("Error fetching turnover analytics: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }
}
