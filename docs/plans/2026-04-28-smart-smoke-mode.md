# Smart Smoke Mode — Implementation Plan

> **For implementation:** REQUIRED SUB-SKILL: Superpowers:executing-plans to implement task-by-task.

**Goal:** Enable a curated subset of fast integration/E2E tests in smoke mode using nextest filter expressions and naming conventions.

**Architecture:** Three changes: (1) fix the script pipeline to run integration phase under smoke mode, (2) tag ~33 integration/E2E tests with smoke naming convention, (3) add safety checks so zero-matches are caught. No new Cargo features, no test binary changes.

**Tech Stack:** Bash (run-tests.sh pipeline), cargo nextest, Playwright

---

### Task 1: Fix `scripts/cli-parser.sh` — Enable integration phase in smoke mode

**Files:**
- Modify: `scripts/cli-parser.sh:414-419` (smoke mode block)
- Modify: `scripts/cli-parser.sh:33` (help text for --smoke)
- Modify: `scripts/cli-parser.sh:75-77` (example in help text)

**Step 1: Change PHASE_INTEGRATION from false to true**

At lines 414-419, the current code is:
```bash
if [[ "$SMOKE" == "true" ]]; then
    PHASE_LINT=true
    PHASE_UNIT=true
    PHASE_INTEGRATION=false
    echo -e "${BLUE}🔥 Smoke mode: running lint + unit only${NC}"
fi
```

Change to:
```bash
if [[ "$SMOKE" == "true" ]]; then
    PHASE_LINT=true
    PHASE_UNIT=true
    PHASE_INTEGRATION=true
    PATTERN="smoke"
    echo -e "${BLUE}🔥 Smoke mode: running lint + unit + fast smoke integration${NC}"
fi
```

**Step 2: Update help text**

Line 33: Change description from `run lint + unit only` to `run lint + unit + fast smoke-tagged integration`

Lines 75-77: Update the smoke examples to mention integration tests being included.

**Rationale:** Setting `PATTERN="smoke"` ensures the downstream pipeline (run-tests.sh → test-integration.sh) receives "smoke" as the test filter, which nextest interprets as a name substring filter and Playwright interprets as a grep pattern.

---

### Task 2: Fix `scripts/test-integration.sh` — Handle smoke filter and add safety checks

**Files:**
- Modify: `scripts/test-integration.sh:17-108`

**Step 1: Add `--no-tests=fail` for nextest when SMOKE mode is active**

The current nextest commands use `--no-tests=pass`. In smoke mode, we want `--no-tests=fail` to catch the case where all smoke tags have been accidentally removed.

At lines 36-37, change from:
```bash
local APISERVER_INT_CMD="cargo nextest run --test '*' $TEST_PATTERN --no-tests=pass"
local AGENT_INT_CMD="cargo nextest run --test '*' $TEST_PATTERN --no-tests=pass"
```
To:
```bash
local NO_TESTS_FLAG="pass"
if [[ "$SMOKE" == "true" ]]; then
    NO_TESTS_FLAG="fail"
fi
local APISERVER_INT_CMD="cargo nextest run --test '*' $TEST_PATTERN --no-tests=$NO_TESTS_FLAG"
local AGENT_INT_CMD="cargo nextest run --test '*' $TEST_PATTERN --no-tests=$NO_TESTS_FLAG"
```

Same change for the Docker versions at lines 79-80.

**Step 2: Show a warning before running smoke integration tests**

Add after line 26:
```bash
if [[ "$SMOKE" == "true" ]] && [[ "$TEST_PATTERN" == "smoke" || "$TEST_PATTERN" == "smoke:"* ]]; then
    echo -e "${YELLOW}🔥 Smoke mode: running only smoke-tagged integration tests${NC}"
fi
```

**Step 3: Add post-run safety check for Playwright**

After the parallel execution, if SMOKE mode, grep the playwright log for 0 tests:
```bash
if [[ "$SMOKE" == "true" ]]; then
    local pw_log="$LOG_DIR/playwright-e2e.log"
    if grep -q "All tests were skipped" "$pw_log" 2>/dev/null || grep -q "0 passed" "$pw_log" 2>/dev/null; then
        echo -e "${RED}⚠️  WARNING: 0 Playwright smoke tests were executed!${NC}"
        echo -e "${RED}   Check if 'smoke:' prefix is correctly set on test titles.${NC}"
    fi
fi
```

---

### Task 3: Fix `run-tests.sh` — Ensure dead code works

**Files:**
- Modify: `run-tests.sh:78-84`

