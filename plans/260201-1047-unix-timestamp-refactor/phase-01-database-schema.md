# Phase 1: Database Schema Changes

## Context Links
- Plan: [plan.md](./plan.md)
- Current Schema: `agent-main/migrations/20260108000000_init_schema.sql`

## Overview
- **Priority**: P1 (Must complete first)
- **Status**: pending
- **Effort**: 1h

Modify the initial migration to change timestamp column type and add timezone to devices.

## Key Insights

1. **SQLite Type Affinity**: `INTEGER` for Unix timestamps is efficient and sortable
2. **IANA Timezone Format**: Use standard format like 'Asia/Ho_Chi_Minh', 'UTC', 'America/New_York'
3. **Default Timezone**: 'UTC' ensures graceful fallback if device config missing

## Requirements

### Functional
- `zkt_raw_attendance.timestamp` column type: `INTEGER` (was `DATETIME`)
- `devices.timezone` column: `TEXT NOT NULL DEFAULT 'UTC'`
- All existing indices remain functional with INTEGER

### Non-functional
- Migration must be idempotent
- Schema should be readable and documented

## Architecture

```sql
-- Before
CREATE TABLE zkt_raw_attendance (
    ...
    timestamp DATETIME NOT NULL,      -- String format
    ...
);

-- After
CREATE TABLE zkt_raw_attendance (
    ...
    timestamp INTEGER NOT NULL,       -- Unix epoch seconds
    ...
);
```

## Related Code Files

### Modify
- `agent-main/migrations/20260108000000_init_schema.sql`

### No Changes Required
- Other migrations (unique index still works with INTEGER)

## Implementation Steps

### Step 1: Backup current migration (optional)
```bash
cp migrations/20260108000000_init_schema.sql migrations/20260108000000_init_schema.sql.bak
```

### Step 2: Modify zkt_raw_attendance table

Change line 6 from:
```sql
timestamp DATETIME NOT NULL,
```
To:
```sql
timestamp INTEGER NOT NULL,  -- Unix timestamp (UTC seconds since epoch)
```

### Step 3: Add timezone column to devices table

Add after line 25 (before closing parenthesis):
```sql
    timezone TEXT NOT NULL DEFAULT 'UTC'  -- IANA timezone (e.g., 'Asia/Ho_Chi_Minh')
```

### Step 4: Verify index compatibility

The existing index:
```sql
CREATE INDEX IF NOT EXISTS idx_timestamp ON zkt_raw_attendance(timestamp);
```
Works correctly with INTEGER type (better B-tree efficiency).

The unique constraint in `20260126000001_add_unique_index.sql`:
```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_log ON zkt_raw_attendance(device_id, user_id, timestamp);
```
Also works with INTEGER.

## Todo List

- [ ] Modify timestamp column type in init_schema.sql
- [ ] Add timezone column to devices table
- [ ] Verify all indices compatible with INTEGER
- [ ] Delete local SQLite database for fresh start
- [ ] Test migration runs without error

## Success Criteria

1. Migration executes without error on fresh database
2. `PRAGMA table_info(zkt_raw_attendance)` shows timestamp as INTEGER
3. `PRAGMA table_info(devices)` shows timezone column exists
4. All indices created successfully

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Index incompatibility | Low | Medium | INTEGER is better for B-tree |
| Migration syntax error | Low | Low | Test on fresh DB |
| Existing data loss | N/A | N/A | DB reset acceptable |

## Security Considerations
- No security impact (schema change only)

## Next Steps
After this phase, proceed to [Phase 2: Dependencies](./phase-02-dependencies.md)
