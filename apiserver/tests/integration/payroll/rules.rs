//! Integration tests for Payroll Rule Engine API

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::{PayrollRepository, PayrollRuleRepository};
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

#[tokio::test]
async fn test_payroll_rule_repository_direct() {
    let (test_db, factory, _state, _app) = setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let repo = apiserver::repositories::DieselPayrollRuleRepository::new(diesel_pools.clone());

    // 1. Create a definition
    let def = repo
        .create_definition(
            apiserver::repositories::payroll_rule_repository::CreatePayrollRuleDefinitionArgs {
                workspace_id: ws_id,
                code: "TEST_RULE".to_string(),
                name: "Test Rule".to_string(),
                category: apiserver::models::enums::PayrollRuleCategory::Allowance,
                applies_to: apiserver::models::enums::RuleAppliesTo::Employee,
                calculation_method: apiserver::models::enums::PayrollCalculationMethod::Fixed,
                rule_config: serde_json::json!({}),
                is_mandatory: false,
                display_order: 10,
                description: Some("Test rule desc".to_string()),
            },
        )
        .await
        .unwrap();

    assert_eq!(def.code, "TEST_RULE");
    assert_eq!(def.name, "Test Rule");

    // 2. Get definitions
    let defs = repo.get_definitions(ws_id).await.unwrap();
    assert!(defs.iter().any(|d| d.id == def.id));

    // 3. Create a tier
    let min_amt = Decimal::from_str("100.0").unwrap();
    let max_amt = Decimal::from_str("1000.0").unwrap();
    let rate = Decimal::from_str("0.15").unwrap();
    let fixed_amt = Decimal::from_str("50.0").unwrap();

    let tier = repo
        .create_tier(ws_id, def.id, 1, min_amt, Some(max_amt), rate, fixed_amt)
        .await
        .unwrap();

    assert_eq!(tier.tier_order, 1);
    assert_eq!(tier.min_amount, min_amt);
    assert_eq!(tier.max_amount, Some(max_amt));

    // 4. Get tiers
    let tiers = repo.get_tiers_for_rule(ws_id, def.id).await.unwrap();
    assert_eq!(tiers.len(), 1);
    assert_eq!(tiers[0].id, tier.id);

    // 5. Assign rule
    let assignment = repo
        .assign_rule(
            apiserver::repositories::payroll_rule_repository::AssignPayrollRuleArgs {
                workspace_id: ws_id,
                rule_definition_id: def.id,
                assignee_type: apiserver::models::enums::AssigneeType::Employee,
                assignee_id: Some(employee_id),
                effective_from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                effective_to: Some(chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
                override_config: Some(serde_json::json!({"custom_rate": 0.2})),
            },
        )
        .await
        .unwrap();

    assert_eq!(assignment.rule_definition_id, def.id);
    assert_eq!(assignment.assignee_id, Some(employee_id));

    // 5b. Attempt to assign overlapping rule to trigger UniqueViolation overlap constraint
    let overlap_res = repo
        .assign_rule(
            apiserver::repositories::payroll_rule_repository::AssignPayrollRuleArgs {
                workspace_id: ws_id,
                rule_definition_id: def.id,
                assignee_type: apiserver::models::enums::AssigneeType::Employee,
                assignee_id: Some(employee_id),
                effective_from: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                effective_to: Some(chrono::NaiveDate::from_ymd_opt(2026, 8, 31).unwrap()),
                override_config: None,
            },
        )
        .await;
    assert!(overlap_res.is_err());
    let overlap_err = overlap_res.unwrap_err();
    assert!(matches!(
        overlap_err,
        apiserver::errors::AppError::Conflict(_, _)
    ));

    // 5c. Attempt to create a definition with duplicate code to trigger UniqueViolation on definitions table
    let duplicate_def_res = repo
        .create_definition(
            apiserver::repositories::payroll_rule_repository::CreatePayrollRuleDefinitionArgs {
                workspace_id: ws_id,
                code: "TEST_RULE".to_string(), // same code
                name: "Test Rule Duplicate".to_string(),
                category: apiserver::models::enums::PayrollRuleCategory::Allowance,
                applies_to: apiserver::models::enums::RuleAppliesTo::Employee,
                calculation_method: apiserver::models::enums::PayrollCalculationMethod::Fixed,
                rule_config: serde_json::json!({}),
                is_mandatory: false,
                display_order: 10,
                description: None,
            },
        )
        .await;
    assert!(duplicate_def_res.is_err());

    // 6. Get assignments
    let assignments = repo.get_assignments_for_rule(ws_id, def.id).await.unwrap();
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].id, assignment.id);

    // 7. Delete assignment
    repo.delete_assignment(ws_id, def.id, assignment.id)
        .await
        .unwrap();
    let assignments_after = repo.get_assignments_for_rule(ws_id, def.id).await.unwrap();
    assert!(assignments_after.is_empty());

    // 8. Delete tier
    repo.delete_tier(ws_id, def.id, tier.id).await.unwrap();
    let tiers_after = repo.get_tiers_for_rule(ws_id, def.id).await.unwrap();
    assert!(tiers_after.is_empty());
}
