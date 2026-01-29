# Phase 1: Design & Memory Model

## Context Links
- [Current database.rs](/home/elt1541/leeattend/agent-main/src/database.rs)
- [Current main.rs](/home/elt1541/leeattend/agent-main/src/main.rs)
- [Current central_client.rs](/home/elt1541/leeattend/agent-main/src/central_client.rs)

## Overview
- **Priority**: P1 (Critical)
- **Status**: pending
- **Effort**: 2h
- **Description**: Design the hybrid memory/disk architecture and data partitioning strategy

## Key Insights

### Current Bottleneck Analysis
1. **Write Path**: Device -> Queue (10K buffer) -> DB Writer -> SQLite disk = ~10-50ms per batch
2. **Read Path**: Sync Loop -> SQLite disk query -> Network = dominated by disk I/O
3. **Sync Interval**: 10s for normal, 500ms when backlogged
4. **Lock Contention**: WAL mode helps but disk I/O still blocks

### Performance Target Breakdown
| Operation | Current | Target | Strategy |
|-----------|---------|--------|----------|
| Log Write | 10-50ms | <1ms | In-memory HashMap |
| Log Read (unsynced) | 5-20ms | <1ms | In-memory iteration |
| Disk Checkpoint | N/A | 5-10s | Background async |
| Sync to Server | 10s | 500ms | Direct from memory |

## Requirements

### Functional Requirements
1. All attendance logs stored in memory first
2. Periodic flush to disk every 5-10 seconds
3. Startup loads disk data into memory
4. Graceful shutdown flushes all in-memory data
5. Status tracking: `0=unsynced`, `1=synced`, `2=in_flight`

### Non-Functional Requirements
1. Memory bounded: max 100K logs (~50MB estimated)
2. Thread-safe concurrent access
3. Lock-free reads where possible
4. Minimal allocation during hot path

## Architecture

### Data Structures

```rust
// Core in-memory store
pub struct InMemoryStore {
    // Primary storage: Log ID -> Log Data
    logs: DashMap<i64, MemoryLog>,

    // Index: Unsynced logs (is_synced=0) for fast sync queries
    // Sorted by ID (FIFO order)
    unsynced_index: RwLock<BTreeSet<i64>>,

    // Index: In-flight logs (is_synced=2) for timeout tracking
    inflight_index: RwLock<HashMap<i64, Instant>>,

    // ID generator (atomic counter, seeded from disk max_id)
    next_id: AtomicI64,

    // Dirty flag for checkpoint optimization
    dirty: AtomicBool,

    // Backpressure: current count
    count: AtomicUsize,
}

#[derive(Clone)]
pub struct MemoryLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: NaiveDateTime,
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub is_synced: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub retry_count: i64,
    // Checkpoint flag: false = not yet persisted to disk
    pub persisted: bool,
}
```

### Component Interactions

```
                    +-----------------+
                    |  InMemoryStore  |
                    |   (DashMap)     |
                    +--------+--------+
                             |
         +-------------------+-------------------+
         |                   |                   |
    +----v----+        +-----v-----+       +-----v-----+
    | Writer  |        | Sync Loop |       | Flush     |
    | (mpsc)  |        | (500ms)   |       | (5-10s)   |
    +---------+        +-----------+       +-----+-----+
         |                   |                   |
         |                   |                   v
         |                   |             +-----+-----+
         |                   +------------>| SQLite    |
         |                                 | (Disk)    |
         +---------------------------------+           |
                   (startup load)          +-----------+
```

### Memory Estimation

| Component | Per-Log Size | 10K Logs | 100K Logs |
|-----------|--------------|----------|-----------|
| MemoryLog struct | ~300 bytes | 3 MB | 30 MB |
| DashMap overhead | ~100 bytes | 1 MB | 10 MB |
| BTreeSet index | ~24 bytes | 240 KB | 2.4 MB |
| HashMap inflight | ~40 bytes | 400 KB | 4 MB |
| **Total** | ~464 bytes | ~5 MB | ~50 MB |

## Related Code Files

### Files to Modify
- `src/database.rs` - Add in-memory layer wrapper
- `src/main.rs` - Initialize hybrid store, add flush worker
- `src/central_client.rs` - Update sync loop to use memory store

### Files to Create
- `src/memory_store.rs` - In-memory store implementation (~200 lines)
- `src/hybrid_database.rs` - Hybrid wrapper combining memory + disk (~150 lines)

## Implementation Steps

1. [ ] Define `MemoryLog` struct with all fields from `RawAttendanceLog`
2. [ ] Design `InMemoryStore` with DashMap + indices
3. [ ] Define `HybridDatabase` wrapper interface (same as `Database`)
4. [ ] Document checkpoint file format (WAL-style or full dump)
5. [ ] Design startup sequence: disk -> memory load
6. [ ] Design shutdown sequence: memory -> disk flush
7. [ ] Define backpressure strategy (bounded queue, drop oldest synced)

## Todo List

- [ ] Document final data structure design
- [ ] Define HybridDatabase public API (match current Database)
- [ ] Memory estimation validation
- [ ] Checkpoint strategy decision (incremental vs full)
- [ ] Review with risk assessment

## Success Criteria

- [ ] Architecture diagram complete
- [ ] Memory budget confirmed (<100MB for 100K logs)
- [ ] API backward compatible with current Database
- [ ] Checkpoint interval decided (5s or 10s)

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| DashMap lock contention | Medium | Use sharded map, minimize critical sections |
| Memory fragmentation | Low | Preallocate where possible |
| Index inconsistency | High | Single writer pattern, atomic updates |

## Security Considerations
- No new security concerns (same data, different storage medium)
- Memory is more secure than disk (no persistent sensitive data)

## Next Steps
After design approval, proceed to Phase 2: In-Memory Store Implementation
