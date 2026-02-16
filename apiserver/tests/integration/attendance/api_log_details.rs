//! Integration test to verify that attendance logs include employee details (name and ID number)

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use apiserver::models::{TimeEntry, RawAttendanceLog, LogSource};
use apiserver::repositories::{time_entry_repository, raw_attendance_log_repository};
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use chrono::Utc;
use uuid::Uuid;
use serde_json::Value;

#[tokio::test]
async fn test_attendance_logs_include_employee_details() -> Result<(), Box<dyn std::error::Error>> {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    
    let emp_name = "TEST USER LOG DETAILS";
    let emp_id_num = "EMP-DETAILS-001";
    let emp_id = factory.employee()
        .with_workspace(ws_id)
        .with_full_name(emp_name)
        .with_employee_id_number(emp_id_num)
        .create().await;

    // 1. Create a TimeEntry
    let entry = TimeEntry {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        workspace_id: ws_id,
        shift_id: None,
        punch_in_time: Utc::now(),
        punch_out_time: Some(Utc::now()),
        duration_minutes: Some(10),
        is_break_overtime: false,
        entry_type: "work".to_string(),
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    time_entry_repository::create_time_entry(&pool, &entry).await?;

    // 2. Create a Device and a RawAttendanceLog
    let device_id = Uuid::new_v4();
    let device_name = "FRONT ENTRANCE DEVICE";
    sqlx::query("INSERT INTO devices (id, name, agent_id, ip, workspace_id) VALUES ($1, $2, 'test-agent', '127.0.0.1', $3)")
        .bind(device_id)
        .bind(device_name)
        .bind(ws_id)
        .execute(&pool)
        .await?;

    let raw_log = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        punch_time: Utc::now(),
        source: LogSource::Device,
        device_id: Some(device_id.to_string()),
        raw_data: None,
        punch: 0,
        status: 0,
        created_at: Utc::now(),
    };
    raw_attendance_log_repository::create_raw_log(&pool, &raw_log).await?;

    // 3. Request /attendance/logs and verify fields
    let response = app.clone().oneshot(
        axum::http::Request::builder()
            .method("GET")
            .uri(format!("/api/workspaces/{ws_id}/attendance/logs"))
            .header("X-User-Id", user_id.to_string())
            .body(axum::body::Body::empty())?,
    ).await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 1024*1024).await?;
    let body: Value = serde_json::from_slice(&body_bytes)?;
    let data = body["data"].as_array().ok_or("Data should be an array")?;
    assert!(!data.is_empty());
    
    let item = &data[0];
    assert_eq!(item["employee_full_name"], emp_name);
    assert_eq!(item["employee_id_number"], emp_id_num);

    // 4. Request /attendance/raw-logs and verify fields
    let response = app.clone().oneshot(
        axum::http::Request::builder()
            .method("GET")
            .uri(format!("/api/workspaces/{ws_id}/attendance/raw-logs"))
            .header("X-User-Id", user_id.to_string())
            .body(axum::body::Body::empty())?,
    ).await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 1024*1024).await?;
    let body: Value = serde_json::from_slice(&body_bytes)?;
    let data = body["data"].as_array().ok_or("Data should be an array")?;
    assert!(!data.is_empty());
    
    let item = &data[0];
    assert_eq!(item["employee_full_name"], emp_name);
    assert_eq!(item["employee_id_number"], emp_id_num);
    assert_eq!(item["device_name"], device_name);
    Ok(())
}
