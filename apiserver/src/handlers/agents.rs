use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono;
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;

use crate::agent_manager::AgentCommand;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    AgentStatus, AuthorizedAgent, CreateAuthorizedAgentRequest, CreateDeviceRequest, Device,
    TrustAgentRequest, UpdateAuthorizedAgentRequest, UpdateDeviceRequest,
};
use crate::repositories::agent_repository::{CreateAuthorizedAgentArgs, CreateDeviceArgs, UpdateDeviceArgs};
use crate::utils::i18n::I18nKey;
use crate::AppState;

// --- WebSocket Handlers ---

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: axum::http::HeaderMap,
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    let agent_id = headers.get("x-agent-id").and_then(|v| v.to_str().ok());
    let token = headers.get("x-token").and_then(|v| v.to_str().ok());

    let (Some(agent_id), Some(token)) = (agent_id, token) else {
        warn!("Rejecting WS connection: Missing x-agent-id or x-token header");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let authorized_agent = match state
        .agents()
        .service()
        .authorize_agent(agent_id, token)
        .await
    {
        Ok(Some(agent)) => agent,
        Ok(None) => {
            warn!(
                "Rejecting WS connection for agent {}: Unauthorized",
                agent_id
            );
            return StatusCode::UNAUTHORIZED.into_response();
        }
        Err(e) => {
            tracing::error!("Database error authorizing agent {}: {}", agent_id, e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut ws_tz = "UTC".to_string();
    let mut ws_date_fmt = "DD/MM/YYYY".to_string();
    let mut ws_time_fmt = "24h".to_string();
    let workspace_id = authorized_agent.workspace_id;

    if let Ok(Some(ws)) = state
        .core()
        .workspace_repo()
        .get_workspace_by_id(workspace_id)
        .await
    {
        ws_tz = ws.timezone().to_string();
        ws_date_fmt = ws.date_format().to_string();
        ws_time_fmt = ws.time_format().to_string();
    }

    let agent_id = agent_id.to_string();

    ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            state,
            agent_id,
            workspace_id,
            ws_tz,
            ws_date_fmt,
            ws_time_fmt,
        )
    })
    .into_response()
}

