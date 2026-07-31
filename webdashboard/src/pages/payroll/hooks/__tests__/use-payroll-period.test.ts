import { renderHook, act } from "@testing-library/react";

import { afterEach, describe, expect, it, vi } from "vitest";

import { usePayrollPeriod } from "../use-payroll-period";

describe("usePayrollPeriod", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns previous month on a normal day", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-15T12:00:00"));

    const { result } = renderHook(() => usePayrollPeriod());

    expect(result.current.calcMonth).toBe(6);
    expect(result.current.calcYear).toBe(2026);
  });

  it("returns correct previous month when today is the 31st (July 31 → June)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-31T12:00:00"));

    const { result } = renderHook(() => usePayrollPeriod());

    expect(result.current.calcMonth).toBe(6);
    expect(result.current.calcYear).toBe(2026);
  });

  it("handles March 31 → February (non-leap year, 28 days)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-31T12:00:00"));

    const { result } = renderHook(() => usePayrollPeriod());

    expect(result.current.calcMonth).toBe(2);
    expect(result.current.calcYear).toBe(2026);
  });

  it("handles March 31 → February (leap year, 29 days)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2024-03-31T12:00:00"));

    const { result } = renderHook(() => usePayrollPeriod());

    expect(result.current.calcMonth).toBe(2);
    expect(result.current.calcYear).toBe(2024);
  });

  it("handles January 31 → December of previous year", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-01-31T12:00:00"));

    const { result } = renderHook(() => usePayrollPeriod());

    expect(result.current.calcMonth).toBe(12);
    expect(result.current.calcYear).toBe(2025);
  });

  it("handles October 31 → September (30-day month)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-31T12:00:00"));

    const { result } = renderHook(() => usePayrollPeriod());

    expect(result.current.calcMonth).toBe(9);
    expect(result.current.calcYear).toBe(2026);
  });

  it("navigates forward by one month via changeMonth(1)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-15T12:00:00"));

    const { result } = renderHook(() => usePayrollPeriod());

    // Initial: June 2026
    expect(result.current.calcMonth).toBe(6);
    expect(result.current.calcYear).toBe(2026);

    act(() => { result.current.changeMonth(1); });

    // After +1: July 2026
    expect(result.current.calcMonth).toBe(7);
    expect(result.current.calcYear).toBe(2026);
  });

  it("wraps December → January of next year via changeMonth(1)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-01-15T12:00:00"));

    const { result } = renderHook(() => usePayrollPeriod());

    // Initial: December 2025
    expect(result.current.calcMonth).toBe(12);
    expect(result.current.calcYear).toBe(2025);

    act(() => { result.current.changeMonth(1); });

    // After +1: January 2026
    expect(result.current.calcMonth).toBe(1);
    expect(result.current.calcYear).toBe(2026);
  });

  it("wraps January → December of previous year via changeMonth(-1)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-02-15T12:00:00"));

    const { result } = renderHook(() => usePayrollPeriod());

    // Initial: January 2026
    expect(result.current.calcMonth).toBe(1);
    expect(result.current.calcYear).toBe(2026);

    act(() => { result.current.changeMonth(-1); });

    // After -1: December 2025
    expect(result.current.calcMonth).toBe(12);
    expect(result.current.calcYear).toBe(2025);
  });
});