The dead code at lines 80-82 is now live because `PHASE_INTEGRATION=true` in smoke mode. However, since we set `PATTERN="smoke"` in cli-parser.sh, the condition `-z "$INT_PATTERN"` at line 80 may or may not be true depending on flow.

Actually, with our Task 1 change, `PATTERN="smoke"` is already set, so `INT_PATTERN="$PATTERN"` at line 79 sets `INT_PATTERN="smoke"`. The dead code at 80-82 checks `-z "$INT_PATTERN"` which is false, so it doesn't override. Everything works naturally. No changes needed here.

**But** add a safety warning after line 82:
```bash
if [[ "$SMOKE" == "true" ]]; then
    echo -e "${YELLOW}🔥 Smoke integration filter active: test names must contain 'smoke'${NC}"
fi
```

---

### Task 4: Tag ~20 apiserver integration tests with `_smoke` suffix

**Files:**
- Modify: `apiserver/tests/integration/auth/session.rs`
- Modify: `apiserver/tests/integration/auth/password.rs`
- Modify: `apiserver/tests/integration/auth/rbac.rs`
- Modify: `apiserver/tests/integration/employee/management.rs`
- Modify: `apiserver/tests/integration/attendance/time_tracking.rs`
- Modify: `apiserver/tests/integration/attendance/adjustment_api.rs`
- Modify: `apiserver/tests/integration/attendance/api_log.rs`
- Modify: `apiserver/tests/integration/leave/api.rs`
- Modify: `apiserver/tests/integration/shift/api.rs`
- Modify: `apiserver/tests/integration/shift/working_pattern.rs`
- Modify: `apiserver/tests/integration/payroll/api_repo.rs`
- Modify: `apiserver/tests/integration/workspace/management.rs`
- Modify: `apiserver/tests/integration/roster/api.rs`
- Modify: `apiserver/tests/integration/department/crud.rs`
- Modify: `apiserver/tests/integration/holiday/crud.rs`
- Modify: `apiserver/tests/integration/agent/crud.rs`
- Modify: `apiserver/tests/integration/ess.rs`

**Step 1: Rename test functions**

For each file, rename the test function to include `_smoke`. Note: These are `#[tokio::test]` async functions. The rename is a simple text replacement at the function declaration and any references. Since Rust test functions are rarely referenced by name in other code, this is a safe rename.

| File | Current Name | New Name |
|------|-------------|----------|
| auth/session.rs | `test_auth_session_lifecycle` | `test_auth_session_lifecycle_smoke` |
| auth/session.rs | `test_get_me_and_logout` | `test_get_me_and_logout_smoke` |
| auth/password.rs | `test_change_password_flow` | `test_change_password_flow_smoke` |
| auth/rbac.rs | `test_rbac_admin_access` | `test_rbac_admin_access_smoke` |
| employee/management.rs | `test_employee_crud_lifecycle` | `test_employee_crud_lifecycle_smoke` |
| employee/management.rs | `test_get_employees_list` | `test_get_employees_list_smoke` |
| attendance/time_tracking.rs | `test_punch_in_grace_period_enforcement` | `test_punch_in_grace_period_enforcement_smoke` |
| attendance/adjustment_api.rs | `test_attendance_adjustment_lifecycle` | `test_attendance_adjustment_lifecycle_smoke` |
| attendance/api_log.rs | `test_get_attendance_logs_filtering_and_pagination` | `test_get_attendance_logs_smoke` |
| leave/api.rs | `test_leave_request_lifecycle` | `test_leave_request_lifecycle_smoke` |
| leave/api.rs | `test_leave_balances` | `test_leave_balances_smoke` |
| shift/api.rs | `test_shift_api_lifecycle` | `test_shift_api_lifecycle_smoke` |
| shift/working_pattern.rs | `test_working_pattern_crud_lifecycle` | `test_working_pattern_crud_lifecycle_smoke` |
| payroll/api_repo.rs | `test_payroll_payslips` | `test_payroll_payslips_smoke` |
| workspace/management.rs | `test_workspace_creation_and_membership` | `test_workspace_creation_and_membership_smoke` |
| roster/api.rs | `test_bulk_upsert_roster_api` | `test_bulk_upsert_roster_api_smoke` |
| department/crud.rs | `test_department_crud` | `test_department_crud_smoke` |
| holiday/crud.rs | `test_holiday_crud_lifecycle` | `test_holiday_crud_lifecycle_smoke` |
| agent/crud.rs | `test_agent_crud_operations` | `test_agent_crud_operations_smoke` |
| ess.rs | `test_ess_get_profile` | `test_ess_get_profile_smoke` |

**Step 2: Commit**

