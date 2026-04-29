//! Report export security integration tests
//!
//! Validates: RBAC enforcement, graceful 503 without Redis/coordinator

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_export_job_requires_admin_role() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let member_id = factory.user().with_email("member@test.com").create().await;
    factory
        .add_member(
            ws_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let body = serde_json::json!({
        "report_type": "summary",
        "start_date": "2026-01-01",
        "end_date": "2026-01-31",
        "format": "xlsx"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/reports/export"))
                .header("Content-Type", "application/json")
                .header("X-User-Id", member_id.to_string())
                .header("X-Workspace-Id", ws_id.to_string())
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_export_job_without_redis_returns_503() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let body = serde_json::json!({
        "report_type": "summary",
        "start_date": "2026-01-01",
        "end_date": "2026-01-31",
        "format": "xlsx"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/reports/export"))
                .header("Content-Type", "application/json")
                .header("X-User-Id", admin_id.to_string())
                .header("X-Workspace-Id", ws_id.to_string())
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_get_job_status_without_redis_returns_503() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let fake_job_id = uuid::Uuid::new_v4();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/reports/jobs/{fake_job_id}"
                ))
                .header("X-User-Id", admin_id.to_string())
                .header("X-Workspace-Id", ws_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_cancel_job_without_redis_returns_503() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let fake_job_id = uuid::Uuid::new_v4();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/workspaces/{ws_id}/reports/jobs/{fake_job_id}"
                ))
                .header("X-User-Id", admin_id.to_string())
                .header("X-Workspace-Id", ws_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_download_report_without_redis_returns_503() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let fake_job_id = uuid::Uuid::new_v4();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/reports/jobs/{fake_job_id}/download"
                ))
                .header("X-User-Id", admin_id.to_string())
                .header("X-Workspace-Id", ws_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}
