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

    let emp_employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // 1. Create a daily summary (simulated) — use string for OtStatus since it's a Diesel-only enum
    let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO attendance_daily_summaries (id, workspace_id, employment_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status, created_at, updated_at) VALUES ($1, $2, $3, $4, 500, 60, $5::ot_status_type, NOW(), NOW())",
        diesel::sql_types::Uuid, uuid::Uuid::new_v4(),
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, emp_employment_id,
        diesel::sql_types::Date, date,
        diesel::sql_types::Text, "pending_employee"
    );

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
                    "/api/v1/workspaces/{ws_id}/attendance/claim-ot/{emp_id}"
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
    let ot_status_str: String = crate::test_query_scalar_string!(
        &factory.diesel_pools.api,
        "SELECT ot_status::text AS val FROM attendance_daily_summaries WHERE employment_id = $1 AND date = $2",
        diesel::sql_types::Uuid, emp_employment_id,
        diesel::sql_types::Date, date
    );

    let claimed_minutes: i32 = crate::test_query_scalar_i32!(
        &factory.diesel_pools.api,
        "SELECT COALESCE(claimed_ot_minutes, 0) AS count FROM attendance_daily_summaries WHERE employment_id = $1 AND date = $2",
        diesel::sql_types::Uuid, emp_employment_id,
        diesel::sql_types::Date, date
    );

    let expected_status = serde_json::to_value(OtStatus::PendingManager)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(ot_status_str, expected_status);
    assert_eq!(claimed_minutes, 60);
}
