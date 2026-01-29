# Scout Report: Admin-Initiated OT Claims Feature

## Exploration Scope
- Target: Existing OT claim workflow patterns, database schema, API endpoints, frontend components
- Boundaries: `/home/elt1541/leeattend` (apiserver, webdashboard)
- Date: 2026-01-28
- Purpose: Validate architectural compatibility with admin-initiated OT claims (Approach A: Draft-and-Notify)

## Database Schema Patterns

### Table: attendance_daily_summaries
**Location**: `/home/elt1541/leeattend/apiserver/migrations/20260128000000_create_attendance_daily_summaries.sql`

**Schema**:
```sql
CREATE TABLE attendance_daily_summaries (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    employee_id UUID NOT NULL REFERENCES employees(id),
    date DATE NOT NULL,
    
    -- Calculated values (System of Record)
    actual_work_minutes INTEGER NOT NULL DEFAULT 0,
    system_detected_ot_minutes INTEGER NOT NULL DEFAULT 0,
    
    -- Workflow values (User input)
    claimed_ot_minutes INTEGER NOT NULL DEFAULT 0,
    approved_ot_minutes INTEGER NOT NULL DEFAULT 0,
    
    ot_status ot_status_type NOT NULL DEFAULT 'none',
    ot_reason TEXT,
    manager_note TEXT,
    
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    
    UNIQUE(employee_id, date)
);
```

### OT Status Enum Values
**Location**: `/home/elt1541/leeattend/apiserver/migrations/20260128000000_create_attendance_daily_summaries.sql:4`

```sql
CREATE TYPE ot_status_type AS ENUM (
    'none',                  -- No OT detected
    'pending_employee',      -- System detected, awaiting employee claim
    'pending_manager',       -- Employee claimed, awaiting manager approval
    'approved',              -- Manager approved
    'rejected'               -- Manager rejected
);
```

**Must Follow**: Status transitions enforce workflow order (see Business Logic section).

## API Endpoint Patterns

### Employee Claim Endpoint
**Location**: `/home/elt1541/leeattend/apiserver/src/handlers/attendance.rs:674`

**Route**: `POST /workspaces/:ws_id/attendance/claim-ot/:employee_id`

**Request Structure**:
```rust
struct ClaimOtRequest {
    date: NaiveDate,
    claimed_minutes: i32,
    reason: Option<String>
}
```

**Authorization**:
- Employee can claim for themselves
- Admin/Owner can claim for any employee

**Service Call**: `AttendanceSummaryService::claim_ot()` (line 705)

### Manager Approval Endpoint
**Location**: `/home/elt1541/leeattend/apiserver/src/handlers/attendance.rs:729`

**Route**: `POST /workspaces/:ws_id/attendance/approve-ot/:employee_id`

**Request Structure**:
```rust
struct ApproveOtRequest {
    date: NaiveDate,
    approved_minutes: i32,
    manager_note: Option<String>,
    status: String  // "approved" | "rejected"
}
```

**Authorization**: Admin/Owner only (line 739)

**Service Call**: `AttendanceSummaryService::approve_ot()` (line 749)

### Summary Query Endpoint
**Location**: `/home/elt1541/leeattend/apiserver/src/handlers/attendance.rs:773`

**Route**: `GET /workspaces/:ws_id/attendance/summaries`

**Query Parameters**:
```rust
struct SummaryQuery {
    start_date: NaiveDate,
    end_date: NaiveDate,
    employee_id: Option<Uuid>
}
```

**Authorization Logic**:
- Admin/Owner: Can query all employees or specific employee
- Non-admin: Can only query their own records

## Business Logic Patterns

### Status Transition Flow
**Location**: `/home/elt1541/leeattend/apiserver/src/services/attendance_summary_service.rs`

**Current Flow**:
1. System detects OT → status becomes `pending_employee` (line 146, 172)
2. Employee claims → status becomes `pending_manager` (line 204)
3. Manager approves/rejects → status becomes `approved` or `rejected` (line 237)

**Validation Rules** (from `/home/elt1541/leeattend/webdashboard/src/components/ess/ClaimOtDialog.tsx:38-46`):
- Claimed minutes must be > 0
- Claimed minutes cannot exceed `system_detected_ot_minutes`
- Reason is required

