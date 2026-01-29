# Phase 02: Backend API

## Context Links
- [Phase 01 - Schema](/home/elt1541/leeattend/plans/260128-2113-ot-paper-form-digitization/phase-01-database-schema.md)
- [Existing Handler](/home/elt1541/leeattend/apiserver/src/handlers/attendance.rs)
- [Existing Service](/home/elt1541/leeattend/apiserver/src/services/attendance_summary_service.rs)

## Overview
- **Priority**: P1
- **Status**: Pending
- **Effort**: 4 hours

Create new API endpoint for department heads/assistants to enter paper OT claims.

## Key Insights

From existing code:
- `claim_ot` handler already allows Admin/Owner to claim for any employee
- Status transition: `claim_ot` sets `ot_status = pending_manager`
- RBAC uses `MemberRole::Admin | MemberRole::Owner` check

Key design decision:
- **New endpoint** preferred over modifying existing (clearer intent, easier audit)
- Route: `POST /workspaces/:ws_id/attendance/enter-paper-ot/:employee_id`
- Skips `pending_employee` state (worker consent via paper signature)

## Requirements

### Functional
- Only Admin/Owner can access endpoint
- Validate claimed_minutes > 0
- Validate claimed_minutes <= system_detected_ot_minutes (if exists) OR allow override with flag
- Store entry metadata (who, when, via)
- Optional photo upload (multipart form or separate upload endpoint)
- Return created/updated summary

### Non-Functional
- Consistent error messages with existing endpoints
- Idempotent (upsert behavior)

## API Design

### Endpoint: Enter Paper OT Claim

```
POST /workspaces/:ws_id/attendance/enter-paper-ot/:employee_id
```

**Headers:**
```
Authorization: Bearer <token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "date": "2026-01-28",
  "claimed_minutes": 120,
  "reason": "Production line overtime - signed paper form #1234",
  "allow_exceed_system_detected": false,
  "paper_form_photo_url": "https://storage.example.com/ot-forms/2026-01-28/emp123.jpg"
}
```

**Response (200 OK):**
```json
{
  "id": "uuid",
  "employee_id": "uuid",
  "date": "2026-01-28",
  "claimed_ot_minutes": 120,
  "system_detected_ot_minutes": 90,
  "ot_status": "pending_manager",
  "entered_via": "paper_form",
  "claim_entered_by_user_id": "uuid",
  "claim_entered_at": "2026-01-28T14:30:00Z"
}
```

**Error Responses:**
- `400 Bad Request` - Invalid input (minutes <= 0, exceeds limit)
- `403 Forbidden` - Not Admin/Owner
- `404 Not Found` - Employee not found

### Endpoint: Upload Paper Form Photo (Optional)

```
POST /workspaces/:ws_id/attendance/upload-paper-form
```

**Request:** Multipart form with `file` field

**Response:**
```json
{
  "url": "/uploads/ot-forms/2026-01-28/uuid.jpg"
}
```

## Implementation

### Request/Response Structs

```rust
// In handlers/attendance.rs

#[derive(Deserialize)]
pub struct EnterPaperOtRequest {
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: String, // Required for paper forms
    #[serde(default)]
    pub allow_exceed_system_detected: bool,
    pub paper_form_photo_url: Option<String>,
}

#[derive(Serialize)]
pub struct EnterPaperOtResponse {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub date: NaiveDate,
    pub claimed_ot_minutes: i32,
    pub system_detected_ot_minutes: i32,
    pub ot_status: OtStatus,
    pub entered_via: String,
    pub claim_entered_by_user_id: Uuid,
    pub claim_entered_at: DateTime<Utc>,
}
```

### Handler Implementation

```rust
pub async fn enter_paper_ot(
    State(state): State<Arc<AppState>>,
    Path((ws_id, employee_id)): Path<(Uuid, Uuid)>,
    MemberContext { user_id, .. }: MemberContext,
    Json(req): Json<EnterPaperOtRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // 1. RBAC: Admin/Owner only
    let role = SqlxWorkspaceRepository { pool: state.db.clone() }
        .find_primary_role(ws_id, user_id)
        .await
        .unwrap_or(None);

    if !matches!(role, Some(MemberRole::Admin) | Some(MemberRole::Owner)) {
        return Err(StatusCode::FORBIDDEN);
    }

    // 2. Validate employee exists in workspace
    let employee = employee_repository::get_employee_by_id(&state.db, employee_id, ws_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 3. Validate minutes
    if req.claimed_minutes <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 4. Call service
    let result = AttendanceSummaryService::enter_paper_ot(
        &state.db,
        employee_id,
        req.date,
        req.claimed_minutes,
        req.reason,
        user_id,
        req.paper_form_photo_url,
        req.allow_exceed_system_detected,
    )
    .await
    .map_err(|e| {
        error!("Enter Paper OT Error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(Json(result))
}
```

