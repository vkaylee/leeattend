use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::agent_manager::AgentCommand;
use crate::models::{GenericResponse, CreateDeviceRequest, UpdateDeviceRequest, Device};
use crate::AppState;

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

    // Task to send messages from our internal channel to the websocket
    let mut send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    let mut agent_id = String::new();

    // Task to receive messages from the websocket
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                        // Handle handshake or messages
                        if v["type"] == "attendance" {
                            tracing::info!("Received attendance log from {}: {:?}", v["agent_id"], v["data"]);
                            // Send ACK back to agent
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
                            if let (Some(id), Some(devices_json)) = (v["agent_id"].as_str(), v["devices"].as_array()) {
                                let devices: Vec<crate::models::DeviceStatus> = serde_json::from_value(serde_json::Value::Array(devices_json.clone())).unwrap_or_default();
                                state_clone.agent_manager.update_status(id.to_string(), devices).await;
                                tracing::debug!("Updated status for agent {}", id);
                            }
                        } else if v["type"] == "handshake" || !agent_id.is_empty() {
                            if agent_id.is_empty() {
                                if let Some(id) = v["agent_id"].as_str() {
                                    let is_duplicate = {
                                        let active_agents = state_clone.agent_manager.agents.read().await;
                                        active_agents.contains_key(id)
                                    };

                                    if is_duplicate {
                                        tracing::warn!("Rejecting duplicate connection attempt for agent {}", id);
                                        let error_msg = serde_json::json!({
                                            "type": "error",
                                            "code": "DUPLICATE_ID",
                                            "message": format!("Agent ID {} is already active on the server.", id)
                                        });
                                        let _ = tx.send(Message::Text(error_msg.to_string()));
                                        break;
                                    }

                                    agent_id = id.to_string();
                                    state_clone.agent_manager.add_agent(agent_id.clone(), tx.clone()).await;
                                    tracing::info!("Agent {} connected", agent_id);
                                    
                                    // Send initial config push
                                    let _ = notify_agent_config_update(&state_clone, &agent_id).await;
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
            tracing::info!("Agent {} disconnected", agent_id);
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
        method: command_request["method"].as_str().unwrap_or("unknown").to_string(),
        device_id: command_request["device_id"].as_str().map(|s| s.to_string()),
        data: command_request.get("data").cloned(),
    };

    match state.agent_manager.send_command(&agent_id, command).await {
        Ok(_) => (StatusCode::OK, "Command sent").into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

// --- Device Management Handlers ---

pub async fn get_devices(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match sqlx::query_as::<_, Device>("SELECT * FROM devices ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
    {
        Ok(devices) => Json(devices).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_device(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateDeviceRequest>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let port = req.port.unwrap_or(4370);

    match sqlx::query(
        "INSERT INTO devices (id, agent_id, name, ip, port) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(id)
    .bind(&req.agent_id)
    .bind(&req.name)
    .bind(&req.ip)
    .bind(port)
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let _ = notify_agent_config_update(&state, &req.agent_id).await;
            (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_device(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateDeviceRequest>,
) -> impl IntoResponse {
    // Get agent_id first for notification
    let current_device = match sqlx::query_as::<_, Device>("SELECT * FROM devices WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await {
            Ok(d) => d,
            Err(_) => return StatusCode::NOT_FOUND.into_response(),
        };

    let name = req.name.unwrap_or(current_device.name);
    let ip = req.ip.unwrap_or(current_device.ip);
    let port = req.port.unwrap_or(current_device.port);
    let is_enabled = req.is_enabled.unwrap_or(current_device.is_enabled);

    match sqlx::query(
        "UPDATE devices SET name = $1, ip = $2, port = $3, is_enabled = $4 WHERE id = $5"
    )
    .bind(name)
    .bind(ip)
    .bind(port)
    .bind(is_enabled)
    .bind(id)
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let _ = notify_agent_config_update(&state, &current_device.agent_id).await;
            StatusCode::OK.into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_device(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Get agent_id first for notification
    let agent_id: Option<String> = sqlx::query_scalar("SELECT agent_id FROM devices WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    match sqlx::query("DELETE FROM devices WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
    {
        Ok(_) => {
            if let Some(aid) = agent_id {
                let _ = notify_agent_config_update(&state, &aid).await;
            }
            StatusCode::NO_CONTENT.into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn notify_agent_config_update(state: &AppState, agent_id: &str) -> Result<(), String> {
    // Fetch all enabled devices for this agent
    let devices = sqlx::query_as::<_, Device>("SELECT * FROM devices WHERE agent_id = $1 AND is_enabled = true")
        .bind(agent_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let config_msg = serde_json::json!({
        "type": "config_update",
        "devices": devices.into_iter().map(|d| {
            serde_json::json!({
                "device_id": d.id.to_string(), // Dùng UUID làm device_id
                "ip": d.ip,
                "port": d.port
            })
        }).collect::<Vec<_>>()
    });

    let command = AgentCommand {
        id: Uuid::new_v4().to_string(),
        r#type: "config_push".to_string(),
        method: "update_config".to_string(),
        device_id: None,
        data: Some(config_msg),
    };

    state.agent_manager.send_command(agent_id, command).await
        .map_err(|e| e.to_string())
}