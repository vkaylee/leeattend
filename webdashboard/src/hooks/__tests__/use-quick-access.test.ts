import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach } from "vitest";

import { useQuickAccess } from "@/hooks/use-quick-access";

const STORAGE_KEY = "leeattend-quick-access";

describe("useQuickAccess", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("returns empty top3 when no history", () => {
    const { result } = renderHook(() => useQuickAccess());
    expect(result.current.top3).toEqual([]);
    expect(result.current.all).toEqual([]);
  });

  it("reads existing quick access from localStorage", () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(["employees", "payroll", "shifts"]));
    const { result } = renderHook(() => useQuickAccess());
    expect(result.current.top3).toEqual(["employees", "payroll", "shifts"]);
    expect(result.current.all).toEqual(["employees", "payroll", "shifts"]);
  });

  it("recordAccess prepends item and deduplicates", () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(["employees", "payroll", "reports"]));
    const { result } = renderHook(() => useQuickAccess());

    act(() => { result.current.recordAccess("payroll"); });

    // payroll should move to front
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
    expect(stored[0]).toBe("payroll");
    expect(stored.length).toBe(3); // Deduplicated, same count
  });

  it("recordAccess adds new item to front", () => {
    const { result } = renderHook(() => useQuickAccess());

    act(() => { result.current.recordAccess("attendance-logs"); });

    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
    expect(stored[0]).toBe("attendance-logs");
    expect(stored.length).toBe(1);
  });

  it("trims to max 10 items", () => {
    const tenItems = Array.from({ length: 10 }, (_, i) => `app-${i}`);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(tenItems));
    const { result } = renderHook(() => useQuickAccess());

    act(() => { result.current.recordAccess("new-app"); });

    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
    expect(stored.length).toBe(10);
    expect(stored[0]).toBe("new-app");
  });

  it("top3 returns only first 3 even when more exist", () => {
    const fiveItems = ["a", "b", "c", "d", "e"];
    localStorage.setItem(STORAGE_KEY, JSON.stringify(fiveItems));
    const { result } = renderHook(() => useQuickAccess());
    expect(result.current.top3).toEqual(["a", "b", "c"]);
    expect(result.current.all).toEqual(["a", "b", "c", "d", "e"]);
  });

  it("handles corrupted localStorage gracefully", () => {
    localStorage.setItem(STORAGE_KEY, "not-json");
    const { result } = renderHook(() => useQuickAccess());
    expect(result.current.top3).toEqual([]);
  });

  it("handles non-array localStorage value", () => {
    localStorage.setItem(STORAGE_KEY, "42");
    const { result } = renderHook(() => useQuickAccess());
    expect(result.current.top3).toEqual([]);
  });
});
