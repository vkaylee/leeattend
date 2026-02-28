use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::shift_swap::{
    ApproveShiftSwapRequest, CreateShiftSwapRequest, ShiftSwapRequestWithNames, ShiftSwapStatus,
};
use crate::services::shift_swap_service::ShiftSwapService;
use crate::AppState;
use crate::UserId;

// ---------------------------------------------------------------------------
// POST /workspaces/:ws_id/shift-swaps
// ---------------------------------------------------------------------------

pub async fn create_shift_swap(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<CreateShiftSwapRequest>,
) -> AppResult<(StatusCode, Json<crate::models::shift_swap::ShiftSwapRequest>)> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

    let swap = ShiftSwapService::create_swap(&state.db, ctx.workspace_id, user_id, req)
        .await
        .map_err(|s| AppError::from((s, "Failed to create shift swap".to_string())))?;

    Ok((StatusCode::CREATED, Json(swap)))
}

// ---------------------------------------------------------------------------
// GET /workspaces/:ws_id/shift-swaps?status=pending_admin
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ShiftSwapQuery {
    pub status: Option<ShiftSwapStatus>,
}

pub async fn list_shift_swaps(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<ShiftSwapQuery>,
) -> AppResult<Json<Vec<ShiftSwapRequestWithNames>>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

    let swaps =
        ShiftSwapService::get_workspace_swaps(&state.db, ctx.workspace_id, query.status)
            .await
            .map_err(|s| {
                AppError::from((s, "Failed to list shift swaps".to_string()))
            })?;

    Ok(Json(swaps))
}

// ---------------------------------------------------------------------------
// PUT /workspaces/:ws_id/shift-swaps/:id/approve
// ---------------------------------------------------------------------------

pub async fn approve_shift_swap(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveShiftSwapRequest>,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

    ShiftSwapService::approve_swap(&state.db, ctx.workspace_id, user_id, id, req.status)
        .await
        .map_err(|s| AppError::from((s, "Failed to approve shift swap".to_string())))?;

    Ok(StatusCode::OK)
}
