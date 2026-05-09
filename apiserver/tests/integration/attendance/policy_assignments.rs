use axum::http::StatusCode;
use chrono::Utc;

use crate::common::app::spawn_app_with_member;

#[tokio::test]
async fn test_upsert_and_list_policy_assignments() {
    let (address, ws_id, owner_id, _member_id, factory, _db) = spawn_app_with_member().await;
    let token = factory.auth().create_token(owner_id);

    let target_emp_id = factory
        .employee()
        .with_user(owner_id)
        .with_workspace(ws_id)
        .create()
        .await;

    let client = reqwest::Client::new();

    // 1. Create an assignment
    let req_body = serde_json::json!({
        "target_type": "USER",
        "target_id": target_emp_id,
        "work_unit_minutes": 60,
        "rounding_policy": "NEAREST_HALF_HOUR",
        "early_arrival_strategy": "block",
        "valid_from": Utc::now().to_rfc3339()
    });

    let res = client
        .post(&format!(
            "{address}/api/workspaces/{ws_id}/attendance/policies/assignments"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .json(&req_body)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    let assignment_id = body["id"].as_str().unwrap().to_string();

    // 2. List assignments
    let res = client
        .get(&format!(
            "{address}/api/workspaces/{ws_id}/attendance/policies/assignments"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let list: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0]["target_id"].as_str().unwrap(),
        target_emp_id.to_string()
    );

    // 3. Upsert again to update
    let req_body2 = serde_json::json!({
        "target_type": "USER",
        "target_id": target_emp_id,
        "work_unit_minutes": 30,
        "rounding_policy": "NO_ROUNDING",
        "early_arrival_strategy": "allow",
        "valid_from": Utc::now().to_rfc3339()
    });

    let res = client
        .post(&format!(
            "{address}/api/workspaces/{ws_id}/attendance/policies/assignments"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .json(&req_body2)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 4. Delete assignment
    let res = client
        .delete(&format!(
            "{address}/api/workspaces/{ws_id}/attendance/policies/assignments/{assignment_id}"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify it's deleted (listed should be empty)
    let res = client
        .get(&format!(
            "{address}/api/workspaces/{ws_id}/attendance/policies/assignments"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    let list: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(list.len(), 0);
}
