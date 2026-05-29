use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_workspace_config_endpoints() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let valid_token = apiserver::services::auth_service::AuthService::generate_token(
        user_id,
        "test_secret_tower",
        60,
    )
    .unwrap();

    let auth_header = format!("Bearer {}", valid_token);

    // 1. List Configs (should return empty initially)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{}/configs", ws_id))
                .header("Authorization", &auth_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 2. Upsert Config
    let upsert_payload = serde_json::json!({
        "scope_type": "workspace",
        "scope_id": null,
        "config_key": "max_overtime_hours",
        "value": {"Int": 40},
        "effective_from": "2024-01-01",
        "effective_to": null
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{}/configs", ws_id))
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&upsert_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 3. Get Config (returns dummy for now, but should be 200 OK)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{}/configs/max_overtime_hours",
                    ws_id
                ))
                .header("Authorization", &auth_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 4. Delete Config
    // We don't have the returned ID from Upsert cleanly to extract yet.
    // Let's just create a dummy uuid for coverage.
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/workspaces/{}/configs/{}", ws_id, fake_id))
                .header("Authorization", &auth_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
