use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::sync::Arc;

use crate::{errors::AppError, handlers::common::MemberContext, AppState};

#[derive(Serialize)]
pub struct SalaryBasesResponse {
    pub groups: Vec<crate::models::SalaryBaseGroup>,
    pub members: Vec<crate::models::SalaryBaseGroupMember>,
}

pub async fn get_salary_bases(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, AppError> {
    let groups = state
        .payroll
        .salary_base_repo
        .get_groups(ctx.workspace_id)
        .await?;
    let members = state
        .payroll
        .salary_base_repo
        .get_group_members(ctx.workspace_id)
        .await?;

    Ok(Json(SalaryBasesResponse { groups, members }))
}

#[derive(Deserialize)]
pub struct CreateBaseRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

pub async fn create_salary_base(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<CreateBaseRequest>,
) -> Result<impl IntoResponse, AppError> {
    let group = state
        .payroll
        .salary_base_repo
        .create_group(
            ctx.workspace_id,
            &payload.name,
            &payload.slug,
            payload.description,
        )
        .await?;

    Ok((axum::http::StatusCode::CREATED, Json(group)))
}

#[derive(Deserialize)]
pub struct AssignMemberRequest {
    pub component_definition_id: Uuid,
}

pub async fn assign_component_to_base(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_workspace_id, base_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<AssignMemberRequest>,
) -> Result<impl IntoResponse, AppError> {
    state
        .payroll
        .salary_base_repo
        .assign_component_to_group(ctx.workspace_id, payload.component_definition_id, base_id)
        .await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn unassign_component_from_base(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_workspace_id, base_id, component_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    state
        .payroll
        .salary_base_repo
        .unassign_component_from_group(ctx.workspace_id, component_id, base_id)
        .await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
