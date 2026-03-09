//! Payroll Settings API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<String>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    builder
        .body(match body {
            Some(b) => axum::body::Body::from(b),
            None => axum::body::Body::empty(),
        })
        .unwrap()
}

async fn body_json(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_payroll_settings_update_and_history() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Seed initial payroll settings
    sqlx::query(
        r#"INSERT INTO payroll_settings 
           (workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, effective_date)
           VALUES ($1, 'fixed', 26, 6, 8, '2026-01-01')
           ON CONFLICT (workspace_id, effective_date) DO NOTHING"#
    )
    .bind(ws_id)
    .execute(&test_db.pool())
    .await
    .unwrap();

    // 1. Get payroll settings
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/payroll-settings"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let settings = body_json(resp).await;
    assert_eq!(settings["std_working_day_mode"].as_str().unwrap(), "fixed");

    // 2. Update payroll settings with partial fields
    let update_body = serde_json::json!({
        "hours_per_day": 7.5,
        "max_ot_hours_per_day": 4,
        "effective_date": "2026-03-01"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/workspaces/{ws_id}/payroll-settings"),
            ws_id,
            admin_id,
            Some(update_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let updated = body_json(resp).await;
    assert_eq!(updated["effective_date"].as_str().unwrap(), "2026-03-01");

    // 3. Get settings history
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/payroll-settings/history"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let history = body_json(resp).await;
    assert!(history.as_array().unwrap().len() >= 2);
}
