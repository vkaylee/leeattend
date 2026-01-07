# Implementation Plan: Filter Sync by Active Devices

### ## Approach
- **Problem**: SyncWorker currently syncs all unsynced logs regardless of whether the device is still active in the agent's configuration.
- **Solution**: Refactor `SyncWorker` to accept a callback that returns the current list of active `device_id`s. Use this list to filter the database query.

### ## Steps
1. Refactor `agent/sync_worker.py` to use `get_active_devices_callback`.
2. Update `agent/main.py` to provide `list(services.keys())`.
3. Update and add tests in `agent/tests/test_sync_worker.py`.
