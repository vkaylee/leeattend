use crate::common::{app::spawn_app_with_factory, factory::TestFactory};
use reqwest::Client;
use reqwest::StatusCode;
use uuid::Uuid;

#[tokio::test]
async fn test_overtime_request_full_flow() {
    let (app_addr, admin_ws_id, admin_user_id, factory, _db) = spawn_app_with_factory().await;

    let manager_user_id = factory.user().create().await;
    let employee_user_id = factory.user().create().await;

    // Assign roles to users
    factory
        .add_member(
            admin_ws_id,
            manager_user_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000005").unwrap(),
        )
        .await;
    factory
        .add_member(
            admin_ws_id,
            employee_user_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let manager_emp_id = factory
        .employee()
        .with_user(manager_user_id)
        .with_workspace(admin_ws_id)
        .create()
        .await;
    let employee_emp_id = factory
        .employee()
        .with_user(employee_user_id)
        .with_workspace(admin_ws_id)
        .create()
        .await;

    let mut manager_client = Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            let token = factory.auth().create_token(manager_user_id);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
            headers
        })
        .build()
        .unwrap();

    let mut admin_client = Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            let token = factory.auth().create_token(admin_user_id);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
            headers
        })
        .build()
        .unwrap();

    // 2. Setup EAV Configs (Quotas)

    // Explicitly grant permissions for the test
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES
        ('a0000000-0000-0000-0000-000000000001', 'approval_workflows:manage'),
        ('a0000000-0000-0000-0000-000000000001', 'approval_workflows:read'),
        ('a0000000-0000-0000-0000-000000000005', 'overtime:request:create'),
        ('a0000000-0000-0000-0000-000000000006', 'overtime:request:create')
        ON CONFLICT DO NOTHING"
    );

    let config_payload = serde_json::json!({
        "scope_type": "workspace",
        "scope_id": null,
        "config_key": "overtime.max_hours_per_month",
        "value": {
            "Decimal": "40.0"
        },
        "effective_from": "2024-01-01",
        "effective_to": null
    });

    let res = admin_client
        .put(&format!(
            "{}/api/v1/workspaces/{}/configs",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&config_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 3. Create Approval Workflow
    let workflow_payload = serde_json::json!({
        "entity_type": "ot_request",
        "name": "Overtime Multi-level Approval",
        "description": "Department Manager -> HR Approval",
        "is_active": true,
        "total_steps": 1,
        "steps": [
            {
                "step_order": 1,
                "step_name": "Manager Review",
                "approver_type": "department_manager",
                "approver_config": {},
                "action_required": "approve",
                "can_skip": false
            }
        ]
    });

    let res = admin_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/approval-workflows",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&workflow_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // Setup behalf assignment: Manager -> Employee
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES
        ('a0000000-0000-0000-0000-000000000005', 'overtime:request:create_on_behalf')
        ON CONFLICT DO NOTHING"
    );

    let behalf_payload = serde_json::json!({
        "assignee_user_id": manager_user_id,
        "target_employee_id": employee_emp_id,
        "target_department_id": null
    });
    let behalf_res = admin_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/behalf-assignments",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&behalf_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(behalf_res.status(), StatusCode::OK);

    // 4. Create OT Request
    let ot_payload = serde_json::json!({
        "department_id": null,
        "planned_date": "2024-05-15",
        "planned_start_time": "18:00:00",
        "planned_end_time": "20:00:00",
        "planned_minutes": 120,
        "reason": "Project deadline",
        "employees": [employee_emp_id]
    });

    let res = manager_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&ot_payload)
        .send()
        .await
        .unwrap();

    let status = res.status();
    let create_response: serde_json::Value = res.json().await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "Failed to create OT request: body={:?}",
        create_response
    );
    let request_id = create_response["id"].as_str().unwrap();

    // 5. Get OT Request
    let res = manager_client
        .get(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/{}",
            app_addr, admin_ws_id, request_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 7. Update Employee Consent (assuming employee logs in and updates it)
    let mut employee_client = Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            let token = factory.auth().create_token(employee_user_id);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
            headers
        })
        .build()
        .unwrap();

    let consent_payload = serde_json::json!({
        "status": "accepted"
    });

    let res = employee_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/{}/consent/{}",
            app_addr, admin_ws_id, request_id, employee_emp_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&consent_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 8. Submit for Approval
    let res = manager_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/{}/submit",
            app_addr, admin_ws_id, request_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 9. List Requests
    let res = manager_client
        .get(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_overtime_request_status_serialization() {
    use apiserver::models::enums::OtRequestStatus;
    use apiserver::models::overtime_request::OvertimeRequest;
    use apiserver::schema::overtime_requests;
    use chrono::{NaiveDate, Utc};
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let (_app_addr, admin_ws_id, admin_user_id, factory, test_db) = spawn_app_with_factory().await;

    let mut tenant_conn = test_db
        .diesel_pools
        .tenants_tx
        .get_connection(&admin_ws_id)
        .await
        .unwrap();

    let statuses = vec![
        OtRequestStatus::Draft,
        OtRequestStatus::PendingApproval,
        OtRequestStatus::Approved,
        OtRequestStatus::PartiallyAccepted,
        OtRequestStatus::Active,
        OtRequestStatus::Completed,
        OtRequestStatus::Cancelled,
        OtRequestStatus::Rejected,
    ];

    for status in statuses {
        let req_id = Uuid::new_v4();
        let overtime_req = OvertimeRequest::new(
            req_id,
            admin_ws_id,
            admin_user_id,
            None,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            None,
            None,
            120,
            format!("Test status {:?}", status),
            status.clone(),
            None,
            0,
            None,
            None,
            None,
            Utc::now(),
            Utc::now(),
            None,
        );

        // Insert
        let inserted: OvertimeRequest = diesel::insert_into(overtime_requests::table)
            .values(&overtime_req)
            .get_result(&mut *tenant_conn)
            .await
            .unwrap();

        assert_eq!(inserted.status(), status);

        // Query back
        let queried: OvertimeRequest = overtime_requests::table
            .filter(overtime_requests::id.eq(req_id))
            .first(&mut *tenant_conn)
            .await
            .unwrap();

        assert_eq!(queried.status(), status);
    }
}

#[tokio::test]
async fn test_overtime_request_behalf_assignments() {
    let (app_addr, admin_ws_id, admin_user_id, factory, _db) = spawn_app_with_factory().await;

    let team_lead_user_id = factory.user().create().await;
    let emp1_user_id = factory.user().create().await;
    let emp2_user_id = factory.user().create().await;

    // Assign roles to users
    // Team Lead role ID is a0000000-0000-0000-0000-000000000005
    factory
        .add_member(
            admin_ws_id,
            team_lead_user_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000005").unwrap(),
        )
        .await;
    factory
        .add_member(
            admin_ws_id,
            emp1_user_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;
    factory
        .add_member(
            admin_ws_id,
            emp2_user_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let team_lead_emp_id = factory
        .employee()
        .with_user(team_lead_user_id)
        .with_workspace(admin_ws_id)
        .create()
        .await;
    let emp1_emp_id = factory
        .employee()
        .with_user(emp1_user_id)
        .with_workspace(admin_ws_id)
        .create()
        .await;
    let emp2_emp_id = factory
        .employee()
        .with_user(emp2_user_id)
        .with_workspace(admin_ws_id)
        .create()
        .await;

    let admin_client = Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            let token = factory.auth().create_token(admin_user_id);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
            headers
        })
        .build()
        .unwrap();

    let team_lead_client = Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            let token = factory.auth().create_token(team_lead_user_id);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
            headers
        })
        .build()
        .unwrap();

    // 1. Initially, Team Lead should not be able to create a Pre-OT request for emp1
    let ot_payload_emp1 = serde_json::json!({
        "department_id": null,
        "planned_date": "2024-05-15",
        "planned_start_time": "18:00:00",
        "planned_end_time": "20:00:00",
        "planned_minutes": 120,
        "reason": "Project deadline",
        "employees": [emp1_emp_id]
    });

    let res = team_lead_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&ot_payload_emp1)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 2. Admin creates behalf assignment: Team Lead -> emp1 with matching employer_id
    let emp1_employer_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT employer_id AS id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp1_emp_id
    );

    let behalf_payload = serde_json::json!({
        "assignee_user_id": team_lead_user_id,
        "target_employee_id": emp1_emp_id,
        "target_department_id": null,
        "employer_id": emp1_employer_id
    });

    let res = admin_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/behalf-assignments",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&behalf_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let assignment: serde_json::Value = res.json().await.unwrap();
    let mut assignment_id = assignment["id"].as_str().unwrap().to_string();

    // 3. Team Lead lists behalf assignments
    let res = admin_client
        .get(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/behalf-assignments",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let paginated: serde_json::Value = res.json().await.unwrap();
    let list = paginated["data"].as_array().unwrap();
    assert!(list
        .iter()
        .any(|item| item["id"].as_str().unwrap() == assignment_id));

    // 4. Team Lead can now create Pre-OT request for emp1
    let res = team_lead_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&ot_payload_emp1)
        .send()
        .await
        .unwrap();

    let status = res.status();
    let body = res.text().await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "Failed to create behalf request: body={}",
        body
    );

    // 5. Team Lead still cannot create Pre-OT request for emp2
    let ot_payload_emp2 = serde_json::json!({
        "department_id": null,
        "planned_date": "2024-05-15",
        "planned_start_time": "18:00:00",
        "planned_end_time": "20:00:00",
        "planned_minutes": 120,
        "reason": "Project deadline",
        "employees": [emp2_emp_id]
    });

    let res = team_lead_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&ot_payload_emp2)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // Delete the first valid assignment temporarily
    let res = admin_client
        .delete(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/behalf-assignments/{}",
            app_addr, admin_ws_id, assignment_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 5b. Create a behalf assignment with a different/wrong employer_id
    let wrong_employer_id = Uuid::new_v4();
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Wrong Test Employer', 'TAX-WRONG', 'Wrong Address', NOW(), NOW())",
        diesel::sql_types::Uuid, wrong_employer_id,
        diesel::sql_types::Uuid, admin_ws_id
    );

    let wrong_behalf_payload = serde_json::json!({
        "assignee_user_id": team_lead_user_id,
        "target_employee_id": emp1_emp_id,
        "target_department_id": null,
        "employer_id": wrong_employer_id
    });

    let res = admin_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/behalf-assignments",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&wrong_behalf_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let wrong_assignment: serde_json::Value = res.json().await.unwrap();
    let wrong_assignment_id = wrong_assignment["id"].as_str().unwrap();

    // Team Lead tries to create a Pre-OT request for emp1, but it should fail (StatusCode::FORBIDDEN)
    // because the matching assignment (by target_employee_id) has a wrong employer_id constraint!
    let res = team_lead_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&ot_payload_emp1)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // Clean up wrong assignment
    let res = admin_client
        .delete(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/behalf-assignments/{}",
            app_addr, admin_ws_id, wrong_assignment_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Restore the valid assignment for subsequent steps
    let res = admin_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/behalf-assignments",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&behalf_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let assignment: serde_json::Value = res.json().await.unwrap();
    assignment_id = assignment["id"].as_str().unwrap().to_string();

    // 6. Team Lead cannot create Pre-OT request for both emp1 and emp2 (all-or-nothing validation)
    let ot_payload_both = serde_json::json!({
        "department_id": null,
        "planned_date": "2024-05-15",
        "planned_start_time": "18:00:00",
        "planned_end_time": "20:00:00",
        "planned_minutes": 120,
        "reason": "Project deadline",
        "employees": [emp1_emp_id, emp2_emp_id]
    });

    let res = team_lead_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&ot_payload_both)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 7. Admin deletes behalf assignment
    let res = admin_client
        .delete(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests/behalf-assignments/{}",
            app_addr, admin_ws_id, assignment_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 8. Team Lead can no longer create Pre-OT request for emp1
    let res = team_lead_client
        .post(&format!(
            "{}/api/v1/workspaces/{}/overtime/requests",
            app_addr, admin_ws_id
        ))
        .header("X-Workspace-Id", admin_ws_id.to_string())
        .json(&ot_payload_emp1)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
