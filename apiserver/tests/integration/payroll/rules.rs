//! Integration tests for Payroll Rule Engine API

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::PayrollRepository;
use apiserver::services::payroll_rule_evaluator::PayrollRuleEvaluator;
use apiserver::AppState;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

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
        .uri(format!("/api/v1/workspaces/{ws_id}/payroll-rules"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&rule_payload).unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_rule: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let rule_id = created_rule["id"].as_str().unwrap().to_string();
    assert_eq!(created_rule["code"], "BHXH");

    // 2. Fetch all rules
    let req = axum::http::Request::builder()
        .method("GET")
        .uri(format!("/api/v1/workspaces/{ws_id}/payroll-rules"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
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
        .uri(format!(
            "/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers"
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&tier_payload).unwrap(),
        ))
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
        .uri(format!(
            "/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments"
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&assign_payload).unwrap(),
        ))
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

    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let rule_repo = apiserver::repositories::DieselPayrollRuleRepository::new(diesel_pools.clone());

    // 1. Insert a rule definition (10% deduction)
    let rule_def = rule_repo
        .create_definition(
            apiserver::repositories::payroll_rule_repository::CreatePayrollRuleDefinitionArgs {
                workspace_id: ws_id,
                code: "TAX10".to_string(),
                name: "Tax 10%".to_string(),
                category: apiserver::models::enums::PayrollRuleCategory::StatutoryDeduction,
                applies_to: apiserver::models::enums::RuleAppliesTo::Employee,
                calculation_method: apiserver::models::enums::PayrollCalculationMethod::Percentage,
                rule_config: serde_json::json!({"rate": 10.0, "base": "gross_salary"}),
                is_mandatory: true,
                display_order: 0,
                description: None,
            },
        )
        .await
        .unwrap();
    let rule_id = rule_def.id;

    // 2. Assign the rule to the employee
    use apiserver::repositories::payroll_rule_repository::PayrollRuleRepository;
    rule_repo
        .assign_rule(
            apiserver::repositories::payroll_rule_repository::AssignPayrollRuleArgs {
                workspace_id: ws_id,
                rule_definition_id: rule_id,
                assignee_type: apiserver::models::enums::AssigneeType::Employee,
                assignee_id: Some(employee_id),
                effective_from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                effective_to: None,
                override_config: None,
            },
        )
        .await
        .unwrap();

    // 3. Evaluate the rule
    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    let gross_salary = Decimal::from_str("10000.0").unwrap();

    let mut base_values = HashMap::new();
    base_values.insert("gross_salary".to_string(), gross_salary);

    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let pr = apiserver::repositories::DieselPayrollRepository::new(diesel_pools.clone());

    let assignments = pr
        .get_resolved_rule_assignments(ws_id, employee_id, None, start_date, end_date)
        .await
        .unwrap();

    let mut rule_assignments_with_tiers = Vec::new();
    for assignment in assignments {
        let tiers = pr
            .get_payroll_rule_tiers(ws_id, assignment.rule_definition_id)
            .await
            .unwrap();
        rule_assignments_with_tiers.push(
            apiserver::services::financial_calculator::RuleAssignmentWithTiers {
                assignment,
                tiers,
            },
        );
    }

    let results =
        PayrollRuleEvaluator::evaluate_rules(&rule_assignments_with_tiers, &base_values).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule_code, "TAX10");
    assert_eq!(results[0].amount, Decimal::from_str("1000.0").unwrap()); // 10% of 10000
}
