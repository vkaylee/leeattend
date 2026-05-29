//! RBAC integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_rbac_teamlead_scope_and_admin_bypass() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // 1. Setup global workspace and owner
    let owner_user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(owner_user_id).await;

    // 2. Create Departments A and B
    let dept_a = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Dept A")
        .create()
        .await;
    let dept_b = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Dept B")
        .create()
        .await;

    // 3. Create TeamLead User and Employee in Dept A
    let tl_user_id = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            tl_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000005").unwrap(),
        )
        .await;
    let emp_tl = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(tl_user_id)
        .create()
        .await;

    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let dept_repo = apiserver::repositories::DieselDepartmentRepository::new(diesel_pools.clone());
    use apiserver::repositories::department_repository::DepartmentRepository;
    dept_repo
        .transfer_employee(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_tl,
            dept_a,
            chrono::Utc::now().date_naive(),
            None,
        )
        .await
        .unwrap();

    // 4. Create target Employee in Dept B
    let emp_target_b = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Target Employee")
        .create()
        .await;
    dept_repo
        .transfer_employee(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_target_b,
            dept_b,
            chrono::Utc::now().date_naive(),
            None,
        )
        .await
        .unwrap();

    // 5. Try to update Employee B using TeamLead (Should fail -> 403 Forbidden)
    let update_req = serde_json::json!({
        "full_name": "Hacked Name"
    });

    let response_tl = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/{emp_target_b}"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", tl_user_id.to_string())
                .body(axum::body::Body::from(update_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response_tl.status(),
        StatusCode::FORBIDDEN,
        "TeamLead A should not update Employee B"
    );

    // 6. Create Admin User (global scope)
    let admin_user_id = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            admin_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000002").unwrap(),
        )
        .await;

    // 7. Try to update Employee B using Admin (Should succeed -> 200 OK)
    // First we grant Admin HrManager powers to update employees (Wait, looking at role_permissions, Admin inherits HrManager which has EmployeesUpdate)
    let response_admin = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/{emp_target_b}"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_user_id.to_string())
                .body(axum::body::Body::from(update_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response_admin.status(),
        StatusCode::OK,
        "Admin should update any employee"
    );
}

#[tokio::test]
async fn test_rbac_audit_log_on_permissions_grant_revoke() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // 1. Setup
    let admin_user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(admin_user_id).await; // Owner has members:roles:manage

    let target_user_id = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            target_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    // 2. Grant temporary permission
    let future_date = chrono::Utc::now() + chrono::Duration::days(1);
    let req_body_grant = serde_json::json!({
        "permission": "shifts:create",
        "expires_at": future_date
    });

    let response_grant = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/members/{target_user_id}/permissions"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_user_id.to_string())
                .body(axum::body::Body::from(req_body_grant.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response_grant.status(),
        StatusCode::CREATED,
        "Grant permission should succeed"
    );

    // Parse response to find perm_id
    let body = axum::body::to_bytes(response_grant.into_body(), usize::MAX)
        .await
        .unwrap();
    let perm_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let perm_id = perm_json["id"].as_str().unwrap();

    // 3. Verify audit log for grant
    let (audit_logs, _) = state
        .core
        .audit_repo
        .find_by_workspace(ws_id, 1, 0, None, None, None)
        .await
        .unwrap();
    let audit_log = audit_logs.into_iter().next().unwrap();

    assert_eq!(audit_log.action, "assign_permission");
    assert!(audit_log.details.is_some());

    // 4. Revoke the permission
    let response_revoke = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/members/{target_user_id}/permissions/{perm_id}"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response_revoke.status(),
        StatusCode::NO_CONTENT,
        "Revoke permission should succeed"
    );

    // 5. Verify audit log for revoke
    let (audit_logs_after, _) = state
        .core
        .audit_repo
        .find_by_workspace(ws_id, 1, 0, None, None, None)
        .await
        .unwrap();
    let audit_log_after = audit_logs_after.into_iter().next().unwrap();

    assert_eq!(audit_log_after.action, "revoke_permission");
}
