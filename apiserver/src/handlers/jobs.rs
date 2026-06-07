use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use crate::handlers::common::MemberContext;
use crate::state::AppState;

/// GET /workspaces/{ws_id}/jobs/{job_id}
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/jobs/{job_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("job_id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Get job status", body = serde_json::Value)
    )
)]
#[tracing::instrument(skip(state, ctx))]
pub async fn get_generic_job_status(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (_ws_id, job_id) = params;
    let queue = state.infra().job_queue()?;

    let meta = queue
        .get_job_metadata(job_id)
        .await
        .map_err(|e| AppError::BackgroundJobError(format!("Redis error: {e}")))?
        .ok_or_else(|| AppError::not_found(I18nKey::JobNotFound))?;

    // Security check
    if meta.workspace_id != ctx.workspace_id {
        return Err(AppError::forbidden(I18nKey::Forbidden));
    }

    let percentage = if meta.total_batches > 0 {
        (meta.completed_batches as f32 / meta.total_batches as f32) * 100.0
    } else {
        0.0
    };

    Ok(Json(serde_json::json!({
        "job_id": meta.job_id,
        "status": format!("{:?}", meta.status).to_lowercase(),
        "progress": {
            "total_batches": meta.total_batches,
            "completed_batches": meta.completed_batches,
            "failed_batches": meta.failed_batches,
            "percentage": percentage
        },
        "error": meta.error_message,
        "created_at": meta.created_at,
        "completed_at": meta.completed_at
    })))
}
