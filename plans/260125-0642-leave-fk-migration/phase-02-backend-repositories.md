# Phase 2: Backend Repository Layer

## Context Links
- Plan: `./plan.md`
- Phase 1: `./phase-01-database-migration.md`
- leave_balance_repository: `apiserver/src/repositories/leave_balance_repository.rs`
- leave_request_repository: `apiserver/src/repositories/leave_request_repository.rs`

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 2h
- **Dependencies**: Phase 1 (migration must run first)

Update all repository functions to use FK joins instead of string matching.

## Key Insights
- Current queries use `leave_type VARCHAR` for filtering and grouping
- After migration, must use `leave_type_id UUID` and JOIN to get display name
- Response models need leave type metadata (code, name, is_paid)

## Requirements

### Functional
- Replace string-based `leave_type` with FK `leave_type_id`
- JOIN to `leave_types` table to get code/name for display
- Update unique constraint handling in upsert operations

### Non-Functional
- Maintain query performance with proper indexes
- Keep backward compatibility in response shapes where possible

## Related Code Files

### Files to Modify
- `apiserver/src/models.rs` - Update model structs
- `apiserver/src/repositories/leave_balance_repository.rs`
- `apiserver/src/repositories/leave_request_repository.rs`

## Implementation Steps

### Step 1: Update Models (models.rs)

Update `LeaveRequest` struct:
```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveRequest {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type_id: Uuid,          // Changed from leave_type: String
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub status: LeaveStatus,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Add response wrapper with leave type details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaveRequestWithType {
    #[serde(flatten)]
    pub request: LeaveRequest,
    pub leave_type_code: String,
    pub leave_type_name: String,
    pub is_paid: bool,
}
```

Update `LeaveBalance` struct:
```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveBalance {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type_id: Uuid,          // Changed from leave_type: String
    pub year: i32,
    pub entitlement: i32,
    pub accrued_amount: rust_decimal::Decimal,
    pub carried_over: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Update `EmployeeLeaveBalance` struct:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeLeaveBalance {
    pub leave_type_id: Uuid,          // Added
    pub leave_type_code: String,      // Changed from leave_type
    pub leave_type_name: String,      // Added for display
    pub is_paid: bool,                // Added
    pub entitlement: i32,
    pub accrued_amount: f64,
    pub carried_over: i32,
    pub used: i32,
    pub remaining: f64,
}
```

Update `WorkspaceLeaveBalance` struct:
```rust
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct WorkspaceLeaveBalance {
    pub employee_id: Uuid,
    pub employee_name: String,
    pub leave_type_id: Uuid,          // Added
    pub leave_type_code: String,      // Changed from leave_type
    pub leave_type_name: String,      // Added
    pub is_paid: bool,                // Added
    pub entitlement: i32,
    pub accrued_amount: f64,
    pub carried_over: i32,
    pub used: i32,
    pub remaining: f64,
    pub policy_name: Option<String>,
    pub assignment_type: Option<String>,
}
```

Update `LeaveAccrualLog` struct:
```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveAccrualLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub policy_assignment_id: Option<Uuid>,
    pub accrual_date: NaiveDate,
    pub amount: rust_decimal::Decimal,
    pub leave_type_id: Uuid,          // Changed from Option<Uuid>
    // Remove: pub leave_type: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

Update request models:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitLeaveRequest {
    pub employee_id: Uuid,
    pub leave_type_id: Uuid,          // Changed from leave_type: String
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdjustBalanceRequest {
    pub leave_type_id: Uuid,          // Changed from leave_type: String
    pub amount: f64,
    pub reason: String,
    pub transaction_date: NaiveDate,
}
```

### Step 2: Update leave_balance_repository.rs

Replace `get_employee_balances`:
```rust
pub async fn get_employee_balances(
    pool: &PgPool,
    employee_id: Uuid,
    year: i32,
) -> Result<Vec<EmployeeLeaveBalance>, sqlx::Error> {
    let balances = query_as!(
        EmployeeLeaveBalance,
        r#"
        SELECT
            lb.leave_type_id as "leave_type_id!",
            lt.code as "leave_type_code!",
            lt.name as "leave_type_name!",
            lt.is_paid as "is_paid!",
            lb.entitlement as "entitlement!",
            lb.accrued_amount::FLOAT8 as "accrued_amount!",
            lb.carried_over as "carried_over!",
            COALESCE(
                (
                    SELECT SUM(end_date - start_date + 1)
                    FROM leave_requests
                    WHERE employee_id = lb.employee_id
                    AND leave_type_id = lb.leave_type_id
                    AND status = 'approved'
                    AND EXTRACT(YEAR FROM start_date) = lb.year
                ),
                0
            )::INT as "used!",
            (
                lb.entitlement + lb.accrued_amount + lb.carried_over - COALESCE(
                    (
                        SELECT SUM(end_date - start_date + 1)
                        FROM leave_requests
                        WHERE employee_id = lb.employee_id
                        AND leave_type_id = lb.leave_type_id
                        AND status = 'approved'
                        AND EXTRACT(YEAR FROM start_date) = lb.year
                    ),
                    0
                )
            )::FLOAT8 as "remaining!"
        FROM leave_balances lb
        JOIN leave_types lt ON lb.leave_type_id = lt.id
        WHERE lb.employee_id = $1 AND lb.year = $2
        "#,
        employee_id,
        year
    )
    .fetch_all(pool)
    .await?;

    Ok(balances)
}
```

