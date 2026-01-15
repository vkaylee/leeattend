# Implementation Plan: Admin Leave Management (Balances & History)

## Approach
To provide Admins with full visibility and control over employee leave, we will implement a dual-layer UI: a high-level **Balances Table** for overview and a detailed **History & Adjustment Modal** for specific actions. On the backend, we will unify accrual logs and leave usage into a single "History" stream and provide an endpoint for manual adjustments.

## Steps

### 1. Backend: Leave History & Adjustment API (45 min)
*   **Goal**: Enable fetching consolidated history and performing manual balance adjustments.
*   **Files**:
    *   `apiserver/src/models.rs`: Add `LeaveHistoryItem`, `AdjustBalanceRequest`.
    *   `apiserver/src/repositories/leave_balance_repository.rs`: Add `get_history`, `adjust_balance`.
    *   `apiserver/src/services/leave_service.rs`: Logic to merge `leave_accrual_logs` and `leave_requests`.
    *   `apiserver/src/handlers.rs`: Add endpoints.
*   **Key Logic**:
    *   `GET .../history`: Union of `leave_accrual_logs` (mapped to `type: accrual/adjustment`) and `leave_requests` (mapped to `type: usage`, negative amount).
    *   `POST .../adjust`: Insert into `leave_accrual_logs` (with `policy_assignment_id` = NULL) AND update `leave_balances`.

### 2. Frontend: Admin Balances View (45 min)
*   **Goal**: "Bird's-eye view" of all employee balances.
*   **Files**:
    *   `webdashboard/src/lib/api.ts`: Add `fetchLeaveBalances (all)`, `fetchLeaveHistory`, `adjustLeaveBalance`.
    *   `webdashboard/src/pages/LeaveBalancesPage.tsx`: New page for Admins.
*   **UI Components**:
    *   **Filter Bar**: Department, Leave Type search.
    *   **Balances Table**: Columns for Employee, Policy, Entitlement, Used, Pending, Balance (with Progress Bar).

### 3. Frontend: History & Adjustment Modal (45 min)
*   **Goal**: Detailed audit trail and manual control.
*   **Files**:
    *   `webdashboard/src/components/leave/LeaveHistoryModal.tsx`: New component.
*   **Features**:
    *   **History Table**: Date | Type (Color-coded) | Amount | Reason | User.
    *   **Adjustment Form**: Add/Deduct radio, Amount input, Reason text (Required).

### 4. Integration & Testing (30 min)
*   **Integration**: Add "Leave Balances" to the navigation menu (visible to Admins).
*   **Testing**:
    *   Verify manual adjustment correctly updates `leave_balances`.
    *   Verify history shows both auto-accruals and manual adjustments.
    *   Check UI filters work.

## Timeline
| Phase | Duration |
|-------|----------|
| Backend API | 45 min |
| Admin Balances UI | 45 min |
| History/Adjust Modal | 45 min |
| Integration & Test | 30 min |
| **Total** | **2h 45m** |

## Security Checklist
- [ ] **RBAC**: Ensure only `admin` or `owner` can access these new endpoints.
- [ ] **Validation**: Prevent adjusting balance to negative values (optional, but good practice to warn).
- [ ] **Audit**: Ensure `adjust_balance` always requires a `reason` note.

## API Specification

### `GET /api/workspaces/:ws_id/employees/:emp_id/leave/history`
**Query Params:** `?year=2026&leave_type=vacation`
**Response:**
```json
[
  {
    "date": "2026-01-01",
    "type": "accrual",
    "amount": 1.0,
    "reason": "Monthly Accrual",
    "reference_id": "uuid..."
  },
  {
    "date": "2026-02-15",
    "type": "usage",
    "amount": -2.0,
    "reason": "Sick Leave",
    "reference_id": "uuid..."
  }
]
```

### `POST /api/workspaces/:ws_id/employees/:emp_id/leave/adjust`
**Body:**
```json
{
  "leave_type": "vacation",
  "amount": 1.0, // Positive to add, Negative to deduct
  "reason": "Correction for missed seniority bonus",
  "transaction_date": "2026-03-10"
}
```
