# Schema Migration Validation Report
**Date**: 2026-01-24 | **Time**: 12:56 UTC
**Component**: apiserver (Rust/SQLx)
**Migration Version**: 20260124170000 through 20260124173000

---

## Executive Summary
Schema migrations for Schema Design Improvements plan failed during execution. Analysis identified a critical issue in the `20260124170000_create_leave_types.sql` migration that prevents deployment.

**Status**: FAILED - Critical Issue Found
**Unit Tests**: PASSED (47 tests)
**Integration Tests**: BLOCKED - Migration Failure

---

## Test Results Overview

### Phase 1: Unit Tests - PASSED
- **agent-main**: 3/3 tests passed
- **agent-zk-worker**: 16/16 tests passed
- **apiserver**: 28/28 tests passed
- **Total Unit Tests**: 47 PASSED, 0 FAILED

### Phase 2: Integration Tests - FAILED
- **Database Setup**: SUCCESS
- **Migration Execution**: FAILED at 20260124170000
- **Error Type**: ON CONFLICT constraint mismatch

---

## Migrations Tested

### Successfully Applied (Before Schema Improvements)
1. ✅ 20260124150000 - fix_employee_id_uniqueness
2. ✅ 20260124160000 - performance_and_search

### Failed to Apply (New Migrations)
3. ❌ 20260124170000 - create_leave_types
4. ⏭️ 20260124171000 - add_missing_fk_indexes (blocked by #3)
5. ⏭️ 20260124172000 - standardize_payslip_incentives_pk (blocked by #3)
6. ⏭️ 20260124173000 - add_compound_indexes (blocked by #3)

---

## Critical Issue Analysis

### Migration: 20260124170000_create_leave_types.sql

**Error Message**:
```
error: while executing migration 20260124170000:
error returned from database:
there is no unique or exclusion constraint matching the ON CONFLICT specification
```

**Root Cause**:
Line 28 of migration attempts `ON CONFLICT (workspace_id, code) DO NOTHING` but the UNIQUE constraint was not created due to `CREATE TABLE IF NOT EXISTS` idempotency issue.

**Problem Scenario**:
1. Line 4-12: `CREATE TABLE IF NOT EXISTS leave_types` creates table with `UNIQUE(workspace_id, code)` constraint
2. If table already exists from previous test run, IF NOT EXISTS skips the entire CREATE statement
3. Existing table lacks the UNIQUE constraint (only has PRIMARY KEY)
4. Line 28: INSERT with `ON CONFLICT (workspace_id, code)` fails because no matching constraint exists

**File Reference**:
- File: `/home/elt1541/leeattend/apiserver/migrations/20260124170000_create_leave_types.sql`
- Line: 28
- Problem: `ON CONFLICT (workspace_id, code) DO NOTHING;` without matching UNIQUE constraint

**Verification**:
```sql
-- In test_database after failed migration:
SELECT constraint_type, constraint_name
FROM information_schema.table_constraints
WHERE table_name = 'leave_types';

-- Result: Only PRIMARY KEY constraint, NO UNIQUE constraint
```

---

## Secondary Issues

### Migration: 20260124171000_add_missing_fk_indexes.sql
- **Status**: Not tested (blocked by #3)
- **Concern**: Uses standard index creation syntax - likely OK
- **File**: `/home/elt1541/leeattend/apiserver/migrations/20260124171000_add_missing_fk_indexes.sql`
- **Lines**: 5, 9 (index creation)

### Migration: 20260124172000_standardize_payslip_incentives_pk.sql
- **Status**: Not tested (blocked by #3)
- **Risk Level**: MEDIUM - Complex multi-step DDL sequence
- **File**: `/home/elt1541/leeattend/apiserver/migrations/20260124172000_standardize_payslip_incentives_pk.sql`
- **Lines**: 10-23 (constraint drop and column operations)
- **Concern**: Drops primary key and drops column - risky if payslip_incentives has existing data

### Migration: 20260124173000_add_compound_indexes.sql
- **Status**: Not tested (blocked by #3)
- **Risk Level**: LOW - Simple compound index creation
- **File**: `/home/elt1541/leeattend/apiserver/migrations/20260124173000_add_compound_indexes.sql`
- **Lines**: 5 (compound index on employees table)

---

## Code Issues Summary

### Issue #1: CREATE TABLE IF NOT EXISTS with Constraints
**Severity**: CRITICAL
**File**: `/home/elt1541/leeattend/apiserver/migrations/20260124170000_create_leave_types.sql`
**Lines**: 4-12, 28

The use of `CREATE TABLE IF NOT EXISTS` combined with `ON CONFLICT` in the subsequent INSERT is problematic because:
1. If table exists without constraints from prior run, the constraint is missing
2. ON CONFLICT clause has no matching unique constraint
3. Migration becomes non-idempotent in certain states

**Solution Options**:
1. Drop and recreate table (if no prod data)
2. Add constraint separately with ALTER TABLE if not exists check
3. Use different approach: check table existence before INSERT

---

## Testing Command Used
```bash
./run-tests.sh --api
```

**Environment**:
- Docker Postgres 16-alpine on port 5433
- DATABASE_URL: postgres://test_user:test_password@localhost:5433/test_database
- Test mode: Integration tests with sqlx migrations

---

## Recommendations

### Immediate Actions (BLOCKING)
1. **Fix Migration 20260124170000**:
   - Replace `CREATE TABLE IF NOT EXISTS` with conditional constraint creation
   - Ensure UNIQUE constraint exists before ON CONFLICT
   - Test with fresh and existing database states

2. **Add Migration Validation Tests**:
   - Run migrations twice on same DB to test idempotency
   - Verify constraints are created correctly

3. **Verify Dependent Migrations**:
   - Once #1 is fixed, re-run all 4 new migrations
   - Ensure migrations 171000, 172000, 173000 apply cleanly

### Suggested Approach for Migration Fix

Replace the current create + seed pattern with:

```sql
-- Step 1: Create table with all constraints
CREATE TABLE IF NOT EXISTS leave_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL,
    name TEXT NOT NULL,
    is_paid BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(workspace_id, code)
);

-- Step 2: Ensure unique constraint exists (idempotent)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE table_name = 'leave_types' AND constraint_type = 'UNIQUE'
    ) THEN
        ALTER TABLE leave_types ADD CONSTRAINT leave_types_workspace_code_unique
        UNIQUE(workspace_id, code);
    END IF;
END $$;

-- Step 3: Then proceed with insert (now safe to use ON CONFLICT)
INSERT INTO leave_types (workspace_id, code, name, is_paid)
...
ON CONFLICT (workspace_id, code) DO NOTHING;
```

Or simpler: remove ON CONFLICT and use DISTINCT:

```sql
INSERT INTO leave_types (workspace_id, code, name, is_paid)
SELECT DISTINCT w.id, lt.code, lt.name, lt.is_paid
FROM workspaces w
CROSS JOIN (VALUES ...) AS lt(code, name, is_paid)
WHERE NOT EXISTS (
    SELECT 1 FROM leave_types lt2
    WHERE lt2.workspace_id = w.id AND lt2.code = lt.code
);
```

---

## Unresolved Questions

1. **Does leave_types table exist in production?** If yes, how should we handle existing constraint mismatch?
2. **Should we validate all 4 new migrations together or separately?** Recommend testing as a suite after #3 fix.
3. **Is there existing leave type data that needs migration?** Migration assumes leave_requests has leave_type VARCHAR - need to verify.
4. **What's the deployment strategy for idempotent migrations in production?** Should migrations be designed to detect and repair missing constraints?

---

## Next Steps

1. Review and fix migration 20260124170000
2. Re-run migration on fresh database
3. Run full integration test suite (./run-tests.sh --api)
4. Validate all 4 schema improvements apply successfully
5. Check for any data consistency issues post-migration

**Estimated Time**: 30 minutes for fix + validation
