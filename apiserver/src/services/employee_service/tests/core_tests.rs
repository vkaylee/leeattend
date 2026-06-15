use super::helpers::{make_employee, make_service, MockEmployeeRepo};
use crate::errors::AppError;
use crate::models::{EmployeeFilterOptions, UpdateEmployeeRequest};
use uuid::Uuid;

#[tokio::test]
async fn get_employee_by_id_found() {
    let ws = Uuid::new_v4();
    let emp = make_employee(ws);
    let emp_id = emp.id();
    let svc = make_service(MockEmployeeRepo::with_employee(emp.clone()));
    let result = svc
        .get_employee_by_id(
            emp_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(result.expect("ok").id(), emp_id);
}

#[tokio::test]
async fn get_employee_by_id_not_found() {
    let svc = make_service(MockEmployeeRepo::empty());
    let result = svc
        .get_employee_by_id(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        )
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn get_all_employees_returns_list_and_count() {
    let ws = Uuid::new_v4();
    let emp = make_employee(ws);
    let mock = MockEmployeeRepo::empty();
    *mock.employees.lock().expect("lock") = vec![emp.clone()];
    *mock.total.lock().expect("lock") = 1;
    let svc = make_service(mock);
    let (list, total) = svc
        .get_all_employees(
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            EmployeeFilterOptions::default(),
        )
        .await
        .expect("ok");
    assert_eq!(list.len(), 1);
    assert_eq!(total, 1);
    assert_eq!(list[0].full_name(), "Test Employee");
}

#[tokio::test]
async fn get_all_employees_empty() {
    let svc = make_service(MockEmployeeRepo::empty());
    let (list, total) = svc
        .get_all_employees(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            EmployeeFilterOptions::default(),
        )
        .await
        .expect("ok");
    assert_eq!(list.len(), 0);
    assert_eq!(total, 0);
}

#[tokio::test]
async fn delete_employee_success() {
    let mock = MockEmployeeRepo::empty();
    *mock.delete_emp_rows.lock().expect("lock") = 1;
    let svc = make_service(mock);
    let result = svc
        .delete_employee(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_employee_not_found() {
    let svc = make_service(MockEmployeeRepo::empty());
    let result = svc
        .delete_employee(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        )
        .await;
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn update_employee_not_found() {
    let svc = make_service(MockEmployeeRepo::empty());
    let req = UpdateEmployeeRequest {
        user_id: None,
        full_name: None,
        employee_id_number: None,
        tags: None,
        max_concurrent_workers: None,
        recruitment_source_id: None,
        force_transfer: None,
    };
    let result = svc
        .update_employee(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            req,
        )
        .await;
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn update_employee_success() {
    let ws = Uuid::new_v4();
    let mock = MockEmployeeRepo::empty();
    let updated = make_employee(ws);
    let updated_id = updated.id();
    *mock.updated_employee.lock().unwrap() = Some(updated.clone());
    let svc = make_service(mock);
    let req = UpdateEmployeeRequest {
        user_id: None,
        full_name: Some("Updated Name".into()),
        employee_id_number: None,
        tags: None,
        max_concurrent_workers: None,
        recruitment_source_id: None,
        force_transfer: None,
    };
    let result = svc
        .update_employee(
            updated_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            req,
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_employee_duplicate_user_conflict() {
    let ws = Uuid::new_v4();
    let target_emp_id = Uuid::new_v4();
    let existing_emp_id = Uuid::new_v4();
    let conflict_user_id = Uuid::new_v4();
    let mock = MockEmployeeRepo::empty();
    *mock.employee_id_by_user_id.lock().unwrap() = Some(existing_emp_id);
    let existing_emp = make_employee(ws);
    *mock.employee.lock().unwrap() = Some(existing_emp.clone());
    let svc = make_service(mock);
    let req = UpdateEmployeeRequest {
        user_id: Some(Some(conflict_user_id)),
        full_name: None,
        employee_id_number: None,
        tags: None,
        max_concurrent_workers: None,
        recruitment_source_id: None,
        force_transfer: None,
    };
    let result = svc
        .update_employee(
            target_emp_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            req,
        )
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
    assert_eq!(
        err.to_string(),
        "Validation error: error.user_already_linked:name=Test Employee"
    );
}

#[tokio::test]
async fn update_employee_duplicate_user_force() {
    let ws = Uuid::new_v4();
    let target_emp_id = Uuid::new_v4();
    let existing_emp_id = Uuid::new_v4();
    let conflict_user_id = Uuid::new_v4();
    let mock = MockEmployeeRepo::empty();
    *mock.employee_id_by_user_id.lock().unwrap() = Some(existing_emp_id);
    let updated_emp = make_employee(ws);
    *mock.updated_employee.lock().unwrap() = Some(updated_emp.clone());
    let svc = make_service(mock);
    let req = UpdateEmployeeRequest {
        user_id: Some(Some(conflict_user_id)),
        full_name: None,
        employee_id_number: None,
        tags: None,
        max_concurrent_workers: None,
        recruitment_source_id: None,
        force_transfer: Some(true),
    };
    let result = svc
        .update_employee(
            target_emp_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            req,
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_department_history_employee_not_found() {
    let svc = make_service(MockEmployeeRepo::empty());
    let result = svc
        .get_department_history(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        )
        .await;
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn transfer_employee_success() {
    let ws = Uuid::new_v4();
    let emp = make_employee(ws);
    let emp_id = emp.id();
    let mock = MockEmployeeRepo::empty();
    *mock.employee.lock().expect("lock") = Some(emp.clone());
    let svc = make_service(mock);
    let req = crate::models::department::TransferEmployeeRequest {
        department_id: Uuid::new_v4(),
        effective_date: chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        notes: None,
    };
    let result = svc
        .transfer_employee(
            emp_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            req,
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn transfer_employee_not_found() {
    let svc = make_service(MockEmployeeRepo::empty());
    let req = crate::models::TransferEmployeeRequest {
        department_id: Uuid::new_v4(),
        effective_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        notes: None,
    };
    let result = svc
        .transfer_employee(
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            req,
        )
        .await;
    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}
