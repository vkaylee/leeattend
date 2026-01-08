# Implementation Plan: WebSocket-based Heartbeat Refactoring

## Approach
- **Why**: Using a redundant HTTP request for heartbeat while a WebSocket tunnel is already open is inefficient and prone to path errors (like the current 404).
- **Solution**: 
    - **Agent**: Send a simple JSON `{ "type": "heartbeat", "agent_id": "..." }` through the existing WebSocket connection every 30 seconds.
    - **Server**: Handle the "heartbeat" message type in the WebSocket loop to update `last_seen_at`.
    - **Cleanup**: Remove the unnecessary HTTP heartbeat API and `httpx` dependency.

## Steps

### 1. Backend Implementation (15 min)
- Modify `apiserver/src/handlers.rs`:
    - Add a case for `heartbeat` message type in `handle_socket`.
    - Use `agent_service.update_last_seen` to update the DB.
- Ensure the `agent_id` in the message matches the authenticated `agent_id` of the connection.

### 2. Agent Implementation (15 min)
- Modify `agent/central_client.py`:
    - Update `_heartbeat` loop to use `websocket.send()` with JSON payload.
    - Remove `httpx` usage and `http_url` configuration.

### 3. Cleanup & Refactoring (15 min)
- Remove `POST /agents/heartbeat` route from `apiserver/src/lib.rs`.
- Remove `agent_heartbeat` handler and `AgentId` extractor from `apiserver/src/handlers.rs` and `lib.rs`.
- Remove `httpx` from `agent/requirements.txt`.

### 4. Testing (15 min)
- Verify `last_seen_at` updates in DB when Agent is connected via WS.
- Verify 404 errors disappear from Agent logs.
- Run integration tests.

## Timeline
| Phase | Duration |
|-------|----------|
| Backend | 15 min |
| Agent | 15 min |
| Cleanup | 15 min |
| Testing | 15 min |
| **Total** | **1 hour** |

## Rollback Plan
- Restore HTTP heartbeat endpoint and fix the URL path in the Agent.

## Security Checklist
- [ ] Validate that heartbeat messages are only processed for authenticated sessions.
- [ ] Prevent spoofing by ensuring the `agent_id` in the JSON matches the session context.
