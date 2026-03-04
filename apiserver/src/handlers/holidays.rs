//! Holiday CRUD handlers for workspace holiday calendar management.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{CreateHolidayRequest, Holiday, UpdateHolidayRequest};
use crate::repositories::holiday_repository::{HolidayRepository, HolidayLeaveResult};
use crate::services::HolidayLeaveService;
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
    let holidays = HolidayRepository::list(state.db.api(), ctx.workspace_id, query.year)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(holidays))
}

/// GET /workspaces/:ws_id/holidays/:id
pub async fn get_holiday(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Holiday>> {
    let holiday = HolidayRepository::get_by_id(state.db.api(), ctx.workspace_id, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Holiday not found".to_string()))?;

    Ok(Json(holiday))
}

/// POST /workspaces/:ws_id/holidays
pub async fn create_holiday(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateHolidayRequest>,
) -> AppResult<(axum::http::StatusCode, Json<Holiday>)> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Admin or Owner role required to manage holidays".to_string(),
        ));
    }

    let holiday = HolidayRepository::create(
        state.db.api(),
        ctx.workspace_id,
        &req.name,
        req.date,
        &req.holiday_type,
        req.is_paid,
        req.leave_type_id,
        req.ot_multiplier,
        req.description.as_deref(),
        req.work_threshold_percent,
        req.is_credit_on_work,
    )
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
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateHolidayRequest>,
) -> AppResult<Json<Holiday>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Admin or Owner role required to manage holidays".to_string(),
        ));
    }

    let holiday = HolidayRepository::update(
        state.db.api(),
        ctx.workspace_id,
        id,
        req.name.as_deref(),
        req.date,
        req.holiday_type.as_deref(),
        req.is_paid,
        req.leave_type_id,
        req.ot_multiplier,
        req.description.as_deref(),
        req.work_threshold_percent,
        req.is_credit_on_work,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Holiday not found".to_string()))?;

    Ok(Json(holiday))
}

/// DELETE /workspaces/:ws_id/holidays/:id
pub async fn delete_holiday(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<axum::http::StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Admin or Owner role required to manage holidays".to_string(),
        ));
    }

    let deleted = HolidayRepository::delete(state.db.api(), ctx.workspace_id, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

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
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<HolidayLeaveResult>>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Admin or Owner role required to process holiday leave".to_string(),
        ));
    }

    let holiday = HolidayRepository::get_by_id(state.db.api(), ctx.workspace_id, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Holiday not found".to_string()))?;

    let results = HolidayLeaveService::process_holiday_leave(
        state.db.api(),
        ctx.workspace_id,
        &holiday,
    )
    .await
    .map_err(AppError::Internal)?;

    Ok(Json(results))
}
