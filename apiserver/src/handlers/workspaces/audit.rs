use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::audit::AuditLog;
use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PaginatedAuditLogs {
    pub data: Vec<AuditLog>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQueryParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub action: Option<String>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// GET /workspaces/{ws_id}/audit-logs
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/audit-logs",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("limit" = Option<i64>, Query, description = "Items per page"),
        ("action" = Option<String>, Query, description = "Filter by action"),
        ("start_date" = Option<chrono::DateTime<chrono::Utc>>, Query, description = "Filter by start date"),
        ("end_date" = Option<chrono::DateTime<chrono::Utc>>, Query, description = "Filter by end date")
    ),
    responses(
        (status = 200, description = "Paginated list of audit logs", body = PaginatedAuditLogs)
    )
)]
pub async fn get_audit_logs(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<AuditLogQueryParams>,
) -> AppResult<Json<PaginatedAuditLogs>> {
    // Only admins or those with specific permission should see audit logs.
    ctx.require_permission(Permission::AuditLogsRead)?;

    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;

    let (logs, total) = state
        .core()
        .audit_repo()
        .find_by_workspace(
            ctx.workspace_id,
            limit,
            offset,
            params.action,
            params.start_date,
            params.end_date,
        )
        .await?;

    Ok(Json(PaginatedAuditLogs {
        data: logs,
        total,
        page,
        limit,
    }))
}
