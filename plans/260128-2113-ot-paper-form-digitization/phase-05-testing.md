# Phase 05: Testing

## Context Links
- [All Phases](/home/elt1541/leeattend/plans/260128-2113-ot-paper-form-digitization/plan.md)
- [Existing OT Tests](/home/elt1541/leeattend/apiserver/tests/ot_workflow_tests.rs)

## Overview
- **Priority**: P1
- **Status**: Pending
- **Effort**: 4 hours

Write unit tests, integration tests, and manual test scenarios for paper form OT workflow.

## Key Insights

From codebase:
- Tests in `apiserver/tests/` use `sqlx::test` for database tests
- Test factory pattern in `apiserver/tests/common/` for setup
- Frontend tests in `webdashboard/tests/` using Playwright
- E2E tests use `data-testid` locators

## Requirements

### Backend Tests
- Unit tests for service validation logic
- Integration tests for API endpoint
- RBAC tests (Admin vs non-Admin)
- Upsert behavior tests

### Frontend Tests
- Component render tests (dialog)
- Form validation tests
- E2E workflow test

### Manual Test Scenarios
- Happy path
- Edge cases
- Permission checks

## Backend Unit Tests

```rust
// apiserver/tests/paper_ot_tests.rs

use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;

mod common;
use common::*;

#[sqlx::test]
async fn test_enter_paper_ot_creates_summary(pool: PgPool) {
    // Setup
    let ws = create_test_workspace(&pool).await;
    let admin = create_test_user(&pool, "admin@test.com").await;
    let employee = create_test_employee(&pool, ws.id).await;
    add_member(&pool, ws.id, admin.id, "admin").await;

    let date = NaiveDate::from_ymd_opt(2026, 1, 28).unwrap();

    // Act
    let result = AttendanceSummaryService::enter_paper_ot(
        &pool,
        employee.id,
        date,
        120,
        "Paper form #1234".to_string(),
        admin.id,
        None,
        false,
    ).await;

    // Assert
    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.claimed_ot_minutes, 120);
    assert_eq!(summary.ot_status, OtStatus::PendingManager);
    assert_eq!(summary.entered_via, Some("paper_form".to_string()));
    assert_eq!(summary.claim_entered_by_user_id, Some(admin.id));
}

#[sqlx::test]
async fn test_enter_paper_ot_validates_minutes_positive(pool: PgPool) {
    let ws = create_test_workspace(&pool).await;
    let admin = create_test_user(&pool, "admin@test.com").await;
    let employee = create_test_employee(&pool, ws.id).await;

    let date = NaiveDate::from_ymd_opt(2026, 1, 28).unwrap();

    let result = AttendanceSummaryService::enter_paper_ot(
        &pool,
        employee.id,
        date,
        0, // Invalid
        "Reason".to_string(),
        admin.id,
        None,
        false,
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("positive"));
}

#[sqlx::test]
async fn test_enter_paper_ot_rejects_exceed_without_flag(pool: PgPool) {
    let ws = create_test_workspace(&pool).await;
    let admin = create_test_user(&pool, "admin@test.com").await;
    let employee = create_test_employee(&pool, ws.id).await;

    let date = NaiveDate::from_ymd_opt(2026, 1, 28).unwrap();

    // Create existing summary with system_detected = 60
    sqlx::query!(
        "INSERT INTO attendance_daily_summaries (workspace_id, employee_id, date, system_detected_ot_minutes)
         VALUES ($1, $2, $3, 60)",
        ws.id, employee.id, date
    ).execute(&pool).await.unwrap();

    let result = AttendanceSummaryService::enter_paper_ot(
        &pool,
        employee.id,
        date,
        120, // Exceeds system detected
        "Reason".to_string(),
        admin.id,
        None,
        false, // Don't allow exceed
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exceeds"));
}

#[sqlx::test]
async fn test_enter_paper_ot_allows_exceed_with_flag(pool: PgPool) {
    let ws = create_test_workspace(&pool).await;
    let admin = create_test_user(&pool, "admin@test.com").await;
    let employee = create_test_employee(&pool, ws.id).await;

    let date = NaiveDate::from_ymd_opt(2026, 1, 28).unwrap();

    // Create existing summary with system_detected = 60
    sqlx::query!(
        "INSERT INTO attendance_daily_summaries (workspace_id, employee_id, date, system_detected_ot_minutes)
         VALUES ($1, $2, $3, 60)",
        ws.id, employee.id, date
    ).execute(&pool).await.unwrap();

    let result = AttendanceSummaryService::enter_paper_ot(
        &pool,
        employee.id,
        date,
        120, // Exceeds system detected
        "Reason".to_string(),
        admin.id,
        None,
        true, // Allow exceed
    ).await;

    assert!(result.is_ok());
}

#[sqlx::test]
async fn test_enter_paper_ot_upserts_existing(pool: PgPool) {
    let ws = create_test_workspace(&pool).await;
    let admin = create_test_user(&pool, "admin@test.com").await;
    let employee = create_test_employee(&pool, ws.id).await;

    let date = NaiveDate::from_ymd_opt(2026, 1, 28).unwrap();

    // First entry
    AttendanceSummaryService::enter_paper_ot(
        &pool, employee.id, date, 60, "First".to_string(), admin.id, None, false,
    ).await.unwrap();

    // Second entry (upsert)
    let result = AttendanceSummaryService::enter_paper_ot(
        &pool, employee.id, date, 120, "Updated".to_string(), admin.id, None, true,
    ).await.unwrap();

    assert_eq!(result.claimed_ot_minutes, 120);
    assert_eq!(result.ot_reason, Some("Updated".to_string()));

    // Only one row
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
        employee.id, date
    ).fetch_one(&pool).await.unwrap().unwrap_or(0);

    assert_eq!(count, 1);
}
```

