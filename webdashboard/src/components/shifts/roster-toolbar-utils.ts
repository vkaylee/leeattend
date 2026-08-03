import type { Shift } from "@/lib/api/types";

import { getShiftIntervals } from "@/lib/shift-card-metrics";

/** Compact time-range label for a shift, e.g. "08:00–17:00". */
export function formatShiftTimeRange(shift: Shift): string {
  const intervals = getShiftIntervals(shift);
  if (intervals.length === 0) return "";
  const starts = intervals.map((iv) => iv.start_time);
  const ends = intervals.map((iv) => iv.end_time);
  return `${starts.join("/")}–${ends.join("/")}`;
}
