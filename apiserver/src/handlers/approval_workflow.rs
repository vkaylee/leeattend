use crate::{
    domain::permissions::Permission,
    errors::AppResult,
    handlers::common::MemberContext,
    models::{
        ApprovalActionLog, ApprovalWorkflow, ApprovalWorkflowTask, CreateWorkflowRequest,
        PaginatedResponse,
    },
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedApprovalWorkflows {
    pub data: Vec<ApprovalWorkflow>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ListWorkflowsQuery {
    pub entity_type: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

/// GET /workspaces/{ws_id}/approval-workflows
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/approval-workflows",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ListWorkflowsQuery
    ),
    responses(
        (status = 200, description = "Approval workflows list", body = PaginatedApprovalWorkflows)
    )
)]
pub async fn list_workflows(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<ListWorkflowsQuery>,
) -> AppResult<Json<PaginatedResponse<serde_json::Value>>> {
    ctx.require_permission(Permission::ApprovalWorkflowsRead)?;
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (workflows, total) = state
        .core
        .approval_workflow_service
        .list_workflows(
            &ctx.auth_token,
            ctx.workspace_id,
            query.entity_type,
            page,
            limit,
        )
        .await?;

    Ok(Json(PaginatedResponse {
        data: workflows,
        total: total as usize,
        page,
        limit,
    }))
}

/// POST /workspaces/{ws_id}/approval-workflows
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/approval-workflows",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateWorkflowRequest,
    responses(
        (status = 201, description = "Created approval workflow", body = ApprovalWorkflow)
    )
)]
pub async fn create_workflow(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkflowRequest>,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_permission(Permission::ApprovalWorkflowsManage)?;

    let workflow_id = crate::utils::id::new_id();
    let workflow = ApprovalWorkflow {
        id: workflow_id,
        workspace_id: ctx.workspace_id,
        entity_type: req.entity_type.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        total_steps: req.steps.len() as i32,
        is_active: req.is_active,
        created_by: Some(ctx.user_id),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let steps: Vec<crate::models::approval_workflow::ApprovalWorkflowStep> = req
        .steps
        .into_iter()
        .map(|s| crate::models::approval_workflow::ApprovalWorkflowStep {
            id: crate::utils::id::new_id(),
            workflow_id,
            step_order: s.step_order,
            step_name: s.step_name,
            approver_type: s.approver_type,
            approver_config: s.approver_config,
            action_required: s.action_required,
            can_skip: s.can_skip,
            created_at: chrono::Utc::now(),
        })
        .collect();

    let (created_wf, created_steps) = state
        .core
        .approval_workflow_service
        .create_workflow(&ctx.auth_token, workflow, steps)
        .await?;

    let mut val = serde_json::to_value(&created_wf)
        .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    if let serde_json::Value::Object(ref mut map) = val {
        map.insert(
            "steps".to_string(),
            serde_json::to_value(&created_steps)
                .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?,
        );
    }

    Ok(Json(val))
}

/// GET /workspaces/{ws_id}/approval-workflows/{id}
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/approval-workflows/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Workflow ID")
    ),
    responses(
        (status = 200, description = "Approval workflow details", body = ApprovalWorkflow)
    )
)]
pub async fn get_workflow(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    let (_, id) = params;
    ctx.require_permission(Permission::ApprovalWorkflowsRead)?;

    let (workflow, steps) = state
        .core
        .approval_workflow_service
        .get_workflow(&ctx.auth_token, id)
        .await?
        .ok_or_else(|| crate::errors::AppError::NotFound("Workflow not found".into()))?;

    let mut val = serde_json::to_value(&workflow)
        .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    if let serde_json::Value::Object(ref mut map) = val {
        map.insert(
            "steps".to_string(),
            serde_json::to_value(&steps)
                .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?,
        );
    }

    Ok(Json(val))
}

