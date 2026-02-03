# Phase 1: Database Migration

## Overview
Create new `attendance_staging_logs` table and add fingerprint column to `attendance_log_errors`.

## Priority
High - Foundation for all other phases

## Related Files
- `apiserver/migrations/` - New migration file

## Migration File
Create: `apiserver/migrations/20260129220000_add_staging_and_fingerprint.sql`

## Implementation

### Action A: New Table `attendance_staging_logs`
```sql
CREATE TABLE IF NOT EXISTS attendance_staging_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    device_id TEXT,
    raw_user_id TEXT NOT NULL,  -- Unknown user ID from device
    punch_time TIMESTAMPTZ NOT NULL,
    raw_data JSONB,
    fingerprint TEXT NOT NULL,  -- Hash(device + raw_user + time + punch + status)
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, processed, dismissed
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Unique index for deduplication
CREATE UNIQUE INDEX idx_staging_fingerprint
ON attendance_staging_logs(workspace_id, fingerprint);

-- Index for querying pending logs
CREATE INDEX idx_staging_pending
ON attendance_staging_logs(workspace_id, status) WHERE status = 'pending';
```

### Action B: Update `attendance_log_errors`
```sql
-- Add fingerprint column (nullable for existing rows)
ALTER TABLE attendance_log_errors
ADD COLUMN IF NOT EXISTS fingerprint TEXT;

-- Unique index for deduplication (partial - only where fingerprint exists)
CREATE UNIQUE INDEX idx_errors_fingerprint
ON attendance_log_errors(workspace_id, fingerprint)
WHERE fingerprint IS NOT NULL;
```

## Todo
- [ ] Create migration file
- [ ] Run migration locally
- [ ] Verify tables created correctly

## Success Criteria
- `attendance_staging_logs` table exists with all columns
- `attendance_log_errors.fingerprint` column exists
- Both unique indexes created
