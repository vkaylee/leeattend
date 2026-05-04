use crate::errors::AppError;
use crate::models::{
    Notification, NotificationCursorRequest, NotificationResponse, UnreadCountResponse,
};
use crate::AppState;
use crate::UserId;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_notifications(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Query(params): Query<NotificationCursorRequest>,
) -> Result<Json<NotificationResponse>, AppError> {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, user_id, type, title, content, is_read, link, created_at FROM notifications WHERE user_id = "
    );
    query_builder.push_bind(user_id);

    if let Some(cursor) = params.cursor {
        query_builder.push(" AND created_at < ");
        query_builder.push_bind(cursor);
    }

    query_builder.push(" ORDER BY created_at DESC LIMIT ");
    query_builder.push_bind(limit + 1);

    let mut notifications: Vec<Notification> = query_builder
        .build_query_as()
        .fetch_all(state.infra.db.api())
        .await?;

    let next_cursor = if notifications.len() > limit as usize {
        let last = notifications.pop();
        last.map(|n| n.created_at)
    } else {
        None
    };

    let unread_count_record = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false",
    )
    .bind(user_id)
    .fetch_one(state.infra.db.api())
    .await?;

    Ok(Json(NotificationResponse {
        notifications,
        next_cursor,
        unread_count: unread_count_record.0,
    }))
}

pub async fn get_unread_count(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> Result<Json<UnreadCountResponse>, AppError> {
    let record = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false",
    )
    .bind(user_id)
    .fetch_one(state.infra.db.api())
    .await?;

    Ok(Json(UnreadCountResponse { count: record.0 }))
}

pub async fn mark_as_read(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2")
        .bind(notification_id)
        .bind(user_id)
        .execute(state.infra.db.api())
        .await?;

    Ok(Json(()))
}

pub async fn mark_all_as_read(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> Result<Json<()>, AppError> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE user_id = $1")
        .bind(user_id)
        .execute(state.infra.db.api())
        .await?;

    Ok(Json(()))
}
