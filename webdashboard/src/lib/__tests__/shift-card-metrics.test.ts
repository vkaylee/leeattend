import { describe, expect, it } from "vitest";

import type { Shift } from "@/lib/api";
import type { ShiftInterval } from "@/lib/shift-card-metrics";

import {
  formatDurationShort,
  getShiftCardMetrics,
  getShiftIntervals,
} from "@/lib/shift-card-metrics";

// ── Fixtures ─────────────────────────────────────────────────────────

function makeInterval(overrides: Partial<ShiftInterval> = {}): ShiftInterval {
  return {
    allow_early_ot: false,
    break_duration_minutes: 0,
    break_end: null,
    break_is_overtime: false,
    break_start: null,
    early_punch_in_grace_period_minutes: 0,
    end_time: "17:00:00",
    id: "iv-1",
    interval_name: null,
    is_break_auto_deduct: false,
    is_night_shift: false,
    max_early_ot_minutes: null,
    ot_aggregation_mode: "daily",
    sort_order: 0,
    start_time: "08:00:00",
    version_id: "v-1",
    ...overrides,
  } as ShiftInterval;
}

function makeShift(intervals: ShiftInterval[]): Shift {
  return { id: "s1", intervals, name: "Test" } as Shift;
}

// ── getShiftIntervals ────────────────────────────────────────────────

describe("getShiftIntervals", () => {
  it("returns an empty array when intervals are missing", () => {
    expect(getShiftIntervals({ id: "s1", name: "x" } as Shift)).toEqual([]);
  });

  it("returns an empty array when intervals are null", () => {
    expect(getShiftIntervals({ id: "s1", intervals: null, name: "x" } as unknown as Shift)).toEqual([]);
  });

  it("sorts intervals by sort_order", () => {
    const shift = makeShift([
      makeInterval({ id: "b", sort_order: 2, start_time: "14:00:00" }),
      makeInterval({ id: "a", sort_order: 1, start_time: "08:00:00" }),
    ]);
    expect(getShiftIntervals(shift).map((iv) => iv.id)).toEqual(["a", "b"]);
  });

  it("does not mutate the source array", () => {
    const intervals = [
      makeInterval({ id: "b", sort_order: 2 }),
      makeInterval({ id: "a", sort_order: 1 }),
    ];
    const shift = makeShift(intervals);
    getShiftIntervals(shift);
    expect(intervals.map((iv) => iv.id)).toEqual(["b", "a"]);
  });

  it("treats a missing sort_order as 0 without throwing", () => {
    const raw = { end_time: "12:00:00", start_time: "08:00:00" } as ShiftInterval;
    expect(getShiftIntervals(makeShift([raw]))).toHaveLength(1);
  });
});

// ── Single-block shifts ──────────────────────────────────────────────

describe("getShiftCardMetrics — single block", () => {
  it("reports zeroed metrics for a shift with no intervals", () => {
    const metrics = getShiftCardMetrics(makeShift([]));
    expect(metrics.blockCount).toBe(0);
    expect(metrics.totalWorkMinutes).toBe(0);
    expect(metrics.segments).toEqual([]);
    expect(metrics.spanStart).toBe("");
  });

  it("measures a plain 08:00–17:00 shift with no break", () => {
    const metrics = getShiftCardMetrics(makeShift([makeInterval()]));
    expect(metrics.blockCount).toBe(1);
    expect(metrics.totalWorkMinutes).toBe(540);
    expect(metrics.totalBreakMinutes).toBe(0);
    expect(metrics.spanStart).toBe("08:00:00");
    expect(metrics.spanEnd).toBe("17:00:00");
    expect(metrics.crossesMidnight).toBe(false);
  });

  it("deducts a fixed break from work minutes", () => {
    const metrics = getShiftCardMetrics(
      makeShift([makeInterval({ break_end: "13:00:00", break_start: "12:00:00" })]),
    );
    expect(metrics.totalBreakMinutes).toBe(60);
    expect(metrics.totalWorkMinutes).toBe(480);
    expect(metrics.intervals[0].isFixedBreak).toBe(true);
  });

  it("deducts a floating break only when auto-deduct is on", () => {
    const deducted = getShiftCardMetrics(
      makeShift([makeInterval({ break_duration_minutes: 30, is_break_auto_deduct: true })]),
    );
    expect(deducted.totalWorkMinutes).toBe(510);

    const paid = getShiftCardMetrics(
      makeShift([makeInterval({ break_duration_minutes: 30, is_break_auto_deduct: false })]),
    );
    expect(paid.totalWorkMinutes).toBe(540);
    expect(paid.totalBreakMinutes).toBe(30);
  });

  it("emits one work segment and one break segment for a fixed break", () => {
    const metrics = getShiftCardMetrics(
      makeShift([makeInterval({ break_end: "13:00:00", break_start: "12:00:00" })]),
    );
    expect(metrics.segments).toEqual([
      { endMin: 1020, kind: "work", startMin: 480 },
      { endMin: 780, kind: "break", startMin: 720 },
    ]);
  });

  it("emits no break segment for a floating break", () => {
    const metrics = getShiftCardMetrics(
      makeShift([makeInterval({ break_duration_minutes: 30, is_break_auto_deduct: true })]),
    );
    expect(metrics.segments.filter((s) => s.kind === "break")).toEqual([]);
  });
});

