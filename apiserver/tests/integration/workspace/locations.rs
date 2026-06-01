#![allow(warnings)]
#![allow(clippy::all)]
use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::work_location::{CreateWorkLocationRequest, UpdateWorkLocationRequest};
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_work_locations_crud() {
    let (address, workspace_id, user_id, factory, _db) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Fetch all locations
    let res = client
        .get(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::OK);
    let locations: Vec<Value> = res.json().await.unwrap();
    assert!(!locations.is_empty());

    // We should have at least the Headquarters (created on workspace init)
    let hq = locations
        .iter()
        .find(|l| l["is_headquarters"].as_bool() == Some(true));
    assert!(hq.is_some(), "Should have a headquarters location");
    let hq_id = hq.unwrap()["id"].as_str().unwrap().to_string();

    // 2. Create a new Location
    let create_payload = CreateWorkLocationRequest {
        name: "Test Location 1".to_string(),
        address: Some("123 Test St".to_string()),
        timezone: Some("Asia/Ho_Chi_Minh".to_string()),
        is_headquarters: Some(false),
        status: Some("active".to_string()),
    };

    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&create_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::CREATED);
    let location_1: Value = res.json().await.unwrap();
    let location_1_id = location_1["id"].as_str().unwrap().to_string();
    assert_eq!(location_1["name"], "Test Location 1");
    assert_eq!(location_1["timezone"], "Asia/Ho_Chi_Minh");

    // 3. Try to delete HQ (should fail)
    let res = client
        .delete(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations/{hq_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to delete HQ");

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 4. Update the new Location
    let update_payload = UpdateWorkLocationRequest {
        name: Some("Updated Location 1".to_string()),
        address: None,
        timezone: None,
        is_headquarters: None,
        status: None,
    };

    let res = client
        .put(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations/{location_1_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::OK);
    let updated_location: Value = res.json().await.unwrap();
    assert_eq!(updated_location["name"], "Updated Location 1");

    // 4.5 Get location details by ID
    let res = client
        .get(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations/{location_1_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::OK);
    let fetched_loc: Value = res.json().await.unwrap();
    assert_eq!(fetched_loc["name"], "Updated Location 1");

    // 4.6 Get a non-existent location (should fail with 404)
    let fake_id = Uuid::new_v4();
    let res = client
        .get(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations/{fake_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // 5. Delete the Location
    let res = client
        .delete(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations/{location_1_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_work_locations_edge_cases() {
    let (address, workspace_id, user_id, factory, _db) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Create a location with None timezone and None status to cover fallback closures
    let create_payload = serde_json::json!({
        "name": "Fallback Location",
        "address": "456 Fallback St",
        "timezone": null,
        "is_headquarters": false,
        "status": null
    });

    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&create_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::CREATED);
    let location: Value = res.json().await.unwrap();
    let location_id = location["id"].as_str().unwrap().to_string();
    assert_eq!(location["timezone"], "UTC"); // fallback default
    assert_eq!(location["status"], "active"); // fallback default

    // 2. Update with ALL fields populated to cover all branches in update method
    let update_payload = serde_json::json!({
        "name": "Fully Updated Location",
        "address": "789 Updated St",
        "timezone": "Asia/Ho_Chi_Minh",
        "is_headquarters": false,
        "status": "active"
    });

    let res = client
        .put(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations/{location_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::OK);

    // 3. Create a location with a name that is too long (300 chars) to trigger DB error on insert (covers insert map_err)
    let long_name = "a".repeat(300);
    let bad_create_payload = serde_json::json!({
        "name": long_name,
        "address": null,
        "timezone": null,
        "is_headquarters": false,
        "status": null
    });

    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&bad_create_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // 4. Update with a name that is too long to trigger DB error on update (covers update map_err)
    let bad_update_payload = serde_json::json!({
        "name": "a".repeat(300),
        "address": null,
        "timezone": null,
        "is_headquarters": null,
        "status": null
    });

    let res = client
        .put(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations/{location_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&bad_update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // 5. Assign an employee to the location, then try to delete it (should fail validation because employee is assigned)
    let emp_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // Manually assign employee to the custom location
    crate::test_execute!(
        &_db.diesel_pools.api,
        "UPDATE employees SET primary_location_id = $1 WHERE id = $2",
        diesel::sql_types::Uuid,
        Uuid::parse_str(&location_id).unwrap(),
        diesel::sql_types::Uuid,
        emp_id
    );

    let res = client
        .delete(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/locations/{location_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