## Backend Integration Tests (API)

```rust
// apiserver/tests/paper_ot_api_tests.rs

use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::*;

#[tokio::test]
async fn test_enter_paper_ot_endpoint_admin_only() {
    let app = create_test_app().await;
    let ws = create_test_workspace(&app.pool).await;
    let admin = create_test_user(&app.pool, "admin@test.com").await;
    let member = create_test_user(&app.pool, "member@test.com").await;
    let employee = create_test_employee(&app.pool, ws.id).await;

    add_member(&app.pool, ws.id, admin.id, "admin").await;
    add_member(&app.pool, ws.id, member.id, "member").await;

    // Admin can access
    let admin_token = login(&app, "admin@test.com").await;
    let res = app.post(
        &format!("/workspaces/{}/attendance/enter-paper-ot/{}", ws.id, employee.id),
        &admin_token,
        json!({
            "date": "2026-01-28",
            "claimed_minutes": 120,
            "reason": "Paper form"
        }),
    ).await;
    assert_eq!(res.status(), StatusCode::OK);

    // Member cannot access
    let member_token = login(&app, "member@test.com").await;
    let res = app.post(
        &format!("/workspaces/{}/attendance/enter-paper-ot/{}", ws.id, employee.id),
        &member_token,
        json!({
            "date": "2026-01-28",
            "claimed_minutes": 60,
            "reason": "Should fail"
        }),
    ).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_enter_paper_ot_endpoint_validates_employee() {
    let app = create_test_app().await;
    let ws = create_test_workspace(&app.pool).await;
    let admin = create_test_user(&app.pool, "admin@test.com").await;
    add_member(&app.pool, ws.id, admin.id, "admin").await;

    let admin_token = login(&app, "admin@test.com").await;
    let fake_employee_id = Uuid::new_v4();

    let res = app.post(
        &format!("/workspaces/{}/attendance/enter-paper-ot/{}", ws.id, fake_employee_id),
        &admin_token,
        json!({
            "date": "2026-01-28",
            "claimed_minutes": 120,
            "reason": "Paper form"
        }),
    ).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
```

## Frontend E2E Tests

```typescript
// webdashboard/tests/paper-ot-workflow.spec.ts

import { test, expect } from "@playwright/test";
import { loginAsAdmin, createTestEmployee } from "./helpers";

test.describe("Paper OT Workflow", () => {
  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page);
  });

  test("admin can enter paper OT form", async ({ page }) => {
    // Navigate to OT Approvals
    await page.goto("/approvals/ot");
    await expect(page.locator("h1")).toContainText("OT Approvals");

    // Click Enter Paper Form button
    await page.click('button:has-text("Enter Paper Form")');
    await expect(page.locator('[role="dialog"]')).toBeVisible();

    // Fill form
    await page.click('[data-testid="employee-search"]');
    await page.fill('[data-testid="employee-search"] input', "Nguyen");
    await page.click('[data-testid="employee-option"]:first-child');

    await page.fill('input[type="date"]', "2026-01-28");
    await page.fill('input[type="number"]', "120");
    await page.fill("textarea", "Paper form #1234 - Production line");

    // Submit
    await page.click('button[type="submit"]');

    // Dialog closes
    await expect(page.locator('[role="dialog"]')).not.toBeVisible();

    // New entry appears in table with Paper badge
    await expect(page.locator("table")).toContainText("Paper");
    await expect(page.locator("table")).toContainText("120m");
  });

  test("filter shows only paper forms", async ({ page }) => {
    await page.goto("/approvals/ot");

    // Click Paper Forms filter
    await page.click('button:has-text("Paper Forms")');

    // Table should only show Paper badge entries
    const rows = page.locator("tbody tr");
    for (const row of await rows.all()) {
      await expect(row.locator("text=Paper")).toBeVisible();
    }
  });

  test("form validation prevents empty submission", async ({ page }) => {
    await page.goto("/approvals/ot");
    await page.click('button:has-text("Enter Paper Form")');

    // Try submit without filling
    await page.click('button[type="submit"]');

    // Error message shown
    await expect(page.locator("text=Select an employee")).toBeVisible();
  });
});
```