Replace `get_workspace_balances`:
```rust
pub async fn get_workspace_balances(
    pool: &PgPool,
    workspace_id: Uuid,
    year: i32,
    limit: i64,
    offset: i64,
    search: Option<String>,
) -> Result<(Vec<WorkspaceLeaveBalance>, i64), sqlx::Error> {
    let search_term = format!("%{}%", search.unwrap_or_default());

    let count_sql = r#"
        SELECT COUNT(*)
        FROM employees e
        LEFT JOIN leave_balances lb ON e.id = lb.employee_id AND lb.year = $2
        LEFT JOIN leave_types lt ON lb.leave_type_id = lt.id
        WHERE e.workspace_id = $1 AND e.is_active = TRUE
          AND ($3 = '%%' OR e.full_name ILIKE $3 OR lt.name ILIKE $3 OR lt.code ILIKE $3)
    "#;

    let total: i64 = sqlx::query_scalar(count_sql)
        .bind(workspace_id)
        .bind(year)
        .bind(&search_term)
        .fetch_one(pool)
        .await?;

    let sql = r#"
        SELECT
            e.id as employee_id,
            e.full_name as employee_name,
            COALESCE(lb.leave_type_id, (SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation' LIMIT 1)) as leave_type_id,
            COALESCE(lt.code, 'vacation') as leave_type_code,
            COALESCE(lt.name, 'Vacation') as leave_type_name,
            COALESCE(lt.is_paid, TRUE) as is_paid,
            COALESCE(lb.entitlement, 0) as entitlement,
            COALESCE(lb.accrued_amount::FLOAT8, 0.0) as accrued_amount,
            COALESCE(lb.carried_over, 0) as carried_over,
            COALESCE(
                (
                    SELECT SUM(end_date - start_date + 1)
                    FROM leave_requests
                    WHERE employee_id = e.id
                    AND leave_type_id = lb.leave_type_id
                    AND status = 'approved'
                    AND EXTRACT(YEAR FROM start_date) = $2
                ),
                0
            )::INT as used,
            (
                COALESCE(lb.entitlement, 0) + COALESCE(lb.accrued_amount, 0) + COALESCE(lb.carried_over, 0) - COALESCE(
                    (
                        SELECT SUM(end_date - start_date + 1)
                        FROM leave_requests
                        WHERE employee_id = e.id
                        AND leave_type_id = lb.leave_type_id
                        AND status = 'approved'
                        AND EXTRACT(YEAR FROM start_date) = $2
                    ),
                    0
                )
            )::FLOAT8 as remaining,
            (
                SELECT d.name
                FROM leave_policy_assignments a
                JOIN leave_policy_definitions d ON a.policy_id = d.id
                WHERE a.workspace_id = $1
                  AND d.leave_type_id = lb.leave_type_id
                  AND (
                    (a.assignee_type = 'employee' AND a.assignee_id = e.id) OR
                    (a.assignee_type = 'department' AND a.assignee_id = (SELECT department_id FROM employee_department_history WHERE employee_id = e.id ORDER BY effective_date DESC LIMIT 1)) OR
                    (a.assignee_type = 'workspace')
                  )
                  AND a.effective_from <= NOW()
                  AND (a.effective_to IS NULL OR a.effective_to > NOW())
                ORDER BY
                  CASE a.assignee_type
                    WHEN 'employee' THEN 1
                    WHEN 'department' THEN 2
                    WHEN 'workspace' THEN 3
                  END ASC
                LIMIT 1
            ) as policy_name,
            (
                SELECT a.assignee_type
                FROM leave_policy_assignments a
                JOIN leave_policy_definitions d ON a.policy_id = d.id
                WHERE a.workspace_id = $1
                  AND d.leave_type_id = lb.leave_type_id
                  AND (
                    (a.assignee_type = 'employee' AND a.assignee_id = e.id) OR
                    (a.assignee_type = 'department' AND a.assignee_id = (SELECT department_id FROM employee_department_history WHERE employee_id = e.id ORDER BY effective_date DESC LIMIT 1)) OR
                    (a.assignee_type = 'workspace')
                  )
                  AND a.effective_from <= NOW()
                  AND (a.effective_to IS NULL OR a.effective_to > NOW())
                ORDER BY
                  CASE a.assignee_type
                    WHEN 'employee' THEN 1
                    WHEN 'department' THEN 2
                    WHEN 'workspace' THEN 3
                  END ASC
                LIMIT 1
            ) as assignment_type
        FROM employees e
        LEFT JOIN leave_balances lb ON e.id = lb.employee_id AND lb.year = $2
        LEFT JOIN leave_types lt ON lb.leave_type_id = lt.id
        WHERE e.workspace_id = $1 AND e.is_active = TRUE
          AND ($3 = '%%' OR e.full_name ILIKE $3 OR lt.name ILIKE $3 OR lt.code ILIKE $3)
        ORDER BY e.full_name, lt.name
        LIMIT $4 OFFSET $5
    "#;

    let data = sqlx::query_as::<_, WorkspaceLeaveBalance>(sql)
        .bind(workspace_id)
        .bind(year)
        .bind(search_term)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok((data, total))
}
```