#[allow(clippy::too_many_lines)]
async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    agent_id: String,
    workspace_id: Uuid,
    ws_tz: String,
    ws_date_fmt: String,
    ws_time_fmt: String,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let mut send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    let state_clone = Arc::clone(&state);

    // Register the agent immediately since they are already pre-authorized at HTTP level
    state
        .agents()
        .manager()
        .add_agent(agent_id.clone(), tx.clone())
        .await;

    let _ = state
        .agents()
        .service()
        .update_last_ip(&agent_id, "websocket_conn")
        .await;

    let _ = state
        .agents()
        .service()
        .notify_config_update(workspace_id, &agent_id)
        .await;

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                        if v["type"] == "attendance" {
                            if !agent_id.is_empty() && !workspace_id.is_nil() {
                                if let (Some(device_id), Some(data)) =
                                    (v["device_id"].as_str(), v.get("data"))
                                {
                                    if let (Some(user_id_number), Some(punch_time_str)) =
                                        (data["user_id"].as_str(), data["timestamp"].as_str())
                                    {
                                        // Try async enqueue via Redis; fall back to sync processing
                                        let result =
                                            if let Some(redis) = state_clone.infra().redis() {
                                                // Async path: enqueue to Redis for worker processing
                                                use crate::queue::{
                                                    Job, JobPayload, JobPriority, JobQueue,
                                                    LogIngestionPayload, TaskType,
                                                };
                                                let payload = LogIngestionPayload {
                                                    workspace_id,
                                                    employee_id_number: user_id_number.to_string(),
                                                    punch_time_raw: punch_time_str.to_string(),
                                                    device_id: Some(device_id.to_string()),
                                                    raw_data: Some(data.clone()),
                                                    timezone: ws_tz.clone(),
                                                    date_fmt: ws_date_fmt.clone(),
                                                    time_fmt: ws_time_fmt.clone(),
                                                };
                                                let job = Job::with_priority(
                                                    crate::utils::id::new_id(),
                                                    0,
                                                    TaskType::LogIngestion,
                                                    workspace_id,
                                                    JobPayload::LogIngestion(payload),
                                                    JobPriority::Low,
                                                );
                                                let queue = JobQueue::new(redis.clone());
                                                queue.enqueue_single(job).await.map_err(|e| {
                                                    format!("Redis enqueue failed: {e}")
                                                })
                                            } else {
                                                Err("Redis not available for async ingestion"
                                                    .to_string())
                                            };
                                        if let Some(msg_id) = v["msg_id"].as_str() {
                                            let (status, message) = match result {
                                                Ok(()) => ("success", None),
                                                Err(e) => ("error", Some(e)),
                                            };
                                            let ack = serde_json::json!({
                                                "type": "ack",
                                                "msg_id": msg_id,
                                                "status": status,
                                                "error": message,
                                                "timestamp": chrono::Utc::now().to_rfc3339()
                                            });
                                            let _ = tx.send(Message::Text(ack.to_string()));
                                        }
                                    }
                                }
                            }
                        } else if v["type"] == "command_response" {
                            if let Ok(resp) = serde_json::from_value::<
                                crate::agent_manager::CentralMessage,
                            >(v.clone())
                            {
                                state_clone.agents().manager().handle_response(resp).await;
                            }
                        } else if v["type"] == "heartbeat" {
                            if let Some(id) = v["agent_id"].as_str() {
                                // Ensure agent updates only their own timestamp
                                if id == agent_id {
                                    state_clone
                                        .agents()
                                        .manager()
                                        .record_heartbeat(id.to_string())
                                        .await;

                                    // Send heartbeat_ack with server_time for clock sync
                                    let _ = tx.send(Message::Text(
                                        serde_json::json!({
                                            "type": "heartbeat_ack",
                                            "server_time": chrono::Utc::now().to_rfc3339()
                                        })
                                        .to_string(),
                                    ));
                                }
                            }
                        } else if v["type"] == "handshake" {
                            // Handshake is already verified at HTTP header level.
                            // Simply log and ignore to maintain backward compatibility.
                            info!(
                                "Received handshake WS frame from pre-authorized agent {}",
                                agent_id
                            );
                        }
                    }
                }
                Message::Close(_) => break,
                _ => (),
            }
        }
        if !agent_id.is_empty() {
            state_clone.agents().manager().remove_agent(&agent_id).await;
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => {
            // Give send_task a chance to flush pending messages (like errors) before closing
            let _ = tokio::time::timeout(Duration::from_secs(2), send_task).await;
        },
    };
}

// --- Agent Handlers ---

#[utoipa::path(
    get,
    path = "/agents",
    responses(
        (status = 200, description = "List all agents status", body = [AgentStatus])
    )
)]
pub async fn get_agents(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<AgentStatus>>> {
    let _workspace_id = ctx.workspace_id;

    // 1. Fetch workspace-scoped authorized agents from DB
    let db_agents = state
        .agents()
        .repo()
        .get_workspace_agents(&ctx.auth_token, false)
        .await?;

    // 2. Get status (online/offline) from AgentManager (Redis-backed)
    let mut agents_status = state
        .agents()
        .manager()
        .get_all_agents_status(db_agents)
        .await;

    if agents_status.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // 3. Batch fetch all devices for these agents to avoid N+1
    let agent_ids: Vec<String> = agents_status.iter().map(|a| a.agent_id.clone()).collect();
    let devices = state
        .agents()
        .repo()
        .get_devices_by_agent_ids(&agent_ids, &ctx.auth_token)
        .await?;

    // Group devices by agent_id
    let mut devices_by_agent: std::collections::HashMap<String, Vec<crate::models::DeviceStatus>> =
        std::collections::HashMap::new();
    for d in devices {
        let is_online = agents_status
            .iter()
            .find(|a| a.agent_id == d.agent_id)
            .is_some_and(|a| a.is_online);

        devices_by_agent
            .entry(d.agent_id.clone())
            .or_default()
            .push(crate::models::DeviceStatus {
                device_id: d.id.to_string(),
                status: if d.is_enabled && is_online {
                    "online".to_string()
                } else {
                    "offline".to_string()
                },
                last_sync: None,
                error: None,
                info: None,
            });
    }

    // Final mapping
    for agent in &mut agents_status {
        agent.devices = devices_by_agent.remove(&agent.agent_id).unwrap_or_default();
    }

    Ok(Json(agents_status))
}

/// POST /agent/{agent_id}/command
#[utoipa::path(
    post,
    path = "/agent/{agent_id}/command",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Command sent successfully")
    )
)]
pub async fn send_command(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(command_request): Json<serde_json::Value>,
) -> AppResult<impl IntoResponse> {
    // Verify that the agent belongs to this workspace
    let agent_exists = state
        .agents()
        .repo()
        .agent_exists_in_workspace(&agent_id, &ctx.auth_token)
        .await?;

    if !agent_exists {
        return Err(AppError::not_found(I18nKey::AgentNotFound));
    }

    let command = AgentCommand {
        id: crate::utils::id::new_id().to_string(),
        r#type: "command".to_string(),
        method: command_request["method"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        device_id: command_request["device_id"]
            .as_str()
            .map(std::string::ToString::to_string),
        data: command_request.get("data").cloned(),
    };

    state
        .agents()
        .manager()
        .send_command(&agent_id, command)
        .await
        .map_err(|_e| AppError::not_found(I18nKey::AgentNotFound))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Command sent" })),
    ))
}

