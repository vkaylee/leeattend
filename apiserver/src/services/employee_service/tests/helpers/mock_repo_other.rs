use super::mock_repo::MockEmployeeRepo;
use crate::errors::AppError;
use crate::models::{
    CreateEmergencyContactRequest, EmployeeDocument, EmployeeEmergencyContact,
    EmployeePayrollConfig, EmployeeProfile, RecruitmentEffectivenessAnalytics,
    RecruitmentEffectivenessQuery, RecruitmentSource, RecruitmentSourceBreakdown,
    TurnoverAnalytics, TurnoverAnalyticsQuery, UpdateEmployeeProfileRequest, WorkingCondition,
};
use crate::repositories::diesel_employee_repository::assignments::EmployeeAssignmentRepository;
use crate::repositories::employee::{
    EmployeePayrollConfigRepository, EmployeeRecruitmentRepository, EmployeeSelfServiceRepository,
    EmployeeWorkingConditionRepository,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
impl EmployeeWorkingConditionRepository for MockEmployeeRepo {
    async fn get_working_conditions(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkingCondition>, i64), AppError> {
        let list = self.working_conditions.lock().expect("lock").clone();
        let total = list.len() as i64;
        let start = ((page - 1) * limit) as usize;
        let end = (start + limit as usize).min(list.len());
        let paginated = if start < list.len() {
            list[start..end].to_vec()
        } else {
            vec![]
        };
        Ok((paginated, total))
    }
    async fn create_working_condition(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _c: &WorkingCondition,
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn delete_working_condition(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _i: Uuid,
        _e: Uuid,
    ) -> Result<bool, AppError> {
        Ok(*self.delete_wc_result.lock().expect("lock"))
    }
    async fn get_working_conditions_at(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _d: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        Ok(vec![])
    }
    async fn get_working_conditions_batch_by_employments(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: &[Uuid],
        _s: NaiveDate,
        _en: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
        Ok(std::collections::HashMap::new())
    }
}

#[async_trait]
impl EmployeePayrollConfigRepository for MockEmployeeRepo {
    async fn upsert_employee_payroll_config(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, AppError> {
        Ok(EmployeePayrollConfig {
            id: Uuid::new_v4(),
            employment_id,
            std_working_days,
            hours_per_day,
            effective_date,
            created_at: chrono::Utc::now(),
        })
    }
    async fn get_employee_payroll_config_at(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _d: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, AppError> {
        Ok(None)
    }
    async fn get_employee_payroll_configs(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
    ) -> Result<Vec<EmployeePayrollConfig>, AppError> {
        Ok(vec![])
    }
    async fn get_employee_payroll_configs_batch(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeePayrollConfig>>, AppError> {
        Ok(std::collections::HashMap::new())
    }
}

#[async_trait]
impl EmployeeRecruitmentRepository for MockEmployeeRepo {
    async fn set_employee_recruitment_source(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _r: Option<Uuid>,
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn create_recruitment_source(&self, _s: &RecruitmentSource) -> Result<(), AppError> {
        Ok(())
    }
    async fn get_recruitment_sources(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<RecruitmentSource>, i64), AppError> {
        let list = self.recruitment_sources.lock().expect("lock").clone();
        let total = list.len() as i64;
        let start = ((page - 1) * limit) as usize;
        let end = (start + limit as usize).min(list.len());
        let paginated = if start < list.len() {
            list[start..end].to_vec()
        } else {
            vec![]
        };
        Ok((paginated, total))
    }
    async fn get_recruitment_source_by_id(
        &self,
        _i: Uuid,
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        Ok(None)
    }
    async fn update_recruitment_source(
        &self,
        _i: Uuid,
        _a: &crate::diesel_pool::TenantAuthToken,
        _r: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        Ok(self
            .updated_recruitment_source
            .lock()
            .expect("lock")
            .clone())
    }
    async fn delete_recruitment_source(
        &self,
        _i: Uuid,
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        Ok(*self.delete_rs_rows.lock().expect("lock"))
    }
    async fn get_recruitment_source_breakdown(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
        Ok(self
            .recruitment_source_breakdown
            .lock()
            .expect("lock")
            .clone())
    }
    async fn get_turnover_analytics(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        query: &TurnoverAnalyticsQuery,
    ) -> Result<TurnoverAnalytics, AppError> {
        Ok(self
            .turnover
            .lock()
            .expect("lock")
            .clone()
            .unwrap_or_else(|| TurnoverAnalytics::empty(query.start_date, query.end_date)))
    }
    async fn get_recruitment_effectiveness(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        query: &RecruitmentEffectivenessQuery,
    ) -> Result<RecruitmentEffectivenessAnalytics, AppError> {
        Ok(self
            .effectiveness
            .lock()
            .expect("lock")
            .clone()
            .unwrap_or_else(|| {
                RecruitmentEffectivenessAnalytics::empty(query.start_date, query.end_date)
            }))
    }
}

#[async_trait]
impl EmployeeSelfServiceRepository for MockEmployeeRepo {
    async fn get_employee_profile(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
    ) -> Result<Option<EmployeeProfile>, AppError> {
        Ok(None)
    }
    async fn upsert_employee_profile(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _r: &UpdateEmployeeProfileRequest,
    ) -> Result<EmployeeProfile, AppError> {
        unimplemented!()
    }
    async fn get_emergency_contacts(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
    ) -> Result<Vec<EmployeeEmergencyContact>, AppError> {
        Ok(vec![])
    }
    async fn upsert_emergency_contact(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _r: &CreateEmergencyContactRequest,
    ) -> Result<EmployeeEmergencyContact, AppError> {
        unimplemented!()
    }
    async fn delete_emergency_contact(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _c: Uuid,
    ) -> Result<u64, AppError> {
        Ok(0)
    }
    async fn get_employee_documents(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
    ) -> Result<Vec<EmployeeDocument>, AppError> {
        Ok(vec![])
    }
    async fn create_employee_document(&self, _d: &EmployeeDocument) -> Result<(), AppError> {
        Ok(())
    }
}

#[async_trait]
impl EmployeeAssignmentRepository for MockEmployeeRepo {
    async fn list_employee_assignments(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
    ) -> Result<Vec<crate::models::employee_assignment::EmployeeAssignment>, AppError> {
        Ok(vec![])
    }
    async fn get_employee_assignment(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _as: Uuid,
    ) -> Result<Option<crate::models::employee_assignment::EmployeeAssignment>, AppError> {
        Ok(None)
    }
    async fn create_employee_assignment(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _as: &crate::models::employee_assignment::NewEmployeeAssignment,
    ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
        Ok(crate::models::employee_assignment::EmployeeAssignment {
            id: Uuid::new_v4(),
            workspace_id: _as.workspace_id,
            employment_id: _as.employment_id,
            home_location_id: _as.home_location_id,
            host_location_id: _as.host_location_id,
            start_date: _as.start_date,
            end_date: _as.end_date,
            reason: _as.reason.clone(),
            status: _as.status,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn update_employee_assignment(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _as: Uuid,
        _r: &crate::models::employee_assignment::UpdateEmployeeAssignmentRequest,
    ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
        Ok(crate::models::employee_assignment::EmployeeAssignment {
            id: _as,
            workspace_id: Uuid::new_v4(),
            employment_id: _e,
            home_location_id: Uuid::new_v4(),
            host_location_id: Uuid::new_v4(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            reason: None,
            status: crate::models::employee_assignment::AssignmentStatusEnum::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn delete_employee_assignment(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _as: Uuid,
    ) -> Result<u64, AppError> {
        Ok(0)
    }
    async fn list_assignments_batch(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: &[Uuid],
    ) -> Result<
        std::collections::HashMap<
            Uuid,
            Vec<crate::models::employee_assignment::EmployeeAssignment>,
        >,
        AppError,
    > {
        Ok(std::collections::HashMap::new())
    }
}
