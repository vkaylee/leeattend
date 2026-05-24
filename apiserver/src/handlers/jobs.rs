#![allow(unused_variables)]
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::handlers::common::MemberContext;
use crate::queue::JobQueue;
use crate::state::AppState;

pub async fn get_generic_job_status(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, job_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let redis = state
        .infra
        .redis
        .as_ref()
        .ok_or_else(|| AppError::Internal("Redis is required for job tracking".to_string()))?;

    let queue = JobQueue::new(redis.clone());

    let meta = queue
        .get_job_metadata(job_id)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {e}")))?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    // Security check
    if meta.workspace_id != ctx.workspace_id {
        return Err(AppError::Forbidden(
            "Cannot access job from another workspace".to_string(),
        ));
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
