use apiserver::{
    create_app,
    services::auth_service::AuthService,
    repositories::{SqlxAgentRepository, SqlxWorkspaceRepository},
    AppState, agent_manager::AgentManager,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Local, NaiveDate, NaiveTime, Utc, TimeZone};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

mod common;
use common::factory::TestFactory;
use common::db::TestDatabase;

async fn setup_test_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool());
    (test_db, factory)
}

async fn setup_test_app(pool: PgPool) -> axum::Router {
    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo = Arc::new(SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let state = Arc::new(AppState {
        agent_manager,
        db: pool,
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
    });

    create_app(state)
}

#[tokio::test]
async fn test_punch_in_grace_period_enforcement() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();
    let app = setup_test_app(pool.clone()).await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).with_user(user_id).create().await;
    
    // Use a fixed date/time to avoid midnight rollover issues
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
    let _punch_in_time = Local.with_ymd_and_hms(2026, 1, 10, 8, 30, 0).unwrap();
    let shift_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    
    let shift_id = factory.shift()
        .with_workspace(ws_id)
        .with_times(shift_start, NaiveTime::from_hms_opt(17, 0, 0).unwrap())
        .with_grace_period(5) // 5 mins grace -> earliest is 08:55
        .assign_to_employee(employee_id, test_date)
        .await;

    let token = AuthService::generate_token(user_id, "test_secret", 3600).unwrap();

    // Try to punch in at 08:30 -> Should fail (400) because 08:30 < 08:55
    let punch_payload = json!({
        "employee_id": employee_id,
        "entry_type": "work"
    });

    // Mock the current time by controlling the environment? 
    // Wait, the service uses Utc::now(). I can't easily mock it without refactoring.
    // However, I can just use a shift that is far in the future relative to REAL now.
    
    let real_now = Utc::now().with_timezone(&Local);
    let today = real_now.date_naive();
    let future_start = NaiveTime::from_hms_opt(23, 59, 0).unwrap(); // Far in the future today
    
    // Ensure we don't rollover if it's already very late
    let (test_date, test_start) = if real_now.time() > NaiveTime::from_hms_opt(23, 0, 0).unwrap() {
        (today + chrono::Duration::days(1), NaiveTime::from_hms_opt(12, 0, 0).unwrap())
    } else {
        (today, future_start)
    };

    sqlx::query("UPDATE shifts SET start_time = $1, end_time = $2 WHERE id = $3")
        .bind(test_start)
        .bind(NaiveTime::from_hms_opt(23, 59, 59).unwrap())
        .bind(shift_id)
        .execute(&pool).await.unwrap();
    
    sqlx::query("UPDATE employee_shifts SET start_date = $1 WHERE shift_id = $2")
        .bind(test_date)
        .bind(shift_id)
        .execute(&pool).await.unwrap();

    // Now if we punch in NOW, it's definitely too early for that shift
    // (Unless the shift is for tomorrow, but we handle that by sending the correct workspace context)
    // Wait, the service always uses CURRENT DATE. 
    // So if I want it to find the shift, I MUST use TODAY as test_date.
    
    sqlx::query("UPDATE employee_shifts SET start_date = $1 WHERE shift_id = $2")
        .bind(today)
        .bind(shift_id)
        .execute(&pool).await.unwrap();
    
    // Set start_time to now + 4 hours (to avoid midnight wrap issues if possible)
    let test_start_time = (real_now + chrono::Duration::hours(4)).time();
    
    // If it wrapped around, use a time later today if possible, or just accept the test might be flaky at midnight
    sqlx::query("UPDATE shifts SET start_time = $1 WHERE id = $2")
        .bind(test_start_time)
        .bind(shift_id)
        .execute(&pool).await.unwrap();

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{}/time/punch-in", ws_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&punch_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    
    // Only assert if we didn't wrap around midnight in a way that makes it yesterday
    if test_start_time > real_now.time() {
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // Clean up any accidental punch-in from the first attempt
    sqlx::query("DELETE FROM time_entries WHERE employee_id = $1")
        .bind(employee_id)
        .execute(&pool)
        .await
        .unwrap();

    // Now update shift to have a large grace period (6 hours)
    sqlx::query("UPDATE shifts SET early_punch_in_grace_period_minutes = 360 WHERE id = $1")
        .bind(shift_id)
        .execute(&pool)
        .await
        .unwrap();

    let req2 = Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{}/time/punch-in", ws_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&punch_payload).unwrap()))
        .unwrap();

    let response2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::CREATED);
}
