use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;
use crate::{
    handlers::common::MemberContext,
    models::{
        AssignPatternRequest, BulkAssignPatternRequest, BulkAssignResponse, ComplianceCheckRequest,
        CreateWorkingPatternRequest, EmployeeWorkingPattern, WorkingPattern, WorkingPatternDetail,
    },
    services::working_pattern_service as service,
    AppState,
};

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/working-patterns",
    request_body = CreateWorkingPatternRequest,
    responses(
        (status = 201, description = "Working pattern created", body = WorkingPattern)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn create_pattern(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingPatternRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::PatternsCreate)?;
    let pattern = service::create_working_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        Arc::clone(state.shifts().shift_repo()),
        Arc::clone(state.shifts().roster_repo()),
        ctx.workspace_id,
        req,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(pattern)))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/working-patterns",
    responses(
        (status = 200, description = "List of working patterns", body = [WorkingPattern])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn list_patterns(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<WorkingPattern>>> {
    let patterns = service::list_patterns(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
    )
    .await?;
    Ok(Json(patterns))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/working-patterns/{pattern_id}",
    responses(
        (status = 200, description = "Working pattern details", body = [WorkingPatternDetail])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID")
    )
)]
pub async fn get_pattern_details(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<WorkingPatternDetail>>> {
    let (_ws_id, pattern_id) = params;
    let details = service::get_pattern_details(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        pattern_id,
    )
    .await?;
    Ok(Json(details))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employments/{id}/working-patterns",
    request_body = AssignPatternRequest,
    responses(
        (status = 200, description = "Pattern assigned to employment", body = EmployeeWorkingPattern)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employment ID")
    )
)]
pub async fn assign_pattern(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<AssignPatternRequest>,
) -> AppResult<Json<EmployeeWorkingPattern>> {
    let (_ws_id, employment_id) = params;
    ctx.require_permission(Permission::PatternsAssign)?;

    // Compliance check: expand pattern dates and validate each shift_id group
    if let Err(e) = check_pattern_compliance(
        &state,
        &ctx,
        req.pattern_id,
        employment_id,
        req.start_date,
        req.end_date,
        req.allow_warnings,
        req.override_reason.as_deref(),
    )
    .await
    {
        // Only propagate compliance errors; other errors (like DB) are unexpected
        if matches!(e, AppError::Validation(_) | AppError::Conflict(_, _)) {
            return Err(e);
        }
        tracing::warn!(
            employment_id = %employment_id,
            "Pattern compliance check failed: {e}"
        );
    }

    let assignment = service::assign_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        employment_id,
        req,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(assignment))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/working-patterns/bulk-assign",
    request_body = BulkAssignPatternRequest,
    responses(
        (status = 200, description = "Bulk pattern assignments completed", body = BulkAssignResponse)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn bulk_assign_pattern(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<BulkAssignPatternRequest>,
) -> AppResult<Json<BulkAssignResponse>> {
    ctx.require_permission(Permission::PatternsAssign)?;

    // Compliance check for bulk assignment: validate each employment against pattern
    if req.employment_ids.len() <= 10 {
        // Only check compliance for small batches to avoid N+1
        for &employment_id in &req.employment_ids {
            if let Err(e) = check_pattern_compliance(
                &state,
                &ctx,
                req.pattern_id,
                employment_id,
                req.start_date,
                req.end_date,
                req.allow_warnings,
                req.override_reason.as_deref(),
            )
            .await
            {
                if matches!(e, AppError::Validation(_) | AppError::Conflict(_, _)) {
                    return Err(e);
                }
                tracing::warn!(
                    employment_id = %employment_id,
                    "Bulk pattern compliance check failed: {e}"
                );
            }
        }
    }

    let result = service::bulk_assign_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        req,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employments/{id}/working-patterns",
    responses(
        (status = 200, description = "Employment working pattern history", body = [EmployeeWorkingPattern])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employment ID")
    )
)]
pub async fn get_employment_pattern_history(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<EmployeeWorkingPattern>>> {
    let (_ws_id, employment_id) = params;
    let history = service::get_employment_pattern_history(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        employment_id,
    )
    .await?;
    Ok(Json(history))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{workspace_id}/working-patterns/{pattern_id}",
    responses(
        (status = 204, description = "Working pattern deleted")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID")
    )
)]
pub async fn delete_pattern(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_ws_id, pattern_id) = params;
    ctx.require_permission(Permission::PatternsDelete)?;
    service::delete_working_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        pattern_id,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/workspaces/{workspace_id}/working-patterns/{pattern_id}",
    request_body = CreateWorkingPatternRequest,
    responses(
        (status = 200, description = "Working pattern updated", body = WorkingPattern)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID")
    )
)]
pub async fn update_pattern(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingPatternRequest>,
) -> AppResult<Json<WorkingPattern>> {
    let (_ws_id, pattern_id) = params;
    ctx.require_permission(Permission::PatternsCreate)?;
    let pattern = service::update_working_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        Arc::clone(state.shifts().shift_repo()),
        Arc::clone(state.shifts().roster_repo()),
        ctx.workspace_id,
        pattern_id,
        req,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(pattern))
}

