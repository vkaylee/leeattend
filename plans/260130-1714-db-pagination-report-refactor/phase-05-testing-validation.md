# Phase 5: Testing & Validation

## Context Links
- [Plan Overview](./plan.md)
- [Scout Report](../reports/scout-260130-1711-attendance-report-architecture.md)
- Test location: `apiserver/tests/`

## Overview
- **Priority**: P1
- **Status**: Pending
- **Effort**: 1h
- **Depends on**: Phases 1-4

Comprehensive testing to validate refactoring correctness and performance.

## Key Insights
1. Existing tests must continue to pass (backward compatibility)
2. New tests needed for pagination edge cases
3. Performance benchmarking critical for acceptance criteria
4. Data consistency between summary and daily reports

## Requirements

### Functional Tests
- T1: Pagination returns correct page count
- T2: Total count matches actual employee count
- T3: Filtering by employee_ids works with pagination
- T4: Empty results handled correctly
- T5: Summary report totals match aggregated daily report

### Performance Tests
- P1: Response time < 500ms for page 1 with 700 employees
- P2: SQL queries show LIMIT clause
- P3: Memory usage bounded (not scaling with total employees)

### Edge Cases
- E1: First page, last page, middle page
- E2: Page beyond total (empty result)
- E3: Limit = 1, Limit = 1000
- E4: Filter to single employee
- E5: Date range with no data

## Test Plan

### Unit Tests: Repository Layer

```rust
// tests/repositories/employee_repository_pagination_test.rs

#[tokio::test]
async fn test_get_paginated_employee_ids_returns_correct_count() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    // Setup: Create 50 employees
    let workspace = factory.create_workspace().await;
    for i in 0..50 {
        factory.create_employee_in_workspace(workspace.id, &format!("Employee {}", i)).await;
    }

    // Test page 1
    let (employees, total) = employee_repository::get_paginated_employee_ids(
        pool, workspace.id, None, 1, 20, false
    ).await.unwrap();

    assert_eq!(employees.len(), 20);
    assert_eq!(total, 50);
}

#[tokio::test]
async fn test_get_paginated_employee_ids_with_filter() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace = factory.create_workspace().await;
    let emp1 = factory.create_employee_in_workspace(workspace.id, "Alice").await;
    let emp2 = factory.create_employee_in_workspace(workspace.id, "Bob").await;
    let _emp3 = factory.create_employee_in_workspace(workspace.id, "Charlie").await;

    let filter = vec![emp1.id, emp2.id];
    let (employees, total) = employee_repository::get_paginated_employee_ids(
        pool, workspace.id, Some(&filter), 1, 20, false
    ).await.unwrap();

    assert_eq!(employees.len(), 2);
    assert_eq!(total, 2);
}

#[tokio::test]
async fn test_get_paginated_employee_ids_sort_order() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace = factory.create_workspace().await;
    factory.create_employee_in_workspace(workspace.id, "Zoe").await;
    factory.create_employee_in_workspace(workspace.id, "Alice").await;
    factory.create_employee_in_workspace(workspace.id, "Bob").await;

    // ASC
    let (employees_asc, _) = employee_repository::get_paginated_employee_ids(
        pool, workspace.id, None, 1, 20, false
    ).await.unwrap();
    assert_eq!(employees_asc[0].full_name, "Alice");

    // DESC
    let (employees_desc, _) = employee_repository::get_paginated_employee_ids(
        pool, workspace.id, None, 1, 20, true
    ).await.unwrap();
    assert_eq!(employees_desc[0].full_name, "Zoe");
}
```

### Unit Tests: Time Entry Batch

```rust
// tests/repositories/time_entry_batch_test.rs

#[tokio::test]
async fn test_get_workspace_time_entries_batch_groups_by_employee() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace = factory.create_workspace().await;
    let emp1 = factory.create_employee_in_workspace(workspace.id, "Alice").await;
    let emp2 = factory.create_employee_in_workspace(workspace.id, "Bob").await;

    // Create time entries
    factory.create_time_entry(emp1.id, workspace.id, today_morning()).await;
    factory.create_time_entry(emp1.id, workspace.id, yesterday_morning()).await;
    factory.create_time_entry(emp2.id, workspace.id, today_morning()).await;

    let result = time_entry_repository::get_workspace_time_entries_batch(
        pool,
        &[emp1.id, emp2.id],
        week_ago_utc(),
        now_utc(),
    ).await.unwrap();

    assert_eq!(result.get(&emp1.id).unwrap().len(), 2);
    assert_eq!(result.get(&emp2.id).unwrap().len(), 1);
}

#[tokio::test]
async fn test_get_workspace_time_entries_batch_empty_ids() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();

    let result = time_entry_repository::get_workspace_time_entries_batch(
        pool, &[], now_utc(), now_utc()
    ).await.unwrap();

    assert!(result.is_empty());
}
```

### Integration Tests: Report Service

