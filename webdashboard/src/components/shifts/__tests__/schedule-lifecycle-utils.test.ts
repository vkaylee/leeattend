import { describe, expect, it } from "vitest";

import { formatScheduleLocalTime } from "../schedule-lifecycle-utils";

describe("formatScheduleLocalTime", () => {
  it("converts evidence timestamps to replan local time", () => {
    expect(formatScheduleLocalTime("2026-11-16T08:00:00.000Z", "Asia/Ho_Chi_Minh")).toBe("15:00:00");
  });

  it("returns null for missing or malformed timestamps", () => {
    expect(formatScheduleLocalTime(undefined, "UTC")).toBeNull();
    expect(formatScheduleLocalTime("not-a-time", "UTC")).toBeNull();
  });
});
