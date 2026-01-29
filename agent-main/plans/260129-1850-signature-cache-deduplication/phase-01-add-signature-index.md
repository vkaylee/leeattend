---
phase: 1
title: "Add Signature Index"
status: complete
effort: 1.5h
completed: 2026-01-29
---

# Phase 1: Add DashSet<u64> Signatures Index

## Context

**Related files:**
- `/home/elt1541/leeattend/agent-main/src/memory_store.rs` (primary)
- `/home/elt1541/leeattend/agent-main/Cargo.toml` (add rustc-hash)

## Overview

Thêm `signatures: DashSet<u64>` vào `InMemoryStore` để lưu hash của mỗi log đã được insert.

## Key Insights

1. `DashSet` là wrapper của `DashMap<K, ()>` - same lock-free concurrent guarantees
2. `rustc-hash::FxHasher` nhanh hơn 2-3x so với `std::hash::DefaultHasher`
3. Hash input cần deterministic: sử dụng Unix timestamp thay vì string timestamp

## Requirements

### Functional
- F1: Add `signatures` field to `InMemoryStore`
- F2: Implement `compute_signature(device_id, user_id, timestamp) -> u64`
- F3: Signature must be deterministic across restarts

### Non-functional
- NF1: No memory leak - signatures must have upper bound
- NF2: Thread-safe - concurrent inserts must not race

## Architecture

```rust
pub struct InMemoryStore {
    logs: DashMap<i64, MemoryLog>,
    signatures: DashSet<u64>,           // NEW
    unsynced_index: Arc<RwLock<BTreeSet<i64>>>,
    // ... rest unchanged
}
```

### Hash Function Design

```rust
use std::hash::{Hash, Hasher};
use rustc_hash::FxHasher;

impl InMemoryStore {
    /// Compute 64-bit signature for deduplication
    fn compute_signature(device_id: &str, user_id: &str, timestamp: NaiveDateTime) -> u64 {
        let mut hasher = FxHasher::default();
        device_id.hash(&mut hasher);
        user_id.hash(&mut hasher);
        timestamp.and_utc().timestamp().hash(&mut hasher);
        hasher.finish()
    }
}
```

**Why this design:**
- `FxHasher::default()` is faster than `DefaultHasher::new()`
- `timestamp.and_utc().timestamp()` converts to i64 Unix seconds - deterministic
- Individual `hash()` calls are faster than string concatenation

## Related Code Files

### Files to Modify
| File | Changes |
|------|---------|
| `src/memory_store.rs` | Add signatures field, compute_signature() |
| `Cargo.toml` | Add `rustc-hash = "2"` dependency |

### Files to Create
- None

## Implementation Steps

### Step 1: Add Dependency (5 min)
```toml
# Cargo.toml
[dependencies]
rustc-hash = "2"
```

### Step 2: Add DashSet Import (2 min)
```rust
// src/memory_store.rs
use dashmap::DashSet;
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};
```

### Step 3: Add Field to Struct (5 min)
```rust
pub struct InMemoryStore {
    logs: DashMap<i64, MemoryLog>,
    signatures: DashSet<u64>,  // NEW: Hash signatures for dedup
    unsynced_index: Arc<RwLock<BTreeSet<i64>>>,
    // ...
}
```

### Step 4: Initialize in new() (5 min)
```rust
impl InMemoryStore {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            logs: DashMap::with_capacity(max_capacity / 2),
            signatures: DashSet::with_capacity(max_capacity),  // NEW
            unsynced_index: Arc::new(RwLock::new(BTreeSet::new())),
            // ...
        }
    }
}
```

### Step 5: Add compute_signature() (10 min)
```rust
impl InMemoryStore {
    /// Compute 64-bit signature for deduplication.
    /// Signature = hash(device_id, user_id, timestamp_unix)
    fn compute_signature(device_id: &str, user_id: &str, timestamp: NaiveDateTime) -> u64 {
        let mut hasher = FxHasher::default();
        device_id.hash(&mut hasher);
        user_id.hash(&mut hasher);
        // Use Unix timestamp for deterministic hashing
        timestamp.and_utc().timestamp().hash(&mut hasher);
        hasher.finish()
    }

    /// Check if a signature already exists (public for testing)
    pub fn has_signature(&self, device_id: &str, user_id: &str, timestamp: NaiveDateTime) -> bool {
        let sig = Self::compute_signature(device_id, user_id, timestamp);
        self.signatures.contains(&sig)
    }

    /// Get signature count (for monitoring)
    pub fn count_signatures(&self) -> usize {
        self.signatures.len()
    }
}
```

### Step 6: Add Unit Tests (20 min)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_compute_signature_deterministic() {
        let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let sig1 = InMemoryStore::compute_signature("D1", "U1", ts);
        let sig2 = InMemoryStore::compute_signature("D1", "U1", ts);
        assert_eq!(sig1, sig2, "Same input should produce same signature");
    }

    #[test]
    fn test_compute_signature_different_inputs() {
        let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let sig1 = InMemoryStore::compute_signature("D1", "U1", ts);
        let sig2 = InMemoryStore::compute_signature("D1", "U2", ts);
        let sig3 = InMemoryStore::compute_signature("D2", "U1", ts);

        assert_ne!(sig1, sig2, "Different user should produce different signature");
        assert_ne!(sig1, sig3, "Different device should produce different signature");
    }

    #[test]
    fn test_signature_count() {
        let store = InMemoryStore::new(1000);
        assert_eq!(store.count_signatures(), 0);
    }
}
```

## Todo List

- [ ] Add `rustc-hash = "2"` to Cargo.toml
- [ ] Add `DashSet` and hash imports to memory_store.rs
- [ ] Add `signatures: DashSet<u64>` field to InMemoryStore
- [ ] Initialize signatures in `new()`
- [ ] Implement `compute_signature()`
- [ ] Add `has_signature()` public method
- [ ] Add `count_signatures()` for monitoring
- [ ] Write unit tests for signature computation
- [ ] Run `cargo test` - verify all tests pass
- [ ] Run `cargo build --release` - verify no warnings

## Success Criteria

1. `InMemoryStore` has `signatures: DashSet<u64>` field
2. `compute_signature()` produces deterministic hashes
3. All 161 existing tests still pass
4. New signature tests pass
5. No compiler warnings

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking existing tests | Low | Medium | Only adding new field, not changing behavior yet |
| Hash collision | Very Low | Low | 2^64 space is huge |

## Security Considerations

- Hash is NOT cryptographic - acceptable for dedup
- No sensitive data in hash output (just u64)

## Next Steps

After Phase 1 complete:
- Phase 2: Integrate signature check into `insert()`