/// PUT /workspaces/{ws_id}/approval-workflows/{id}
#[utoipa::path(
    put,
    path = "/workspaces/{ws_id}/approval-workflows/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Workflow ID")
    ),
    request_body = CreateWorkflowRequest,
    responses(
        (status = 200, description = "Updated approval workflow", body = ApprovalWorkflow)
    )
)]
pub async fn update_workflow(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkflowRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let (_, id) = params;
    ctx.require_permission(Permission::ApprovalWorkflowsManage)?;

    let workflow = ApprovalWorkflow {
        id,
        workspace_id: ctx.workspace_id,
        entity_type: req.entity_type.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        total_steps: req.steps.len() as i32,
        is_active: req.is_active,
        created_by: Some(ctx.user_id),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let steps: Vec<crate::models::approval_workflow::ApprovalWorkflowStep> = req
        .steps
        .into_iter()
        .map(|s| crate::models::approval_workflow::ApprovalWorkflowStep {
            id: crate::utils::id::new_id(),
            workflow_id: id,
            step_order: s.step_order,
            step_name: s.step_name,
            approver_type: s.approver_type,
            approver_config: s.approver_config,
            action_required: s.action_required,
            can_skip: s.can_skip,
            created_at: chrono::Utc::now(),
        })
        .collect();

    let (updated_wf, created_steps) = state
        .core
        .approval_workflow_service
        .update_workflow_definition(&ctx.auth_token, workflow, steps)
        .await?;

    let mut val = serde_json::to_value(&updated_wf)
        .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    if let serde_json::Value::Object(ref mut map) = val {
        map.insert(
            "steps".to_string(),
            serde_json::to_value(&created_steps)
                .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?,
        );
    }

    Ok(Json(val))
}

/// DELETE /workspaces/{ws_id}/approval-workflows/{id}
#[utoipa::path(
    delete,
    path = "/workspaces/{ws_id}/approval-workflows/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Workflow ID")
    ),
    responses(
        (status = 200, description = "Deleted approval workflow")
    )
)]
pub async fn delete_workflow(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_, id) = params;
    ctx.require_permission(Permission::ApprovalWorkflowsManage)?;

    state
        .core
        .approval_workflow_service
        .delete_workflow(&ctx.auth_token, id)
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ProcessApprovalRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub note: Option<String>,
    pub new_approver_user_id: Option<Uuid>,
}

/// POST /workspaces/{ws_id}/approval-workflows/approve
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/approval-workflows/approve",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = ProcessApprovalRequest,
    responses(
        (status = 200, description = "Processed approval step")
    )
)]
pub async fn process_approval(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<ProcessApprovalRequest>,
) -> AppResult<StatusCode> {
    if req.entity_type == "ot_request" {
        ctx.require_permission(Permission::OvertimeApproveWorkflow)?;
    }

    let instance = state
        .core
        .approval_workflow_service
        .get_workflow_instance_by_entity(&ctx.auth_token, &req.entity_type, req.entity_id)
        .await?
        .ok_or_else(|| {
            crate::errors::AppError::NotFound("Workflow instance not found for this entity".into())
        })?;

    state
        .core
        .approval_workflow_service
        .process_approval_action(
            &ctx.auth_token,
            ctx.user_id,
            instance.id,
            req.action,
            req.note,
            req.new_approver_user_id,
        )
        .await?;

    Ok(StatusCode::OK)
}

/// GET /workspaces/{ws_id}/approval-workflows/history/{type}/{id}
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/approval-workflows/history/{type}/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("type" = String, Path, description = "Entity Type"),
        ("id" = Uuid, Path, description = "Entity ID")
    ),
    responses(
        (status = 200, description = "Approval history action logs list", body = [ApprovalActionLog])
    )
)]
pub async fn get_approval_history(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, String, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<ApprovalActionLog>>> {
    let (_, entity_type, entity_id) = params;
    if entity_type == "ot_request" {
        ctx.require_permission(Permission::OvertimeRequestRead)?;
    }

    let history = state
        .core
        .approval_workflow_service
        .get_action_logs(&ctx.auth_token, entity_id)
        .await?;

    Ok(Json(history))
}

/// GET /workspaces/{ws_id}/approval-workflows/pending-tasks
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/approval-workflows/pending-tasks",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Pending approval tasks for the current user", body = [ApprovalWorkflowTask])
    )
)]
pub async fn get_pending_tasks(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<crate::models::ApprovalWorkflowTask>>> {
    ctx.require_permission(Permission::ApprovalWorkflowsRead)?;
    let tasks = state
        .core
        .approval_workflow_service
        .get_pending_tasks_for_user(&ctx.auth_token, ctx.user_id)
        .await?;

    Ok(Json(tasks))
}
