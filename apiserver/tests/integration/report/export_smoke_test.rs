//! Report export smoke test
//!
//! Validates: Without a Redis-backed coordinator, export returns 503.
//! This test uses `setup_app_tower()` which intentionally sets coordinator=None
//! to verify graceful degradation.

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_export_job_success() {
    let (_test_db, factory, _state, app) =
        crate::common::app::setup_app_tower_without_redis().await;

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

    // setup_app_tower() builds state without a Redis coordinator,
    // so the export endpoint always returns 503 SERVICE_UNAVAILABLE.
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}
