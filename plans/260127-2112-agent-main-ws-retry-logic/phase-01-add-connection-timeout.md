# Phase 1: Add Connection Timeout

**Priority:** High | **Status:** Pending

---

## Context

- File: `/home/elt1541/leeattend/agent-main/src/central_client.rs`
- Current: `connect_async(request).await` at line 215 has no timeout
- Problem: If server is unresponsive, agent hangs indefinitely

---

## Overview

Add a 30-second timeout to the WebSocket connection attempt to prevent indefinite hangs.

---

## Key Insights

- `tokio::time::timeout` wraps any async operation
- Should timeout the entire `connect_async` call
- On timeout, treat as connection failure (trigger retry)

---

## Requirements

### Functional
- Connection attempt must timeout after 30 seconds
- Timeout error should be logged clearly
- Should trigger normal retry logic on timeout

### Non-Functional
- No performance impact on successful connections
- Timeout value should be configurable (constant for now)

---

## Implementation Steps

1. Add `tokio::time::timeout` import if not present
2. Wrap `connect_async(request).await` with `timeout(Duration::from_secs(30), ...)`
3. Handle `Elapsed` error case
4. Log timeout distinctly from other connection errors

---

## Code Changes

**File:** `agent-main/src/central_client.rs`

**Before (line 215):**
```rust
let (ws_stream, response) = connect_async(request).await?;
```

**After:**
```rust
use tokio::time::timeout;

const CONNECTION_TIMEOUT_SECS: u64 = 30;

// In connect_central function:
let connect_future = connect_async(request);
let (ws_stream, response) = timeout(
    Duration::from_secs(CONNECTION_TIMEOUT_SECS),
    connect_future
).await
    .map_err(|_| "Connection timeout after 30 seconds")?
    .map_err(|e| format!("WebSocket error: {}", e))?;
```

---

## Todo List

- [ ] Add CONNECTION_TIMEOUT_SECS constant
- [ ] Wrap connect_async with timeout
- [ ] Update error handling in connect_central
- [ ] Update error type to handle both timeout and ws errors

---

## Success Criteria

- Connection times out after 30 seconds if server unresponsive
- Timeout triggers normal retry logic
- Clear log message: "Connection timeout after 30 seconds"

---

## Risk Assessment

- **Low risk:** tokio::time::timeout is well-tested
- **Mitigation:** Test with slow/unresponsive mock server

---

## Security Considerations

- None - this is connection resilience, not auth change
