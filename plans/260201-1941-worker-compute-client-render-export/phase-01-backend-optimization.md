# Phase 1: Backend Optimization

**Priority**: P0
**Status**: Pending
**Estimated**: Medium complexity

---

## Context Links
- [Plan Overview](./plan.md)
- [Scout Report](../reports/scout-260201-1941-export-system-architecture.md)

---

## Overview

Optimize batch processing performance by increasing BATCH_SIZE and fixing N+1 query patterns.

---

## Key Insights

- Current BATCH_SIZE = 20 causes excessive job management overhead
- Policy rules are fetched per-employee (N+1 query pattern)
- Target: < 100ms for 200 records

---

## Requirements

### Functional
- Increase BATCH_SIZE from 20 to 200
- Batch-load policy rules for all employees in one query
- Maintain calculation accuracy

### Non-Functional
- Query time < 50ms for 200 employees
- Memory usage increase < 10x (proportional)

---

## Related Code Files

### Modify
- `apiserver/src/services/report_coordinator.rs:23` - BATCH_SIZE constant

### Create
- `apiserver/src/domain/policy/batch_resolver.rs` - Batch policy resolution

### Analyze
- `apiserver/src/worker/handlers/attendance.rs` - Find N+1 query patterns

---

## Implementation Steps

### Step 1: Update BATCH_SIZE
```rust
// apiserver/src/services/report_coordinator.rs
const BATCH_SIZE: usize = 200; // was 20
```

### Step 2: Identify N+1 Query Pattern
Search for policy fetching in attendance handler:
```rust
// Current (likely per-employee loop)
for employee in employees {
    let rules = get_attendance_rules(employee.id); // N queries!
}

// Target (batch)
let rules_map = get_attendance_rules_batch(&employee_ids); // 1 query
```

### Step 3: Create Batch Resolver

```rust
// apiserver/src/domain/policy/batch_resolver.rs

use std::collections::HashMap;
use uuid::Uuid;

pub struct BatchPolicyResolver;

impl BatchPolicyResolver {
    /// Resolve attendance rules for multiple employees in single query
    pub async fn resolve_attendance_rules_batch(
        pool: &PgPool,
        employee_ids: &[Uuid],
        workspace_id: Uuid,
    ) -> Result<HashMap<Uuid, AttendanceRules>, Error> {
        // Single query with IN clause
        let policies = sqlx::query_as!(
            PolicyRow,
            r#"
            SELECT
                e.id as employee_id,
                s.id as shift_id,
                s.start_time, s.end_time, s.grace_period_minutes,
                s.early_leave_threshold_minutes,
                s.overtime_threshold_minutes,
                COALESCE(sa.effective_date, '1970-01-01') as effective_date
            FROM employees e
            LEFT JOIN shift_assignments sa ON sa.employee_id = e.id
                AND sa.effective_date <= CURRENT_DATE
            LEFT JOIN shifts s ON s.id = sa.shift_id
            WHERE e.id = ANY($1)
                AND e.workspace_id = $2
            ORDER BY e.id, sa.effective_date DESC
            "#,
            employee_ids,
            workspace_id
        )
        .fetch_all(pool)
        .await?;

        // Group by employee_id, take latest assignment
        let mut result = HashMap::with_capacity(employee_ids.len());
        for row in policies {
            result.entry(row.employee_id)
                .or_insert_with(|| AttendanceRules::from_row(&row));
        }

        Ok(result)
    }
}
```

### Step 4: Integrate into Attendance Handler

```rust
// apiserver/src/worker/handlers/attendance.rs

// Before processing loop
let rules_map = BatchPolicyResolver::resolve_attendance_rules_batch(
    &pool,
    &employee_ids,
    workspace_id
).await?;

// In processing loop
for employee in &employees {
    let rules = rules_map.get(&employee.id)
        .unwrap_or(&default_rules);
    // ... calculate attendance using rules
}
```

---

## Todo List

- [ ] Update BATCH_SIZE to 200 in report_coordinator.rs
- [ ] Analyze attendance.rs for N+1 query patterns
- [ ] Create batch_resolver.rs module
- [ ] Add batch query for policy rules
- [ ] Integrate batch resolver in attendance handler
- [ ] Run existing tests to verify no regression
- [ ] Benchmark: compare 20 vs 200 batch performance

---

## Success Criteria

- [ ] BATCH_SIZE = 200 in production code
- [ ] Single SQL query fetches policies for all 200 employees
- [ ] Processing time < 100ms for 200 employees
- [ ] All existing report tests pass

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Memory spike | Medium | Monitor with larger batches in staging |
| Query timeout | Low | Add query timeout, index optimization |
| Calculation drift | High | Compare results: old vs new on same data |

---

## Security Considerations

- Workspace isolation: Batch query must filter by workspace_id
- Employee access: Verify caller has access to all employees in batch
