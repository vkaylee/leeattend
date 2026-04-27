//! Attendance Paper Forms (Paper OT) API integration tests

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
async fn test_enter_paper_ot_employee_not_found() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let fake_emp_id = Uuid::new_v4();
    let body = serde_json::json!({
        "date": "2026-01-15",
        "claimed_minutes": 60,
        "reason": "Weekend work"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/attendance/enter-paper-ot/{fake_emp_id}"),
            ws_id,
            admin_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_enter_paper_ot_invalid_minutes() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let body = serde_json::json!({
        "date": "2026-01-15",
        "claimed_minutes": -10,
        "reason": "Invalid test"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/attendance/enter-paper-ot/{emp_id}"),
            ws_id,
            admin_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    // Should return validation error
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn test_enter_paper_ot_zero_minutes() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let body = serde_json::json!({
        "date": "2026-01-15",
        "claimed_minutes": 0,
        "reason": "Zero test"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/attendance/enter-paper-ot/{emp_id}"),
            ws_id,
            admin_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn test_enter_paper_ot_rbac_member_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let member_id = factory.user().with_email("member@example.com").create().await;
    factory.add_member(ws_id, member_id, uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap()).await;

    let body = serde_json::json!({
        "date": "2026-01-15",
        "claimed_minutes": 60,
        "reason": "Forbidden test"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/attendance/enter-paper-ot/{emp_id}"),
            ws_id,
            member_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_bulk_upload_paper_ot_no_file() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Send empty multipart (no file)
    let boundary = "----WebKitFormBoundary";
    let body = format!("--{boundary}--\r\n");

    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(&format!("/api/workspaces/{ws_id}/attendance/bulk-upload-paper-ot"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    // Should fail because no file was uploaded
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn test_bulk_upload_paper_ot_with_csv() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create employee for the CSV
    let _emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("EMP001")
        .create()
        .await;

    // Build multipart body with CSV content
    let boundary = "----WebKitFormBoundary123";
    let csv_content = "employee_id,date,minutes,reason\nEMP001,2026-01-15,60,Weekend work\nINVALID_EMP,2026-01-15,30,Test";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"ot.csv\"\r\nContent-Type: text/csv\r\n\r\n{csv_content}\r\n--{boundary}--\r\n"
    );

    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(&format!("/api/workspaces/{ws_id}/attendance/bulk-upload-paper-ot"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let result = body_json(resp).await;
    assert_eq!(result["total_rows"].as_i64().unwrap(), 2);
    // INVALID_EMP should fail, so failed_count >= 1
    assert!(result["failed_count"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_bulk_upload_paper_ot_rbac_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory.user().with_email("member@example.com").create().await;
    factory.add_member(ws_id, member_id, uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap()).await;

    let boundary = "----WebKitFormBoundary";
    let body = format!("--{boundary}--\r\n");

    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(&format!("/api/workspaces/{ws_id}/attendance/bulk-upload-paper-ot"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", member_id.to_string())
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