```bash
git add apiserver/tests/integration/
git commit -m "test: tag 20 apiserver integration tests as smoke" </dev/null
```

---

### Task 5: Tag ~8 agent-main integration tests with `_smoke` suffix

**Files:**
- Modify: `agent-main/tests/disconnect_lifecycle_test.rs`
- Modify: `agent-main/tests/push_logs_integration_test.rs`
- Modify: `agent-main/tests/batch_worker_test.rs`
- Modify: `agent-main/tests/duplicate_detection_test.rs`
- Modify: `agent-main/tests/retry_limit_test.rs`
- Modify: `agent-main/tests/employee_id_normalization_test.rs`
- Modify: `agent-main/tests/redb_store_ops_test.rs`
- Modify: `agent-main/tests/sync_log_classification_test.rs`

**Step 1: Rename test functions**

| File | Current Name | New Name |
|------|-------------|----------|
| disconnect_lifecycle_test.rs | `test_worker_lifecycle_connect_operations_disconnect` | `test_worker_lifecycle_connect_operations_disconnect_smoke` |
| push_logs_integration_test.rs | `test_end_to_end_log_push_data_flow` | `test_end_to_end_log_push_data_flow_smoke` |
| batch_worker_test.rs | `test_batch_worker_inserts_records` | `test_batch_worker_inserts_records_smoke` |
| duplicate_detection_test.rs | `test_duplicate_detection_filters_repeated_logs` | `test_duplicate_detection_filters_repeated_logs_smoke` |
| retry_limit_test.rs | `test_retry_count_increments` | `test_retry_count_increments_smoke` |
| employee_id_normalization_test.rs | `test_employee_id_stored_with_leading_zeros` | `test_employee_id_stored_with_leading_zeros_smoke` |
| redb_store_ops_test.rs | `test_device_signature_load_unload` | `test_device_signature_load_unload_smoke` |
| sync_log_classification_test.rs | `test_sync_log_classification_new_vs_retry` | `test_sync_log_classification_new_vs_retry_smoke` |

**Step 2: Commit**

```bash
git add agent-main/tests/
git commit -m "test: tag 8 agent-main integration tests as smoke" </dev/null
```

---

### Task 6: Tag ~5 Playwright E2E tests with `smoke:` prefix

**Files:**
- Modify: `webdashboard/tests/build-smoke.spec.ts`
- Modify: `webdashboard/tests/login-success.spec.ts`
- Modify: `webdashboard/tests/attendance-logs.spec.ts`
- Modify: `webdashboard/tests/specs/device-crud.spec.ts`
- Modify: `webdashboard/tests/sidebar-navigation.spec.ts`

**Step 1: Update test titles**

For each `.spec.ts` file, find the `test('...', ...)` call and add `smoke: ` prefix to the title string. Playwright's `--grep "smoke"` will match these.

Example: change `test('should load the application...')` to `test('smoke: app renders and boots')`

For `build-smoke.spec.ts` (which already has "smoke" in its name), the test title should also explicitly include `smoke:` for grep matching.

**Step 2: Commit**

```bash
git add webdashboard/tests/
git commit -m "test: tag 5 Playwright E2E tests as smoke" </dev/null
```

---

### Task 7: Update AGENTS.md smoke mode documentation

**Files:**
- Modify: `AGENTS.md` (the test commands section)

**Step 1: Update the smoke command description**

Current line mentions `lint + unit` — update to note that smoke mode now includes fast smoke-tagged integration tests.

---

### Task 8: Verify — dry-run and actual run

**Files:** None (verification only)

**Step 1: Dry-run to verify pipeline**

```bash
./run-tests.sh --api --smoke --dry-run </dev/null
```

Expected: Shows lint, unit, and integration phases all enabled. Pattern shows "smoke".

**Step 2: Run smoke mode and verify integration tests execute**

```bash
./run-tests.sh --api --smoke </dev/null
```

Expected:
- Lint phase runs
- Unit phase runs
- Integration phase runs with filter "smoke"
- Only smoke-tagged tests execute (verify in logs)
- Smoke integration runs in < 3 minutes

**Step 3: Verify that `--smoke` without smoke-tagged tests fails**

To test the `--no-tests=fail` safety check, temporarily rename one smoke test file suffix to break the filter and verify it fails. Then restore.

**Step 4: Run full test to verify no regressions**

```bash
./run-tests.sh --api </dev/null
```

Expected: All tests pass (renamed tests still run, just with updated names).

---

### Execution Order

Tasks 1-3 (script changes) can run in parallel. Tasks 4-6 (test tagging) can also run in parallel. Task 7 (docs) depends on 1-3. Task 8 (verify) is the final step after all changes.
