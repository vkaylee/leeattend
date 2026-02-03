# Phase 3: Service Layer - Summary Report Refactoring

## Context Links
- [Phase 1: Repository Layer](./phase-01-repository-layer.md)
- [Phase 2: Daily Report](./phase-02-service-layer-daily-report.md)
- [Scout Report](../reports/scout-260130-1711-attendance-report-architecture.md)
- Related: `apiserver/src/services/report_service.rs:251-515`

## Overview
- **Priority**: P1
- **Status**: Pending
- **Effort**: 1.5h
- **Depends on**: Phase 1

Refactor `get_workspace_attendance_summary()` using same pattern as daily report. CRITICAL: Maintain calculation consistency between summary and detail reports (see code comment at lines 241-248).

## Key Insights
1. Summary report MUST use same AttendanceCalculator as daily report (data consistency requirement)
2. Same N+1 problem exists for time entry fetching
3. Sorting by calculated fields (work_hours, leave_days) requires special handling
4. Leave days fetched via separate batch query (already optimized)

## Requirements

### Functional
- F1: Use `get_paginated_employee_ids()` for DB-level pagination
- F2: Fetch time entries in batch (not per-employee)
- F3: Maintain SAME calculation logic as daily report
- F4: Handle sorting edge cases (see Phase 4)

### Non-Functional
- NF1: Response time < 500ms for page 1
- NF2: Memory bounded by batch size
- NF3: Data consistency with daily report

## Architecture

### Before
```
1. get_workspace_schedules(ALL employees)
2. for each employee: get_employee_time_entries (N+1)
3. calculate + aggregate per employee
4. sort all results
5. skip/take pagination
```

### After
```
1. get_paginated_employee_ids(page, limit, sort_by_name?)
2. get_workspace_schedules(filtered_ids)
3. get_workspace_time_entries_batch(filtered_ids)
4. calculate + aggregate per employee
5. sort within batch (if sort_by_name, already sorted from DB)
```

## Related Code Files

### Files to Modify
- `apiserver/src/services/report_service.rs` (refactor `get_workspace_attendance_summary`)

### Files to Reference
- Phase 2 implementation for consistent patterns

## Implementation Steps

### Step 1: Refactor Implementation

Replace lines 251-515 in `report_service.rs`:

