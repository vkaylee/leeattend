# Backend Implementation Report: Phase 02 - Paper OT Entry API

**Date:** 2026-01-28 21:27
**Phase:** 02 - Backend API
**Status:** Complete
**Engineer:** backend-engineer

---

## Implementation Summary

Created new API endpoint `POST /workspaces/:ws_id/attendance/enter-paper-ot/:employee_id` for Admin/Owner users to digitize paper OT forms. The implementation reuses existing service patterns and follows established authentication/authorization patterns.

---

## Changes Made

| File | Change | Purpose |
|------|--------|---------|
| `apiserver/src/handlers/attendance.rs` | Added `EnterPaperOtRequest` struct | Request validation structure |
| `apiserver/src/handlers/attendance.rs` | Added `enter_paper_ot` handler | HTTP endpoint handler with RBAC |
| `apiserver/src/services/attendance_summary_service.rs` | Added `enter_paper_ot` method | Business logic for paper form entry |
| `apiserver/src/lib.rs` | Registered route | Wire endpoint to handler |

---

## API Endpoint Details

### Endpoint
```
POST /workspaces/:ws_id/attendance/enter-paper-ot/:employee_id
```

### Request Body
```json
{
  "date": "2026-01-28",
  "claimed_minutes": 120,
  "reason": "Production line overtime - signed paper form #1234",
  "allow_exceed_system_detected": false,
  "paper_form_photo_url": "https://storage.example.com/ot-forms/2026-01-28/emp123.jpg"
}
```

### Response (200 OK)
Returns full `AttendanceDailySummary` object with:
- `ot_status`: Set to `pending_manager` (skips `pending_employee`)
- `entered_via`: Set to `paper_form`
- `claim_entered_by_user_id`: Current user ID
- `claim_entered_at`: Current timestamp
- `paper_form_photo_url`: Optional photo URL

### Error Responses
- `400 BAD_REQUEST`: Invalid input (minutes <= 0, exceeds system limit)
- `403 FORBIDDEN`: Not Admin/Owner
- `404 NOT_FOUND`: Employee not found

---

## Implementation Details

### 1. Request Struct
```rust
#[derive(Deserialize)]
pub struct EnterPaperOtRequest {
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: String,
    #[serde(default)]
    pub allow_exceed_system_detected: bool,
    pub paper_form_photo_url: Option<String>,
}
```

### 2. Handler Flow
1. RBAC: Verify Admin/Owner role
2. Validate employee exists in workspace
3. Validate claimed_minutes > 0
4. Call service method
5. Return result or error

### 3. Service Method
```rust
pub async fn enter_paper_ot(
    pool: &PgPool,
    employee_id: Uuid,
    date: NaiveDate,
    claimed_minutes: i32,
    reason: String,
    entered_by_user_id: Uuid,
    photo_url: Option<String>,
    allow_exceed: bool,
) -> Result<AttendanceDailySummary, String>
```

**Logic:**
1. Fetch existing summary (if any)
2. Validate claimed_minutes <= system_detected_ot_minutes (unless allow_exceed flag)
3. Upsert summary with:
   - `claimed_ot_minutes`
   - `ot_reason`
   - `ot_status = 'pending_manager'`
   - `claim_entered_by_user_id`
   - `claim_entered_at`
   - `paper_form_photo_url`
   - `entered_via = 'paper_form'`
4. Return updated summary

---

## Security Considerations

- RBAC enforced at handler level (Admin/Owner only)
- User ID from JWT token, not request body
- Employee workspace membership validated
- Input validation (minutes > 0)
- Photo URL stored as-is (validation can be added later)

---

## Key Design Decisions

1. **Separate Endpoint**: New endpoint instead of modifying existing `claim-ot` for clearer intent and easier audit trail

2. **Status Flow**: Directly set to `pending_manager` (skips `pending_employee`) since worker consent is captured via paper signature

3. **Validation Logic**: Reused existing validation pattern with optional override flag

4. **Upsert Behavior**: Uses ON CONFLICT to handle both create and update scenarios

5. **Audit Trail**: Captures who entered the data and when for accountability

---

## Testing Notes

To test manually:
```bash
curl -X POST http://localhost:8000/workspaces/{ws_id}/attendance/enter-paper-ot/{employee_id} \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "date": "2026-01-28",
    "claimed_minutes": 120,
    "reason": "Production overtime - paper form #123",
    "allow_exceed_system_detected": false
  }'
```

Expected: 200 OK with AttendanceDailySummary JSON response

---

## Compilation Status

✅ **Success**: Code compiles without errors
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.93s
```

---

## Next Steps

1. Phase 03: Frontend API client functions
2. Phase 04: UI components for paper form entry
3. Phase 05: Photo upload endpoint implementation
4. Integration testing with frontend

---

## Unresolved Questions

None - Implementation complete per plan specifications.
