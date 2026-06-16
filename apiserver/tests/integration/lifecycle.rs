use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn make_req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if let Some(b) = body {
        builder
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    }
}

#[tokio::test]
async fn test_lifecycle_checklists_e2e() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create an employee
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Create an employer record because employments need an employer
    let employer_id = Uuid::new_v4();
    crate::test_execute!(
        &_test_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Test Employer', 'MST-1', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    // Create primary employment
    let create_emp_payload = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": null,
        "employment_type": "primary",
        "job_title": "Software Engineer"
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(create_emp_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_emp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let employment_id = created_emp["id"].as_str().unwrap().to_string();

    // 1. Create a template
    let template_payload = json!({
        "name": "Standard Onboarding",
        "description": "Onboarding for normal employees",
        "checklist_type": "onboarding",
        "is_default": true
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/lifecycle-templates"),
            ws_id,
            user_id,
            Some(template_payload),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let template: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let template_id = template["id"].as_str().unwrap().to_string();

    // 2. Add an item to the template
    let item_payload = json!({
        "title": "Setup Laptop",
        "description": "Configure dev machine",
        "category": "it_setup",
        "assignee_type": "employee",
        "due_days_offset": 3,
        "is_required": true,
        "display_order": 1
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/lifecycle-templates/{template_id}/items"),
            ws_id,
            user_id,
            Some(item_payload),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    // 3. Instantiate checklist for the employment profile
    let instantiate_payload = json!({
        "template_id": template_id
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employments/{employment_id}/lifecycle"),
            ws_id,
            user_id,
            Some(instantiate_payload),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let checklist: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let checklist_id = checklist["id"].as_str().unwrap().to_string();
    let item_id = checklist["items"][0]["id"].as_str().unwrap().to_string();

    // 4. Update the checklist item status & notes
    let update_item_payload = json!({
        "status": "completed",
        "notes": "Finished setting up the MacBook Pro"
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/lifecycle/{checklist_id}/items/{item_id}"),
            ws_id,
            user_id,
            Some(update_item_payload),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated_checklist: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(updated_checklist["progress_percent"].as_u64().unwrap(), 100);
    assert_eq!(
        updated_checklist["items"][0]["status"].as_str().unwrap(),
        "completed"
    );
    assert_eq!(
        updated_checklist["items"][0]["notes"].as_str().unwrap(),
        "Finished setting up the MacBook Pro"
    );

    // 5. Get pending tasks
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/lifecycle/my-tasks"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_lifecycle_pending_tasks() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create an employee with a custom name and associate it with the user ID
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .with_full_name("John Doe Checklist")
        .create()
        .await;

    // Create an employer record because employments need an employer
    let employer_id = Uuid::new_v4();
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Test Employer', 'MST-1', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    // Create primary employment
    let create_emp_payload = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": null,
        "employment_type": "primary",
        "job_title": "Software Engineer"
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(create_emp_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_emp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let employment_id = created_emp["id"].as_str().unwrap().to_string();

    // 1. Create a template
    let template_payload = json!({
        "name": "Onboarding Temp",
        "description": "Onboarding for John Doe",
        "checklist_type": "onboarding",
        "is_default": true
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/lifecycle-templates"),
            ws_id,
            user_id,
            Some(template_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let template: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let template_id = template["id"].as_str().unwrap().to_string();

    // 2. Add first item to the template (assignee_type: employee)
    let item_payload_1 = json!({
        "title": "Welcome Email",
        "description": "Send welcome email",
        "category": "it_setup",
        "assignee_type": "employee",
        "due_days_offset": 1,
        "is_required": true,
        "display_order": 1
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/lifecycle-templates/{template_id}/items"),
            ws_id,
            user_id,
            Some(item_payload_1),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // Add second item to the template (assignee_type: specific_user)
    let item_payload_2 = json!({
        "title": "Prepare Workspace",
        "description": "Get desk and equipment ready",
        "category": "it_setup",
        "assignee_type": "specific_user",
        "assignee_user_id": user_id.to_string(),
        "due_days_offset": 2,
        "is_required": true,
        "display_order": 2
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/lifecycle-templates/{template_id}/items"),
            ws_id,
            user_id,
            Some(item_payload_2),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 3. Instantiate checklist
    let instantiate_payload = json!({
        "template_id": template_id
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employments/{employment_id}/lifecycle"),
            ws_id,
            user_id,
            Some(instantiate_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 4. Get pending tasks
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/lifecycle/my-tasks"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let tasks: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let tasks_arr = tasks.as_array().unwrap();

    assert_eq!(tasks_arr.len(), 2);
    assert_eq!(tasks_arr[0]["title"].as_str().unwrap(), "Welcome Email");
    assert_eq!(
        tasks_arr[0]["employee_name"].as_str().unwrap(),
        "John Doe Checklist"
    );
    assert_eq!(tasks_arr[1]["title"].as_str().unwrap(), "Prepare Workspace");
    assert_eq!(
        tasks_arr[1]["employee_name"].as_str().unwrap(),
        "John Doe Checklist"
    );
}