```rust
#[allow(clippy::too_many_arguments)]
pub async fn get_workspace_attendance_summary(
    pool: &PgPool,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    sort: Option<AttendanceReportSort>,
    page: Option<i64>,
    limit: Option<i64>,
    employee_ids: Option<Vec<Uuid>>,
) -> Result<PaginatedResponse<AttendanceSummary>, StatusCode> {
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

    // Check if sorting by name (can be done at DB level)
    let sort_by_name = matches!(
        sort,
        Some(AttendanceReportSort::NameAsc) | Some(AttendanceReportSort::NameDesc) | None
    );

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

    // 2. Get employee IDs and names
    let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();
    let emp_names: HashMap<Uuid, String> = paginated_employees
        .iter()
        .map(|e| (e.id, e.full_name.clone()))
        .collect();

    // 3. Get schedules for paginated employees
    let schedules = roster_repository::get_workspace_schedules(
        pool,
        workspace_id,
        start_date,
        end_date,
        Some(&emp_ids),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 4. Convert dates
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

    // 5. BATCH fetch time entries
    let entries_map = time_entry_repository::get_workspace_time_entries_batch(
        pool,
        &emp_ids,
        start_dt,
        end_dt,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 6. Batch fetch leave data (existing optimized query)
    use sqlx::Row;
    let leave_data: HashMap<Uuid, i64> = sqlx::query(
        r#"
        SELECT
            lr.employee_id,
            SUM(lr.end_date - lr.start_date + 1)::bigint as leave_days
        FROM leave_requests lr
        WHERE lr.employee_id = ANY($1)
          AND lr.status = 'approved'
          AND lr.start_date >= $2::date
          AND lr.end_date <= $3::date
        GROUP BY lr.employee_id
        "#,
    )
    .bind(&emp_ids)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching leave data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .map(|row| (
        row.get::<Uuid, _>("employee_id"),
        row.get::<i64, _>("leave_days"),
    ))
    .collect();

    // 7. Calculate work units (SAME logic as daily report - CRITICAL)
    let mut employee_totals: HashMap<Uuid, i32> = HashMap::new();

    for emp_sched in schedules {
        let entries = entries_map.get(&emp_sched.employee_id).cloned().unwrap_or_default();
        let mut employee_total_units = 0i32;

        for daily in &emp_sched.schedules {
            let date = daily.date;

            let day_entries: Vec<_> = entries
                .iter()
                .filter(|e| e.punch_in_time.with_timezone(&tz).date_naive() == date)
                .collect();

            let actual_punch_in = day_entries.first().map(|e| e.punch_in_time.with_timezone(&tz));
            let actual_punch_out = day_entries
                .last()
                .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&tz)));

            if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                let actual_break_minutes: i32 = day_entries
                    .iter()
                    .filter(|e| e.entry_type == "break")
                    .map(|e| e.duration_minutes.unwrap_or(0))
                    .sum();

                let start_dt = daily.start_time
                    .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());
                let end_dt = daily.end_time
                    .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());

                let mut rules = PolicyResolver::resolve_attendance_rules(
                    pool,
                    emp_sched.employee_id,
                    workspace.id,
                    date,
                )
                .await
                .unwrap_or(PolicyRules {
                    grace_period_minutes: daily.early_punch_in_grace_period_minutes.unwrap_or(0),
                    authorized_early_leave_minutes: 0,
                    is_early_leave_paid: false,
                    early_arrival_strategy: workspace.early_arrival_strategy,
                    is_break_auto_deduct: daily.is_break_auto_deduct.unwrap_or(false),
                    is_break_overtime: false,
                    allow_early_ot: daily.allow_early_ot.unwrap_or(false),
                });

                // Override with shift-specific settings
                if let Some(gp) = daily.early_punch_in_grace_period_minutes {
                    rules.grace_period_minutes = gp;
                }
                if let Some(bad) = daily.is_break_auto_deduct {
                    rules.is_break_auto_deduct = bad;
                }
                if let Some(aeo) = daily.allow_early_ot {
                    rules.allow_early_ot = aeo;
                }

                let (shift_start, shift_end) = if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                    if daily.end_time < daily.start_time {
                        end += chrono::Duration::days(1);
                    }
                    (Some(start), Some(end))
                } else {
                    (None, None)
                };

                let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                    punch_in,
                    punch_out,
                    shift_start,
                    shift_end,
                    daily.break_duration_minutes.unwrap_or(0),
                    actual_break_minutes,
                    workspace.work_unit_minutes,
                    &workspace.rounding_policy,
                    &rules,
                );

                employee_total_units += std_units + ot_units;
            }
        }

        employee_totals.insert(emp_sched.employee_id, employee_total_units);
    }

    // 8. Build results
    let mut results: Vec<AttendanceSummary> = paginated_employees
        .iter()
        .map(|emp| AttendanceSummary {
            employee_id: emp.id,
            employee_name: emp.full_name.clone(),
            total_work_units: *employee_totals.get(&emp.id).unwrap_or(&0),
            total_leave_days: leave_data.get(&emp.id).copied().unwrap_or(0),
        })
        .collect();

    // 9. Sort within batch (for calculated fields only)
    // Note: Name sorting is already handled by DB ORDER BY
    if let Some(s) = &sort {
        match s {
            AttendanceReportSort::NameAsc | AttendanceReportSort::NameDesc => {
                // Already sorted from DB - only reverse if DESC
                if matches!(s, AttendanceReportSort::NameDesc) {
                    results.reverse();
                }
            }
            AttendanceReportSort::WorkHoursAsc => {
                results.sort_by(|a, b| a.total_work_units.cmp(&b.total_work_units))
            }
            AttendanceReportSort::WorkHoursDesc => {
                results.sort_by(|a, b| b.total_work_units.cmp(&a.total_work_units))
            }
            AttendanceReportSort::LeaveDaysAsc => {
                results.sort_by(|a, b| a.total_leave_days.cmp(&b.total_leave_days))
            }
            AttendanceReportSort::LeaveDaysDesc => {
                results.sort_by(|a, b| b.total_leave_days.cmp(&a.total_leave_days))
            }
        }
    }

    // 10. Return response (NO in-memory skip/take)
    Ok(PaginatedResponse {
        data: results,
        total: total_employees as usize,
        page,
        limit,
    })
}
```

### Step 2: Add Required Imports
```rust
use std::collections::HashMap;
use crate::repositories::employee_repository;
```

## Todo List
- [ ] Add HashMap import (if not present)
- [ ] Add employee_repository import
- [ ] Replace pagination logic with DB-level
- [ ] Replace time entry fetch with batch
- [ ] Keep calculation logic identical to daily report
- [ ] Optimize leave query to use emp_ids filter
- [ ] Handle sorting appropriately
- [ ] Test data consistency with daily report

## Success Criteria
1. Response time < 500ms for page 1
2. Only 1 time_entries query per request
3. Summary totals match aggregated daily report data
4. Name sorting works correctly
5. Calculated field sorting works within batch

## Risk Assessment
- **Medium Risk**: Sorting by calculated fields only accurate within current page
- **Mitigation**: See Phase 4 for proper handling
- **Low Risk**: Calculation logic unchanged, consistency preserved

## Security Considerations
- All queries filter by employee_ids (from pagination)
- Leave query scoped to paginated employees only
- No escalation of data access

## Next Steps
- Proceed to Phase 4 (Sorting) after both Phase 2 & 3 complete
- Phase 4 addresses edge cases for calculated field sorting
