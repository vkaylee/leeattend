use crate::errors::AppError;
use crate::models::{NotificationCursorRequest, NotificationResponse, UnreadCountResponse};
use crate::AppState;
use crate::UserId;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/v1/notifications",
    params(
        ("limit" = Option<i64>, Query, description = "Number of notifications to return"),
        ("cursor" = Option<DateTime<Utc>>, Query, description = "Pagination cursor")
    ),
    responses(
        (status = 200, description = "List of notifications", body = NotificationResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip(state))]
pub async fn get_notifications(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Query(params): Query<NotificationCursorRequest>,
) -> Result<Json<NotificationResponse>, AppError> {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let mut notifications = state
        .core()
        .user_repo()
        .get_notifications(user_id, limit + 1, params.cursor)
        .await?;

    let next_cursor = if notifications.len() > limit as usize {
        let last = notifications.pop();
        last.map(|n| n.created_at)
    } else {
        None
    };

    let unread_count = state
        .core()
        .user_repo()
        .get_unread_notification_count(user_id)
        .await?;

    Ok(Json(NotificationResponse {
        notifications,
        next_cursor,
        unread_count,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/notifications/unread-count",
    responses(
        (status = 200, description = "Unread notification count", body = UnreadCountResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip(state))]
pub async fn get_unread_count(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> Result<Json<UnreadCountResponse>, AppError> {
    let count = state
        .core()
        .user_repo()
        .get_unread_notification_count(user_id)
        .await?;
    Ok(Json(UnreadCountResponse { count }))
}

#[utoipa::path(
    patch,
    path = "/v1/notifications/{id}/read",
    params(
        ("id" = Uuid, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification marked as read")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip(state))]
pub async fn mark_as_read(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    state
        .core()
        .user_repo()
        .mark_notification_as_read(user_id, notification_id)
        .await?;
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/v1/notifications/read-all",
    responses(
        (status = 200, description = "All notifications marked as read")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip(state))]
pub async fn mark_all_as_read(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> Result<Json<()>, AppError> {
    state
        .core()
        .user_repo()
        .mark_all_notifications_as_read(user_id)
        .await?;
    Ok(Json(()))
}
