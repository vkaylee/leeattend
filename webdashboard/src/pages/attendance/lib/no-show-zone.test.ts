import { describe, expect, it } from "vitest";

import {
  computeZone,
  isViewingPastDate,
  parseTimeToMinutes,
} from "./no-show-zone";

/** Build a `Date` at HH:MM:SS on an arbitrary reference day (date part is irrelevant). */
const at = (hh: number, mm = 0, ss = 0): Date =>
  new Date(2026, 0, 15, hh, mm, ss);

describe("parseTimeToMinutes", () => {
  it("parses HH:MM:SS", () => {
    expect(parseTimeToMinutes("08:30:00")).toBe(510);
  });
  it("parses HH:MM", () => {
    expect(parseTimeToMinutes("17:00")).toBe(1020);
  });
  it("returns null for null/empty/invalid", () => {
    expect(parseTimeToMinutes(null)).toBeNull();
    expect(parseTimeToMinutes(undefined)).toBeNull();
    expect(parseTimeToMinutes("")).toBeNull();
    expect(parseTimeToMinutes("nope")).toBeNull();
  });
});

describe("computeZone", () => {
  const shift = { start: "08:00:00", end: "17:00:00" };

  it("Zone A while within the grace window", () => {
    // 15-min grace → grace window ends at 08:15.
    const info = computeZone(shift.start, shift.end, 15, at(8, 10));
    expect(info.zone).toBe("a");
    expect(info.minutes).toBe(5); // 5 min of grace left
  });

  it("Zone A exactly at start + grace (boundary inclusive)", () => {
    const info = computeZone(shift.start, shift.end, 15, at(8, 15, 0));
    expect(info.zone).toBe("a");
    expect(info.minutes).toBe(0);
  });

  it("Zone B one second past the grace window", () => {
    const info = computeZone(shift.start, shift.end, 15, at(8, 15, 1));
    expect(info.zone).toBe("b");
    expect(info.minutes).toBe(1);
  });

  it("Zone B mid-shift with elapsed minutes", () => {
    const info = computeZone(shift.start, shift.end, 15, at(10, 0));
    expect(info.zone).toBe("b");
    expect(info.minutes).toBe(105); // 08:15 → 10:00
  });

  it("Zone C once the shift has ended (boundary inclusive)", () => {
    const info = computeZone(shift.start, shift.end, 15, at(17, 0, 0));
    expect(info.zone).toBe("c");
    expect(info.minutes).toBeNull();
  });

  it("Zone C after the shift ended", () => {
    expect(computeZone(shift.start, shift.end, 15, at(18, 30)).zone).toBe("c");
  });

  it("defaults grace to 15 when null", () => {
    // With default 15 grace, 08:10 → Zone A (5 min left).
    const info = computeZone(shift.start, shift.end, null, at(8, 10));
    expect(info.zone).toBe("a");
    expect(info.minutes).toBe(5);
  });

  it("falls back to Zone C when shift times are missing", () => {
    expect(computeZone(null, shift.end, 15, at(10, 0)).zone).toBe("c");
    expect(computeZone(shift.start, null, 15, at(10, 0)).zone).toBe("c");
    expect(computeZone(null, null, 15, at(10, 0)).zone).toBe("c");
  });

  it("uses start-of-midnight shift edge", () => {
    const info = computeZone("22:00:00", "06:00:00", 15, at(23, 0));
    // past start+grace (22:15) and before end → Zone B
    expect(info.zone).toBe("b");
  });
});

describe("isViewingPastDate", () => {
  it("true for a date before today", () => {
    expect(isViewingPastDate("2026-01-01", "2026-07-10")).toBe(true);
  });
  it("false for today", () => {
    expect(isViewingPastDate("2026-07-10", "2026-07-10")).toBe(false);
  });
  it("false for a future date", () => {
    expect(isViewingPastDate("2026-12-31", "2026-07-10")).toBe(false);
  });
  it("false when inputs are empty", () => {
    expect(isViewingPastDate("", "2026-07-10")).toBe(false);
    expect(isViewingPastDate("2026-01-01", "")).toBe(false);
  });
});
