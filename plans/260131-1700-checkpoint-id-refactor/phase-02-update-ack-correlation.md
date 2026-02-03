# Phase 2: Update ACK Correlation

## Context Links
- [Phase 1: Refactor Checkpoint](/home/elt1541/leeattend/plans/260131-1700-checkpoint-id-refactor/phase-01-refactor-checkpoint-insert.md)
- [Central Client](/home/elt1541/leeattend/agent-main/src/central_client.rs)
- [Hybrid Database](/home/elt1541/leeattend/agent-main/src/hybrid_database.rs)
- [Memory Store](/home/elt1541/leeattend/agent-main/src/memory_store.rs)

## Overview
- **Priority:** P1 Critical
- **Status:** Pending (blocked by Phase 1)
- **Effort:** 2h

Refactor ACK correlation from using numeric `id` to using composite key `(device_id, user_id, timestamp)`.

## Key Insights

1. Current ACK flow: `msg_id: log.id.to_string()` → Server → ACK `msg_id` → `mark_synced(&[id])`
2. Problem: Memory IDs no longer match disk IDs after Phase 1
3. Solution: Use composite key as `msg_id`, maintain memory_id -> composite_key mapping

## Requirements

### Functional
- ACK messages use composite key instead of numeric ID
- Memory store can lookup logs by composite key
- `mark_synced` accepts composite keys

### Non-Functional
- ACK processing remains O(1)
- No memory overhead for mapping (reuse existing signature logic)

## Architecture

```
SEND FLOW:
┌──────────────┐      msg_id="DEV1:U001:1706767200"     ┌──────────────┐
│ Memory Store │ ──────────────────────────────────────> │ Central API  │
│ mem_id=1001  │                                         │              │
│ composite_key│                                         └──────────────┘
└──────────────┘

ACK FLOW:
┌──────────────┐      ACK msg_id="DEV1:U001:1706767200" ┌──────────────┐
│ Memory Store │ <────────────────────────────────────── │ Central API  │
│ lookup by key│                                         │              │
│ mark_synced  │                                         └──────────────┘
└──────────────┘
```

## Related Code Files

### Modify
- `/home/elt1541/leeattend/agent-main/src/central_client.rs` (lines 126-151, 286-291)
- `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs` (lines 68-98)
- `/home/elt1541/leeattend/agent-main/src/memory_store.rs` (lines 359-405)

### Create
- None (add methods to existing modules)

## Implementation Steps

### Step 1: Add composite key helper to MemoryLog (memory_store.rs)

Add method to generate canonical composite key string:

```rust
impl MemoryLog {
    /// Generate composite key string for ACK correlation
    /// Format: "device_id:user_id:timestamp_unix"
    pub fn composite_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.device_id,
            self.user_id,
            self.timestamp.and_utc().timestamp()
        )
    }
}
```

### Step 2: Add composite key index to InMemoryStore (memory_store.rs)

Add DashMap for O(1) lookup by composite key:

```rust
pub struct InMemoryStore {
    logs: DashMap<i64, MemoryLog>,
    signatures: DashSet<u64>,
    composite_index: DashMap<String, i64>,  // NEW: composite_key -> memory_id
    // ... rest unchanged
}
```

Update `insert` to populate index:

```rust
// In insert() after self.logs.insert(id, log):
let composite_key = format!(
    "{}:{}:{}",
    device_id,
    data.user_id,
    timestamp.and_utc().timestamp()
);
self.composite_index.insert(composite_key, id);
```

Update `load_from_disk` similarly:

```rust
// After self.logs.insert(log.id, memory_log):
let composite_key = format!(
    "{}:{}:{}",
    log.device_id,
    log.user_id,
    log.timestamp.and_utc().timestamp()
);
self.composite_index.insert(composite_key, log.id);
```

### Step 3: Add lookup by composite key (memory_store.rs)

```rust
/// Get memory ID by composite key
pub fn get_id_by_composite_key(&self, composite_key: &str) -> Option<i64> {
    self.composite_index.get(composite_key).map(|entry| *entry)
}

/// Mark synced by composite key (for ACK handling)
pub fn mark_synced_by_key(&self, composite_key: &str) -> bool {
    if let Some(id) = self.get_id_by_composite_key(composite_key) {
        self.mark_synced(&[id]);
        true
    } else {
        false
    }
}

/// Reset to unsynced by composite key (for failed ACK)
pub fn reset_to_unsynced_by_key(&self, composite_key: &str) -> bool {
    if let Some(id) = self.get_id_by_composite_key(composite_key) {
        self.reset_to_unsynced(id);
        true
    } else {
        false
    }
}
```

