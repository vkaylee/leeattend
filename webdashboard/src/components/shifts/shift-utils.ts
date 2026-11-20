import type { ShiftWithIntervals } from "@/pages/shifts/hooks/use-shifts-types";

import { Shift } from "@/lib/api";

/** Check whether any interval of a shift is a night shift. */
export function isNightShift(shift: Shift): boolean {
  const typed = shift as ShiftWithIntervals;
  return typed.intervals?.some((iv) => {
    const start = iv.start_time ?? "";
    const end = iv.end_time ?? "";
    return start > end;
  }) ?? false;
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

  return intervalTimesMatchQuery(shift, q);
}

function intervalTimesMatchQuery(shift: Shift, q: string): boolean {
  const typed = shift as ShiftWithIntervals;
  const intervals = typed.intervals ?? [];
  return intervals.some(
    (iv) => iv.start_time.includes(q) || iv.end_time.includes(q),
  );
}
