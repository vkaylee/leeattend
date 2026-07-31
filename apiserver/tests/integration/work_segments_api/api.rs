use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

/// Helper: build an authenticated request for the work-segments API.
fn build_request(
    method: &str,
    uri: &str,
    ws_id: uuid::Uuid,
    user_id: uuid::Uuid,
    body_json: Option<&serde_json::Value>,
) -> Request<body::Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if body_json.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    let body = match body_json {
        Some(json) => body::Body::from(json.to_string()),
        None => body::Body::empty(),
    };

    builder.body(body).unwrap()
}

/// Helper: extract response body as JSON value.
async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = body::to_bytes(response.into_body(), 1_000_000)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// Full CRUD lifecycle: create → get → update → list → bulk → delete → get 404.
#[tokio::test]
async fn test_work_segment_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .create_with_owner(admin_id)
        .await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_id
    );
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    // 1. CREATE
    let create_body = serde_json::json!({
        "employment_id": employment_id,
        "date": "2026-07-31",
        "template_id": shift_id,
        "scheduled_start": "08:00:00",
        "scheduled_end": "17:00:00",
        "source": "manual",
        "segment_type": "work",
        "segment_status": "scheduled",
    });

    let response = app
        .clone()
        .oneshot(build_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/work-segments"),
            ws_id,
            admin_id,
            Some(&create_body),
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "CREATE should return 201"
    );

    let segment = body_json(response).await;
    let segment_id = segment["id"].as_str().unwrap();
    assert_eq!(segment["employment_id"].as_str().unwrap(), employment_id.to_string());
    assert_eq!(segment["date"].as_str().unwrap(), "2026-07-31");
    assert_eq!(segment["source"].as_str().unwrap(), "manual");

    // 2. GET
    let response = app
        .clone()
        .oneshot(build_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/work-segments/{segment_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "GET should return 200");

    let fetched = body_json(response).await;
    assert_eq!(fetched["id"].as_str().unwrap(), segment_id);

    // 3. UPDATE
    let update_body = serde_json::json!({
        "notes": "Updated for testing",
    });
    let response = app
        .clone()
        .oneshot(build_request(
            "PUT",
            &format!("/api/v1/workspaces/{ws_id}/work-segments/{segment_id}"),
            ws_id,
            admin_id,
            Some(&update_body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "UPDATE should return 200");

    let updated = body_json(response).await;
    assert_eq!(updated["notes"].as_str().unwrap(), "Updated for testing");

    // 4. LIST with date filter
    let response = app
        .clone()
        .oneshot(build_request(
            "GET",
            &format!(
                "/api/v1/workspaces/{ws_id}/work-segments?start_date=2026-07-01&end_date=2026-07-31"
            ),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "LIST should return 200");

    let list = body_json(response).await;
    assert!(
        list["total_count"].as_i64().unwrap() >= 1,
        "LIST should return at least 1 segment"
    );

    // 5. DELETE
    let response = app
        .clone()
        .oneshot(build_request(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/work-segments/{segment_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "DELETE should return 204"
    );

    // 6. GET after delete → 404
    let response = app
        .clone()
        .oneshot(build_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/work-segments/{segment_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "GET after DELETE should return 404"
    );
}

/// Bulk create endpoint works and does not get routed to /:segment_id.
#[tokio::test]
async fn test_bulk_create_work_segments() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .create_with_owner(admin_id)
        .await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_id
    );
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    let bulk_body = serde_json::json!({
        "segments": [
            {
                "employment_id": employment_id,
                "date": "2026-08-01",
                "template_id": shift_id,
                "scheduled_start": "08:00:00",
                "scheduled_end": "12:00:00",
                "source": "manual",
                "segment_type": "work",
                "segment_status": "scheduled",
            },
            {
                "employment_id": employment_id,
                "date": "2026-08-01",
                "template_id": shift_id,
                "scheduled_start": "13:00:00",
                "scheduled_end": "17:00:00",
                "source": "manual",
                "segment_type": "work",
                "segment_status": "scheduled",
            }
        ],
        "replace_existing": false,
    });

    let response = app
        .clone()
        .oneshot(build_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/work-segments/bulk"),
            ws_id,
            admin_id,
            Some(&bulk_body),
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "BULK CREATE should return 201"
    );

    let segments = body_json(response).await;
    let arr = segments.as_array().unwrap();
    assert_eq!(arr.len(), 2, "Should return 2 segments");
}

/// Non-existent segment returns 404.
#[tokio::test]
async fn test_get_nonexistent_segment_returns_404() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .create_with_owner(admin_id)
        .await;
    let fake_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(build_request(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/work-segments/{fake_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "GET non-existent should return 404"
    );
}

/// Delete non-existent segment returns 404.
#[tokio::test]
async fn test_delete_nonexistent_segment_returns_404() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .create_with_owner(admin_id)
        .await;
    let fake_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(build_request(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/work-segments/{fake_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "DELETE non-existent should return 404"
    );
}

/// List with employment_ids filter returns only matching segments.
#[tokio::test]
async fn test_list_with_employment_filter() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .create_with_owner(admin_id)
        .await;

    // Create two employees
    let emp1_id = factory.employee().with_workspace(ws_id).create().await;
    let emp2_id = factory.employee().with_workspace(ws_id).create().await;
    let employment1_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp1_id
    );
    let employment2_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp2_id
    );
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    // Create segments for both employees
    for (eid, date) in [(employment1_id, "2026-08-10"), (employment2_id, "2026-08-10")] {
        let body = serde_json::json!({
            "employment_id": eid,
            "date": date,
            "template_id": shift_id,
            "scheduled_start": "08:00:00",
            "scheduled_end": "17:00:00",
            "source": "manual",
            "segment_type": "work",
            "segment_status": "scheduled",
        });
        let response = app
            .clone()
            .oneshot(build_request(
                "POST",
                &format!("/api/v1/workspaces/{ws_id}/work-segments"),
                ws_id,
                admin_id,
                Some(&body),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // List filtered by employment1 only
    let response = app
        .clone()
        .oneshot(build_request(
            "GET",
            &format!(
                "/api/v1/workspaces/{ws_id}/work-segments?employment_ids={employment1_id}"
            ),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let list = body_json(response).await;
    let count = list["total_count"].as_i64().unwrap();
    assert_eq!(count, 1, "Filtered list should return exactly 1 segment");
}
