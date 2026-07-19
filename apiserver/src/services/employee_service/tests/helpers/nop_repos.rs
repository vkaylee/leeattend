use crate::errors::AppError;
use crate::models::{
    Department, DepartmentPayrollConfig, EmployeeDepartmentHistory, Notification, RawAttendanceLog,
    UpdateDepartmentRequest, User,
};
use crate::repositories::department_repository::DepartmentRepository;
use crate::repositories::staging_log_repository::{StagingLog, StagingLogRepository};
use crate::repositories::user_repository::UserRepository;
use crate::utils::i18n::I18nKey;
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

pub struct NopDepartmentRepo;

#[async_trait]
impl DepartmentRepository for NopDepartmentRepo {
    async fn get_employee_department_history_batch(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeeDepartmentHistory>>, AppError> {
        Ok(std::collections::HashMap::new())
    }
    async fn get_department_ids_by_names(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _n: &[String],
    ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
        Ok(std::collections::HashMap::new())
    }
    async fn bulk_create_departments_skip_duplicates(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _d: &[Department],
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn create_department(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _n: String,
        _d: Option<String>,
        _m: Option<Uuid>,
    ) -> Result<Department, AppError> {
        unimplemented!()
    }
    async fn get_department_by_id(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _i: Uuid,
    ) -> Result<Option<Department>, AppError> {
        unimplemented!()
    }
    async fn get_departments_by_ids(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _ids: &[Uuid],
    ) -> Result<Vec<Department>, AppError> {
        Ok(Vec::new())
    }
    async fn get_department_by_name(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _n: &str,
    ) -> Result<Option<Uuid>, AppError> {
        unimplemented!()
    }
    async fn update_department(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _i: Uuid,
        _r: UpdateDepartmentRequest,
    ) -> Result<Department, AppError> {
        unimplemented!()
    }
    async fn delete_department(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _i: Uuid,
    ) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn get_departments(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _p: i64,
        _l: i64,
    ) -> Result<(Vec<Department>, i64), AppError> {
        unimplemented!()
    }
    async fn transfer_employee(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        emp: Uuid,
        dept: Uuid,
        date: NaiveDate,
        notes: Option<String>,
    ) -> Result<EmployeeDepartmentHistory, AppError> {
        Ok(EmployeeDepartmentHistory::new(
            Uuid::new_v4(),
            emp,
            dept,
            date,
            notes,
            chrono::Utc::now(),
        ))
    }
    async fn get_employee_department_history(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
    ) -> Result<Vec<EmployeeDepartmentHistory>, AppError> {
        unimplemented!()
    }
    async fn get_current_department(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
    ) -> Result<Option<EmployeeDepartmentHistory>, AppError> {
        Ok(None)
    }
    async fn get_current_departments_batch(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, EmployeeDepartmentHistory>, AppError> {
        Ok(std::collections::HashMap::new())
    }
    async fn upsert_department_payroll_config(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _de: Uuid,
        _s: rust_decimal::Decimal,
        _h: rust_decimal::Decimal,
        _da: NaiveDate,
    ) -> Result<DepartmentPayrollConfig, AppError> {
        unimplemented!()
    }
    async fn get_department_payroll_config_at(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _de: Uuid,
        _da: NaiveDate,
    ) -> Result<Option<DepartmentPayrollConfig>, AppError> {
        unimplemented!()
    }
    async fn get_department_payroll_configs(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _de: Uuid,
    ) -> Result<Vec<DepartmentPayrollConfig>, AppError> {
        unimplemented!()
    }
    async fn get_department_payroll_configs_batch(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _d: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<DepartmentPayrollConfig>>, AppError> {
        Ok(std::collections::HashMap::new())
    }
}

pub struct NopUserRepo;

#[async_trait]
impl UserRepository for NopUserRepo {
    async fn create_user(&self, _u: &User) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn create_user_with_admin_check(&self, _u: &User) -> Result<bool, AppError> {
        unimplemented!()
    }
    async fn get_user_count(&self) -> Result<i64, AppError> {
        unimplemented!()
    }
    async fn assign_platform_admin(&self, _u: Uuid) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn get_user_by_email(&self, _e: &str) -> Result<Option<User>, AppError> {
        unimplemented!()
    }
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        Ok(Some(User::new(
            id,
            "test@example.com".to_string(),
            "hash".to_string(),
            chrono::Utc::now(),
            Some("Test User".to_string()),
            None,
            None,
            None,
            None,
        )))
    }
    async fn get_users_by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>, AppError> {
        let mut users = Vec::new();
        for id in ids {
            users.push(User::new(
                *id,
                "test@example.com".to_string(),
                "hash".to_string(),
                chrono::Utc::now(),
                Some("Test User".to_string()),
                None,
                None,
                None,
                None,
            ));
        }
        Ok(users)
    }
    async fn update_password_hash(&self, _u: Uuid, _h: &str) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn get_system_roles_for_user(&self, _u: Uuid) -> Result<Vec<String>, AppError> {
        unimplemented!()
    }
    async fn update_user_profile(
        &self,
        _u: Uuid,
        _f: Option<String>,
        _a: Option<String>,
        _b: Option<String>,
    ) -> Result<User, AppError> {
        unimplemented!()
    }
    async fn get_workspace_users(&self, _w: Uuid) -> Result<Vec<User>, AppError> {
        unimplemented!()
    }
    async fn get_dashboard_preferences(
        &self,
        _u: Uuid,
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<crate::handlers::dashboard_preferences::DashboardPreferences>, AppError>
    {
        unimplemented!()
    }
    async fn update_dashboard_preferences(
        &self,
        _u: Uuid,
        _a: &crate::diesel_pool::TenantAuthToken,
        _r: crate::handlers::dashboard_preferences::UpdateDashboardPreferences,
    ) -> Result<crate::handlers::dashboard_preferences::DashboardPreferences, AppError> {
        unimplemented!()
    }
    async fn get_notifications(
        &self,
        _u: Uuid,
        _l: i64,
        _c: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Notification>, AppError> {
        unimplemented!()
    }
    async fn get_unread_notification_count(&self, _u: Uuid) -> Result<i64, AppError> {
        unimplemented!()
    }
    async fn mark_notification_as_read(&self, _u: Uuid, _n: Uuid) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn mark_all_notifications_as_read(&self, _u: Uuid) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn check_recent_notification_exists(
        &self,
        _u: Uuid,
        _t: &str,
        _l: &str,
        _s: i32,
    ) -> Result<bool, AppError> {
        Ok(false)
    }
    async fn create_notification(&self, _n: &Notification) -> Result<(), AppError> {
        Ok(())
    }
    async fn create_notifications_batch(&self, _notifications: &[Notification]) -> Result<(), AppError> {
        Ok(())
    }
}

pub struct NopStagingRepo;

#[async_trait]
impl StagingLogRepository for NopStagingRepo {
    async fn create_staging_log(
        &self,
        _w: Uuid,
        _d: Option<String>,
        _r: &str,
        _p: chrono::DateTime<chrono::Utc>,
        _ra: Option<serde_json::Value>,
        _f: &str,
    ) -> Result<bool, AppError> {
        Ok(false)
    }
    async fn get_pending_staged_logs(
        &self,
        _w: Uuid,
        _e: &str,
    ) -> Result<Vec<StagingLog>, AppError> {
        Ok(vec![])
    }
    async fn update_staging_status(&self, _i: Uuid, _w: Uuid, _s: &str) -> Result<bool, AppError> {
        Ok(false)
    }
    async fn get_pending_staging_logs(
        &self,
        _w: Uuid,
        _l: i64,
    ) -> Result<Vec<StagingLog>, AppError> {
        Ok(vec![])
    }
    async fn count_pending_staging_logs(&self, _w: Uuid) -> Result<i64, AppError> {
        Ok(0)
    }
    async fn claim_staged_logs(&self, _w: Uuid, _r: &str, _e: Uuid) -> Result<u64, AppError> {
        Ok(0)
    }
    async fn resolve_staging_log(
        &self,
        _l: Uuid,
        _w: Uuid,
        _e: Uuid,
    ) -> Result<RawAttendanceLog, AppError> {
        Err(AppError::not_found(I18nKey::NotFound))
    }
}
