mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::models::{BulkAssignPatternRequest, BulkAssignResponse};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use chrono::NaiveDate;
use common::db::TestDatabase;
use common::factory::TestFactory;
use reqwest::StatusCode;
use std::sync::Arc;

async fn spawn_app() -> (String, TestDatabase) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();

    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    // Setup test storage

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret_123".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{}", port), test_db)
}

#[tokio::test]
async fn test_bulk_assign_happy_path() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);

    let pattern = apiserver::repositories::working_pattern_repository::create_pattern(
        &pool,
        workspace_id,
        "Pattern A",
        None,
    )
    .await
    .unwrap();

    let emp1 = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let emp2 = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // Assign to both
    let req = BulkAssignPatternRequest {
        employee_ids: vec![emp1, emp2],
        pattern_id: pattern.id,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: None,
    };

    let response = client
        .post(format!(
            "{}/api/workspaces/{}/working-patterns/bulk-assign",
            address, workspace_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to send");

    assert_eq!(response.status(), StatusCode::OK);
    let result: BulkAssignResponse = response.json().await.unwrap();

    assert_eq!(result.success_count, 2);
    assert!(result.failed_employees.is_empty());

    // Verify DB
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM employee_working_patterns WHERE pattern_id = $1")
            .bind(pattern.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_bulk_assign_conflict_rollback() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);

    let pattern_a = apiserver::repositories::working_pattern_repository::create_pattern(
        &pool,
        workspace_id,
        "Pattern A",
        None,
    )
    .await
    .unwrap();

    let pattern_b = apiserver::repositories::working_pattern_repository::create_pattern(
        &pool,
        workspace_id,
        "Pattern B",
        None,
    )
    .await
    .unwrap();

    let emp1 = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let emp2 = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    // 1. Manually assign Pattern A to Emp1
    apiserver::repositories::working_pattern_repository::assign_pattern(
        &pool,
        workspace_id,
        emp1,
        pattern_a.id,
        start_date,
        None,
    )
    .await
    .unwrap();

    // 2. Try to Bulk Assign Pattern B to Emp1 (Conflict) and Emp2 (Clean)
    let req = BulkAssignPatternRequest {
        employee_ids: vec![emp1, emp2],
        pattern_id: pattern_b.id,
        start_date,
        end_date: None,
    };

    let response = client
        .post(format!(
            "{}/api/workspaces/{}/working-patterns/bulk-assign",
            address, workspace_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to send");

    assert_eq!(response.status(), StatusCode::OK);
    let result: BulkAssignResponse = response.json().await.unwrap();

    // 3. Verify Result: Should fail for ALL because of conflict logic (Robustness)
    // "If one fails, the entire batch rolls back" logic implemented in Step 3.2.2
    assert_eq!(result.success_count, 0);
    assert_eq!(result.failed_employees.len(), 1);
    assert_eq!(result.failed_employees[0], emp1);

    // 4. Verify DB: Emp2 should NOT have Pattern B
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM employee_working_patterns WHERE employee_id = $1 AND pattern_id = $2",
    )
    .bind(emp2)
    .bind(pattern_b.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        count, 0,
        "Emp2 should not have been assigned due to rollback"
    );
}
