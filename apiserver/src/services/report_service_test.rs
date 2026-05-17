#[cfg(test)]
mod tests {
    use crate::errors::AppError;
    use crate::models::{
        AssignmentHistory, AttendanceDailySummary, AttendancePolicyVersion, EmployeeDocument,
        EmployeeEmergencyContact, EmployeePayrollConfig, EmployeeProfile, EmployeeSchedule,
        Holiday, LeaveRequest, LeaveStatus, OtStatus, RecruitmentSource,
        RecruitmentSourceBreakdown, RosterItem, ScheduleTrace, ShiftRoster, TimeEntry,
        TimeEntryWithEmployee, TurnoverAnalytics, WorkingCondition, Workspace,
    };
    use crate::repositories::employee::EmployeeBasicInfo;
    use crate::repositories::leave_request_repository::{LeaveBreakdown, LeaveHistoryRequestItem};
    use crate::services::report_service::{AttendanceReportArgs, ReportService, ReportServiceArgs};
    use async_trait::async_trait;
    use chrono::{DateTime, NaiveDate, Utc};
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    // Create manual mocks for the repositories
    struct MockWorkspaceRepo;
    #[async_trait]
    impl crate::repositories::workspace_repository::WorkspaceRepository for MockWorkspaceRepo {
        async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, AppError> {
            Ok(Some(Workspace {
                id,
                name: "Mocked Workspace".into(),
                created_at: Utc::now(),
                date_format: "YYYY-MM-DD".into(),
                time_format: "HH:mm:ss".into(),
                timezone: "UTC".into(),
                theme_primary_color: "#000000".into(),
                default_language: "en".into(),
                holiday_overnight_strategy: "split".into(),
                default_holiday_ot_multiplier: rust_decimal::Decimal::new(15, 1),
                logo_url: None,
                tagline: None,
                custom_db_url_encrypted: None,
            }))
        }
        async fn create_workspace_with_owner(
            &self,
            _n: &str,
            _o: Uuid,
        ) -> Result<Workspace, AppError> {
            unimplemented!()
        }
        async fn get_user_workspaces(&self, _u: Uuid) -> Result<Vec<Workspace>, AppError> {
            unimplemented!()
        }
        async fn clear_workspace_data(&self, _w: Uuid, _g: &str) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn add_member(&self, _w: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn update_member_role(&self, _w: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn remove_member(&self, _w: Uuid, _u: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn find_primary_role(&self, _w: Uuid, _u: Uuid) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_workspace_members(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::WorkspaceMemberDetail>, AppError> {
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
        ) -> Result<Vec<crate::models::WorkspaceRole>, AppError> {
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
            _id: Uuid,
            _n: Option<String>,
            _df: Option<String>,
            _tf: Option<String>,
            _tz: Option<String>,
            _tc: Option<String>,
            _dl: Option<String>,
            _ho: Option<String>,
            _dho: Option<bigdecimal::BigDecimal>,
            _l: Option<String>,
            _t: Option<String>,
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
    }

    struct MockEmployeeRepo;
    #[async_trait]
    impl crate::repositories::employee::EmployeeRepository for MockEmployeeRepo {
        async fn get_paginated_employee_ids(
            &self,
            _w: Uuid,
            _args: crate::models::EmployeePaginationArgs<'_>,
        ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError> {
            Ok((vec![], 0))
        }
        async fn find_by_number(
            &self,
            _w: Uuid,
            _id: &str,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employee_by_id(
            &self,
            _i: Uuid,
            _w: Uuid,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employee_by_id_number(
            &self,
            _id: &str,
            _w: Uuid,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_all_employees(
            &self,
            _w: Uuid,
            _o: crate::models::EmployeeFilterOptions,
        ) -> Result<(Vec<crate::models::Employee>, i64), AppError> {
            unimplemented!()
        }
        async fn get_active_employees(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_active_employee_ids(
            &self,
            _w: Uuid,
            _f: Option<&Vec<Uuid>>,
        ) -> Result<Vec<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_employee_departments_at(
            &self,
            _w: Uuid,
            _d: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
            unimplemented!()
        }
        async fn update_employee(
            &self,
            _i: Uuid,
            _w: Uuid,
            _r: &crate::models::UpdateEmployeeRequest,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn create_employee(&self, _e: &crate::models::Employee) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_employee_skip_duplicates(
            &self,
            _e: &crate::models::Employee,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn rehire_employee(
            &self,
            _w: Uuid,
            _e: Uuid,
            _h: NaiveDate,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn delete_employee(&self, _i: Uuid, _w: Uuid) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<WorkingCondition>, AppError> {
            unimplemented!()
        }
        async fn create_working_condition(
            &self,
            _ws: Uuid,
            _c: &WorkingCondition,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_working_condition(
            &self,
            _ws: Uuid,
            _i: Uuid,
            _e: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_existing_employee_ids_by_numbers(
            &self,
            _w: Uuid,
            _i: &[String],
        ) -> Result<Vec<String>, AppError> {
            unimplemented!()
        }
        async fn upsert_employee_payroll_config(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _s: rust_decimal::Decimal,
            _h: rust_decimal::Decimal,
            _d: NaiveDate,
        ) -> Result<EmployeePayrollConfig, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_config_at(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<EmployeePayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_configs(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<EmployeePayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn set_employee_recruitment_source(
            &self,
            _w: Uuid,
            _e: Uuid,
            _r: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_recruitment_source(&self, _s: &RecruitmentSource) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_recruitment_sources(
            &self,
            _w: Uuid,
        ) -> Result<Vec<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn get_recruitment_source_by_id(
            &self,
            _i: Uuid,
            _w: Uuid,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn update_recruitment_source(
            &self,
            _i: Uuid,
            _w: Uuid,
            _r: &crate::models::UpdateRecruitmentSourceRequest,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn delete_recruitment_source(&self, _i: Uuid, _w: Uuid) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_recruitment_source_breakdown(
            &self,
            _w: Uuid,
        ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
            unimplemented!()
        }
        async fn get_turnover_analytics(
            &self,
            _w: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<TurnoverAnalytics, AppError> {
            unimplemented!()
        }
        async fn get_managed_departments(&self, _w: Uuid, _u: Uuid) -> Result<Vec<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_employee_id_by_user_id(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions_at(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Vec<WorkingCondition>, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions_batch(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
            unimplemented!()
        }
        async fn get_employee_profile(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Option<EmployeeProfile>, AppError> {
            unimplemented!()
        }
        async fn upsert_employee_profile(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _r: &crate::models::UpdateEmployeeProfileRequest,
        ) -> Result<EmployeeProfile, AppError> {
            unimplemented!()
        }
        async fn get_emergency_contacts(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<EmployeeEmergencyContact>, AppError> {
            unimplemented!()
        }
        async fn upsert_emergency_contact(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _r: &crate::models::CreateEmergencyContactRequest,
        ) -> Result<EmployeeEmergencyContact, AppError> {
            unimplemented!()
        }
        async fn delete_emergency_contact(
            &self,
            _w: Uuid,
            _e: Uuid,
            _c: Uuid,
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_employee_documents(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<EmployeeDocument>, AppError> {
            unimplemented!()
        }
        async fn create_employee_document(&self, _d: &EmployeeDocument) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct DummyTimeEntryRepo;
    #[async_trait]
    impl crate::repositories::TimeEntryRepository for DummyTimeEntryRepo {
        async fn create_time_entry(
            &self,
            _workspace_id: Uuid,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_time_entry_by_id(
            &self,
            _w: Uuid,
            _i: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_active_time_entry(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_active_entry(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_entry_for_employee(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn update_time_entry(&self, _w: Uuid, _e: &TimeEntry) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_time_entry(&self, _w: Uuid, _i: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employee_time_entries(
            &self,
            _w: Uuid,
            _e: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_logs(
            &self,
            _w: Uuid,
            _f: crate::models::TimeEntryLogFilter,
        ) -> Result<Vec<TimeEntryWithEmployee>, AppError> {
            unimplemented!()
        }
        async fn count_logs(
            &self,
            _w: Uuid,
            _e: Option<Uuid>,
            _f: Option<DateTime<Utc>>,
            _t: Option<DateTime<Utc>>,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_workspace_time_entries_batch(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, Vec<TimeEntry>>, AppError> {
            unimplemented!()
        }
        async fn replace_time_entries(
            &self,
            _w: Uuid,
            _e: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days(
            &self,
            _w: Uuid,
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
        async fn get_workspace_schedules(
            &self,
            _c: Option<&crate::cache::RosterCache>,
            _w: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
            _emp: Option<&Vec<Uuid>>,
        ) -> Result<Vec<EmployeeSchedule>, AppError> {
            unimplemented!()
        }
        async fn get_assignment_history(
            &self,
            _w: Uuid,
            _l: i64,
        ) -> Result<Vec<AssignmentHistory>, AppError> {
            unimplemented!()
        }
        async fn get_employee_rosters(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<ShiftRoster>, AppError> {
            unimplemented!()
        }
        async fn upsert_roster(
            &self,
            _ws: Uuid,
            _r: &ShiftRoster,
        ) -> Result<ShiftRoster, AppError> {
            unimplemented!()
        }
        async fn bulk_upsert_roster(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _i: Vec<RosterItem>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_effective_shift(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<(Option<Uuid>, Vec<ScheduleTrace>), AppError> {
            unimplemented!()
        }
        async fn delete_roster(&self, _w: Uuid, _e: Uuid, _d: NaiveDate) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct DummyPolicyRepo;
    #[async_trait]
    impl crate::repositories::AttendancePolicyRepository for DummyPolicyRepo {
        async fn get_employee_effective_policy(
            &self,
            _w: Uuid,
            _e: Uuid,
            _t: DateTime<Utc>,
        ) -> Result<Option<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn get_employee_effective_policies_batch(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn get_default_policy_rules(
            &self,
            _w: Uuid,
        ) -> Result<Option<crate::models::AttendanceRules>, AppError> {
            unimplemented!()
        }
        async fn get_default_policy_versions(
            &self,
            _w: Uuid,
        ) -> Result<Vec<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn update_default_policy_rules(
            &self,
            _w: Uuid,
            _u: Uuid,
            _r: crate::handlers::attendance::policies::UpdateAttendanceRulesRequest,
        ) -> Result<AttendancePolicyVersion, AppError> {
            unimplemented!()
        }
        async fn list_policy_assignments(
            &self,
            _w: Uuid,
        ) -> Result<
            Vec<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse>,
            AppError,
        > {
            unimplemented!()
        }
        async fn upsert_policy_assignment(
            &self,
            _w: Uuid,
            _u: Uuid,
            _r: crate::handlers::attendance::policy_assignments::UpsertPolicyAssignmentRequest,
        ) -> Result<
            crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse,
            AppError,
        > {
            unimplemented!()
        }
        async fn delete_policy_assignment(&self, _w: Uuid, _a: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct DummyLeaveRepo;
    #[async_trait]
    impl crate::repositories::leave_request_repository::LeaveRequestRepository for DummyLeaveRepo {
        async fn create_leave_request(&self, _w: Uuid, _r: &LeaveRequest) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_leave_request_by_id(
            &self,
            _w: Uuid,
            _i: Uuid,
        ) -> Result<Option<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn update_leave_request_status(
            &self,
            _w: Uuid,
            _i: Uuid,
            _s: LeaveStatus,
            _a: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employee_leave_requests(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn get_workspace_leave_requests(
            &self,
            _w: Uuid,
            _s: Option<LeaveStatus>,
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn get_approved_leave_breakdown(
            &self,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
            _w: Uuid,
        ) -> Result<LeaveBreakdown, AppError> {
            unimplemented!()
        }
        async fn get_employee_department_for_request(
            &self,
            _r: Uuid,
            _w: Uuid,
        ) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_leave_types(&self, _w: Uuid) -> Result<Vec<(String, String)>, AppError> {
            unimplemented!()
        }
        async fn get_system_generated_holiday_leave(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<(Uuid, Uuid, String)>, AppError> {
            unimplemented!()
        }
        async fn cancel_leave_request(&self, _w: Uuid, _i: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employee_leave_history_requests(
            &self,
            _w: Uuid,
            _e: Uuid,
            _y: i32,
            _l: Option<Uuid>,
        ) -> Result<Vec<LeaveHistoryRequestItem>, AppError> {
            unimplemented!()
        }
        async fn check_leave_type_exists(&self, _w: Uuid, _l: Uuid) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_leave_type_name(
            &self,
            _w: Uuid,
            _l: Uuid,
        ) -> Result<Option<String>, AppError> {
            unimplemented!()
        }
        async fn get_unpaid_holiday_days_without_leave(
            &self,
            _w: Uuid,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_batch_leave_days(
            &self,
            _ws: Uuid,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<HashMap<Uuid, i64>, AppError> {
            unimplemented!()
        }
        async fn get_approved_leave_dates(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError> {
            unimplemented!()
        }
        async fn get_approved_leave_periods(
            &self,
            _w: Uuid,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError> {
            Ok(vec![])
        }
    }

    struct DummyDailySummaryRepo;
    #[async_trait]
    impl crate::repositories::AttendanceDailySummaryRepository for DummyDailySummaryRepo {
        async fn get_summary(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn upsert_calculated_summary(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
            _a: i32,
            _s: i32,
            _n: OtStatus,
            _i: bool,
            _h: Option<String>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn claim_ot(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
            _c: i32,
            _r: Option<String>,
        ) -> Result<usize, AppError> {
            unimplemented!()
        }
        async fn approve_ot(
            &self,
            _w: Uuid,
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
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
            _c: i32,
            _r: String,
            _en: Uuid,
            _p: Option<String>,
        ) -> Result<AttendanceDailySummary, AppError> {
            unimplemented!()
        }
        async fn get_batch_ot_statuses(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<HashMap<(Uuid, NaiveDate), String>, AppError> {
            unimplemented!()
        }
        async fn get_summaries(
            &self,
            _w: Uuid,
            _e: Option<Uuid>,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days_from_summary(
            &self,
            _w: Uuid,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<NaiveDate>, AppError> {
            Ok(vec![])
        }
    }

    struct DummyHolidayRepo;
    #[async_trait]
    impl crate::repositories::holiday_repository::HolidayRepository for DummyHolidayRepo {
        async fn list(&self, _w: Uuid, _y: Option<i32>) -> Result<Vec<Holiday>, AppError> {
            unimplemented!()
        }
        async fn get_by_id(&self, _w: Uuid, _i: Uuid) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
        async fn create_holiday(
            &self,
            _w: Uuid,
            _r: &crate::models::CreateHolidayRequest,
        ) -> Result<Holiday, AppError> {
            unimplemented!()
        }
        async fn update_holiday(
            &self,
            _w: Uuid,
            _i: Uuid,
            _r: &crate::models::UpdateHolidayRequest,
        ) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
        async fn delete(&self, _w: Uuid, _i: Uuid) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_holidays_in_range(
            &self,
            _w: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<HashMap<NaiveDate, Holiday>, AppError> {
            unimplemented!()
        }
        async fn is_holiday(&self, _w: Uuid, _d: NaiveDate) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_get_daily_attendance_report_early_return_with_mock() {
        let report_service = ReportService::new(ReportServiceArgs {
            workspace_repo: Arc::new(MockWorkspaceRepo),
            employee_repo: Arc::new(MockEmployeeRepo),
            time_entry_repo: Arc::new(DummyTimeEntryRepo),
            roster_repo: Arc::new(DummyRosterRepo),
            policy_repo: Arc::new(DummyPolicyRepo),
            leave_repo: Arc::new(DummyLeaveRepo),
            attendance_daily_summary_repo: Arc::new(DummyDailySummaryRepo),
            holiday_repo: Arc::new(DummyHolidayRepo),
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
        };

        // When we call get_daily_attendance_report, it will hit WorkspaceRepo (succeeds)
        // and then EmployeeRepo (returns empty list), triggering an early return with empty Data!
        let result = report_service
            .get_daily_attendance_report(args)
            .await
            .unwrap();

        assert_eq!(result.total, 0);
        assert!(result.data.is_empty());
    }

    // --- SECOND TEST: Full logic for an absent employee ---
    struct AbsentEmployeeRepo;
    #[async_trait]
    impl crate::repositories::employee::EmployeeRepository for AbsentEmployeeRepo {
        async fn get_paginated_employee_ids(
            &self,
            _w: Uuid,
            _args: crate::models::EmployeePaginationArgs<'_>,
        ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError> {
            Ok((
                vec![EmployeeBasicInfo {
                    id: Uuid::nil(),
                    full_name: "Test User".into(),
                    employee_id_number: "EMP001".into(),
                }],
                1,
            ))
        }
        async fn find_by_number(
            &self,
            _w: Uuid,
            _id: &str,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employee_by_id(
            &self,
            _i: Uuid,
            _w: Uuid,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_employee_by_id_number(
            &self,
            _id: &str,
            _w: Uuid,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_all_employees(
            &self,
            _w: Uuid,
            _o: crate::models::EmployeeFilterOptions,
        ) -> Result<(Vec<crate::models::Employee>, i64), AppError> {
            unimplemented!()
        }
        async fn get_active_employees(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn get_active_employee_ids(
            &self,
            _w: Uuid,
            _f: Option<&Vec<Uuid>>,
        ) -> Result<Vec<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_employee_departments_at(
            &self,
            _w: Uuid,
            _d: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
            unimplemented!()
        }
        async fn update_employee(
            &self,
            _i: Uuid,
            _w: Uuid,
            _r: &crate::models::UpdateEmployeeRequest,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn create_employee(&self, _e: &crate::models::Employee) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_employee_skip_duplicates(
            &self,
            _e: &crate::models::Employee,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn rehire_employee(
            &self,
            _w: Uuid,
            _e: Uuid,
            _h: NaiveDate,
        ) -> Result<Option<crate::models::Employee>, AppError> {
            unimplemented!()
        }
        async fn delete_employee(&self, _i: Uuid, _w: Uuid) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<WorkingCondition>, AppError> {
            unimplemented!()
        }
        async fn create_working_condition(
            &self,
            _ws: Uuid,
            _c: &WorkingCondition,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_working_condition(
            &self,
            _ws: Uuid,
            _i: Uuid,
            _e: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_existing_employee_ids_by_numbers(
            &self,
            _w: Uuid,
            _i: &[String],
        ) -> Result<Vec<String>, AppError> {
            unimplemented!()
        }
        async fn upsert_employee_payroll_config(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _s: rust_decimal::Decimal,
            _h: rust_decimal::Decimal,
            _d: NaiveDate,
        ) -> Result<EmployeePayrollConfig, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_config_at(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<EmployeePayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_configs(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<EmployeePayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn set_employee_recruitment_source(
            &self,
            _w: Uuid,
            _e: Uuid,
            _r: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_recruitment_source(&self, _s: &RecruitmentSource) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_recruitment_sources(
            &self,
            _w: Uuid,
        ) -> Result<Vec<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn get_recruitment_source_by_id(
            &self,
            _i: Uuid,
            _w: Uuid,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn update_recruitment_source(
            &self,
            _i: Uuid,
            _w: Uuid,
            _r: &crate::models::UpdateRecruitmentSourceRequest,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            unimplemented!()
        }
        async fn delete_recruitment_source(&self, _i: Uuid, _w: Uuid) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_recruitment_source_breakdown(
            &self,
            _w: Uuid,
        ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
            unimplemented!()
        }
        async fn get_turnover_analytics(
            &self,
            _w: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<TurnoverAnalytics, AppError> {
            unimplemented!()
        }
        async fn get_managed_departments(&self, _w: Uuid, _u: Uuid) -> Result<Vec<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_employee_id_by_user_id(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions_at(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Vec<WorkingCondition>, AppError> {
            unimplemented!()
        }
        async fn get_working_conditions_batch(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
            unimplemented!()
        }
        async fn get_employee_profile(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Option<EmployeeProfile>, AppError> {
            unimplemented!()
        }
        async fn upsert_employee_profile(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _r: &crate::models::UpdateEmployeeProfileRequest,
        ) -> Result<EmployeeProfile, AppError> {
            unimplemented!()
        }
        async fn get_emergency_contacts(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<EmployeeEmergencyContact>, AppError> {
            unimplemented!()
        }
        async fn upsert_emergency_contact(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _r: &crate::models::CreateEmergencyContactRequest,
        ) -> Result<EmployeeEmergencyContact, AppError> {
            unimplemented!()
        }
        async fn delete_emergency_contact(
            &self,
            _w: Uuid,
            _e: Uuid,
            _c: Uuid,
        ) -> Result<u64, AppError> {
            unimplemented!()
        }
        async fn get_employee_documents(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<EmployeeDocument>, AppError> {
            unimplemented!()
        }
        async fn create_employee_document(&self, _d: &EmployeeDocument) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct ScheduleRosterRepo;
    #[async_trait]
    impl crate::repositories::roster::RosterRepository for ScheduleRosterRepo {
        async fn get_workspace_schedules(
            &self,
            _c: Option<&crate::cache::RosterCache>,
            _w: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
            _emp: Option<&Vec<Uuid>>,
        ) -> Result<Vec<EmployeeSchedule>, AppError> {
            Ok(vec![EmployeeSchedule {
                employee_id: Uuid::nil(),
                employee_id_number: "EMP001".into(),
                full_name: "Test User".into(),
                schedules: vec![crate::models::schedule::EffectiveSchedule {
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
                    traces: vec![],
                    is_holiday: false,
                    holiday_name: None,
                }],
            }])
        }
        async fn get_assignment_history(
            &self,
            _w: Uuid,
            _l: i64,
        ) -> Result<Vec<AssignmentHistory>, AppError> {
            unimplemented!()
        }
        async fn get_employee_rosters(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<ShiftRoster>, AppError> {
            unimplemented!()
        }
        async fn upsert_roster(
            &self,
            _ws: Uuid,
            _r: &ShiftRoster,
        ) -> Result<ShiftRoster, AppError> {
            unimplemented!()
        }
        async fn bulk_upsert_roster(
            &self,
            _ws: Uuid,
            _e: Uuid,
            _i: Vec<RosterItem>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_effective_shift(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<(Option<Uuid>, Vec<ScheduleTrace>), AppError> {
            unimplemented!()
        }
        async fn delete_roster(&self, _w: Uuid, _e: Uuid, _d: NaiveDate) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct EmptyTimeEntryRepo;
    #[async_trait]
    impl crate::repositories::TimeEntryRepository for EmptyTimeEntryRepo {
        async fn create_time_entry(
            &self,
            _workspace_id: Uuid,
            _e: &TimeEntry,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_time_entry_by_id(
            &self,
            _w: Uuid,
            _i: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_active_time_entry(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_active_entry(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_latest_entry_for_employee(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Option<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn update_time_entry(&self, _w: Uuid, _e: &TimeEntry) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn delete_time_entry(&self, _w: Uuid, _i: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employee_time_entries(
            &self,
            _w: Uuid,
            _e: Uuid,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<TimeEntry>, AppError> {
            unimplemented!()
        }
        async fn get_paginated_logs(
            &self,
            _w: Uuid,
            _f: crate::models::TimeEntryLogFilter,
        ) -> Result<Vec<TimeEntryWithEmployee>, AppError> {
            unimplemented!()
        }
        async fn count_logs(
            &self,
            _w: Uuid,
            _e: Option<Uuid>,
            _f: Option<DateTime<Utc>>,
            _t: Option<DateTime<Utc>>,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_workspace_time_entries_batch(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, Vec<TimeEntry>>, AppError> {
            Ok(HashMap::new())
        }
        async fn replace_time_entries(
            &self,
            _w: Uuid,
            _e: Uuid,
            _s: DateTime<Utc>,
            _en: DateTime<Utc>,
            _ent: Vec<TimeEntry>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days(
            &self,
            _w: Uuid,
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
            _w: Uuid,
            _e: Uuid,
            _t: DateTime<Utc>,
        ) -> Result<Option<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn get_employee_effective_policies_batch(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _t: DateTime<Utc>,
        ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, AppError> {
            Ok(HashMap::new())
        }
        async fn get_default_policy_rules(
            &self,
            _w: Uuid,
        ) -> Result<Option<crate::models::AttendanceRules>, AppError> {
            unimplemented!()
        }
        async fn get_default_policy_versions(
            &self,
            _w: Uuid,
        ) -> Result<Vec<AttendancePolicyVersion>, AppError> {
            unimplemented!()
        }
        async fn update_default_policy_rules(
            &self,
            _w: Uuid,
            _u: Uuid,
            _r: crate::handlers::attendance::policies::UpdateAttendanceRulesRequest,
        ) -> Result<AttendancePolicyVersion, AppError> {
            unimplemented!()
        }
        async fn list_policy_assignments(
            &self,
            _w: Uuid,
        ) -> Result<
            Vec<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse>,
            AppError,
        > {
            unimplemented!()
        }
        async fn upsert_policy_assignment(
            &self,
            _w: Uuid,
            _u: Uuid,
            _r: crate::handlers::attendance::policy_assignments::UpsertPolicyAssignmentRequest,
        ) -> Result<
            crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse,
            AppError,
        > {
            unimplemented!()
        }
        async fn delete_policy_assignment(&self, _w: Uuid, _a: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct EmptyDailySummaryRepo;
    #[async_trait]
    impl crate::repositories::AttendanceDailySummaryRepository for EmptyDailySummaryRepo {
        async fn get_summary(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn upsert_calculated_summary(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
            _a: i32,
            _s: i32,
            _n: OtStatus,
            _i: bool,
            _h: Option<String>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn claim_ot(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
            _c: i32,
            _r: Option<String>,
        ) -> Result<usize, AppError> {
            unimplemented!()
        }
        async fn approve_ot(
            &self,
            _w: Uuid,
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
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
            _c: i32,
            _r: String,
            _en: Uuid,
            _p: Option<String>,
        ) -> Result<AttendanceDailySummary, AppError> {
            unimplemented!()
        }
        async fn get_batch_ot_statuses(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<HashMap<(Uuid, NaiveDate), String>, AppError> {
            Ok(HashMap::new())
        }
        async fn get_summaries(
            &self,
            _w: Uuid,
            _e: Option<Uuid>,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<AttendanceDailySummary>, AppError> {
            unimplemented!()
        }
        async fn get_distinct_work_days_from_summary(
            &self,
            _w: Uuid,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<NaiveDate>, AppError> {
            Ok(vec![])
        }
    }

    struct EmptyLeaveRepo;
    #[async_trait]
    impl crate::repositories::leave_request_repository::LeaveRequestRepository for EmptyLeaveRepo {
        async fn create_leave_request(&self, _w: Uuid, _r: &LeaveRequest) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_leave_request_by_id(
            &self,
            _w: Uuid,
            _i: Uuid,
        ) -> Result<Option<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn update_leave_request_status(
            &self,
            _w: Uuid,
            _i: Uuid,
            _s: LeaveStatus,
            _a: Option<Uuid>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employee_leave_requests(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn get_workspace_leave_requests(
            &self,
            _w: Uuid,
            _s: Option<LeaveStatus>,
        ) -> Result<Vec<LeaveRequest>, AppError> {
            unimplemented!()
        }
        async fn get_approved_leave_breakdown(
            &self,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
            _w: Uuid,
        ) -> Result<LeaveBreakdown, AppError> {
            unimplemented!()
        }
        async fn get_employee_department_for_request(
            &self,
            _r: Uuid,
            _w: Uuid,
        ) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn get_leave_types(&self, _w: Uuid) -> Result<Vec<(String, String)>, AppError> {
            unimplemented!()
        }
        async fn get_system_generated_holiday_leave(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<(Uuid, Uuid, String)>, AppError> {
            unimplemented!()
        }
        async fn cancel_leave_request(&self, _w: Uuid, _i: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_employee_leave_history_requests(
            &self,
            _w: Uuid,
            _e: Uuid,
            _y: i32,
            _l: Option<Uuid>,
        ) -> Result<Vec<LeaveHistoryRequestItem>, AppError> {
            unimplemented!()
        }
        async fn check_leave_type_exists(&self, _w: Uuid, _l: Uuid) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_leave_type_name(
            &self,
            _w: Uuid,
            _l: Uuid,
        ) -> Result<Option<String>, AppError> {
            unimplemented!()
        }
        async fn get_unpaid_holiday_days_without_leave(
            &self,
            _w: Uuid,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn get_batch_leave_days(
            &self,
            _ws: Uuid,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<HashMap<Uuid, i64>, AppError> {
            unimplemented!()
        }
        async fn get_approved_leave_dates(
            &self,
            _w: Uuid,
            _e: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError> {
            Ok(std::collections::HashSet::new())
        }
        async fn get_approved_leave_periods(
            &self,
            _w: Uuid,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError> {
            Ok(vec![])
        }
    }

    struct EmptyHolidayRepo;
    #[async_trait]
    impl crate::repositories::holiday_repository::HolidayRepository for EmptyHolidayRepo {
        async fn list(&self, _w: Uuid, _y: Option<i32>) -> Result<Vec<Holiday>, AppError> {
            unimplemented!()
        }
        async fn get_by_id(&self, _w: Uuid, _i: Uuid) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
        async fn create_holiday(
            &self,
            _w: Uuid,
            _r: &crate::models::CreateHolidayRequest,
        ) -> Result<Holiday, AppError> {
            unimplemented!()
        }
        async fn update_holiday(
            &self,
            _w: Uuid,
            _i: Uuid,
            _r: &crate::models::UpdateHolidayRequest,
        ) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
        async fn delete(&self, _w: Uuid, _i: Uuid) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_holidays_in_range(
            &self,
            _w: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<HashMap<NaiveDate, Holiday>, AppError> {
            Ok(HashMap::new())
        }
        async fn is_holiday(&self, _w: Uuid, _d: NaiveDate) -> Result<Option<Holiday>, AppError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_get_daily_attendance_report_absent_employee() {
        let report_service = ReportService::new(ReportServiceArgs {
            workspace_repo: Arc::new(MockWorkspaceRepo), // Returns valid workspace
            employee_repo: Arc::new(AbsentEmployeeRepo),
            time_entry_repo: Arc::new(EmptyTimeEntryRepo),
            roster_repo: Arc::new(ScheduleRosterRepo),
            policy_repo: Arc::new(EmptyPolicyRepo),
            leave_repo: Arc::new(EmptyLeaveRepo),
            attendance_daily_summary_repo: Arc::new(EmptyDailySummaryRepo),
            holiday_repo: Arc::new(EmptyHolidayRepo),
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
        };

        let result = report_service
            .get_daily_attendance_report(args)
            .await
            .unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.data.len(), 1);
        let report_item = &result.data[0];
        assert_eq!(report_item.employee_id, Uuid::nil());
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
}