```rust
// tests/services/report_service_pagination_test.rs

#[tokio::test]
async fn test_daily_report_pagination_limits_queries() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    // Setup: 100 employees with time entries
    let workspace = factory.create_workspace().await;
    for i in 0..100 {
        let emp = factory.create_employee_in_workspace(workspace.id, &format!("Emp {}", i)).await;
        factory.create_time_entry(emp.id, workspace.id, today_morning()).await;
    }

    // Request page 1, limit 20
    let result = ReportService::get_daily_attendance_report(
        pool,
        workspace.id,
        today(),
        today(),
        Some(1),
        Some(20),
        None,
    ).await.unwrap();

    // Should return 20 records (20 employees x 1 day)
    assert_eq!(result.data.len(), 20);
    assert_eq!(result.total, 100);  // Total employees
    assert_eq!(result.page, 1);
    assert_eq!(result.limit, 20);
}

#[tokio::test]
async fn test_summary_report_matches_daily_report_totals() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace = factory.create_workspace().await;
    let emp = factory.create_employee_in_workspace(workspace.id, "Alice").await;

    // Create time entries for 5 days
    for day in 0..5 {
        let date = today() - chrono::Duration::days(day);
        factory.create_complete_time_entry(emp.id, workspace.id, date, 8, 17).await;
    }

    // Get daily report
    let daily = ReportService::get_daily_attendance_report(
        pool, workspace.id, today() - chrono::Duration::days(4), today(),
        Some(1), Some(100), Some(vec![emp.id])
    ).await.unwrap();

    // Get summary report
    let summary = ReportService::get_workspace_attendance_summary(
        pool, workspace.id, today() - chrono::Duration::days(4), today(),
        None, Some(1), Some(100), Some(vec![emp.id])
    ).await.unwrap();

    // Verify consistency
    let daily_total: i32 = daily.data.iter().map(|d| d.work_units).sum();
    assert_eq!(daily_total, summary.data[0].total_work_units);
}

#[tokio::test]
async fn test_pagination_beyond_total_returns_empty() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace = factory.create_workspace().await;
    for i in 0..5 {
        factory.create_employee_in_workspace(workspace.id, &format!("Emp {}", i)).await;
    }

    let result = ReportService::get_workspace_attendance_summary(
        pool, workspace.id, today(), today(),
        None, Some(10), Some(20), None  // Page 10 with only 5 employees
    ).await.unwrap();

    assert!(result.data.is_empty());
    assert_eq!(result.total, 5);
}
```

### Performance Tests

```rust
// tests/performance/report_pagination_perf_test.rs

#[tokio::test]
#[ignore] // Run manually with --ignored
async fn test_response_time_with_700_employees() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    // Setup: 700 employees with time entries
    let workspace = factory.create_workspace().await;
    for i in 0..700 {
        let emp = factory.create_employee_in_workspace(workspace.id, &format!("Emp {:04}", i)).await;
        factory.create_time_entry(emp.id, workspace.id, today_morning()).await;
    }

    // Measure response time
    let start = std::time::Instant::now();

    let result = ReportService::get_workspace_attendance_summary(
        pool, workspace.id, today(), today(),
        None, Some(1), Some(20), None
    ).await.unwrap();

    let duration = start.elapsed();

    // Assertions
    assert_eq!(result.data.len(), 20);
    assert_eq!(result.total, 700);
    assert!(duration.as_millis() < 500, "Response time {} ms exceeds 500ms", duration.as_millis());

    println!("Response time: {} ms", duration.as_millis());
}
```

## Validation Checklist

### Pre-Implementation Baseline
- [ ] Record current response time for existing implementation
- [ ] Document current SQL query patterns (via EXPLAIN ANALYZE)
- [ ] Capture memory usage baseline

### Post-Implementation Validation
- [ ] All existing tests pass
- [ ] New pagination tests pass
- [ ] Response time < 500ms verified
- [ ] SQL shows LIMIT clause (check via logs/tracing)
- [ ] Memory usage doesn't grow with employee count

### Manual Testing
- [ ] Test via API endpoint with curl/Postman
- [ ] Verify page navigation works correctly
- [ ] Verify sorting works as documented
- [ ] Test with production-like data volume

## Todo List
- [ ] Create test factory helpers (if not exist)
- [ ] Write repository unit tests
- [ ] Write service integration tests
- [ ] Write performance benchmark test
- [ ] Run full test suite
- [ ] Document any test failures and fixes
- [ ] Update CI configuration if needed

## Success Criteria
1. All existing tests pass
2. All new tests pass
3. Performance benchmark meets < 500ms requirement
4. No regressions in data accuracy

## Risk Assessment
- **Low Risk**: Test isolation via TestDatabase
- **Medium Risk**: Performance test may need dedicated environment
- **Mitigation**: Mark perf tests as `#[ignore]` for regular CI

## Security Considerations
- Tests use isolated databases
- No production data in tests
- Sensitive test data cleaned up

## Commands

Run tests (via run-tests.sh as per CLAUDE.md rules):
```bash
./run-tests.sh --api pagination
./run-tests.sh --api report
```

Run performance test manually:
```bash
./run-tests.sh --api response_time_with_700
```

## Next Steps
- After all tests pass, mark implementation as complete
- Update API documentation if response structure changed
- Consider monitoring/alerting for production response times
