use axum::{extract::{Path, Query, State}, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::system_auth::SystemContext;
use crate::models::diesel::monitor_alert::{MonitorAlert, MonitorAlertEvent};
use crate::models::enums::SystemRoleType;
use crate::repositories::monitor_alert_repository::ListMonitorAlertsParams;
use crate::AppState;

const MAX_ALERT_LIMIT: i64 = 100;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct MonitorAlertQuery {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MonitorAlertResponse {
    pub id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub summary: String,
    pub source: String,
    pub first_seen_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
    pub occurrence_count: i32,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MonitorAlertDetailResponse {
    #[serde(flatten)]
    pub alert: MonitorAlertResponse,
    pub events: Vec<MonitorAlertEventResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MonitorAlertEventResponse {
    pub id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub summary: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

impl From<MonitorAlert> for MonitorAlertResponse {
    fn from(alert: MonitorAlert) -> Self {
        Self {
            id: alert.id,
            alert_type: alert.alert_type,
            severity: alert.severity,
            status: alert.status,
            title: alert.title,
            summary: sanitize_summary(&alert.summary),
            source: sanitize_code(&alert.source),
            first_seen_at: alert.first_seen_at,
            last_seen_at: alert.last_seen_at,
            occurrence_count: alert.occurrence_count,
            resolved_at: alert.resolved_at,
        }
    }
}

impl From<MonitorAlertEvent> for MonitorAlertEventResponse {
    fn from(event: MonitorAlertEvent) -> Self {
        Self {
            id: event.id,
            event_type: sanitize_code(&event.event_type),
            severity: event.severity,
            summary: sanitize_summary(&event.summary),
            occurred_at: event.occurred_at,
        }
    }
}

fn sanitize_summary(value: &str) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let redacted = normalized
        .split(' ')
        .map(|part| {
            let digits = part.chars().filter(char::is_ascii_digit).count();
            if part.contains('@') || digits >= 8 {
                "[REDACTED]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    redacted.chars().take(500).collect()
}

fn sanitize_code(value: &str) -> String {
    value.trim().chars().take(100).collect()
}

fn validate_filter(value: Option<String>, allowed: &[&str]) -> AppResult<Option<String>> {
    let Some(value) = value else { return Ok(None); };
    let normalized = value.trim().to_ascii_lowercase();
    if allowed.contains(&normalized.as_str()) {
        Ok(Some(normalized))
    } else {
        Err(AppError::validation(crate::utils::i18n::I18nKey::Validation))
    }
}

#[utoipa::path(get, path = "/monitor/alerts", params(MonitorAlertQuery), responses((status = 200, body = [MonitorAlertResponse])))]
pub async fn list_monitor_alerts(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Query(query): Query<MonitorAlertQuery>,
) -> AppResult<Json<Vec<MonitorAlertResponse>>> {
    ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])?;
    let status = validate_filter(query.status, &["active", "resolved"])?;
    let severity = validate_filter(query.severity, &["info", "warning", "critical"])?;
    let limit = query.limit.unwrap_or(50).clamp(1, MAX_ALERT_LIMIT);
    let alerts = state
        .infra()
        .monitor_alert_service()
        .list_alerts(ListMonitorAlertsParams { status, severity, limit })
        .await?;
    Ok(Json(alerts.into_iter().map(MonitorAlertResponse::from).collect()))
}

#[utoipa::path(get, path = "/monitor/alerts/{alert_id}", params(("alert_id" = Uuid, Path)), responses((status = 200, body = MonitorAlertDetailResponse), (status = 404)))]
pub async fn get_monitor_alert(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Path(alert_id): Path<Uuid>,
) -> AppResult<Json<MonitorAlertDetailResponse>> {
    ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])?;
    let (alert, events) = state
        .infra()
        .monitor_alert_service()
        .get_alert(alert_id)
        .await?
        .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;
    Ok(Json(MonitorAlertDetailResponse {
        alert: alert.into(),
        events: events.into_iter().map(Into::into).collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_payload_like_values() {
        let value = sanitize_summary("email user@example.com request 12345678 raw payload");
        assert!(!value.contains("user@example.com"));
        assert!(!value.contains("12345678"));
        assert!(!value.contains("payload"));
    }
}
