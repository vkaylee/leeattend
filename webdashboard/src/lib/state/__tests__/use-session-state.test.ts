import { renderHook, act } from "@testing-library/react";
import { describe, expect, it, beforeEach, vi } from "vitest";

import { useSessionState, usePersistentState } from "../use-session-state";

describe("useSessionState", () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.restoreAllMocks();
  });

  it("returns the initial value when sessionStorage is empty", () => {
    const { result } = renderHook(() =>
      useSessionState("test:count", 42),
    );
    expect(result.current[0]).toBe(42);
  });

  it("reads an existing value from sessionStorage", () => {
    sessionStorage.setItem("test:count", JSON.stringify(99));
    const { result } = renderHook(() =>
      useSessionState("test:count", 42),
    );
    expect(result.current[0]).toBe(99);
  });

  it("updates state and persists when setting a direct value", () => {
    const { result } = renderHook(() =>
      useSessionState("test:name", "Alice"),
    );

    act(() => {
      result.current[1]("Bob");
    });

    expect(result.current[0]).toBe("Bob");
    expect(JSON.parse(sessionStorage.getItem("test:name")!)).toBe("Bob");
  });

  it("updates state using a function updater", () => {
    const { result } = renderHook(() =>
      useSessionState("test:count", 10),
    );

    act(() => {
      result.current[1]((prev) => prev + 5);
    });

    expect(result.current[0]).toBe(15);
    expect(JSON.parse(sessionStorage.getItem("test:count")!)).toBe(15);
  });

  it("falls back on corrupt JSON", () => {
    sessionStorage.setItem("test:corrupt", "{not-valid");
    const logSpy = vi.fn();
    vi.stubGlobal("console", { ...console, error: logSpy });

    const { result } = renderHook(() =>
      useSessionState("test:corrupt", "safe"),
    );

    expect(result.current[0]).toBe("safe");
  });

  it("handles objects correctly", () => {
    const initial = { x: 1, y: 2 };
    const { result } = renderHook(() =>
      useSessionState("test:point", initial),
    );

    act(() => {
      result.current[1]({ x: 10, y: 20 });
    });

    expect(result.current[0]).toEqual({ x: 10, y: 20 });
    const stored = JSON.parse(sessionStorage.getItem("test:point")!);
    expect(stored).toEqual({ x: 10, y: 20 });
  });
});

describe("usePersistentState", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
  });

  it("returns the initial value when localStorage is empty", () => {
    const { result } = renderHook(() =>
      usePersistentState("theme", "light"),
    );
    expect(result.current[0]).toBe("light");
  });

  it("reads an existing value from localStorage", () => {
    localStorage.setItem("theme", JSON.stringify("dark"));
    const { result } = renderHook(() =>
      usePersistentState("theme", "light"),
    );
    expect(result.current[0]).toBe("dark");
  });

  it("persists updates to localStorage", () => {
    const { result } = renderHook(() =>
      usePersistentState("lang", "en"),
    );

    act(() => {
      result.current[1]("vi");
    });

    expect(result.current[0]).toBe("vi");
    expect(JSON.parse(localStorage.getItem("lang")!)).toBe("vi");
  });
});
