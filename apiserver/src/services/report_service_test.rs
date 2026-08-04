#[cfg(test)]
mod tests {
    use crate::errors::AppError;
    use crate::models::{
        AssignmentHistory, AttendanceDailySummary, AttendancePolicyVersion, EmployeeDocument,
        EmployeeEmergencyContact, EmployeePayrollConfig, EmployeeProfile, EmployeeSchedule,
        Holiday, LeaveRequest, LeaveStatus, NewEffectiveScheduleParams, NewEmploymentParams,
        NewTimeEntryParams, NewWorkspaceParams, OtStatus, RecruitmentSource,
        RecruitmentSourceBreakdown, RosterItem, ScheduleTrace, ShiftRoster, TimeEntry,
        TimeEntryWithEmployee, TurnoverAnalytics, WorkingCondition, Workspace,
    };
    use crate::repositories::attendance_daily_summary_repository::{
        GetSummariesPaginatedArgs, UpsertCalculatedSummaryArgs, UpsertCalculatedSummaryTxArgs,
        UpsertPaperOtArgs,
    };
    use crate::repositories::leave_request_repository::{LeaveBreakdown, LeaveHistoryRequestItem};
    use crate::services::report_service::{AttendanceReportArgs, ReportService, ReportServiceArgs};
    use async_trait::async_trait;
    use chrono::{DateTime, NaiveDate, Utc};
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    // Create manual mocks for the repositories
    fn empty_condition_type_repo() -> Arc<dyn crate::repositories::ConditionTypeRepository> {
        let mut repo =
            crate::repositories::condition_type_repository::MockConditionTypeRepository::new();
        repo.expect_list_condition_types()
            .returning(|_, _| Box::pin(async move { Ok(vec![]) }));
        Arc::new(repo)
    }

