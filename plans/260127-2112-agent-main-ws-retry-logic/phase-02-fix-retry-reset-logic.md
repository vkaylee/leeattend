# Phase 2: Fix Retry Reset Logic

**Priority:** High | **Status:** Pending

---

## Context

- File: `/home/elt1541/leeattend/agent-main/src/central_client.rs`
- Current issue at line 142: `manager.record_stable()` resets retry on every message
- Problem: Flaky connections rapidly cycle through retry delays

---

## Overview

Change retry reset logic to only reset after a period of stable connection (60 seconds), not on every received message.

---

## Key Insights

- Current: `record_stable()` at line 142 sets `retry_count = 0` on every message
- Should: Only reset after sustained stability (e.g., 60s without disconnect)
- Track `last_stable_time` and reset only if connection lasted > threshold

---

## Requirements

### Functional
- Retry count only resets after 60 seconds of stable connection
- Connection establishment should NOT immediately reset retry count
- First message after reconnection starts the stability timer

### Non-Functional
- Minimal memory overhead (just a timestamp)
- No blocking operations

---

## Implementation Steps

1. Add `Instant` import from `std::time`
2. Add `stable_since: Option<Instant>` to RetryManager
3. Modify `record_connection_established()` to NOT reset count (just log)
4. Modify `record_stable()` to:
   - Set `stable_since` if None
   - Check if stable for > 60s, then reset retry_count
5. `record_connection_failed()` and `record_dropped_prematurely()` reset `stable_since` to None

---

## Code Changes

**File:** `agent-main/src/central_client.rs`

**Before (lines 219-235):**
```rust
struct RetryManager {
    retry_count: u32,
}

impl RetryManager {
    fn new() -> Self { Self { retry_count: 0 } }
    fn record_connection_established(&mut self) { self.retry_count = 0; }
    fn record_connection_failed(&mut self) { self.retry_count = std::cmp::min(self.retry_count + 1, 10); }
    fn record_dropped_prematurely(&mut self) { self.record_connection_failed(); }
    fn record_stable(&mut self) { self.retry_count = 0; }
}
```

**After:**
```rust
use std::time::Instant;

const STABILITY_THRESHOLD_SECS: u64 = 60;

struct RetryManager {
    retry_count: u32,
    stable_since: Option<Instant>,
}

impl RetryManager {
    fn new() -> Self {
        Self {
            retry_count: 0,
            stable_since: None,
        }
    }

    fn get_delay(&self) -> Duration {
        if self.retry_count == 0 { return Duration::from_secs(0); }
        let base = 1u64;
        let delay = std::cmp::min(base * 2u64.pow(std::cmp::min(self.retry_count - 1, 31)), 60);
        Duration::from_secs(delay)
    }

    fn record_connection_established(&mut self) {
        // Don't reset retry_count immediately - wait for stability
        self.stable_since = Some(Instant::now());
    }

    fn record_connection_failed(&mut self) {
        self.retry_count = std::cmp::min(self.retry_count + 1, 10);
        self.stable_since = None;
    }

    fn record_dropped_prematurely(&mut self) {
        self.record_connection_failed();
    }

    fn record_stable(&mut self) {
        if let Some(since) = self.stable_since {
            if since.elapsed() >= Duration::from_secs(STABILITY_THRESHOLD_SECS) {
                self.retry_count = 0;
                // Keep stable_since as-is to avoid repeated resets
            }
        } else {
            self.stable_since = Some(Instant::now());
        }
    }
}
```

---

## Todo List

- [ ] Add Instant import
- [ ] Add STABILITY_THRESHOLD_SECS constant
- [ ] Add stable_since field to RetryManager
- [ ] Update record_connection_established to not reset count
- [ ] Update record_stable to check elapsed time
- [ ] Update record_connection_failed to reset stable_since

---

## Success Criteria

- Retry count does NOT reset on connection establishment
- Retry count resets only after 60s of stable operation
- Flaky connections maintain proper backoff

---

## Risk Assessment

- **Low risk:** Logic change is isolated
- **Test:** Verify with short-lived connections that backoff increases

---

## Security Considerations

- None
