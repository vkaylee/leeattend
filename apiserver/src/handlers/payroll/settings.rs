use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::UpdatePayrollSettingsRequest;
use crate::AppState;

pub async fn get_payroll_settings(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_payroll_settings(&state.db, workspace_id)
        .await
    {
        Ok(Some(settings)) => Json(settings).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_payroll_settings_history(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as!(
        crate::models::PayrollSettings,
        r#"SELECT workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, effective_date, created_at as "created_at!", updated_at as "updated_at!" 
           FROM payroll_settings 
           WHERE workspace_id = $1
           ORDER BY effective_date DESC"#,
        workspace_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(history) => Json(history).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_payroll_settings(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<UpdatePayrollSettingsRequest>,
) -> impl IntoResponse {
    // Get current settings first to fill in missing fields if needed
    let effective_date = req.effective_date.unwrap_or_else(|| Utc::now().date_naive());
    let current =
        crate::repositories::payroll_repository::get_payroll_settings_at(&state.db, workspace_id, effective_date)
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

    match crate::repositories::payroll_repository::update_payroll_settings(
        &state.db,
        workspace_id,
        mode,
        fixed_days,
        days_per_week,
        hours_per_day,
        effective_date,
    )
    .await
    {
        Ok(settings) => Json(settings).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
