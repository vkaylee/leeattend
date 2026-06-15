use super::helpers::{make_service, MockEmployeeRepo};
use uuid::Uuid;

#[tokio::test]
async fn create_employee_success() {
    let ws = Uuid::new_v4();
    let svc = make_service(MockEmployeeRepo::empty());
    let emp = crate::models::CreateEmployeeRequest {
        user_id: None,
        full_name: "New Employee".into(),
        employee_id_number: crate::models::SanitizedEmployeeId::new("EMP001"),
        hire_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        tags: None,
        recruitment_source_id: None,
        primary_location_id: Some(Uuid::new_v4()),
        employer_id: Uuid::new_v4(),
        department_id: None,
    };
    let result = svc
        .create_employee(
            &crate::diesel_pool::TenantAuthToken::system_override(ws),
            emp,
        )
        .await;
    assert!(result.is_ok());
}
