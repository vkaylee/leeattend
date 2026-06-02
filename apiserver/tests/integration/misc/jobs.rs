//! Integration tests for background jobs tracking.
//!
//! Validates that:
//! - GET /`workspaces/:ws_id/jobs/:job_id/status` returns 200 with valid job metadata
//! - Returns 404 for non-existent job
//! - Returns 403 when trying to access a job from a different workspace

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::{
    queue::JobQueue,
    queue::{JobMetadata, JobStatus},
    AppState,
};

use axum::http::StatusCode;
use chrono::Utc;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn body_json(response: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn make_req(uri: &str, ws_id: Uuid, user_id: Uuid) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method("GET")
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap()
}

#[tokio::test]
async fn test_get_job_status_success() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let job_id = Uuid::new_v4();

    // Insert mock job metadata into Redis
    let redis = state.infra.redis.clone().unwrap();
    let queue = JobQueue::new(redis);

    let meta = JobMetadata {
        job_id,
        workspace_id: ws_id,
        status: JobStatus::Processing,
        total_batches: 10,
        completed_batches: 5,
        failed_batches: 1,
        created_at: Utc::now(),
        error_message: None,
        completed_at: None,
        requester_id: user_id,
        report_type: None,
        date_range: None,
        task_type: None,
        payload: None,
    };

    queue.update_job_metadata(&meta).await.unwrap();

    let res = app
        .clone()
        .oneshot(make_req(
            &format!("/api/v1/workspaces/{ws_id}/jobs/{job_id}"),
            ws_id,
            user_id,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = body_json(res).await;
    assert_eq!(body["job_id"], job_id.to_string());
    assert_eq!(body["status"], "processing");
    assert_eq!(body["progress"]["total_batches"], 10);
    assert_eq!(body["progress"]["completed_batches"], 5);
    assert_eq!(body["progress"]["failed_batches"], 1);
    assert_eq!(body["progress"]["percentage"].as_f64().unwrap(), 50.0);
}

#[tokio::test]
async fn test_get_job_status_not_found() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let job_id = Uuid::new_v4();

    let res = app
        .clone()
        .oneshot(make_req(
            &format!("/api/v1/workspaces/{ws_id}/jobs/{job_id}"),
            ws_id,
            user_id,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_job_status_forbidden_cross_workspace() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_1 = factory.workspace().create_with_owner(user_id).await;
    let ws_2 = factory.workspace().create_with_owner(user_id).await;
    let job_id = Uuid::new_v4();

    // Create job belonging to ws_1
    let redis = state.infra.redis.clone().unwrap();
    let queue = JobQueue::new(redis);

    let meta = JobMetadata {
        job_id,
        workspace_id: ws_1,
        status: JobStatus::Pending,
        total_batches: 0,
        completed_batches: 0,
        failed_batches: 0,
        created_at: Utc::now(),
        error_message: None,
        completed_at: None,
        requester_id: user_id,
        report_type: None,
        date_range: None,
        task_type: None,
        payload: None,
    };

    queue.update_job_metadata(&meta).await.unwrap();

    // Try to access it from ws_2
    let res = app
        .clone()
        .oneshot(make_req(
            &format!("/api/v1/workspaces/{ws_2}/jobs/{job_id}"),
            ws_2,
            user_id,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
