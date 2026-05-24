#![allow(unused_variables)]
//! Holiday CRUD handlers for workspace holiday calendar management.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{CreateHolidayRequest, Holiday, UpdateHolidayRequest};
use crate::repositories::holiday_repository::HolidayLeaveResult;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListHolidaysQuery {
    pub year: Option<i32>,
}

/// GET /workspaces/:ws_id/holidays
pub async fn list_holidays(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<ListHolidaysQuery>,
) -> AppResult<Json<Vec<Holiday>>> {
    let holidays = state
        .leaves
        .holiday_repo
        .list(ctx.workspace_id, query.year)
        .await?;

    Ok(Json(holidays))
}

/// GET /workspaces/:ws_id/holidays/:id
pub async fn get_holiday(
    State(state): State<Arc<AppState>>,
    Path((_, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Holiday>> {
    let holiday = state
        .leaves
        .holiday_repo
        .get_by_id(ctx.workspace_id, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Holiday not found".to_string()))?;

    Ok(Json(holiday))
}

/// POST /workspaces/:ws_id/holidays
pub async fn create_holiday(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateHolidayRequest>,
) -> AppResult<(axum::http::StatusCode, Json<Holiday>)> {
    ctx.require_permission(Permission::HolidaysManage)?;

    let holiday = state
        .leaves
        .holiday_repo
        .create_holiday(ctx.workspace_id, &req)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                AppError::Conflict(format!("A holiday already exists on {}", req.date))
            } else {
                AppError::Internal(e.to_string())
            }
        })?;

    Ok((axum::http::StatusCode::CREATED, Json(holiday)))
}

/// PUT /workspaces/:ws_id/holidays/:id
pub async fn update_holiday(
    State(state): State<Arc<AppState>>,
    Path((_, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateHolidayRequest>,
) -> AppResult<Json<Holiday>> {
    ctx.require_permission(Permission::HolidaysManage)?;

    let holiday = state
        .leaves
        .holiday_repo
        .update_holiday(ctx.workspace_id, id, &req)
        .await?
        .ok_or_else(|| AppError::NotFound("Holiday not found".to_string()))?;

    Ok(Json(holiday))
}

/// DELETE /workspaces/:ws_id/holidays/:id
pub async fn delete_holiday(
    State(state): State<Arc<AppState>>,
    Path((_, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<axum::http::StatusCode> {
    ctx.require_permission(Permission::HolidaysManage)?;

    let deleted = state
        .leaves
        .holiday_repo
        .delete(ctx.workspace_id, id)
        .await?;

    if deleted {
        Ok(axum::http::StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Holiday not found".to_string()))
    }
}

/// POST /workspaces/:ws_id/holidays/:id/process-leave
/// Triggers automatic leave deduction for employees on an unpaid holiday.
pub async fn process_holiday_leave(
    State(state): State<Arc<AppState>>,
    Path((_, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<HolidayLeaveResult>>> {
    ctx.require_permission(Permission::HolidaysManage)?;

    let holiday = state
        .leaves
        .holiday_repo
        .get_by_id(ctx.workspace_id, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Holiday not found".to_string()))?;

    let results = state
        .leaves
        .holiday_leave_service
        .process_holiday_leave(ctx.workspace_id, &holiday)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(results))
}
