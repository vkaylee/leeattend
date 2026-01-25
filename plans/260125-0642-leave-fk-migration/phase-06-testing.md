# Phase 6: Testing & Validation

## Context Links
- Plan: `./plan.md`
- Test utilities: `scripts/test-utils.sh`
- API tests: `apiserver/tests/leave_policy_api_tests.rs`
- Repo tests: `apiserver/tests/leave_policy_repo_tests.rs`

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 1h
- **Dependencies**: All previous phases

Comprehensive testing and validation of the FK migration.

## Key Insights
- Existing tests use string-based `leave_type`
- Need to update test fixtures and assertions
- Integration tests require database with migrated schema

## Requirements

### Functional
- All existing tests pass with updated code
- New tests for FK validation
- Migration rollback tested

### Non-Functional
- Test isolation maintained
- CI/CD pipeline passes

## Related Code Files

### Files to Modify
- `apiserver/tests/leave_policy_api_tests.rs`
- `apiserver/tests/leave_policy_repo_tests.rs`
- `apiserver/tests/common/factory.rs` (if leave factories exist)

## Implementation Steps

### Step 1: Update Test Factory (if exists)

If `factory.rs` has leave-related helpers, update them:

```rust
// In factory.rs, add helper to get or create leave type
impl TestFactory {
    pub async fn get_vacation_leave_type(&self, workspace_id: Uuid) -> Uuid {
        sqlx::query_scalar!(
            "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
            workspace_id
        )
        .fetch_one(&self.pool)
        .await
        .expect("Vacation leave type should exist from migration")
    }

    pub async fn create_leave_request(
        &self,
        employee_id: Uuid,
        leave_type_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> LeaveRequest {
        let request = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id,
            leave_type_id,
            start_date,
            end_date,
            reason: Some("Test leave".to_string()),
            status: LeaveStatus::Pending,
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        sqlx::query!(
            r#"
            INSERT INTO leave_requests
                (id, employee_id, leave_type_id, start_date, end_date, reason, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            request.id,
            request.employee_id,
            request.leave_type_id,
            request.start_date,
            request.end_date,
            request.reason,
            request.status as LeaveStatus,
            request.created_at,
            request.updated_at,
        )
        .execute(&self.pool)
        .await
        .expect("Failed to create leave request");

        request
    }
}
```

### Step 2: Update leave_policy_repo_tests.rs

Update tests to use UUID:

```rust
#[tokio::test]
async fn test_create_leave_balance() {
    let db = TestDatabase::new().await;
    let factory = TestFactory::new(db.pool());

    let workspace = factory.create_workspace("Test WS").await;
    let employee = factory.create_employee(&workspace.id, "John Doe").await;
    let leave_type_id = factory.get_vacation_leave_type(workspace.id).await;

    let balance = LeaveBalance {
        id: Uuid::new_v4(),
        employee_id: employee.id,
        leave_type_id,
        year: 2026,
        entitlement: 12,
        accrued_amount: rust_decimal::Decimal::ZERO,
        carried_over: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    leave_balance_repository::upsert_leave_balance(&db.pool(), &balance)
        .await
        .expect("Should insert balance");

    let balances = leave_balance_repository::get_employee_balances(
        &db.pool(),
        employee.id,
        2026,
    )
    .await
    .expect("Should fetch balances");

    assert_eq!(balances.len(), 1);
    assert_eq!(balances[0].leave_type_id, leave_type_id);
    assert_eq!(balances[0].entitlement, 12);
}
```

### Step 3: Update leave_policy_api_tests.rs

Update API tests:

