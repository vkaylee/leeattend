use crate::common::app::spawn_app_with_factory;
use apiserver::models::{CreateEmployerRequest, UpdateEmployerRequest};
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_employers_crud() {
    let (address, workspace_id, user_id, factory, _db) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Fetch all employers (should be empty initially)
    let res = client
        .get(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to execute list request");

    assert_eq!(res.status(), StatusCode::OK);
    let employers: Vec<Value> = res.json().await.unwrap();
    assert!(
        employers.is_empty(),
        "Employers list should be empty initially"
    );

    // 2. Create a new Employer
    let create_payload = CreateEmployerRequest {
        name: "Test Employer Corp".to_string(),
        tax_code: Some("MST-999888".to_string()),
        legal_address: Some("123 Corporate Way".to_string()),
    };

    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&create_payload)
        .send()
        .await
        .expect("Failed to create employer");

    assert_eq!(res.status(), StatusCode::CREATED);
    let employer_1: Value = res.json().await.unwrap();
    let employer_1_id = employer_1["id"].as_str().unwrap().to_string();
    assert_eq!(employer_1["name"], "Test Employer Corp");
    assert_eq!(employer_1["tax_code"], "MST-999888");
    assert_eq!(employer_1["legal_address"], "123 Corporate Way");

    // 3. Get employer details by ID
    let res = client
        .get(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers/{employer_1_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to fetch employer");

    assert_eq!(res.status(), StatusCode::OK);
    let fetched: Value = res.json().await.unwrap();
    assert_eq!(fetched["name"], "Test Employer Corp");
    assert_eq!(fetched["tax_code"], "MST-999888");

    // 4. Update the Employer
    let update_payload = UpdateEmployerRequest {
        name: Some("Updated Employer Corp".to_string()),
        tax_code: Some("MST-111222".to_string()),
        legal_address: Some("456 Executive Blvd".to_string()),
    };

    let res = client
        .put(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers/{employer_1_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to update employer");

    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Updated Employer Corp");
    assert_eq!(updated["tax_code"], "MST-111222");
    assert_eq!(updated["legal_address"], "456 Executive Blvd");

    // 5. Try to get a non-existent employer (should be 404)
    let fake_id = Uuid::new_v4();
    let res = client
        .get(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers/{fake_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to fetch fake employer");

    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // 6. Assign an employee to the employer, then try to delete it (should fail validation)
    let emp_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // Manually assign employee to the custom employer
    crate::test_execute!(
        &_db.diesel_pools.api,
        "UPDATE employees SET employer_id = $1 WHERE id = $2",
        diesel::sql_types::Uuid,
        Uuid::parse_str(&employer_1_id).unwrap(),
        diesel::sql_types::Uuid,
        emp_id
    );

    let res = client
        .delete(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers/{employer_1_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to delete employer");

    // Should return 400 Bad Request because of assigned employee constraint
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 7. Unassign the employee, then delete should succeed
    crate::test_execute!(
        &_db.diesel_pools.api,
        "UPDATE employees SET employer_id = NULL WHERE id = $1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let res = client
        .delete(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers/{employer_1_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to delete employer");

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify it is gone
    let res = client
        .get(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to fetch list");

    let list: Vec<Value> = res.json().await.unwrap();
    assert!(list.is_empty(), "Employers list should be empty again");
}
