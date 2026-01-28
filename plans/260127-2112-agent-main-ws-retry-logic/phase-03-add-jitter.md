# Phase 3: Add Jitter to Backoff

**Priority:** Medium | **Status:** Pending

---

## Context

- File: `/home/elt1541/leeattend/agent-main/src/central_client.rs`
- Current: Fixed exponential delays (1s, 2s, 4s, ...)
- Problem: If multiple agents reconnect, they'll hit server simultaneously

---

## Overview

Add random jitter (±25%) to retry delays to prevent thundering herd problem when multiple agents try to reconnect after a server outage.

---

## Key Insights

- Jitter range: delay ± 25% (e.g., 8s becomes 6s-10s)
- Use `rand` crate for random number generation
- Apply jitter only when delay > 0

---

## Requirements

### Functional
- Add ±25% jitter to all retry delays
- Minimum delay after jitter: 500ms (prevent immediate retries)
- Maximum delay unchanged: 60s

### Non-Functional
- rand crate is lightweight
- Deterministic in tests (can seed)

---

## Implementation Steps

1. Add `rand` to Cargo.toml: `rand = "0.8"`
2. Import `rand::Rng`
3. Modify `get_delay()` to apply jitter
4. Ensure minimum 500ms floor

---

## Code Changes

**File:** `agent-main/Cargo.toml`
```toml
[dependencies]
rand = "0.8"
```

**File:** `agent-main/src/central_client.rs`

**Before:**
```rust
fn get_delay(&self) -> Duration {
    if self.retry_count == 0 { return Duration::from_secs(0); }
    let base = 1u64;
    let delay = std::cmp::min(base * 2u64.pow(std::cmp::min(self.retry_count - 1, 31)), 60);
    Duration::from_secs(delay)
}
```

**After:**
```rust
use rand::Rng;

fn get_delay(&self) -> Duration {
    if self.retry_count == 0 {
        return Duration::from_secs(0);
    }

    let base = 1u64;
    let base_delay = std::cmp::min(base * 2u64.pow(std::cmp::min(self.retry_count - 1, 31)), 60);

    // Apply ±25% jitter
    let mut rng = rand::thread_rng();
    let jitter_factor = rng.gen_range(0.75..=1.25);
    let jittered = (base_delay as f64 * jitter_factor) as u64;

    // Ensure minimum 500ms and maximum 60s
    let final_delay = std::cmp::max(1, std::cmp::min(jittered, 60));
    Duration::from_secs(final_delay)
}
```

---

## Todo List

- [ ] Add rand = "0.8" to Cargo.toml
- [ ] Add rand::Rng import
- [ ] Modify get_delay() with jitter calculation
- [ ] Add minimum/maximum bounds

---

## Success Criteria

- Delays vary by ±25% on each retry
- Multiple agents don't all retry at exact same time
- Bounds respected: min 500ms, max 60s

---

## Risk Assessment

- **Low risk:** Jitter is additive improvement
- **Note:** rand crate adds ~50KB to binary (acceptable)

---

## Security Considerations

- None - using rand for timing only, not crypto
