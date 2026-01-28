# Plan: Agent-Main WebSocket Retry Logic Enhancement

**Date:** 2026-01-27 | **Status:** Complete | **Priority:** High

---

## Overview

Enhance the WebSocket retry logic in `agent-main` when connecting to `apiserver` to improve resilience and prevent indefinite hangs.

### Current State
- Basic exponential backoff exists (1s → 2s → 4s → ... → 60s max)
- **Issue 1:** No connection timeout - `connect_async()` can hang indefinitely
- **Issue 2:** `record_stable()` resets retry count on every message (causes rapid cycling)
- **Issue 3:** No jitter in backoff delays (thundering herd problem)

### Target State
- Connection timeout prevents indefinite hangs
- Proper reset logic (only on stable connection, not every message)
- Jitter prevents synchronized reconnection storms

---

## Phases

| Phase | Description | File | Status |
|-------|-------------|------|--------|
| [1](./phase-01-add-connection-timeout.md) | Add connection timeout | phase-01-add-connection-timeout.md | ✅ Complete |
| [2](./phase-02-fix-retry-reset-logic.md) | Fix retry reset logic | phase-02-fix-retry-reset-logic.md | ✅ Complete |
| [3](./phase-03-add-jitter.md) | Add jitter to backoff | phase-03-add-jitter.md | ✅ Complete |
| [4](./phase-04-testing-validation.md) | Testing & validation | phase-04-testing-validation.md | ✅ Complete |

---

## Related Files

| File | Purpose |
|------|---------|
| `agent-main/src/central_client.rs` | Main WS client - all changes here |
| `agent-main/Cargo.toml` | Add rand dependency (Phase 3) |

---

## Dependencies

- None (uses existing tokio features)
- New: `rand = "0.8"` for jitter (Phase 3)

---

## Estimated Changes

| Component | Lines Changed |
|-----------|--------------|
| central_client.rs | ~40 lines modified |
| Cargo.toml | 1 line added |
| Tests | ~50 lines added |

---

## Success Criteria

- [x] Connection attempts timeout after 30 seconds
- [x] Retry count only resets after 60s of stable connection
- [x] Jitter applied (±25%) to prevent thundering herd
- [x] All existing tests pass (19 tests)
- [x] New tests for RetryManager logic (7 tests added)

---

## Architecture Decision

**Keep It Simple:**
- All changes in single file (`central_client.rs`)
- No new modules or abstractions
- Reuse existing RetryManager struct

---

## Rollback Plan

If issues arise:
1. Revert `central_client.rs` to previous version
2. Remove `rand` from Cargo.toml
3. Redeploy agent

---

## Post-Implementation

1. Monitor agent logs for connection patterns
2. Verify backoff timing in staging
3. Document retry behavior in API reference
