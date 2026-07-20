use crate::worker::WorkerDeps;
use std::sync::Arc;

use crate::repositories::attendance_adjustment_repository::MockAttendanceAdjustmentRepository;
use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
use crate::repositories::attendance_log_error_repository::MockAttendanceLogErrorRepository;
use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;
use crate::repositories::condition_type_repository::MockConditionTypeRepository;
use crate::repositories::custom_field_repository::MockCustomFieldRepository;
use crate::repositories::department_repository::MockDepartmentRepository;
use crate::repositories::employee::MockEmployeeRepository;
use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
use crate::repositories::employment_repository::MockEmploymentRepository;
use crate::repositories::holiday_repository::MockHolidayRepository;
use crate::repositories::import_job_repository::MockImportJobRepository;
use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
use crate::repositories::leave_balance_repository::MockLeaveBalanceRepository;
use crate::repositories::leave_policy_repository::LeavePolicyRepository;
use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
use crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository;
use crate::repositories::roster::MockRosterRepository;
use crate::repositories::session_repository::MockSessionRepository;
use crate::repositories::shift_repository::MockShiftRepository;
use crate::repositories::staging_log_repository::MockStagingLogRepository;
use crate::repositories::time_entry_repository::MockTimeEntryRepository;
use crate::repositories::workspace_repository::MockWorkspaceRepository;

pub struct DummyLeavePolicyRepository;

#[async_trait::async_trait]
impl LeavePolicyRepository for DummyLeavePolicyRepository {
    async fn find_default_paid_leave_type(
        &self,
        _workspace_id: uuid::Uuid,
    ) -> Result<Option<uuid::Uuid>, crate::errors::AppError> {
        Ok(None)
    }

    async fn get_leave_type_name(
        &self,
        _id: uuid::Uuid,
        _workspace_id: uuid::Uuid,
    ) -> Result<String, crate::errors::AppError> {
        Ok("Leave".to_string())
    }

    async fn get_leave_types(
        &self,
        _workspace_id: uuid::Uuid,
    ) -> Result<Vec<crate::models::LeaveType>, crate::errors::AppError> {
        unimplemented!()
    }

    async fn create_leave_type(
        &self,
        _workspace_id: uuid::Uuid,
        _code: String,
        _name: String,
        _is_paid: bool,
        _policy_config: crate::models::LeaveTypeConfig,
    ) -> Result<crate::models::LeaveType, crate::errors::AppError> {
        unimplemented!()
    }

    async fn update_leave_type_config(
        &self,
        _id: uuid::Uuid,
        _workspace_id: uuid::Uuid,
        _policy_config: crate::models::LeaveTypeConfig,
    ) -> Result<crate::models::LeaveType, crate::errors::AppError> {
        unimplemented!()
    }

    async fn delete_leave_type(
        &self,
        _id: uuid::Uuid,
        _workspace_id: uuid::Uuid,
    ) -> Result<u64, crate::errors::AppError> {
        unimplemented!()
    }

    async fn restore_leave_type(
        &self,
        _id: uuid::Uuid,
        _workspace_id: uuid::Uuid,
    ) -> Result<crate::models::LeaveType, crate::errors::AppError> {
        unimplemented!()
    }

    async fn find_retired_leave_type_by_code(
        &self,
        _workspace_id: uuid::Uuid,
        _code: &str,
    ) -> Result<Option<crate::models::diesel::leave::LeaveTypeEntity>, crate::errors::AppError>
    {
        unimplemented!()
    }

    async fn create_policy_definition(
        &self,
        _workspace_id: uuid::Uuid,
        _name: &str,
        _description: Option<&str>,
        _leave_type_id: uuid::Uuid,
        _rule_config: crate::models::LeavePolicyConfig,
    ) -> Result<crate::models::LeavePolicyDefinition, crate::errors::AppError> {
        unimplemented!()
    }

    async fn get_policy_definitions(
        &self,
        _workspace_id: uuid::Uuid,
    ) -> Result<Vec<crate::models::LeavePolicyDefinition>, crate::errors::AppError> {
        unimplemented!()
    }

    async fn update_policy_definition(
        &self,
        _id: uuid::Uuid,
        _workspace_id: uuid::Uuid,
        _name: Option<&str>,
        _description: Option<&str>,
        _leave_type_id: Option<uuid::Uuid>,
        _rule_config: Option<crate::models::LeavePolicyConfig>,
    ) -> Result<crate::models::LeavePolicyDefinition, crate::errors::AppError> {
        unimplemented!()
    }

    async fn delete_policy_definition(
        &self,
        _id: uuid::Uuid,
        _workspace_id: uuid::Uuid,
    ) -> Result<u64, crate::errors::AppError> {
        unimplemented!()
    }

