use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: String,
    pub title: String,
    pub content: String,
    pub is_read: bool,
    pub link: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationCursorRequest {
    pub limit: Option<i64>,
    pub cursor: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub notifications: Vec<Notification>,
    pub next_cursor: Option<DateTime<Utc>>,
    pub unread_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}
