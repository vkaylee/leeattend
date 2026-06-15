use super::mock_repo::MockEmployeeRepo;
use super::nop_repos::{NopDepartmentRepo, NopStagingRepo, NopUserRepo};
use crate::models::{Employee, WorkingCondition};
use crate::services::EmployeeService;
use chrono::{NaiveDate, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub fn make_employee(ws: Uuid) -> Employee {
    let emp_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();
    Employee::new(
        emp_id,
        ws,
        None,
        "Test Employee".into(),
        "EMP001".into(),
        vec![],
        Utc::now(),
        Utc::now(),
        None,
    )
    .with_employment_info(Some(employment_id), None, None)
}

pub fn make_working_condition(emp_id: Uuid) -> WorkingCondition {
    WorkingCondition::new(
        Uuid::new_v4(),
        emp_id,
        "late_allowance".into(),
        15,
        true,
        NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
        None,
    )
}

pub fn make_service(mock: MockEmployeeRepo) -> EmployeeService {
    EmployeeService::new(
        Arc::new(mock),
        Arc::new(crate::repositories::employment_repository::MockEmploymentRepository::new()),
        Arc::new(NopDepartmentRepo),
        Arc::new(NopUserRepo),
        Arc::new(NopStagingRepo),
        Arc::new(crate::repositories::leave_balance_repository::MockLeaveBalanceRepository::new()),
    )
}
