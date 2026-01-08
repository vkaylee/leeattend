# Implementation Plan: Agent Authorization Error Notification

### ## Approach
- Update `CentralClient` to support an error callback.
- Connect this callback in `main.py` to display authorization errors prominently in the main runner's console output.

### ## Steps
1. Add `on_error_received` to `CentralClient` in `agent/central_client.py`.
2. Implement and register error handler in `agent/main.py`.
3. Verify output formatting.
