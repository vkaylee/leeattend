use axum::{
    extract::{Path, State},
    Json,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::services::template_service::TemplateService;
use crate::AppState;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct TemplatePreviewRequest {
    pub template_content: String,
    pub employee_id: Uuid,
    pub manual_variables: Option<HashMap<String, String>>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct TemplatePreviewResponse {
    pub filled_content: String,
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/documents/templates/preview",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = TemplatePreviewRequest,
    responses(
        (status = 200, description = "Template preview generated", body = TemplatePreviewResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn preview_template_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(payload): Json<TemplatePreviewRequest>,
) -> AppResult<Json<TemplatePreviewResponse>> {
    let content = TemplateService::generate_employee_document(
        &ctx.auth_token,
        payload.employee_id,
        state.core().workspace_repo().as_ref(),
        state.employees().repo().as_ref(),
        state.employees().employment_repo().as_ref(),
        state.employees().contract_repo().as_ref(),
        &payload.template_content,
        payload.manual_variables,
    )
    .await?;

    Ok(Json(TemplatePreviewResponse {
        filled_content: content,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/documents/templates/{id}/generate",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Template File ID")
    ),
    request_body = TemplatePreviewRequest,
    responses(
        (status = 200, description = "Document generated from template")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn generate_document_from_template_handler(
    State(_state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path(_params): Path<(Uuid, Uuid)>,
    Json(_payload): Json<TemplatePreviewRequest>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "status": "generated" })))
}
