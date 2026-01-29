---
title: "Signature Cache Deduplication"
description: "Optimize memory & disk I/O by using hash-based signature cache for log deduplication"
status: complete
priority: P1
effort: 6h
branch: main
tags: [performance, memory-optimization, deduplication]
created: 2026-01-29
completed: 2026-01-29
---

# Signature Cache Deduplication

## Summary

Chuyển đổi cơ chế kiểm tra trùng lặp từ "Check Disk" hoặc "Full Memory" sang "Signature Cache" (chỉ lưu Hash trên RAM) để tối ưu Memory & Disk I/O.

## Problem Statement

**Current state:**
- `InMemoryStore` lưu full log data (~200+ bytes/log) trong DashMap
- Khi sync xong → `cleanup_synced()` xóa log khỏi memory
- Nhưng nếu device gửi lại log cũ → không có gì ngăn duplicate được accept

**Root cause:**
- Không có persistent dedup index sau khi log synced
- Memory cleanup = mất dedup protection

## Solution: Signature Cache

```
Log arrives (Device1, UserA, 8:00)
  ↓
Calculate Hash: H = hash("Device1_UserA_8:00")
  ↓
Check signatures.contains(H)?
  ↓ YES → Reject (Duplicate)
  ↓ NO → Insert (Add H to signatures, add full log to queue)
  ↓
Sync success → Remove log from queue, KEEP H in signatures
```

**Key insight:** Hash signature (~8 bytes) vs full log (~200+ bytes) = 25x memory savings

## Phases

| Phase | Description | Status | Effort |
|-------|-------------|--------|--------|
| [Phase 1](./phase-01-add-signature-index.md) | Add DashSet<u64> signatures index | complete | 1.5h |
| [Phase 2](./phase-02-refactor-insert-dedup.md) | Refactor insert() với hash check | complete | 1h |
| [Phase 3](./phase-03-optimize-startup-load.md) | Optimize load_from_disk() | complete | 1.5h |
| [Phase 4](./phase-04-testing-benchmarks.md) | Unit tests + benchmarks | complete | 2h |

## Key Design Decisions

### Hash Function: FxHash (rustc-hash)
- **Why:** Fastest non-crypto hash, used by rustc
- **Alt considered:** SipHash (std), xxHash
- **Decision:** FxHash for speed, collision-resistant enough for dedup

### Signature Structure: DashSet<u64>
- **Why:** Lock-free concurrent access, same as DashMap
- **Alt considered:** parking_lot::RwLock<HashSet>
- **Decision:** DashSet for consistency with existing codebase

### Hash Input Format
```rust
format!("{}|{}|{}", device_id, user_id, timestamp.timestamp())
```
- Using Unix timestamp (i64) for consistent hashing
- Pipe separator prevents collision: "A|BC" vs "AB|C"

## Memory Impact Analysis

| Scenario | Current | With Signatures | Savings |
|----------|---------|-----------------|---------|
| 100K logs | ~20MB | ~0.8MB (hash only) | 96% |
| After cleanup | 0MB | 0.8MB (signatures remain) | N/A |
| Dedup protection | LOST | MAINTAINED | Critical |

**Worst case estimate:**
- 100K signatures × 8 bytes = 800KB
- DashSet overhead (~2x) = 1.6MB total
- Well within 43MB available

## Dependencies

- `dashmap` 6.x (already in Cargo.toml)
- `rustc-hash` (FxHasher) - NEW dependency

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Hash collision | Low - 1 in 2^64 | Log collision to monitoring |
| Memory growth | Medium | Cap signatures at 1M entries, LRU eviction |
| Startup slowdown | Low | Two-phase load (signatures first, unsynced second) |

## Success Criteria

1. No duplicate logs accepted after sync+cleanup
2. Startup time < 2s for 100K logs
3. Memory < 5MB for 100K signatures
4. All 161 existing tests pass
5. New unit tests for dedup logic

## Rollback Plan

1. Keep original `cleanup_synced()` logic commented
2. Feature flag: `ENABLE_SIGNATURE_CACHE=true`
3. If issues: set flag to false, redeploy
