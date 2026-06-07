use axum::{extract::State, response::Json};
use chrono::Utc;
use std::sync::Arc;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{PayrollSettings, UpdatePayrollSettingsRequest};
use crate::utils::i18n::I18nKey;
use crate::AppState;

/// GET /workspaces/{ws_id}/payroll-settings
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/payroll-settings",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Payroll settings", body = PayrollSettings)
    )
)]
pub async fn get_payroll_settings(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<PayrollSettings>> {
    let workspace_id = ctx.workspace_id;

    let settings = state
        .payroll()
        .repo()
        .get_payroll_settings(workspace_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

    Ok(Json(settings))
}

/// GET /workspaces/{ws_id}/payroll-settings/history
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/payroll-settings/history",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Payroll settings history list", body = [PayrollSettings])
    )
)]
pub async fn get_payroll_settings_history(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<PayrollSettings>>> {
    let workspace_id = ctx.workspace_id;

    let history = state
        .payroll()
        .repo()
        .get_payroll_settings_history(workspace_id)
        .await?;

    Ok(Json(history))
}

/// PATCH /workspaces/{ws_id}/payroll-settings
#[utoipa::path(
    patch,
    path = "/workspaces/{ws_id}/payroll-settings",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = UpdatePayrollSettingsRequest,
    responses(
        (status = 200, description = "Updated payroll settings", body = PayrollSettings)
    )
)]
pub async fn update_payroll_settings(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdatePayrollSettingsRequest>,
) -> AppResult<Json<PayrollSettings>> {
    let workspace_id = ctx.workspace_id;

    // Get current settings first to fill in missing fields if needed
    let effective_date = req
        .effective_date
        .unwrap_or_else(|| Utc::now().date_naive());
    let current = state
        .payroll()
        .repo()
        .get_payroll_settings_at(workspace_id, effective_date)
        .await
        .unwrap_or(None);

    let mode = req.std_working_day_mode.unwrap_or_else(|| {
        current
            .as_ref()
            .map_or_else(|| "fixed".to_string(), |s| s.std_working_day_mode.clone())
    });
    let fixed_days = req.fixed_std_days.unwrap_or_else(|| {
        current
            .as_ref()
            .map_or_else(|| rust_decimal::Decimal::from(26), |s| s.fixed_std_days)
    });
    let days_per_week = req
        .working_days_per_week
        .unwrap_or_else(|| current.as_ref().map_or(6, |s| s.working_days_per_week));
    let hours_per_day = req.hours_per_day.unwrap_or_else(|| {
        current
            .as_ref()
            .map_or_else(|| rust_decimal::Decimal::from(8), |s| s.hours_per_day)
    });
    // max_ot fields: Option<Option<Decimal>>
    // None = not provided (keep current), Some(None) = explicitly set to null (no limit), Some(Some(v)) = set value
    let max_ot_per_day = match req.max_ot_hours_per_day {
        Some(v) => v,
        None => current.as_ref().and_then(|s| s.max_ot_hours_per_day),
    };
    let max_ot_per_month = match req.max_ot_hours_per_month {
        Some(v) => v,
        None => current.as_ref().and_then(|s| s.max_ot_hours_per_month),
    };

    let settings = state
        .payroll()
        .repo()
        .update_payroll_settings(
            workspace_id,
            &crate::models::SavePayrollSettings {
                std_working_day_mode: mode,
                fixed_std_days: fixed_days,
                working_days_per_week: days_per_week,
                hours_per_day,
                max_ot_hours_per_day: max_ot_per_day,
                max_ot_hours_per_month: max_ot_per_month,
                effective_date,
            },
        )
        .await?;

    Ok(Json(settings))
}
