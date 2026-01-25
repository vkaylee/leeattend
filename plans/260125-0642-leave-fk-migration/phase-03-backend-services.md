# Phase 3: Backend Service Layer

## Context Links
- Plan: `./plan.md`
- Phase 2: `./phase-02-backend-repositories.md`
- leave_service: `apiserver/src/services/leave_service.rs`
- leave_accrual_service: `apiserver/src/services/leave_accrual_service.rs`

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 1.5h
- **Dependencies**: Phase 2 (repositories must be updated first)

Update service layer to work with UUID leave_type_id instead of strings.

## Key Insights
- `LeaveService` uses string `leave_type` throughout
- `LeaveAccrualService` already partially uses `leave_type_id` in some queries
- Services construct model objects that need updated field names

## Requirements

### Functional
- Update `LeaveService` to accept `leave_type_id` in requests
- Update `LeaveAccrualService` to use FK consistently
- Update `get_employee_leave_history` to join leave_types for display

### Non-Functional
- Maintain existing business logic
- Keep accrual calculations accurate

## Related Code Files

### Files to Modify
- `apiserver/src/services/leave_service.rs`
- `apiserver/src/services/leave_accrual_service.rs`

## Implementation Steps

### Step 1: Update LeaveService (leave_service.rs)

Update `submit_leave_request`:
```rust
pub async fn submit_leave_request(
    pool: &PgPool,
    workspace_id: Uuid,
    user_id: Uuid,
    req: SubmitLeaveRequest,
) -> Result<LeaveRequest, StatusCode> {
    // Verify employee belongs to workspace
    let _ = employee_repository::get_employee_by_id(pool, req.employee_id, workspace_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Validate leave_type_id exists and belongs to workspace
    let leave_type = sqlx::query!(
        "SELECT id FROM leave_types WHERE id = $1 AND workspace_id = $2",
        req.leave_type_id,
        workspace_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::BAD_REQUEST)?; // Leave type not found

    let request = LeaveRequest {
        id: Uuid::new_v4(),
        employee_id: req.employee_id,
        leave_type_id: req.leave_type_id,
        start_date: req.start_date,
        end_date: req.end_date,
        reason: req.reason,
        status: LeaveStatus::Pending,
        requested_by_user_id: Some(user_id),
        approved_by_user_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    leave_request_repository::create_leave_request(pool, &request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(request)
}
```

Update `approve_leave_request` to get leave type code for roster note:
```rust
pub async fn approve_leave_request(
    pool: &PgPool,
    workspace_id: Uuid,
    user_id: Uuid,
    request_id: Uuid,
    req: ApproveLeaveRequest,
) -> Result<(), StatusCode> {
    let request = leave_request_repository::get_leave_request_by_id(pool, request_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verify employee of this request belongs to the workspace
    let _ = employee_repository::get_employee_by_id(pool, request.employee_id, workspace_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    leave_request_repository::update_leave_request_status(
        pool,
        request_id,
        req.status,
        Some(user_id),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If approved, update roster
    if req.status == LeaveStatus::Approved {
        // Get leave type name for roster note
        let leave_type = sqlx::query!(
            "SELECT name FROM leave_types WHERE id = $1",
            request.leave_type_id
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut current_date = request.start_date;
        while current_date <= request.end_date {
            let roster = ShiftRoster {
                id: Uuid::new_v4(),
                employee_id: request.employee_id,
                shift_id: None,
                date: current_date,
                status: "on_leave".to_string(),
                notes: Some(format!("Leave: {}", leave_type.name)),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let _ = roster_repository::upsert_roster(pool, &roster)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if let Some(next_date) = current_date.succ_opt() {
                current_date = next_date;
            } else {
                break;
            }
        }
    }

    Ok(())
}
```

Update `adjust_balance`:
```rust
pub async fn adjust_balance(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    req: AdjustBalanceRequest,
) -> Result<(), StatusCode> {
    // Validate leave_type_id
    let _ = sqlx::query!(
        "SELECT id FROM leave_types WHERE id = $1 AND workspace_id = $2",
        req.leave_type_id,
        workspace_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::BAD_REQUEST)?;

    let amount_dec =
        rust_decimal::Decimal::from_f64_retain(req.amount).ok_or(StatusCode::BAD_REQUEST)?;

    leave_balance_repository::adjust_balance(
        pool,
        workspace_id,
        employee_id,
        req.leave_type_id,
        amount_dec,
        req.transaction_date,
        &req.reason,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
```

