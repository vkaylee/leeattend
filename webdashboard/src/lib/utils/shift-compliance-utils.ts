/**
 * Client-side shift compliance utilities.
 * Detects overlapping shifts, insufficient rest between shifts,
 * and weekly hours violations — all computed in the browser
 * without requiring API changes.
 */

/** Represents a time interval in HH:mm format */
export interface TimeRange {
  start: string; // HH:mm
  end: string;   // HH:mm
}

/** A shift with its identifying info and time range */
export interface ShiftTimeBlock {
  shiftId: string;
  shiftName: string;
  start: string; // HH:mm
  end: string;   // HH:mm
  day?: number;  // schedule_day index, if applicable
}

/** Detected overlapping shifts */
export interface OverlapConflict {
  type: "overlap";
  shiftA: string;
  shiftB: string;
  day?: number;
}

/** Detected insufficient rest between shifts */
export interface RestViolation {
  type: "rest";
  /** Name of the earlier shift */
  shiftA: string;
  /** Name of the later shift */
  shiftB: string;
  /** Rest duration in hours (always < 12) */
  restHours: number;
  day?: number;
}

/** Detected weekly hours exceeding the limit */
export interface HoursViolation {
  type: "hours";
  /** Total weekly minutes worked */
  totalMinutes: number;
  /** The configured limit in minutes (default 2880 = 48h) */
  limitMinutes: number;
}

export type ComplianceIssue = OverlapConflict | RestViolation | HoursViolation;

/**
 * Parse a "HH:mm" string into total minutes since midnight.
 * Returns 0 for invalid inputs.
 */
function parseMinutes(time: string): number {
  const parts = time.split(":");
  const h = parseInt(parts[0] ?? "0", 10);
  const m = parseInt(parts[1] ?? "0", 10);
  if (isNaN(h) || isNaN(m)) return 0;
  return h * 60 + m;
}

/**
 * Detect overlapping shift blocks within the same set.
 * A shift that wraps past midnight (end < start) is treated as
 * running from start to 24:00 + end for overlap comparison.
 *
 * Returns conflicts where two blocks' time ranges intersect.
 */
export function checkShiftOverlap(blocks: ShiftTimeBlock[]): OverlapConflict[] {
  const conflicts: OverlapConflict[] = [];
  const withMinutes = blocks.map((b) => {
    const s = parseMinutes(b.start);
    let e = parseMinutes(b.end);
    // Handle overnight shifts (e.g., 22:00–06:00)
    if (e <= s) e += 24 * 60;
    return { ...b, startMin: s, endMin: e };
  });

  for (let i = 0; i < withMinutes.length; i++) {
    for (let j = i + 1; j < withMinutes.length; j++) {
      const a = withMinutes[i];
      const b = withMinutes[j];
      // Overlap exists if a.start < b.end AND b.start < a.end
      if (a.startMin < b.endMin && b.startMin < a.endMin) {
        conflicts.push({
          type: "overlap",
          shiftA: a.shiftName,
          shiftB: b.shiftName,
          day: a.day ?? b.day,
        });
      }
    }
  }

  return conflicts;
}

/**
 * Check rest hours between consecutive shifts sorted by start time.
 * Returns violations where rest < 12 hours (720 minutes).
 */
export function checkRestHours(
  blocks: ShiftTimeBlock[],
  day?: number,
): RestViolation[] {
  if (blocks.length < 2) return [];

  const MIN_REST_MINUTES = 12 * 60; // 12 hours

  // Sort by start time
  const sorted = [...blocks]
    .map((b) => {
      const s = parseMinutes(b.start);
      let e = parseMinutes(b.end);
      if (e <= s) e += 24 * 60;
      return { ...b, startMin: s, endMin: e };
    })
    .sort((a, b) => a.startMin - b.startMin);

  const violations: RestViolation[] = [];

  for (let i = 0; i < sorted.length - 1; i++) {
    const current = sorted[i];
    const next = sorted[i + 1];
    // Rest = next start - current end
    const restMinutes = next.startMin - current.endMin;
    if (restMinutes < MIN_REST_MINUTES) {
      violations.push({
        type: "rest",
        shiftA: current.shiftName,
        shiftB: next.shiftName,
        restHours: Math.round((restMinutes / 60) * 10) / 10,
        day: day ?? current.day,
      });
    }
  }

  return violations;
}

/**
 * Check if total weekly minutes exceed the configured limit.
 * Default limit: 48 hours (2880 minutes).
 *
 * @param totalMinutes Total minutes worked in the week
 * @param limitMinutes Weekly limit in minutes (default 2880)
 * @returns HoursViolation if exceeded, null otherwise
 */
export function checkWeeklyHours(
  totalMinutes: number,
  limitMinutes = 2880,
): HoursViolation | null {
  if (totalMinutes > limitMinutes) {
    return {
      type: "hours",
      totalMinutes,
      limitMinutes,
    };
  }
  return null;
}

/**
 * Compute net working minutes for a shift block,
 * subtracting break duration.
 */
export function computeNetMinutes(
  start: string,
  end: string,
  breakDurationMinutes = 0,
): number {
  let s = parseMinutes(start);
  let e = parseMinutes(end);
  if (e <= s) e += 24 * 60;
  const gross = e - s;
  return Math.max(0, gross - breakDurationMinutes);
}

/**
 * Format minutes as a human-readable hours string.
 * Examples: 480 → "8h", 510 → "8.5h", 45 → "45m"
 */
export function formatDuration(minutes: number): string {
  if (minutes < 60) return `${minutes}m`;
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  if (m === 0) return `${h}h`;
  return `${h}.${Math.round(m / 6)}h`;
}

/**
 * Aggregate net weekly minutes from a set of shift details
 * and their corresponding shift definitions.
 */
export interface ShiftRef {
  id: string;
  name: string;
  intervals?: Array<{
    start_time?: string;
    end_time?: string;
    break_duration_minutes?: number;
  }> | null;
}

export interface DayShiftAssignment {
  scheduleDay: number;
  shiftId: string;
}

export function computeWeeklyTotal(
  assignments: DayShiftAssignment[],
  shifts: ShiftRef[],
): number {
  let total = 0;
  for (const a of assignments) {
    const shift = shifts.find((s) => s.id === a.shiftId);
    if (!shift?.intervals?.length) continue;
    // Sum all intervals for the shift
    for (const iv of shift.intervals) {
      if (iv.start_time && iv.end_time) {
        total += computeNetMinutes(
          iv.start_time,
          iv.end_time,
          iv.break_duration_minutes ?? 0,
        );
      }
    }
  }
  return total;
}
