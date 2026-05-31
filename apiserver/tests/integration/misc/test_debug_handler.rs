use crate::common::app::spawn_app_with_factory;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_debug_users_endpoint() {
    let (address, _workspace_id, _user_id, _factory, _test_db) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/v1/debug/users", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    let body = response.text().await.unwrap();
    if status != StatusCode::OK {
        panic!(
            "test_debug_users_endpoint failed: status={}, body={}",
            status, body
        );
    }
}

#[tokio::test]
async fn test_clear_debug_db_endpoint() {
    let (address, workspace_id, _user_id, _factory, _test_db) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // Test with invalid group
    let response = client
        .post(format!(
            "{}/api/v1/debug/workspaces/{}/clear",
            &address, workspace_id
        ))
        .json(&json!({ "group": "invalid_group_name" }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test with "seed" group
    let response = client
        .post(format!(
            "{}/api/v1/debug/workspaces/{}/clear",
            &address, workspace_id
        ))
        .json(&json!({ "group": "seed" }))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    let body = response.text().await.unwrap();
    if status != StatusCode::OK {
        panic!(
            "test_clear_debug_db_endpoint (seed) failed: status={}, body={}",
            status, body
        );
    }

    // Test with "attendance" group
    let response = client
        .post(format!(
            "{}/api/v1/debug/workspaces/{}/clear",
            &address, workspace_id
        ))
        .json(&json!({ "group": "attendance" }))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    let body = response.text().await.unwrap();
    if status != StatusCode::OK {
        panic!(
            "test_clear_debug_db_endpoint (attendance) failed: status={}, body={}",
            status, body
        );
    }

    // Test with "payroll" group
    let response = client
        .post(format!(
            "{}/api/v1/debug/workspaces/{}/clear",
            &address, workspace_id
        ))
        .json(&json!({ "group": "payroll" }))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    let body = response.text().await.unwrap();
    if status != StatusCode::OK {
        panic!(
            "test_clear_debug_db_endpoint (payroll) failed: status={}, body={}",
            status, body
        );
    }

    // Test with "rosters" group
    let response = client
        .post(format!(
            "{}/api/v1/debug/workspaces/{}/clear",
            &address, workspace_id
        ))
        .json(&json!({ "group": "rosters" }))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    let body = response.text().await.unwrap();
    if status != StatusCode::OK {
        panic!(
            "test_clear_debug_db_endpoint (rosters) failed: status={}, body={}",
            status, body
        );
    }

    // Test with "employees" group
    let response = client
        .post(format!(
            "{}/api/v1/debug/workspaces/{}/clear",
            &address, workspace_id
        ))
        .json(&json!({ "group": "employees" }))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    let body = response.text().await.unwrap();
    if status != StatusCode::OK {
        panic!(
            "test_clear_debug_db_endpoint (employees) failed: status={}, body={}",
            status, body
        );
    }

    // Test with "full_reset" group
    let response = client
        .post(format!(
            "{}/api/v1/debug/workspaces/{}/clear",
            &address, workspace_id
        ))
        .json(&json!({ "group": "full_reset" }))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    let body = response.text().await.unwrap();
    if status != StatusCode::OK {
        panic!(
            "test_clear_debug_db_endpoint (full_reset) failed: status={}, body={}",
            status, body
        );
    }
}
