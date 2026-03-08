use axum::{
    extract::{Path, State},
    response::Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{PayrollSettings, UpdatePayrollSettingsRequest};
use crate::AppState;

pub async fn get_payroll_settings(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> AppResult<Json<PayrollSettings>> {
    let settings =
        state.payroll.repo.get_payroll_settings(workspace_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Settings not found".to_string()))?;

    Ok(Json(settings))
}

pub async fn get_payroll_settings_history(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> AppResult<Json<Vec<PayrollSettings>>> {
    let history = sqlx::query_as!(
        crate::models::PayrollSettings,
        r#"SELECT workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, max_ot_hours_per_day, max_ot_hours_per_month, effective_date, created_at as "created_at!", updated_at as "updated_at!"
           FROM payroll_settings
           WHERE workspace_id = $1
           ORDER BY effective_date DESC"#,
        workspace_id
    )
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(history))
}

pub async fn update_payroll_settings(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<UpdatePayrollSettingsRequest>,
) -> AppResult<Json<PayrollSettings>> {
    // Get current settings first to fill in missing fields if needed
    let effective_date = req
        .effective_date
        .unwrap_or_else(|| Utc::now().date_naive());
    let current = state.payroll.repo.get_payroll_settings_at(workspace_id,
        effective_date,
    )
    .await
    .unwrap_or(None);

    let mode = req.std_working_day_mode.unwrap_or_else(|| {
        current
            .as_ref()
            .map(|s| s.std_working_day_mode.clone())
            .unwrap_or("fixed".to_string())
    });
    let fixed_days = req.fixed_std_days.unwrap_or_else(|| {
        current
            .as_ref()
            .map(|s| s.fixed_std_days)
            .unwrap_or(rust_decimal::Decimal::from(26))
    });
    let days_per_week = req.working_days_per_week.unwrap_or_else(|| {
        current
            .as_ref()
            .map(|s| s.working_days_per_week)
            .unwrap_or(6)
    });
    let hours_per_day = req.hours_per_day.unwrap_or_else(|| {
        current
            .as_ref()
            .map(|s| s.hours_per_day)
            .unwrap_or(rust_decimal::Decimal::from(8))
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

    let settings = state.payroll.repo.update_payroll_settings(workspace_id,
        mode,
        fixed_days,
        days_per_week,
        hours_per_day,
        max_ot_per_day,
        max_ot_per_month,
        effective_date,
    )
    .await?;

    Ok(Json(settings))
}

