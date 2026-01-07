# Implementation Plan: Convert Agent to Pure CLI (Websocket-Only)

## Approach
Convert the agent from a FastAPI-hosted application to a pure background service. This removes the inbound HTTP server entirely, relying solely on the outbound Websocket connection to the Central Server for all commands and status reporting.

## Benefits
- **Resource Efficiency**: Significant reduction in RAM and CPU overhead.
- **Enhanced Security**: No open inbound ports on the device.
- **Simplicity**: No more port conflicts or firewall configuration issues.

## Steps

### 1. Dependency Cleanup
- Remove `fastapi` and `uvicorn` from `requirements.txt`.

### 2. Main Entry Point Refactoring
- Rewrite `agent/main.py` to be a pure CLI script.
- Use `asyncio.run()` to manage the lifecycle of the `AttendanceAgent` instance.
- Implement robust signal handling (SIGINT, SIGTERM) for graceful shutdown.

### 3. Test Suite Migration
- Refactor all tests that currently use `fastapi.testclient.TestClient`.
- Verify that API-related logic (like `/status` or `/attendance/raw`) is still tested via direct method calls on `AttendanceAgent` or its repository.

### 4. Validation
- Run the full 55-test suite.
- Monitor memory usage compared to the HTTP-enabled version.

## Success Criteria
- Agent starts without opening any listening ports.
- All commands (ping, sync-time, etc.) work via Central Server Websocket.
- Full test suite passes.
