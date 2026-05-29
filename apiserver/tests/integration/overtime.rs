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

    let create_response: serde_json::Value = res.json().await.unwrap();
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
