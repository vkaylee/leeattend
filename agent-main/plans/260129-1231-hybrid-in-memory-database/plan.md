---
title: "Hybrid In-Memory + Disk Database Architecture"
description: "Implement hybrid database with in-memory primary storage and periodic disk sync for ~500ms sync latency"
status: pending
priority: P1
effort: 16h
branch: main
tags: [performance, database, architecture, rust, sqlite]
created: 2026-01-29
---

# Hybrid In-Memory + Disk Database Architecture

## Overview

Transform the current disk-only SQLite architecture into a hybrid in-memory + disk system to achieve ~500ms sync latency (down from ~10s).

### Current Architecture
```
Device --> Queue --> DB Writer (batch 100/200ms) --> SQLite Disk --> Sync Loop (10s) --> Server
                                                           ^
                                                    [BOTTLENECK: ~10s latency]
```

### Target Architecture
```
Device --> Queue --> In-Memory DB (<1ms write) --> Sync Loop (500ms) --> Server
                           |
                           +--> Periodic Flush (5-10s) --> SQLite Disk (backup/recovery)
```

## Phases

| Phase | Description | Status | Effort |
|-------|-------------|--------|--------|
| 1 | Design & Memory Model | pending | 2h |
| 2 | In-Memory Store Implementation | pending | 4h |
| 3 | Sync Loop Optimization | pending | 3h |
| 4 | Disk Persistence Layer | pending | 3h |
| 5 | Crash Recovery | pending | 2h |
| 6 | Testing & Validation | pending | 2h |

## Key Dependencies

- SQLx 0.8 (already in use)
- tokio async runtime (already in use)
- chrono for timestamps (already in use)

## Phase Details

- [Phase 1: Design & Memory Model](./phase-01-design-memory-model.md)
- [Phase 2: In-Memory Store Implementation](./phase-02-in-memory-store.md)
- [Phase 3: Sync Loop Optimization](./phase-03-sync-loop-optimization.md)
- [Phase 4: Disk Persistence Layer](./phase-04-disk-persistence.md)
- [Phase 5: Crash Recovery](./phase-05-crash-recovery.md)
- [Phase 6: Testing & Validation](./phase-06-testing-validation.md)

## Success Criteria

1. Sync latency reduced to ~500ms (measured end-to-end)
2. Memory usage bounded (<100MB for 10K pending logs)
3. No data loss on graceful shutdown
4. Maximum 5-10s data loss window on crash
5. All 100 existing tests pass
6. Backward compatible with existing migrations

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Memory overflow | High | Medium | Bounded queue, backpressure |
| Data loss on crash | High | Low | Acceptable window, disk checkpoints |
| SQLite lock contention | Medium | Low | Single writer, async flush |
| Test regression | Medium | Medium | Comprehensive test coverage |
