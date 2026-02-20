//! WebSocket endpoint for streaming report data.
//!
//! Connects clients to Redis Pub/Sub for real-time report chunk delivery.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;

use crate::queue::{ReportSubscriber, StreamMessage};
use crate::AppState;

/// Query parameters for WebSocket connection.
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// Optional bearer token for authentication.
    pub token: Option<String>,
}

/// Middleware that validates JWT token from query params before WebSocket upgrade.
/// This runs before the WebSocketUpgrade extractor, ensuring auth errors return 401.
pub async fn ws_auth_middleware(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let token = match query.token.as_deref() {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, "Missing authentication token").into_response();
        }
    };

    match crate::services::auth_service::AuthService::verify_token(token, &state.jwt_secret) {
        Ok(_) => next.run(request).await,
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response(),
    }
}

/// WebSocket upgrade handler for report streaming.
///
/// Route: GET /ws/reports/:job_id
/// Note: Auth is handled by ws_auth_middleware applied as a layer on this route.
pub async fn report_stream_ws(
    ws: WebSocketUpgrade,
    Path(job_id): Path<Uuid>,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> Response {
    // 1. Authenticate user and extract ID
    let user_id = match query.token.as_deref().and_then(|t| {
        crate::services::auth_service::AuthService::verify_token(t, &state.jwt_secret).ok()
    }) {
        Some(uid) => uid,
        None => return (StatusCode::UNAUTHORIZED, "Invalid or missing token").into_response(),
    };

    // 2. Check if Redis is available
    let redis = match &state.redis {
        Some(redis) => redis.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Streaming not available: Redis not configured",
            )
                .into_response();
        }
    };

    // 3. Verify job ownership
    let queue = crate::queue::ReportQueue::new(redis.clone());
    let metadata = match queue.get_job_metadata(job_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "Job not found").into_response(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Queue error: {e}"),
            )
                .into_response()
        }
    };

    // Check if user belongs to the workspace of this job
    let is_member = match state
        .workspace_repo
        .find_primary_role(metadata.workspace_id, user_id)
        .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            warn!(error = %e, "Failed to verify workspace membership");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Auth check failed").into_response();
        }
    };

    if !is_member {
        warn!(job_id = %job_id, user_id = %user_id, "Unauthorized WS access attempt");
        return (
            StatusCode::FORBIDDEN,
            "Not authorized to access this report stream",
        )
            .into_response();
    }

    info!(job_id = %job_id, user_id = %user_id, "WebSocket upgrade authorized for report stream");

    ws.on_upgrade(move |socket| handle_report_socket(socket, job_id, redis))
}

/// Handle the WebSocket connection for report streaming.
async fn handle_report_socket(socket: WebSocket, job_id: Uuid, redis: crate::queue::RedisPool) {
    let (mut sender, mut receiver) = socket.split();

    // Create Redis subscriber
    let subscriber = match ReportSubscriber::new(&redis, job_id).await {
        Ok(sub) => sub,
        Err(e) => {
            warn!(job_id = %job_id, error = %e, "Failed to subscribe to report stream");
            let error_msg = serde_json::json!({
                "type": "ERROR",
                "message": format!("Failed to subscribe: {e}"),
                "batch_index": null
            });
            let _ = sender.send(Message::Text(error_msg.to_string())).await;
            return;
        }
    };

    info!(job_id = %job_id, "Client connected to report stream");

    // Spawn task to forward Redis messages to WebSocket
    let job_id_clone = job_id;
    let forward_task = tokio::spawn(async move {
        let mut subscriber = subscriber;
        let timeout_duration = Duration::from_secs(600); // 10 minutes

        loop {
            tokio::select! {
                // Receive from Redis
                msg = subscriber.next_message() => {
                    match msg {
                        Some(stream_msg) => {
                            let json = match serde_json::to_string(&stream_msg) {
                                Ok(j) => j,
                                Err(e) => {
                                    warn!(job_id = %job_id_clone, error = %e, "Failed to serialize message");
                                    continue;
                                }
                            };

                            if sender.send(Message::Text(json)).await.is_err() {
                                info!(job_id = %job_id_clone, "Client disconnected");
                                break;
                            }

                            // Close on terminal messages
                            match stream_msg {
                                StreamMessage::Done { .. } => {
                                    info!(job_id = %job_id_clone, "Report complete, closing connection");
                                    let _ = sender.close().await;
                                    break;
                                }
                                StreamMessage::Error { .. } => {
                                    warn!(job_id = %job_id_clone, "Report error, closing connection");
                                    let _ = sender.close().await;
                                    break;
                                }
                                _ => {}
                            }
                        }
                        None => {
                            info!(job_id = %job_id_clone, "Redis subscription closed");
                            break;
                        }
                    }
                }

                // Timeout after 10 minutes
                _ = tokio::time::sleep(timeout_duration) => {
                    warn!(job_id = %job_id_clone, "WebSocket timeout");
                    let timeout_msg = serde_json::json!({
                        "type": "ERROR",
                        "message": "Connection timeout",
                        "batch_index": null
                    });
                    let _ = sender.send(Message::Text(timeout_msg.to_string())).await;
                    let _ = sender.close().await;
                    break;
                }
            }
        }

        // Cleanup subscriber
        if let Err(e) = subscriber.close().await {
            warn!(job_id = %job_id_clone, error = %e, "Failed to close subscriber");
        }
    });

    // Handle incoming messages (pings, close requests)
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Close(_)) => {
                info!(job_id = %job_id, "Client sent close frame");
                break;
            }
            Ok(Message::Ping(_)) => {
                // Pong is handled automatically by axum
            }
            Err(e) => {
                warn!(job_id = %job_id, error = %e, "WebSocket receive error");
                break;
            }
            _ => {}
        }
    }

    // Abort forward task on client disconnect
    forward_task.abort();
    info!(job_id = %job_id, "Report stream connection closed");
}
