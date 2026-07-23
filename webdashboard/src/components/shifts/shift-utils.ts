import type { ShiftWithIntervals } from "@/pages/shifts/hooks/use-shifts-types";

import { Shift } from "@/lib/api";

/** Check whether any interval of a shift is a night shift. */
export function isNightShift(shift: Shift): boolean {
  const typed = shift as ShiftWithIntervals;
  return typed.intervals?.some((iv) => iv.is_night_shift === true) ?? false;
}

/** Check whether a shift is a split/multi-block shift (> 1 interval). */
export function isSplitShift(shift: Shift): boolean {
  const typed = shift as ShiftWithIntervals;
  return (typed.intervals?.length ?? 0) > 1;
}

/** Returns true when the query matches the shift name, report symbol, or any interval time. */
export function shiftMatchesQuery(shift: Shift, query: string): boolean {
  const q = query.toLowerCase().trim();
  if (!q) return true;

  if (shift.name.toLowerCase().includes(q)) return true;
  if (shift.report_symbol?.toLowerCase().includes(q)) return true;

  const typed = shift as ShiftWithIntervals;
  const intervals = typed.intervals ?? [];
  for (const iv of intervals) {
    if (iv.start_time?.includes(q)) return true;
    if (iv.end_time?.includes(q)) return true;
  }

  return false;
}
