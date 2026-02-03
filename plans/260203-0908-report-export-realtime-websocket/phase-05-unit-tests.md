# Phase 5: Unit Tests

**Status**: pending
**Priority**: P2
**Effort**: 1h

---

## Context

Write comprehensive unit tests for `useReportStream` hook and dialog state logic.

---

## Requirements

### Functional
- Test WebSocket connection lifecycle
- Test message handling (CHUNK, DONE, ERROR)
- Test reconnection logic with exponential backoff
- Test state transitions
- Test error scenarios

### Non-Functional
- 80%+ code coverage
- Fast execution (< 5s)
- No real WebSocket connections (mock)

---

## Related Files

**Create**:
- `webdashboard/src/hooks/__tests__/use-report-stream.test.ts`

**Reference**:
- `webdashboard/src/hooks/use-report-stream.ts`

---

## Test Structure

```typescript
// webdashboard/src/hooks/__tests__/use-report-stream.test.ts

import { renderHook, waitFor } from "@testing-library/react";
import { useReportStream } from "../use-report-stream";

// Mock WebSocket
class MockWebSocket {
  onopen: (() => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;

  constructor(public url: string) {
    setTimeout(() => this.onopen?.(), 0);
  }

  send(data: string) {}
  close() {
    setTimeout(() => this.onclose?.({ code: 1000, reason: "" } as CloseEvent), 0);
  }
}

global.WebSocket = MockWebSocket as any;

describe("useReportStream", () => {
  // Test cases below
});
```

---

## Implementation Steps

### 1. Test: Initial State

```typescript
test("should start in idle state with no job ID", () => {
  const { result } = renderHook(() =>
    useReportStream({ jobId: null })
  );

  expect(result.current.status).toBe("idle");
  expect(result.current.progress).toBe(0);
  expect(result.current.chunks).toEqual([]);
  expect(result.current.error).toBeNull();
});
```

### 2. Test: WebSocket Connection

```typescript
test("should connect when jobId provided", async () => {
  const { result } = renderHook(() =>
    useReportStream({ jobId: "test-job-123" })
  );

  expect(result.current.status).toBe("connecting");

  await waitFor(() => {
    expect(result.current.status).toBe("streaming");
  });
});
```

### 3. Test: CHUNK Message Handling

```typescript
test("should handle CHUNK messages and update progress", async () => {
  let mockWs: MockWebSocket;

  (global.WebSocket as any) = class extends MockWebSocket {
    constructor(url: string) {
      super(url);
      mockWs = this;
    }
  };

  const { result } = renderHook(() =>
    useReportStream({ jobId: "test-job-123" })
  );

  await waitFor(() => expect(result.current.status).toBe("streaming"));

  // Simulate CHUNK message
  const chunkMessage = {
    type: "CHUNK",
    batch_index: 0,
    total_batches: 4,
    data: [{ id: 1, name: "Employee 1" }],
  };

  mockWs.onmessage?.({
    data: JSON.stringify(chunkMessage),
  } as MessageEvent);

  await waitFor(() => {
    expect(result.current.progress).toBe(25); // 1/4 * 100
    expect(result.current.chunks).toHaveLength(1);
  });
});
```

### 4. Test: Multiple Chunks

```typescript
test("should accumulate multiple chunks", async () => {
  let mockWs: MockWebSocket;

  (global.WebSocket as any) = class extends MockWebSocket {
    constructor(url: string) {
      super(url);
      mockWs = this;
    }
  };

  const { result } = renderHook(() =>
    useReportStream({ jobId: "test-job-123" })
  );

  await waitFor(() => expect(result.current.status).toBe("streaming"));

  // Send multiple chunks
  for (let i = 0; i < 3; i++) {
    mockWs.onmessage?.({
      data: JSON.stringify({
        type: "CHUNK",
        batch_index: i,
        total_batches: 3,
        data: [{ batch: i }],
      }),
    } as MessageEvent);
  }

  await waitFor(() => {
    expect(result.current.chunks).toHaveLength(3);
    expect(result.current.progress).toBeCloseTo(100, 0);
  });
});
```

### 5. Test: DONE Message

```typescript
test("should complete on DONE message", async () => {
  let mockWs: MockWebSocket;
  const onComplete = jest.fn();

  (global.WebSocket as any) = class extends MockWebSocket {
    constructor(url: string) {
      super(url);
      mockWs = this;
    }
  };

  const { result } = renderHook(() =>
    useReportStream({
      jobId: "test-job-123",
      onComplete,
    })
  );

  await waitFor(() => expect(result.current.status).toBe("streaming"));

  // Send DONE message
  mockWs.onmessage?.({
    data: JSON.stringify({
      type: "DONE",
      total_records: 100,
    }),
  } as MessageEvent);

  await waitFor(() => {
    expect(result.current.status).toBe("completed");
    expect(result.current.progress).toBe(100);
    expect(onComplete).toHaveBeenCalledWith(expect.any(Array));
  });
});
```

### 6. Test: ERROR Message

```typescript
test("should handle ERROR message", async () => {
  let mockWs: MockWebSocket;
  const onError = jest.fn();

  (global.WebSocket as any) = class extends MockWebSocket {
    constructor(url: string) {
      super(url);
      mockWs = this;
    }
  };

  const { result } = renderHook(() =>
    useReportStream({
      jobId: "test-job-123",
      onError,
    })
  );

  await waitFor(() => expect(result.current.status).toBe("streaming"));

  // Send ERROR message
  mockWs.onmessage?.({
    data: JSON.stringify({
      type: "ERROR",
      message: "Database connection failed",
    }),
  } as MessageEvent);

  await waitFor(() => {
    expect(result.current.status).toBe("error");
    expect(result.current.error).toBe("Database connection failed");
    expect(onError).toHaveBeenCalledWith("Database connection failed");
  });
});
```

