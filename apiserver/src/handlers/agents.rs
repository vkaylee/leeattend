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
use uuid::Uuid;

use crate::agent_manager::AgentCommand;
use crate::handlers::common::MemberContext;
use crate::models::{
    AuthorizedAgent, CreateAuthorizedAgentRequest, CreateDeviceRequest, Device, TrustAgentRequest,
    UpdateAuthorizedAgentRequest, UpdateDeviceRequest,
};
use crate::AppState;

// --- WebSocket Handlers ---

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
                        } else if (v["type"] == "handshake" || !agent_id.is_empty())
                            && agent_id.is_empty()
                        {
                            if let (Some(id), Some(token)) =
                                (v["agent_id"].as_str(), v["token"].as_str())
                            {
                                match state_clone.agent_service.authorize_agent(id, token).await {
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

// --- Agent Handlers ---

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

    match sqlx::query_as::<_, AuthorizedAgent>(&query)
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
    Json(req): Json<TrustAgentRequest>,
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
