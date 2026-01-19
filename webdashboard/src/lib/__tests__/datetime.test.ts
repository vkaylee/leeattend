import { describe, it, expect, vi } from "vitest";
import { toUTCISO } from "../datetime";

describe("toUTCISO utility", () => {
  it("converts Vietnam time (UTC+7) correctly", () => {
    // 08:00 AM VN = 01:00 AM UTC
    const result = toUTCISO("2026-01-16", "08:00", "Asia/Ho_Chi_Minh");
    expect(result).toBe("2026-01-16T01:00:00.000Z");
  });

  it("converts Bangkok time (UTC+7) correctly", () => {
    // 17:00 PM BKK = 10:00 AM UTC
    const result = toUTCISO("2026-01-16", "17:00", "Asia/Bangkok");
    expect(result).toBe("2026-01-16T10:00:00.000Z");
  });

  it("converts New York time (UTC-5 in winter) correctly", () => {
    // 08:00 AM NY (Winter) = 13:00 PM UTC
    const result = toUTCISO("2026-01-16", "08:00", "America/New_York");
    expect(result).toBe("2026-01-16T13:00:00.000Z");
  });

  it("converts New York time (UTC-4 in summer/DST) correctly", () => {
    // 08:00 AM NY (Summer) = 12:00 PM UTC
    const result = toUTCISO("2026-07-16", "08:00", "America/New_York");
    expect(result).toBe("2026-07-16T12:00:00.000Z");
  });

  it("handles UTC timezone correctly", () => {
    const result = toUTCISO("2026-01-16", "08:00", "UTC");
    expect(result).toBe("2026-01-16T08:00:00.000Z");
  });

  it("falls back to Z-suffix on invalid timezone", () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    
    const result = toUTCISO("2026-01-16", "08:00", "Invalid/Zone");
    
    // Should fallback to adding Z
    expect(result).toBe("2026-01-16T08:00:00Z");
    expect(consoleSpy).toHaveBeenCalled();
    
    consoleSpy.mockRestore();
  });
});
