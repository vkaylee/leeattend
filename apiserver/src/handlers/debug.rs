use axum::{extract::State, Json, response::IntoResponse};
use std::sync::Arc;
use crate::AppState;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
pub struct DebugUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
}

pub async fn get_debug_users(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    #[cfg(debug_assertions)]
    {
        let users = sqlx::query_as::<_, DebugUser>(
            r#"SELECT id, COALESCE(full_name, 'Unknown') as name, email, role FROM users LIMIT 20"#
        )
        .fetch_all(&state.db)
        .await;

        match users {
            Ok(u) => Json(u).into_response(),
            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }

    #[cfg(not(debug_assertions))]
    {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}
