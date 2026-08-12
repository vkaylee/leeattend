use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::system_auth::SystemContext;
use crate::models::diesel::monitor_alert::{MonitorAlert, MonitorAlertEvent};
use crate::models::enums::SystemRoleType;
use crate::repositories::monitor_alert_repository::{
    ListMonitorAlertsParams, MonitorAlertAction, MonitorAlertCursor,
};
use crate::AppState;

const MAX_ALERT_LIMIT: i64 = 100;
const MAX_REASON_CHARS: usize = 500;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct MonitorAlertQuery {
    pub status: Option<String>,
    pub severity: Option<String>,
    /// Opaque cursor from the previous page, ordered by last_seen_at DESC, id DESC.
    pub cursor: Option<String>,
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
pub struct MonitorAlertsResponse {
    pub data: Vec<MonitorAlertResponse>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
    pub next_cursor: Option<String>,
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct MonitorAlertMutationRequest {
    #[schema(min_length = 1, max_length = 500)]
    pub reason: String,
}

fn sanitize_reason(value: &str) -> AppResult<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() || normalized.chars().count() > MAX_REASON_CHARS {
        return Err(AppError::validation(
            crate::utils::i18n::I18nKey::Validation,
        ));
    }
    Ok(sanitize_summary(&normalized))
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
    let parts: Vec<&str> = normalized.split(' ').collect();
    let mut output = Vec::with_capacity(parts.len());
    let mut index = 0;

    while index < parts.len() {
        let part = parts[index];
        if part.contains('@') || part.chars().filter(char::is_ascii_digit).count() >= 8 {
            output.push("[REDACTED]");
            index += 1;
            continue;
        }

        let mut end = index;
        let mut digit_count = 0usize;
        while end < parts.len()
            && parts[end]
                .chars()
                .all(|character| character.is_ascii_digit() || "+-().".contains(character))
        {
            digit_count += parts[end].chars().filter(char::is_ascii_digit).count();
            end += 1;
        }
        if end > index && digit_count >= 8 {
            output.push("[REDACTED]");
            index = end;
        } else {
            output.push(part);
            index += 1;
        }
    }

    output.join(" ").chars().take(500).collect()
}

fn sanitize_code(value: &str) -> String {
    value.trim().chars().take(100).collect()
}

fn alert_cursor_token(cursor: &MonitorAlertCursor) -> String {
    format!("{}:{}", cursor.last_seen_at.timestamp_micros(), cursor.id)
}

fn parse_alert_cursor(token: &str) -> AppResult<MonitorAlertCursor> {
    let (timestamp, id) = token
        .split_once(':')
        .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
    let micros = timestamp
        .parse::<i64>()
        .map_err(|_parse_error| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
    let last_seen_at = chrono::DateTime::<chrono::Utc>::from_timestamp_micros(micros)
        .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
    let id = Uuid::parse_str(id)
        .map_err(|_parse_error| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
    Ok(MonitorAlertCursor { last_seen_at, id })
}

fn validate_filter(value: Option<String>, allowed: &[&str]) -> AppResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let normalized = value.trim().to_ascii_lowercase();
    if allowed.contains(&normalized.as_str()) {
        Ok(Some(normalized))
    } else {
        Err(AppError::validation(
            crate::utils::i18n::I18nKey::Validation,
        ))
    }
}

#[utoipa::path(
    get,
    path = "/monitor/alerts",
    params(MonitorAlertQuery),
    responses(
        (status = 200, description = "Paginated monitor alerts", body = MonitorAlertsResponse),
        (status = 400, description = "Invalid alert filters"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_monitor_alerts(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Query(query): Query<MonitorAlertQuery>,
) -> AppResult<Json<MonitorAlertsResponse>> {
    ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])?;
    let status = validate_filter(query.status, &["active", "resolved"])?;
    let severity = validate_filter(query.severity, &["info", "warning", "critical"])?;
    let cursor = query
        .cursor
        .as_deref()
        .map(parse_alert_cursor)
        .transpose()?;
    let limit = query.limit.unwrap_or(50).clamp(1, MAX_ALERT_LIMIT);
    let page = state
        .infra()
        .monitor_alert_service()
        .list_alerts(ListMonitorAlertsParams {
            status,
            severity,
            cursor,
            limit,
        })
        .await?;
    Ok(Json(MonitorAlertsResponse {
        data: page
            .alerts
            .into_iter()
            .map(MonitorAlertResponse::from)
            .collect(),
        total: usize::try_from(page.total).unwrap_or(0),
        page: 1,
        limit,
        next_cursor: page.next_cursor.as_ref().map(alert_cursor_token),
    }))
}

async fn mutate_monitor_alert(
    state: Arc<AppState>,
    ctx: SystemContext,
    alert_id: Uuid,
    request: MonitorAlertMutationRequest,
    action: MonitorAlertAction,
) -> AppResult<Json<MonitorAlertResponse>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;
    let reason = sanitize_reason(&request.reason)?;
    let alert = state
        .infra()
        .monitor_alert_service()
        .mutate_alert(alert_id, action, reason, ctx.user_id)
        .await?;
    Ok(Json(alert.into()))
}

#[utoipa::path(
    post,
    path = "/monitor/alerts/{alert_id}/acknowledge",
    params(("alert_id" = Uuid, Path)),
    request_body = MonitorAlertMutationRequest,
    responses(
        (status = 200, body = MonitorAlertResponse),
        (status = 400),
        (status = 403),
        (status = 404),
        (status = 409),
        (status = 500)
    )
)]
pub async fn acknowledge_monitor_alert(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Path(alert_id): Path<Uuid>,
    Json(request): Json<MonitorAlertMutationRequest>,
) -> AppResult<Json<MonitorAlertResponse>> {
    mutate_monitor_alert(
        state,
        ctx,
        alert_id,
        request,
        MonitorAlertAction::Acknowledge,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/monitor/alerts/{alert_id}/resolve",
    params(("alert_id" = Uuid, Path)),
    request_body = MonitorAlertMutationRequest,
    responses(
        (status = 200, body = MonitorAlertResponse),
        (status = 400),
        (status = 403),
        (status = 404),
        (status = 409),
        (status = 500)
    )
)]
pub async fn resolve_monitor_alert(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Path(alert_id): Path<Uuid>,
    Json(request): Json<MonitorAlertMutationRequest>,
) -> AppResult<Json<MonitorAlertResponse>> {
    mutate_monitor_alert(state, ctx, alert_id, request, MonitorAlertAction::Resolve).await
}

#[utoipa::path(
    get,
    path = "/monitor/alerts/{alert_id}",
    params(("alert_id" = Uuid, Path)),
    responses(
        (status = 200, body = MonitorAlertDetailResponse),
        (status = 403),
        (status = 404),
        (status = 500)
    )
)]
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
        assert!(value.contains("raw payload"));
    }

    #[test]
    fn sanitizes_segmented_phone_like_values() {
        for value in [
            "call 098-765-4321 now",
            "call 098 765 4321 now",
            "+84 987 654 321",
        ] {
            let sanitized = sanitize_summary(value);
            assert!(!sanitized
                .chars()
                .filter(char::is_ascii_digit)
                .count()
                .eq(&10));
            assert!(sanitized.contains("[REDACTED]"));
        }
    }
}
