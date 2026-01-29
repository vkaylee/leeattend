# Phase 01: Database Schema

## Context Links
- [Scout Report](/home/elt1541/leeattend/plans/reports/scouter-260128-2047-admin-ot-claims.md)
- [Existing Migration](/home/elt1541/leeattend/apiserver/migrations/20260128000000_create_attendance_daily_summaries.sql)
- [Models](/home/elt1541/leeattend/apiserver/src/models.rs)

## Overview
- **Priority**: P1 (blocker for all other phases)
- **Status**: Pending
- **Effort**: 1 hour

Add columns to track paper form entry metadata in existing `attendance_daily_summaries` table.

## Key Insights

From scout report:
- Table already has `employee_id`, `date`, `claimed_ot_minutes`, `ot_status`, `ot_reason`, `manager_note`
- UNIQUE constraint on `(employee_id, date)` means we upsert, not insert duplicate
- No existing column tracks "who entered" or "entry method"

## Requirements

### Functional
- Track which user entered the paper form claim
- Track when the claim was entered (separate from `created_at` which is system timestamp)
- Track entry method (paper_form vs self_service)
- Optional: store photo URL of scanned paper form

### Non-Functional
- Backward compatible (all new columns nullable or have defaults)
- No index needed for new columns (queried rarely)

## Schema Changes

### New Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| `claim_entered_by_user_id` | UUID | YES | NULL | User who entered paper claim |
| `claim_entered_at` | TIMESTAMPTZ | YES | NULL | When claim was entered |
| `paper_form_photo_url` | TEXT | YES | NULL | URL to uploaded photo |
| `entered_via` | TEXT | YES | 'self_service' | Entry method |

### Migration SQL

```sql
-- Migration: 20260129000000_add_paper_form_entry_columns.sql

ALTER TABLE attendance_daily_summaries
ADD COLUMN IF NOT EXISTS claim_entered_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS claim_entered_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS paper_form_photo_url TEXT,
ADD COLUMN IF NOT EXISTS entered_via TEXT DEFAULT 'self_service';

-- Add comment for documentation
COMMENT ON COLUMN attendance_daily_summaries.claim_entered_by_user_id IS 'User ID who entered the paper form claim (NULL if self-service)';
COMMENT ON COLUMN attendance_daily_summaries.entered_via IS 'Entry method: self_service | paper_form';
```

### Rollback SQL

```sql
-- Down Migration
ALTER TABLE attendance_daily_summaries
DROP COLUMN IF EXISTS claim_entered_by_user_id,
DROP COLUMN IF EXISTS claim_entered_at,
DROP COLUMN IF EXISTS paper_form_photo_url,
DROP COLUMN IF EXISTS entered_via;
```

## Related Code Files

### To Modify
- `/home/elt1541/leeattend/apiserver/src/models.rs` - Add new fields to `AttendanceDailySummary` struct

### To Create
- `/home/elt1541/leeattend/apiserver/migrations/20260129000000_add_paper_form_entry_columns.sql`

## Implementation Steps

1. [ ] Create migration file `20260129000000_add_paper_form_entry_columns.sql`
2. [ ] Update `AttendanceDailySummary` struct in `models.rs`:
   ```rust
   pub struct AttendanceDailySummary {
       // ... existing fields
       pub claim_entered_by_user_id: Option<Uuid>,
       pub claim_entered_at: Option<DateTime<Utc>>,
       pub paper_form_photo_url: Option<String>,
       pub entered_via: Option<String>,
   }
   ```
3. [ ] Update all `sqlx::query_as!` calls for `AttendanceDailySummary` to include new columns
4. [ ] Run migration: `sqlx migrate run`
5. [ ] Verify with psql: `\d attendance_daily_summaries`

## Todo List

- [ ] Write migration SQL
- [ ] Update Rust model
- [ ] Update all SELECT queries
- [ ] Run migration
- [ ] Test rollback

## Success Criteria

- [ ] Migration runs without error
- [ ] `\d attendance_daily_summaries` shows 4 new columns
- [ ] Existing data unaffected (all new columns NULL or default)
- [ ] `cargo build` succeeds with updated model

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Migration fails on production | High | Test on staging first, rollback ready |
| Query breakage | Medium | Search all `AttendanceDailySummary` usages |

## Security Considerations

- `claim_entered_by_user_id` references `users(id)` with `ON DELETE SET NULL` - if user deleted, audit trail preserved but anonymized
- No sensitive data stored (photo URL only, not actual file content)

## Next Steps

After this phase:
- Phase 02 needs new columns to store entry metadata
- Phase 03 needs TypeScript types updated
