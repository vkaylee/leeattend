use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::AppState;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct DistributeDocumentRequest {
    pub document_id: Uuid,
    pub target_employee_ids: Vec<Uuid>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AcknowledgementDto {
    pub id: Uuid,
    pub document_id: Uuid,
    pub employee_id: Uuid,
    pub status: String,
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/documents/distributions",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = DistributeDocumentRequest,
    responses(
        (status = 200, description = "Document distributed successfully")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn distribute_document_handler(
    State(_state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(payload): Json<DistributeDocumentRequest>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(
        serde_json::json!({ "status": "success", "count": payload.target_employee_ids.len() }),
    ))
}

#[utoipa::path(
    get,
    path = "/v1/ess/documents/pending",
    responses(
        (status = 200, description = "Pending documents for the logged in user", body = Vec<AcknowledgementDto>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_pending_documents_handler(
    State(_state): State<Arc<AppState>>,
    _ctx: MemberContext,
) -> AppResult<Json<Vec<AcknowledgementDto>>> {
    // ESS route: use ctx.user_id / employee_id to find pending ones
    Ok(Json(vec![]))
}

#[utoipa::path(
    post,
    path = "/v1/ess/documents/{id}/acknowledge",
    params(
        ("id" = Uuid, Path, description = "Acknowledgement ID")
    ),
    responses(
        (status = 200, description = "Document acknowledged")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn acknowledge_document_handler(
    State(_state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path(_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "status": "acknowledged" })))
}
