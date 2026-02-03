# Phase 1: Repository Layer Upgrade

## Context Links
- [Scout Report](../reports/scout-260130-1711-attendance-report-architecture.md)
- Related: `apiserver/src/repositories/employee_repository.rs`
- Related: `apiserver/src/repositories/time_entry_repository.rs`

## Overview
- **Priority**: P1 (Blocking for Phase 2-3)
- **Status**: Pending
- **Effort**: 1h

Create paginated employee ID fetching and batch time entry functions to enable database-level pagination.

## Key Insights
1. Current `get_active_employees()` returns ALL employees without pagination support
2. Time entries fetched per-employee (N+1 pattern) - need batch fetch
3. Existing `get_all_employees()` has pagination but returns full Employee struct (overkill for reports)

## Requirements

### Functional
- F1: New function `get_paginated_employee_ids()` returning (Vec<Uuid>, total_count)
- F2: New function `get_workspace_time_entries_batch()` returning HashMap<Uuid, Vec<TimeEntry>>
- F3: Support optional employee_ids filter in both functions

### Non-Functional
- NF1: Single SQL query for paginated IDs (no N+1)
- NF2: Single SQL query for batch time entries
- NF3: Maintain existing error handling patterns

## Architecture

### Data Flow
```
get_paginated_employee_ids(workspace_id, filter_ids, page, limit)
    │
    ├─> COUNT(*) query (total)
    │
    └─> SELECT id, full_name FROM employees
        WHERE workspace_id = $1 AND is_active = true
        AND ($2::uuid[] IS NULL OR id = ANY($2))
        ORDER BY full_name
        LIMIT $3 OFFSET $4

get_workspace_time_entries_batch(employee_ids, from, to)
    │
    └─> SELECT * FROM time_entries
        WHERE employee_id = ANY($1)
        AND punch_in_time >= $2 AND punch_in_time <= $3
        ORDER BY employee_id, punch_in_time DESC
```

## Related Code Files

### Files to Modify
- `apiserver/src/repositories/employee_repository.rs` (add new function)
- `apiserver/src/repositories/time_entry_repository.rs` (add batch function)

### Files to Create
- None

## Implementation Steps

### Step 1: Add `get_paginated_employee_ids()` to employee_repository.rs

```rust
/// Paginated minimal employee info for reports
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmployeeBasicInfo {
    pub id: Uuid,
    pub full_name: String,
}

/// Get paginated employee IDs with optional filter
/// Returns: (employees, total_count)
pub async fn get_paginated_employee_ids(
    pool: &PgPool,
    workspace_id: Uuid,
    filter_ids: Option<&Vec<Uuid>>,
    page: i64,
    limit: i64,
) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error> {
    // Count query
    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))"#
    )
    .bind(workspace_id)
    .bind(filter_ids)
    .fetch_one(pool)
    .await?;

    // Data query
    let offset = (page - 1) * limit;
    let employees = sqlx::query_as::<_, EmployeeBasicInfo>(
        r#"SELECT id, full_name FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))
           ORDER BY full_name
           LIMIT $3 OFFSET $4"#
    )
    .bind(workspace_id)
    .bind(filter_ids)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((employees, total))
}
```

### Step 2: Add `get_workspace_time_entries_batch()` to time_entry_repository.rs

```rust
use std::collections::HashMap;

/// Batch fetch time entries for multiple employees
/// Returns HashMap keyed by employee_id for O(1) lookup
pub async fn get_workspace_time_entries_batch(
    pool: &PgPool,
    employee_ids: &[Uuid],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<HashMap<Uuid, Vec<TimeEntry>>, sqlx::Error> {
    if employee_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let entries = sqlx::query_as::<_, TimeEntry>(
        r#"SELECT * FROM time_entries
           WHERE employee_id = ANY($1)
           AND punch_in_time >= $2 AND punch_in_time <= $3
           ORDER BY employee_id, punch_in_time DESC"#
    )
    .bind(employee_ids)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    // Group by employee_id
    let mut result: HashMap<Uuid, Vec<TimeEntry>> = HashMap::new();
    for entry in entries {
        result.entry(entry.employee_id).or_default().push(entry);
    }

    Ok(result)
}
```

## Todo List
- [ ] Add `EmployeeBasicInfo` struct to models or repository
- [ ] Implement `get_paginated_employee_ids()`
- [ ] Implement `get_workspace_time_entries_batch()`
- [ ] Add unit tests for both functions
- [ ] Verify SQL execution plan shows index usage

## Success Criteria
1. Both functions compile without warnings
2. `get_paginated_employee_ids` returns correct count with filter
3. `get_workspace_time_entries_batch` returns grouped HashMap
4. No N+1 queries in batch function

## Risk Assessment
- **Low Risk**: Simple CRUD operations
- **Watch**: Ensure `employee_id` index exists on time_entries table

## Security Considerations
- Both functions require workspace_id filtering (multi-tenancy)
- employee_ids filter prevents unauthorized data access

## Next Steps
- After completion, proceed to Phase 2 (Daily Report) and Phase 3 (Summary Report)
