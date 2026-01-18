mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::models::{CreateWorkingPatternRequest, WorkingPatternDetailRequest, AssignPatternRequest, MemberRole};
use apiserver::AppState;
use chrono::NaiveDate;
use common::factory::TestFactory;
use common::db::TestDatabase;
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

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret_123".to_string(),
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
async fn test_create_and_list_patterns() {
    let (address, test_db) = spawn_app().await;
    let factory = TestFactory::new(test_db.pool());
    let client = reqwest::Client::new();
    
    // 1. Setup Admin
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);

    let shift_id = factory.shift().with_workspace(workspace_id).create().await;

    // 2. Create Pattern
    let req = CreateWorkingPatternRequest {
        name: "Standard Office".to_string(),
        description: Some("Mon-Fri 9-5".to_string()),
        details: vec![
            WorkingPatternDetailRequest { day_of_week: 1, shift_id: Some(shift_id) }, // Mon
            WorkingPatternDetailRequest { day_of_week: 2, shift_id: Some(shift_id) }, // Tue
            WorkingPatternDetailRequest { day_of_week: 6, shift_id: None },           // Sat (Off)
        ],
    };

    let response = client
        .post(&format!("{}/api/workspaces/{}/working-patterns", address, workspace_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let pattern: apiserver::models::WorkingPattern = response.json().await.expect("Failed to parse pattern");
    assert_eq!(pattern.name, "Standard Office");

    // 3. List Patterns
    let list_response = client
        .get(&format!("{}/api/workspaces/{}/working-patterns", address, workspace_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .send()
        .await
        .expect("Failed to list");

    assert_eq!(list_response.status(), StatusCode::OK);
    let patterns: Vec<apiserver::models::WorkingPattern> = list_response.json().await.expect("Failed to parse list");
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].id, pattern.id);

    // 4. Get Details
    let details_response = client
        .get(&format!("{}/api/workspaces/{}/working-patterns/{}", address, workspace_id, pattern.id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .send()
        .await
        .expect("Failed to get details");

    assert_eq!(details_response.status(), StatusCode::OK);
    let details: Vec<apiserver::models::WorkingPatternDetail> = details_response.json().await.expect("Failed to parse details");
    assert_eq!(details.len(), 3); // Mon, Tue, Sat
}

#[tokio::test]
async fn test_assign_pattern_api() {
    let (address, test_db) = spawn_app().await;
    let factory = TestFactory::new(test_db.pool());
    let client = reqwest::Client::new();
    
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);
    let employee_id = factory.employee().with_workspace(workspace_id).create().await;

    // Create a pattern first
    let pattern = apiserver::repositories::working_pattern_repository::create_pattern(
        &factory.pool,
        workspace_id,
        "Test Pattern",
        None
    ).await.unwrap();

    // Assign via API
    let req = AssignPatternRequest {
        pattern_id: pattern.id,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: None,
    };

    let response = client
        .post(&format!("{}/api/workspaces/{}/employees/{}/working-patterns", address, workspace_id, employee_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to assign");

    assert_eq!(response.status(), StatusCode::OK);
    
    // Verify History
    let history_response = client
        .get(&format!("{}/api/workspaces/{}/employees/{}/working-patterns", address, workspace_id, employee_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .send()
        .await
        .expect("Failed to get history");

    let history: Vec<apiserver::models::EmployeeWorkingPattern> = history_response.json().await.expect("Failed to parse history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].pattern_id, pattern.id);
}

#[tokio::test]
async fn test_rbac_security() {
    let (address, test_db) = spawn_app().await;
    let factory = TestFactory::new(test_db.pool());
    let client = reqwest::Client::new();
    
    // 1. Owner Setup
    let owner_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(owner_id).await;
    
    // 2. Regular Member Setup
    let member_id = factory.user().create().await;
    factory.add_member(workspace_id, member_id, MemberRole::Member).await;
    let member_token = factory.auth().create_token(member_id);

    // 3. Attempt Create Pattern as Member
    let req = CreateWorkingPatternRequest {
        name: "Hacker Pattern".to_string(),
        description: None,
        details: vec![],
    };

    let response = client
        .post(&format!("{}/api/workspaces/{}/working-patterns", address, workspace_id))
        .header("Authorization", format!("Bearer {}", member_token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to send");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_bulk_assign_pattern_api() {
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
        "Bulk Pattern",
        None
    ).await.unwrap();

    let emp1 = factory.employee().with_workspace(workspace_id).create().await;
    let emp2 = factory.employee().with_workspace(workspace_id).create().await;

    let req = apiserver::models::BulkAssignPatternRequest {
        employee_ids: vec![emp1, emp2],
        pattern_id: pattern.id,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: None,
    };

    let response = client
        .post(&format!("{}/api/workspaces/{}/working-patterns/bulk-assign", address, workspace_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to send");

    assert_eq!(response.status(), StatusCode::OK);
    
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM employee_working_patterns WHERE pattern_id = $1")
        .bind(pattern.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        
    assert_eq!(count, 2);
}
