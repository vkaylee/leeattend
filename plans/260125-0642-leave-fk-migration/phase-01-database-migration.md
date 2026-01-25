# Phase 1: Database Migration

## Context Links
- Current migrations: `apiserver/migrations/20260124170000_create_leave_types.sql`
- Existing FK migration: `apiserver/migrations/20260124180000_refactor_leave_policies_to_fk.sql`

## Overview
- **Priority**: P1 (blocking)
- **Status**: pending
- **Effort**: 1h

Create a new migration that:
1. Adds `leave_type_id` FK column to `leave_balances`
2. Backfills all FK columns from existing string data
3. Handles orphan records gracefully
4. Makes FK columns NOT NULL
5. Drops legacy string columns

## Key Insights
- `leave_requests` and `leave_accrual_logs` already have nullable `leave_type_id` columns
- `leave_balances` lacks FK column entirely
- Some records may have `leave_type` strings that don't match any `leave_types.code`
- Must handle cross-workspace orphans (employee's workspace_id needed for FK lookup)

## Requirements

### Functional
- Add FK column to `leave_balances`
- Populate all FK columns from string values
- Handle orphan records by mapping to "vacation" leave type
- Make FK columns NOT NULL after population
- Drop string columns

### Non-Functional
- Migration must be idempotent
- No data loss
- Transaction-safe

## Related Code Files

### Files to Create
- `apiserver/migrations/20260125XXXXXX_complete_leave_fk_migration.sql`

## Implementation Steps

### Step 1: Create Migration File
Create new migration at `apiserver/migrations/20260125180000_complete_leave_fk_migration.sql`:

```sql
-- Up Migration: Complete Leave Management FK Migration

-- 1. Add leave_type_id to leave_balances (if not exists)
ALTER TABLE leave_balances
ADD COLUMN IF NOT EXISTS leave_type_id UUID REFERENCES leave_types(id) ON DELETE RESTRICT;

-- 2. Backfill leave_balances.leave_type_id
-- Join via employees to get workspace_id for proper FK lookup
UPDATE leave_balances lb
SET leave_type_id = lt.id
FROM leave_types lt
JOIN employees e ON e.id = lb.employee_id
WHERE lt.workspace_id = e.workspace_id
  AND lt.code = lb.leave_type
  AND lb.leave_type_id IS NULL;

-- 3. Handle orphan records in leave_balances (map to 'vacation')
UPDATE leave_balances lb
SET leave_type_id = lt.id
FROM leave_types lt
JOIN employees e ON e.id = lb.employee_id
WHERE lt.workspace_id = e.workspace_id
  AND lt.code = 'vacation'
  AND lb.leave_type_id IS NULL;

-- 4. Backfill leave_requests.leave_type_id (already exists, ensure populated)
UPDATE leave_requests lr
SET leave_type_id = lt.id
FROM leave_types lt
JOIN employees e ON e.id = lr.employee_id
WHERE lt.workspace_id = e.workspace_id
  AND lt.code = lr.leave_type
  AND lr.leave_type_id IS NULL;

-- 5. Handle orphan records in leave_requests
UPDATE leave_requests lr
SET leave_type_id = lt.id
FROM leave_types lt
JOIN employees e ON e.id = lr.employee_id
WHERE lt.workspace_id = e.workspace_id
  AND lt.code = 'vacation'
  AND lr.leave_type_id IS NULL;

-- 6. Backfill leave_accrual_logs.leave_type_id
UPDATE leave_accrual_logs lal
SET leave_type_id = lt.id
FROM leave_types lt
WHERE lt.workspace_id = lal.workspace_id
  AND lt.code = lal.leave_type
  AND lal.leave_type_id IS NULL;

-- 7. Handle orphan records in leave_accrual_logs
UPDATE leave_accrual_logs lal
SET leave_type_id = lt.id
FROM leave_types lt
WHERE lt.workspace_id = lal.workspace_id
  AND lt.code = 'vacation'
  AND lal.leave_type_id IS NULL;

-- 8. Create indexes for new FK
CREATE INDEX IF NOT EXISTS idx_leave_balances_leave_type_id ON leave_balances(leave_type_id);

-- 9. Make FK columns NOT NULL
ALTER TABLE leave_balances ALTER COLUMN leave_type_id SET NOT NULL;
ALTER TABLE leave_requests ALTER COLUMN leave_type_id SET NOT NULL;
ALTER TABLE leave_accrual_logs ALTER COLUMN leave_type_id SET NOT NULL;

-- 10. Update unique constraint on leave_balances
-- Old: (employee_id, leave_type, year)
-- New: (employee_id, leave_type_id, year)
ALTER TABLE leave_balances DROP CONSTRAINT IF EXISTS leave_balances_employee_id_leave_type_year_key;
ALTER TABLE leave_balances ADD CONSTRAINT leave_balances_employee_leave_type_id_year_key
    UNIQUE (employee_id, leave_type_id, year);

-- 11. Update unique constraint on leave_accrual_logs
ALTER TABLE leave_accrual_logs DROP CONSTRAINT IF EXISTS leave_accrual_logs_employee_id_leave_type_accrual_date_key;
ALTER TABLE leave_accrual_logs ADD CONSTRAINT leave_accrual_logs_employee_leave_type_id_date_key
    UNIQUE (employee_id, leave_type_id, accrual_date);

-- 12. Drop legacy string columns
ALTER TABLE leave_balances DROP COLUMN leave_type;
ALTER TABLE leave_requests DROP COLUMN leave_type;
ALTER TABLE leave_accrual_logs DROP COLUMN leave_type;
```

### Step 2: Generate SQLx Metadata
After migration runs, regenerate SQLx query cache:
```bash
cd apiserver && cargo sqlx prepare
```

## Todo List
- [ ] Create migration file with proper timestamp
- [ ] Test migration on local database
- [ ] Run `cargo sqlx prepare` to update `.sqlx/` cache
- [ ] Verify no orphan records remain

## Success Criteria
- Migration runs without errors
- All `leave_type_id` columns are NOT NULL
- No `leave_type` VARCHAR columns exist
- Unique constraints updated
- SQLx metadata regenerated

## Risk Assessment
- **Orphan Records**: Handled by mapping to 'vacation' type
- **Constraint Violation**: Transaction ensures atomicity
- **Data Loss**: None - FK populated before string column dropped

## Security Considerations
- FK constraints prevent referencing non-existent leave types
- Workspace isolation maintained via leave_types.workspace_id join

## Next Steps
After migration completes, proceed to Phase 2 (Backend Repository Layer).
