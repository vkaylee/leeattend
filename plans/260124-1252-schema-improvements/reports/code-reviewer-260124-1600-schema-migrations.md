# Code Review: Schema Design Improvements

## Scope
- Files reviewed: 4 migration files
- Lines analyzed: ~150 SQL lines
- Focus: Schema migration correctness, security, performance
- Plan: /home/elt1541/leeattend/plans/260124-1252-schema-improvements/plan.md

## Overall Assessment

**Score: 8.5/10**

Migrations well-structured, idempotent, preserve data. Good multi-tenant security. Minor improvements needed for production safety.

## Critical Issues

**None found** - No security vulnerabilities or data loss risks.

## High Priority Findings

### 1. Migration 20260124170000 - Data Migration Risk
**Location**: `create_leave_types.sql` lines 44-58

**Issue**: Data migration assumes all employees have workspace_id accessible through join. If employee's workspace_id is NULL or orphaned, migration will silently skip records.

**Impact**: Partial data migration, inconsistent state

**Fix**: Add validation/logging
```sql
-- After UPDATE leave_requests (line 51)
DO $$
DECLARE
  unmigrated_count INT;
BEGIN
  SELECT COUNT(*) INTO unmigrated_count
  FROM leave_requests
  WHERE leave_type_id IS NULL;

  IF unmigrated_count > 0 THEN
    RAISE WARNING 'leave_requests: % records not migrated', unmigrated_count;
  END IF;
END $$;

-- Similar for leave_accrual_logs
```

### 2. Migration 20260124172000 - No FK Validation
**Location**: `standardize_payslip_incentives_pk.sql`

**Issue**: No verification that payslip_id FKs remain valid after PK change. Although payslips table isn't shown to have FKs TO payslip_incentives, best practice is to verify.

**Impact**: Potential FK constraint violations if assumptions wrong

**Fix**: Already mitigated (verified no external FKs reference this table in Rust code). Document assumption.

### 3. Missing Concurrent Index Creation
**Location**: Migrations 20260124171000, 20260124173000

**Issue**: Index creation without CONCURRENTLY may lock tables during creation on large datasets.

**Impact**: Downtime during deployment on production with significant data

**Recommendation**: For production, use
```sql
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_name ON table(col);
```

**Note**: CONCURRENTLY cannot run in transaction block, needs separate migration or manual execution.

## Medium Priority Improvements

### 4. Inconsistent NOT NULL Handling
**Location**: `create_leave_types.sql` lines 41-42

**Finding**: New FK columns added as nullable, but migration populates them. Missing step to add NOT NULL constraint after migration.

**Current**:
```sql
ALTER TABLE leave_requests ADD COLUMN IF NOT EXISTS leave_type_id UUID REFERENCES...
```

**Suggested**: Add in future migration or same file
```sql
-- After data migration completes
ALTER TABLE leave_requests ALTER COLUMN leave_type_id SET NOT NULL;
ALTER TABLE leave_accrual_logs ALTER COLUMN leave_type_id SET NOT NULL;
```

**Reason**: Phase plan mentions this (step 6) but implementation omits it. Keeping nullable is acceptable for backward compatibility as noted in comment, but should be documented in phase file.

### 5. Leave Type Code Validation
**Location**: `create_leave_types.sql` line 7

**Finding**: No CHECK constraint on code format (e.g., lowercase, alphanumeric)

**Recommendation**: Add constraint to prevent future issues
```sql
code VARCHAR(50) NOT NULL CHECK (code ~ '^[a-z][a-z0-9_]*$'),
```

### 6. Missing Down Migrations
**Finding**: All down migrations commented out

**Reason**: Acceptable for irreversible migrations (data migration, PK changes), but should be noted in deployment docs.

**Recommendation**: Document migration irreversibility in deployment guide.

## Low Priority Suggestions

### 7. Compound Index Selectivity
**Location**: `add_compound_indexes.sql` line 5

**Observation**: Index on (workspace_id, is_active) may have low selectivity if most employees are active.

**Recommendation**: Monitor index usage with
```sql
SELECT * FROM pg_stat_user_indexes WHERE indexrelname = 'idx_employees_workspace_active';
```

If rarely used, consider removing. But likely useful for "show only inactive employees" admin queries.

### 8. Leave Types Seeding
**Location**: `create_leave_types.sql` lines 27-38

**Observation**: Hardcoded default types might not fit all locales (e.g., no "bereavement", "study leave")

**Recommendation**: Fine as default. Workspaces can add custom types. Consider documenting customization in API.

### 9. Index Naming Consistency
**Finding**: Index names inconsistent
- `idx_leave_types_workspace_id` (full table name)
- `idx_leave_requests_approved_by` (abbreviated column)

**Recommendation**: Standardize to `idx_<table>_<col1>_<col2>` format. Current naming acceptable, just note for future.

## Positive Observations

1. **Idempotency**: All migrations use `IF NOT EXISTS` - excellent for replayability
2. **Multi-tenant Security**: Proper workspace_id scoping in leave_types with UNIQUE constraint
3. **Cascade Rules**: Appropriate ON DELETE CASCADE/RESTRICT usage
4. **Data Preservation**: Keeps legacy VARCHAR columns for backward compatibility
5. **Compound Index**: Smart optimization for common query pattern
6. **Transaction Safety**: Migrations atomic (except if using CONCURRENTLY)

## Recommended Actions

### Immediate (Before Merge)
1. Add data migration validation in 20260124170000 (or accept risk if dataset small)
2. Update phase files to mark TODO items as complete
3. Document NOT NULL constraint decision (keep nullable vs add constraint)

### Before Production Deploy
1. Review index creation strategy (CONCURRENTLY vs downtime window)
2. Test migrations on production-size dataset replica
3. Add monitoring for query performance post-migration
4. Update deployment guide with rollback strategy

### Post-Deploy
1. Run `cargo sqlx prepare` to update metadata
2. Monitor pg_stat_user_indexes for new indexes
3. Verify application queries use new leave_type_id FK
4. Plan deprecation timeline for legacy VARCHAR columns

## Metrics

- **Idempotency**: ✅ 100% (all use IF NOT EXISTS)
- **Security**: ✅ Multi-tenant isolation enforced
- **Backward Compatibility**: ✅ Legacy columns preserved
- **Data Safety**: ⚠️ Migration assumes clean data (employee.workspace_id always valid)
- **Performance Impact**: ✅ Index creation, minimal write overhead

## YAGNI/KISS/DRY Assessment

✅ **YAGNI**: No over-engineering. All changes address identified schema issues.
✅ **KISS**: Straightforward migrations, clear intent.
✅ **DRY**: leave_types eliminates repeated VARCHAR validation logic.

## Plan Status Update

All migration files created as specified in phases 01-04. Ready for Phase 05 (SQLx prepare).

### Phase Checklist
- [x] Phase 01: leave_types table created
- [x] Phase 02: FK indexes added
- [x] Phase 03: PK standardized to UUID
- [x] Phase 04: Compound index added
- [ ] Phase 05: Run `cargo sqlx prepare`

## Unresolved Questions

1. Should NOT NULL constraint be added to leave_type_id in future migration, or keep nullable permanently?
2. What is downtime tolerance for production deploy (affects CONCURRENTLY decision)?
3. Are there any external systems that reference payslip_incentives.id (API responses, webhooks)?
4. Timeline for deprecating legacy leave_type VARCHAR columns?
