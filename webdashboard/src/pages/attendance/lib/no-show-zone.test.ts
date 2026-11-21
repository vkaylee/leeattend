import { describe, expect, it } from "vitest";

import {
  computeZone,
  epochMinutesInTz,
  isViewingPastDate,
  parseTimeToMinutes,
  type ComputeZoneInput,
} from "./no-show-zone";

/**
 * The new `computeZone` is TZ-aware and works in absolute epoch-minutes. To keep
 * tests deterministic we pin the organization timezone to UTC and build `referenceTime`
 * with `Date.UTC` (so results do not depend on the runner's local timezone).
 */
const TZ = "UTC";
const SHIFT_DATE = "2026-01-15";

/** Build a `Date` at an absolute UTC instant (y/M/d are UTC components). */
const atUTC = (hh: number, mm = 0, ss = 0): Date =>
  new Date(Date.UTC(2026, 0, 15, hh, mm, ss));

/** Convenience: a full `ComputeZoneInput` with the shift on SHIFT_DATE in UTC. */
const input = (o: Partial<ComputeZoneInput>): ComputeZoneInput => ({
  scheduledStart: "08:00:00",
  scheduledEnd: "17:00:00",
  graceMinutes: 15,
  referenceTime: atUTC(8, 0),
  shiftDate: SHIFT_DATE,
  tz: TZ,
  ...o,
});

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

describe("epochMinutesInTz", () => {
  it("maps a UTC wall-clock to the same UTC epoch-minutes", () => {
    // 08:00 UTC on 2026-01-15 → 0*24*60... but epoch-minutes is absolute since
    // 1970, so compare against Date.UTC directly rather than a literal.
    const mins = epochMinutesInTz("2026-01-15", "08:00:00", "UTC");
    expect(mins).toBe(Math.round(Date.UTC(2026, 0, 15, 8, 0, 0) / 60_000));
  });

  it("respects a non-UTC timezone offset", () => {
    // 08:00 in +07:00 == 01:00 UTC.
    const tz = "Asia/Ho_Chi_Minh"; // +07:00, no DST
    const mins = epochMinutesInTz("2026-01-15", "08:00:00", tz);
    expect(mins).toBe(Math.round(Date.UTC(2026, 0, 15, 1, 0, 0) / 60_000));
  });

  it("handles HH:MM (no seconds)", () => {
    const mins = epochMinutesInTz("2026-01-15", "08:30", "UTC");
    expect(mins).toBe(Math.round(Date.UTC(2026, 0, 15, 8, 30, 0) / 60_000));
  });
});

describe("computeZone", () => {
  it("Zone A while within the grace window", () => {
    // 15-min grace → grace window ends at 08:15.
    const info = computeZone(input({ referenceTime: atUTC(8, 10) }));
    expect(info.zone).toBe("a");
    expect(info.minutes).toBe(5); // 5 min of grace left
  });

  it("Zone A exactly at start + grace (boundary inclusive)", () => {
    const info = computeZone(input({ referenceTime: atUTC(8, 15, 0) }));
    expect(info.zone).toBe("a");
    expect(info.minutes).toBe(0);
  });

  it("Zone B one second past the grace window", () => {
    const info = computeZone(input({ referenceTime: atUTC(8, 15, 1) }));
    expect(info.zone).toBe("b");
    expect(info.minutes).toBe(1);
  });

  it("Zone B mid-shift with elapsed minutes", () => {
    const info = computeZone(input({ referenceTime: atUTC(10, 0) }));
    expect(info.zone).toBe("b");
    expect(info.minutes).toBe(105); // 08:15 → 10:00
  });

  it("Zone C once the shift has ended (boundary inclusive)", () => {
    const info = computeZone(input({ referenceTime: atUTC(17, 0, 0) }));
    expect(info.zone).toBe("c");
    expect(info.minutes).toBeNull();
  });

  it("Zone C after the shift ended", () => {
    expect(computeZone(input({ referenceTime: atUTC(18, 30) })).zone).toBe("c");
  });

  it("defaults grace to 15 when null", () => {
    // With default 15 grace, 08:10 → Zone A (5 min left).
    const info = computeZone(input({ graceMinutes: null, referenceTime: atUTC(8, 10) }));
    expect(info.zone).toBe("a");
    expect(info.minutes).toBe(5);
  });

  it("falls back to Zone C when shift times are missing", () => {
    expect(computeZone(input({ scheduledStart: null })).zone).toBe("c");
    expect(computeZone(input({ scheduledEnd: null })).zone).toBe("c");
    expect(computeZone(input({ scheduledStart: null, scheduledEnd: null })).zone).toBe("c");
  });

  it("falls back to Zone C when shiftDate/tz are missing", () => {
    expect(computeZone(input({ shiftDate: "" })).zone).toBe("c");
    expect(computeZone(input({ tz: "" })).zone).toBe("c");
  });

  it("handles an overnight shift crossing midnight", () => {
    // 22:00 → 06:00 next day. At 23:00 (same calendar day) we are past
    // start+grace (22:15) and before end → Zone B.
    const info = computeZone(
      input({
        scheduledStart: "22:00:00",
        scheduledEnd: "06:00:00",
        referenceTime: atUTC(23, 0),
      }),
    );
    expect(info.zone).toBe("b");
  });

  it("is TZ-aware: the same wall-clock in different tz yields the same zone", () => {
    // Shift 08:00-17:00. Reference 08:10 local.
    const base = {
      scheduledStart: "08:00:00",
      scheduledEnd: "17:00:00",
      graceMinutes: 15,
      shiftDate: "2026-01-15",
    };
    const utc = computeZone({ ...base, referenceTime: atUTC(8, 10), tz: "UTC" });
    // 08:10 in +07:00 == 01:10 UTC, but the shift is ALSO interpreted in +07:00,
    // so it is still 08:10 local → still in grace.
    const saigon = computeZone({
      ...base,
      referenceTime: new Date(Date.UTC(2026, 0, 15, 1, 10, 0)),
      tz: "Asia/Ho_Chi_Minh",
    });
    expect(utc.zone).toBe("a");
    expect(saigon.zone).toBe("a");
    expect(saigon.minutes).toBe(utc.minutes);
  });

  it("supports a pinned past reference time (historical scrub)", () => {
    // 08:10 on the shift day, but pinned as a past instant — same math, Zone A.
    const info = computeZone(input({ referenceTime: atUTC(8, 10) }));
    expect(info.zone).toBe("a");
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
