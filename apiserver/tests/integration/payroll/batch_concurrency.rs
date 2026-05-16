//! Test batch payroll concurrency limit (Issue 4)
//!
//! Verifies that `calculate_batch_payslips` works correctly with `buffer_unordered(10)`
//! when processing many employees simultaneously.

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{BatchCalculatePayrollRequest, BatchCalculatePayrollResponse};
use reqwest::StatusCode;
use rust_decimal::Decimal;
use uuid::Uuid;

#[tokio::test]
async fn test_batch_payroll_handles_many_employees() {
    let (address, _, user_id, factory, pool): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    // Setup base salary component
    let sc_def_id: Uuid = crate::test_query_scalar_uuid!(&pool.diesel_pools.api,
        "INSERT INTO salary_component_definitions (workspace_id, name, code) VALUES ($1, 'Base', 'BASE') RETURNING id",
        diesel::sql_types::Uuid, ws_id
    );

    // Create 15 employees to exercise concurrency limit (buffer_unordered(10))
    let mut employee_ids = Vec::new();
    for i in 1..=15 {
        let emp_id = factory
            .employee()
            .with_workspace(ws_id)
            .with_full_name(&format!("Employee {i}"))
            .with_employee_id_number(&format!("E{i:03}"))
            .create()
            .await;

        // Assign salary to each employee
        crate::test_execute!(&pool.diesel_pools.api,
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, emp_id,
        diesel::sql_types::Uuid, sc_def_id,
        diesel::sql_types::Numeric, Decimal::from(5_000_000 + (i * 100_000)),
        diesel::sql_types::Date, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

        employee_ids.push(emp_id);
    }

    // Batch calculate payroll for all 15 employees
    let req_body = BatchCalculatePayrollRequest {
        employee_ids: employee_ids.clone(),
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

    // Verify all 15 employees processed successfully
    assert_eq!(batch_res.results.len(), 15);

    for result in batch_res.results {
        assert_eq!(
            result.status, "success",
            "Employee {} failed: {:?}",
            result.employee_id, result.error_message
        );
        assert!(
            result.payslip.is_some(),
            "Employee {} has no payslip",
            result.employee_id
        );
    }
}
