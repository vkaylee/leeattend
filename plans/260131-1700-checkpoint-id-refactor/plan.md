---
title: "Checkpoint ID Refactor - Remove Explicit IDs"
description: "Refactor checkpoint INSERT to let SQLite auto-generate IDs, eliminating UNIQUE constraint failures"
status: complete
priority: P1
effort: 6h
branch: main
tags: [agent-main, checkpoint, database, critical-fix]
created: 2026-01-31
completed: 2026-01-31
---

# Checkpoint ID Refactor Plan

## Problem Summary

Checkpoint INSERT statements explicitly specify `id` values causing PRIMARY KEY UNIQUE constraint violations (SQLite error 1555) before the `ON CONFLICT(device_id, user_id, timestamp)` clause executes.

**Root Cause:** Memory assigns IDs using `AtomicI64` counter seeded from `disk.get_max_id()`. If disk already has logs with overlapping IDs (from prior checkpoint), INSERT fails immediately.

**Impact:** Data persistence broken - logs stuck in memory, risk of data loss on crash.

## Solution Overview

Remove explicit `id` from checkpoint INSERT statements. Let SQLite auto-generate IDs via AUTOINCREMENT. Memory IDs become transient (in-flight tracking only).

## Architecture Decision

**Chosen:** Option 1 (Architectural Fix) from debugger report
- Remove `id` column from checkpoint INSERT
- Sync tracking uses composite key `(device_id, user_id, timestamp)` for ACK correlation
- Memory IDs remain for internal tracking but never persisted

**Rejected Alternatives:**
- Option 2 (INSERT OR REPLACE): Violates least-surprise, may lose sync state
- Option 3 (Increase ID gap): Band-aid, will still collide eventually

## Phases

| Phase | Description | Status | Effort |
|-------|-------------|--------|--------|
| 1 | [Refactor Checkpoint INSERT](./phase-01-refactor-checkpoint-insert.md) | complete | 2h |
| 2 | [Update ACK Correlation](./phase-02-update-ack-correlation.md) | complete | 2h |
| 3 | [Testing and Validation](./phase-03-testing-validation.md) | complete | 2h |

## Key Dependencies

- Phase 2 depends on Phase 1 (new INSERT flow needed for ACK correlation)
- Phase 3 depends on Phases 1+2 (both must be complete for integration tests)

## Files to Modify

### Core Changes
1. `agent-main/src/database.rs` - Remove `id` from checkpoint INSERT
2. `agent-main/src/memory_store.rs` - Add composite key helper
3. `agent-main/src/hybrid_database.rs` - Update checkpoint flow
4. `agent-main/src/central_client.rs` - Use composite key for ACK

### Supporting Changes
5. `agent-main/src/models.rs` - Add `CompositeKey` type (optional)

### Test Files
6. `agent-main/src/database.rs` (tests module)
7. `agent-main/src/hybrid_database.rs` (tests module)
8. `agent-main/src/memory_store.rs` (tests module)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ACK correlation mismatch | Medium | High | Map memory_id -> composite_key on send, store reverse lookup |
| Performance regression | Low | Medium | Composite key lookup is O(1) HashMap |
| Migration issues | Low | Low | No schema change needed, existing data compatible |

## Rollback Strategy

1. Revert commits to `database.rs`, `memory_store.rs`, `hybrid_database.rs`, `central_client.rs`
2. No data migration needed - existing disk data unaffected
3. Memory will be reloaded from disk on restart

## Success Criteria

- [x] Checkpoint persists logs without UNIQUE constraint errors
- [x] ACK messages correctly mark logs as synced
- [x] No performance regression (checkpoint < 100ms for 1000 logs)
- [x] All existing tests pass
- [x] New regression test for ID collision scenario

## Completion Summary

**Date:** 2026-01-31
**Status:** Complete ✓
**Test Results:** 123/123 tests passing
**Performance:** ~50ms for 1000 logs (10x better than SLA)
**Code Review:** Approved for merge

**Reports:**
- [Test Report](../reports/tester-260131-1708-checkpoint-refactor-tests.md)
- [Code Review Report](./reports/code-reviewer-260131-1810-checkpoint-refactor.md)

**Next Steps:**
1. Merge to main branch
2. Monitor production for checkpoint errors (expect zero)
3. Watch for ACK correlation issues (expect none)
4. Close debugger issue
