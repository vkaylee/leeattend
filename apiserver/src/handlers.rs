use axum::{
    async_trait,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRef, FromRequestParts, Path, Query, State,
    },
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::agent_manager::AgentCommand;
use crate::models::{
    AddMemberRequest, AssignRoleRequest, AuthResponse, AuthorizedAgent, ChangePasswordRequest,
    CreateAuthorizedAgentRequest, CreateDeviceRequest, CreateWorkspaceRequest, Device,
    GenericResponse, LoginRequest, MemberExtraRole, RefreshRequest, RegisterRequest,
    UpdateAuthorizedAgentRequest, UpdateDeviceRequest, UpdateUserRequest, User, Workspace,
};
use crate::services::auth_service::AuthService;
use crate::services::WorkspaceService;
use crate::{AppState, UserId};
use chrono::Utc;

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
        // Note: For complex queries with conditions like expires_at > NOW(),
        // we should add a specific method to repository trait or keep it simple if it's very specific.
        // Let's assume find_extra_roles returns all, and we filter here,
        // OR better, add a specific method to WorkspaceRepository for active roles.
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
                                state_clone
                                    .agent_manager
                                    .update_status(id.to_string(), devices)
                                    .await;
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
    match sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(user)) => Json(user).into_response(),
        Ok(None) => {
            tracing::warn!("User {} not found in database", user_id);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(e) => {
            tracing::error!("Database error fetching user {}: {}", user_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
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
        Ok(user) => Json(user).into_response(),
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
    let exists: bool =
        match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(&req.email)
            .fetch_one(&state.db)
            .await
        {
            Ok(e) => e,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };

    if exists {
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
        password_hash: Some(password_hash.clone()),
    };

    if let Err(e) = sqlx::query(
        "INSERT INTO users (id, email, password_hash, full_name, created_at) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(user.id)
    .bind(&user.email)
    .bind(&password_hash)
    .bind(&user.full_name)
    .bind(user.created_at)
    .execute(&state.db)
    .await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

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
        user,
    })
    .into_response()
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(u)) => u,
        _ => return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response(),
    };

    let password_hash = match user.password_hash.as_ref() {
        Some(h) => h,
        None => return (StatusCode::UNAUTHORIZED, "User has no password set").into_response(),
    };

    if !AuthService::verify_password(&req.password, password_hash) {
        return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response();
    }

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
        user,
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
    // 1. Lấy password_hash hiện tại
    let current_hash: String =
        match sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&state.db)
            .await
        {
            Ok(Some(h)) => h,
            Ok(None) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "User has no password set",
                )
                    .into_response()
            }
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };

    // 2. Xác thực mật khẩu cũ
    if !AuthService::verify_password(&req.old_password, &current_hash) {
        return (StatusCode::UNAUTHORIZED, "Mật khẩu cũ không chính xác").into_response();
    }

    // 3. Hash mật khẩu mới
    let new_hash = match AuthService::hash_password(&req.new_password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    // 4. Cập nhật DB
    if let Err(e) = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(&state.db)
        .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // 5. Thu hồi tất cả session (Bắt buộc login lại)
    let _ = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await;

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
    match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
    {
        Ok(user) => Json(user).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

pub async fn update_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = sqlx::query(
        "UPDATE users SET full_name = COALESCE($1, full_name), avatar_url = COALESCE($2, avatar_url), bio = COALESCE($3, bio) WHERE id = $4"
    )
    .bind(req.full_name)
    .bind(req.avatar_url)
    .bind(req.bio)
    .bind(user_id)
    .execute(&state.db)
    .await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
    {
        Ok(user) => Json(user).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}