**Recalculation Logic** (`attendance_summary_service.rs:19`):
- `recalculate_daily_summary()` computes `actual_work_minutes` and `system_detected_ot_minutes`
- Upserts into `attendance_daily_summaries` table
- Auto-transitions status to `pending_employee` if OT detected and status was `none`

### Permission Patterns
**Admin Authorization Pattern** (attendance.rs:689, 736):
```rust
let role = SqlxWorkspaceRepository::find_primary_role(ws_id, user_id).await;
if !matches!(role, Some(MemberRole::Admin) | Some(MemberRole::Owner)) {
    return Err(StatusCode::FORBIDDEN);
}
```

**Must Follow**: Use `MemberRole` enum for RBAC checks.

## Frontend Component Patterns

### Employee OT Claim Dialog
**Location**: `/home/elt1541/leeattend/webdashboard/src/components/ess/ClaimOtDialog.tsx`

**UI Features**:
- Displays `system_detected_ot_minutes` (read-only warning)
- Input field for `claimed_minutes` (number, max = system detected)
- Textarea for `reason` (required)
- Validation: minutes > 0, minutes <= system detected

**API Integration Hook**: `useSubmitOtClaim()` from `use-ess.ts:265`

### Manager Approval Page
**Location**: `/home/elt1541/leeattend/webdashboard/src/pages/approvals/OtApprovalsPage.tsx`

**Features**:
- Displays pending claims filtered by `ot_status === "pending_manager"` (line 27)
- Shows: date, employee_id, system_detected_ot_minutes, claimed_ot_minutes, ot_reason
- Actions: Approve (full claimed amount) or Reject (0 minutes)
- Month/year picker for filtering

**API Integration Hook**: `useAttendanceSummaries()` and `useApproveOt()` from `use-attendance.ts`

### ESS Attendance Page
**Location**: `/home/elt1541/leeattend/webdashboard/src/pages/ess/MyAttendancePage.tsx`

**Integration Point**: Uses `useMyAttendanceSummaries()` hook to fetch daily summaries (line 43)

**Status Display Pattern**: `getSummary(date)` helper function (line 48)

## Integration Points for Admin-Initiated Claims

### 1. Database Level (Compatible)
**Current constraint**: UNIQUE(employee_id, date) on `attendance_daily_summaries`
- Admin-initiated claim would **update** existing row (if present) or **insert** new row
- No schema changes required

**Proposed Flow**:
1. Admin creates draft → Insert with `claimed_ot_minutes > 0`, `ot_status = 'pending_employee'`, `manager_note = 'Admin draft'`
2. System detected would populate `system_detected_ot_minutes` field separately (via recalculation service)

### 2. API Level (Compatible with Extension)
**Existing Authorization**: Admin can already use `claim-ot` endpoint for any employee (line 692-693)

**Proposed New Endpoint** (optional for clearer semantics):
```
POST /workspaces/:ws_id/attendance/draft-ot/:employee_id
```
- Same payload as `ClaimOtRequest`
- Status transition: `none` → `pending_employee` (with admin note)
- Notification trigger point

**Alternative**: Reuse `claim-ot` endpoint with role detection in service layer:
```rust
if caller_is_admin && !is_self_claim {
    ot_status = 'pending_employee'  // Admin draft
    manager_note = 'Drafted by admin'
} else {
    ot_status = 'pending_manager'   // Normal employee claim
}
```

### 3. Frontend Level (New Component Needed)

**Proposed Admin Draft Component**:
- Path: `/home/elt1541/leeattend/webdashboard/src/components/admin/DraftOtForEmployeeDialog.tsx`
- Similar structure to `ClaimOtDialog.tsx` but:
  - No validation against `system_detected_ot_minutes` (can exceed or create without system detection)
  - Additional field: employee selector (dropdown/autocomplete)
  - Target status: `pending_employee` instead of `pending_manager`

**Proposed Integration Page**:
- Path: `/home/elt1541/leeattend/webdashboard/src/pages/attendance/OtManagementPage.tsx`
- Features:
  - List all summaries (all employees, filterable)
  - "Draft OT" button for each employee/date
  - Status badges showing workflow state

### 4. Service Layer (Minor Extension)

**Location**: `/home/elt1541/leeattend/apiserver/src/services/attendance_summary_service.rs`

