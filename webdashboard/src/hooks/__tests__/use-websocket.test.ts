import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";

import { useWebSocket } from "../use-websocket";

describe("useWebSocket Hook", () => {
  let mockOnOpen: ((ev?: Event) => void) | null;
  let mockSend: any;
  let mockClose: any;

  class MockWebSocket {
    static OPEN = 1;
    readyState = 1; // WebSocket.OPEN
    constructor() {}
    send = vi.fn().mockImplementation((val) => {
      if (mockSend) mockSend(val);
    });
    close = vi.fn().mockImplementation(() => {
      if (mockClose) mockClose();
    });
    set onopen(fn: ((ev?: Event) => void) | null) { mockOnOpen = fn; }
    set onmessage(_: unknown) { }
    set onclose(_: unknown) { }
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
