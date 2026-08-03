import { renderHook, act } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

const mockGetBackoffDelay = vi.fn();

vi.mock("../use-report-stream-handlers", () => ({
  getBackoffDelay: (...args: unknown[]) => mockGetBackoffDelay(...args),
}));

import { useReconnectionBackoff } from "../use-reconnection-backoff";

describe("useReconnectionBackoff", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    mockGetBackoffDelay.mockReturnValue(2000);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("computes the delay from the current retry count, then increments it", () => {
    const retryCountRef = { current: 1 };
    const shouldReconnectRef = { current: true };
    const onReconnect = vi.fn();

    const { result } = renderHook(() =>
      useReconnectionBackoff({ retryCountRef, shouldReconnectRef, onReconnect }),
    );

    act(() => {
      result.current.scheduleReconnect();
    });

    expect(mockGetBackoffDelay).toHaveBeenCalledWith(1);
    expect(retryCountRef.current).toBe(2);
  });

  it("calls onReconnect after the backoff delay when reconnection is enabled", () => {
    const retryCountRef = { current: 0 };
    const shouldReconnectRef = { current: true };
    const onReconnect = vi.fn();

    const { result } = renderHook(() =>
      useReconnectionBackoff({ retryCountRef, shouldReconnectRef, onReconnect }),
    );

    act(() => {
      result.current.scheduleReconnect();
    });
    expect(onReconnect).not.toHaveBeenCalled();

    act(() => {
      vi.advanceTimersByTime(2000);
    });

    expect(onReconnect).toHaveBeenCalledTimes(1);
  });

  it("does not call onReconnect when reconnection is disabled by the time the timer fires", () => {
    const retryCountRef = { current: 0 };
    const shouldReconnectRef = { current: false };
    const onReconnect = vi.fn();

    const { result } = renderHook(() =>
      useReconnectionBackoff({ retryCountRef, shouldReconnectRef, onReconnect }),
    );

    act(() => {
      result.current.scheduleReconnect();
    });
    act(() => {
      vi.advanceTimersByTime(2000);
    });

    expect(onReconnect).not.toHaveBeenCalled();
  });

  it("reset returns the retry count to zero", () => {
    const retryCountRef = { current: 5 };
    const shouldReconnectRef = { current: true };
    const onReconnect = vi.fn();

    const { result } = renderHook(() =>
      useReconnectionBackoff({ retryCountRef, shouldReconnectRef, onReconnect }),
    );

    act(() => {
      result.current.reset();
    });

    expect(retryCountRef.current).toBe(0);
  });

  it("clears the pending timer on unmount so onReconnect never fires", () => {
    const retryCountRef = { current: 0 };
    const shouldReconnectRef = { current: true };
    const onReconnect = vi.fn();

    const { result, unmount } = renderHook(() =>
      useReconnectionBackoff({ retryCountRef, shouldReconnectRef, onReconnect }),
    );

    act(() => {
      result.current.scheduleReconnect();
    });

    unmount();

    act(() => {
      vi.advanceTimersByTime(5000);
    });

    expect(onReconnect).not.toHaveBeenCalled();
  });
});
