mod common;

use apiserver::models::{PunchInRequest, PunchOutRequest, UpdateTimeEntryRequest};
use apiserver::services::time_tracking_service::TimeTrackingService;
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_punch_in_already_active() {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool().clone());
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // First punch in
    let req = PunchInRequest {
        employee_id,
        entry_type: Some("work".to_string()),
        note: None,
        is_break_overtime: None,
    };
    TimeTrackingService::punch_in(&pool, ws_id, req.clone())
        .await
        .unwrap();

    // Second punch in (should fail)
    let result = TimeTrackingService::punch_in(&pool, ws_id, req).await;
    assert_eq!(result.unwrap_err(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_punch_in_inactive_employee() {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool().clone());
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // Deactivate employee
    sqlx::query("UPDATE employees SET is_active = FALSE WHERE id = $1")
        .bind(employee_id)
        .execute(&pool)
        .await
        .unwrap();

    let req = PunchInRequest {
        employee_id,
        entry_type: Some("work".to_string()),
        note: None,
        is_break_overtime: None,
    };
    let result = TimeTrackingService::punch_in(&pool, ws_id, req).await;
    assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_punch_out_not_active() {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool().clone());
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    let req = PunchOutRequest { note: None };
    let result = TimeTrackingService::punch_out(&pool, ws_id, employee_id, req).await;
    assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_time_entry_update_and_delete() {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool().clone());
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // Create an entry manually using factory
    let entry_id = factory
        .time_entry()
        .with_employee(employee_id)
        .create()
        .await;

    // Update
    let update_req = UpdateTimeEntryRequest {
        punch_in_time: Some(Utc::now() - Duration::hours(1)),
        punch_out_time: Some(Utc::now()),
        entry_type: Some("break".to_string()),
        note: Some("Updated note".to_string()),
    };
    let updated = TimeTrackingService::update_time_entry(&pool, ws_id, entry_id, update_req)
        .await
        .unwrap();
    assert_eq!(updated.entry_type, "break");
    assert_eq!(updated.note.unwrap(), "Updated note");
    assert!(updated.duration_minutes.is_some());

    // Delete
    TimeTrackingService::delete_time_entry(&pool, ws_id, entry_id)
        .await
        .unwrap();

    // Verify deleted
    let result =
        apiserver::repositories::time_entry_repository::get_time_entry_by_id(&pool, entry_id)
            .await
            .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_access_time_entry_wrong_workspace() {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool().clone());
    let pool = test_db.pool();

    let user1 = factory.user().create().await;
    let ws1 = factory.workspace().create_with_owner(user1).await;
    let emp1 = factory.employee().with_workspace(ws1).create().await;
    let entry_id = factory.time_entry().with_employee(emp1).create().await;

    let user2 = factory.user().create().await;
    let ws2 = factory.workspace().create_with_owner(user2).await;

    // Try to update entry from ws1 using ws2 context
    let update_req = UpdateTimeEntryRequest {
        punch_in_time: None,
        punch_out_time: None,
        entry_type: None,
        note: Some("Evil update".to_string()),
    };
    let result = TimeTrackingService::update_time_entry(&pool, ws2, entry_id, update_req).await;
    assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);

    // Try to delete
    let result = TimeTrackingService::delete_time_entry(&pool, ws2, entry_id).await;
    assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_time_history_service() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    factory.time_entry().with_employee(emp_id).create().await;

    let history = TimeTrackingService::get_employee_history(
        &pool,
        ws_id,
        emp_id,
        Utc::now() - Duration::days(1),
        Utc::now() + Duration::days(1),
    )
    .await
    .unwrap();
    assert_eq!(history.len(), 1);
}

use apiserver::agent_manager::AgentManager;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
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
        jwt_secret: "test_secret_tt".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_get_time_history_api() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    factory.time_entry().with_employee(emp_id).create().await;

    let start = (Utc::now() - Duration::days(1))
        .to_rfc3339()
        .replace("+", "%2B");
    let end = (Utc::now() + Duration::days(1))
        .to_rfc3339()
        .replace("+", "%2B");

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/time/history/{}?start={}&end={}",
                    ws_id, emp_id, start, end
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_punch_in_break_overtime_inheritance() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Create shift with break_is_overtime = true
    let shift_id = sqlx::query_scalar!(
        "INSERT INTO shifts (workspace_id, name, start_time, end_time, break_duration_minutes, break_is_overtime) VALUES ($1, 'OT Break', '09:00:00', '17:00:00', 60, true) RETURNING id",
        ws_id
    ).fetch_one(&pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
        emp_id,
        shift_id,
        Utc::now().date_naive()
    )
    .execute(&pool)
    .await
    .unwrap();

    let req = PunchInRequest {
        employee_id: emp_id,
        entry_type: Some("break".to_string()),
        is_break_overtime: None, // Should inherit
        note: None,
    };

    let entry = TimeTrackingService::punch_in(&pool, ws_id, req)
        .await
        .unwrap();
    assert!(entry.is_break_overtime);
}

#[tokio::test]
async fn test_punch_out_with_note() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let req_in = PunchInRequest {
        employee_id: emp_id,
        entry_type: None,
        is_break_overtime: None,
        note: Some("Morning".to_string()),
    };
    TimeTrackingService::punch_in(&pool, ws_id, req_in)
        .await
        .unwrap();

    let req_out = PunchOutRequest {
        note: Some("Evening".to_string()),
    };
    let entry = TimeTrackingService::punch_out(&pool, ws_id, emp_id, req_out)
        .await
        .unwrap();
    assert!(entry.note.unwrap().contains("Evening"));
}
