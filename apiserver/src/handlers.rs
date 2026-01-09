use axum::{
    async_trait,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, FromRef, FromRequestParts, Path, Query, State,
    },
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::error;
use uuid::Uuid;

use crate::agent_manager::AgentCommand;
use crate::models::*;
use crate::repositories::user_repository;
use crate::services::auth_service::AuthService;
use crate::services::WorkspaceService;
use crate::{AppState, UserId};

// --- Extractors ---

pub struct WorkspaceId(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for WorkspaceId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let workspace_id_str = parts
            .headers
            .get("X-Workspace-Id")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::BAD_REQUEST,
                "Missing X-Workspace-Id header".to_string(),
            ))?;

        let workspace_id = Uuid::parse_str(workspace_id_str).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid X-Workspace-Id format".to_string(),
            )
        })?;

        Ok(WorkspaceId(workspace_id))
    }
}

pub struct MemberContext {
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub roles: Vec<String>,
}

impl MemberContext {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn is_admin_or_owner(&self) -> bool {
        self.has_role("admin") || self.has_role("owner")
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for MemberContext
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let UserId(user_id) = UserId::from_request_parts(parts, state).await?;
        let WorkspaceId(workspace_id) = WorkspaceId::from_request_parts(parts, state).await?;

        let app_state = Arc::<AppState>::from_ref(state);

