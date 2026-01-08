# Implementation Plan: Refactor Agent to SRP & Factory Pattern

## Approach
Move from a global-variable-heavy architecture to an instance-based container model. This enforces the Single Responsibility Principle (SRP) and ensures perfect isolation for testing.

## New Modules Structure
1. **`agent/repository.py`**: Handles all SQLAlchemy operations.
2. **`agent/device_manager.py`**: Manages ZK device connections and individual monitoring tasks.
3. **`agent/app_container.py`**: A mediator class (`AttendanceAgent`) that coordinates all components.
4. **`agent/main.py`**: Re-implemented as an App Factory for FastAPI.

## Steps

### 1. Extraction
- Move DB logic to `AttendanceRepository`.
- Move Device lifecycle logic to `DeviceManager`.
- Encapsulate `agent_stats` into the container.

### 2. Encapsulation
- Create `AttendanceAgent` class.
- Inject dependencies (Repo, Manager, Client) into the constructor.
- Implement unified `start()` and `stop()` methods.

### 3. API Integration
- Refactor FastAPI endpoints to use the `AttendanceAgent` instance.
- Use `app.state.agent` or DI to access the agent.

### 4. Testing
- Simplify `conftest.py`: each test gets its own `AttendanceAgent` instance.
- Remove `AsyncTestFactory` hacks as isolation is now native to the architecture.

## Success Criteria
- `agent/main.py` is focused only on framework setup.
- Zero global variables in `agent/` (excluding constants and the app factory).
- All existing 62 tests pass with the new architecture.
