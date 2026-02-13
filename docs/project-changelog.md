# Project Changelog

Detailed record of significant changes, features, and fixes in LeeAttend.

---

## 2026-02-13

### Bug Fixes

#### WebSocket ACK Starvation Fix (agent-main)
**Severity:** Critical
**Component:** Edge Agent - WebSocket Sync
**Impact:** Prevented logs from syncing when outgoing queue was busy

**Problem:**
- WS Connection Manager used single `tokio::select!` loop for both reading ACKs and writing outgoing messages
- When sync loop queued many outgoing messages, select loop starved read branch
- ACKs never received → logs stuck as unsynced indefinitely
- Affected high-volume installations with 500+ logs per sync cycle

**Solution:**
- Split into independent read/write tokio tasks
- Read task: Dedicated for processing incoming ACKs and server messages
- Write task: Dedicated for sending attendance logs and heartbeats
- Graceful shutdown via `oneshot` channel preserves `mpsc::Receiver` across reconnections
- Both tasks run concurrently, eliminating starvation

**Files Changed:**
- `agent-main/src/central_client/connection.rs` (lines 238-299)
  - Replaced single select loop with split read/write tasks
  - Added shutdown signal for graceful rx preservation
  - Write task returns rx on shutdown for reconnect reuse

**Tests Added:**
- `agent-main/tests/ws_split_readwrite_test.rs` (4 tests)
  - `test_concurrent_ack_processing_during_bulk_send` - Verifies ACKs processed during bulk send
  - `test_reconnect_after_server_close` - Validates rx preservation across reconnect
  - `test_connection_failure_preserves_receiver` - Ensures rx not lost on connection failure
  - `test_no_ack_starvation_under_load` - Load test with 50 logs and 2ms server delay

**Verification:**
- All 4 new tests pass consistently
- Total agent test suite: 182 tests (was 178)
- No regressions in existing WebSocket connection tests

**Related Documentation:**
- Updated `docs/system-architecture.md` section 2.1 with split read/write architecture

---

## Template for Future Entries

### [Feature|Bug Fix|Enhancement|Security]

#### [Short Title]
**Severity:** [Critical|High|Medium|Low]
**Component:** [Component Name]
**Impact:** [User/System Impact Description]

**Problem:** (for bug fixes)
- Detailed description

**Solution:**
- Implementation approach

**Files Changed:**
- File paths with brief description

**Tests Added:** (if applicable)
- Test file and descriptions

**Breaking Changes:** (if any)
- Migration steps

**Related Issues:** (if tracked)
- Issue numbers or links

---
