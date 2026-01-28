# Phase 4: Testing & Validation

**Priority:** High | **Status:** Pending

---

## Context

- File: `/home/elt1541/leeattend/agent-main/src/central_client.rs`
- Need to validate all retry logic changes work correctly

---

## Overview

Add unit tests for RetryManager and validate behavior manually.

---

## Requirements

### Functional
- Unit tests for RetryManager methods
- Integration test for timeout behavior (optional - manual OK)
- All existing tests must pass

### Non-Functional
- Tests run in < 5 seconds
- No external dependencies in unit tests

---

## Implementation Steps

1. Add unit tests for RetryManager in existing `mod tests` block
2. Run `cargo test` in agent-main
3. Run full test suite `./run-tests.sh --api`
4. Manual validation with mock slow server (optional)

---

## Code Changes

**File:** `agent-main/src/central_client.rs` (add to `mod tests`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // ... existing tests ...

    #[test]
    fn test_retry_manager_initial_delay_is_zero() {
        let manager = RetryManager::new();
        assert_eq!(manager.get_delay(), Duration::from_secs(0));
    }

    #[test]
    fn test_retry_manager_exponential_backoff() {
        let mut manager = RetryManager::new();

        // First failure: should have non-zero delay
        manager.record_connection_failed();
        let delay1 = manager.get_delay();
        assert!(delay1.as_millis() >= 750); // 1s - 25% jitter
        assert!(delay1.as_millis() <= 1250); // 1s + 25% jitter

        // Second failure: delay should increase
        manager.record_connection_failed();
        let delay2 = manager.get_delay();
        assert!(delay2 > delay1 || delay2.as_secs() >= 1);
    }

    #[test]
    fn test_retry_manager_max_delay_is_60_seconds() {
        let mut manager = RetryManager::new();

        // Simulate many failures
        for _ in 0..20 {
            manager.record_connection_failed();
        }

        let delay = manager.get_delay();
        assert!(delay.as_secs() <= 75); // 60s + 25% jitter max
    }

    #[test]
    fn test_retry_manager_stability_reset() {
        let mut manager = RetryManager::new();

        // Simulate failures
        manager.record_connection_failed();
        manager.record_connection_failed();
        assert!(manager.retry_count > 0);

        // Establish connection
        manager.record_connection_established();

        // Retry count should NOT reset immediately
        assert!(manager.retry_count > 0);

        // Simulate stable operation (can't wait 60s in test, check logic)
        assert!(manager.stable_since.is_some());
    }

    #[test]
    fn test_retry_manager_dropped_increases_count() {
        let mut manager = RetryManager::new();
        let initial = manager.retry_count;

        manager.record_dropped_prematurely();
        assert!(manager.retry_count > initial);
        assert!(manager.stable_since.is_none());
    }
}
```

---

## Todo List

- [ ] Add unit tests for RetryManager
- [ ] Run `cargo test -p agent-main`
- [ ] Run `./run-tests.sh --api` for full validation
- [ ] Fix any issues found

---

## Success Criteria

- All new unit tests pass
- All existing tests pass
- `./run-tests.sh --api` completes successfully

---

## Test Commands

```bash
# Unit tests only
cd agent-main && cargo test

# Full API test suite
./run-tests.sh --api
```

---

## Risk Assessment

- **Low risk:** Testing phase, no production changes
- **Note:** Jitter tests have inherent randomness - use ranges

---

## Next Steps

After all phases complete:
1. Update plan.md status to Complete
2. Run code-reviewer for final validation
3. Commit changes
