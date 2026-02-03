# Phase 4: Sorting Edge Cases & Optimization

## Context Links
- [Phase 2: Daily Report](./phase-02-service-layer-daily-report.md)
- [Phase 3: Summary Report](./phase-03-service-layer-summary-report.md)
- Related: `apiserver/src/models.rs` (AttendanceReportSort enum)

## Overview
- **Priority**: P2
- **Status**: Pending
- **Effort**: 1h
- **Depends on**: Phase 2 & 3 conceptually, but can be implemented in parallel

Address sorting edge cases when paginating at database level.

## Key Insights

### The Sorting Problem
With database pagination, we can only sort by columns that exist IN the database BEFORE calculation.

| Sort Field | Can Do at DB Level? | Notes |
|------------|---------------------|-------|
| Name (Asc/Desc) | **YES** | `ORDER BY full_name` |
| Work Hours (Asc/Desc) | **NO** | Calculated field |
| Leave Days (Asc/Desc) | **PARTIAL** | Could pre-aggregate in subquery |

### Trade-offs

**Option A: Accept Limitation**
- Sort by calculated fields only affects order within current page
- Document behavior clearly in API
- Simplest implementation

**Option B: Two-Phase Query (Complex)**
- Phase 1: Calculate work units for ALL employees (expensive)
- Phase 2: Sort + paginate
- Defeats purpose of DB pagination for large datasets

**Option C: Pre-computed Cache Table**
- Store daily/monthly aggregates in `attendance_daily_summaries`
- Sort and paginate from cached data
- Best performance but requires cache maintenance

## Requirements

### Functional
- F1: Name sorting MUST be accurate across all pages
- F2: Calculated field sorting behavior documented
- F3: Optional warning in response for calculated sorts

### Non-Functional
- NF1: Name sort adds no performance overhead
- NF2: Calculated sort doesn't break pagination

## Architecture

### Recommended Approach: Option A (Accept Limitation)

```
If sort = Name:
  -> ORDER BY full_name in SQL
  -> Accurate globally

If sort = WorkHours or LeaveDays:
  -> Fetch paginated employees (by name)
  -> Calculate units for batch
  -> Sort within batch ONLY
  -> Add warning flag in response (optional)
```

### Enhanced Repository Function

Update `get_paginated_employee_ids()` to support sort direction:

```rust
pub async fn get_paginated_employee_ids(
    pool: &PgPool,
    workspace_id: Uuid,
    filter_ids: Option<&Vec<Uuid>>,
    page: i64,
    limit: i64,
    sort_desc: bool,  // NEW: true for DESC, false for ASC
) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error> {
    let order = if sort_desc { "DESC" } else { "ASC" };

    let query = format!(
        r#"SELECT id, full_name FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))
           ORDER BY full_name {}
           LIMIT $3 OFFSET $4"#,
        order
    );

    // ... execute query ...
}
```

## Implementation Steps

### Step 1: Update Repository Function

Add `sort_desc` parameter to `get_paginated_employee_ids()`:

```rust
pub async fn get_paginated_employee_ids(
    pool: &PgPool,
    workspace_id: Uuid,
    filter_ids: Option<&Vec<Uuid>>,
    page: i64,
    limit: i64,
    sort_desc: bool,
) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error> {
    // Count query (unchanged)
    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))"#
    )
    .bind(workspace_id)
    .bind(filter_ids)
    .fetch_one(pool)
    .await?;

    // Data query with dynamic ORDER BY
    let offset = (page - 1) * limit;
    let order_dir = if sort_desc { "DESC" } else { "ASC" };

    let sql = format!(
        r#"SELECT id, full_name FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))
           ORDER BY full_name {}
           LIMIT $3 OFFSET $4"#,
        order_dir
    );

    let employees = sqlx::query_as::<_, EmployeeBasicInfo>(&sql)
        .bind(workspace_id)
        .bind(filter_ids)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok((employees, total))
}
```

### Step 2: Update Service Layer Calls

In `get_workspace_attendance_summary()`:

```rust
// Determine sort direction for name
let sort_desc = matches!(sort, Some(AttendanceReportSort::NameDesc));

let (paginated_employees, total_employees) = employee_repository::get_paginated_employee_ids(
    pool,
    workspace_id,
    employee_ids.as_ref(),
    page,
    limit,
    sort_desc,  // Pass sort direction
)
.await
.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
```

### Step 3: Document API Behavior (Optional Enhancement)

Add response metadata to indicate sort limitation:

```rust
#[derive(Debug, Serialize)]
pub struct PaginatedResponseWithMeta<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_warning: Option<String>,
}
```

Usage:
```rust
let sort_warning = match sort {
    Some(AttendanceReportSort::WorkHoursAsc | AttendanceReportSort::WorkHoursDesc |
         AttendanceReportSort::LeaveDaysAsc | AttendanceReportSort::LeaveDaysDesc) => {
        Some("Sort by calculated fields is accurate within current page only".to_string())
    }
    _ => None
};
```

### Alternative: Leave Days Sorting at DB Level (Partial Solution)

Leave days CAN be sorted at DB level with a subquery:

```sql
SELECT e.id, e.full_name,
       COALESCE(l.leave_days, 0) as leave_days
FROM employees e
LEFT JOIN (
    SELECT employee_id, SUM(end_date - start_date + 1) as leave_days
    FROM leave_requests
    WHERE status = 'approved'
      AND start_date >= $date_from AND end_date <= $date_to
    GROUP BY employee_id
) l ON e.id = l.employee_id
WHERE e.workspace_id = $1 AND e.is_active = true
ORDER BY leave_days DESC
LIMIT $2 OFFSET $3
```

**Trade-off**: More complex query but accurate sorting. Consider if leave_days sorting is important use case.

## Todo List
- [ ] Add `sort_desc` parameter to `get_paginated_employee_ids()`
- [ ] Update service layer calls with sort direction
- [ ] Decide: Add sort warning to response? (recommended)
- [ ] Consider: Leave days DB-level sorting? (optional)
- [ ] Update API documentation
- [ ] Add unit tests for sort behavior

## Success Criteria
1. Name sorting accurate across all pages
2. DESC/ASC works correctly for name sorting
3. Calculated field sorting works within page
4. No performance regression
5. Behavior documented

## Risk Assessment
- **Low Risk**: Simple parameter addition
- **Medium Risk**: User confusion if expecting global sort for calculated fields
- **Mitigation**: Clear documentation and optional warning in response

## Security Considerations
- Sort parameter validated against enum (no SQL injection)
- No new data exposure

## Unresolved Questions
1. Should we add sort warning to API response?
2. Is leave_days sorting important enough for DB-level implementation?
3. Future: Should we implement pre-computed aggregates for better sort support?

## Next Steps
- After implementation, proceed to Phase 5 (Testing)
- Consider future enhancement: caching layer for calculated sorts
