use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::AppState;

// ── Response / Request types ───────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DashboardPreferences {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub active_view: String,
    pub visible_widgets: serde_json::Value,
    pub widget_order: serde_json::Value,
    pub collapsed_sections: serde_json::Value,
}

#[derive(Debug, Deserialize)]
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
pub async fn get_dashboard_preferences(
    State(state): State<Arc<AppState>>,
    MemberContext {
        user_id,
        workspace_id,
        ..
    }: MemberContext,
) -> AppResult<Json<DashboardPreferences>> {
    let prefs = sqlx::query_as::<_, DashboardPreferences>(
        "SELECT * FROM dashboard_preferences WHERE user_id = $1 AND workspace_id = $2",
    )
    .bind(user_id)
    .bind(workspace_id)
    .fetch_optional(&state.db)
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
pub async fn update_dashboard_preferences(
    State(state): State<Arc<AppState>>,
    MemberContext {
        user_id,
        workspace_id,
        ..
    }: MemberContext,
    Json(body): Json<UpdateDashboardPreferences>,
) -> AppResult<Json<DashboardPreferences>> {
    let result = sqlx::query_as::<_, DashboardPreferences>(
        r#"
        INSERT INTO dashboard_preferences (user_id, workspace_id, active_view, visible_widgets, widget_order, collapsed_sections)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id, workspace_id)
        DO UPDATE SET
            active_view = COALESCE($3, dashboard_preferences.active_view),
            visible_widgets = COALESCE($4, dashboard_preferences.visible_widgets),
            widget_order = COALESCE($5, dashboard_preferences.widget_order),
            collapsed_sections = COALESCE($6, dashboard_preferences.collapsed_sections),
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(workspace_id)
    .bind(body.active_view.unwrap_or_else(|| "overview".to_string()))
    .bind(body.visible_widgets.unwrap_or_else(default_visible_widgets))
    .bind(body.widget_order.unwrap_or_else(default_widget_order))
    .bind(
        body.collapsed_sections
            .unwrap_or_else(default_collapsed_sections),
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(result))
}