        // 1. Fetch Primary Role
        let primary_role = app_state
            .workspace_repo
            .find_primary_role(workspace_id, user_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
            .ok_or((
                StatusCode::FORBIDDEN,
                "Not a member of this workspace".to_string(),
            ))?;

        // 2. Fetch Active Extra Roles
        let all_extra_roles = app_state
            .workspace_repo
            .find_extra_roles(workspace_id, user_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        let mut roles: Vec<String> = all_extra_roles
            .into_iter()
            .filter(|r| r.expires_at > chrono::Utc::now())
            .map(|r| r.role)
            .collect();

        roles.push(primary_role);

        Ok(MemberContext {
            user_id,
            workspace_id,
            roles,
        })
    }
}

// --- Basic Handlers ---

pub async fn health_check() -> impl IntoResponse {
    let response = GenericResponse {
        message: "API Server is running!".to_string(),
    };
    Json(response)
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let mut send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    let mut agent_id = String::new();
    let state_clone = state.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                        if v["type"] == "attendance" {
                            if let Some(msg_id) = v["msg_id"].as_str() {
                                let ack = serde_json::json!({
                                    "type": "ack",
                                    "msg_id": msg_id,
                                    "status": "success",
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                });
                                let _ = tx.send(Message::Text(ack.to_string()));
                            }
                        } else if v["type"] == "status_update" {
                            if let (Some(id), Some(devices_json)) =
                                (v["agent_id"].as_str(), v["devices"].as_array())
                            {
                                let devices: Vec<crate::models::DeviceStatus> =
                                    serde_json::from_value(serde_json::Value::Array(
                                        devices_json.clone(),
                                    ))
                                    .unwrap_or_default();
                                let _ = state_clone
                                    .agent_manager
                                    .update_status(id.to_string(), devices)
                                    .await;
                            }
                        } else if v["type"] == "heartbeat" {
                            if let Some(id) = v["agent_id"].as_str() {
                                // Ensure agent updates only their own timestamp
                                if id == agent_id {
                                    let _ = state_clone.agent_service.update_last_seen(id).await;
                                }
                            }
                        } else if v["type"] == "handshake" || !agent_id.is_empty() {
                            if agent_id.is_empty() {
                                if let (Some(id), Some(token)) =
                                    (v["agent_id"].as_str(), v["token"].as_str())
                                {
                                    match state_clone.agent_service.authorize_agent(id, token).await
                                    {
                                        Ok(Some(_agent)) => {
                                            agent_id = id.to_string();
                                            state_clone
                                                .agent_manager
                                                .add_agent(agent_id.clone(), tx.clone())
                                                .await;

                                            let _ = state_clone
                                                .agent_service
                                                .update_last_ip(&agent_id, "websocket_conn")
                                                .await;

                                            let _ = state_clone
                                                .agent_service
                                                .notify_config_update(&agent_id)
                                                .await;
                                        }
                                        Ok(None) => {
                                            let _ = tx.send(Message::Text(
                                                serde_json::json!({
                                                    "type": "error",
                                                    "code": "UNAUTHORIZED_AGENT",
                                                    "message": "Agent ID không hợp lệ."
                                                })
                                                .to_string(),
                                            ));
                                            break;
                                        }
                                        Err(code) => {
                                            let message = match code.as_str() {
                                                "INVALID_TOKEN" => "Token không chính xác.",
                                                "AGENT_NOT_TRUSTED" => "Agent chưa được phê duyệt.",
                                                _ => "Lỗi xác thực.",
                                            };
                                            let _ = tx.send(Message::Text(
                                                serde_json::json!({
                                                    "type": "error",
                                                    "code": code,
                                                    "message": message
                                                })
                                                .to_string(),
                                            ));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => (),
            }
        }
        if !agent_id.is_empty() {
            state_clone.agent_manager.remove_agent(&agent_id).await;
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

pub async fn get_agents(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let agents = state.agent_manager.get_all_agents_status().await;
    Json(agents)
}

pub async fn send_command(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(command_request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let command = AgentCommand {
        id: Uuid::new_v4().to_string(),
        r#type: "command".to_string(),
        method: command_request["method"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        device_id: command_request["device_id"].as_str().map(|s| s.to_string()),
        data: command_request.get("data").cloned(),
    };

    match state.agent_manager.send_command(&agent_id, command).await {
        Ok(_) => (StatusCode::OK, "Command sent").into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

// --- Device Management Handlers ---

pub async fn get_devices(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as::<_, Device>(
        "SELECT * FROM devices WHERE workspace_id = $1 ORDER BY created_at DESC",
    )
    .bind(workspace_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(devices) => Json(devices).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_device(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<CreateDeviceRequest>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let port = req.port.unwrap_or(4370);

    match sqlx::query(
        "INSERT INTO devices (id, agent_id, name, ip, port, workspace_id) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(&req.agent_id)
    .bind(&req.name)
    .bind(&req.ip)
    .bind(port)
    .bind(workspace_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let _ = state.agent_service.notify_config_update(&req.agent_id).await;
            (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_device(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<UpdateDeviceRequest>,
) -> impl IntoResponse {
    let current_device = match sqlx::query_as::<_, Device>(
        "SELECT * FROM devices WHERE id = $1 AND workspace_id = $2",
    )
    .bind(id)
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(d) => d,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let name = req.name.unwrap_or(current_device.name);
    let ip = req.ip.unwrap_or(current_device.ip);
    let port = req.port.unwrap_or(current_device.port);
    let is_enabled = req.is_enabled.unwrap_or(current_device.is_enabled);

    match sqlx::query(
        "UPDATE devices SET name = $1, ip = $2, port = $3, is_enabled = $4 WHERE id = $5 AND workspace_id = $6",
    )
    .bind(name)
    .bind(ip)
    .bind(port)
    .bind(is_enabled)
    .bind(id)
    .bind(workspace_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let _ = state.agent_service.notify_config_update(&current_device.agent_id).await;
            StatusCode::OK.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_device(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    let agent_id: Option<String> =
        sqlx::query_scalar("SELECT agent_id FROM devices WHERE id = $1 AND workspace_id = $2")
            .bind(id)
            .bind(workspace_id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

    match sqlx::query("DELETE FROM devices WHERE id = $1 AND workspace_id = $2")
        .bind(id)
        .bind(workspace_id)
        .execute(&state.db)
        .await
    {
        Ok(res) if res.rows_affected() > 0 => {
            if let Some(aid) = agent_id {
                let _ = state.agent_service.notify_config_update(&aid).await;
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Authorized Agents Handlers ---

#[derive(Debug, Deserialize)]
pub struct GetAgentsQueryParams {
    pub include_inactive: Option<bool>,
}

pub async fn get_authorized_agents(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(params): Query<GetAgentsQueryParams>,
) -> impl IntoResponse {
    let include_inactive = params.include_inactive.unwrap_or(false);

    let query = if include_inactive {
        "SELECT * FROM authorized_agents WHERE workspace_id = $1 ORDER BY created_at DESC"
            .to_string()
    } else {
        "SELECT * FROM authorized_agents WHERE workspace_id = $1 AND is_active = TRUE ORDER BY created_at DESC"
            .to_string()
    };

    match sqlx::query_as::<_, crate::models::AuthorizedAgent>(&query)
        .bind(workspace_id)
        .fetch_all(&state.db)
        .await
    {
        Ok(agents) => Json(agents).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_authorized_agent(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<CreateAuthorizedAgentRequest>,
) -> impl IntoResponse {
    match sqlx::query(
        "INSERT INTO authorized_agents (id, name, token, workspace_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(&req.id)
    .bind(&req.name)
    .bind(&req.token)
    .bind(workspace_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": req.id })),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_agent_trust(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<crate::models::TrustAgentRequest>,
) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE authorized_agents SET is_trusted = $1 WHERE id = $2 AND workspace_id = $3",
    )
    .bind(req.is_trusted)
    .bind(&id)
    .bind(workspace_id)
    .execute(&state.db)
    .await
    {
        Ok(res) if res.rows_affected() > 0 => {
            if !req.is_trusted {
                state.agent_manager.remove_agent(&id).await;
            }
            StatusCode::OK.into_response()
        }
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_authorized_agent_by_id(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as::<_, AuthorizedAgent>(
        "SELECT * FROM authorized_agents WHERE id = $1 AND workspace_id = $2",
    )
    .bind(id)
    .bind(workspace_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(agent)) => Json(agent).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_authorized_agent(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<UpdateAuthorizedAgentRequest>,
) -> impl IntoResponse {
    let mut query_builder: sqlx::query_builder::QueryBuilder<sqlx::Postgres> =
        sqlx::query_builder::QueryBuilder::new("UPDATE authorized_agents SET ");
    let mut has_fields_to_update = false;

    if let Some(name) = req.name {
        query_builder.push("name = ");
        query_builder.push_bind(name);
        has_fields_to_update = true;
    }
    if let Some(token) = req.token {
        if has_fields_to_update {
            query_builder.push(", ");
        }
        query_builder.push("token = ");
        query_builder.push_bind(token);
        has_fields_to_update = true;
    }
    if let Some(is_trusted) = req.is_trusted {
        if has_fields_to_update {
            query_builder.push(", ");
        }
        query_builder.push("is_trusted = ");
        query_builder.push_bind(is_trusted);
        has_fields_to_update = true;
    }
    if let Some(is_active) = req.is_active {
        if has_fields_to_update {
            query_builder.push(", ");
        }
        query_builder.push("is_active = ");
        query_builder.push_bind(is_active);
        has_fields_to_update = true;
    }

    if !has_fields_to_update {
        return (StatusCode::BAD_REQUEST, "No fields to update").into_response();
    }

    query_builder.push(" WHERE id = ");
    query_builder.push_bind(id.clone());
    query_builder.push(" AND workspace_id = ");
    query_builder.push_bind(workspace_id);
    query_builder.push(" RETURNING *");

    match query_builder
        .build_query_as::<AuthorizedAgent>()
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(agent)) => {
            if (req.is_trusted == Some(false) || req.is_active == Some(false))
                && state.agent_manager.is_online(&agent.id).await
            {
                state.agent_manager.remove_agent(&agent.id).await;
            }
            Json(agent).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_authorized_agent(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE authorized_agents SET is_active = FALSE WHERE id = $1 AND workspace_id = $2",
    )
    .bind(&id)
    .bind(workspace_id)
    .execute(&state.db)
    .await
    {
        Ok(res) if res.rows_affected() > 0 => {
            state.agent_manager.remove_agent(&id).await;
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Workspace Management Handlers ---

pub async fn get_workspaces(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> impl IntoResponse {
    match sqlx::query_as::<_, Workspace>(
        "SELECT w.* FROM workspaces w JOIN workspace_members wm ON w.id = wm.workspace_id WHERE wm.user_id = $1 ORDER BY w.created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(workspaces) => Json(workspaces).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<CreateWorkspaceRequest>,
) -> impl IntoResponse {
    match WorkspaceService::create_workspace(&state.db, &req.name, user_id).await {
        Ok(workspace) => (StatusCode::CREATED, Json(workspace)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn get_workspace_members(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as::<_, crate::models::WorkspaceMemberDetail>(
        "SELECT wm.workspace_id, wm.user_id, wm.role, u.email, u.full_name, u.avatar_url 
         FROM workspace_members wm
         JOIN users u ON wm.user_id = u.id
         WHERE wm.workspace_id = $1",
    )
    .bind(workspace_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(members) => Json(members).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn add_workspace_member(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AddMemberRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can invite members").into_response();
    }

    let user_id: Uuid = match sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "User with this email not found").into_response();
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 3. Add to workspace
    match sqlx::query(
        "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(&req.role)
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: String,
}

pub async fn update_workspace_member_role(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (
            StatusCode::FORBIDDEN,
            "Only admins can change roles".to_string(),
        )
            .into_response();
    }

    match state
        .workspace_repo
        .update_member_role(ctx.workspace_id, user_id, &req.role)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn remove_workspace_member(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (
            StatusCode::FORBIDDEN,
            "Only admins can remove members".to_string(),
        )
            .into_response();
    }

    // Don't allow removing yourself (prevent lockout if only one admin, though owner should probably remain)
    if user_id == ctx.user_id {
        return (
            StatusCode::BAD_REQUEST,
            "You cannot remove yourself from the workspace".to_string(),
        )
            .into_response();
    }

    match state
        .workspace_repo
        .remove_member(ctx.workspace_id, user_id)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

// --- Temporal Roles Handlers ---

pub async fn get_member_roles(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    match WorkspaceService::get_member_roles(
        state.workspace_repo.as_ref(),
        ctx.workspace_id,
        user_id,
    )
    .await
    {
        Ok((primary, extra)) => Json(serde_json::json!({
            "primary": primary,
            "extra": extra
        }))
        .into_response(),
        Err(e) if e == "NOT_FOUND" => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn assign_extra_role(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignRoleRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can assign roles").into_response();
    }

    if req.expires_at <= chrono::Utc::now() {
        return (
            StatusCode::BAD_REQUEST,
            "Expiration time must be in the future".to_string(),
        )
            .into_response();
    }

    match sqlx::query_as::<_, MemberExtraRole>(
        "INSERT INTO member_extra_roles (workspace_id, user_id, role, expires_at) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(&req.role)
    .bind(req.expires_at)
    .fetch_one(&state.db)
    .await
    {
        Ok(extra_role) => (StatusCode::CREATED, Json(extra_role)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn revoke_extra_role(
    Path((_ws_id, _user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can revoke roles").into_response();
    }

    match sqlx::query("DELETE FROM member_extra_roles WHERE id = $1 AND workspace_id = $2")
        .bind(role_id)
        .bind(ctx.workspace_id)
        .execute(&state.db)
        .await
    {
        Ok(res) if res.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> impl IntoResponse {
    tracing::debug!("Fetching profile for user: {}", user_id);
    match user_repository::get_user_by_id(&state.db, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user {}: {}", user_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }) {
        Ok(Some(user)) => Json(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
        })
        .into_response(),
        Ok(None) => {
            tracing::warn!("User {} not found in database", user_id);
            (StatusCode::NOT_FOUND, "User not found".to_string()).into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn get_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match sqlx::query_as::<_, crate::models::User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
    {
        Ok(users) => Json(users).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_user_profile(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<crate::models::UpdateUserRequest>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, crate::models::User>(
        "UPDATE users SET full_name = COALESCE($1, full_name), avatar_url = COALESCE($2, avatar_url), bio = COALESCE($3, bio) WHERE id = $4 RETURNING *"
    )
    .bind(req.full_name)
    .bind(req.avatar_url)
    .bind(req.bio)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => Json(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
        })
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[cfg(debug_assertions)]
pub async fn get_dev_super_user(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    use serde_json::json;
    tracing::info!("Dev fast login requested");

    // 1. Thử lấy user admin mặc định (từ seed)
    let user_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE email = 'admin@leeattend.com' LIMIT 1",
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(id) => id,
        Err(_) => {
            // Nếu không thấy admin@, lấy đại bất kỳ user nào
            match sqlx::query_scalar::<_, Uuid>("SELECT id FROM users LIMIT 1")
                .fetch_one(&state.db)
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Dev login failed: No users found in DB: {}", e);
                    return (
                        StatusCode::NOT_FOUND,
                        "No users found in database. Run migrations or seed your DB.",
                    )
                        .into_response();
                }
            }
        }
    };

    // 2. Lấy workspace đầu tiên mà user này là thành viên
    let workspace_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT workspace_id FROM workspace_members WHERE user_id = $1 LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(id) => id,
        Err(_) => {
            // Nếu user đầu tiên không có workspace, thử tìm bất kỳ workspace nào
            match sqlx::query_scalar::<_, Uuid>(
                "SELECT workspace_id FROM workspace_members LIMIT 1",
            )
            .fetch_one(&state.db)
            .await
            {
                Ok(ws_id) => ws_id,
                Err(e) => {
                    tracing::error!("Dev login failed: No workspace memberships found: {}", e);
                    return (
                        StatusCode::NOT_FOUND,
                        "No workspace memberships found. Please invite user to a workspace.",
                    )
                        .into_response();
                }
            }
        }
    };

    tracing::info!(
        "Dev login successful: user={}, workspace={}",
        user_id,
        workspace_id
    );

    // Sinh JWT cho dev mode
    let access_token =
        AuthService::generate_token(user_id, &state.jwt_secret, 60 * 24 * 7).unwrap_or_default();

    Json(json!({
        "user_id": user_id,
        "workspace_id": workspace_id,
        "access_token": access_token,
        "refresh_token": "dev_refresh_token"
    }))
    .into_response()
}

// --- Auth Handlers ---

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    // 1. Check if email exists
    let existing_user = match user_repository::get_user_by_email(&state.db, &req.email).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error checking existing user: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    if existing_user.is_some() {
        return (StatusCode::CONFLICT, "Email already exists").into_response();
    }

    // 2. Hash password
    let password_hash = match AuthService::hash_password(&req.password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    // 3. Create User
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();
    let user = User {
        id: user_id,
        email: req.email.clone(),
        created_at,
        full_name: Some(req.full_name.clone()),
        avatar_url: None,
        bio: None,
        password_hash, // Used the hashed password
    };

    if let Err(e) = user_repository::create_user(&state.db, &user).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // Convert to UserResponse for the client
    let user_response = UserResponse {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
    };

    // 4. Generate Tokens
    let access_token = match AuthService::generate_token(user.id, &state.jwt_secret, 60) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let refresh_token = match AuthService::create_session(&state.db, user.id).await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    Json(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    })
    .into_response()
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match user_repository::get_user_by_email(&state.db, &req.email).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response(),
        Err(e) => {
            eprintln!("Error fetching user: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    if !AuthService::verify_password(&req.password, &user.password_hash) {
        return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response();
    }

    // Convert to UserResponse for the client
    let user_response = UserResponse {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
    };

    let access_token = match AuthService::generate_token(user.id, &state.jwt_secret, 60) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let refresh_token = match AuthService::create_session(&state.db, user.id).await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    Json(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    })
    .into_response()
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    match AuthService::refresh_session(&state.db, &req.refresh_token, &state.jwt_secret).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => (StatusCode::UNAUTHORIZED, e).into_response(),
    }
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // 1. Get current user to verify old password
    let user = match user_repository::get_user_by_id(&state.db, user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => {
            eprintln!("Error fetching user for password change: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    // 2. Verify old password
    if !AuthService::verify_password(&req.old_password, &user.password_hash) {
        return (StatusCode::UNAUTHORIZED, "Mật khẩu cũ không chính xác").into_response();
    }

    // 3. Hash new password
    let new_hash = match AuthService::hash_password(&req.new_password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    // 4. Update DB
    if let Err(e) = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(&state.db)
        .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // 5. Revoke all sessions (Force re-login)
    let _ = AuthService::revoke_all_sessions_for_user(&state.db, user_id).await;

    (
        StatusCode::OK,
        "Mật khẩu đã được thay đổi. Vui lòng đăng nhập lại.",
    )
        .into_response()
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    match AuthService::revoke_session(&state.db, &req.refresh_token).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> impl IntoResponse {
    tracing::debug!("Fetching profile for user: {}", user_id);
    match user_repository::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) => Json(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
        })
        .into_response(),
        Ok(None) => {
            tracing::warn!("User {} not found in database", user_id);
            (StatusCode::NOT_FOUND, "User not found".to_string()).into_response()
        }
        Err(e) => {
            tracing::error!("Database error fetching user {}: {}", user_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn update_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<crate::models::UpdateUserRequest>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, crate::models::User>(
        "UPDATE users SET full_name = COALESCE($1, full_name), avatar_url = COALESCE($2, avatar_url), bio = COALESCE($3, bio) WHERE id = $4 RETURNING *"
    )
    .bind(req.full_name)
    .bind(req.avatar_url)
    .bind(req.bio)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => Json(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
        })
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Employee Management Handlers ---

pub async fn get_employees(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::services::employee_service::EmployeeService::get_all_employees(
        &state.db,
        workspace_id,
    )
    .await
    {
        Ok(employees) => Json(employees).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_employee(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::services::employee_service::EmployeeService::get_employee_by_id(
        &state.db,
        id,
        workspace_id,
    )
    .await
    {
        Ok(employee) => Json(employee).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn create_employee(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<crate::models::CreateEmployeeRequest>,
) -> impl IntoResponse {
    match crate::services::employee_service::EmployeeService::create_employee(
        &state.db,
        workspace_id,
        req,
    )
    .await
    {
        Ok(employee) => (StatusCode::CREATED, Json(employee)).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn update_employee(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<crate::models::UpdateEmployeeRequest>,
) -> impl IntoResponse {
    match crate::services::employee_service::EmployeeService::update_employee(
        &state.db,
        id,
        workspace_id,
        req,
    )
    .await
    {
        Ok(employee) => Json(employee).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn delete_employee(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::services::employee_service::EmployeeService::delete_employee(
        &state.db,
        id,
        workspace_id,
    )
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(status) => status.into_response(),
    }
}

// --- Time Tracking Handlers ---

pub async fn punch_in(
    State(state): State<Arc<AppState>>,
    Path(_workspace_id): Path<Uuid>,
    Json(req): Json<PunchInRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let entry = TimeEntry {
        id: Uuid::new_v4(),
        employee_id: req.employee_id,
        punch_in_time: Utc::now(),
        punch_out_time: None,
        duration_minutes: None,
        is_break_overtime: req.is_break_overtime.unwrap_or(false),
        entry_type: req.entry_type.unwrap_or_else(|| "work".to_string()),
        note: req.note,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match crate::repositories::time_entry_repository::create_time_entry(&state.db, &entry).await {
        Ok(_) => Ok((StatusCode::CREATED, Json(entry))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn punch_out(
    Path(employee_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<crate::models::PunchOutRequest>,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::punch_out(
        &state.db,
        workspace_id,
        employee_id,
        req,
    )
    .await
    {
        Ok(entry) => Json(entry).into_response(),
        Err(status) => status.into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

pub async fn get_time_history(
    Path(employee_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::get_employee_history(
        &state.db,
        workspace_id,
        employee_id,
        query.start,
        query.end,
    )
    .await
    {
        Ok(entries) => Json(entries).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn update_time_entry(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<crate::models::UpdateTimeEntryRequest>,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::update_time_entry(
        &state.db,
        workspace_id,
        id,
        req,
    )
    .await
    {
        Ok(entry) => Json(entry).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn delete_time_entry(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::delete_time_entry(
        &state.db,
        workspace_id,
        id,
    )
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(status) => status.into_response(),
    }
}

// --- Leave Management Handlers ---

pub async fn submit_leave_request(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<crate::models::SubmitLeaveRequest>,
) -> impl IntoResponse {
    match crate::services::leave_service::LeaveService::submit_leave_request(
        &state.db,
        workspace_id,
        user_id,
        req,
    )
    .await
    {
        Ok(request) => (StatusCode::CREATED, Json(request)).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn approve_leave_request(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<crate::models::ApproveLeaveRequest>,
) -> impl IntoResponse {
    match crate::services::leave_service::LeaveService::approve_leave_request(
        &state.db,
        workspace_id,
        user_id,
        id,
        req,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_my_leave_requests(
    Path(id): Path<Uuid>,
    Extension(_user): Extension<crate::models::User>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::models::LeaveRequest>>, StatusCode> {
    match crate::services::leave_service::LeaveService::get_employee_leave_requests(
        &state.db,
        Uuid::nil(),
        id,
    )
    .await
    {
        Ok(requests) => Ok(Json(requests)),
        Err(status) => Err(status),
    }
}

pub async fn get_employee_leave_balances(
    Path((ws_id, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::models::EmployeeLeaveBalance>>, StatusCode> {
    let year = Utc::now().year();
    match crate::services::leave_service::LeaveService::get_employee_balances(&state.db, ws_id, emp_id, year).await {
        Ok(balances) => Ok(Json(balances)),
        Err(status) => Err(status),
    }
}

#[derive(Deserialize)]
pub struct LeaveStatusQuery {
    pub status: Option<String>,
}

pub async fn get_workspace_leave_requests(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<LeaveStatusQuery>,
) -> impl IntoResponse {
    match crate::services::leave_service::LeaveService::get_workspace_leave_requests(
        &state.db,
        workspace_id,
        query.status,
    )
    .await
    {
        Ok(requests) => Json(requests).into_response(),
        Err(status) => status.into_response(),
    }
}

// --- Attendance Adjustment Handlers ---

pub async fn submit_attendance_adjustment(
    State(state): State<Arc<AppState>>,
    Path(_workspace_id): Path<Uuid>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<CreateAttendanceAdjustmentRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::submit_request(
        &state.db,
        _workspace_id,
        user_id,
        req,
    )
    .await
    {
        Ok(adjustment) => Ok((StatusCode::CREATED, Json(adjustment))),
        Err(e) => Err(e),
    }
}

pub async fn approve_attendance_adjustment(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<ApproveAttendanceAdjustmentRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::approve_request(
        &state.db,
        _workspace_id,
        user_id,
        id,
        req.status,
    )
    .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(e),
    }
}

pub async fn get_my_attendance_adjustments(
    Path(employee_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::get_employee_requests(&state.db, employee_id).await {
        Ok(requests) => Json(requests).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_workspace_attendance_adjustments(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<LeaveStatusQuery>, // Reusing LeaveStatusQuery as it also has status
) -> impl IntoResponse {
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::get_workspace_requests(&state.db, workspace_id, query.status).await {
        Ok(requests) => Json(requests).into_response(),
        Err(status) => status.into_response(),
    }
}

// --- Basic Reporting Handlers ---

pub async fn get_attendance_report(
    State(state): State<Arc<AppState>>,

    MemberContext { workspace_id, .. }: MemberContext,

    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    match crate::services::report_service::ReportService::get_workspace_attendance_summary(
        &state.db,
        workspace_id,
        query.start,
        query.end,
    )
    .await
    {
        Ok(summary) => Json(summary).into_response(),

        Err(status) => status.into_response(),
    }
}

// --- Payroll Handlers ---

pub async fn get_salary_history(
    Path(employee_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_salary_history(&state.db, employee_id).await
    {
        Ok(history) => Json(history).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn add_salary_history(
    Path(employee_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<crate::models::CreateSalaryHistoryRequest>,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::add_salary_history(
        &state.db,
        employee_id,
        req.base_salary,
        req.salary_type,
        req.currency,
        req.effective_date,
    )
    .await
    {
        Ok(history) => (StatusCode::CREATED, Json(history)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct PayrollQuery {
    pub year: i32,
    pub month: u32,
}

pub async fn calculate_payroll(
    Path(employee_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<PayrollQuery>,
) -> impl IntoResponse {
    match crate::services::payroll_service::PayrollService::calculate_monthly_payslip(
        &state.db,
        employee_id,
        workspace_id,
        query.year,
        query.month,
    )
    .await
    {
        Ok(payslip) => Json(payslip).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_employee_payslips(
    Path(employee_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_payslips_by_employee(&state.db, employee_id)
        .await
    {
        Ok(payslips) => Json(payslips).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Department Handlers ---

pub async fn get_departments(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::department_repository::get_departments(&state.db, workspace_id).await
    {
        Ok(departments) => Json(departments).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_department(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<crate::models::CreateDepartmentRequest>,
) -> impl IntoResponse {
    match crate::repositories::department_repository::create_department(
        &state.db,
        workspace_id,
        req.name,
        req.description,
        req.manager_id,
    )
    .await
    {
        Ok(dept) => (StatusCode::CREATED, Json(dept)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn transfer_employee(
    Path(employee_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<crate::models::TransferEmployeeRequest>,
) -> impl IntoResponse {
    match crate::repositories::department_repository::transfer_employee(
        &state.db,
        employee_id,
        req.department_id,
        req.effective_date,
        req.notes,
    )
    .await
    {
        Ok(history) => (StatusCode::CREATED, Json(history)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_employee_department_history(
    Path(employee_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::department_repository::get_employee_department_history(
        &state.db,
        employee_id,
    )
    .await
    {
        Ok(history) => Json(history).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Shift Handlers ---

pub async fn get_shifts(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::services::shift_service::ShiftService::get_workspace_shifts(
        &state.db,
        workspace_id,
    )
    .await
    {
        Ok(shifts) => Json(shifts).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn create_shift(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<CreateShiftRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    match crate::services::shift_service::ShiftService::create_shift(
        &state.db,
        workspace_id,
        req.name,
        req.start_time,
        req.end_time,
        req.break_duration_minutes,
        req.break_start,
        req.break_end,
        req.is_break_auto_deduct,
        req.break_is_overtime,
        req.early_punch_in_grace_period_minutes,
    )
    .await
    {
        Ok(shift) => Ok((StatusCode::CREATED, Json(shift))),
        Err(e) => Err(e),
    }
}

pub async fn update_shift(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, shift_id)): Path<(Uuid, Uuid)>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<UpdateShiftRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    match crate::services::shift_service::ShiftService::update_shift(
        &state.db,
        shift_id,
        req,
    )
    .await
    {
        Ok(shift) => Ok(Json(shift)),
        Err(status) => Err(status),
    }
}


pub async fn assign_shift(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<crate::models::AssignShiftRequest>,
) -> impl IntoResponse {
    match crate::services::shift_service::ShiftService::assign_shift(
        &state.db,
        req.employee_id,
        req.shift_id,
        req.start_date,
        req.end_date,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(status) => status.into_response(),
    }
}

// --- Roster Handlers ---

pub async fn get_workspace_schedules(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<crate::models::EmployeeSchedule>>, StatusCode> {
    let start_date = params.get("start_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let end_date = params.get("end_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let schedules = crate::repositories::roster_repository::get_workspace_schedules(
        &state.db,
        workspace_id,
        start_date,
        end_date
    )
    .await
    .map_err(|e| {
        error!("Failed to get workspace schedules: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(schedules))
}

pub async fn get_assignment_history(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::AssignmentHistory>>, StatusCode> {
    let history = crate::repositories::roster_repository::get_assignment_history(
        &state.db,
        workspace_id,
        50 // Limit to last 50 entries
    )
    .await
    .map_err(|e| {
        error!("Failed to get assignment history: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(history))
}

pub async fn get_employee_rosters(
    State(state): State<Arc<AppState>>,

    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,

    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<crate::models::ShiftRoster>>, StatusCode> {
    let start_date = params
        .get("start_date")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let end_date = params
        .get("end_date")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| start_date + chrono::Duration::days(30));

    let rosters = crate::repositories::roster_repository::get_employee_rosters(
        &state.db,
        employee_id,
        start_date,
        end_date,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rosters))
}

pub async fn upsert_roster(
    State(state): State<Arc<AppState>>,

    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,

    Json(req): Json<crate::models::CreateRosterRequest>,
) -> Result<Json<crate::models::ShiftRoster>, StatusCode> {
    let roster = crate::models::ShiftRoster {
        id: Uuid::new_v4(),

        employee_id,

        shift_id: req.shift_id,

        date: req.date,

        status: "confirmed".to_string(),

        notes: req.notes,

        created_at: chrono::Utc::now(),

        updated_at: chrono::Utc::now(),
    };

    let saved = crate::repositories::roster_repository::upsert_roster(&state.db, &roster)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(saved))
}
