//! Integration test to verify that attendance logs include employee details (name and ID number)

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{LogSource, NewTimeEntryParams, RawAttendanceLog, TimeEntry};
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_attendance_logs_include_employee_details() -> Result<(), Box<dyn std::error::Error>> {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp_name = "TEST USER LOG DETAILS";
    let emp_id_num = "EMP-DETAILS-001";
    let normalized_emp_id_num = apiserver::utils::normalize_employee_id(emp_id_num);
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name(emp_name)
        .with_employee_id_number(emp_id_num)
        .create()
        .await;

    let emp_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // 1. Create a TimeEntry
    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: emp_employment_id,
        workspace_id: ws_id,
        shift_id: None,
        segment_id: None,
        punch_in_time: Utc::now(),
        punch_out_time: Some(Utc::now()),
        duration_minutes: Some(10),
        is_break_overtime: false,
        entry_type: apiserver::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let te_repo = apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone());
    let raw_log_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    te_repo
        .create_time_entry(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            &entry,
        )
        .await?;

    // 2. Create a Device and a RawAttendanceLog
    let device_id = Uuid::new_v4();
    let device_name = "FRONT ENTRANCE DEVICE";
    crate::test_execute!(&pool.api,
        "INSERT INTO devices (id, name, agent_id, ip, workspace_id) VALUES ($1, $2, 'test-agent', '127.0.0.1', $3)",
        diesel::sql_types::Uuid, device_id,
        diesel::sql_types::Text, device_name,
        diesel::sql_types::Uuid, ws_id
    );

    let raw_log = RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        emp_employment_id,
        Utc::now(),
        LogSource::Device,
        Some(device_id.to_string()),
        None,
        0,
        0,
        Utc::now(),
    );
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
    raw_log_repo.create_raw_log(ws_id, &raw_log).await?;

    // 3. Request /attendance/logs and verify fields
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/attendance/logs"))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024).await?;
    let body: Value = serde_json::from_slice(&body_bytes)?;
    let data = body["data"].as_array().ok_or("Data should be an array")?;
    assert!(!data.is_empty());

    let item = &data[0];
    assert_eq!(item["employee_full_name"], emp_name);
    assert_eq!(item["employee_id_number"], normalized_emp_id_num);

    // 4. Request /attendance/raw-logs and verify fields
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/attendance/raw-logs"))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024).await?;
    let body: Value = serde_json::from_slice(&body_bytes)?;
    let data = body["data"].as_array().ok_or("Data should be an array")?;
    assert!(!data.is_empty());

    let item = &data[0];
    assert_eq!(item["employee_full_name"], emp_name);
    assert_eq!(item["employee_id_number"], normalized_emp_id_num);
    assert_eq!(item["device_name"], device_name);
    Ok(())
}