### Step 4: Update central_client.rs to use composite key (lines 134-136)

```rust
// BEFORE
for log in logs {
    let msg = CentralMessage::Attendance {
        msg_id: log.id.to_string(),  // Numeric ID
        // ...
    };
}

// AFTER
for log in logs {
    let msg = CentralMessage::Attendance {
        msg_id: log.composite_key(),  // Composite key string
        // ...
    };
}
```

### Step 5: Update ACK handler (central_client.rs:286-291)

```rust
// BEFORE
CentralMessage::Ack { msg_id, status, .. } => {
    if let Ok(id) = msg_id.parse::<i64>() {
        if status.as_deref() == Some("success") {
            hybrid_db.mark_synced(&[id]);
        }
    }
}

// AFTER
CentralMessage::Ack { msg_id, status, .. } => {
    // msg_id is now composite key "device_id:user_id:timestamp_unix"
    if status.as_deref() == Some("success") {
        if !hybrid_db.mark_synced_by_key(&msg_id) {
            warn!("ACK for unknown log: {}", msg_id);
        }
    } else {
        // Failed ACK - reset to unsynced for retry
        if !hybrid_db.reset_to_unsynced_by_key(&msg_id) {
            warn!("Failed ACK for unknown log: {}", msg_id);
        }
    }
}
```

### Step 6: Add HybridDatabase wrapper methods (hybrid_database.rs)

```rust
/// Mark synced by composite key
pub fn mark_synced_by_key(&self, composite_key: &str) -> bool {
    self.memory.mark_synced_by_key(composite_key)
}

/// Reset to unsynced by composite key
pub fn reset_to_unsynced_by_key(&self, composite_key: &str) -> bool {
    self.memory.reset_to_unsynced_by_key(composite_key)
}
```

### Step 7: Update hybrid_database.rs checkpoint caller (lines 83-98)

Update to handle new return type from Phase 1:

```rust
pub async fn checkpoint(&self) -> Result<usize, sqlx::Error> {
    let unpersisted = self.memory.get_unpersisted();
    if unpersisted.is_empty() {
        return Ok(0);
    }

    // Phase 1 changed return type to include composite keys
    let (inserted, _skipped, _keys) = self.disk.checkpoint_batch(&unpersisted).await?;

    // Mark as persisted - use memory IDs (internal tracking)
    let ids: Vec<i64> = unpersisted.iter().map(|l| l.id).collect();
    self.memory.mark_persisted(&ids);
    self.memory.clear_dirty();

    Ok(inserted)
}
```

### Step 8: Clean up composite_index on log removal (memory_store.rs)

Update `cleanup_synced` to remove from composite_index:

```rust
pub fn cleanup_synced(&self, max_age_secs: u64) -> usize {
    let threshold = chrono::Utc::now().naive_utc() - chrono::Duration::seconds(max_age_secs as i64);
    let mut to_remove = Vec::new();

    for entry in self.logs.iter() {
        if entry.is_synced == 1 && entry.created_at < threshold {
            to_remove.push((*entry.key(), entry.composite_key()));
        }
    }

    for (id, composite_key) in &to_remove {
        self.logs.remove(id);
        self.composite_index.remove(composite_key);
    }

    self.count.fetch_sub(to_remove.len(), Ordering::SeqCst);
    to_remove.len()
}
```

## Todo List

- [ ] Add `composite_key()` method to MemoryLog
- [ ] Add `composite_index: DashMap<String, i64>` to InMemoryStore
- [ ] Populate composite_index in `insert()` and `load_from_disk()`
- [ ] Add `get_id_by_composite_key()`, `mark_synced_by_key()`, `reset_to_unsynced_by_key()`
- [ ] Update `central_client.rs` to use composite key for msg_id
- [ ] Update ACK handler to parse composite key
- [ ] Add HybridDatabase wrapper methods
- [ ] Update `cleanup_synced` to clean composite_index
- [ ] Run unit tests: `./run-tests.sh --api memory`

## Success Criteria

- [ ] ACK messages correctly mark logs as synced using composite key
- [ ] Failed ACKs reset logs to unsynced for retry
- [ ] Memory cleanup removes stale entries from composite_index
- [ ] All existing tests pass

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Composite key collision | Use device_id:user_id:unix_timestamp - same uniqueness as UNIQUE index |
| Server expects numeric msg_id | Server already treats msg_id as opaque string |
| Memory overhead | DashMap<String, i64> adds ~100 bytes per log (acceptable) |

## Security Considerations

- Composite key exposes same data already in API payload (device_id, user_id, timestamp)
- No additional data exposure

## Next Steps

After Phase 2 complete:
- Phase 3: Testing and Validation
