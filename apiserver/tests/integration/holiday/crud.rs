//! Holiday CRUD integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::Holiday;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Helper: build a JSON request with workspace auth headers
fn json_request(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if let Some(body) = body {
        builder
            .body(axum::body::Body::from(
                serde_json::to_string(&body).unwrap(),
            ))
            .unwrap()
    } else {
        builder.body(axum::body::Body::empty()).unwrap()
    }
}

/// Helper: read response body as JSON
async fn read_body<T: serde::de::DeserializeOwned>(
    response: axum::http::Response<axum::body::Body>,
) -> T {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_holiday_crud_lifecycle_smoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 1. List holidays — should be empty
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/holidays"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let holidays: Vec<Holiday> = read_body(response).await;
    assert!(holidays.is_empty());

    // 2. Create a holiday
    let create_body = serde_json::json!({
        "name": "Tết Dương Lịch",
        "date": "2026-01-01",
        "holiday_type": "national",
        "is_paid": true,
        "description": "New Year's Day"
    });

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/holidays"),
            ws_id,
            user_id,
            Some(create_body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let holiday: Holiday = read_body(response).await;
    assert_eq!(holiday.name(), "Tết Dương Lịch");
    assert_eq!(holiday.holiday_type(), "national");
    assert!(holiday.is_paid());
    assert_eq!(holiday.date().to_string(), "2026-01-01");

    let holiday_id = holiday.id();

    // 3. Get holiday by ID
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/holidays/{holiday_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let fetched: Holiday = read_body(response).await;
    assert_eq!(fetched.id(), holiday_id);

    // 4. Update holiday
    let update_body = serde_json::json!({
        "name": "New Year 2026",
        "is_paid": false
    });

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/v1/workspaces/{ws_id}/holidays/{holiday_id}"),
            ws_id,
            user_id,
            Some(update_body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated: Holiday = read_body(response).await;
    assert_eq!(updated.name(), "New Year 2026");
    assert!(!updated.is_paid());

    // 5. List holidays — should have 1
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/holidays"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let holidays: Vec<Holiday> = read_body(response).await;
    assert_eq!(holidays.len(), 1);

    // 6. Delete holiday
    let response = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/holidays/{holiday_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 7. List holidays — should be empty again
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/holidays"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let holidays: Vec<Holiday> = read_body(response).await;
    assert!(holidays.is_empty());
}

#[tokio::test]
async fn test_holiday_duplicate_date_returns_conflict() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let create_body = serde_json::json!({
        "name": "Quốc Khánh",
        "date": "2026-09-02",
        "holiday_type": "national"
    });

    // First create — should succeed
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/holidays"),
            ws_id,
            user_id,
            Some(create_body.clone()),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Second create — same date — should return 409 Conflict
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/holidays"),
            ws_id,
            user_id,
            Some(create_body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_holiday_year_filter() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create holidays in 2026 and 2027
    for (name, date) in [
        ("Holiday A", "2026-05-01"),
        ("Holiday B", "2026-09-02"),
        ("Holiday C", "2027-01-01"),
    ] {
        let body = serde_json::json!({ "name": name, "date": date });
        let response = app
            .clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/v1/workspaces/{ws_id}/holidays"),
                ws_id,
                user_id,
                Some(body),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Filter by 2026 — should return 2
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/holidays?year=2026"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let holidays: Vec<Holiday> = read_body(response).await;
    assert_eq!(holidays.len(), 2);

    // Filter by 2027 — should return 1
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/holidays?year=2027"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let holidays: Vec<Holiday> = read_body(response).await;
    assert_eq!(holidays.len(), 1);

    // No filter — should return all 3
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/holidays"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let holidays: Vec<Holiday> = read_body(response).await;
    assert_eq!(holidays.len(), 3);
}

#[tokio::test]
async fn test_holiday_not_found() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let fake_id = Uuid::new_v4();

    // GET non-existent
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/holidays/{fake_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // DELETE non-existent
    let response = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/holidays/{fake_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_holiday_patch_update() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create a holiday first
    let create_body = serde_json::json!({
        "name": "Giỗ Tổ Hùng Vương",
        "date": "2026-04-12",
        "holiday_type": "national",
        "is_paid": true,
        "description": "Hung Kings' Temple Festival"
    });

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/holidays"),
            ws_id,
            user_id,
            Some(create_body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let holiday: Holiday = read_body(response).await;
    let holiday_id = holiday.id();

    // Update via PATCH (the method used by the frontend)
    let patch_body = serde_json::json!({
        "name": "Giỗ Tổ Hùng Vương 2026",
        "is_paid": false
    });

    let response = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/holidays/{holiday_id}"),
            ws_id,
            user_id,
            Some(patch_body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated: Holiday = read_body(response).await;
    assert_eq!(updated.name(), "Giỗ Tổ Hùng Vương 2026");
    assert!(!updated.is_paid());
    // Unchanged fields should be preserved
    assert_eq!(updated.holiday_type(), "national");
    assert_eq!(updated.date().to_string(), "2026-04-12");
}

#[tokio::test]
async fn test_holiday_with_ot_multiplier() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let body = serde_json::json!({
        "name": "Quốc tế Lao động",
        "date": "2026-05-01",
        "holiday_type": "national",
        "is_paid": true,
        "ot_multiplier": 4.0,
        "description": "International Workers' Day"
    });

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/holidays"),
            ws_id,
            user_id,
            Some(body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let holiday: Holiday = read_body(response).await;
    assert_eq!(
        holiday.ot_multiplier(),
        Some(rust_decimal::Decimal::new(40, 1)) // 4.0
    );
    assert_eq!(holiday.description().unwrap(), "International Workers' Day");
}
