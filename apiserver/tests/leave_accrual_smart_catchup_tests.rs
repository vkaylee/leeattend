mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::NaiveDate;
use common::db::TestDatabase;
use common::factory::TestFactory;
use rust_decimal::prelude::FromPrimitive;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(
        pool.clone(),
    ));
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_smart_catchup_accrual() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup Data
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory.employee().with_workspace(ws_id).create().await;

    // 2. Create Policy (1 day/month)
    let config = serde_json::json!({
        "frequency": "monthly",
        "accrual_amount": 1.0,
        "accrual_timing": "start_of_period",
        "seniority_bonus": [],
        "max_balance": null,
        "reset_month": null
    });

    let policy_id = factory.leave_policy()
        .with_workspace(ws_id)
        .with_config(config)
        .create()
        .await;

    // 3. Assign Policy retroactively (3 months ago)
    // Assume current date is 2026-01-21.
    // Effective from: 2025-10-01.
    // Expected accruals: 2025-10-01, 2025-11-01, 2025-12-01, 2026-01-01 (Total 4 days)
    let effective_from = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
    
    // Manually assign to control effective date precisely
    sqlx::query!(
        r#"
        INSERT INTO leave_policy_assignments (workspace_id, assignee_type, assignee_id, policy_id, effective_from)
        VALUES ($1, 'employee', $2, $3, $4)
        "#,
        ws_id,
        emp, // emp is Uuid
        policy_id, // policy_id is Uuid
        effective_from
    )
    .execute(&factory.pool)
    .await
    .unwrap();

    // 4. Trigger Accrual Run (Simulate running on 2026-01-21)
    let target_date = NaiveDate::from_ymd_opt(2026, 1, 21).unwrap();
    let body = serde_json::json!({ "date": target_date.to_string() });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/accrual/run", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string()) // As Admin
                .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify Logs (Should have 4 entries)
    let logs = sqlx::query!(
        "SELECT accrual_date, amount FROM leave_accrual_logs WHERE employee_id = $1 ORDER BY accrual_date",
        emp
    )
    .fetch_all(&factory.pool)
    .await
    .unwrap();

    assert_eq!(logs.len(), 4, "Should create 4 accrual logs for catch-up");
    
    // Verify dates
    assert_eq!(logs[0].accrual_date, NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
    assert_eq!(logs[1].accrual_date, NaiveDate::from_ymd_opt(2025, 11, 1).unwrap());
    assert_eq!(logs[2].accrual_date, NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
    assert_eq!(logs[3].accrual_date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());

    // 6. Verify Balances
    // Note: Balances are split by year.
    // 2025: 3.0 days
    // 2026: 1.0 day
    
    let balance_2025 = sqlx::query!(
        "SELECT accrued_amount FROM leave_balances WHERE employee_id = $1 AND year = 2025",
        emp
    )
    .fetch_one(&factory.pool)
    .await
    .unwrap();
    
    let balance_2026 = sqlx::query!(
        "SELECT accrued_amount FROM leave_balances WHERE employee_id = $1 AND year = 2026",
        emp
    )
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    assert_eq!(balance_2025.accrued_amount, rust_decimal::Decimal::from_f64(3.0).unwrap());
    assert_eq!(balance_2026.accrued_amount, rust_decimal::Decimal::from_f64(1.0).unwrap());
}
