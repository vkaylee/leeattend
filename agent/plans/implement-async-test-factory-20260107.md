# Implementation Plan: Async Test Factory for Agent

## Approach
Implement a centralized `AsyncTestFactory` to provide a controlled environment for testing asynchronous operations. This solves state leakage between tests and prevents busy-loops when mocking `asyncio.sleep`.

## Steps

### 1. Infrastructure
- Add `pytest-timeout` to ensure no test hangs indefinitely.
- Create `agent/tests/test_utils.py` to house the factory logic.

### 2. Implementation: AsyncTestFactory
- **State Management:** Methods to backup and restore `main.services`, `main.log_queue`, and `main.agent_stats`.
- **Safe Loop Control:** A context manager that patches `asyncio.sleep` to raise a `LoopBreak` exception after a configurable number of iterations.
- **Queue Helper:** Utilities to wait for specific messages in `log_queue` with a local timeout.

### 3. Integration
- Define a `test_factory` fixture in `agent/tests/conftest.py`.
- Apply `@pytest.mark.timeout(5)` globally via `pytest.ini` or conftest.

### 4. Refactoring
- Refactor `test_auth_error.py` to use `factory.break_loop()`.
- Refactor `test_queue.py` to use `factory.clear_state()`.

## Success Criteria
- Full test suite passes in one go.
- No test hangs (enforced by timeout).
- Zero state leakage between tests.