Update `get_employee_leave_history` to join leave_types:
```rust
pub async fn get_employee_leave_history(
    pool: &PgPool,
    _workspace_id: Uuid,
    employee_id: Uuid,
    year: i32,
) -> Result<Vec<LeaveHistoryItem>, StatusCode> {
    // Get Accruals & Adjustments with leave type name
    let logs = sqlx::query!(
        r#"
        SELECT lal.id, lal.accrual_date, lal.amount, lt.code as leave_type_code,
               lt.name as leave_type_name, lal.notes, lal.created_at, lal.policy_assignment_id
        FROM leave_accrual_logs lal
        JOIN leave_types lt ON lal.leave_type_id = lt.id
        WHERE lal.employee_id = $1
          AND EXTRACT(YEAR FROM lal.accrual_date) = $2
        "#,
        employee_id,
        year as f64
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get Usage (Approved Requests) with leave type name
    let requests = sqlx::query!(
        r#"
        SELECT lr.id, lr.start_date, lr.end_date, lt.code as leave_type_code,
               lt.name as leave_type_name, lr.reason, lr.created_at
        FROM leave_requests lr
        JOIN leave_types lt ON lr.leave_type_id = lt.id
        WHERE lr.employee_id = $1
          AND lr.status = 'approved'
          AND EXTRACT(YEAR FROM lr.start_date) = $2
        "#,
        employee_id,
        year as f64
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut history: Vec<LeaveHistoryItem> = Vec::new();

    for log in logs {
        let transaction_type = if log.policy_assignment_id.is_some() {
            LeaveTransactionType::Accrual
        } else {
            LeaveTransactionType::Adjustment
        };
        history.push(LeaveHistoryItem {
            id: log.id,
            date: log.accrual_date,
            transaction_type,
            amount: log.amount.to_f64().unwrap_or(0.0),
            reason: log.notes.or(Some(format!("{} ({})", log.leave_type_name, log.leave_type_code))),
            reference_id: None,
            created_at: log.created_at.unwrap_or_else(chrono::Utc::now),
        });
    }

    for req in requests {
        let days = (req.end_date - req.start_date).num_days() + 1;
        history.push(LeaveHistoryItem {
            id: Uuid::new_v4(),
            date: req.start_date,
            transaction_type: LeaveTransactionType::Usage,
            amount: -(days as f64),
            reason: req.reason.or(Some(format!("{} ({})", req.leave_type_name, req.leave_type_code))),
            reference_id: Some(req.id),
            created_at: req.created_at.unwrap_or_else(chrono::Utc::now),
        });
    }

    history.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(history)
}
```

### Step 2: Update LeaveAccrualService (leave_accrual_service.rs)

The `process_workspace_accruals` function needs updates.

Key changes in the main loop:
```rust
// In process_workspace_accruals, replace string-based log checks:

// Old:
// let existing_log = sqlx::query!(
//     "SELECT id, amount FROM leave_accrual_logs WHERE employee_id = $1 AND leave_type = $2 AND accrual_date = $3",
//     emp.id, leave_type_code, current_check_date
// )

// New:
let existing_log = sqlx::query!(
    r#"
    SELECT id, amount FROM leave_accrual_logs
    WHERE employee_id = $1 AND leave_type_id = $2 AND accrual_date = $3
    "#,
    emp.id,
    policy.leave_type_id,
    current_check_date
)
.fetch_optional(pool).await?;
```

Update balance upsert:
```rust
// Old:
// INSERT INTO leave_balances (employee_id, year, leave_type, ...)
// ON CONFLICT (employee_id, leave_type, year)

// New:
sqlx::query!(
    r#"
    INSERT INTO leave_balances (employee_id, year, leave_type_id, entitlement, accrued_amount, carried_over)
    VALUES ($1, $2, $3, 0, $4, 0)
    ON CONFLICT (employee_id, leave_type_id, year)
    DO UPDATE SET
        accrued_amount = leave_balances.accrued_amount + EXCLUDED.accrued_amount,
        updated_at = NOW()
    "#,
    emp.id,
    current_check_date.year(),
    policy.leave_type_id,
    decimal_amount
)
.execute(pool).await?;
```

Update log insertion:
```rust
// Remove leave_type string parameter, use only leave_type_id
let log = sqlx::query_as!(
    LeaveAccrualLog,
    r#"
    INSERT INTO leave_accrual_logs (workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type_id)
    VALUES ($1, $2, $3, $4, $5, $6)
    ON CONFLICT (employee_id, leave_type_id, accrual_date) DO NOTHING
    RETURNING id, workspace_id, employee_id, policy_assignment_id, accrual_date, amount,
              leave_type_id, notes, created_at as "created_at!"
    "#,
    workspace_id,
    emp.id,
    Some(assignment_id),
    current_check_date,
    decimal_amount,
    policy.leave_type_id
)
.fetch_optional(pool)
.await?;
```

## Todo List
- [ ] Update LeaveService::submit_leave_request
- [ ] Update LeaveService::approve_leave_request
- [ ] Update LeaveService::adjust_balance
- [ ] Update LeaveService::get_employee_leave_history
- [ ] Update LeaveAccrualService::process_workspace_accruals
- [ ] Update balance upsert logic in accrual service
- [ ] Update log queries to use leave_type_id
- [ ] Run `cargo check` to verify compilation

## Success Criteria
- All service functions compile
- Leave type validation uses FK lookup
- Accrual logs use leave_type_id consistently

## Next Steps
Proceed to Phase 4 (Backend API Handlers).
