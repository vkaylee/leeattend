# Research Report: Hybrid In-Memory Database Architecture

**Report ID**: planner-260129-1231-hybrid-db-research
**Date**: 2026-01-29
**Status**: Complete

## Executive Summary

Analyzed current Rust agent architecture for attendance sync system. Identified disk I/O as primary bottleneck causing ~10s sync latency. Proposed hybrid in-memory + disk architecture to achieve ~500ms target latency.

## Current State Analysis

### Architecture Review

| Component | File | Lines | Function |
|-----------|------|-------|----------|
| Database | src/database.rs | 973 | SQLite operations (direct disk) |
| Main | src/main.rs | 922 | Orchestrator, DB writer, workers |
| Sync Client | src/central_client.rs | 489 | WebSocket sync loop |
| Models | src/models.rs | 110 | Data structures |

### Current Data Flow

```
Device Polling (5s)
    --> mpsc channel (10K buffer)
    --> DB Writer Worker (batch 100/200ms)
    --> SQLite Disk (WAL mode)
    --> Sync Loop (10s normal, 500ms backlogged)
    --> WebSocket to Server
```

### Bottleneck Analysis

| Operation | Current Latency | Root Cause |
|-----------|-----------------|------------|
| Batch insert | 10-50ms | Disk I/O, transaction commit |
| Query unsynced | 5-20ms | Index scan on disk |
| Mark in-flight | 10-30ms | UPDATE + fsync |
| **Total per cycle** | **25-100ms** | Dominated by disk |

### SQLite Configuration (database.rs:20)
```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA busy_timeout=30000;
PRAGMA cache_size=-64000;
PRAGMA temp_store=MEMORY;
```
WAL mode helps but doesn't eliminate disk I/O for writes.

## Proposed Solution

### Hybrid Architecture

```
Device --> Queue --> InMemoryStore (<1ms) --> Sync Loop (500ms) --> Server
                           |
                           +--> Checkpoint Worker (5-10s) --> SQLite (durability)
```

### Key Components

1. **InMemoryStore**: DashMap-based concurrent store
   - O(1) insert/lookup
   - BTreeSet index for FIFO ordering
   - Atomic counters for backpressure

2. **HybridDatabase**: Wrapper combining memory + disk
   - Facade pattern matching current Database API
   - Transparent checkpoint management

3. **Checkpoint Worker**: Background persistence
   - 5-second interval (configurable)
   - Incremental (dirty-only) strategy
   - Full flush on shutdown

4. **Crash Recovery**: Startup reconciliation
   - Reset in-flight logs
   - ID gap for collision prevention
   - Crash marker file

### Memory Budget

| Capacity | Estimated Memory | Notes |
|----------|------------------|-------|
| 10K logs | ~5 MB | Normal operation |
| 50K logs | ~25 MB | Backlogged |
| 100K logs | ~50 MB | Maximum allowed |

### Performance Targets

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Insert latency | 10-50ms | <1ms | 10-50x |
| Query latency | 5-20ms | <1ms | 5-20x |
| Sync cycle | 10s | 500ms | 20x |
| End-to-end | 10-15s | <1s | 10-15x |

## Implementation Phases

### Phase 1: Design (2h)
- Define data structures
- Memory estimation
- API design

### Phase 2: In-Memory Store (4h)
- DashMap + indices
- Thread-safe operations
- Backpressure handling

### Phase 3: Sync Loop Optimization (3h)
- Read from memory
- Adaptive sleep timing
- ACK handling

### Phase 4: Disk Persistence (3h)
- Checkpoint worker
- Batch INSERT
- Graceful shutdown

### Phase 5: Crash Recovery (2h)
- Crash marker
- ID reconciliation
- In-flight reset

### Phase 6: Testing (2h)
- Unit tests
- Integration tests
- Performance benchmarks

**Total Effort**: 16 hours

## Dependencies

### New Crate Dependencies
```toml
dashmap = "6"       # Concurrent HashMap
parking_lot = "0.12" # Faster RwLock
```

### Existing Dependencies (Unchanged)
- sqlx 0.8 (SQLite)
- tokio (async runtime)
- chrono (timestamps)

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Memory overflow | High | Medium | Bounded capacity, backpressure |
| Data loss (crash) | Medium | Low | 5-10s acceptable window |
| ID collision | High | Low | 1000 ID gap on restart |
| Test regression | Medium | Medium | Comprehensive test coverage |
| Lock contention | Medium | Low | DashMap sharding |

## Backward Compatibility

- All existing migrations preserved
- Database schema unchanged
- API surface identical (HybridDatabase wraps Database)
- All 100 existing tests must pass

## Recommendations

1. **Implement in phases** - Each phase independently testable
2. **Feature flag** - Allow fallback to disk-only mode
3. **Monitoring** - Add metrics for memory usage, checkpoint latency
4. **Gradual rollout** - Test with single agent before fleet-wide

## Unresolved Questions

1. Should checkpoint interval be configurable via agent_config table?
2. Is 100K log capacity sufficient for all deployment scenarios?
3. Should failed checkpoints trigger alerts?
