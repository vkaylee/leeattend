use axum::{
    extract::{Query, State},
    response::Json,
};
use std::sync::Arc;

use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{DashboardMetricsQuery, DashboardMetricsResponse};
use crate::services::dashboard_metrics_service::build_dashboard_metrics;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/dashboard/executive-overview",
    params(DashboardMetricsQuery),
    responses((status = 200, description = "Get workspace-scoped executive dashboard metrics", body = DashboardMetricsResponse))
)]
#[tracing::instrument(skip(state, ctx), fields(workspace_id = %ctx.workspace_id), err)]
pub async fn get_dashboard_metrics(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<DashboardMetricsQuery>,
) -> AppResult<Json<DashboardMetricsResponse>> {
    ctx.require_permission(Permission::DashboardRead)?;
    let response = build_dashboard_metrics(
        state.core().workspace_repo(),
        ctx.workspace_id,
        &query,
        chrono::Utc::now(),
        ctx.managed_department_ids.as_ref(),
    )
    .await?;
    Ok(Json(response))
}