Replace `adjust_balance`:
```rust
pub async fn adjust_balance(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    leave_type_id: Uuid,
    amount: rust_decimal::Decimal,
    transaction_date: chrono::NaiveDate,
    reason: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Log the adjustment
    sqlx::query!(
        r#"
        INSERT INTO leave_accrual_logs (workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type_id, notes)
        VALUES ($1, $2, NULL, $3, $4, $5, $6)
        "#,
        workspace_id,
        employee_id,
        transaction_date,
        amount,
        leave_type_id,
        reason
    )
    .execute(&mut *tx)
    .await?;

    let year = transaction_date.year();

    sqlx::query!(
        r#"
        INSERT INTO leave_balances (employee_id, leave_type_id, year, entitlement, accrued_amount, carried_over)
        VALUES ($1, $2, $3, 0, $4, 0)
        ON CONFLICT (employee_id, leave_type_id, year)
        DO UPDATE SET
            accrued_amount = leave_balances.accrued_amount + EXCLUDED.accrued_amount,
            updated_at = CURRENT_TIMESTAMP
        "#,
        employee_id,
        leave_type_id,
        year,
        amount
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
```

### Step 3: Update leave_request_repository.rs

Replace `create_leave_request`:
```rust
pub async fn create_leave_request(
    pool: &PgPool,
    request: &LeaveRequest,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO leave_requests
            (id, employee_id, leave_type_id, start_date, end_date, reason, status,
             requested_by_user_id, approved_by_user_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
        request.id,
        request.employee_id,
        request.leave_type_id,
        request.start_date,
        request.end_date,
        request.reason,
        request.status as LeaveStatus,
        request.requested_by_user_id,
        request.approved_by_user_id,
        request.created_at,
        request.updated_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}
```

Replace all SELECT queries to use FK:
```rust
pub async fn get_leave_request_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<LeaveRequest>, sqlx::Error> {
    query_as!(
        LeaveRequest,
        r#"
        SELECT id, employee_id, leave_type_id, start_date, end_date, reason,
               status as "status: LeaveStatus", requested_by_user_id, approved_by_user_id,
               created_at as "created_at!", updated_at as "updated_at!"
        FROM leave_requests WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}
```

Update `get_approved_leave_breakdown` to use FK:
```rust
pub async fn get_approved_leave_breakdown(
    pool: &PgPool,
    employee_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<LeaveBreakdown, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT lt.code as leave_type_code, SUM(lr.end_date - lr.start_date + 1) as days
        FROM leave_requests lr
        JOIN leave_types lt ON lr.leave_type_id = lt.id
        WHERE lr.employee_id = $1
          AND lr.status = 'approved'
          AND lr.start_date <= $3
          AND lr.end_date >= $2
        GROUP BY lt.code
        "#,
    )
    .bind(employee_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    let mut breakdown = LeaveBreakdown { total_days: 0.0, unpaid_days: 0.0, sick_days: 0.0 };
    for row in rows {
        let days: i64 = row.get("days");
        let days_f = days as f64;
        breakdown.total_days += days_f;
        let leave_type_code: String = row.get("leave_type_code");
        if leave_type_code == "unpaid" || leave_type_code == "personal" {
            breakdown.unpaid_days += days_f;
        } else if leave_type_code == "sick" {
            breakdown.sick_days += days_f;
        }
    }
    Ok(breakdown)
}
```

## Todo List
- [ ] Update LeaveRequest model in models.rs
- [ ] Update LeaveBalance model in models.rs
- [ ] Update EmployeeLeaveBalance model
- [ ] Update WorkspaceLeaveBalance model
- [ ] Update LeaveAccrualLog model
- [ ] Update SubmitLeaveRequest model
- [ ] Update AdjustBalanceRequest model
- [ ] Update leave_balance_repository.rs - get_employee_balances
- [ ] Update leave_balance_repository.rs - get_workspace_balances
- [ ] Update leave_balance_repository.rs - adjust_balance
- [ ] Update leave_balance_repository.rs - upsert_leave_balance
- [ ] Update leave_request_repository.rs - create_leave_request
- [ ] Update leave_request_repository.rs - get_leave_request_by_id
- [ ] Update leave_request_repository.rs - get_employee_leave_requests
- [ ] Update leave_request_repository.rs - get_workspace_leave_requests
- [ ] Update leave_request_repository.rs - get_approved_leave_breakdown
- [ ] Run `cargo check` to verify compilation

## Success Criteria
- All repository functions compile
- Queries use FK joins
- Response models include leave type metadata

## Next Steps
Proceed to Phase 3 (Backend Service Layer).
