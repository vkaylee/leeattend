//! Payroll batch calculation tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{BatchCalculatePayrollRequest, BatchCalculatePayrollResponse};
use reqwest::StatusCode;
use rust_decimal::Decimal;
use uuid::Uuid;

#[tokio::test]
async fn test_batch_calculate_payroll() {
    let (address, _, user_id, factory, pool): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    // 1. Create employees
    let emp1_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Emp1")
        .with_employee_id_number("E001")
        .create()
        .await;

    let emp2_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Emp2")
        .with_employee_id_number("E002")
        .create()
        .await;

    // 2. Setup base salary
    let sc_def_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "INSERT INTO salary_component_definitions (workspace_id, name, code) VALUES ($1, 'Base', 'BASE') RETURNING id",
        diesel::sql_types::Uuid, ws_id
    );

    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, emp1_id,
        diesel::sql_types::Uuid, sc_def_id,
        diesel::sql_types::Numeric, Decimal::from(10_000_000),
        diesel::sql_types::Date, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, emp2_id,
        diesel::sql_types::Uuid, sc_def_id,
        diesel::sql_types::Numeric, Decimal::from(12_000_000),
        diesel::sql_types::Date, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    // 3. Test Batch Calculation
    let req_body = BatchCalculatePayrollRequest {
        employee_ids: vec![emp1_id, emp2_id],
        year: 2026,
        month: 1,
    };

    let res = client
        .post(format!(
            "{address}/api/workspaces/{ws_id}/payroll/calculate-batch"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&req_body)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let batch_res: BatchCalculatePayrollResponse = res.json().await.unwrap();
    assert_eq!(batch_res.results.len(), 2);

    for result in batch_res.results {
        assert_eq!(result.status, "success");
        assert!(result.payslip.is_some());
    }
}