## Manual Test Scenarios

### Happy Path

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Login as Admin | Dashboard loads |
| 2 | Navigate to /approvals/ot | OT Approvals page loads |
| 3 | Click "Enter Paper Form" | Dialog opens |
| 4 | Search and select employee | Employee name shown |
| 5 | Enter date: 2026-01-28 | Date accepted |
| 6 | Enter minutes: 120 | Value shown |
| 7 | Enter reason: "Paper form #1234" | Text shown |
| 8 | Click Submit | Dialog closes, toast shows success |
| 9 | Check table | New row with "Paper" badge, status "pending_manager" |
| 10 | Click approve checkmark | Status changes to "approved" |
| 11 | Check payroll | Approved OT minutes reflected |

### Edge Cases

| Scenario | Input | Expected |
|----------|-------|----------|
| Zero minutes | minutes=0 | Error: "Minutes must be > 0" |
| Empty reason | reason="" | Error: "Reason is required" |
| Future date | date=tomorrow | Error (date picker blocks) |
| Same employee/date twice | Submit same entry | Upsert: updates existing row |
| Employee from other workspace | Select cross-ws employee | Employee not in dropdown |
| Non-admin user | Member login | "Enter Paper Form" button hidden |

### Permission Matrix

| Role | Enter Form | View List | Approve/Reject |
|------|------------|-----------|----------------|
| Owner | Yes | Yes | Yes |
| Admin | Yes | Yes | Yes |
| Manager | No | View own dept? | View own dept? |
| Member | No | No | No |

## Test Data Requirements

### Database Fixtures

```sql
-- Test workspace
INSERT INTO workspaces (id, name) VALUES
  ('ws-test-uuid', 'Test Workspace');

-- Test users
INSERT INTO users (id, email) VALUES
  ('admin-uuid', 'admin@test.com'),
  ('member-uuid', 'member@test.com');

-- Test employees
INSERT INTO employees (id, workspace_id, full_name, employee_id_number) VALUES
  ('emp1-uuid', 'ws-test-uuid', 'Nguyen Van A', 'EMP001'),
  ('emp2-uuid', 'ws-test-uuid', 'Tran Thi B', 'EMP002');

-- Workspace members
INSERT INTO workspace_members (workspace_id, user_id, role) VALUES
  ('ws-test-uuid', 'admin-uuid', 'admin'),
  ('ws-test-uuid', 'member-uuid', 'member');
```

## Implementation Steps

1. [ ] Create `paper_ot_tests.rs` in `apiserver/tests/`
2. [ ] Add service unit tests
3. [ ] Add API integration tests
4. [ ] Create `paper-ot-workflow.spec.ts` in `webdashboard/tests/`
5. [ ] Add E2E tests
6. [ ] Run full test suite
7. [ ] Document manual test results

## Todo List

- [ ] Write service unit tests (4 tests)
- [ ] Write API integration tests (2 tests)
- [ ] Write E2E tests (3 tests)
- [ ] Execute manual test scenarios
- [ ] Document results

## Success Criteria

- [ ] All backend unit tests pass
- [ ] All API integration tests pass
- [ ] All E2E tests pass
- [ ] Manual happy path verified
- [ ] Edge cases documented

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Flaky E2E tests | Medium | Use stable locators, add waits |
| Test data pollution | Medium | Use transactions, rollback |
| CI timeout | Low | Parallelize tests |

## Training Materials (Optional)

For department heads/assistants, create brief documentation:

1. **Quick Guide**: How to enter a paper OT form (5 steps with screenshots)
2. **FAQ**: Common questions (duplicate entries, photo upload, etc.)
3. **Troubleshooting**: Error messages and solutions

## Next Steps

After testing complete:
- Deploy to staging for UAT
- Gather feedback from department heads
- Iterate on UX based on feedback
- Deploy to production