    async fn create_policy_assignment(
        &self,
        _workspace_id: uuid::Uuid,
        _assignment: &crate::models::LeavePolicyAssignment,
    ) -> Result<crate::models::LeavePolicyAssignment, crate::errors::AppError> {
        unimplemented!()
    }

    async fn delete_policy_assignment(
        &self,
        _workspace_id: uuid::Uuid,
        _id: uuid::Uuid,
    ) -> Result<(), crate::errors::AppError> {
        Ok(())
    }

    async fn find_active_assignment_for_employee(
        &self,
        _workspace_id: uuid::Uuid,
        _employee_id: uuid::Uuid,
        _department_id: Option<uuid::Uuid>,
        _date: chrono::NaiveDate,
    ) -> Result<
        Option<(crate::models::LeavePolicyDefinition, uuid::Uuid, String)>,
        crate::errors::AppError,
    > {
        unimplemented!()
    }

    async fn find_all_active_assignments_for_workspace(
        &self,
        _workspace_id: uuid::Uuid,
        _date: chrono::NaiveDate,
    ) -> Result<
        Vec<(
            crate::models::LeavePolicyAssignment,
            crate::models::LeavePolicyDefinition,
            String,
        )>,
        crate::errors::AppError,
    > {
        Ok(vec![])
    }

    async fn get_policy_assignments(
        &self,
        _workspace_id: uuid::Uuid,
    ) -> Result<Vec<crate::models::LeavePolicyAssignmentDetail>, crate::errors::AppError> {
        unimplemented!()
    }
}
use crate::repositories::user_repository::MockUserRepository;

pub struct MockWorkerDepsBuilder {
    pub time_entry_repo: MockTimeEntryRepository,
    pub roster_repo: MockRosterRepository,
    pub policy_repo: MockAttendancePolicyRepository,
    pub condition_type_repo: MockConditionTypeRepository,
    pub employee_repo: MockEmployeeRepository,
    pub raw_log_repo: MockRawAttendanceLogRepository,
    pub staging_repo: MockStagingLogRepository,
    pub error_repo: MockAttendanceLogErrorRepository,
    pub workspace_repo: MockWorkspaceRepository,
    pub holiday_repo: MockHolidayRepository,
    pub leave_request_repo: MockLeaveRequestRepository,
    pub daily_summary_repo: MockAttendanceDailySummaryRepository,
    pub shift_repo: MockShiftRepository,
    pub adjustment_repo: MockAttendanceAdjustmentRepository,
    pub department_repo: MockDepartmentRepository,
    pub import_job_repo: MockImportJobRepository,
    pub session_repo: MockSessionRepository,
    pub leave_balance_repo: MockLeaveBalanceRepository,
    pub contract_repo: MockEmployeeContractRepository,
    pub employment_repo: MockEmploymentRepository,
    pub leave_accrual_log_repo: MockLeaveAccrualLogRepository,
    pub leave_policy_repo: DummyLeavePolicyRepository,
    pub user_repo: MockUserRepository,
    pub system_repo: crate::repositories::system_repository::MockSystemRepository,
    pub outbox_repo: crate::repositories::outbox_job_repository::MockOutboxJobRepository,
    pub workspace_config_repo:
        crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository,
    pub custom_field_repo: MockCustomFieldRepository,
    pub employer_repo: crate::repositories::employer_repository::MockEmployerRepository,
}

