use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::sync::Arc;

use crate::{
    errors::AppError, handlers::common::MemberContext,
    repositories::salary_base_repository::SalaryBaseRepository, AppState,
};

#[derive(Serialize)]
pub struct SalaryBasesResponse {
    pub groups: Vec<crate::models::SalaryBaseGroup>,
    pub members: Vec<crate::models::SalaryBaseGroupMember>,
}

pub async fn get_salary_bases(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, AppError> {
    let groups = SalaryBaseRepository::get_groups(state.infra.db.api(), ctx.workspace_id).await?;
    let members =
        SalaryBaseRepository::get_group_members(state.infra.db.api(), ctx.workspace_id).await?;

    Ok(Json(SalaryBasesResponse { groups, members }))
}

#[derive(Deserialize)]
pub struct AssignMemberRequest {
    pub component_definition_id: Uuid,
}

pub async fn assign_component_to_base(
    State(state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path((_workspace_id, base_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<AssignMemberRequest>,
) -> Result<impl IntoResponse, AppError> {
    SalaryBaseRepository::assign_component_to_group(
        state.infra.db.api(),
        payload.component_definition_id,
        base_id,
    )
    .await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn unassign_component_from_base(
    State(state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path((_workspace_id, base_id, component_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    SalaryBaseRepository::unassign_component_from_group(
        state.infra.db.api(),
        component_id,
        base_id,
    )
    .await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