// ── Cross-midnight shifts ────────────────────────────────────────────

describe("getShiftCardMetrics — cross midnight", () => {
  it("measures a 22:00–06:00 night shift as 8 hours", () => {
    const metrics = getShiftCardMetrics(
      makeShift([makeInterval({ end_time: "06:00:00", is_night_shift: true, start_time: "22:00:00" })]),
    );
    expect(metrics.totalWorkMinutes).toBe(480);
    expect(metrics.crossesMidnight).toBe(true);
  });

  it("wraps the work bar into two segments at midnight", () => {
    const metrics = getShiftCardMetrics(
      makeShift([makeInterval({ end_time: "06:00:00", start_time: "22:00:00" })]),
    );
    expect(metrics.segments).toEqual([
      { endMin: 1440, kind: "work", startMin: 1320 },
      { endMin: 360, kind: "work", startMin: 0 },
    ]);
  });

  it("wraps a break that itself crosses midnight", () => {
    const metrics = getShiftCardMetrics(
      makeShift([
        makeInterval({
          break_end: "00:30:00",
          break_start: "23:30:00",
          end_time: "06:00:00",
          start_time: "22:00:00",
        }),
      ]),
    );
    const breaks = metrics.segments.filter((s) => s.kind === "break");
    expect(breaks).toEqual([
      { endMin: 1440, kind: "break", startMin: 1410 },
      { endMin: 30, kind: "break", startMin: 0 },
    ]);
    expect(metrics.totalBreakMinutes).toBe(60);
    expect(metrics.totalWorkMinutes).toBe(420);
  });
});

// ── Split shifts (the bug this module fixes) ──────────────────────────

describe("getShiftCardMetrics — split shift", () => {
  const splitShift = makeShift([
    makeInterval({ end_time: "12:00:00", id: "a", sort_order: 0, start_time: "08:00:00" }),
    makeInterval({ end_time: "18:00:00", id: "b", sort_order: 1, start_time: "14:00:00" }),
  ]);

  it("counts both blocks", () => {
    expect(getShiftCardMetrics(splitShift).blockCount).toBe(2);
  });

  it("sums work minutes across every block, not just the first", () => {
    expect(getShiftCardMetrics(splitShift).totalWorkMinutes).toBe(480);
  });

  it("spans from the first block start to the last block end", () => {
    const metrics = getShiftCardMetrics(splitShift);
    expect(metrics.spanStart).toBe("08:00:00");
    expect(metrics.spanEnd).toBe("18:00:00");
  });

  it("emits one work segment per block", () => {
    const metrics = getShiftCardMetrics(splitShift);
    expect(metrics.segments).toEqual([
      { endMin: 720, kind: "work", startMin: 480 },
      { endMin: 1080, kind: "work", startMin: 840 },
    ]);
  });

  it("keeps per-interval detail for rendering each block", () => {
    const metrics = getShiftCardMetrics(splitShift);
    expect(metrics.intervals.map((iv) => [iv.startTime, iv.endTime])).toEqual([
      ["08:00:00", "12:00:00"],
      ["14:00:00", "18:00:00"],
    ]);
  });

  it("flags cross-midnight when only the last block wraps", () => {
    const overnightSplit = makeShift([
      makeInterval({ end_time: "12:00:00", sort_order: 0, start_time: "08:00:00" }),
      makeInterval({ end_time: "02:00:00", sort_order: 1, start_time: "20:00:00" }),
    ]);
    expect(getShiftCardMetrics(overnightSplit).crossesMidnight).toBe(true);
  });
});

// ── formatDurationShort ──────────────────────────────────────────────

describe("formatDurationShort", () => {
  it.each([
    [0, "—"],
    [-30, "—"],
    [45, "45m"],
    [60, "1h"],
    [480, "8h"],
    [510, "8h30"],
    [485, "8h05"],
  ])("formats %i minutes as %s", (minutes, expected) => {
    expect(formatDurationShort(minutes)).toBe(expected);
  });
});
