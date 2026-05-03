import { renderHook, act } from "@testing-library/react";

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";

import { useWebSocket } from "../use-websocket";

describe("useWebSocket Hook", () => {
  let mockOnOpen: ((event_?: Event) => void) | null;
 
  let mockSend: ReturnType<typeof vi.fn>;
 
  let mockClose: ReturnType<typeof vi.fn>;

  class MockWebSocket {
    static OPEN = 1;
    readyState = 1; // WebSocket.OPEN
    constructor() { /* empty */ }
    send = vi.fn().mockImplementation((value: string) => {
      if (mockSend) (mockSend as (v: string) => void)(value);
    });
    close = vi.fn().mockImplementation(() => {
      if (mockClose) (mockClose as () => void)();
    });
    set onopen(function_: ((event_?: Event) => void) | null) { mockOnOpen = function_; }
    set onmessage(_: unknown) { /* empty */ }
    set onclose(_: unknown) { /* empty */ }
  }

  beforeEach(() => {
    mockOnOpen = null;
    mockSend = vi.fn();
    mockClose = vi.fn();
    vi.stubGlobal("WebSocket", MockWebSocket);
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("should start a 30s ping interval on open", () => {
    renderHook(() => useWebSocket());

    act(() => {
      if (mockOnOpen) mockOnOpen();
    });

    // The handshake message is sent immediately on open
    expect(mockSend).toHaveBeenCalledWith(
      JSON.stringify({ type: "handshake", client_type: "dashboard" })
    );

    mockSend.mockClear();

    // Advance time by 30 seconds
    act(() => {
      vi.advanceTimersByTime(30000);
    });

    expect(mockSend).toHaveBeenCalledWith(JSON.stringify({ type: "ping" }));

    // Advance by another 30 seconds
    act(() => {
      vi.advanceTimersByTime(30000);
    });

    expect(mockSend).toHaveBeenCalledTimes(2);
  });

  it("should clear ping interval on unmount", () => {
    const { unmount } = renderHook(() => useWebSocket());

    act(() => {
      if (mockOnOpen) mockOnOpen();
    });

    mockSend.mockClear();
    unmount();

    // Advance time by 30 seconds after unmount
    act(() => {
      vi.advanceTimersByTime(30000);
    });

    // Should not have sent any pings because interval was cleared
    expect(mockSend).not.toHaveBeenCalled();
    expect(mockClose).toHaveBeenCalled();
  });
});
