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
use crate::repositories::holiday_repository::HolidayRepository;
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
    let holidays = HolidayRepository::list(&state.db, ctx.workspace_id, query.year)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(holidays))
}

/// GET /workspaces/:ws_id/holidays/:id
pub async fn get_holiday(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    _ctx: MemberContext,
) -> AppResult<Json<Holiday>> {
    let holiday = HolidayRepository::get_by_id(&state.db, id)
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
        &state.db,
        ctx.workspace_id,
        &req.name,
        req.date,
        &req.holiday_type,
        req.is_paid,
        req.ot_multiplier,
        req.description.as_deref(),
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
        &state.db,
        id,
        req.name.as_deref(),
        req.date,
        req.holiday_type.as_deref(),
        req.is_paid,
        req.ot_multiplier,
        req.description.as_deref(),
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

    let deleted = HolidayRepository::delete(&state.db, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if deleted {
        Ok(axum::http::StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Holiday not found".to_string()))
    }
}
