# Phase 2: Service Layer - Daily Report Refactoring

## Context Links
- [Phase 1: Repository Layer](./phase-01-repository-layer.md)
- [Scout Report](../reports/scout-260130-1711-attendance-report-architecture.md)
- Related: `apiserver/src/services/report_service.rs:22-239`

## Overview
- **Priority**: P1
- **Status**: Pending
- **Effort**: 1.5h
- **Depends on**: Phase 1

Refactor `get_daily_attendance_report()` to use database-level pagination instead of in-memory skip/take.

## Key Insights
1. Current flow: Fetch ALL schedules -> Fetch ALL time entries (N+1) -> Calculate ALL -> Skip/Take
2. Target flow: Fetch paginated employee IDs -> Fetch schedules/entries ONLY for those -> Calculate
3. `roster_repository::get_workspace_schedules()` already supports `employee_ids` filter
4. Total count comes from employee pagination, NOT from calculated reports

## Requirements

### Functional
- F1: Use `get_paginated_employee_ids()` for DB-level pagination
- F2: Fetch schedules/time entries only for paginated employee batch
- F3: Calculate reports only for paginated batch
- F4: Total count reflects total employees (not calculated records)

### Non-Functional
- NF1: Response time < 500ms for page 1 with 700 employees
- NF2: Memory usage constant regardless of total employees
- NF3: Maintain data consistency with existing calculation logic

## Architecture

### Before (In-Memory Pagination)
```
1. get_workspace_schedules(ALL employees)        -> O(N) employees
2. for each employee: get_employee_time_entries  -> O(N) queries
3. for each schedule: calculate_daily_units      -> O(N*D) calculations
4. reports.skip(offset).take(limit)              -> Memory: O(N*D)
```

### After (Database Pagination)
```
1. get_paginated_employee_ids(page, limit)       -> O(1) query, returns IDs + total
2. get_workspace_schedules(filtered_ids)         -> O(L) employees (L = limit)
3. get_workspace_time_entries_batch(filtered_ids)-> O(1) query
4. for each schedule: calculate_daily_units      -> O(L*D) calculations
```

## Related Code Files

### Files to Modify
- `apiserver/src/services/report_service.rs` (refactor `get_daily_attendance_report`)

### Files to Reference
- `apiserver/src/repositories/roster_repository.rs` (existing schedule fetch)
- `apiserver/src/services/attendance_calculator.rs` (calculation logic - unchanged)

## Implementation Steps

### Step 1: Update Function Signature (Unchanged)
Keep signature unchanged for backward compatibility:
```rust
pub async fn get_daily_attendance_report(
    pool: &PgPool,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    page: Option<i64>,
    limit: Option<i64>,
    employee_ids: Option<Vec<Uuid>>,
) -> Result<PaginatedResponse<DailyAttendanceReport>, StatusCode>
```

### Step 2: Refactor Implementation

Replace lines 22-239 in `report_service.rs`:

```rust
pub async fn get_daily_attendance_report(
    pool: &PgPool,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    page: Option<i64>,
    limit: Option<i64>,
    employee_ids: Option<Vec<Uuid>>,
) -> Result<PaginatedResponse<DailyAttendanceReport>, StatusCode> {
    // 0. Get Workspace Settings
    let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
    let workspace = workspace_repo
        .get_workspace_by_id(workspace_id)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching workspace: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

    // 1. PAGINATION AT DATABASE LEVEL
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(20);

    let (paginated_employees, total_employees) = employee_repository::get_paginated_employee_ids(
        pool,
        workspace_id,
        employee_ids.as_ref(),
        page,
        limit,
    )
    .await
    .map_err(|e| {
        tracing::error!("Error fetching paginated employees: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Early return if no employees
    if paginated_employees.is_empty() {
        return Ok(PaginatedResponse {
            data: Vec::new(),
            total: total_employees as usize,
            page,
            limit,
        });
    }

    // 2. Get employee IDs for filtered queries
    let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

    // 3. Get schedules ONLY for paginated employees
    let schedules = roster_repository::get_workspace_schedules(
        pool,
        workspace_id,
        start_date,
        end_date,
        Some(&emp_ids),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 4. Convert dates for time entry query
    let start_dt = tz
        .from_local_datetime(&start_date.and_hms_opt(0, 0, 0).ok_or(StatusCode::BAD_REQUEST)?)
        .single()
        .ok_or(StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);

    let end_dt = tz
        .from_local_datetime(&end_date.and_hms_opt(23, 59, 59).ok_or(StatusCode::BAD_REQUEST)?)
        .single()
        .ok_or(StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);

    // 5. BATCH fetch time entries (eliminates N+1)
    let entries_map = time_entry_repository::get_workspace_time_entries_batch(
        pool,
        &emp_ids,
        start_dt,
        end_dt,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 6. Calculate reports (same logic as before)
    let mut reports = Vec::new();

    for emp_sched in schedules {
        // Get entries from HashMap (O(1) lookup instead of DB query)
        let entries = entries_map.get(&emp_sched.employee_id).cloned().unwrap_or_default();

        for daily in emp_sched.schedules {
            let date = daily.date;

            // Filter entries for this day (existing logic)
            let day_entries: Vec<_> = entries
                .iter()
                .filter(|e| e.punch_in_time.with_timezone(&tz).date_naive() == date)
                .collect();

            // ... REST OF CALCULATION LOGIC UNCHANGED ...
            // (Copy lines 93-213 from original implementation)
        }
    }

    // 7. Return paginated response (NO in-memory skip/take)
    Ok(PaginatedResponse {
        data: reports,
        total: total_employees as usize,  // Total from DB pagination
        page,
        limit,
    })
}
```

### Step 3: Add Import
Add to imports at top of file:
```rust
use crate::repositories::employee_repository;
```

### Step 4: Important Note on Total Count

**CRITICAL**: The `total` now represents total EMPLOYEES, not total report ROWS.

For daily report, total rows = employees * days in range. But pagination is by EMPLOYEE, not by row.

If requirement is to paginate by individual day-employee records, additional changes needed:
- Add date-based pagination parameters
- Restructure query to return flattened results

Current implementation: Paginate by employee, return all days for each employee in batch.

## Todo List
- [ ] Add `employee_repository` import
- [ ] Replace pagination logic (DB-level instead of in-memory)
- [ ] Replace time entry fetch (batch instead of N+1)
- [ ] Keep calculation logic unchanged
- [ ] Update total count source
- [ ] Test with 700+ employees dataset
- [ ] Verify response time < 500ms

## Success Criteria
1. SQL shows `LIMIT 20` when fetching employees
2. Only 1 time_entries query (batch) instead of N queries
3. Response time < 500ms for page 1
4. Data output identical to original for same employee set
5. Memory usage bounded by batch size, not total employees

## Risk Assessment
- **Medium Risk**: Total count semantics changed (employees vs rows)
- **Mitigation**: Document behavior change, update API docs if needed
- **Low Risk**: Calculation logic unchanged, data consistency preserved

## Security Considerations
- workspace_id filter applied at pagination level
- employee_ids filter restricts to authorized employees
- No bypass of existing access controls

## Next Steps
- Proceed to Phase 3 (Summary Report) - can run in parallel if team available
- After both Phase 2 & 3, proceed to Phase 4 (Sorting)