**Proposed New Method**:
```rust
pub async fn draft_ot_for_employee(
    pool: &PgPool,
    employee_id: Uuid,
    date: NaiveDate,
    claimed_minutes: i32,
    reason: Option<String>,
    drafted_by_admin: bool
) -> Result<(), String> {
    // Similar to claim_ot but:
    // - status = 'pending_employee' (not 'pending_manager')
    // - manager_note = 'Drafted by admin' if drafted_by_admin
}
```

## Compatibility with Approach A (Draft-and-Notify)

### Status: HIGHLY COMPATIBLE

**No Breaking Changes Required**:
- Existing status enum supports workflow (`pending_employee` already exists)
- Database schema supports admin metadata (`manager_note` can store admin context)
- Authorization patterns support admin actions

**Required Additions**:
1. Service method: `draft_ot_for_employee()` (or modify `claim_ot()` with role detection)
2. Frontend component: `DraftOtForEmployeeDialog.tsx`
3. Optional: New API route for semantic clarity (or reuse existing with role-based logic)

**Notification Integration Points**:
- Trigger: After successful draft creation (in service layer or handler)
- Target: Employee associated with `employee_id`
- Context: Pass `date`, `claimed_minutes`, `reason`, `manager_note`

**No Conflicts**:
- Admin drafts follow same status transitions as system-detected OT
- Employee workflow remains unchanged (still sees `pending_employee` → claims → `pending_manager`)
- Manager approval logic requires no changes

## Potential Blockers

**None identified**. All architectural patterns align with Approach A.

**Minor Considerations**:
1. **UI Validation Difference**: Employee claim dialog enforces `claimed_minutes <= system_detected_ot_minutes`, but admin draft should allow exceeding or creating without system detection. Solution: Conditional validation based on user role.

2. **Audit Trail**: Current schema does not track "who drafted" (admin vs system). Solution: Use `manager_note` field to store admin context (e.g., "Drafted by admin@example.com on 2026-01-28").

3. **Notification Service**: Not yet implemented in codebase. Solution: Add notification trigger hooks in service layer (separate feature).

## Warnings

- System recalculation (`recalculate_daily_summary`) auto-transitions `none` → `pending_employee` when OT detected. Admin draft creation should not conflict (same target status).
- UNIQUE constraint on (employee_id, date) means admin cannot create duplicate drafts for same date. UI should check existing summary before opening draft dialog.
- Current frontend displays `employee_id` UUID in tables (line 111 of OtApprovalsPage.tsx). Should join with employee name for better UX.

## Related Code Files

### Database
- `/home/elt1541/leeattend/apiserver/migrations/20260128000000_create_attendance_daily_summaries.sql`
- `/home/elt1541/leeattend/apiserver/migrations/20260128000001_fix_summary_nulls.sql`

### Backend (Rust)
- `/home/elt1541/leeattend/apiserver/src/models.rs` (lines 1507-1533)
- `/home/elt1541/leeattend/apiserver/src/services/attendance_summary_service.rs`
- `/home/elt1541/leeattend/apiserver/src/handlers/attendance.rs` (lines 674-826)
- `/home/elt1541/leeattend/apiserver/tests/ot_workflow_tests.rs`

### Frontend (TypeScript/React)
- `/home/elt1541/leeattend/webdashboard/src/components/ess/ClaimOtDialog.tsx`
- `/home/elt1541/leeattend/webdashboard/src/pages/approvals/OtApprovalsPage.tsx`
- `/home/elt1541/leeattend/webdashboard/src/pages/ess/MyAttendancePage.tsx`
- `/home/elt1541/leeattend/webdashboard/src/lib/api/attendance.ts` (lines 266-309)
- `/home/elt1541/leeattend/webdashboard/src/lib/api/ess.ts` (lines 179-232)
- `/home/elt1541/leeattend/webdashboard/src/lib/api/types.ts` (lines 797-811)
- `/home/elt1541/leeattend/webdashboard/src/hooks/use-ess.ts` (lines 246-280)

## Unresolved Questions

1. Should admin-drafted claims require a separate approval from another admin before notifying employee, or immediately notify employee?
2. Should admin be able to draft claims for dates in the future, or only past/current dates?
3. If system later detects OT for a date with admin draft, should it overwrite `system_detected_ot_minutes` or skip?
