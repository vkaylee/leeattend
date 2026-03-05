use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::audit::AuditLog;
use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::PaginationParams;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedAuditLogs {
    pub data: Vec<AuditLog>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

pub async fn get_audit_logs(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedAuditLogs>> {
    // Only admins or those with specific permission should see audit logs.
    // Let's reuse MembersRolesManage or we can just require admin via specific logic.
    ctx.require_permission(Permission::MembersRolesManage)?;

    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;

    let (logs, total) = state
        .audit_repo
        .find_by_workspace(ws_id, limit, offset)
        .await?;

    Ok(Json(PaginatedAuditLogs {
        data: logs,
        total,
        page,
        limit,
    }))
}
