import { describe, it, expect } from "vitest";

import {
  initializeGridFromDetails,
  convertGridToDetails,
  isCycleLengthInvalid,
  shouldInit,
  getFirstShiftId,
} from "../use-working-pattern-form-helpers";

import type { WorkingPatternDetail } from "@/lib/api/types";

// ── Helpers ──────────────────────────────────────────────────────────────────

function makeDetail(overrides: Partial<WorkingPatternDetail> = {}): WorkingPatternDetail {
  return {
    id: "det-1",
    pattern_id: "pat-1",
    shift_id: "s1",
    schedule_day: 0,
    shift_name: "Morning",
    ...overrides,
  } as WorkingPatternDetail;
}

// ── initializeGridFromDetails ────────────────────────────────────────────────

describe("initializeGridFromDetails", () => {
  it("creates a grid of the given length with empty arrays", () => {
    const grid = initializeGridFromDetails(5, []);
    expect(grid).toHaveLength(5);
    expect(grid.every((day) => day.length === 0)).toBe(true);
  });

  it("populates the correct day index with shift ids", () => {
    const details: WorkingPatternDetail[] = [
      makeDetail({ schedule_day: 0, shift_id: "s1" }),
      makeDetail({ schedule_day: 0, shift_id: "s2" }),
      makeDetail({ schedule_day: 2, shift_id: "s3" }),
    ];
    const grid = initializeGridFromDetails(7, details);

    expect(grid[0]).toEqual(["s1", "s2"]);
    expect(grid[1]).toEqual([]);
    expect(grid[2]).toEqual(["s3"]);
    expect(grid[3]).toEqual([]);
    expect(grid[4]).toEqual([]);
    expect(grid[5]).toEqual([]);
    expect(grid[6]).toEqual([]);
  });

  it("skips details with null shift_id", () => {
    const details: WorkingPatternDetail[] = [
      makeDetail({ schedule_day: 0, shift_id: null }),
      makeDetail({ schedule_day: 1, shift_id: "s1" }),
    ];
    const grid = initializeGridFromDetails(3, details);

    expect(grid[0]).toEqual([]);
    expect(grid[1]).toEqual(["s1"]);
  });

  it("skips details with negative schedule_day (out of bounds)", () => {
    const details: WorkingPatternDetail[] = [
      makeDetail({ schedule_day: -1, shift_id: "s1" }),
      makeDetail({ schedule_day: 0, shift_id: "s2" }),
    ];
    const grid = initializeGridFromDetails(3, details);

    expect(grid[0]).toEqual(["s2"]);
    expect(grid[1]).toEqual([]);
  });

  it("skips details with schedule_day >= length (out of bounds)", () => {
    const details: WorkingPatternDetail[] = [
      makeDetail({ schedule_day: 5, shift_id: "s1" }),
      makeDetail({ schedule_day: 0, shift_id: "s2" }),
    ];
    const grid = initializeGridFromDetails(3, details);

    expect(grid[0]).toEqual(["s2"]);
    // schedule_day=5 is >= 3 → skipped
    expect(grid).toHaveLength(3);
  });

  it("handles an empty details array", () => {
    const grid = initializeGridFromDetails(10, []);
    expect(grid).toHaveLength(10);
    expect(grid.every((d) => d.length === 0)).toBe(true);
  });

  it("handles len=0 gracefully (empty grid)", () => {
    const details: WorkingPatternDetail[] = [
      makeDetail({ schedule_day: 0, shift_id: "s1" }),
    ];
    const grid = initializeGridFromDetails(0, details);
    expect(grid).toEqual([]);
  });
});

// ── convertGridToDetails ────────────────────────────────────────────────────

