import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { useDebounce } from "../use-debounce";

describe("useDebounce", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("should return initial value immediately", () => {
    const { result } = renderHook(() => useDebounce("initial", 500));
    expect(result.current).toBe("initial");
  });

  it("should debounce value changes", () => {
    const { result, rerender } = renderHook(
      ({ value }) => useDebounce(value, 500),
      { initialProps: { value: "first" } }
    );

    expect(result.current).toBe("first");

    // Update prop
    rerender({ value: "second" });
    expect(result.current).toBe("first"); // Still first

    // Fast forward time
    act(() => {
      vi.advanceTimersByTime(250);
    });
    expect(result.current).toBe("first");

    act(() => {
      vi.advanceTimersByTime(250);
    });
    expect(result.current).toBe("second");
  });

  it("should reset timer on rapid changes", () => {
    const { result, rerender } = renderHook(
      ({ value }) => useDebounce(value, 500),
      { initialProps: { value: "a" } }
    );

    rerender({ value: "ab" });
    act(() => { vi.advanceTimersByTime(300); });
    expect(result.current).toBe("a");

    rerender({ value: "abc" });
    act(() => { vi.advanceTimersByTime(300); });
    expect(result.current).toBe("a"); // Timer should have reset

    act(() => { vi.advanceTimersByTime(200); });
    expect(result.current).toBe("abc");
  });
});
