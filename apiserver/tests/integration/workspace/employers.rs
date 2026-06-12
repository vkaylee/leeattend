use crate::common::app::spawn_app_with_factory;
use apiserver::models::{CreateEmployerRequest, UpdateEmployerRequest};
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_employers_crud() {
    let (address, workspace_id, user_id, factory, _db) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // Clean up default employer seeded by workspace creation
    crate::test_execute!(
        &_db.diesel_pools.api,
        "DELETE FROM employers WHERE workspace_id = $1",
        diesel::sql_types::Uuid,
        workspace_id
    );

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
    let paginated: Value = res.json().await.unwrap();
    let employers = paginated["data"].as_array().unwrap().clone();
    assert!(
        employers.is_empty(),
        "Employers list should be empty initially"
    );

    // 2. Create a new Employer
    let create_payload = CreateEmployerRequest {
        name: "Test Employer Corp".to_string(),
        tax_code: Some("MST-999888".to_string()),
        legal_address: Some("123 Corporate Way".to_string()),
        parent_id: None,
        relationship_type: None,
        business_type: Some("llc".to_string()),
        registration_number: Some("REG999".to_string()),
        phone: Some("+123456789".to_string()),
        email: Some("corp@test.com".to_string()),
        website: Some("https://test.com".to_string()),
        incorporation_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        country_code: Some("USA".to_string()),
        currency_code: Some("USD".to_string()),
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
    assert_eq!(employer_1["business_type"], "llc");
    assert_eq!(employer_1["registration_number"], "REG999");
    assert_eq!(employer_1["phone"], "+123456789");
    assert_eq!(employer_1["email"], "corp@test.com");
    assert_eq!(employer_1["website"], "https://test.com");
    assert_eq!(employer_1["incorporation_date"], "2026-01-01");
    assert_eq!(employer_1["country_code"], "USA");
    assert_eq!(employer_1["currency_code"], "USD");

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
    assert_eq!(fetched["business_type"], "llc");

    // 4. Update the Employer
    let update_payload = UpdateEmployerRequest {
        name: Some("Updated Employer Corp".to_string()),
        tax_code: Some("MST-111222".to_string()),
        legal_address: Some("456 Executive Blvd".to_string()),
        parent_id: None,
        relationship_type: None,
        business_type: Some("corporation".to_string()),
        registration_number: Some("REG888".to_string()),
        phone: Some("+987654321".to_string()),
        email: Some("updated@test.com".to_string()),
        website: Some("https://updated.com".to_string()),
        incorporation_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()),
        country_code: Some("VNM".to_string()),
        currency_code: Some("VND".to_string()),
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
    assert_eq!(updated["business_type"], "corporation");
    assert_eq!(updated["registration_number"], "REG888");
    assert_eq!(updated["phone"], "+987654321");
    assert_eq!(updated["email"], "updated@test.com");
    assert_eq!(updated["website"], "https://updated.com");
    assert_eq!(updated["incorporation_date"], "2026-02-02");
    assert_eq!(updated["country_code"], "VNM");
    assert_eq!(updated["currency_code"], "VND");

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
        "UPDATE employments SET employer_id = $1 WHERE employee_id = $2 AND status = 'active'",
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
        "DELETE FROM employments WHERE employee_id = $1",
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

    let paginated: Value = res.json().await.unwrap();
    let list = paginated["data"].as_array().unwrap().clone();
    assert!(list.is_empty(), "Employers list should be empty again");
}

#[tokio::test]
async fn test_employer_hierarchy() {
    let (address, workspace_id, user_id, _factory, _db) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Create a parent employer
    let parent_payload = CreateEmployerRequest {
        name: "Parent Company".to_string(),
        tax_code: Some("TAX-PARENT".to_string()),
        legal_address: Some("1 Parent Rd".to_string()),
        parent_id: None,
        relationship_type: None,
        business_type: None,
        registration_number: None,
        phone: None,
        email: None,
        website: None,
        incorporation_date: None,
        country_code: None,
        currency_code: None,
    };
    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&parent_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let parent: Value = res.json().await.unwrap();
    let parent_id = parent["id"].as_str().unwrap().to_string();

    // 2. Create a subsidiary employer linked to the parent
    let child_payload = CreateEmployerRequest {
        name: "Subsidiary Company".to_string(),
        tax_code: Some("TAX-CHILD".to_string()),
        legal_address: Some("2 Child Rd".to_string()),
        parent_id: Some(Uuid::parse_str(&parent_id).unwrap()),
        relationship_type: Some("subsidiary".to_string()),
        business_type: None,
        registration_number: None,
        phone: None,
        email: None,
        website: None,
        incorporation_date: None,
        country_code: None,
        currency_code: None,
    };
    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&child_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let child: Value = res.json().await.unwrap();
    let child_id = child["id"].as_str().unwrap().to_string();
    assert_eq!(child["parent_id"], parent_id);
    assert_eq!(child["relationship_type"], "subsidiary");

    // 3. Try to link child to a parent in a different workspace
    let other_workspace_id = Uuid::new_v4();
    let other_parent_id = Uuid::new_v4();
    crate::test_execute!(
        &_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, created_at, updated_at) VALUES ($1, $2, 'Other Parent', 'TAX-OTHER', NOW(), NOW())",
        diesel::sql_types::Uuid,
        other_parent_id,
        diesel::sql_types::Uuid,
        other_workspace_id
    );

    let bad_child_payload = CreateEmployerRequest {
        name: "Bad Subsidiary".to_string(),
        tax_code: Some("TAX-BAD".to_string()),
        legal_address: None,
        parent_id: Some(other_parent_id),
        relationship_type: Some("subsidiary".to_string()),
        business_type: None,
        registration_number: None,
        phone: None,
        email: None,
        website: None,
        incorporation_date: None,
        country_code: None,
        currency_code: None,
    };
    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&bad_child_payload)
        .send()
        .await
        .unwrap();
    // Should fail validation (400 Bad Request)
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 4. Update parent to have child as its parent (circular dependency test)
    let bad_parent_update = UpdateEmployerRequest {
        name: None,
        tax_code: None,
        legal_address: None,
        parent_id: Some(Some(Uuid::parse_str(&child_id).unwrap())),
        relationship_type: Some(Some("subsidiary".to_string())),
        business_type: None,
        registration_number: None,
        phone: None,
        email: None,
        website: None,
        incorporation_date: None,
        country_code: None,
        currency_code: None,
    };
    let res = client
        .put(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers/{parent_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&bad_parent_update)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 5. Update child to have itself as its parent (self-reference test)
    let self_parent_update = UpdateEmployerRequest {
        name: None,
        tax_code: None,
        legal_address: None,
        parent_id: Some(Some(Uuid::parse_str(&child_id).unwrap())),
        relationship_type: None,
        business_type: None,
        registration_number: None,
        phone: None,
        email: None,
        website: None,
        incorporation_date: None,
        country_code: None,
        currency_code: None,
    };
    let res = client
        .put(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers/{child_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&self_parent_update)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 6. Update child to remove parent (set parent_id to null)
    let remove_parent_payload = UpdateEmployerRequest {
        name: None,
        tax_code: None,
        legal_address: None,
        parent_id: Some(None),
        relationship_type: Some(None),
        business_type: None,
        registration_number: None,
        phone: None,
        email: None,
        website: None,
        incorporation_date: None,
        country_code: None,
        currency_code: None,
    };
    let res = client
        .put(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers/{child_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&remove_parent_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated_child: Value = res.json().await.unwrap();
    assert!(updated_child["parent_id"].is_null());
    assert!(updated_child["relationship_type"].is_null());
}

#[tokio::test]
async fn test_employer_invalid_validations() {
    let (address, workspace_id, user_id, _factory, _db) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Invalid Email
    let bad_email_payload = CreateEmployerRequest {
        name: "Bad Email Corp".to_string(),
        tax_code: None,
        legal_address: None,
        parent_id: None,
        relationship_type: None,
        business_type: None,
        registration_number: None,
        phone: None,
        email: Some("invalid-email".to_string()),
        website: None,
        incorporation_date: None,
        country_code: None,
        currency_code: None,
    };
    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&bad_email_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 2. Invalid Country Code (too short)
    let bad_country_payload = CreateEmployerRequest {
        name: "Bad Country Corp".to_string(),
        tax_code: None,
        legal_address: None,
        parent_id: None,
        relationship_type: None,
        business_type: None,
        registration_number: None,
        phone: None,
        email: None,
        website: None,
        incorporation_date: None,
        country_code: Some("US".to_string()),
        currency_code: None,
    };
    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&bad_country_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 3. Invalid Currency Code (too long)
    let bad_currency_payload = CreateEmployerRequest {
        name: "Bad Currency Corp".to_string(),
        tax_code: None,
        legal_address: None,
        parent_id: None,
        relationship_type: None,
        business_type: None,
        registration_number: None,
        phone: None,
        email: None,
        website: None,
        incorporation_date: None,
        country_code: None,
        currency_code: Some("USDD".to_string()),
    };
    let res = client
        .post(&format!(
            "{address}/api/v1/workspaces/{workspace_id}/employers"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&bad_currency_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
