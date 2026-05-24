#![allow(unused_variables)]
//! System DB Cluster Management endpoints.
//!
//! All routes here require `SystemContext` with `PlatformAdmin` role.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::Utc;

use crate::errors::AppResult;
use crate::handlers::common::ApiContext;
use crate::handlers::system_auth::SystemContext;
use crate::models::enums::SystemRoleType;
use crate::models::system_db_cluster::{NewSystemDbCluster, SystemDbCluster};
use crate::AppState;

#[derive(Deserialize, ToSchema)]
pub struct CreateDbClusterRequest {
    pub name: String,
    pub admin_database_url: String,
    pub region: String,
    pub max_tenants: i32,
    pub is_active: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateDbClusterStatusRequest {
    pub is_active: bool,
}

/// GET /system/db-clusters
/// List all system database clusters.
pub async fn list_db_clusters(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<Json<Vec<SystemDbCluster>>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let clusters = state.infra.system_db_cluster_repo.list_clusters().await?;

    Ok(Json(clusters))
}

/// POST /system/db-clusters
/// Create a new database cluster.
pub async fn create_db_cluster(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
    Json(req): Json<CreateDbClusterRequest>,
) -> AppResult<Json<SystemDbCluster>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let new_cluster = NewSystemDbCluster {
        id: Uuid::new_v4(),
        name: req.name,
        admin_database_url: req.admin_database_url,
        region: req.region,
        max_tenants: req.max_tenants,
        active_tenants: 0,
        is_active: req.is_active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let cluster = state
        .infra
        .system_db_cluster_repo
        .create_cluster(&new_cluster)
        .await?;

    Ok(Json(cluster))
}

/// DELETE /system/db-clusters/:id
/// Delete a database cluster by ID.
pub async fn delete_db_cluster(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    state.infra.system_db_cluster_repo.delete_cluster(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /system/db-clusters/:id/status
/// Update the active status of a database cluster.
pub async fn update_db_cluster_status(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
    Json(req): Json<UpdateDbClusterStatusRequest>,
) -> AppResult<Json<SystemDbCluster>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let cluster = state
        .infra
        .system_db_cluster_repo
        .update_cluster_status(id, req.is_active)
        .await?;

    Ok(Json(cluster))
}
