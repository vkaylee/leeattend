#![allow(unused_variables)]
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
use crate::AppState;

// --- WebSocket Handlers ---

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[allow(clippy::too_many_lines)]
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
    let mut workspace_id = Uuid::nil();
    let mut ws_tz = "UTC".to_string();
    let mut ws_date_fmt = "DD/MM/YYYY".to_string();
    let mut ws_time_fmt = "24h".to_string();
    let state_clone = Arc::clone(&state);

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
                                            if let Some(ref redis) = state_clone.infra.redis {
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
                                                Ok(_) => ("success", None),
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
                                state_clone.agents.manager.handle_response(resp).await;
                            }
                        } else if v["type"] == "heartbeat" {
                            if let Some(id) = v["agent_id"].as_str() {
                                // Ensure agent updates only their own timestamp
                                if id == agent_id {
                                    state_clone
                                        .agents
                                        .manager
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
                        } else if (v["type"] == "handshake" || !agent_id.is_empty())
                            && agent_id.is_empty()
                        {
                            if let (Some(id), Some(token)) =
                                (v["agent_id"].as_str(), v["token"].as_str())
                            {
                                match state_clone.agents.service.authorize_agent(id, token).await {
                                    Ok(Some(agent)) => {
                                        agent_id = id.to_string();
                                        workspace_id = agent.workspace_id;

                                        // Fetch workspace config for timezone aware parsing
                                        if let Ok(Some(ws)) = state_clone
                                            .core
                                            .workspace_repo
                                            .get_workspace_by_id(workspace_id)
                                            .await
                                        {
                                            ws_tz = ws.timezone;
                                            ws_date_fmt = ws.date_format;
                                            ws_time_fmt = ws.time_format;
                                            info!("Agent {} authorized for workspace {} (Timezone: {})", agent_id, workspace_id, ws_tz);
                                        }

                                        state_clone
                                            .agents
                                            .manager
                                            .add_agent(agent_id.clone(), tx.clone())
                                            .await;

                                        let _ = state_clone
                                            .agents
                                            .service
                                            .update_last_ip(&agent_id, "websocket_conn")
                                            .await;

                                        let _ = state_clone
                                            .agents
                                            .service
                                            .notify_config_update(workspace_id, &agent_id)
                                            .await;
                                    }
                                    Ok(None) => {
                                        warn!(
                                            "Agent authorization failed for {}: UNAUTHORIZED_AGENT",
                                            id
                                        );
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
                                        warn!("Agent authorization error for {}: {}", id, code);
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
                Message::Close(_) => break,
                _ => (),
            }
        }
        if !agent_id.is_empty() {
            state_clone.agents.manager.remove_agent(&agent_id).await;
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
    let workspace_id = ctx.workspace_id;

    // 1. Fetch workspace-scoped authorized agents from DB
    let db_agents = state
        .agents
        .repo
        .get_workspace_agents(&ctx.auth_token, false)
        .await?;

    // 2. Get status (online/offline) from AgentManager (Redis-backed)
    let mut agents_status = state.agents.manager.get_all_agents_status(db_agents).await;

    if agents_status.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // 3. Batch fetch all devices for these agents to avoid N+1
    let agent_ids: Vec<String> = agents_status.iter().map(|a| a.agent_id.clone()).collect();
    let devices = state
        .agents
        .repo
        .get_devices_by_agent_ids(&agent_ids, &ctx.auth_token)
        .await?;

    // Group devices by agent_id
    let mut devices_by_agent: std::collections::HashMap<String, Vec<crate::models::DeviceStatus>> =
        std::collections::HashMap::new();
    for d in devices {
        let is_online = agents_status
            .iter()
            .find(|a| a.agent_id == d.agent_id)
            .map(|a| a.is_online)
            .unwrap_or(false);

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

pub async fn send_command(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(command_request): Json<serde_json::Value>,
) -> AppResult<impl IntoResponse> {
    // Verify that the agent belongs to this workspace
    let agent_exists = state
        .agents
        .repo
        .agent_exists_in_workspace(&agent_id, &ctx.auth_token)
        .await?;

    if !agent_exists {
        return Err(AppError::NotFound(
            "Agent not found in this workspace".to_string(),
        ));
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
        .agents
        .manager
        .send_command(&agent_id, command)
        .await
        .map_err(AppError::NotFound)?;

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
    let workspace_id = ctx.workspace_id;

    let devices = state
        .agents
        .repo
        .get_workspace_devices(&ctx.auth_token)
        .await?;

    Ok(Json(devices))
}

pub async fn create_device(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateDeviceRequest>,
) -> AppResult<impl IntoResponse> {
    let workspace_id = ctx.workspace_id;

    let id = crate::utils::id::new_id();
    let port = req.port.unwrap_or(4370);

    state
        .agents
        .repo
        .create_device(
            id,
            &req.agent_id,
            &req.name,
            &req.ip,
            port,
            &ctx.auth_token,
            &req.brand.unwrap_or_else(|| "zkteco".to_string()),
            req.password.as_deref().unwrap_or(""),
            &req.timezone.unwrap_or_else(|| "UTC".to_string()),
        )
        .await?;

    let _ = state
        .agents
        .service
        .notify_config_update(workspace_id, &req.agent_id)
        .await;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn update_device(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateDeviceRequest>,
) -> AppResult<impl IntoResponse> {
    let workspace_id = ctx.workspace_id;

    let current_device = state
        .agents
        .repo
        .get_device_by_id(id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

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
        .agents
        .repo
        .update_device(
            id,
            &ctx.auth_token,
            &name,
            &ip,
            port,
            is_enabled,
            &brand,
            password.as_deref(),
            &timezone,
        )
        .await?;

    let _ = state
        .agents
        .service
        .notify_config_update(workspace_id, &current_device.agent_id)
        .await;

    Ok(StatusCode::OK)
}

pub async fn delete_device(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<impl IntoResponse> {
    let workspace_id = ctx.workspace_id;

    let agent_id = state
        .agents
        .repo
        .get_agent_id_for_device(id, &ctx.auth_token)
        .await?;

    let rows_affected = state.agents.repo.delete_device(id, &ctx.auth_token).await?;

    if rows_affected > 0 {
        if let Some(aid) = agent_id {
            let _ = state
                .agents
                .service
                .notify_config_update(workspace_id, &aid)
                .await;
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Device not found".to_string()))
    }
}

// --- Authorized Agents Handlers ---

#[derive(Debug, Deserialize)]
pub struct GetAgentsQueryParams {
    pub include_inactive: Option<bool>,
}

pub async fn get_authorized_agents(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<GetAgentsQueryParams>,
) -> AppResult<Json<Vec<AuthorizedAgent>>> {
    let workspace_id = ctx.workspace_id;

    let include_inactive = params.include_inactive.unwrap_or(false);
    let agents = state
        .agents
        .repo
        .get_workspace_agents(&ctx.auth_token, include_inactive)
        .await?;

    Ok(Json(agents))
}

pub async fn create_authorized_agent(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateAuthorizedAgentRequest>,
) -> AppResult<impl IntoResponse> {
    let workspace_id = ctx.workspace_id;

    state
        .agents
        .repo
        .create_authorized_agent(
            &req.id,
            &req.name,
            &req.token,
            &ctx.auth_token,
            req.max_concurrent_workers.unwrap_or(2),
            req.success_cooldown_secs.unwrap_or(300),
            req.retention_days.unwrap_or(365),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": req.id })),
    ))
}

pub async fn update_agent_trust(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<TrustAgentRequest>,
) -> AppResult<impl IntoResponse> {
    let workspace_id = ctx.workspace_id;

    let rows_affected = state
        .agents
        .repo
        .update_agent_trust(&id, &ctx.auth_token, req.is_trusted)
        .await?;

    if rows_affected > 0 {
        if !req.is_trusted {
            state.agents.manager.remove_agent(&id).await;
        }
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound("Agent not found".to_string()))
    }
}

pub async fn get_authorized_agent_by_id(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<AuthorizedAgent>> {
    let workspace_id = ctx.workspace_id;

    let agent = state
        .agents
        .repo
        .get_agent_by_id_in_workspace(&id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::NotFound("Agent not found".to_string()))?;

    Ok(Json(agent))
}

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
        return Err(AppError::Validation("No fields to update".to_string()));
    }

    // We fetch current agent first to use as fallback for bounds since diesel raw queries
    // do not have named parameter skipping if we bind them in order.
    // Fetch existing
    let mut current_agent = state
        .agents
        .repo
        .get_agent_by_id_in_workspace(&id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::NotFound("Agent not found".to_string()))?;

    let mut conn = state
        .infra
        .diesel
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Apply updates
    if let Some(name) = req.name {
        current_agent.name = name;
    }
    if let Some(token) = req.token {
        current_agent.token = token;
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

    use diesel_async::RunQueryDsl;

    diesel::sql_query(r#"
        UPDATE authorized_agents
        SET name = $1,
            token = $2,
            is_trusted = $3,
            is_active = $4,
            max_concurrent_workers = $5,
            success_cooldown_secs = $6,
            retention_days = $7
        WHERE id = $8 AND workspace_id = $9
        "#,
    )
    .bind::<diesel::sql_types::Text, _>(&current_agent.name)
    .bind::<diesel::sql_types::Text, _>(&current_agent.token)
    .bind::<diesel::sql_types::Bool, _>(current_agent.is_trusted)
    .bind::<diesel::sql_types::Bool, _>(current_agent.is_active)
    .bind::<diesel::sql_types::Integer, _>(current_agent.max_concurrent_workers)
    .bind::<diesel::sql_types::Integer, _>(current_agent.success_cooldown_secs)
    .bind::<diesel::sql_types::Integer, _>(current_agent.retention_days)
    .bind::<diesel::sql_types::Text, _>(&current_agent.id)
    .bind::<diesel::sql_types::Uuid, _>(current_agent.workspace_id)
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if (!current_agent.is_trusted || !current_agent.is_active)
        && state.agents.manager.is_online(&current_agent.id).await
    {
        state.agents.manager.remove_agent(&current_agent.id).await;
    } else {
        let _ = state
            .agents
            .service
            .notify_config_update(workspace_id, &current_agent.id)
            .await;
    }
    Ok(Json(current_agent))
}

pub async fn get_device_info_on_demand(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    let workspace_id = ctx.workspace_id;

    let device = state
        .agents
        .repo
        .get_device_by_id(id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

    let info = state
        .agents
        .manager
        .get_device_info(&device.agent_id, &device.id.to_string())
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(info))
}

pub async fn delete_authorized_agent(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<impl IntoResponse> {
    let workspace_id = ctx.workspace_id;

    let rows_affected = state
        .agents
        .repo
        .deactivate_agent(&id, &ctx.auth_token)
        .await?;

    if rows_affected > 0 {
        state.agents.manager.remove_agent(&id).await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Agent not found".to_string()))
    }
}