describe("convertGridToDetails", () => {
  it("flattens multiple shifts per day", () => {
    const grid = [["s1", "s2"], ["s3"]];
    expect(convertGridToDetails(grid)).toEqual([
      { schedule_day: 0, shift_id: "s1" },
      { schedule_day: 0, shift_id: "s2" },
      { schedule_day: 1, shift_id: "s3" },
    ]);
  });

  it("emits null shift_id for empty days", () => {
    const grid = [["s1"], [], ["s2"]];
    expect(convertGridToDetails(grid)).toEqual([
      { schedule_day: 0, shift_id: "s1" },
      { schedule_day: 1, shift_id: null },
      { schedule_day: 2, shift_id: "s2" },
    ]);
  });

  it("returns an empty array for an empty grid", () => {
    expect(convertGridToDetails([])).toEqual([]);
  });

  it("handles a grid where every day is empty", () => {
    const grid = [[], [], []];
    expect(convertGridToDetails(grid)).toEqual([
      { schedule_day: 0, shift_id: null },
      { schedule_day: 1, shift_id: null },
      { schedule_day: 2, shift_id: null },
    ]);
  });

  it("handles a single-day grid with multiple shifts", () => {
    const grid = [["s1", "s2", "s3"]];
    expect(convertGridToDetails(grid)).toEqual([
      { schedule_day: 0, shift_id: "s1" },
      { schedule_day: 0, shift_id: "s2" },
      { schedule_day: 0, shift_id: "s3" },
    ]);
  });
});

// ── isCycleLengthInvalid ────────────────────────────────────────────────────

describe("isCycleLengthInvalid", () => {
  it("returns true for rotation with len < 1", () => {
    expect(isCycleLengthInvalid("rotation", 0)).toBe(true);
    expect(isCycleLengthInvalid("rotation", -10)).toBe(true);
  });

  it("returns true for rotation with len > 366", () => {
    expect(isCycleLengthInvalid("rotation", 367)).toBe(true);
    expect(isCycleLengthInvalid("rotation", 500)).toBe(true);
    expect(isCycleLengthInvalid("rotation", 1000)).toBe(true);
  });

  it("returns false for rotation with valid range", () => {
    expect(isCycleLengthInvalid("rotation", 1)).toBe(false);
    expect(isCycleLengthInvalid("rotation", 7)).toBe(false);
    expect(isCycleLengthInvalid("rotation", 30)).toBe(false);
    expect(isCycleLengthInvalid("rotation", 365)).toBe(false);
    expect(isCycleLengthInvalid("rotation", 366)).toBe(false);
  });

  it("always returns false for weekly pattern type regardless of length", () => {
    expect(isCycleLengthInvalid("weekly", 0)).toBe(false);
    expect(isCycleLengthInvalid("weekly", -1)).toBe(false);
    expect(isCycleLengthInvalid("weekly", 1000)).toBe(false);
    expect(isCycleLengthInvalid("weekly", 7)).toBe(false);
  });

  it("returns false for unknown pattern types", () => {
    expect(isCycleLengthInvalid("custom", 367)).toBe(false);
    expect(isCycleLengthInvalid("", 0)).toBe(false);
  });
});

// ── shouldInit ──────────────────────────────────────────────────────────────

describe("shouldInit", () => {
  it("returns true when in edit mode, pattern exists, not loading, and not yet initialised", () => {
    expect(shouldInit(true, { id: "p1" }, false, false)).toBe(true);
  });

  it("returns false when not in edit mode", () => {
    expect(shouldInit(false, { id: "p1" }, false, false)).toBe(false);
  });

  it("returns false when pattern is falsy (null/undefined/false)", () => {
    expect(shouldInit(true, null, false, false)).toBe(false);
    expect(shouldInit(true, undefined, false, false)).toBe(false);
    expect(shouldInit(true, false, false, false)).toBe(false);
    expect(shouldInit(true, 0, false, false)).toBe(false);
  });

  it("returns false when still loading", () => {
    expect(shouldInit(true, { id: "p1" }, true, false)).toBe(false);
  });

  it("returns false when already initialised", () => {
    expect(shouldInit(true, { id: "p1" }, false, true)).toBe(false);
  });

  it("returns false when all conditions fail", () => {
    expect(shouldInit(false, null, true, true)).toBe(false);
  });
});

// ── getFirstShiftId ─────────────────────────────────────────────────────────

describe("getFirstShiftId", () => {
  it("returns the id of the first shift in the list", () => {
    expect(getFirstShiftId([{ id: "s1" }, { id: "s2" }])).toBe("s1");
  });

  it("returns undefined for an empty shifts list", () => {
    expect(getFirstShiftId([])).toBeUndefined();
  });

  it("returns the first shift id even when shift objects have extra fields", () => {
    const shifts = [
      { id: "s-prime", name: "Prime Shift", start_time: "08:00" },
      { id: "s-late", name: "Late Shift" },
    ];
    expect(getFirstShiftId(shifts)).toBe("s-prime");
  });
});
