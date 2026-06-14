use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    handlers::common::MemberContext,
    models::signature::{CreateSignatureRequestPayload, SignDocumentPayload, SignatureRequest},
    state::AppState,
};

/// [API] Yêu cầu Ký tài liệu
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/documents/{document_id}/signature-requests",
    tag = "signatures",
    request_body = CreateSignatureRequestPayload,
    responses(
        (status = 200, description = "Signature request created", body = SignatureRequest)
    ),
    security(("bearerAuth" = []))
)]
pub async fn create_signature_request(
    State(state): State<std::sync::Arc<AppState>>,
    ctx: MemberContext,
    Path((_workspace_id, document_id)): Path<(Uuid, Uuid)>,
    Json(mut payload): Json<CreateSignatureRequestPayload>,
) -> AppResult<Json<SignatureRequest>> {
    // Override the document_id from the path parameter
    payload.document_id = document_id;
    let request = state
        .core()
        .signature_service()
        .create_request(&ctx.auth_token, payload)
        .await?;
    Ok(Json(request))
}

/// [ESS] Lấy danh sách yêu cầu Ký đang chờ duyệt của Employee
#[utoipa::path(
    get,
    path = "/v1/ess/workspaces/{workspace_id}/signatures/pending",
    tag = "ess_signatures",
    responses(
        (status = 200, description = "List of pending signature requests", body = Vec<SignatureRequest>)
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_pending_signatures(
    State(state): State<std::sync::Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<SignatureRequest>>> {
    // Assuming auth_token.user_id() matches signer_id,
    // In actual implementation, we might need to resolve user_id -> employee_id
    let signer_id = ctx.user_id; // Get user ID directly from MemberContext
    let requests = state
        .core()
        .signature_service()
        .get_pending_requests_for_signer(&ctx.auth_token, signer_id)
        .await?;
    Ok(Json(requests))
}

/// [ESS] Gửi Chữ ký (Ký nhận)
#[utoipa::path(
    post,
    path = "/v1/ess/workspaces/{workspace_id}/signatures/{request_id}/sign",
    tag = "ess_signatures",
    request_body = SignDocumentPayload,
    responses(
        (status = 200, description = "Signature accepted", body = SignatureRequest)
    ),
    security(("bearerAuth" = []))
)]
pub async fn sign_document(
    State(state): State<std::sync::Arc<AppState>>,
    ctx: MemberContext,
    Path((_workspace_id, request_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<SignDocumentPayload>,
) -> AppResult<Json<SignatureRequest>> {
    let signed_request = state
        .core()
        .signature_service()
        .sign_document(&ctx.auth_token, request_id, payload)
        .await?;
    Ok(Json(signed_request))
}

pub fn router() -> Router<std::sync::Arc<AppState>> {
    Router::new()
        // Admin Document Signatures
        .route(
            "/v1/workspaces/:workspace_id/documents/:document_id/signature-requests",
            post(create_signature_request),
        )
        // ESS Signatures
        .route(
            "/v1/ess/workspaces/:workspace_id/signatures/pending",
            get(get_pending_signatures),
        )
        .route(
            "/v1/ess/workspaces/:workspace_id/signatures/:request_id/sign",
            post(sign_document),
        )
}