// ---------------------------------------------------------------------------
// Helper: pattern compliance check
// ---------------------------------------------------------------------------

/// Expands a pattern's date range and validates each (shift_id, dates[]) group
/// against the workspace compliance rules.
async fn check_pattern_compliance(
    state: &Arc<AppState>,
    ctx: &MemberContext,
    pattern_id: Uuid,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    allow_warnings: bool,
    override_reason: Option<&str>,
) -> AppResult<()> {
    // Get pattern details to know shift_id per schedule_day
    let details = state
        .shifts()
        .working_pattern_repo()
        .get_pattern_details(ctx.workspace_id, pattern_id)
        .await?;

    // Only validate days that have a shift assigned (skip day-off slots)
    let shift_day_map: std::collections::HashMap<i32, Uuid> = details
        .iter()
        .filter_map(|d| d.shift_id.map(|sid| (d.schedule_day, sid)))
        .collect();

    if shift_day_map.is_empty() {
        return Ok(()); // All days off — nothing to validate
    }

    // Find the pattern to get cycle_length_days
    let patterns = state
        .shifts()
        .working_pattern_repo()
        .get_patterns(ctx.workspace_id)
        .await?;
    let pattern = patterns
        .iter()
        .find(|p| p.id == pattern_id)
        .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

    // Expand dates
    let actual_end = end_date.unwrap_or(start_date + chrono::Duration::days(90));
    let mut current = start_date;
    let mut dates = Vec::new();
    while current <= actual_end {
        dates.push(current);
        current += chrono::Duration::days(1);
    }
    dates.truncate(366); // Safety cap

    // Compute which schedule_day each date maps to
    let cycle_days = pattern.cycle_length_days.unwrap_or(7);
    let anchor = start_date;

    // Group dates by shift_id
    let mut shift_dates: std::collections::HashMap<Uuid, Vec<NaiveDate>> =
        std::collections::HashMap::new();
    for date in &dates {
        let days_since_anchor = (*date - anchor).num_days();
        let normalized =
            ((days_since_anchor % cycle_days as i64) + cycle_days as i64) % cycle_days as i64;
        let schedule_day = (normalized as i32) + 1; // 1-indexed

        if let Some(&shift_id) = shift_day_map.get(&schedule_day) {
            shift_dates.entry(shift_id).or_default().push(*date);
        }
    }

    // Validate each shift_id group
    for (shift_id, dates) in &shift_dates {
        let compliance_req = ComplianceCheckRequest {
            employment_id,
            shift_id: *shift_id,
            dates: dates.clone(),
            allow_warnings,
            override_reason: override_reason.map(String::from),
        };
        state
            .shifts()
            .compliance()
            .validate_assignments(ctx.workspace_id, &ctx.auth_token, &compliance_req)
            .await?;
    }

    Ok(())
}
