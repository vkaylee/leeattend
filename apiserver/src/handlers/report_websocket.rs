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
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let mut token_opt = query.token.clone();

    // If token is not in query, try to get it from cookie
    if token_opt.is_none() {
        if let Some(cookie_header) = request.headers().get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some(token) = cookie.strip_prefix("access_token=") {
                        token_opt = Some(token.to_string());
                        break;
                    }
                }
            }
        }
    }

    let Some(token) = token_opt.as_deref() else {
        return (StatusCode::UNAUTHORIZED, "Missing authentication token").into_response();
    };

    match crate::services::auth_service::AuthService::verify_token(token, &state.core.jwt_secret) {
        Ok(user_id) => {
            request.extensions_mut().insert(user_id);
            next.run(request).await
        }
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
    axum::Extension(user_id): axum::Extension<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Response {
    // 2. Check if Redis is available
    let redis = match &state.infra.redis {
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
    let queue = crate::queue::JobQueue::new(redis.clone());
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
        .core
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
#[allow(clippy::too_many_lines)]
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

    // RACE CONDITION FIX: Check if job already completed before we subscribed.
    // Redis Pub/Sub has no history — if DONE was published before we subscribed,
    // we'll never receive it. Check metadata to catch this case.
    //
    // IMPORTANT: Subscriber is created BEFORE this check, so if the job completes
    // between subscribe and this check, we catch it via either path.
    let queue = crate::queue::JobQueue::new(redis.clone());
    if let Ok(Some(meta)) = queue.get_job_metadata(job_id).await {
        match meta.status {
            crate::queue::JobStatus::Completed => {
                info!(job_id = %job_id, "Job already completed before WS subscribed (race condition handled)");
                let done_msg = StreamMessage::Done {
                    total_batches: meta.total_batches,
                    total_records: 0,
                };
                let json = serde_json::to_string(&done_msg).unwrap_or_else(|e| {
                    warn!(job_id = %job_id, error = %e, "Failed to serialize Done message");
                    r#"{"type":"DONE"}"#.to_string()
                });
                let _ = sender.send(Message::Text(json)).await;
                let _ = sender.close().await;
                let _ = subscriber.close().await;
                return;
            }
            crate::queue::JobStatus::Failed => {
                info!(job_id = %job_id, "Job already failed before WS subscribed (race condition handled)");
                let err_msg = StreamMessage::Error {
                    message: meta.error_message.unwrap_or_else(|| "Job failed".into()),
                    batch_index: None,
                };
                let json = serde_json::to_string(&err_msg).unwrap_or_else(|e| {
                    warn!(job_id = %job_id, error = %e, "Failed to serialize Error message");
                    r#"{"type":"ERROR","message":"Job failed"}"#.to_string()
                });
                let _ = sender.send(Message::Text(json)).await;
                let _ = sender.close().await;
                let _ = subscriber.close().await;
                return;
            }
            _ => {} // Job still running, continue with normal Pub/Sub flow
        }
    }

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
                    if let Some(stream_msg) = msg {
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
                            StreamMessage::Chunk { .. } => {}
                        }
                    } else {
                        info!(job_id = %job_id_clone, "Redis subscription closed");
                        break;
                    }
                }

                // Timeout after 10 minutes
                () = tokio::time::sleep(timeout_duration) => {
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

#[cfg(test)]
mod tests {
    use crate::queue::StreamMessage;

    /// The hardcoded fallback JSON for Done must be valid JSON parseable as a generic object.
    #[test]
    fn test_done_fallback_json_is_valid() {
        let fallback = r#"{"type":"DONE"}"#;
        let parsed: serde_json::Value =
            serde_json::from_str(fallback).expect("Done fallback JSON must be valid");
        assert_eq!(parsed["type"], "DONE");
    }

    /// The hardcoded fallback JSON for Error must be valid JSON with type and message fields.
    #[test]
    fn test_error_fallback_json_is_valid() {
        let fallback = r#"{"type":"ERROR","message":"Job failed"}"#;
        let parsed: serde_json::Value =
            serde_json::from_str(fallback).expect("Error fallback JSON must be valid");
        assert_eq!(parsed["type"], "ERROR");
        assert_eq!(parsed["message"], "Job failed");
    }

    /// Normal StreamMessage::Done serialization should always succeed.
    #[test]
    fn test_stream_message_done_serializes() {
        let msg = StreamMessage::Done {
            total_batches: 5,
            total_records: 0,
        };
        let json = serde_json::to_string(&msg);
        assert!(json.is_ok(), "Done serialization must not fail");
        assert!(json
            .expect("Done serialization must not fail")
            .contains("\"type\":\"DONE\""));
    }

    /// Normal StreamMessage::Error serialization should always succeed.
    #[test]
    fn test_stream_message_error_serializes() {
        let msg = StreamMessage::Error {
            message: "Job failed".to_string(),
            batch_index: None,
        };
        let json = serde_json::to_string(&msg);
        assert!(json.is_ok(), "Error serialization must not fail");
        assert!(json
            .expect("Error serialization must not fail")
            .contains("\"type\":\"ERROR\""));
    }
}
