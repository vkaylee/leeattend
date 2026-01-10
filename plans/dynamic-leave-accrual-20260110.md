# Plan: Implement Dynamic Leave Accrual with Temporal Validity

## Goal
Implement a flexible leave accrual system that supports various company policies (monthly, quarterly, seniority-based) and handles policy changes over time (effective dating).

## 1. Database Schema Design
We need to store "Rules" and "Assignments".

### 1.1 New Tables
- **`leave_policy_definitions`**: Stores the rule logic.
    - `id`: UUID (PK)
    - `workspace_id`: UUID (FK)
    - `name`: Text (e.g., "Standard Monthly")
    - `rule_config`: JSONB (The parametric rules: frequency, amount, seniority logic)
    - `created_at`, `updated_at`
- **`leave_policy_assignments`**: Maps policies to entities (Global/Dept/Employee) over time.
    - `id`: UUID (PK)
    - `workspace_id`: UUID
    - `assignee_type`: Text ('workspace', 'department', 'employee')
    - `assignee_id`: UUID (Nullable for workspace-wide)
    - `policy_id`: UUID (FK)
    - `effective_from`: Date
    - `effective_to`: Date (Nullable)
    - *Constraint:* No overlapping time ranges for the same assignee.
- **`leave_accrual_logs`**: Audit trail of system-generated accruals.
    - `id`: UUID
    - `employee_id`: UUID
    - `policy_id`: UUID
    - `amount`: Numeric
    - `accrual_date`: Date
    - `notes`: Text (e.g., "Monthly accrual for Feb 2026")

### 1.2 Update Existing Tables
- Modify `leave_balances`:
    - Add `accrued_amount`: Numeric (Total accrued so far).
    - *Decision:* `entitlement` can remain as the "Manual Override" or "Projected Total", while `accrued_amount` is what the system calculated. Or we treat `entitlement` as the bucket we fill. Let's add `accrued_amount` for clarity.

## 2. Domain Logic (Rust)

### 2.1 Structs & Enums
- `LeavePolicyRule` (Enum/Struct): serialization/deserialization of the JSONB.
    - `Frequency`: Monthly, Quarterly, Yearly.
    - `AccrualType`: Flat, SeniorityBased.
    - `ResetRule`: When to reset.

### 2.2 Core Service: `LeaveAccrualService`
- `find_active_policy(entity_id, date)`: Resolves hierarchy (Employee > Dept > Workspace).
- `calculate_accrual(policy, employee_stats)`: Pure function returning amount.
- `process_accruals(date)`: Batch job to run for all employees.

## 3. API Endpoints
- `POST /api/leave-policies`: Create new policy definition.
- `POST /api/leave-policy-assignments`: Assign policy (with conflict check).
- `GET /api/leave-policies`: List.
- `POST /api/debug/trigger-accrual`: (Dev only) Trigger calculation for testing.

## 4. Testing Strategy
- **Unit Tests**:
    - Test `calculate_accrual` with:
        - Case A: 12 days/year (1/month).
        - Case B: Seniority (Year 1: 12, Year 5: 13).
        - Case C: Quarterly (3 months = 1 day).
- **Integration Tests**:
    - Setup Employee, Assign Policy A (Jan-Mar), Policy B (Apr-Dec).
    - Run Accrual for Feb -> Check result A.
    - Run Accrual for May -> Check result B.
