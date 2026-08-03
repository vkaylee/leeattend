import type { Shift } from "@/lib/api";

/** Extract interval fields from the first interval of a shift. */
// eslint-disable-next-line complexity -- intentionally co-located null-safe accessors; each ?? branch is trivial
export function parseShiftInterval(shift: Shift) {
  const iv = shift.intervals?.[0];
  return {
    startTime: iv?.start_time ?? null,
    endTime: iv?.end_time ?? null,
    breakStart: iv?.break_start ?? null,
    breakEnd: iv?.break_end ?? null,
    breakDuration: iv?.break_duration_minutes ?? 0,
    isAutoDeduct: iv?.is_break_auto_deduct ?? true,
  };
}
