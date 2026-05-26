use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ClearWorkspaceDataQuery {
    pub group: String,
}

pub async fn clear_workspace_data(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<ClearWorkspaceDataQuery>,
) -> AppResult<StatusCode> {
    state
        .core
        .workspace_repo
        .clear_workspace_data(&ctx.auth_token, &query.group)
        .await?;
    Ok(StatusCode::OK)
}
