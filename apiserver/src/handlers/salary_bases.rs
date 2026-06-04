use crate::models::SalaryBaseGroup;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::sync::Arc;

use crate::{errors::AppError, handlers::common::MemberContext, AppState};

#[derive(Serialize, utoipa::ToSchema)]
pub struct SalaryBasesResponse {
    pub groups: Vec<crate::models::SalaryBaseGroup>,
    pub members: Vec<crate::models::SalaryBaseGroupMember>,
}

/// GET /workspaces/{ws_id}/salary-bases
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/salary-bases",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Salary bases response", body = SalaryBasesResponse)
    )
)]
pub async fn get_salary_bases(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, AppError> {
    let groups = state
        .payroll()
        .salary_base_repo()
        .get_groups(ctx.workspace_id)
        .await?;
    let members = state
        .payroll()
        .salary_base_repo()
        .get_group_members(ctx.workspace_id)
        .await?;

    Ok(Json(SalaryBasesResponse { groups, members }))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateBaseRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

/// POST /workspaces/{ws_id}/salary-bases
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/salary-bases",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateBaseRequest,
    responses(
        (status = 201, description = "Created salary base group", body = SalaryBaseGroup)
    )
)]
pub async fn create_salary_base(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<CreateBaseRequest>,
) -> Result<impl IntoResponse, AppError> {
    let group = state
        .payroll()
        .salary_base_repo()
        .create_group(
            ctx.workspace_id,
            &payload.name,
            &payload.slug,
            payload.description,
        )
        .await?;

    Ok((axum::http::StatusCode::CREATED, Json(group)))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AssignMemberRequest {
    pub component_definition_id: Uuid,
}

/// POST /workspaces/{ws_id}/salary-bases/{base_id}/members
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/salary-bases/{base_id}/members",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("base_id" = Uuid, Path, description = "Salary Base Group ID")
    ),
    request_body = AssignMemberRequest,
    responses(
        (status = 200, description = "Assigned component to salary base group")
    )
)]
pub async fn assign_component_to_base(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
    Json(payload): Json<AssignMemberRequest>,
) -> Result<impl IntoResponse, AppError> {
    let (_workspace_id, base_id) = params;
    state
        .payroll()
        .salary_base_repo()
        .assign_component_to_group(ctx.workspace_id, payload.component_definition_id, base_id)
        .await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// DELETE /workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id}
#[utoipa::path(
    delete,
    path = "/workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("base_id" = Uuid, Path, description = "Salary Base Group ID"),
        ("component_id" = Uuid, Path, description = "Component Definition ID")
    ),
    responses(
        (status = 200, description = "Unassigned component from salary base group")
    )
)]
pub async fn unassign_component_from_base(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let (_workspace_id, base_id, component_id) = params;
    state
        .payroll()
        .salary_base_repo()
        .unassign_component_from_group(ctx.workspace_id, component_id, base_id)
        .await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
