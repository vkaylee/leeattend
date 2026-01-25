# Phase 4: Backend API Handlers

## Context Links
- Plan: `./plan.md`
- Phase 3: `./phase-03-backend-services.md`
- leaves handler: `apiserver/src/handlers/leaves.rs`
- leave_types handler: `apiserver/src/handlers/leave_types.rs`

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 1h
- **Dependencies**: Phase 3 (services must be updated first)

Update API handlers to accept leave_type_id in requests and return enriched responses with leave type metadata.

## Key Insights
- Handler functions mostly pass through to services
- Response types already flow from repositories
- Need to add endpoints for fetching leave types (already exists)

## Requirements

### Functional
- Update request parsing to use leave_type_id
- Ensure responses include leave type metadata
- Add helper endpoint to get leave type by code (for backward compat during transition)

### Non-Functional
- Maintain API backward compatibility where possible
- Clear error messages for invalid leave_type_id

## Related Code Files

### Files to Modify
- `apiserver/src/handlers/leaves.rs`
- `apiserver/src/handlers/leave_types.rs` (minor: add get_by_code endpoint)

## Implementation Steps

### Step 1: Update leaves.rs Handlers

The handlers mostly just call services, so changes are minimal. Ensure request types match:

```rust
// No changes needed for most handlers as they use updated models.
// The SubmitLeaveRequest now has leave_type_id: Uuid

// Optionally add response enrichment for leave requests list:
pub async fn get_workspace_leave_requests(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<LeaveStatusQuery>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // Get requests with leave type info
    match get_leave_requests_with_type(&state.db, ctx.workspace_id, query.status).await {
        Ok(requests) => Json(requests).into_response(),
        Err(status) => status.into_response(),
    }
}

// Helper function to enrich leave requests with type info
async fn get_leave_requests_with_type(
    pool: &PgPool,
    workspace_id: Uuid,
    status: Option<LeaveStatus>,
) -> Result<Vec<LeaveRequestWithType>, StatusCode> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        r#"
        SELECT
            lr.id, lr.employee_id, lr.leave_type_id, lr.start_date, lr.end_date,
            lr.reason, lr.status, lr.requested_by_user_id, lr.approved_by_user_id,
            lr.created_at, lr.updated_at,
            lt.code as leave_type_code, lt.name as leave_type_name, lt.is_paid
        FROM leave_requests lr
        JOIN leave_types lt ON lr.leave_type_id = lt.id
        JOIN employees e ON lr.employee_id = e.id
        WHERE e.workspace_id =
        "#
    );
    query_builder.push_bind(workspace_id);

    if let Some(s) = status {
        query_builder.push(" AND lr.status = ");
        query_builder.push_bind(s);
    }

    query_builder.push(" ORDER BY lr.created_at DESC");

    let rows = query_builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut results = Vec::new();
    for row in rows {
        use sqlx::Row;
        results.push(LeaveRequestWithType {
            request: LeaveRequest {
                id: row.get("id"),
                employee_id: row.get("employee_id"),
                leave_type_id: row.get("leave_type_id"),
                start_date: row.get("start_date"),
                end_date: row.get("end_date"),
                reason: row.get("reason"),
                status: row.get("status"),
                requested_by_user_id: row.get("requested_by_user_id"),
                approved_by_user_id: row.get("approved_by_user_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            },
            leave_type_code: row.get("leave_type_code"),
            leave_type_name: row.get("leave_type_name"),
            is_paid: row.get("is_paid"),
        });
    }

    Ok(results)
}
```

### Step 2: Add Helper Endpoint in leave_types.rs

Add endpoint to lookup leave type by code (helps frontend transition):

```rust
pub async fn get_leave_type_by_code(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, code)): Path<(Uuid, String)>,
    _ctx: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as!(
        LeaveType,
        r#"
        SELECT id, workspace_id, code, name, is_paid,
               policy_config as "policy_config: sqlx::types::Json<LeaveTypeConfig>",
               created_at
        FROM leave_types
        WHERE workspace_id = $1 AND code = $2
        "#,
        workspace_id,
        code
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(lt)) => (StatusCode::OK, Json(lt)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Leave type not found").into_response(),
        Err(e) => {
            eprintln!("Error fetching leave type: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

// Update router:
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_leave_types))
        .route("/:id/config", put(update_leave_type_config))
        .route("/code/:code", get(get_leave_type_by_code))  // New
}
```

### Step 3: Update lib.rs Routes (if needed)

Verify routes are properly mounted. Current routes likely already work since leave_types handler is mounted.

## Todo List
- [ ] Update leaves.rs to use enriched response types
- [ ] Add get_leave_type_by_code endpoint
- [ ] Update router in leave_types.rs
- [ ] Verify route mounting in lib.rs
- [ ] Run `cargo check` to verify compilation
- [ ] Test API endpoints manually

## Success Criteria
- API returns leave type metadata in responses
- leave_type_id accepted in POST requests
- Helper endpoint returns leave type by code

## Risk Assessment
- **Breaking Change**: API response shape changes
- **Mitigation**: Include both `leave_type_code` (for display) and `leave_type_id` (for references)

## Next Steps
Proceed to Phase 5 (Frontend Updates).
