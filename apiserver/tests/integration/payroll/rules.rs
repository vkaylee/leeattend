//! Integration tests for Payroll Rule Engine API

use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;
use tower::ServiceExt;

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use apiserver::services::payroll_rule_evaluator::PayrollRuleEvaluator;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::test]
async fn test_payroll_rules_crud_api() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. Create a Payroll Rule Definition (BHXH)
    let rule_payload = json!({
        "name": "Social Insurance",
        "description": "BHXH 8%",
        "code": "BHXH",
        "category": "statutory_deduction",
        "applies_to": "employee",
        "calculation_method": "percentage",
        "rule_config": {
            "rate": 8.0,
            "base": "gross_salary"
        },
        "is_mandatory": true,
        "display_order": 1
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{ws_id}/payroll-rules"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&rule_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_rule: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let rule_id = created_rule["id"].as_str().unwrap().to_string();
    assert_eq!(created_rule["code"], "BHXH");

    // 2. Fetch all rules
    let req = axum::http::Request::builder()
        .method("GET")
        .uri(format!("/api/workspaces/{ws_id}/payroll-rules"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let rules: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(rules.len(), 1);

    // 3. Create a Tier for a tiered rule
    let tier_payload = json!({
        "tier_order": 1,
        "min_amount": 0.0,
        "max_amount": 5000000.0,
        "rate": 5.0,
        "fixed_amount": 0.0
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&tier_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Assign the rule to the employee
    let assign_payload = json!({
        "assignee_type": "employee",
        "assignee_id": employee_id,
        "effective_from": "2026-01-01",
        "effective_to": null,
        "override_config": null
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&assign_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_payroll_rule_evaluator() {
    let (test_db, factory, _state, _app) = setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. Insert a rule definition (10% deduction)
    let rule_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO payroll_rule_definitions (id, workspace_id, code, name, category, applies_to, calculation_method, rule_config, is_mandatory, is_active)
        VALUES ($1, $2, 'TAX10', 'Tax 10%', 'statutory_deduction', 'employee', 'percentage', '{"rate": 10.0, "base": "gross_salary"}', true, true)
        "#,
        rule_id,
        ws_id
    )
    .execute(&test_db.pool())
    .await
    .unwrap();

    // 2. Assign the rule to the employee
    sqlx::query!(
        r#"
        INSERT INTO payroll_rule_assignments (id, workspace_id, rule_definition_id, assignee_type, assignee_id, effective_from)
        VALUES ($1, $2, $3, 'employee', $4, '2026-01-01')
        "#,
        Uuid::new_v4(),
        ws_id,
        rule_id,
        employee_id
    )
    .execute(&test_db.pool())
    .await
    .unwrap();

    // 3. Evaluate the rule
    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    let gross_salary = Decimal::from_str("10000.0").unwrap();

    let mut base_values = HashMap::new();
    base_values.insert("gross_salary".to_string(), gross_salary);

    let results = PayrollRuleEvaluator::evaluate_rules(
        &test_db.pool(),
        ws_id,
        employee_id,
        None,
        start_date,
        end_date,
        &base_values,
    )
    .await
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule_code, "TAX10");
    assert_eq!(results[0].amount, Decimal::from_str("1000.0").unwrap()); // 10% of 10000
}
