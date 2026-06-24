use super::helpers::{make_service, MockEmployeeRepo};
use crate::errors::AppError;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

#[tokio::test]
async fn rehire_employee_not_found() {
    let svc = make_service(MockEmployeeRepo::empty());
    let req = crate::models::RehireEmployeeRequest {
        hire_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        employer_id: Uuid::new_v4(),
        primary_location_id: None,
        department_id: None,
    };
    let result = svc
        .rehire_employee(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            req,
        )
        .await;
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn test_terminate_employment_success() -> Result<(), Box<dyn std::error::Error>> {
    let ws = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();
    let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws);
    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).ok_or("invalid start_date")?;
    let termination_date =
        NaiveDate::from_ymd_opt(2026, 6, 30).ok_or("invalid termination_date")?;

    // 1. Mock Employment
    let mock_emp = crate::models::employment::Employment::new(
        employment_id,
        ws,
        emp_id,
        Uuid::new_v4(),
        None,
        None,
        "active".to_string(),
        start_date,
        None,
        None,
        "primary".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );

    let mut mock_employment_repo =
        crate::repositories::employment_repository::MockEmploymentRepository::new();
    let mock_emp_clone = mock_emp.clone();
    mock_employment_repo
        .expect_get_employment_by_id()
        .returning(move |_, _| Ok(Some(mock_emp_clone.clone())));

    let mock_emp_terminated = crate::models::employment::Employment::new(
        employment_id,
        ws,
        emp_id,
        Uuid::new_v4(),
        None,
        None,
        "terminated".to_string(),
        start_date,
        Some(termination_date),
        None,
        "primary".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        Some("voluntary_resignation".to_string()),
        None,
    );

    // 2. Mock Leave Balances
    let mut mock_leave_balance_repo =
        crate::repositories::leave_balance_repository::MockLeaveBalanceRepository::new();
    let leave_type_id = Uuid::new_v4();
    let mock_balances = vec![crate::models::EmployeeLeaveBalance {
        leave_type_id,
        leave_type_code: "AL".to_string(),
        leave_type_name: "Annual Leave".to_string(),
        is_paid: true,
        entitlement: 12,
        accrued_amount: 0.0,
        carried_over: 0,
        used: 0,
        remaining: 12.0,
    }];
    mock_leave_balance_repo
        .expect_get_employee_balances()
        .returning(move |_, _, _| Ok(mock_balances.clone()));

    let mock_emp_terminated_clone = mock_emp_terminated.clone();
    mock_employment_repo
        .expect_terminate_employment_transactional()
        .returning(move |_, _, _, _, _, balances| {
            assert_eq!(balances.len(), 1);
            // Assert that entitlement has been prorated.
            // From 2026-01-01 to 2026-06-30 is 181 days. 181 / 365 * 12 = 5.95 -> round to 6.
            assert_eq!(balances[0].entitlement(), 6);
            Ok(mock_emp_terminated_clone.clone())
        });

    // 3. Create service and run
    let svc = crate::services::employee_service::EmployeeService::new(
        std::sync::Arc::new(MockEmployeeRepo::empty()),
        std::sync::Arc::new(mock_employment_repo),
        std::sync::Arc::new(super::helpers::NopDepartmentRepo),
        std::sync::Arc::new(super::helpers::NopUserRepo),
        std::sync::Arc::new(super::helpers::NopStagingRepo),
        std::sync::Arc::new(mock_leave_balance_repo),
    );

    let res = svc
        .terminate_employment(
            &auth_token,
            employment_id,
            termination_date,
            "voluntary_resignation".to_string(),
            None,
        )
        .await;

    assert!(res.is_ok());
    let updated = res?;
    assert_eq!(updated.status(), "terminated");
    assert_eq!(updated.end_date(), Some(termination_date));
    Ok(())
}
