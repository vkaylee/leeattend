//! OT Workflow integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use apiserver::models::{OtStatus, AttendanceDailySummary};
use axum::http::StatusCode;
use chrono::NaiveDate;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_ot_claim_approval_workflow() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).with_user(user_id).create().await; // Employee is also user for self-claim

    // 1. Create a daily summary (simulated)
    let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
    sqlx::query("INSERT INTO attendance_daily_summaries (id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status, created_at, updated_at) VALUES ($1, $2, $3, $4, 500, 60, 'none', NOW(), NOW())")
        .bind(uuid::Uuid::new_v4())
        .bind(ws_id)
        .bind(emp_id)
        .bind(date)
        .execute(&factory.pool).await.unwrap();

    // 2. Employee Claims OT
    let claim_req = serde_json::json!({
        "date": date.to_string(),
        "claimed_minutes": 60,
        "reason": "Project deadline"
    });

    let res = app.clone().oneshot(
        axum::http::Request::builder()
            .method("POST")
            .uri(format!("/api/workspaces/{}/attendance/claim-ot/{}", ws_id, emp_id))
            .header("Content-Type", "application/json")
            .header("X-User-Id", user_id.to_string())
            .body(axum::body::Body::from(claim_req.to_string()))
            .unwrap()
    ).await.unwrap();
    
    assert_eq!(res.status(), StatusCode::OK);
    
    // Verify status
    let summary: AttendanceDailySummary = sqlx::query_as("SELECT * FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2")
        .bind(emp_id).bind(date).fetch_one(&factory.pool).await.unwrap();
    
    assert_eq!(summary.ot_status, OtStatus::PendingManager);
    assert_eq!(summary.claimed_ot_minutes, 60);
}
