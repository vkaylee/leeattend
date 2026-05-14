//! OT Workflow integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::OtStatus;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::NaiveDate;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_ot_claim_approval_workflow() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await; // Employee is also user for self-claim

    // 1. Create a daily summary (simulated) — use string for OtStatus since it's a Diesel-only enum
    let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
    sqlx::query("INSERT INTO attendance_daily_summaries (id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status, created_at, updated_at) VALUES ($1, $2, $3, $4, 500, 60, $5::ot_status_type, NOW(), NOW())")
        .bind(uuid::Uuid::new_v4())
        .bind(ws_id)
        .bind(emp_id)
        .bind(date)
        .bind("pending_employee") // OtStatus::PendingEmployee as snake_case string
        .execute(&factory.pool).await.unwrap();

    // 2. Employee Claims OT
    let claim_req = serde_json::json!({
        "date": date.to_string(),
        "claimed_minutes": 60,
        "reason": "Project deadline"
    });

    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{ws_id}/attendance/claim-ot/{emp_id}"
                ))
                .header("Content-Type", "application/json")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(claim_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    if res.status() != StatusCode::OK {
        let status = res.status();
        let body_bytes = axum::body::to_bytes(res.into_body(), 1024).await.unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        panic!("Claim OT failed with status {}. Body: {}", status, body_str);
    }

    // Verify status via raw query (returns JSON-like scalar)
    let ot_status_str: String = sqlx::query_scalar(
        "SELECT ot_status::text FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
    )
    .bind(emp_id)
    .bind(date)
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    let claimed_minutes: i32 = sqlx::query_scalar(
        "SELECT claimed_ot_minutes FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
    )
    .bind(emp_id)
    .bind(date)
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    let expected_status = serde_json::to_value(OtStatus::PendingManager)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(ot_status_str, expected_status);
    assert_eq!(claimed_minutes, 60);
}