### 7. Test: Reconnection Logic

```typescript
test("should reconnect with exponential backoff", async () => {
  jest.useFakeTimers();
  let connectCount = 0;

  (global.WebSocket as any) = class extends MockWebSocket {
    constructor(url: string) {
      super(url);
      connectCount++;
      // Force close immediately to trigger reconnect
      setTimeout(() => this.onclose?.({ code: 1006, reason: "" } as CloseEvent), 0);
    }
  };

  renderHook(() =>
    useReportStream({ jobId: "test-job-123", maxRetries: 3 })
  );

  // Initial connection
  expect(connectCount).toBe(1);

  // First retry (1s)
  jest.advanceTimersByTime(1000);
  await waitFor(() => expect(connectCount).toBe(2));

  // Second retry (2s)
  jest.advanceTimersByTime(2000);
  await waitFor(() => expect(connectCount).toBe(3));

  // Third retry (4s)
  jest.advanceTimersByTime(4000);
  await waitFor(() => expect(connectCount).toBe(4));

  // Should stop after maxRetries
  jest.advanceTimersByTime(10000);
  expect(connectCount).toBe(4);

  jest.useRealTimers();
});
```

### 8. Test: Max Retries Exceeded

```typescript
test("should set error after max retries", async () => {
  jest.useFakeTimers();
  const onError = jest.fn();

  (global.WebSocket as any) = class extends MockWebSocket {
    constructor(url: string) {
      super(url);
      setTimeout(() => this.onclose?.({ code: 1006, reason: "" } as CloseEvent), 0);
    }
  };

  const { result } = renderHook(() =>
    useReportStream({
      jobId: "test-job-123",
      maxRetries: 2,
      onError,
    })
  );

  // Exhaust retries
  jest.advanceTimersByTime(1000); // Retry 1
  jest.advanceTimersByTime(2000); // Retry 2
  jest.advanceTimersByTime(4000); // Max exceeded

  await waitFor(() => {
    expect(result.current.status).toBe("error");
    expect(result.current.error).toBe("Connection lost. Max retries exceeded.");
    expect(onError).toHaveBeenCalledWith("Connection lost. Max retries exceeded.");
  });

  jest.useRealTimers();
});
```

### 9. Test: Manual Reconnect

```typescript
test("should allow manual reconnect", async () => {
  let mockWs: MockWebSocket;

  (global.WebSocket as any) = class extends MockWebSocket {
    constructor(url: string) {
      super(url);
      mockWs = this;
    }
  };

  const { result } = renderHook(() =>
    useReportStream({ jobId: "test-job-123" })
  );

  await waitFor(() => expect(result.current.status).toBe("streaming"));

  // Simulate error
  mockWs.onmessage?.({
    data: JSON.stringify({ type: "ERROR", message: "Test error" }),
  } as MessageEvent);

  await waitFor(() => expect(result.current.status).toBe("error"));

  // Manual reconnect
  result.current.reconnect();

  await waitFor(() => {
    expect(result.current.status).toBe("streaming");
    expect(result.current.error).toBeNull();
    expect(result.current.chunks).toEqual([]); // Reset
  });
});
```

### 10. Test: Cleanup on Unmount

```typescript
test("should cleanup WebSocket on unmount", async () => {
  let closeCalled = false;

  (global.WebSocket as any) = class extends MockWebSocket {
    close() {
      closeCalled = true;
      super.close();
    }
  };

  const { unmount } = renderHook(() =>
    useReportStream({ jobId: "test-job-123" })
  );

  await waitFor(() => expect(closeCalled).toBe(false));

  unmount();

  expect(closeCalled).toBe(true);
});
```

---

## Test Coverage Goals

| Component | Target | Focus Areas |
|-----------|--------|-------------|
| Connection lifecycle | 90% | open, close, error, reconnect |
| Message handling | 95% | CHUNK, DONE, ERROR parsing |
| State management | 90% | status, progress, chunks, error |
| Retry logic | 85% | backoff, max retries, manual reconnect |
| Edge cases | 80% | invalid JSON, malformed messages |

---

## Todo List

- [ ] Create test file
- [ ] Setup WebSocket mock
- [ ] Test: Initial state
- [ ] Test: Connection lifecycle
- [ ] Test: CHUNK message handling
- [ ] Test: Multiple chunks accumulation
- [ ] Test: DONE message
- [ ] Test: ERROR message
- [ ] Test: Reconnection with backoff
- [ ] Test: Max retries exceeded
- [ ] Test: Manual reconnect
- [ ] Test: Cleanup on unmount
- [ ] Run tests: `npm run test -- use-report-stream`
- [ ] Verify 80%+ coverage

---

## Success Criteria

✅ All tests pass
✅ 80%+ code coverage
✅ Tests run in < 5s
✅ No flaky tests
✅ Edge cases covered

---

## Running Tests

```bash
# Run specific test file
npm run test -- use-report-stream.test.ts

# Run with coverage
npm run test -- --coverage use-report-stream.test.ts

# Watch mode (development)
npm run test -- --watch use-report-stream.test.ts
```

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Flaky timer tests | Medium | Use fake timers consistently |
| Mock WebSocket incomplete | Low | Test against real behavior |
| Async race conditions | Medium | Proper `waitFor` usage |

---

## Security Considerations

- No real network connections in tests
- Mock data doesn't contain sensitive info

---

## Next Steps

→ Phase 6: Write E2E tests
