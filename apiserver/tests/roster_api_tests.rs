mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::{BulkRosterRequest, RosterItem, ShiftRoster};
use apiserver::repositories::roster_repository;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::{NaiveDate, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use uuid::Uuid;

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

    // Setup test storage

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_bulk_upsert_roster_api() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let shift_id = factory.shift().with_workspace(workspace_id).create().await;

    let req = BulkRosterRequest {
        items: vec![
            RosterItem {
                date: NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
                shift_id: Some(shift_id),
                notes: Some("Day 1".to_string()),
            },
            RosterItem {
                date: NaiveDate::from_ymd_opt(2026, 1, 11).unwrap(),
                shift_id: None, // Day Off
                notes: Some("Day 2 Off".to_string()),
            },
        ],
    };

    use tower::ServiceExt;
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}/rosters/bulk",
                    workspace_id, employee_id
                ))
                .header("Content-Type", "application/json")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify in DB
    let rosters: Vec<ShiftRoster> = sqlx::query_as::<_, ShiftRoster>(
        "SELECT * FROM shift_rosters WHERE employee_id = $1 ORDER BY date ASC",
    )
    .bind(employee_id)
    .fetch_all(&factory.pool)
    .await
    .unwrap();

    assert_eq!(rosters.len(), 2);
    assert_eq!(
        rosters[0].date,
        NaiveDate::from_ymd_opt(2026, 1, 10).unwrap()
    );
    assert_eq!(rosters[0].shift_id, Some(shift_id));
    assert_eq!(rosters[1].shift_id, None);
}

#[tokio::test]
async fn test_delete_roster_reverts_to_default() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let fixed_shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Fixed")
        .create()
        .await;
    let roster_shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Roster")
        .create()
        .await;
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // 1. Assign Fixed Shift
    sqlx::query("INSERT INTO employee_shifts (id, employee_id, shift_id, start_date) VALUES ($1, $2, $3, $4)")
        .bind(Uuid::new_v4())
        .bind(employee_id)
        .bind(fixed_shift_id)
        .bind(date)
        .execute(&factory.pool).await.unwrap();

    // 2. Override with Roster Shift
    sqlx::query(
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(employee_id)
    .bind(roster_shift_id)
    .bind(date)
    .execute(&factory.pool)
    .await
    .unwrap();

    // Verify current effective is Roster
    let (effective, _) = roster_repository::get_effective_shift(&factory.pool, employee_id, date)
        .await
        .unwrap();
    assert_eq!(effective, Some(roster_shift_id));

    // 3. Delete Roster Override via API
    use tower::ServiceExt;
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}/rosters/{}",
                    workspace_id, employee_id, date
                ))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 4. Verify effective is now Fixed
    let (effective_after, _) = roster_repository::get_effective_shift(&factory.pool, employee_id, date)
        .await
        .unwrap();
    assert_eq!(effective_after, Some(fixed_shift_id));
}

#[tokio::test]
async fn test_single_upsert_and_get_rosters() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let shift_id = factory.shift().with_workspace(workspace_id).create().await;
    let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();

    // 1. Single Upsert
    let req = serde_json::json!({
        "employee_id": employee_id,
        "shift_id": shift_id,
        "date": date,
        "notes": "Single update"
    });

    use tower::ServiceExt;
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}/rosters",
                    workspace_id, employee_id
                ))
                .header("Content-Type", "application/json")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Get Employee Rosters
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}/rosters?start_date={}&end_date={}",
                    workspace_id, employee_id, date, date
                ))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let rosters: Vec<ShiftRoster> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(rosters.len(), 1);

    // Test invalid date (should fallback to default)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}/rosters?start_date=INVALID",
                    workspace_id, employee_id
                ))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_assignment_history_api() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    // Create an assignment
    sqlx::query("INSERT INTO employee_shifts (id, employee_id, shift_id, start_date) VALUES ($1, $2, $3, $4)")
        .bind(Uuid::new_v4()).bind(emp_id).bind(shift_id).bind(Utc::now().date_naive()).execute(&factory.pool).await.unwrap();

    use tower::ServiceExt;
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/assignment-history", ws_id))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test get workspace schedules
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/schedules?start_date=2026-01-01&end_date=2026-01-31",
                    ws_id
                ))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test invalid date
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/schedules?start_date=INVALID&end_date=2026-01-31",
                    ws_id
                ))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
