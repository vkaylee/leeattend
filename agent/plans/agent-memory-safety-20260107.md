# Implementation Plan: Agent Memory Safety & Sync Optimization

## Root Cause Analysis
1. **Symptom:** RAM usage increases over time during network instability.
2. **Technical Cause:** `CentralClient.pending_acks` holds `asyncio.Future` objects that are never removed if a timeout occurs in `SyncWorker`.
3. **Secondary Issue:** `SyncWorker` continues to pile messages into `send_queue` even if the server is not responding, leading to memory pressure and potential redundant data processing.

## Fix Plan

### 1. CentralClient Enhancements (`agent/central_client.py`)
- Modify `push_attendance_sync` to return both the `Future` and the `msg_id`.
- Add `discard_pending_ack(msg_id)` method to safely remove stale futures.
- Implement a maximum size for `pending_acks` as a safety guard.

### 2. SyncWorker Robustness (`agent/sync_worker.py`)
- Update `sync_batch` to handle the new return type from `push_attendance_sync`.
- Use a `try...finally` block to ensure `discard_pending_ack` is called after timeout or error.
- Add checks to skip sync if the `send_queue` is backlogged.

### 3. Verification
- Create `agent/tests/test_memory_safety.py` to specifically test:
    - Future cleanup on timeout.
    - Future cleanup on success.
    - Queue pressure handling.

## Rollback
- Revert changes to `central_client.py` and `sync_worker.py`.

## Security
- No changes to authentication or data encryption logic.
