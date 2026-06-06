use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::AppState;

// ── Response / Request types ───────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DashboardPreferences {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub active_view: String,
    pub visible_widgets: serde_json::Value,
    pub widget_order: serde_json::Value,
    pub collapsed_sections: serde_json::Value,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateDashboardPreferences {
    pub active_view: Option<String>,
    pub visible_widgets: Option<serde_json::Value>,
    pub widget_order: Option<serde_json::Value>,
    pub collapsed_sections: Option<serde_json::Value>,
}

// ── Default preferences ────────────────────────────────────────────

fn default_visible_widgets() -> serde_json::Value {
    serde_json::json!({
        "kpi_cards": true,
        "donut_chart": true,
        "quick_stats": true,
        "attendance_trend": true,
        "department_chart": true,
        "overtime_trend": false,
        "peak_times": false,
        "leave_types": false,
        "lateness_by_day": false,
        "turnover_analytics": false,
        "promo_cards": false
    })
}

fn default_widget_order() -> serde_json::Value {
    serde_json::json!([
        "kpi_cards",
        "donut_chart",
        "quick_stats",
        "attendance_trend",
        "department_chart",
        "overtime_trend",
        "peak_times",
        "leave_types",
        "lateness_by_day",
        "turnover_analytics",
        "promo_cards"
    ])
}

fn default_collapsed_sections() -> serde_json::Value {
    serde_json::json!({
        "summary": false,
        "charts": false,
        "actions": true
    })
}

// ── Handlers ───────────────────────────────────────────────────────

/// GET /dashboard/preferences
/// Returns the current user's dashboard preferences for their active workspace.
/// If none exist, returns smart defaults without persisting.
#[utoipa::path(
    get,
    path = "/v1/dashboard/preferences",
    responses(
        (status = 200, description = "Dashboard preferences", body = DashboardPreferences)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip(state, ctx))]
pub async fn get_dashboard_preferences(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<DashboardPreferences>> {
    let workspace_id = ctx.workspace_id;
    let user_id = ctx.user_id;

    let prefs = state
        .core()
        .user_repo()
        .get_dashboard_preferences(user_id, &ctx.auth_token)
        .await?;

    match prefs {
        Some(p) => Ok(Json(p)),
        None => {
            // Return defaults without persisting
            Ok(Json(DashboardPreferences {
                id: Uuid::nil(),
                user_id,
                workspace_id,
                active_view: "overview".to_string(),
                visible_widgets: default_visible_widgets(),
                widget_order: default_widget_order(),
                collapsed_sections: default_collapsed_sections(),
            }))
        }
    }
}

/// PUT /dashboard/preferences
/// Creates or updates the current user's dashboard preferences (upsert).
#[utoipa::path(
    put,
    path = "/v1/dashboard/preferences",
    request_body = UpdateDashboardPreferences,
    responses(
        (status = 200, description = "Dashboard preferences updated", body = DashboardPreferences)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip(state, ctx))]
pub async fn update_dashboard_preferences(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(body): Json<UpdateDashboardPreferences>,
) -> AppResult<Json<DashboardPreferences>> {
    let _workspace_id = ctx.workspace_id;
    let user_id = ctx.user_id;

    let result = state
        .core()
        .user_repo()
        .update_dashboard_preferences(user_id, &ctx.auth_token, body)
        .await?;
    Ok(Json(result))
}
