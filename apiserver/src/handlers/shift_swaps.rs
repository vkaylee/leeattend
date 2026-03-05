use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
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
    ctx.require_permission(Permission::ShiftSwapsManage)?;

    let swap = ShiftSwapService::create_swap(state.db.api(), ctx.workspace_id, user_id, req)
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
    ctx.require_permission(Permission::ShiftSwapsManage)?;

    let swaps =
        ShiftSwapService::get_workspace_swaps(state.db.api(), ctx.workspace_id, query.status)
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
    ctx.require_permission(Permission::ShiftSwapsApprove)?;

    ShiftSwapService::approve_swap(state.db.api(), ctx.workspace_id, user_id, id, req.status)
        .await
        .map_err(|s| AppError::from((s, "Failed to approve shift swap".to_string())))?;

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// GET /workspaces/:ws_id/shift-swaps/stats
// ---------------------------------------------------------------------------

pub async fn get_shift_swap_stats(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<crate::repositories::shift_swap_repository::SwapStats>> {
    ctx.require_permission(Permission::ShiftSwapsManage)?;

    let stats =
        crate::repositories::shift_swap_repository::get_workspace_swap_stats(
            state.db.api(),
            ctx.workspace_id,
        )
        .await
        .map_err(|e| {
            AppError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get swap stats: {e}"),
            ))
        })?;

    Ok(Json(stats))
}
