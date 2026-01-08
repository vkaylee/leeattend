# Implementation Plan: Dev Fast Login

Implement a one-click "Fast Login" feature for the development environment to automatically set up authentication headers (`X-User-Id`, `X-Workspace-Id`) by discovering the first available user and workspace in the database.

## Approach
- **Backend**: Add a debug-only endpoint `GET /api/dev/super-user` that queries the database for the first user and their workspace membership.
- **Frontend**: Add a fixed button in the Sidebar (visible only in `import.meta.env.DEV`) that calls this endpoint and persists the results to `localStorage`.

## Steps

### 1. Backend: Implement Dev Handler (10 min)
- File: `apiserver/src/handlers.rs`
- Add a new handler `get_dev_super_user` wrapped in `#[cfg(debug_assertions)]`.
```rust
#[cfg(debug_assertions)]
pub async fn get_dev_super_user(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Logic: 
    // 1. SELECT * FROM users LIMIT 1;
    // 2. SELECT * FROM workspace_members WHERE user_id = ... LIMIT 1;
    // 3. Return { user_id, workspace_id }
}
```

### 2. Backend: Register Route (5 min)
- File: `apiserver/src/lib.rs`
- Add the route to `create_app` inside a conditional block.
```rust
#[cfg(debug_assertions)]
let app = app.route("/api/dev/super-user", get(handlers::get_dev_super_user));
```

### 3. Frontend: API Client Update (5 min)
- File: `webdashboard/src/lib/api.ts`
- Add `fetchDevSuperUser` function.

### 4. Frontend: UI Integration (15 min)
- File: `webdashboard/src/App.tsx` (Inside `Layout` component)
- Add a button at the bottom of the sidebar or near the Workspace Switcher.
- Use `import.meta.env.DEV` to gate visibility.
- Logic: Fetch IDs -> `setCurrentUserId`/`setCurrentWorkspaceId` -> `window.location.reload()`.

### 5. Verification (10 min)
- Restart backend in debug mode.
- Verify the button appears in Vite dev server.
- Click the button and ensure the dashboard loads with data.

## Timeline
| Phase | Duration |
|-------|----------|
| Backend Implementation | 15 min |
| Frontend Integration | 20 min |
| Verification | 10 min |
| **Total** | **45 min** |

## Rollback Plan
- Remove the `#[cfg(debug_assertions)]` blocks and the frontend button component.

## Security Checklist
- [x] Debug-only: Endpoint strictly wrapped in `debug_assertions`.
- [x] Environment-gate: Frontend button strictly wrapped in `import.meta.env.DEV`.
- [ ] Input validation: N/A (GET request).
- [x] Error handling: Handle cases where DB is empty.
