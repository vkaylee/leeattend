# Phase 1: Refactor Checkpoint INSERT

## Context Links
- [Debugger Report](/home/elt1541/leeattend/plans/reports/debugger-260131-1655-checkpoint-unique-constraint.md)
- [Database Module](/home/elt1541/leeattend/agent-main/src/database.rs)

## Overview
- **Priority:** P1 Critical
- **Status:** Pending
- **Effort:** 2h

Remove explicit `id` from checkpoint INSERT to let SQLite AUTOINCREMENT handle ID generation. This eliminates PRIMARY KEY collisions.

## Key Insights

1. Current INSERT explicitly binds `log.id` (line 577) causing collision with existing disk rows
2. `ON CONFLICT(device_id, user_id, timestamp)` only handles business duplicates, not PRIMARY KEY
3. Memory IDs serve two purposes: (a) in-flight tracking, (b) sync ACK correlation
4. Solution: decouple memory ID from disk ID

## Requirements

### Functional
- Checkpoint INSERT must not specify `id` column
- SQLite generates ID via AUTOINCREMENT
- Must return composite keys of inserted rows for ACK correlation

### Non-Functional
- No performance regression (batch INSERT still < 100ms for 1000 logs)
- Maintain transactional integrity

## Architecture

```
BEFORE:
┌──────────────┐      INSERT (id=1001)     ┌──────────────┐
│ Memory Store │ ─────────────────────────> │   SQLite     │
│ id=1001      │      ❌ UNIQUE failed      │ id=1001 exists│
└──────────────┘                            └──────────────┘

AFTER:
┌──────────────┐      INSERT (no id)       ┌──────────────┐
│ Memory Store │ ─────────────────────────> │   SQLite     │
│ mem_id=1001  │      ✅ Auto-generates    │ disk_id=5000 │
│ key=(d,u,t)  │      Returns: 0 or 1      │              │
└──────────────┘                            └──────────────┘
```

## Related Code Files

### Modify
- `/home/elt1541/leeattend/agent-main/src/database.rs` (lines 550-595)

### Read Only
- `/home/elt1541/leeattend/agent-main/migrations/20260108000000_init_schema.sql`

## Implementation Steps

### Step 1: Update `checkpoint_batch` SQL (database.rs:565-573)

Change INSERT statement to omit `id` column:

```rust
// BEFORE (line 566-572)
let query = format!(
    r#"
    INSERT INTO zkt_raw_attendance
        (id, device_id, user_id, timestamp, status, punch, is_synced, created_at, retry_count)
    VALUES {}
    ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
    "#,
    placeholders.join(",")
);

// AFTER
let query = format!(
    r#"
    INSERT INTO zkt_raw_attendance
        (device_id, user_id, timestamp, status, punch, is_synced, created_at, retry_count)
    VALUES {}
    ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
    "#,
    placeholders.join(",")
);
```

### Step 2: Update placeholder count (database.rs:562-564)

Reduce from 9 to 8 placeholders:

```rust
// BEFORE
let placeholders: Vec<&str> = chunk.iter()
    .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?, ?)")
    .collect();

// AFTER
let placeholders: Vec<&str> = chunk.iter()
    .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?)")
    .collect();
```

### Step 3: Remove `log.id` binding (database.rs:575-586)

```rust
// BEFORE
let mut q = sqlx::query(&query);
for log in chunk {
    q = q.bind(log.id)           // REMOVE THIS LINE
        .bind(&log.device_id)
        .bind(&log.user_id)
        .bind(log.timestamp)
        .bind(log.status)
        .bind(log.punch)
        .bind(log.is_synced)
        .bind(log.created_at)
        .bind(log.retry_count);
}

// AFTER
let mut q = sqlx::query(&query);
for log in chunk {
    q = q.bind(&log.device_id)
        .bind(&log.user_id)
        .bind(log.timestamp)
        .bind(log.status)
        .bind(log.punch)
        .bind(log.is_synced)
        .bind(log.created_at)
        .bind(log.retry_count);
}
```

### Step 4: Return inserted composite keys (database.rs:550-595)

Change return type to include composite keys for ACK correlation:

```rust
// Define composite key type
pub type CompositeKey = (String, String, chrono::NaiveDateTime); // (device_id, user_id, timestamp)

// BEFORE
pub async fn checkpoint_batch(&self, logs: &[crate::memory_store::MemoryLog]) -> Result<(usize, usize), Error>

// AFTER
pub async fn checkpoint_batch(&self, logs: &[crate::memory_store::MemoryLog])
    -> Result<(usize, usize, Vec<CompositeKey>), Error>
```

Build composite keys before INSERT:

```rust
pub async fn checkpoint_batch(&self, logs: &[crate::memory_store::MemoryLog])
    -> Result<(usize, usize, Vec<CompositeKey>), Error> {
    if logs.is_empty() {
        return Ok((0, 0, vec![]));
    }

    // Collect composite keys for all logs being inserted
    let composite_keys: Vec<CompositeKey> = logs.iter()
        .map(|l| (l.device_id.clone(), l.user_id.clone(), l.timestamp))
        .collect();

    let mut tx = self.pool.begin().await?;
    let mut inserted = 0;
    let mut skipped = 0;

    // ... existing batch INSERT logic (modified per steps 1-3) ...

    tx.commit().await?;

    // Return all composite keys (caller filters by rows_affected if needed)
    Ok((inserted, skipped, composite_keys))
}
```

## Todo List

- [ ] Update SQL statement to remove `id` column
- [ ] Update placeholder count from 9 to 8
- [ ] Remove `log.id` binding
- [ ] Change return type to include composite keys
- [ ] Update all callers of `checkpoint_batch`
- [ ] Run unit tests: `./run-tests.sh --api checkpoint`

## Success Criteria

- [ ] Checkpoint INSERT succeeds even when disk has overlapping IDs
- [ ] `rows_affected` correctly counts inserted rows
- [ ] No compile errors
- [ ] Existing `checkpoint_batch` tests pass

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Caller expects old return type | Search all usages, update in Phase 1 |
| Performance impact of collecting keys | Negligible - just clone existing strings |

## Security Considerations

- No auth/data exposure changes
- Composite key contains same data already in INSERT, no new exposure

## Next Steps

After Phase 1 complete:
- Phase 2: Update `hybrid_database.rs` and `central_client.rs` for ACK correlation