```rust
#[tokio::test]
async fn test_submit_leave_request_with_type_id() {
    let db = TestDatabase::new().await;
    let factory = TestFactory::new(db.pool());

    let workspace = factory.create_workspace("Test WS").await;
    let user = factory.create_user("test@example.com").await;
    let employee = factory.create_employee(&workspace.id, "John Doe").await;
    factory.add_member(&workspace.id, &user.id, MemberRole::Admin).await;

    let leave_type_id = factory.get_vacation_leave_type(workspace.id).await;

    let app = create_test_app(db.pool()).await;
    let token = create_test_token(&user);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/submit", workspace.id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "employee_id": employee.id,
                    "leave_type_id": leave_type_id,
                    "start_date": "2026-02-01",
                    "end_date": "2026-02-03",
                    "reason": "Vacation"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body: LeaveRequest = response_body(response).await;
    assert_eq!(body.leave_type_id, leave_type_id);
    assert_eq!(body.status, LeaveStatus::Pending);
}

#[tokio::test]
async fn test_submit_leave_request_invalid_type_id() {
    let db = TestDatabase::new().await;
    let factory = TestFactory::new(db.pool());

    let workspace = factory.create_workspace("Test WS").await;
    let user = factory.create_user("test@example.com").await;
    let employee = factory.create_employee(&workspace.id, "John Doe").await;
    factory.add_member(&workspace.id, &user.id, MemberRole::Admin).await;

    let app = create_test_app(db.pool()).await;
    let token = create_test_token(&user);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/submit", workspace.id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "employee_id": employee.id,
                    "leave_type_id": Uuid::new_v4(), // Non-existent ID
                    "start_date": "2026-02-01",
                    "end_date": "2026-02-03",
                    "reason": "Vacation"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

### Step 4: Run Full Test Suite

```bash
# Run all tests
./run-tests.sh --all

# Or just API tests
./run-tests.sh --api leave
```

### Step 5: Manual Testing Checklist

1. **Database Migration**
   - [ ] Run migration on fresh database
   - [ ] Run migration on existing database with data
   - [ ] Verify no orphan records remain
   - [ ] Verify unique constraints work

2. **API Endpoints**
   - [ ] POST /leave/submit with valid leave_type_id
   - [ ] POST /leave/submit with invalid leave_type_id (should fail)
   - [ ] GET /leave/requests returns leave_type_name
   - [ ] GET /leave/balances returns leave_type_name
   - [ ] POST /leave/adjust with leave_type_id

3. **Frontend**
   - [ ] LeaveTypeSelect component loads types
   - [ ] Submit leave request form works
   - [ ] Leave requests table shows type name
   - [ ] Leave balances page shows type name
   - [ ] Balance adjustment works

### Step 6: Verify SQLx Metadata

```bash
cd apiserver
cargo sqlx prepare --check
```

## Todo List
- [ ] Update test factory with leave type helpers
- [ ] Update leave_policy_repo_tests.rs
- [ ] Update leave_policy_api_tests.rs
- [ ] Run `./run-tests.sh --all`
- [ ] Run `cargo sqlx prepare --check`
- [ ] Manual testing: database migration
- [ ] Manual testing: API endpoints
- [ ] Manual testing: frontend

## Success Criteria
- All automated tests pass
- Manual testing checklist complete
- SQLx metadata up to date
- No orphan records in database

## Risk Assessment
- **Test Failures**: Expected during migration, update as needed
- **Data Issues**: Run validation queries after migration

## Validation Queries

Run these queries after migration to verify data integrity:

```sql
-- Check for NULL leave_type_id in leave_requests
SELECT COUNT(*) FROM leave_requests WHERE leave_type_id IS NULL;
-- Expected: 0

-- Check for NULL leave_type_id in leave_balances
SELECT COUNT(*) FROM leave_balances WHERE leave_type_id IS NULL;
-- Expected: 0

-- Check for NULL leave_type_id in leave_accrual_logs
SELECT COUNT(*) FROM leave_accrual_logs WHERE leave_type_id IS NULL;
-- Expected: 0

-- Verify FK integrity
SELECT lr.id FROM leave_requests lr
LEFT JOIN leave_types lt ON lr.leave_type_id = lt.id
WHERE lt.id IS NULL;
-- Expected: 0 rows

-- Check unique constraint
SELECT employee_id, leave_type_id, year, COUNT(*)
FROM leave_balances
GROUP BY employee_id, leave_type_id, year
HAVING COUNT(*) > 1;
-- Expected: 0 rows
```

## Next Steps
After all tests pass:
1. Create PR for review
2. Deploy to staging environment
3. Run migration on staging
4. Verify staging functionality
5. Schedule production deployment