impl Default for MockWorkerDepsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MockWorkerDepsBuilder {
    pub fn new() -> Self {
        let mut leave_request_repo = MockLeaveRequestRepository::new();
        leave_request_repo
            .expect_get_approved_leave_dates()
            .returning(|_, _, _, _| Box::pin(async move { Ok(std::collections::HashSet::new()) }));
        leave_request_repo
            .expect_get_approved_leave_details()
            .returning(|_, _, _, _| Box::pin(async move { Ok(std::collections::HashMap::new()) }));

        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo
            .expect_get_employments_for_employees()
            .returning(|_, employee_ids| {
                let mut employments = Vec::new();
                for &emp_id in employee_ids {
                    employments.push(crate::models::employment::Employment::new(
                        emp_id,
                        uuid::Uuid::new_v4(),
                        emp_id,
                        uuid::Uuid::new_v4(),
                        None,
                        None,
                        "active".to_string(),
                        chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                        None,
                        None,
                        "full_time".to_string(),
                        None,
                        chrono::Utc::now(),
                        chrono::Utc::now(),
                        None,
                        None,
                    ));
                }
                Ok(employments)
            });

        employment_repo
            .expect_get_employment_by_id()
            .returning(|_, employment_id| {
                Ok(Some(crate::models::employment::Employment::new(
                    employment_id,
                    uuid::Uuid::new_v4(),
                    employment_id,
                    uuid::Uuid::new_v4(),
                    None,
                    None,
                    "active".to_string(),
                    chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    None,
                    None,
                    "full_time".to_string(),
                    None,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                    None,
                    None,
                )))
            });

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employees_by_ids()
            .returning(|employee_ids, _| {
                let mut employees = Vec::new();
                for &id in employee_ids {
                    employees.push(
                        crate::models::Employee::new(
                            id,
                            uuid::Uuid::new_v4(),
                            None,
                            "Test Employee".to_string(),
                            "EMP001".to_string(),
                            vec![],
                            chrono::Utc::now(),
                            chrono::Utc::now(),
                            None,
                        )
                        .with_employment_info(Some(id), None, None),
                    );
                }
                Ok(employees)
            });

        let mut time_entry_repo = MockTimeEntryRepository::new();
        time_entry_repo
            .expect_get_employment_time_entries()
            .returning(|_, _, _, _| Ok(vec![]));

        let mut condition_type_repo = MockConditionTypeRepository::new();
        condition_type_repo
            .expect_list_condition_types()
            .returning(|_, _| Box::pin(async move { Ok(vec![]) }));

        let mut custom_field_repo = MockCustomFieldRepository::new();
        custom_field_repo
            .expect_list_definitions()
            .returning(|_, _| Box::pin(async move { Ok(vec![]) }));

        Self {
            time_entry_repo,
            roster_repo: MockRosterRepository::new(),
            policy_repo: MockAttendancePolicyRepository::new(),
            condition_type_repo,
            employee_repo,
            raw_log_repo: MockRawAttendanceLogRepository::new(),
            staging_repo: MockStagingLogRepository::new(),
            error_repo: MockAttendanceLogErrorRepository::new(),
            workspace_repo: MockWorkspaceRepository::new(),
            holiday_repo: MockHolidayRepository::new(),
            leave_request_repo,
            daily_summary_repo: MockAttendanceDailySummaryRepository::new(),
            shift_repo: MockShiftRepository::new(),
            adjustment_repo: MockAttendanceAdjustmentRepository::new(),
            department_repo: MockDepartmentRepository::new(),
            import_job_repo: MockImportJobRepository::new(),
            session_repo: MockSessionRepository::new(),
            leave_balance_repo: MockLeaveBalanceRepository::new(),
            contract_repo: MockEmployeeContractRepository::new(),
            employment_repo,
            leave_accrual_log_repo: MockLeaveAccrualLogRepository::new(),
            leave_policy_repo: DummyLeavePolicyRepository,
            user_repo: MockUserRepository::new(),
            system_repo: crate::repositories::system_repository::MockSystemRepository::new(),
            outbox_repo: crate::repositories::outbox_job_repository::MockOutboxJobRepository::new(),
            workspace_config_repo:
                crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository::new(
                ),
            custom_field_repo,
            employer_repo: crate::repositories::employer_repository::MockEmployerRepository::new(),
        }
    }

    pub fn build(self) -> WorkerDeps {
        WorkerDeps {
            time_entry_repo: Arc::new(self.time_entry_repo),
            roster_repo: Arc::new(self.roster_repo),
            policy_repo: Arc::new(self.policy_repo),
            condition_type_repo: Arc::new(self.condition_type_repo),
            employee_repo: Arc::new(self.employee_repo),
            raw_log_repo: Arc::new(self.raw_log_repo),
            staging_repo: Arc::new(self.staging_repo),
            error_repo: Arc::new(self.error_repo),
            workspace_repo: Arc::new(self.workspace_repo),
            holiday_repo: Arc::new(self.holiday_repo),
            leave_request_repo: Arc::new(self.leave_request_repo),
            daily_summary_repo: Arc::new(self.daily_summary_repo),
            shift_repo: Arc::new(self.shift_repo),
            adjustment_repo: Arc::new(self.adjustment_repo),
            department_repo: Arc::new(self.department_repo),
            import_job_repo: Arc::new(self.import_job_repo),
            session_repo: Arc::new(self.session_repo),
            leave_balance_repo: Arc::new(self.leave_balance_repo),
            contract_repo: Arc::new(self.contract_repo),
            employment_repo: Arc::new(self.employment_repo),
            leave_accrual_log_repo: Arc::new(self.leave_accrual_log_repo),
            leave_policy_repo: Arc::new(self.leave_policy_repo),
            user_repo: Arc::new(self.user_repo),
            system_repo: Arc::new(self.system_repo),
            outbox_repo: Arc::new(self.outbox_repo),
            workspace_config_repo: Arc::new(self.workspace_config_repo),
            custom_field_repo: Arc::new(self.custom_field_repo),
            employer_repo: Arc::new(self.employer_repo),
            diesel_pools: crate::diesel_pool::DieselPools::for_test(
                "postgres://user:pass@localhost:5432/leeattend_test_db",
            ),
        }
    }
}
