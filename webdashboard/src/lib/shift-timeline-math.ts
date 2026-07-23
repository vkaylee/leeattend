/**
 * Pure math utilities for shift timeline visualization.
 * No React dependencies — fully testable in isolation.
 */

/** Convert "HH:MM" to minutes since midnight. Returns 0 for empty/missing strings. */
export const timeToMinutes = (t: string): number => {
  if (!t) return 0;
  const [h, m] = t.split(":").map(Number);
  return (h || 0) * 60 + (m || 0);
};

/** Calculate CSS left/width percentages for a timeline segment. */
export const getSegmentStyles = (start: number, end: number) => {
  const left = (start / 1440) * 100;
  const width = end >= start ? ((end - start) / 1440) * 100 : 0;
  return { left: `${left}%`, width: `${width}%` };
};

/** Calculate shift duration in minutes, handling cross-midnight shifts. */
export const calculateShiftDuration = (
  shiftStart: string,
  shiftEnd: string,
  startMin: number,
  endMin: number,
): number => {
  if (!shiftStart || !shiftEnd) return 0;
  if (endMin >= startMin) {
    return endMin - startMin;
  }
  return 1440 - startMin + endMin;
};

/** Calculate break duration in minutes. Handles fixed-break timing and cross-midnight. */
export const calculateBreakDuration = (
  isFixedBreak: boolean,
  breakStart: string,
  breakEnd: string,
  shiftBreak: number,
): number => {
  if (isFixedBreak && breakStart && breakEnd) {
    const bStart = timeToMinutes(breakStart);
    const bEnd = timeToMinutes(breakEnd);
    if (bEnd >= bStart) {
      return bEnd - bStart;
    }
    return 1440 - bStart + bEnd;
  }
  if (!isFixedBreak) {
    return shiftBreak || 0;
  }
  return 0;
};
