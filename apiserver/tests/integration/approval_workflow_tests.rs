use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::diesel_pool::TenantAuthToken;
use apiserver::models::{ApprovalAction, ApprovalWorkflow, ApprovalWorkflowStep};
use uuid::Uuid;

#[tokio::test]
async fn test_service_create_and_get_workflow() {
    let (_test_db, factory, state, _app) = crate::common::app::setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = TenantAuthToken::system_override(ws_id);
    let service = state.core().approval_workflow_service();

    // 1. Create Workflow definition via service
    let workflow_id = Uuid::new_v4();
    let workflow = ApprovalWorkflow {
        id: workflow_id,
        workspace_id: ws_id,
        entity_type: "leave_request".to_string(),
        name: "Service Test Workflow".to_string(),
        description: Some("Description".to_string()),
        total_steps: 1,
        is_active: true,
        created_by: Some(user_id),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let step = ApprovalWorkflowStep {
        id: Uuid::new_v4(),
        workflow_id,
        step_order: 1,
        step_name: "Manager step".to_string(),
        approver_type: "specific_user".to_string(),
        approver_config: serde_json::json!({}),
        action_required: "approve".to_string(),
        can_skip: false,
        created_at: chrono::Utc::now(),
    };

    let created = service
        .create_workflow(&auth_token, workflow, vec![step])
        .await
        .unwrap();
    assert_eq!(created.0.id, workflow_id);

    // 2. Retrieve
    let fetched = service
        .get_workflow(&auth_token, workflow_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.0.name, "Service Test Workflow");

    // 3. List
    let (list, total) = service
        .list_workflows(&auth_token, ws_id, Some("leave_request".to_string()), 1, 10)
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(list[0]["id"].as_str().unwrap(), workflow_id.to_string());
}

#[tokio::test]
async fn test_service_submit_and_approve_fallback() {
    let (_test_db, factory, state, _app) = crate::common::app::setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = TenantAuthToken::system_override(ws_id);
    let service = state.core().approval_workflow_service();

    let entity_id = Uuid::new_v4();
    let beneficiary_emp_id = Uuid::new_v4();

    // 1. Submit to workflow (since there is no active workflow, it will fallback to direct manager approval step)
    let instance = service
        .submit_to_workflow(
            &auth_token,
            "dummy_workflow_entity".to_string(),
            entity_id,
            beneficiary_emp_id,
        )
        .await
        .unwrap();
    assert_eq!(instance.status, "pending");
    assert_eq!(instance.current_step_order, 1);

    // 2. Process approval action (as admin user_id)
    let processed = service
        .process_approval_action(
            &auth_token,
            user_id,
            instance.id,
            ApprovalAction::Approve,
            Some("approved by service test".to_string()),
            None,
        )
        .await
        .unwrap();
    assert_eq!(processed.status, "approved");
}

#[tokio::test]
async fn test_service_submit_and_reject_fallback() {
    let (_test_db, factory, state, _app) = crate::common::app::setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = TenantAuthToken::system_override(ws_id);
    let service = state.core().approval_workflow_service();

    let entity_id = Uuid::new_v4();
    let beneficiary_emp_id = Uuid::new_v4();

    // 1. Submit to workflow
    let instance = service
        .submit_to_workflow(
            &auth_token,
            "dummy_workflow_entity".to_string(),
            entity_id,
            beneficiary_emp_id,
        )
        .await
        .unwrap();

    // 2. Process rejection
    let processed = service
        .process_approval_action(
            &auth_token,
            user_id,
            instance.id,
            ApprovalAction::Reject,
            Some("rejected by service test".to_string()),
            None,
        )
        .await
        .unwrap();
    assert_eq!(processed.status, "rejected");
}
