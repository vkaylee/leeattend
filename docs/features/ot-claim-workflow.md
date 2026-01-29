# Feature: Overtime (OT) Claim and Approval Workflow

## Overview
The OT Claim and Approval Workflow ensures that overtime compensation is accurate, consensual, and compliant. It implements a "System Detect -> Employee Claim -> Manager Approve" model, replacing purely automatic calculations for payroll purposes.

## Workflow

### 1. System Detection (Automatic)
- **Trigger:** Real-time attendance log processing or manual time entry updates.
- **Logic:** The `AttendanceCalculator` compares actual punch times against the assigned `Shift` and `PolicyRules`.
- **Output:**
    - Calculates `actual_work_minutes`.
    - Calculates `system_detected_ot_minutes` (Early OT + Late OT).
    - Updates `attendance_daily_summaries` table.
    - **State Transition:** If `system_detected_ot_minutes > 0` and current status is `None`, status becomes `PendingEmployee`.

### 2. Employee Claim (ESS)
- **Actor:** Employee via "My Attendance" page.
- **Action:**
    - Employees see a "Claim" button for days with system-detected OT.
    - They input `claimed_minutes` (capped at system detected amount) and a `reason`.
- **State Transition:** Status becomes `PendingManager`.

### 3. Manager Approval
- **Actor:** Manager/Admin via "OT Approvals" page.
- **Action:**
    - Reviews pending claims.
    - **Approve:** Sets `approved_ot_minutes` (usually equal to claimed) and status to `Approved`.
    - **Reject:** Sets `approved_ot_minutes` to 0 and status to `Rejected`.
- **Impact:** Payroll calculations will use `approved_ot_minutes`.

## Data Model

### `attendance_daily_summaries` Table
| Column | Type | Description |
|--------|------|-------------|
| `date` | Date | The specific work day. |
| `actual_work_minutes` | Int | Standard work duration (excluding OT). |
| `system_detected_ot_minutes` | Int | Raw OT calculated by system policies. |
| `claimed_ot_minutes` | Int | Amount requested by employee. |
| `approved_ot_minutes` | Int | Final amount approved for payroll. |
| `ot_status` | Enum | `None`, `PendingEmployee`, `PendingManager`, `Approved`, `Rejected` |

## API Endpoints

### Employee Actions
- `POST /api/v1/workspaces/:id/attendance/claim-ot/:employee_id`
    - Payload: `{ "date": "2024-01-01", "claimed_minutes": 60, "reason": "Bug fix" }`

### Manager Actions
- `POST /api/v1/workspaces/:id/attendance/approve-ot/:employee_id`
    - Payload: `{ "date": "2024-01-01", "approved_minutes": 60, "status": "approved", "manager_note": "Ok" }`

### Data Retrieval
- `GET /api/v1/workspaces/:id/attendance/summaries`
    - Query Params: `start_date`, `end_date`, `employee_id` (optional for admins).

## UI Components
- **ESS:** `MyAttendancePage`, `ClaimOtDialog`.
- **Manager:** `OtApprovalsPage`.