// --- Device Management Handlers ---

#[utoipa::path(
    get,
    path = "/devices",
    responses(
        (status = 200, description = "List all devices", body = [Device])
    )
)]
pub async fn get_devices(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<Device>>> {
    let _workspace_id = ctx.workspace_id;

    let devices = state
        .agents()
        .repo()
        .get_workspace_devices(&ctx.auth_token)
        .await?;

    Ok(Json(devices))
}

/// POST /devices
#[utoipa::path(
    post,
    path = "/devices",
    request_body = CreateDeviceRequest,
    responses(
        (status = 201, description = "Device created successfully", body = Device)
    )
)]
pub async fn create_device(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateDeviceRequest>,
) -> AppResult<impl IntoResponse> {
    let workspace_id = ctx.workspace_id;

    let id = crate::utils::id::new_id();
    let port = req.port.unwrap_or(4370);

    state
        .agents()
        .repo()
        .create_device(CreateDeviceArgs {
            id,
            agent_id: &req.agent_id,
            name: &req.name,
            ip: &req.ip,
            port,
            auth_token: &ctx.auth_token,
            brand: &req.brand.unwrap_or_else(|| "zkteco".to_string()),
            password: req.password.as_deref().unwrap_or(""),
            timezone: &req.timezone.unwrap_or_else(|| "UTC".to_string()),
        })
        .await?;

    let _ = state
        .agents()
        .service()
        .notify_config_update(workspace_id, &req.agent_id)
        .await;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

/// PUT /devices/{id}
#[utoipa::path(
    put,
    path = "/devices/{id}",
    params(
        ("id" = Uuid, Path, description = "Device ID")
    ),
    request_body = UpdateDeviceRequest,
    responses(
        (status = 200, description = "Device updated successfully")
    )
)]
pub async fn update_device(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateDeviceRequest>,
) -> AppResult<impl IntoResponse> {
    let workspace_id = ctx.workspace_id;

    let current_device = state
        .agents()
        .repo()
        .get_device_by_id(id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::DeviceNotFound))?;

    let name = req.name.unwrap_or(current_device.name);
    let ip = req.ip.unwrap_or(current_device.ip);
    let port = req.port.unwrap_or(current_device.port);
    let is_enabled = req.is_enabled.unwrap_or(current_device.is_enabled);
    let brand = req.brand.unwrap_or(current_device.brand);
    let password = if req.password.is_some() {
        req.password
    } else {
        current_device.password
    };
    let timezone = req.timezone.unwrap_or(current_device.timezone);

    state
        .agents()
        .repo()
        .update_device(UpdateDeviceArgs {
            id,
            auth_token: &ctx.auth_token,
            name: &name,
            ip: &ip,
            port,
            is_enabled,
            brand: &brand,
            password: password.as_deref(),
            timezone: &timezone,
        })
        .await?;

    let _ = state
        .agents()
        .service()
        .notify_config_update(workspace_id, &current_device.agent_id)
        .await;

    Ok(StatusCode::OK)
}