### Service Implementation

```rust
// In services/attendance_summary_service.rs

impl AttendanceSummaryService {
    pub async fn enter_paper_ot(
        pool: &PgPool,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: String,
        entered_by_user_id: Uuid,
        photo_url: Option<String>,
        allow_exceed: bool,
    ) -> Result<AttendanceDailySummary, String> {
        // 1. Check existing summary
        let existing = sqlx::query_as!(
            AttendanceDailySummary,
            "SELECT * FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
            employee_id, date
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        // 2. Validate against system detected (if not allowing exceed)
        if !allow_exceed {
            if let Some(ref ex) = existing {
                if claimed_minutes > ex.system_detected_ot_minutes {
                    return Err(format!(
                        "Claimed {} exceeds system detected {}",
                        claimed_minutes, ex.system_detected_ot_minutes
                    ));
                }
            }
        }

        let now = Utc::now();

        // 3. Upsert with paper form metadata
        let result = sqlx::query_as!(
            AttendanceDailySummary,
            r#"
            INSERT INTO attendance_daily_summaries (
                workspace_id, employee_id, date,
                claimed_ot_minutes, ot_reason, ot_status,
                claim_entered_by_user_id, claim_entered_at,
                paper_form_photo_url, entered_via
            )
            SELECT
                e.workspace_id, $1, $2,
                $3, $4, 'pending_manager',
                $5, $6, $7, 'paper_form'
            FROM employees e WHERE e.id = $1
            ON CONFLICT (employee_id, date) DO UPDATE SET
                claimed_ot_minutes = EXCLUDED.claimed_ot_minutes,
                ot_reason = EXCLUDED.ot_reason,
                ot_status = 'pending_manager',
                claim_entered_by_user_id = EXCLUDED.claim_entered_by_user_id,
                claim_entered_at = EXCLUDED.claim_entered_at,
                paper_form_photo_url = EXCLUDED.paper_form_photo_url,
                entered_via = 'paper_form',
                updated_at = CURRENT_TIMESTAMP
            RETURNING *
            "#,
            employee_id, date, claimed_minutes, reason,
            entered_by_user_id, now, photo_url
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result)
    }
}
```

### Route Registration

```rust
// In lib.rs router setup

.route("/workspaces/:ws_id/attendance/enter-paper-ot/:employee_id",
    post(attendance::enter_paper_ot))
```

## Related Code Files

### To Modify
- `/home/elt1541/leeattend/apiserver/src/handlers/attendance.rs` - Add handler
- `/home/elt1541/leeattend/apiserver/src/services/attendance_summary_service.rs` - Add service method
- `/home/elt1541/leeattend/apiserver/src/lib.rs` - Register route

### To Create
- None (all in existing files)

## Implementation Steps

1. [ ] Add `EnterPaperOtRequest` and `EnterPaperOtResponse` structs to `attendance.rs`
2. [ ] Implement `enter_paper_ot` handler in `attendance.rs`
3. [ ] Implement `enter_paper_ot` service method in `attendance_summary_service.rs`
4. [ ] Register route in `lib.rs`
5. [ ] Update all `SELECT *` queries to include new columns
6. [ ] Test with curl/httpie

## Todo List

- [ ] Write request/response structs
- [ ] Implement handler with RBAC
- [ ] Implement service with upsert logic
- [ ] Register route
- [ ] Manual API testing
- [ ] Handle photo upload (optional, Phase 5)

## Success Criteria

- [ ] `POST /enter-paper-ot/:employee_id` returns 200 for Admin
- [ ] `POST /enter-paper-ot/:employee_id` returns 403 for non-Admin
- [ ] Summary created with `entered_via = 'paper_form'`
- [ ] Summary shows `claim_entered_by_user_id` populated
- [ ] Validation rejects claimed_minutes > system_detected (unless flag)

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| SQL injection | High | Use parameterized queries (sqlx) |
| Unauthorized access | High | RBAC check at handler level |
| Data loss on upsert | Medium | Only update claim fields, preserve system fields |

## Security Considerations

- RBAC enforced at handler level
- User ID from JWT token, not request body
- Photo URL validation (only allow internal storage URLs later)

## Next Steps

After this phase:
- Phase 03 needs API client functions in frontend
- Phase 04 needs response type for display
