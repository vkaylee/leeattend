use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
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

const WIDGET_IDS: &[&str] = &[
    "kpi_cards",
    "donut_chart",
    "quick_stats",
    "attendance_trend",
    "department_chart",
    "overtime_trend",
    "peak_times",
    "leave_types",
    "lateness_by_day",
    "recruitment_sources",
    "turnover_analytics",
    "unexcused_absences",
    "promo_cards",
];

fn invalid_preferences() -> AppError {
    AppError::validation(crate::utils::i18n::I18nKey::Validation)
}

fn default_object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}

fn normalize_visible_widgets(value: Value) -> AppResult<Value> {
    let Value::Object(raw) = value else {
        return Err(invalid_preferences());
    };
    let mut merged = default_object(default_visible_widgets());
    for (widget_id, enabled) in raw {
        if let Some(default_enabled) = merged.get_mut(&widget_id) {
            if !enabled.is_boolean() {
                return Err(invalid_preferences());
            }
            *default_enabled = enabled;
        }
    }
    Ok(Value::Object(merged))
}

fn normalize_widget_order(value: Value) -> AppResult<Value> {
    let Value::Array(raw) = value else {
        return Err(invalid_preferences());
    };
    let mut seen = std::collections::HashSet::new();
    let mut order = Vec::with_capacity(WIDGET_IDS.len());
    for item in raw {
        let Some(widget_id) = item.as_str() else {
            return Err(invalid_preferences());
        };
        if WIDGET_IDS.contains(&widget_id) && seen.insert(widget_id) {
            order.push(Value::String(widget_id.to_string()));
        }
    }
    for widget_id in WIDGET_IDS {
        if seen.insert(*widget_id) {
            order.push(Value::String((*widget_id).to_string()));
        }
    }
    Ok(Value::Array(order))
}

fn normalize_collapsed_sections(value: Value) -> AppResult<Value> {
    let Value::Object(raw) = value else {
        return Err(invalid_preferences());
    };
    let mut merged = default_object(default_collapsed_sections());
    for (section_id, collapsed) in raw {
        if !collapsed.is_boolean() {
            return Err(invalid_preferences());
        }
        merged.insert(section_id, collapsed);
    }
    Ok(Value::Object(merged))
}

fn normalize_update(body: UpdateDashboardPreferences) -> AppResult<UpdateDashboardPreferences> {
    let active_view = body
        .active_view
        .map(|view| {
            if matches!(
                view.as_str(),
                "overview" | "attendance" | "trends" | "custom"
            ) {
                Ok(view)
            } else {
                Err(invalid_preferences())
            }
        })
        .transpose()?;

    Ok(UpdateDashboardPreferences {
        active_view,
        visible_widgets: body
            .visible_widgets
            .map(normalize_visible_widgets)
            .transpose()?,
        widget_order: body.widget_order.map(normalize_widget_order).transpose()?,
        collapsed_sections: body
            .collapsed_sections
            .map(normalize_collapsed_sections)
            .transpose()?,
    })
}

fn normalize_response(preferences: DashboardPreferences) -> AppResult<DashboardPreferences> {
    let normalized = normalize_update(UpdateDashboardPreferences {
        active_view: Some(preferences.active_view),
        visible_widgets: Some(preferences.visible_widgets),
        widget_order: Some(preferences.widget_order),
        collapsed_sections: Some(preferences.collapsed_sections),
    })?;

    Ok(DashboardPreferences {
        id: preferences.id,
        user_id: preferences.user_id,
        workspace_id: preferences.workspace_id,
        active_view: normalized
            .active_view
            .unwrap_or_else(|| "overview".to_string()),
        visible_widgets: normalized
            .visible_widgets
            .unwrap_or_else(default_visible_widgets),
        widget_order: normalized.widget_order.unwrap_or_else(default_widget_order),
        collapsed_sections: normalized
            .collapsed_sections
            .unwrap_or_else(default_collapsed_sections),
    })
}

// ── Default preferences ────────────────────────────────────────────

pub(crate) fn default_visible_widgets() -> serde_json::Value {
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
        "recruitment_sources": true,
        "turnover_analytics": false,
        "unexcused_absences": false,
        "promo_cards": false
    })
}

pub(crate) fn default_widget_order() -> serde_json::Value {
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
        "recruitment_sources",
        "turnover_analytics",
        "unexcused_absences",
        "promo_cards"
    ])
}

pub(crate) fn default_collapsed_sections() -> serde_json::Value {
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
    ctx.require_permission(Permission::DashboardPreferences)?;
    let workspace_id = ctx.workspace_id;
    let user_id = ctx.user_id;

    let prefs = state
        .core()
        .user_repo()
        .get_dashboard_preferences(user_id, &ctx.auth_token)
        .await?;

    match prefs {
        Some(p) => Ok(Json(normalize_response(p)?)),
        None => Ok(Json(DashboardPreferences {
            id: Uuid::nil(),
            user_id,
            workspace_id,
            active_view: "overview".to_string(),
            visible_widgets: default_visible_widgets(),
            widget_order: default_widget_order(),
            collapsed_sections: default_collapsed_sections(),
        })),
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
    ctx.require_permission(Permission::DashboardPreferences)?;
    let user_id = ctx.user_id;
    let body = normalize_update(body)?;

    let result = state
        .core()
        .user_repo()
        .update_dashboard_preferences(user_id, &ctx.auth_token, body)
        .await?;
    Ok(Json(result))
}
