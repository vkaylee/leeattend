# Planner Report: Checkpoint ID Refactor

**Date:** 2026-01-31 17:00
**Agent:** planner
**Plan:** `/home/elt1541/leeattend/plans/260131-1700-checkpoint-id-refactor/`

---

## Summary

Created implementation plan to resolve checkpoint UNIQUE constraint failures (SQLite error 1555) by removing explicit `id` from INSERT statements and refactoring ACK correlation to use composite keys.

## Root Cause (from debugger report)

Memory assigns IDs via `AtomicI64` seeded from `disk.get_max_id()`. If disk has overlapping IDs from prior checkpoint, INSERT fails immediately on PRIMARY KEY before `ON CONFLICT(device_id, user_id, timestamp)` executes.

## Solution

1. **Remove `id` from checkpoint INSERT** - Let SQLite AUTOINCREMENT handle IDs
2. **Decouple memory ID from disk ID** - Memory IDs for internal tracking only
3. **Use composite key for ACK** - `device_id:user_id:timestamp_unix` as msg_id

## Plan Structure

```
plans/260131-1700-checkpoint-id-refactor/
├── plan.md                                    # Overview (P1, 6h total)
├── phase-01-refactor-checkpoint-insert.md     # 2h - Remove id from INSERT
├── phase-02-update-ack-correlation.md         # 2h - Composite key for ACK
└── phase-03-testing-validation.md             # 2h - Regression tests
```

## Files to Modify

| File | Changes |
|------|---------|
| `agent-main/src/database.rs` | Remove `id` column from INSERT, update return type |
| `agent-main/src/memory_store.rs` | Add composite_key(), composite_index, lookup methods |
| `agent-main/src/hybrid_database.rs` | Update checkpoint caller, add wrapper methods |
| `agent-main/src/central_client.rs` | Use composite key for msg_id and ACK handling |

## Key Code Changes

### Phase 1: database.rs:566-586

```rust
// BEFORE
INSERT INTO zkt_raw_attendance
    (id, device_id, user_id, timestamp, ...)
VALUES (?, ?, ?, ?, ...)

// AFTER
INSERT INTO zkt_raw_attendance
    (device_id, user_id, timestamp, ...)
VALUES (?, ?, ?, ...)
```

### Phase 2: central_client.rs:136

```rust
// BEFORE
msg_id: log.id.to_string()

// AFTER
msg_id: log.composite_key()  // "DEV1:U001:1706767200"
```

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ACK mismatch | Medium | High | Composite key == UNIQUE index, guaranteed unique |
| Performance | Low | Medium | HashMap O(1) lookup, negligible overhead |
| Migration | N/A | N/A | No schema change, existing data compatible |

## Effort Estimate

- Phase 1: 2h (core fix)
- Phase 2: 2h (ACK refactor)
- Phase 3: 2h (testing)
- **Total: 6h**

## Next Steps

1. Assign to backend-engineer for Phase 1 implementation
2. Run `./run-tests.sh --api checkpoint` after each phase
3. Monitor production after merge

---

## Unresolved Questions

1. Should we add metric for checkpoint success/failure rate?
2. Consider UUID for log IDs long-term? (trade-off: larger index, slower inserts)