/// DELETE /devices/{id}
#[utoipa::path(
    delete,
    path = "/devices/{id}",
    params(
        ("id" = Uuid, Path, description = "Device ID")
    ),
    responses(
        (status = 204, description = "Device deleted successfully")
    )
)]
pub async fn delete_device(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<impl IntoResponse> {
    let workspace_id = ctx.workspace_id;

    let agent_id = state
        .agents()
        .repo()
        .get_agent_id_for_device(id, &ctx.auth_token)
        .await?;

    let rows_affected = state
        .agents()
        .repo()
        .delete_device(id, &ctx.auth_token)
        .await?;

    if rows_affected > 0 {
        if let Some(aid) = agent_id {
            let _ = state
                .agents()
                .service()
                .notify_config_update(workspace_id, &aid)
                .await;
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(I18nKey::DeviceNotFound))
    }
}

// --- Authorized Agents Handlers ---

#[derive(Debug, Deserialize)]
pub struct GetAgentsQueryParams {
    pub include_inactive: Option<bool>,
}

/// GET /authorized-agents
#[utoipa::path(
    get,
    path = "/authorized-agents",
    params(
        ("include_inactive" = Option<bool>, Query, description = "Include inactive agents")
    ),
    responses(
        (status = 200, description = "List all authorized agents", body = [AuthorizedAgent])
    )
)]
pub async fn get_authorized_agents(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<GetAgentsQueryParams>,
) -> AppResult<Json<Vec<AuthorizedAgent>>> {
    let _workspace_id = ctx.workspace_id;

    let include_inactive = params.include_inactive.unwrap_or(false);
    let mut agents = state
        .agents()
        .repo()
        .get_workspace_agents(&ctx.auth_token, include_inactive)
        .await?;

    for agent in &mut agents {
        agent.token = "••••••••".to_string();
    }

    Ok(Json(agents))
}

/// POST /authorized-agents
#[utoipa::path(
    post,
    path = "/authorized-agents",
    request_body = CreateAuthorizedAgentRequest,
    responses(
        (status = 201, description = "Authorized agent created successfully", body = AuthorizedAgent)
    )
)]
pub async fn create_authorized_agent(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateAuthorizedAgentRequest>,
) -> AppResult<impl IntoResponse> {
    let _workspace_id = ctx.workspace_id;

    state
        .agents()
        .repo()
        .create_authorized_agent(CreateAuthorizedAgentArgs {
            id: &req.id,
            name: &req.name,
            token: &req.token,
            auth_token: &ctx.auth_token,
            max_concurrent_workers: req.max_concurrent_workers.unwrap_or(2),
            success_cooldown_secs: req.success_cooldown_secs.unwrap_or(300),
            retention_days: req.retention_days.unwrap_or(365),
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": req.id })),
    ))
}

/// PUT /authorized-agents/{id}/trust
#[utoipa::path(
    put,
    path = "/authorized-agents/{id}/trust",
    params(
        ("id" = String, Path, description = "Agent ID")
    ),
    request_body = TrustAgentRequest,
    responses(
        (status = 200, description = "Agent trust updated successfully")
    )
)]
pub async fn update_agent_trust(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<TrustAgentRequest>,
) -> AppResult<impl IntoResponse> {
    let _workspace_id = ctx.workspace_id;

    let rows_affected = state
        .agents()
        .repo()
        .update_agent_trust(&id, &ctx.auth_token, req.is_trusted)
        .await?;

    if rows_affected > 0 {
        if !req.is_trusted {
            state.agents().manager().remove_agent(&id).await;
        }
        Ok(StatusCode::OK)
    } else {
        Err(AppError::not_found(I18nKey::AgentNotFound))
    }
}

