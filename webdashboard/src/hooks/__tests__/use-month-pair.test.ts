import { afterEach, describe, expect, it, vi } from "vitest";

import { getMonthPair } from "../use-month-pair";

describe("getMonthPair", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns current and previous month on a normal day", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-15T12:00:00"));

    const { curMonth, curYear, prevMonth, prevYear } = getMonthPair();

    expect(curMonth).toBe(7);
    expect(curYear).toBe(2026);
    expect(prevMonth).toBe(6);
    expect(prevYear).toBe(2026);
  });

  it("handles day-31 → 30-day month (July 31 → June)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-31T12:00:00"));

    const { curMonth, curYear, prevMonth, prevYear } = getMonthPair();

    expect(curMonth).toBe(7);
    expect(curYear).toBe(2026);
    expect(prevMonth).toBe(6);
    expect(prevYear).toBe(2026);
  });

  it("handles day-31 → February non-leap (March 31 → February)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-31T12:00:00"));

    const { prevMonth, prevYear } = getMonthPair();

    expect(prevMonth).toBe(2);
    expect(prevYear).toBe(2026);
  });

  it("handles day-31 → February leap year (March 31, 2024 → February)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2024-03-31T12:00:00"));

    const { prevMonth, prevYear } = getMonthPair();

    expect(prevMonth).toBe(2);
    expect(prevYear).toBe(2024);
  });

  it("handles year boundary (January 31 → December of previous year)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-01-31T12:00:00"));

    const { prevMonth, prevYear } = getMonthPair();

    expect(prevMonth).toBe(12);
    expect(prevYear).toBe(2025);
  });

  it("handles October 31 → September (30-day month)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-31T12:00:00"));

    const { prevMonth, prevYear } = getMonthPair();

    expect(prevMonth).toBe(9);
    expect(prevYear).toBe(2026);
  });

  it("accepts an explicit date argument", () => {
    const date = new Date("2026-08-31T12:00:00");
    const { curMonth, curYear, prevMonth, prevYear } = getMonthPair(date);

    expect(curMonth).toBe(8);
    expect(curYear).toBe(2026);
    expect(prevMonth).toBe(7);
    expect(prevYear).toBe(2026);
  });
});
