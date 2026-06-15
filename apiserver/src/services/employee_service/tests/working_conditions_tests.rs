use super::helpers::{make_employee, make_service, make_working_condition, MockEmployeeRepo};
use crate::errors::AppError;
use crate::models::CreateWorkingConditionRequest;
use chrono::NaiveDate;
use uuid::Uuid;

#[tokio::test]
async fn get_working_conditions_returns_conditions() {
    let ws = Uuid::new_v4();
    let emp = make_employee(ws);
    let cond = make_working_condition(emp.id());
    let mock = MockEmployeeRepo::with_employee(emp.clone());
    *mock.working_conditions.lock().expect("lock") = vec![cond.clone()];
    let svc = make_service(mock);
    let result = svc
        .get_working_conditions(
            emp.id(),
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            1,
            50,
        )
        .await
        .expect("ok");
    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].type_(), "late_allowance");
    assert_eq!(result.0[0].minute_allowance(), 15);
}

#[tokio::test]
async fn get_working_conditions_employee_not_found() {
    let svc = make_service(MockEmployeeRepo::empty());
    let result = svc
        .get_working_conditions(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            1,
            50,
        )
        .await;
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

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
    let result = svc
        .create_working_condition(
            emp.id(),
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            req,
        )
        .await;
    assert!(result.is_ok());
    let wc = result.expect("ok");
    assert_eq!(wc.type_(), "early_leave");
    assert_eq!(wc.minute_allowance(), 30);
    assert!(!wc.is_paid());
    assert_eq!(wc.employment_id(), emp.active_employment_id().unwrap());
}

#[tokio::test]
async fn delete_working_condition_success() {
    let ws = Uuid::new_v4();
    let emp = make_employee(ws);
    let mock = MockEmployeeRepo::with_employee(emp.clone());
    *mock.delete_wc_result.lock().expect("lock") = true;
    let svc = make_service(mock);
    let result = svc
        .delete_working_condition(
            emp.id(),
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            Uuid::new_v4(),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_working_condition_not_found() {
    let ws = Uuid::new_v4();
    let emp = make_employee(ws);
    let mock = MockEmployeeRepo::with_employee(emp.clone());
    let svc = make_service(mock);
    let result = svc
        .delete_working_condition(
            emp.id(),
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            Uuid::new_v4(),
        )
        .await;
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn delete_working_condition_employee_not_found() {
    let svc = make_service(MockEmployeeRepo::empty());
    let result = svc
        .delete_working_condition(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
        )
        .await;
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

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
        .create_working_condition(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            req,
        )
        .await;
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}