    struct MockEmployerRepo;
    #[async_trait]
    impl crate::repositories::EmployerRepository for MockEmployerRepo {
        async fn create_employer(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _data: crate::models::CreateEmployerRequest,
        ) -> Result<crate::models::Employer, AppError> {
            unimplemented!()
        }
        async fn get_employer_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<Option<crate::models::Employer>, AppError> {
            unimplemented!()
        }
        async fn get_employers_by_ids(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<Vec<crate::models::Employer>, AppError> {
            unimplemented!()
        }
        async fn get_employers(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::Employer>, i64), AppError> {
            Ok((Vec::new(), 0))
        }
        async fn update_employer(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _data: crate::models::UpdateEmployerRequest,
        ) -> Result<crate::models::Employer, AppError> {
            unimplemented!()
        }
        async fn delete_employer(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn count_employees_by_employer(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
    }

    struct MockWorkspaceRepo;
    #[async_trait]
    impl crate::repositories::workspace_repository::WorkspaceRepository for MockWorkspaceRepo {
        async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, AppError> {
            Ok(Some(Workspace::new(NewWorkspaceParams {
            id: id,
            name: "Mocked Workspace".into(),
            date_format: "YYYY-MM-DD".into(),
            time_format: "HH:mm:ss".into(),
            timezone: "UTC".into(),
            theme_primary_color: "#000000".into(),
            default_language: "en".into(),
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: "split".into(),
            default_holiday_ot_multiplier: rust_decimal::Decimal::new(15, 1),
            created_at: Utc::now(),
            custom_db_url_encrypted: None,
        })))
        }
        async fn create_workspace_with_owner(
            &self,
            _id: Uuid,
            _n: &str,
            _o: Uuid,
            _c: Option<String>,
        ) -> Result<Workspace, AppError> {
            unimplemented!()
        }
        async fn get_user_workspaces(&self, _u: Uuid) -> Result<Vec<Workspace>, AppError> {
            unimplemented!()
        }
        async fn clear_workspace_data(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _g: &str,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn add_member(&self, _ws: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn update_member_role(&self, _ws: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn remove_member(&self, _ws: Uuid, _u: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn find_primary_role(&self, _ws: Uuid, _u: Uuid) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_workspace_members(
            &self,
            _w: Uuid,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::WorkspaceMemberDetail>, i64), AppError> {
            unimplemented!()
        }
        async fn get_all_extra_roles(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::MemberExtraRole>, AppError> {
            unimplemented!()
        }
        async fn find_extra_roles(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Vec<crate::models::MemberExtraRole>, AppError> {
            unimplemented!()
        }
        async fn find_extra_permissions(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Vec<String>, AppError> {
            unimplemented!()
        }
        async fn get_member_extra_permissions(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Vec<crate::models::MemberExtraPermission>, AppError> {
            unimplemented!()
        }
        async fn assign_extra_permission(
            &self,
            _w: Uuid,
            _u: Uuid,
            _p: String,
            _e: DateTime<Utc>,
            _a: Uuid,
        ) -> Result<crate::models::MemberExtraPermission, AppError> {
            unimplemented!()
        }
        async fn revoke_extra_permission(
            &self,
            _w: Uuid,
            _u: Uuid,
            _p: Uuid,
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn assign_extra_role(
            &self,
            _w: Uuid,
            _u: Uuid,
            _r: Uuid,
            _e: DateTime<Utc>,
        ) -> Result<crate::models::MemberExtraRole, AppError> {
            unimplemented!()
        }
        async fn revoke_extra_role(&self, _w: Uuid, _e: Uuid) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_workspace_roles(
            &self,
            _w: Uuid,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::WorkspaceRole>, i64), AppError> {
            unimplemented!()
        }
        async fn create_workspace_role(
            &self,
            _w: Uuid,
            _n: String,
            _d: Option<String>,
            _c: String,
            _s: String,
        ) -> Result<crate::models::WorkspaceRole, AppError> {
            unimplemented!()
        }
        async fn update_workspace_role(
            &self,
            _w: Uuid,
            _r: Uuid,
            _n: Option<String>,
            _d: Option<String>,
            _c: Option<String>,
        ) -> Result<Option<crate::models::WorkspaceRole>, AppError> {
            unimplemented!()
        }
        async fn delete_workspace_role(&self, _w: Uuid, _r: Uuid) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_role_permissions_by_role_id(
            &self,
            _w: Uuid,
            _r: Uuid,
        ) -> Result<Vec<String>, AppError> {
            unimplemented!()
        }
        async fn role_exists_and_is_not_system(
            &self,
            _w: Uuid,
            _r: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn set_role_permissions(&self, _r: Uuid, _p: Vec<String>) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_permissions_for_roles(&self, _r: &[Uuid]) -> Result<Vec<String>, AppError> {
            unimplemented!()
        }
        async fn update_workspace(
            &self,
            _params: crate::repositories::workspace_repository::UpdateWorkspaceParams,
        ) -> Result<Workspace, AppError> {
            unimplemented!()
        }
        async fn is_any_role_global(&self, _r: &[Uuid]) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn update_workspace_byod_url(
            &self,
            _workspace_id: Uuid,
            _encrypted_url: Option<String>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_all_workspaces(&self) -> Result<Vec<Workspace>, AppError> {
            unimplemented!()
        }
        async fn get_admins_with_permissions(
            &self,
            _workspace_id: Uuid,
            _permissions: &[&str],
        ) -> Result<Vec<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_users_with_permission(
            &self,
            _workspace_id: Uuid,
            _permission: &str,
        ) -> Result<Vec<Uuid>, AppError> {
            Ok(vec![])
        }

        async fn delete_workspace(&self, _workspace_id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct MockEmployeeRepo;
    #[async_trait]
    impl crate::repositories::employee::EmployeeCoreRepository for MockEmployeeRepo {
        async fn find_by_number(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _n: &str,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employee_by_id(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employee_by_id_number(
            &self,
            _n: &str,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employees_by_ids(
            &self,
            _ids: &[Uuid],
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<crate::models::Employee>, AppError> {
            Ok(vec![])
        }
        async fn get_employees_by_id_numbers(
            &self,
            _id_numbers: &[String],
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<crate::models::Employee>, AppError> {
            Ok(vec![])
        }
        async fn get_all_employees(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _o: crate::models::EmployeeFilterOptions,
        ) -> Result<(Vec<crate::models::Employee>, i64), AppError> {
            unimplemented!()
        }
        async fn get_active_employees(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_active_employee_ids(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _f: Option<&Vec<Uuid>>,
        ) -> Result<Vec<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_employee_ids(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _args: crate::models::EmployeePaginationArgs<'_>,
        ) -> Result<(Vec<crate::repositories::employee::EmployeeBasicInfo>, i64), AppError>
        {
            Ok((vec![], 0))
        }
        async fn get_employee_departments_at(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _d: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
            unimplemented!()
        }
        async fn update_employee(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
            _r: &crate::models::UpdateEmployeeRequest,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn update_employee_transactional(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
            _r: &crate::models::UpdateEmployeeRequest,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn create_employee(&self, _e: &crate::models::Employee) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_employee_transactional(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: &crate::models::Employee,
            _hire_date: chrono::NaiveDate,
            _r: Option<Uuid>,
            _employer_id: Uuid,
            _primary_location_id: Option<Uuid>,
            _department_id: Option<Uuid>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn create_employee_skip_duplicates(
            &self,
            _e: &crate::models::Employee,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn rehire_employee(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _hire_date: NaiveDate,
            _employer_id: Uuid,
            _primary_location_id: Option<Uuid>,
            _department_id: Option<Uuid>,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn delete_employee(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_existing_employee_ids_by_numbers(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _n: &[String],
        ) -> Result<Vec<String>, AppError> {
            unimplemented!()
        }
        async fn get_employee_ids_by_numbers(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _n: &[String],
        ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn bulk_create_employees_skip_duplicates(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: &[crate::models::Employee],
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_managed_departments(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _u: Uuid,
        ) -> Result<Vec<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_employee_id_by_user_id(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _u: Uuid,
        ) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_employee_timezone_fallback(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<chrono_tz::Tz, AppError> {
            Ok(chrono_tz::UTC)
        }
        async fn get_employee_timezone_at_date(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _d: NaiveDate,
        ) -> Result<chrono_tz::Tz, AppError> {
            Ok(chrono_tz::UTC)
        }
        async fn get_employee_timezones_batch(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            ids: &[Uuid],
        ) -> Result<std::collections::HashMap<Uuid, chrono_tz::Tz>, AppError> {
            Ok(ids.iter().map(|&id| (id, chrono_tz::UTC)).collect())
        }
        async fn get_headquarters_location_id(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeWorkingConditionRepository for MockEmployeeRepo {
        async fn get_working_conditions(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<WorkingCondition>, i64), AppError> {
            unimplemented!()
        }
        async fn create_working_condition(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _c: &WorkingCondition,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_working_condition(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _e: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions_at(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Vec<WorkingCondition>, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions_batch_by_employments(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
            Ok(std::collections::HashMap::new())
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeePayrollConfigRepository for MockEmployeeRepo {
        async fn upsert_employee_payroll_config(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _w: rust_decimal::Decimal,
            _h: rust_decimal::Decimal,
            _d: NaiveDate,
        ) -> Result<EmployeePayrollConfig, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_config_at(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<EmployeePayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_configs(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<Vec<EmployeePayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_configs_batch(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
        ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeePayrollConfig>>, AppError> {
            Ok(std::collections::HashMap::new())
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeRecruitmentRepository for MockEmployeeRepo {
        async fn set_employee_recruitment_source(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _r: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_recruitment_source(&self, _s: &RecruitmentSource) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_recruitment_sources(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<RecruitmentSource>, i64), AppError> {
            unimplemented!()
        }
        async fn get_recruitment_source_by_id(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn update_recruitment_source(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
            _r: &crate::models::UpdateRecruitmentSourceRequest,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn delete_recruitment_source(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_recruitment_source_breakdown(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
            unimplemented!()
        }
        async fn get_turnover_analytics(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<TurnoverAnalytics, AppError> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeSelfServiceRepository for MockEmployeeRepo {
        async fn get_employee_profile(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<EmployeeProfile>, AppError> {
            unimplemented!()
        }
        async fn upsert_employee_profile(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _r: &crate::models::UpdateEmployeeProfileRequest,
        ) -> Result<EmployeeProfile, AppError> {
            unimplemented!()
        }
        async fn get_emergency_contacts(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Vec<EmployeeEmergencyContact>, AppError> {
            unimplemented!()
        }
        async fn upsert_emergency_contact(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _r: &crate::models::CreateEmergencyContactRequest,
        ) -> Result<EmployeeEmergencyContact, AppError> {
            unimplemented!()
        }
        async fn delete_emergency_contact(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _c: Uuid,
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_employee_documents(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Vec<EmployeeDocument>, AppError> {
            unimplemented!()
        }
        async fn create_employee_document(&self, _d: &EmployeeDocument) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl crate::repositories::diesel_employee_repository::assignments::EmployeeAssignmentRepository
        for MockEmployeeRepo
    {
        async fn list_employee_assignments(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
        ) -> Result<Vec<crate::models::employee_assignment::EmployeeAssignment>, AppError> {
            Ok(vec![])
        }
        async fn get_employee_assignment(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _as_id: Uuid,
        ) -> Result<Option<crate::models::employee_assignment::EmployeeAssignment>, AppError>
        {
            Ok(None)
        }
        async fn create_employee_assignment(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _as_: &crate::models::employee_assignment::NewEmployeeAssignment,
        ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
            unimplemented!()
        }
        async fn update_employee_assignment(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _as_id: Uuid,
            _r: &crate::models::employee_assignment::UpdateEmployeeAssignmentRequest,
        ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
            unimplemented!()
        }
        async fn delete_employee_assignment(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _as_id: Uuid,
        ) -> Result<u64, AppError> {
            Ok(0)
        }
    }

    struct DummyTimeEntryRepo;
    #[async_trait]
    impl crate::repositories::TimeEntryRepository for DummyTimeEntryRepo {
        async fn replace_time_entries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employment_time_entries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn create_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_time_entry_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_active_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_active_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_entry_for_employment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn update_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employment_time_entries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_logs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _f: crate::models::TimeEntryLogFilter,
        ) -> Result<Vec<TimeEntryWithEmployee>, AppError> {
            unimplemented!()
        }
        async fn count_logs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Option<Uuid>,
            _f: Option<DateTime<Utc>>,
            _t: Option<DateTime<Utc>>,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_workspace_time_entries_batch_by_employments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, Vec<TimeEntry>>, AppError> {
            unimplemented!()
        }
        async fn replace_time_entries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _tz: &str,
        ) -> Result<Vec<chrono::NaiveDate>, AppError> {
            Ok(vec![])
        }
    }

    struct DummyRosterRepo;
    #[async_trait]
    impl crate::repositories::roster::RosterRepository for DummyRosterRepo {
        async fn get_employment_rosters_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[uuid::Uuid],
            _start_date: chrono::NaiveDate,
            _end_date: chrono::NaiveDate,
        ) -> Result<
            std::collections::HashMap<uuid::Uuid, Vec<crate::models::ShiftRoster>>,
            crate::errors::AppError,
        > {
            Ok(std::collections::HashMap::new())
        }

        async fn get_raw_roster_data(
            &self,
            _workspace_id: Uuid,
        ) -> Result<std::sync::Arc<crate::cache::RosterData>, AppError> {
            Ok(std::sync::Arc::new(crate::cache::RosterData::default()))
        }

        async fn get_workspace_schedules(
            &self,
            _c: Option<&crate::cache::RosterCache>,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _s: NaiveDate,
            _e: NaiveDate,
            _emp: Option<&Vec<Uuid>>,
            _page: Option<i64>,
            _limit: Option<i64>,
        ) -> Result<(Vec<EmployeeSchedule>, i64), AppError> {
            unimplemented!()
        }
        async fn get_assignment_history(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _page: Option<i64>,
            _limit: Option<i64>,
        ) -> Result<(Vec<AssignmentHistory>, i64), AppError> {
            unimplemented!()
        }
        async fn get_employment_rosters(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
            _page: Option<i64>,
            _limit: Option<i64>,
        ) -> Result<(Vec<ShiftRoster>, i64), AppError> {
            unimplemented!()
        }
        async fn upsert_roster(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _r: &ShiftRoster,
        ) -> Result<ShiftRoster, AppError> {
            unimplemented!()
        }
        async fn bulk_upsert_roster(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _i: Vec<RosterItem>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_effective_shift(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<
            (
                Option<Uuid>,
                Vec<ScheduleTrace>,
                std::sync::Arc<crate::cache::RosterData>,
            ),
            AppError,
        > {
            unimplemented!()
        }
        async fn delete_roster(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct DummyPolicyRepo;
    #[async_trait]
    impl crate::repositories::AttendancePolicyRepository for DummyPolicyRepo {
        async fn get_employee_effective_policy(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _t: DateTime<Utc>,
        ) -> Result<Option<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn get_employee_effective_policies_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn get_employment_effective_policy(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _t: DateTime<Utc>,
        ) -> Result<Option<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn get_employment_effective_policies_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn get_default_policy_rules(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<crate::models::AttendanceRules>, AppError> {
            unimplemented!()
        }
        async fn get_default_policy_versions(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn update_default_policy_rules(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _u: Uuid,
            _r: crate::handlers::attendance::policies::UpdateAttendanceRulesRequest,
        ) -> Result<AttendancePolicyVersion, AppError> {
            unimplemented!()
        }
        async fn list_policy_assignments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<
            Vec<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse>,
            AppError,
        > {
            unimplemented!()
        }
        async fn upsert_policy_assignment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _u: Uuid,
            _r: crate::handlers::attendance::policy_assignments::UpsertPolicyAssignmentRequest,
        ) -> Result<
            crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse,
            AppError,
        > {
            unimplemented!()
        }
        async fn delete_policy_assignment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _a: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct DummyLeaveRepo;
    #[async_trait]
    impl crate::repositories::leave_request_repository::LeaveRequestRepository for DummyLeaveRepo {
        async fn create_leave_request(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _r: &LeaveRequest,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_leave_request_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<Option<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn update_leave_request_status(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
            _s: LeaveStatus,
            _a: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employee_leave_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn get_workspace_leave_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _s: Option<LeaveStatus>,
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_workspace_leave_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _status: Option<LeaveStatus>,
            _employee_id: Option<Uuid>,
            _limit: i64,
            _offset: i64,
        ) -> Result<(Vec<LeaveRequest>, i64), AppError> {
            unimplemented!()
        }
        async fn get_approved_leave_breakdown(
            &self,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<LeaveBreakdown, AppError> {
            unimplemented!()
        }
        async fn get_employee_department_for_request(
            &self,
            _r: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_leave_types(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<(String, String)>, AppError> {
            unimplemented!()
        }
        async fn get_system_generated_holiday_leave(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<(Uuid, Uuid, String)>, AppError> {
            unimplemented!()
        }
        async fn cancel_leave_request(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn bulk_cancel_leave_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<u64, AppError> {
            Ok(0)
        }
        async fn create_leave_request_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            __r: &LeaveRequest,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn bulk_cancel_leave_requests_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_employee_leave_history_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _y: i32,
            _l: Option<Uuid>,
        ) -> Result<Vec<LeaveHistoryRequestItem>, AppError> {
            unimplemented!()
        }
        async fn check_leave_type_exists(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _l: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_leave_type_name(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _l: Uuid,
        ) -> Result<Option<String>, AppError> {
            unimplemented!()
        }
        async fn get_unpaid_holiday_days_without_leave(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_batch_leave_days(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<HashMap<Uuid, i64>, AppError> {
            unimplemented!()
        }
        async fn get_approved_leave_dates(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError> {
            unimplemented!()
        }
        async fn get_approved_leave_details(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), bool>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn get_approved_leave_periods(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError> {
            Ok(vec![])
        }
        async fn get_approved_leave_breakdowns_batch(
            &self,
            _employee_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<
            std::collections::HashMap<
                Uuid,
                crate::repositories::leave_request_repository::LeaveBreakdown,
            >,
            AppError,
        > {
            Ok(std::collections::HashMap::new())
        }
        async fn get_unpaid_holiday_days_without_leave_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn get_leave_requests_by_ids(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn update_leave_request_status_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _status: LeaveStatus,
            _approved_by: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct DummyDailySummaryRepo;
    #[async_trait]
    impl crate::repositories::AttendanceDailySummaryRepository for DummyDailySummaryRepo {
        async fn upsert_calculated_summary_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _args: UpsertCalculatedSummaryTxArgs,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn batch_upsert_calculated_summaries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: uuid::Uuid,
            _summaries: &[crate::repositories::attendance_daily_summary_repository::BatchSummaryInput],
        ) -> Result<(), crate::errors::AppError> {
            Ok(())
        }
        async fn get_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn get_summary_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<Option<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn upsert_calculated_summary(
            &self,
            _args: UpsertCalculatedSummaryArgs,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn batch_upsert_calculated_summaries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: uuid::Uuid,
            _summaries: &[crate::repositories::attendance_daily_summary_repository::BatchSummaryInput],
        ) -> Result<(), crate::errors::AppError> {
            Ok(())
        }
        async fn claim_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
            _c: i32,
            _r: Option<String>,
        ) -> Result<usize, AppError> {
            unimplemented!()
        }
        async fn approve_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
            _a: i32,
            _m: Option<String>,
            _s: OtStatus,
        ) -> Result<usize, AppError> {
            unimplemented!()
        }
        async fn upsert_paper_ot(
            &self,
            _args: UpsertPaperOtArgs,
        ) -> Result<AttendanceDailySummary, AppError> {
            unimplemented!()
        }
        async fn get_batch_ot_statuses(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<HashMap<(Uuid, NaiveDate), String>, AppError> {
            unimplemented!()
        }
        async fn get_summaries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Option<Uuid>,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn get_summaries_by_employments_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), AttendanceDailySummary>, AppError>
        {
            unimplemented!()
        }
        async fn get_summaries_paginated(
            &self,
            _args: GetSummariesPaginatedArgs,
        ) -> Result<(Vec<AttendanceDailySummary>, usize), AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days_from_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<NaiveDate>, AppError> {
            Ok(vec![])
        }
        async fn get_summaries_by_ids(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<Vec<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn get_summaries_for_period(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<Vec<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
    }

    struct DummyHolidayRepo;
    #[async_trait]
    impl crate::repositories::holiday_repository::HolidayRepository for DummyHolidayRepo {
        async fn apply_holiday_leave_deductions(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _requests: &[crate::models::LeaveRequest],
            _rosters: &[crate::models::ShiftRoster],
            _logs: &[(Uuid, Uuid, rust_decimal::Decimal, chrono::NaiveDate, String)],
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn list(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _y: Option<i32>,
            _location_id: Option<Uuid>,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<Holiday>, i64), AppError> {
            unimplemented!()
        }
        async fn get_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
        async fn create_holiday(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _r: &crate::models::CreateHolidayRequest,
        ) -> Result<Holiday, AppError> {
            unimplemented!()
        }
        async fn update_holiday(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
            _r: &crate::models::UpdateHolidayRequest,
        ) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
        async fn delete(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_holidays_in_range(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<HashMap<NaiveDate, Holiday>, AppError> {
            unimplemented!()
        }
        async fn is_holiday(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _d: NaiveDate,
            _location_id: Option<Uuid>,
        ) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
    }

    struct DummyEmploymentRepo;
    #[async_trait]
    impl crate::repositories::employment_repository::EmploymentRepository for DummyEmploymentRepo {
        async fn create_employment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _req: &crate::models::employment::CreateEmploymentRequest,
        ) -> Result<crate::models::employment::Employment, AppError> {
            unimplemented!()
        }

        async fn get_employment_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
        ) -> Result<Option<crate::models::employment::Employment>, AppError> {
            unimplemented!()
        }

        async fn list_employee_employments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
        ) -> Result<Vec<crate::models::employment::Employment>, AppError> {
            Ok(vec![])
        }

        async fn update_employment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _req: &crate::models::employment::UpdateEmploymentRequest,
        ) -> Result<crate::models::employment::Employment, AppError> {
            unimplemented!()
        }

        async fn delete_employment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn get_active_employment_on_date(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _date: chrono::NaiveDate,
        ) -> Result<Option<crate::models::employment::Employment>, AppError> {
            Ok(None)
        }

        async fn get_active_employments_for_workspace(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<crate::models::employment::Employment>, AppError> {
            Ok(vec![])
        }

        async fn get_active_employments_for_employees(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
            _date: chrono::NaiveDate,
        ) -> Result<Vec<crate::models::employment::Employment>, AppError> {
            Ok(vec![])
        }

        async fn get_employments_for_employees(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            employee_ids: &[Uuid],
        ) -> Result<Vec<crate::models::employment::Employment>, AppError> {
            Ok(employee_ids
                .iter()
                .map(|&employee_id| {
                    crate::models::employment::Employment::new(NewEmploymentParams {
                        id: Uuid::nil(),
                        workspace_id: Uuid::nil(),
                        employee_id: employee_id,
                        employer_id: Uuid::nil(),
                        primary_location_id: None,
                        department_id: None,
                        status: "active".to_string(),
                        start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                        end_date: None,
                        job_title: None,
                        employment_type: "full_time".to_string(),
                        manager_employment_id: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        termination_reason: None,
                        notes: None,
                    })
                })
                .collect())
        }

        async fn get_employments_by_ids(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            employment_ids: &[Uuid],
        ) -> Result<Vec<crate::models::employment::Employment>, AppError> {
            Ok(employment_ids
                .iter()
                .map(|&id| {
                    crate::models::employment::Employment::new(NewEmploymentParams {
                        id: id,
                        workspace_id: Uuid::nil(),
                        employee_id: Uuid::nil(),
                        employer_id: Uuid::nil(),
                        primary_location_id: None,
                        department_id: None,
                        status: "active".to_string(),
                        start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                        end_date: None,
                        job_title: None,
                        employment_type: "full_time".to_string(),
                        manager_employment_id: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        termination_reason: None,
                        notes: None,
                    })
                })
                .collect())
        }

        async fn terminate_employment_transactional(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _termination_date: chrono::NaiveDate,
            _termination_reason: String,
            _notes: Option<String>,
            _balances: Vec<crate::models::LeaveBalance>,
        ) -> Result<crate::models::employment::Employment, AppError> {
            unimplemented!()
        }

        async fn resolve_employment_on_date(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _date: chrono::NaiveDate,
        ) -> Result<crate::models::employment::EmploymentResolution, AppError> {
            Ok(crate::models::employment::EmploymentResolution::NotFound)
        }

        async fn list_employments_effective_in_range(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _start_date: chrono::NaiveDate,
            _end_date: chrono::NaiveDate,
        ) -> Result<Vec<crate::models::employment::Employment>, AppError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_get_daily_attendance_report_early_return_with_mock() {
        let report_service = ReportService::new(ReportServiceArgs {
            workspace_repo: Arc::new(MockWorkspaceRepo),
            employee_repo: Arc::new(MockEmployeeRepo),
            condition_type_repo: empty_condition_type_repo(),
            employment_repo: Arc::new(DummyEmploymentRepo),
            time_entry_repo: Arc::new(DummyTimeEntryRepo),
            roster_repo: Arc::new(DummyRosterRepo),
            policy_repo: Arc::new(DummyPolicyRepo),
            leave_repo: Arc::new(DummyLeaveRepo),
            attendance_daily_summary_repo: Arc::new(DummyDailySummaryRepo),
            holiday_repo: Arc::new(DummyHolidayRepo),
            employer_repo: Arc::new(MockEmployerRepo),
            work_segment_repo: None,
        });

        let args = AttendanceReportArgs {
            workspace_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            page: Some(1),
            limit: Some(20),
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: None,
            employer_id: None,
            employment_id: None,
            shift_type: None,
        };

        // When we call get_daily_attendance_report, it will hit WorkspaceRepo (succeeds)
        // and then EmployeeRepo (returns empty list), triggering an early return with empty Data!
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);
        let result = report_service
            .get_daily_attendance_report(&auth_token, args)
            .await
            .unwrap();

        assert_eq!(result.total, 0);
        assert!(result.data.is_empty());
    }

    // --- SECOND TEST: Full logic for an absent employee ---
    struct AbsentEmployeeRepo;
    #[async_trait]
    impl crate::repositories::employee::EmployeeCoreRepository for AbsentEmployeeRepo {
        async fn find_by_number(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _n: &str,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employee_by_id(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employee_by_id_number(
            &self,
            _n: &str,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employees_by_ids(
            &self,
            _ids: &[Uuid],
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<crate::models::Employee>, AppError> {
            Ok(vec![])
        }
        async fn get_employees_by_id_numbers(
            &self,
            _id_numbers: &[String],
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<crate::models::Employee>, AppError> {
            Ok(vec![])
        }
        async fn get_all_employees(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _o: crate::models::EmployeeFilterOptions,
        ) -> Result<(Vec<crate::models::Employee>, i64), AppError> {
            unimplemented!()
        }
        async fn get_active_employees(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_active_employee_ids(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _f: Option<&Vec<Uuid>>,
        ) -> Result<Vec<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_employee_ids(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _args: crate::models::EmployeePaginationArgs<'_>,
        ) -> Result<(Vec<crate::repositories::employee::EmployeeBasicInfo>, i64), AppError>
        {
            Ok((
                vec![crate::repositories::employee::EmployeeBasicInfo {
                    id: Uuid::nil(),
                    full_name: "Test User".into(),
                    employee_id_number: "EMP001".into(),
                }],
                1,
            ))
        }
        async fn get_employee_departments_at(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _d: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
            unimplemented!()
        }
        async fn update_employee(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
            _r: &crate::models::UpdateEmployeeRequest,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn update_employee_transactional(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
            _r: &crate::models::UpdateEmployeeRequest,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn create_employee(&self, _e: &crate::models::Employee) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_employee_transactional(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: &crate::models::Employee,
            _hire_date: chrono::NaiveDate,
            _r: Option<Uuid>,
            _employer_id: Uuid,
            _primary_location_id: Option<Uuid>,
            _department_id: Option<Uuid>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn create_employee_skip_duplicates(
            &self,
            _e: &crate::models::Employee,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn rehire_employee(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _hire_date: NaiveDate,
            _employer_id: Uuid,
            _primary_location_id: Option<Uuid>,
            _department_id: Option<Uuid>,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn delete_employee(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_existing_employee_ids_by_numbers(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _n: &[String],
        ) -> Result<Vec<String>, AppError> {
            unimplemented!()
        }
        async fn get_employee_ids_by_numbers(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _n: &[String],
        ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn bulk_create_employees_skip_duplicates(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: &[crate::models::Employee],
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_managed_departments(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _u: Uuid,
        ) -> Result<Vec<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_employee_id_by_user_id(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _u: Uuid,
        ) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_employee_timezone_fallback(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<chrono_tz::Tz, AppError> {
            Ok(chrono_tz::UTC)
        }
        async fn get_employee_timezone_at_date(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _d: NaiveDate,
        ) -> Result<chrono_tz::Tz, AppError> {
            Ok(chrono_tz::UTC)
        }
        async fn get_employee_timezones_batch(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            ids: &[Uuid],
        ) -> Result<std::collections::HashMap<Uuid, chrono_tz::Tz>, AppError> {
            Ok(ids.iter().map(|&id| (id, chrono_tz::UTC)).collect())
        }
        async fn get_headquarters_location_id(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeWorkingConditionRepository for AbsentEmployeeRepo {
        async fn get_working_conditions(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<WorkingCondition>, i64), AppError> {
            unimplemented!()
        }
        async fn create_working_condition(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _c: &WorkingCondition,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_working_condition(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _e: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions_at(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Vec<WorkingCondition>, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions_batch_by_employments(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
            Ok(std::collections::HashMap::new())
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeePayrollConfigRepository for AbsentEmployeeRepo {
        async fn upsert_employee_payroll_config(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _w: rust_decimal::Decimal,
            _h: rust_decimal::Decimal,
            _d: NaiveDate,
        ) -> Result<EmployeePayrollConfig, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_config_at(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<EmployeePayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_configs(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<Vec<EmployeePayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_configs_batch(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
        ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeePayrollConfig>>, AppError> {
            Ok(std::collections::HashMap::new())
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeRecruitmentRepository for AbsentEmployeeRepo {
        async fn set_employee_recruitment_source(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _r: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_recruitment_source(&self, _s: &RecruitmentSource) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_recruitment_sources(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<RecruitmentSource>, i64), AppError> {
            unimplemented!()
        }
        async fn get_recruitment_source_by_id(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn update_recruitment_source(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
            _r: &crate::models::UpdateRecruitmentSourceRequest,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn delete_recruitment_source(
            &self,
            _i: Uuid,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_recruitment_source_breakdown(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
            unimplemented!()
        }
        async fn get_turnover_analytics(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<TurnoverAnalytics, AppError> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeSelfServiceRepository for AbsentEmployeeRepo {
        async fn get_employee_profile(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<EmployeeProfile>, AppError> {
            unimplemented!()
        }
        async fn upsert_employee_profile(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _r: &crate::models::UpdateEmployeeProfileRequest,
        ) -> Result<EmployeeProfile, AppError> {
            unimplemented!()
        }
        async fn get_emergency_contacts(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Vec<EmployeeEmergencyContact>, AppError> {
            unimplemented!()
        }
        async fn upsert_emergency_contact(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _r: &crate::models::CreateEmergencyContactRequest,
        ) -> Result<EmployeeEmergencyContact, AppError> {
            unimplemented!()
        }
        async fn delete_emergency_contact(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _c: Uuid,
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_employee_documents(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Vec<EmployeeDocument>, AppError> {
            unimplemented!()
        }
        async fn create_employee_document(&self, _d: &EmployeeDocument) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl crate::repositories::diesel_employee_repository::assignments::EmployeeAssignmentRepository
        for AbsentEmployeeRepo
    {
        async fn list_employee_assignments(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
        ) -> Result<Vec<crate::models::employee_assignment::EmployeeAssignment>, AppError> {
            Ok(vec![])
        }
        async fn get_employee_assignment(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _as_id: Uuid,
        ) -> Result<Option<crate::models::employee_assignment::EmployeeAssignment>, AppError>
        {
            Ok(None)
        }
        async fn create_employee_assignment(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _as_: &crate::models::employee_assignment::NewEmployeeAssignment,
        ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
            unimplemented!()
        }
        async fn update_employee_assignment(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _as_id: Uuid,
            _r: &crate::models::employee_assignment::UpdateEmployeeAssignmentRequest,
        ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
            unimplemented!()
        }
        async fn delete_employee_assignment(
            &self,
            _a: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _as_id: Uuid,
        ) -> Result<u64, AppError> {
            Ok(0)
        }
    }

    struct ScheduleRosterRepo;
    #[async_trait]
    impl crate::repositories::roster::RosterRepository for ScheduleRosterRepo {
        async fn get_employment_rosters_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[uuid::Uuid],
            _start_date: chrono::NaiveDate,
            _end_date: chrono::NaiveDate,
        ) -> Result<
            std::collections::HashMap<uuid::Uuid, Vec<crate::models::ShiftRoster>>,
            crate::errors::AppError,
        > {
            Ok(std::collections::HashMap::new())
        }

        async fn get_raw_roster_data(
            &self,
            _workspace_id: Uuid,
        ) -> Result<std::sync::Arc<crate::cache::RosterData>, AppError> {
            Ok(std::sync::Arc::new(crate::cache::RosterData::default()))
        }

        async fn get_workspace_schedules(
            &self,
            _c: Option<&crate::cache::RosterCache>,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _s: NaiveDate,
            _e: NaiveDate,
            _emp: Option<&Vec<Uuid>>,
            _page: Option<i64>,
            _limit: Option<i64>,
        ) -> Result<(Vec<EmployeeSchedule>, i64), AppError> {
            let list = vec![EmployeeSchedule::new(
                Uuid::nil(),
                Uuid::nil(),
                "EMP001".into(),
                "Test User".into(),
                vec![crate::models::schedule::EffectiveSchedule::new(NewEffectiveScheduleParams {
                    date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                    shift_id: Some(Uuid::nil()),
                    source: "fixed".into(),
                    shift_name: Some("Day Shift".into()),
                    shift_symbol: Some("D".into()),
                    start_time: Some(chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
                    end_time: Some(chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
                    break_duration_minutes: Some(60),
                    break_start: None,
                    break_end: None,
                    is_break_auto_deduct: Some(true),
                    early_punch_in_grace_period_minutes: Some(0),
                    allow_early_ot: Some(false),
                    break_is_overtime: None,
                    max_early_ot_minutes: None,
                    traces: vec![],
                    is_holiday: false,
                    is_holiday_paid: false,
                    holiday_name: None,
                })],
            )];
            let total = list.len() as i64;
            Ok((list, total))
        }
        async fn get_workspace_segments(
            &self,
            _c: Option<&crate::cache::RosterCache>,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _s: NaiveDate,
            _e: NaiveDate,
            _emp: Option<&Vec<Uuid>>,
            _page: Option<i64>,
            _limit: Option<i64>,
        ) -> Result<(Vec<crate::models::schedule::EmployeeSegments>, i64), AppError> {
            let segment = crate::models::ResolvedSegment {
                template_id: Uuid::nil(),
                interval_id: None,
                sort_order: 0,
                scheduled_start: chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                scheduled_end: chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                source: "fixed".into(),
                date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
                shift_name: Some("Day Shift".into()),
                shift_symbol: Some("D".into()),
                interval_name: None,
                segment_type: "work".into(),
                break_duration_minutes: Some(60),
                break_start: None,
                break_end: None,
                is_break_auto_deduct: Some(true),
                break_is_overtime: Some(false),
                allow_early_ot: false,
                max_early_ot_minutes: None,
                early_punch_in_grace_period_minutes: 0,
                ot_aggregation_mode: Default::default(),
                is_night_shift: false,
                traces: vec![],
                is_holiday: false,
                is_holiday_paid: false,
                holiday_name: None,
            };
            let emp_seg = crate::models::schedule::EmployeeSegments::new(
                Uuid::nil(),
                Uuid::nil(),
                "EMP001".into(),
                "Test User".into(),
                vec![segment],
            );
            Ok((vec![emp_seg], 1))
        }
        async fn get_assignment_history(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _page: Option<i64>,
            _limit: Option<i64>,
        ) -> Result<(Vec<AssignmentHistory>, i64), AppError> {
            unimplemented!()
        }
        async fn get_employment_rosters(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
            _page: Option<i64>,
            _limit: Option<i64>,
        ) -> Result<(Vec<ShiftRoster>, i64), AppError> {
            unimplemented!()
        }
        async fn upsert_roster(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _r: &ShiftRoster,
        ) -> Result<ShiftRoster, AppError> {
            unimplemented!()
        }
        async fn bulk_upsert_roster(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _i: Vec<RosterItem>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_effective_shift(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<
            (
                Option<Uuid>,
                Vec<ScheduleTrace>,
                std::sync::Arc<crate::cache::RosterData>,
            ),
            AppError,
        > {
            unimplemented!()
        }
        async fn delete_roster(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct EmptyTimeEntryRepo;
    #[async_trait]
    impl crate::repositories::TimeEntryRepository for EmptyTimeEntryRepo {
        async fn replace_time_entries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employment_time_entries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn create_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_time_entry_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_active_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_active_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_entry_for_employment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn update_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employment_time_entries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_logs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _f: crate::models::TimeEntryLogFilter,
        ) -> Result<Vec<TimeEntryWithEmployee>, AppError> {
            unimplemented!()
        }
        async fn count_logs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Option<Uuid>,
            _f: Option<DateTime<Utc>>,
            _t: Option<DateTime<Utc>>,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_workspace_time_entries_batch_by_employments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, Vec<TimeEntry>>, AppError> {
            Ok(HashMap::new())
        }
        async fn replace_time_entries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _tz: &str,
        ) -> Result<Vec<chrono::NaiveDate>, AppError> {
            Ok(vec![])
        }
    }

    struct EmptyPolicyRepo;
    #[async_trait]
    impl crate::repositories::AttendancePolicyRepository for EmptyPolicyRepo {
        async fn get_employee_effective_policy(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _t: DateTime<Utc>,
        ) -> Result<Option<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn get_employee_effective_policies_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, AppError> {
            Ok(HashMap::new())
        }
        async fn get_employment_effective_policy(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _t: DateTime<Utc>,
        ) -> Result<Option<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn get_employment_effective_policies_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, AppError> {
            Ok(HashMap::new())
        }
        async fn get_default_policy_rules(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<crate::models::AttendanceRules>, AppError> {
            unimplemented!()
        }
        async fn get_default_policy_versions(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn update_default_policy_rules(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _u: Uuid,
            _r: crate::handlers::attendance::policies::UpdateAttendanceRulesRequest,
        ) -> Result<AttendancePolicyVersion, AppError> {
            unimplemented!()
        }
        async fn list_policy_assignments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<
            Vec<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse>,
            AppError,
        > {
            unimplemented!()
        }
        async fn upsert_policy_assignment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _u: Uuid,
            _r: crate::handlers::attendance::policy_assignments::UpsertPolicyAssignmentRequest,
        ) -> Result<
            crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse,
            AppError,
        > {
            unimplemented!()
        }
        async fn delete_policy_assignment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _a: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct EmptyDailySummaryRepo;
    #[async_trait]
    impl crate::repositories::AttendanceDailySummaryRepository for EmptyDailySummaryRepo {
        async fn upsert_calculated_summary_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _args: UpsertCalculatedSummaryTxArgs,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn batch_upsert_calculated_summaries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: uuid::Uuid,
            _summaries: &[crate::repositories::attendance_daily_summary_repository::BatchSummaryInput],
        ) -> Result<(), crate::errors::AppError> {
            Ok(())
        }
        async fn get_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn get_summary_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<Option<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn upsert_calculated_summary(
            &self,
            _args: UpsertCalculatedSummaryArgs,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn batch_upsert_calculated_summaries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: uuid::Uuid,
            _summaries: &[crate::repositories::attendance_daily_summary_repository::BatchSummaryInput],
        ) -> Result<(), crate::errors::AppError> {
            Ok(())
        }
        async fn claim_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
            _c: i32,
            _r: Option<String>,
        ) -> Result<usize, AppError> {
            unimplemented!()
        }
        async fn approve_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
            _a: i32,
            _m: Option<String>,
            _s: OtStatus,
        ) -> Result<usize, AppError> {
            unimplemented!()
        }
        async fn upsert_paper_ot(
            &self,
            _args: UpsertPaperOtArgs,
        ) -> Result<AttendanceDailySummary, AppError> {
            unimplemented!()
        }
        async fn get_batch_ot_statuses(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<HashMap<(Uuid, NaiveDate), String>, AppError> {
            Ok(HashMap::new())
        }
        async fn get_summaries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Option<Uuid>,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn get_summaries_by_employments_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), AttendanceDailySummary>, AppError>
        {
            unimplemented!()
        }
        async fn get_summaries_paginated(
            &self,
            _args: GetSummariesPaginatedArgs,
        ) -> Result<(Vec<AttendanceDailySummary>, usize), AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days_from_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<NaiveDate>, AppError> {
            Ok(vec![])
        }
        async fn get_summaries_by_ids(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<Vec<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn get_summaries_for_period(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<Vec<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
    }

    struct EmptyLeaveRepo;
    #[async_trait]
    impl crate::repositories::leave_request_repository::LeaveRequestRepository for EmptyLeaveRepo {
        async fn create_leave_request(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _r: &LeaveRequest,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_leave_request_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<Option<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn update_leave_request_status(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
            _s: LeaveStatus,
            _a: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employee_leave_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn get_workspace_leave_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _s: Option<LeaveStatus>,
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_workspace_leave_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _status: Option<LeaveStatus>,
            _employee_id: Option<Uuid>,
            _limit: i64,
            _offset: i64,
        ) -> Result<(Vec<LeaveRequest>, i64), AppError> {
            unimplemented!()
        }
        async fn get_approved_leave_breakdown(
            &self,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<LeaveBreakdown, AppError> {
            unimplemented!()
        }
        async fn get_employee_department_for_request(
            &self,
            _r: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_leave_types(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<(String, String)>, AppError> {
            unimplemented!()
        }
        async fn get_system_generated_holiday_leave(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<(Uuid, Uuid, String)>, AppError> {
            unimplemented!()
        }
        async fn cancel_leave_request(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn bulk_cancel_leave_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<u64, AppError> {
            Ok(0)
        }
        async fn create_leave_request_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            __r: &LeaveRequest,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn bulk_cancel_leave_requests_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_employee_leave_history_requests(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _y: i32,
            _l: Option<Uuid>,
        ) -> Result<Vec<LeaveHistoryRequestItem>, AppError> {
            unimplemented!()
        }
        async fn check_leave_type_exists(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _l: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_leave_type_name(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _l: Uuid,
        ) -> Result<Option<String>, AppError> {
            unimplemented!()
        }
        async fn get_unpaid_holiday_days_without_leave(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_batch_leave_days(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<HashMap<Uuid, i64>, AppError> {
            Ok(HashMap::new())
        }
        async fn get_approved_leave_dates(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError> {
            Ok(std::collections::HashSet::new())
        }
        async fn get_approved_leave_details(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), bool>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn get_approved_leave_periods(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError> {
            Ok(vec![])
        }
        async fn get_approved_leave_breakdowns_batch(
            &self,
            _employee_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<
            std::collections::HashMap<
                Uuid,
                crate::repositories::leave_request_repository::LeaveBreakdown,
            >,
            AppError,
        > {
            Ok(std::collections::HashMap::new())
        }
        async fn get_unpaid_holiday_days_without_leave_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn get_leave_requests_by_ids(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn update_leave_request_status_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _status: LeaveStatus,
            _approved_by: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct EmptyHolidayRepo;
    #[async_trait]
    impl crate::repositories::holiday_repository::HolidayRepository for EmptyHolidayRepo {
        async fn apply_holiday_leave_deductions(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _requests: &[crate::models::LeaveRequest],
            _rosters: &[crate::models::ShiftRoster],
            _logs: &[(Uuid, Uuid, rust_decimal::Decimal, chrono::NaiveDate, String)],
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn list(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _y: Option<i32>,
            _location_id: Option<Uuid>,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<Holiday>, i64), AppError> {
            unimplemented!()
        }
        async fn get_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
        async fn create_holiday(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _r: &crate::models::CreateHolidayRequest,
        ) -> Result<Holiday, AppError> {
            unimplemented!()
        }
        async fn update_holiday(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
            _r: &crate::models::UpdateHolidayRequest,
        ) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
        async fn delete(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_holidays_in_range(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<HashMap<NaiveDate, Holiday>, AppError> {
            Ok(HashMap::new())
        }
        async fn is_holiday(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _d: NaiveDate,
            _location_id: Option<Uuid>,
        ) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_get_daily_attendance_report_absent_employee() {
        let report_service = ReportService::new(ReportServiceArgs {
            workspace_repo: Arc::new(MockWorkspaceRepo), // Returns valid workspace
            employee_repo: Arc::new(AbsentEmployeeRepo),
            condition_type_repo: empty_condition_type_repo(),
            employment_repo: Arc::new(DummyEmploymentRepo),
            time_entry_repo: Arc::new(EmptyTimeEntryRepo),
            roster_repo: Arc::new(ScheduleRosterRepo),
            policy_repo: Arc::new(EmptyPolicyRepo),
            leave_repo: Arc::new(EmptyLeaveRepo),
            attendance_daily_summary_repo: Arc::new(EmptyDailySummaryRepo),
            holiday_repo: Arc::new(EmptyHolidayRepo),
            employer_repo: Arc::new(MockEmployerRepo),
            work_segment_repo: None,
        });

        let args = AttendanceReportArgs {
            workspace_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            page: Some(1),
            limit: Some(20),
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: None,
            employer_id: None,
            employment_id: None,
            shift_type: None,
        };

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);
        let result = report_service
            .get_daily_attendance_report(&auth_token, args)
            .await
            .unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.data.len(), 1);
        let report_item = &result.data[0];
        assert_eq!(report_item.employment_id, Uuid::nil());
        assert_eq!(report_item.employee_name, "Test User");
        assert_eq!(report_item.employee_id_number, "EMP001");

        assert_eq!(
            report_item.date,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
        );
        // Since they had a shift but no punch-in, status should be 'absent'
        assert_eq!(report_item.status.to_string(), "absent");
        assert_eq!(report_item.std_work_units, 0);
        assert_eq!(report_item.ot_work_units, 0);
    }

    struct PresentTimeEntryRepo;
    #[async_trait]
    impl crate::repositories::TimeEntryRepository for PresentTimeEntryRepo {
        async fn replace_time_entries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employment_time_entries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn create_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_time_entry_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_active_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_active_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_entry_for_employment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn update_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employment_time_entries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_logs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _f: crate::models::TimeEntryLogFilter,
        ) -> Result<Vec<TimeEntryWithEmployee>, AppError> {
            unimplemented!()
        }
        async fn count_logs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Option<Uuid>,
            _f: Option<DateTime<Utc>>,
            _t: Option<DateTime<Utc>>,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_workspace_time_entries_batch_by_employments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, Vec<TimeEntry>>, AppError> {
            use chrono::TimeZone;
            let mut map = HashMap::new();
            let entry = TimeEntry::new(NewTimeEntryParams {
                id: Uuid::new_v4(),
                employment_id: Uuid::nil(),
                workspace_id: Uuid::nil(),
                shift_id: Some(Uuid::nil()),
                segment_id: None,
                punch_in_time: Utc.with_ymd_and_hms(2023, 1, 1, 9, 0, 0).unwrap(),
                punch_out_time: Some(Utc.with_ymd_and_hms(2023, 1, 1, 17, 0, 0).unwrap()),
                duration_minutes: Some(480),
                is_break_overtime: false,
                entry_type: crate::services::pairing::types::EntryType::Work,
                note: None,
                generation_metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
            map.insert(Uuid::nil(), vec![entry]);
            Ok(map)
        }
        async fn replace_time_entries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _tz: &str,
        ) -> Result<Vec<chrono::NaiveDate>, AppError> {
            Ok(vec![NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()])
        }
    }

    #[tokio::test]
    async fn test_get_daily_attendance_report_present_employee() {
        let report_service = ReportService::new(ReportServiceArgs {
            workspace_repo: Arc::new(MockWorkspaceRepo),
            employee_repo: Arc::new(AbsentEmployeeRepo),
            condition_type_repo: empty_condition_type_repo(),
            employment_repo: Arc::new(DummyEmploymentRepo),
            time_entry_repo: Arc::new(PresentTimeEntryRepo),
            roster_repo: Arc::new(ScheduleRosterRepo),
            policy_repo: Arc::new(EmptyPolicyRepo),
            leave_repo: Arc::new(EmptyLeaveRepo),
            attendance_daily_summary_repo: Arc::new(EmptyDailySummaryRepo),
            holiday_repo: Arc::new(EmptyHolidayRepo),
            employer_repo: Arc::new(MockEmployerRepo),
            work_segment_repo: None,
        });

        let args = AttendanceReportArgs {
            workspace_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            page: Some(1),
            limit: Some(20),
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: None,
            employer_id: None,
            employment_id: None,
            shift_type: None,
        };

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);
        let result = report_service
            .get_daily_attendance_report(&auth_token, args)
            .await
            .unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.data.len(), 1);
        let report_item = &result.data[0];
        assert_eq!(report_item.employment_id, Uuid::nil());
        assert_eq!(report_item.employee_name, "Test User");
        assert_eq!(report_item.employee_id_number, "EMP001");
        assert_eq!(
            report_item.date,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
        );
        assert_eq!(report_item.status.to_string(), "present");
        assert!(report_item.std_work_units > 0);
    }

    #[tokio::test]
    async fn test_get_workspace_attendance_summary() {
        let report_service = ReportService::new(ReportServiceArgs {
            workspace_repo: Arc::new(MockWorkspaceRepo),
            employee_repo: Arc::new(AbsentEmployeeRepo),
            condition_type_repo: empty_condition_type_repo(),
            employment_repo: Arc::new(DummyEmploymentRepo),
            time_entry_repo: Arc::new(PresentTimeEntryRepo),
            roster_repo: Arc::new(ScheduleRosterRepo),
            policy_repo: Arc::new(EmptyPolicyRepo),
            leave_repo: Arc::new(EmptyLeaveRepo),
            attendance_daily_summary_repo: Arc::new(EmptyDailySummaryRepo),
            holiday_repo: Arc::new(EmptyHolidayRepo),
            employer_repo: Arc::new(MockEmployerRepo),
            work_segment_repo: None,
        });

        let args = AttendanceReportArgs {
            workspace_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            page: Some(1),
            limit: Some(20),
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: None,
            employer_id: None,
            employment_id: None,
            shift_type: None,
        };

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);
        let result = report_service
            .get_workspace_attendance_summary(&auth_token, args)
            .await
            .unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.data.len(), 1);
        let summary_item = &result.data[0];
        assert_eq!(summary_item.employment_id, Uuid::nil());
        assert_eq!(summary_item.employee_name, "Test User");
        assert!(summary_item.total_std_units > 0);
        assert_eq!(summary_item.total_leave_days, 0);
    }

    /// Time entry repo that returns two work sessions with a break in
    /// between. Morning: 09:00–12:00, Break: 12:00–13:00,
    /// Afternoon: 13:00–17:00. Total work: 420 min = 14 units.
    struct MultiSessionTimeEntryRepo;
    #[async_trait]
    impl crate::repositories::TimeEntryRepository for MultiSessionTimeEntryRepo {
        async fn replace_time_entries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employment_time_entries_tx(
            &self,
            _conn: &mut diesel_async::AsyncPgConnection,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn create_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_time_entry_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_active_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_active_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_entry_for_employment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn update_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_time_entry(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _i: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employment_time_entries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_logs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _f: crate::models::TimeEntryLogFilter,
        ) -> Result<Vec<TimeEntryWithEmployee>, AppError> {
            unimplemented!()
        }
        async fn count_logs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Option<Uuid>,
            _f: Option<DateTime<Utc>>,
            _t: Option<DateTime<Utc>>,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_workspace_time_entries_batch_by_employments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, Vec<TimeEntry>>, AppError> {
            use chrono::TimeZone;
            let mut map = HashMap::new();

            let morning = TimeEntry::new(NewTimeEntryParams {
                id: Uuid::new_v4(),
                employment_id: Uuid::nil(),
                workspace_id: Uuid::nil(),
                shift_id: Some(Uuid::nil()),
                segment_id: None,
                punch_in_time: Utc.with_ymd_and_hms(2023, 1, 1, 9, 0, 0).unwrap(),
                punch_out_time: Some(Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap()),
                duration_minutes: Some(180),
                is_break_overtime: false,
                entry_type: crate::services::pairing::types::EntryType::Work,
                note: None,
                generation_metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });

            let lunch = TimeEntry::new(NewTimeEntryParams {
                id: Uuid::new_v4(),
                employment_id: Uuid::nil(),
                workspace_id: Uuid::nil(),
                shift_id: Some(Uuid::nil()),
                segment_id: None,
                punch_in_time: Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap(),
                punch_out_time: Some(Utc.with_ymd_and_hms(2023, 1, 1, 13, 0, 0).unwrap()),
                duration_minutes: Some(60),
                is_break_overtime: false,
                entry_type: crate::services::pairing::types::EntryType::Break,
                note: None,
                generation_metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });

            let afternoon = TimeEntry::new(NewTimeEntryParams {
                id: Uuid::new_v4(),
                employment_id: Uuid::nil(),
                workspace_id: Uuid::nil(),
                shift_id: Some(Uuid::nil()),
                segment_id: None,
                punch_in_time: Utc.with_ymd_and_hms(2023, 1, 1, 13, 0, 0).unwrap(),
                punch_out_time: Some(Utc.with_ymd_and_hms(2023, 1, 1, 17, 0, 0).unwrap()),
                duration_minutes: Some(240),
                is_break_overtime: false,
                entry_type: crate::services::pairing::types::EntryType::Work,
                note: None,
                generation_metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });

            map.insert(Uuid::nil(), vec![morning, lunch, afternoon]);
            Ok(map)
        }
        async fn replace_time_entries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
            _tz: &str,
        ) -> Result<Vec<NaiveDate>, AppError> {
            Ok(vec![])
        }
    }

    /// Multi-session daily report: two work sessions separated by a lunch
    /// break. The one-hour gap must NOT be counted as work.
    /// 420 minutes ÷ 30 min/unit = 14 units (7 hours).
    #[tokio::test]
    async fn test_daily_report_multi_session_gap_not_counted() {
        let report_service = ReportService::new(ReportServiceArgs {
            workspace_repo: Arc::new(MockWorkspaceRepo),
            employee_repo: Arc::new(AbsentEmployeeRepo),
            condition_type_repo: empty_condition_type_repo(),
            employment_repo: Arc::new(DummyEmploymentRepo),
            time_entry_repo: Arc::new(MultiSessionTimeEntryRepo),
            roster_repo: Arc::new(ScheduleRosterRepo),
            policy_repo: Arc::new(EmptyPolicyRepo),
            leave_repo: Arc::new(EmptyLeaveRepo),
            attendance_daily_summary_repo: Arc::new(EmptyDailySummaryRepo),
            holiday_repo: Arc::new(EmptyHolidayRepo),
            employer_repo: Arc::new(MockEmployerRepo),
            work_segment_repo: None,
        });

        let args = AttendanceReportArgs {
            workspace_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            page: Some(1),
            limit: Some(20),
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: None,
            employer_id: None,
            employment_id: None,
            shift_type: None,
        };

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);
        let result = report_service
            .get_daily_attendance_report(&auth_token, args)
            .await
            .unwrap();

        assert_eq!(result.total, 1);
        let report_item = &result.data[0];
        // 420 min ÷ 30 min/unit = 14 units (no gap leakage)
        assert_eq!(report_item.std_work_units, 14);
        assert_eq!(report_item.ot_work_units, 0);
        // The actual_punch_in must be the first work session (09:00),
        // not the break (12:00) nor the afternoon session (13:00).
        assert!(report_item.actual_punch_in.is_some());
        assert!(report_item.actual_punch_out.is_some());
        assert_eq!(report_item.status.to_string(), "present");
    }

    /// Segment report: same multi-session data as the daily test above.
    /// The segment-centric path (default in API handler since
    /// `segment_centric.unwrap_or(true)`) must produce the same day totals.
    #[tokio::test]
    async fn test_segment_report_multi_session_via_segments() {
        let report_service = ReportService::new(ReportServiceArgs {
            workspace_repo: Arc::new(MockWorkspaceRepo),
            employee_repo: Arc::new(AbsentEmployeeRepo),
            condition_type_repo: empty_condition_type_repo(),
            employment_repo: Arc::new(DummyEmploymentRepo),
            time_entry_repo: Arc::new(MultiSessionTimeEntryRepo),
            roster_repo: Arc::new(ScheduleRosterRepo),
            policy_repo: Arc::new(EmptyPolicyRepo),
            leave_repo: Arc::new(EmptyLeaveRepo),
            attendance_daily_summary_repo: Arc::new(EmptyDailySummaryRepo),
            holiday_repo: Arc::new(EmptyHolidayRepo),
            employer_repo: Arc::new(MockEmployerRepo),
            work_segment_repo: None, // ← compute fallback via get_workspace_segments
        });

        let args = AttendanceReportArgs {
            workspace_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            page: Some(1),
            limit: Some(20),
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: None,
            employer_id: None,
            employment_id: None,
            shift_type: None,
        };

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);
        let result = report_service
            .get_segment_attendance_report(&auth_token, args)
            .await
            .unwrap();

        assert_eq!(result.total, 1);
        let report_item = &result.data[0];

        // 09:00-12:00 + 13:00-17:00 = 420 min → 14 units (7h).
        // segment_units(0) = total since there is 1 segment.
        assert_eq!(report_item.std_work_units, 14);
        assert_eq!(report_item.ot_work_units, 0);
        assert_eq!(report_item.status.to_string(), "present");

        // day_totals in generation_metadata must match the row.
        let meta = report_item
            .generation_metadata
            .as_ref()
            .expect("segment rows carry generation_metadata");
        let day_totals = meta
            .get("day_totals")
            .expect("generation_metadata includes day_totals");
        assert_eq!(day_totals["std_work_units"].as_i64(), Some(14));
        assert_eq!(day_totals["ot_work_units"].as_i64(), Some(0));
    }
}
