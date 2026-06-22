use crate::errors::AppResult;
use crate::AppState;
#[allow(unused_imports)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct DebugUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct ClearDbQuery {
    pub group: String,
}

pub async fn get_debug_users(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<DebugUser>>> {
    #[cfg(debug_assertions)]
    {
        use crate::repositories::DebugRepository;
        let repo = crate::repositories::DieselDebugRepository::new(state.infra().diesel().clone());
        let users = repo.get_debug_users().await?;

        let mapped = users
            .into_iter()
            .map(|u| DebugUser {
                id: u.id,
                name: u.name,
                email: u.email,
                role: u.role,
            })
            .collect();

        Ok(Json(mapped))
    }

    #[cfg(not(debug_assertions))]
    {
        use crate::errors::AppError;
        use crate::utils::i18n::I18nKey;
        let _ = state; // Suppress unused warning in release mode
        Err(AppError::not_found(I18nKey::NotFound))
    }
}

#[allow(clippy::too_many_lines)]
pub async fn clear_debug_db(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ClearDbQuery>,
) -> AppResult<impl IntoResponse> {
    #[cfg(debug_assertions)]
    {
        use crate::repositories::DebugRepository;
        let repo = crate::repositories::DieselDebugRepository::new(state.infra().diesel().clone());
        repo.clear_debug_db(ws_id, &payload.group).await?;

        Ok((StatusCode::OK, "Cleared"))
    }

    #[cfg(not(debug_assertions))]
    {
        use crate::errors::AppError;
        use crate::utils::i18n::I18nKey;
        let _ = (ws_id, state, payload);
        Err::<(), AppError>(AppError::not_found(I18nKey::NotFound))
    }
}
