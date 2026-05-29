use axum::http::StatusCode;
use uuid::Uuid;

use crate::common::app::spawn_app_with_member;

#[tokio::test]
async fn test_create_and_get_export_job() {
    let (address, ws_id, owner_id, _member_id, factory, _db) = spawn_app_with_member().await;
    let token = factory.auth().create_token(owner_id);

    let client = reqwest::Client::new();

    // Create an employee so the report has active employees
    let _emp_id = factory
        .employee()
        .with_user(owner_id)
        .with_workspace(ws_id)
        .create()
        .await;

    // 1. Create job
    let now = chrono::Utc::now();
    let start_date = now.format("%Y-%m-01").to_string();
    let end_date = now.format("%Y-%m-28").to_string(); // Safe end date

    let body = serde_json::json!({
        "report_type": "summary",
        "start_date": start_date,
        "end_date": end_date,
        "format": "xlsx"
    });

    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{ws_id}/reports/export"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .unwrap();

    let status = res.status();
    if status != StatusCode::ACCEPTED {
        let body_text = res.text().await.unwrap();
        panic!("Expected 202 ACCEPTED, got {status}: {body_text}");
    }
    assert_eq!(status, StatusCode::ACCEPTED);

    let res_body: serde_json::Value = res.json().await.unwrap();
    let job_id = res_body["job_id"].as_str().unwrap();

    // 2. Poll status
    let res = client
        .get(&format!(
            "{address}/api/v1/workspaces/{ws_id}/reports/jobs/{job_id}"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let status_body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(status_body["status"].as_str().unwrap(), "pending");

    // 3. Download report (Should be 202 because pending)
    let res = client
        .get(&format!(
            "{address}/api/v1/workspaces/{ws_id}/reports/jobs/{job_id}/download"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::ACCEPTED);

    // 4. Cancel job
    let res = client
        .delete(&format!(
            "{address}/api/v1/workspaces/{ws_id}/reports/jobs/{job_id}"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_create_export_job_invalid_dates() {
    let (address, ws_id, owner_id, _member_id, factory, _db) = spawn_app_with_member().await;
    let token = factory.auth().create_token(owner_id);

    let client = reqwest::Client::new();

    // End date before start date
    let now = chrono::Utc::now();
    let start_date = now.format("%Y-%m-28").to_string();
    let end_date = now.format("%Y-%m-01").to_string();

    let body = serde_json::json!({
        "report_type": "summary",
        "start_date": start_date,
        "end_date": end_date,
        "format": "xlsx"
    });

    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{ws_id}/reports/export"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
