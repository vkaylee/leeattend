# Implementation Plan: Heartbeat Mechanism for Device Status

## Approach
- **Why**: Current "Online/Offline" status is inaccurate because it relies on explicit WebSocket connect/disconnect events, which fail during network jitters or improper shutdowns.
- **Solution**: Implement a time-based Heartbeat mechanism:
    - **Agent**: Sends a "ping" to the Server every 30 seconds.
    - **Server**: Updates `last_seen_at` in the `authorized_agents` table.
    - **Frontend**: Calculates "Online" status if `now - last_seen_at < 90s`.

## Steps

### 1. Database Migration (5 min)
Create a new migration to add `last_seen_at` to `authorized_agents`.
```sql
ALTER TABLE authorized_agents ADD COLUMN last_seen_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;
```

### 2. Backend Implementation (Rust) (20 min)
- Modify `apiserver/src/models.rs` to include `last_seen_at` in `AuthorizedAgent`.
- Update `apiserver/src/repositories/agent_repository.rs` with `update_last_seen`.
- Add a heartbeat handler in `apiserver/src/handlers.rs`.
- Route: `POST /api/agents/heartbeat` (Authenticated via Agent Token).

### 3. Agent Implementation (Python) (15 min)
- Update `agent/central_client.py` to include a background heartbeat loop.
- Ensure the heartbeat is sent only when the agent is authenticated.

### 4. Frontend Integration (React) (15 min)
- Update `webdashboard/src/lib/api.ts` to include `last_seen_at` in Agent types.
- Modify `webdashboard/src/pages/AgentsPage.tsx` or status indicators to use time-based online check.

### 5. Testing (15 min)
- Verify `last_seen_at` updates in DB when agent is running.
- Verify status changes to offline in UI when agent is stopped.

## Timeline
| Phase | Duration |
|-------|----------|
| Database Migration | 5 min |
| Backend (Rust) | 20 min |
| Agent (Python) | 15 min |
| Frontend (React) | 15 min |
| Testing | 15 min |
| **Total** | **1 hour 10 min** |

## Rollback Plan
- Run down migration to remove the column.
- Revert code changes in apiserver and agent.

## Security Checklist
- [ ] Agent Token validation for heartbeat endpoint.
- [ ] Rate limit heartbeat processing to prevent DB thrashing.
- [ ] Ensure `last_seen_at` is updated atomically.
