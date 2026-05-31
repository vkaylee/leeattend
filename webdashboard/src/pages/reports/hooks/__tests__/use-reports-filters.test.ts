import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { getMonthRange } from "../use-reports-filters";

describe("getMonthRange edge cases and leap years", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("handles standard 31st to 30th day transition (e.g., today is May 31, 2026)", () => {
    // May 31, 2026 (Month index 4)
    vi.setSystemTime(new Date("2026-05-31T12:00:00"));

    // Check last month (April 2026)
    const lastMonth = getMonthRange(-1);
    expect(lastMonth.start).toBe("2026-04-01");
    expect(lastMonth.end).toBe("2026-04-30");

    // Check this month (May 2026)
    const thisMonth = getMonthRange(0);
    expect(thisMonth.start).toBe("2026-05-01");
    expect(thisMonth.end).toBe("2026-05-31");
  });

  it("handles non-leap year February transition (e.g., today is March 31, 2026)", () => {
    // March 31, 2026 (Month index 2)
    vi.setSystemTime(new Date("2026-03-31T12:00:00"));

    // Check last month (February 2026 - should have 28 days)
    const lastMonth = getMonthRange(-1);
    expect(lastMonth.start).toBe("2026-02-01");
    expect(lastMonth.end).toBe("2026-02-28");
  });

  it("handles leap year February transition (e.g., today is March 31, 2024)", () => {
    // March 31, 2024 (Month index 2)
    vi.setSystemTime(new Date("2024-03-31T12:00:00"));

    // Check last month (February 2024 - should have 29 days)
    const lastMonth = getMonthRange(-1);
    expect(lastMonth.start).toBe("2024-02-01");
    expect(lastMonth.end).toBe("2024-02-29");
  });

  it("handles today is leap day itself (e.g., today is Feb 29, 2024)", () => {
    // February 29, 2024 (Month index 1)
    vi.setSystemTime(new Date("2024-02-29T12:00:00"));

    // Check this month (February 2024)
    const thisMonth = getMonthRange(0);
    expect(thisMonth.start).toBe("2024-02-01");
    expect(thisMonth.end).toBe("2024-02-29");

    // Check last month (January 2024)
    const lastMonth = getMonthRange(-1);
    expect(lastMonth.start).toBe("2024-01-01");
    expect(lastMonth.end).toBe("2024-01-31");
  });

  it("handles today is non-leap year Feb 28th (e.g., today is Feb 28, 2026)", () => {
    // February 28, 2026 (Month index 1)
    vi.setSystemTime(new Date("2026-02-28T12:00:00"));

    // Check this month (February 2026)
    const thisMonth = getMonthRange(0);
    expect(thisMonth.start).toBe("2026-02-01");
    expect(thisMonth.end).toBe("2026-02-28");

    // Check last month (January 2026)
    const lastMonth = getMonthRange(-1);
    expect(lastMonth.start).toBe("2026-01-01");
    expect(lastMonth.end).toBe("2026-01-31");
  });

  it("handles year boundaries (e.g., today is Jan 31, 2026)", () => {
    // January 31, 2026 (Month index 0)
    vi.setSystemTime(new Date("2026-01-31T12:00:00"));

    // Check last month (December 2025)
    const lastMonth = getMonthRange(-1);
    expect(lastMonth.start).toBe("2025-12-01");
    expect(lastMonth.end).toBe("2025-12-31");
  });
});
