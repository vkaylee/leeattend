# Phase 3: WebSocket Endpoint

**Priority**: P0
**Status**: Complete
**Estimated**: Medium complexity

---

## Context Links
- [Plan Overview](./plan.md)
- [Phase 2: Redis Pub/Sub](./phase-02-redis-pubsub.md)

---

## Overview

Create WebSocket endpoint for streaming report data from Redis Pub/Sub to browser clients.

---

## Key Insights

- Axum has WebSocket support via `axum::extract::ws`
- Need dedicated connection per job_id (not shared broadcast)
- Redis subscriber runs in background task, forwards to WS sink
- Connection lifecycle: Subscribe → Forward → Close on DONE/ERROR

---

## Requirements

### Functional
- Route: `GET /ws/reports/{job_id}`
- Authenticate via query param token or cookie
- Subscribe to `report:stream:{job_id}` Redis channel
- Forward all messages to WebSocket client
- Close connection on DONE or ERROR message
- Timeout after 10 minutes if no activity

### Non-Functional
- Latency < 50ms from Redis PUBLISH to WS delivery
- Support 100+ concurrent WebSocket connections
- Graceful cleanup on client disconnect

---

## Related Code Files

### Create
- `apiserver/src/handlers/report_websocket.rs` - WS handler

### Modify
- `apiserver/src/handlers/mod.rs` - Export handler
- `apiserver/src/routes.rs` - Add WS route

---

## Implementation Steps

### Step 1: Create WebSocket Handler

```rust
// apiserver/src/handlers/report_websocket.rs

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;

use crate::{
    auth::verify_token,
    queue::pubsub::{ReportSubscriber, StreamMessage},
    AppState,
};

#[derive(Debug, serde::Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

/// WebSocket upgrade handler for report streaming
pub async fn report_stream_ws(
    ws: WebSocketUpgrade,
    Path(job_id): Path<Uuid>,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    // Verify token (from query param or will check cookie in socket)
    if let Some(token) = &query.token {
        if verify_token(&state.pool, token).await.is_err() {
            return Response::builder()
                .status(401)
                .body("Unauthorized".into())
                .unwrap();
        }
    }

    ws.on_upgrade(move |socket| handle_report_socket(socket, job_id, state))
}

async fn handle_report_socket(socket: WebSocket, job_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Create Redis subscriber
    let subscriber = match ReportSubscriber::new(&state.redis_client, job_id).await {
        Ok(sub) => sub,
        Err(e) => {
            let _ = sender
                .send(Message::Text(format!(
                    r#"{{"type":"ERROR","message":"Failed to subscribe: {}"}}"#,
                    e
                )))
                .await;
            return;
        }
    };

    // Spawn task to forward Redis messages to WebSocket
    let forward_task = tokio::spawn(async move {
        let mut subscriber = subscriber;

        loop {
            tokio::select! {
                // Receive from Redis
                msg = subscriber.next_message() => {
                    match msg {
                        Some(stream_msg) => {
                            let json = serde_json::to_string(&stream_msg).unwrap();
                            if sender.send(Message::Text(json)).await.is_err() {
                                break; // Client disconnected
                            }

                            // Close on terminal messages
                            match stream_msg {
                                StreamMessage::Done { .. } | StreamMessage::Error { .. } => {
                                    let _ = sender.close().await;
                                    break;
                                }
                                _ => {}
                            }
                        }
                        None => break, // Redis connection closed
                    }
                }

                // Timeout after 10 minutes
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(600)) => {
                    let _ = sender.send(Message::Text(
                        r#"{"type":"ERROR","message":"Timeout"}"#.to_string()
                    )).await;
                    break;
                }
            }
        }

        let _ = subscriber.close().await;
    });

    // Handle incoming messages (pings, close requests)
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(data)) => {
                // Pong is handled automatically by axum
            }
            Err(_) => break,
            _ => {}
        }
    }

    forward_task.abort();
}
```

### Step 2: Add Route

```rust
// apiserver/src/routes.rs

use crate::handlers::report_websocket::report_stream_ws;

// In router setup
.route("/ws/reports/:job_id", get(report_stream_ws))
```

### Step 3: Export Handler Module

```rust
// apiserver/src/handlers/mod.rs

pub mod report_websocket;
pub use report_websocket::report_stream_ws;
```

### Step 4: Add Token Query Auth (Optional Enhancement)

```rust
// For clients that can't set cookies on WebSocket
// Token can be passed as query param: /ws/reports/{id}?token=xxx

// In verify_token, support both session cookie and bearer token
pub async fn verify_ws_auth(
    pool: &PgPool,
    query_token: Option<&str>,
    cookie_header: Option<&str>,
) -> Result<UserId, AuthError> {
    // Try query token first
    if let Some(token) = query_token {
        return verify_bearer_token(pool, token).await;
    }

    // Fall back to session cookie
    if let Some(cookie) = cookie_header {
        return verify_session_cookie(pool, cookie).await;
    }

    Err(AuthError::Unauthorized)
}
```

---

## Todo List

- [x] Create report_websocket.rs handler file
- [x] Implement WebSocket upgrade with auth check
- [x] Implement Redis subscriber → WebSocket forwarding
- [x] Add timeout handling (10 min max)
- [x] Handle client disconnect cleanup
- [x] Add route to router
- [x] Test with wscat or browser DevTools
- [x] Add connection metrics logging

---

## Success Criteria

- [ ] `wscat -c ws://localhost:3000/ws/reports/{job_id}` connects
- [ ] Messages from Redis PUBLISH appear in WebSocket client
- [ ] Connection closes after DONE message
- [ ] Unauthorized requests rejected with 401
- [ ] No resource leaks on disconnect

---

## Testing Commands

```bash
# Terminal 1: Connect WebSocket
wscat -c "ws://localhost:3000/ws/reports/test-job-id?token=xxx"

# Terminal 2: Publish test message
redis-cli PUBLISH "report:stream:test-job-id" '{"type":"CHUNK","data":[],"batch_index":0,"total_batches":1,"progress":100}'

# Terminal 3: Trigger real export
curl -X POST http://localhost:3000/api/workspaces/{ws_id}/reports/export \
  -H "Authorization: Bearer xxx" \
  -d '{"type":"summary"}'
```

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Connection leak | High | Abort forward task on disconnect |
| Auth bypass | Critical | Verify token before upgrade |
| Memory pressure | Medium | Limit concurrent connections |

---

## Security Considerations

- Validate job_id belongs to user's workspace
- Token verification before WebSocket upgrade
- Rate limit WebSocket connections per user
