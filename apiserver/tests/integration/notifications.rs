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
async fn test_notifications_crud() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let user_id = factory
        .user()
        .with_email("notif@example.com")
        .create()
        .await;

    // Insert 2 notifications directly into DB
    let notif1_id = Uuid::new_v4();
    let notif2_id = Uuid::new_v4();

    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO notifications (id, user_id, type, title, content, is_read) VALUES ($1, $2, 'info', 'Test 1', 'Content 1', false)",
        diesel::sql_types::Uuid, notif1_id,
        diesel::sql_types::Uuid, user_id
    );

    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO notifications (id, user_id, type, title, content, is_read) VALUES ($1, $2, 'alert', 'Test 2', 'Content 2', false)",
        diesel::sql_types::Uuid, notif2_id,
        diesel::sql_types::Uuid, user_id
    );

    // 1. Get Unread Count
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/v1/notifications/unread-count",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let count_body = body_json(resp).await;
    assert_eq!(count_body["count"].as_i64().unwrap(), 2);

    // 2. Get Notifications List
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/v1/notifications?limit=10",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list_body = body_json(resp).await;
    let items = list_body["notifications"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(list_body["unread_count"].as_i64().unwrap(), 2);

    // 3. Mark one as read
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/notifications/{}/read", notif1_id),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Check Unread Count again
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/v1/notifications/unread-count",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    let count_body = body_json(resp).await;
    assert_eq!(count_body["count"].as_i64().unwrap(), 1);

    // 5. Mark all as read
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/notifications/read-all",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 6. Check Unread Count again
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/v1/notifications/unread-count",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    let count_body = body_json(resp).await;
    assert_eq!(count_body["count"].as_i64().unwrap(), 0);
}