/// GET /authorized-agents/{id}
#[utoipa::path(
    get,
    path = "/authorized-agents/{id}",
    params(
        ("id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Get authorized agent details", body = AuthorizedAgent)
    )
)]
pub async fn get_authorized_agent_by_id(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<AuthorizedAgent>> {
    let _workspace_id = ctx.workspace_id;

    let mut agent = state
        .agents()
        .repo()
        .get_agent_by_id_in_workspace(&id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::AgentNotFound))?;

    agent.token = "••••••••".to_string();

    Ok(Json(agent))
}

/// PATCH /authorized-agents/{id}
#[utoipa::path(
    patch,
    path = "/authorized-agents/{id}",
    params(
        ("id" = String, Path, description = "Agent ID")
    ),
    request_body = UpdateAuthorizedAgentRequest,
    responses(
        (status = 200, description = "Agent updated successfully", body = AuthorizedAgent)
    )
)]
pub async fn update_authorized_agent(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateAuthorizedAgentRequest>,
) -> AppResult<Json<AuthorizedAgent>> {
    let workspace_id = ctx.workspace_id;

    if req.name.is_none()
        && req.token.is_none()
        && req.is_trusted.is_none()
        && req.is_active.is_none()
        && req.max_concurrent_workers.is_none()
        && req.success_cooldown_secs.is_none()
        && req.retention_days.is_none()
    {
        return Err(AppError::validation(I18nKey::NoFieldsToUpdate));
    }

    // Fetch existing
    let mut current_agent = state
        .agents()
        .repo()
        .get_agent_by_id_in_workspace(&id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::AgentNotFound))?;

    // Apply updates
    if let Some(name) = req.name {
        current_agent.name = name;
    }
    if let Some(token) = req.token {
        let trimmed_token = token.trim();
        if trimmed_token != "••••••••" && !trimmed_token.is_empty() {
            current_agent.token = crate::crypto::hash_agent_token(trimmed_token);
        }
    }
    if let Some(is_trusted) = req.is_trusted {
        current_agent.is_trusted = is_trusted;
    }
    if let Some(is_active) = req.is_active {
        current_agent.is_active = is_active;
    }
    if let Some(max_concurrent_workers) = req.max_concurrent_workers {
        current_agent.max_concurrent_workers = max_concurrent_workers;
    }
    if let Some(success_cooldown_secs) = req.success_cooldown_secs {
        current_agent.success_cooldown_secs = success_cooldown_secs;
    }
    if let Some(retention_days) = req.retention_days {
        current_agent.retention_days = retention_days;
    }

    state
        .agents()
        .repo()
        .update_authorized_agent(&current_agent)
        .await?;

    if (!current_agent.is_trusted || !current_agent.is_active)
        && state.agents().manager().is_online(&current_agent.id).await
    {
        state
            .agents()
            .manager()
            .remove_agent(&current_agent.id)
            .await;
    } else {
        let _ = state
            .agents()
            .service()
            .notify_config_update(workspace_id, &current_agent.id)
            .await;
    }

    current_agent.token = "••••••••".to_string();
    Ok(Json(current_agent))
}

/// GET /devices/{id}/info
#[utoipa::path(
    get,
    path = "/devices/{id}/info",
    params(
        ("id" = Uuid, Path, description = "Device ID")
    ),
    responses(
        (status = 200, description = "Get device details on demand", body = serde_json::Value)
    )
)]
pub async fn get_device_info_on_demand(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    let _workspace_id = ctx.workspace_id;

    let device = state
        .agents()
        .repo()
        .get_device_by_id(id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::DeviceNotFound))?;

    let info = state
        .agents()
        .manager()
        .get_device_info(&device.agent_id, &device.id.to_string())
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(info))
}

/// DELETE /authorized-agents/{id}
#[utoipa::path(
    delete,
    path = "/authorized-agents/{id}",
    params(
        ("id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 204, description = "Agent deleted/deactivated successfully")
    )
)]
pub async fn delete_authorized_agent(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<impl IntoResponse> {
    let _workspace_id = ctx.workspace_id;

    let rows_affected = state
        .agents()
        .repo()
        .deactivate_agent(&id, &ctx.auth_token)
        .await?;

    if rows_affected > 0 {
        state.agents().manager().remove_agent(&id).await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(I18nKey::AgentNotFound))
    }
}
