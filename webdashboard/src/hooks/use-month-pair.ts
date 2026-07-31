import { useMemo } from "react";

export interface MonthPair {
  curMonth: number;
  curYear: number;
  prevMonth: number;
  prevYear: number;
}

/**
 * Compute current and previous month/year from a given date.
 * Uses the `new Date(year, monthIndex, 1)` constructor to avoid
 * the JavaScript Date bug where `setMonth()` on day 31 silently
 * rolls over into the next month when the target month has fewer
 * than 31 days (e.g. July 31 → setMonth(June) → July 1).
 *
 * Pure function — safe to call outside React components.
 */
export function getMonthPair(now: Date = new Date()): MonthPair {
  const curMonth = now.getMonth() + 1;
  const curYear = now.getFullYear();
  const prev = new Date(curYear, now.getMonth() - 1, 1);
  const prevMonth = prev.getMonth() + 1;
  const prevYear = prev.getFullYear();
  return { curMonth, curYear, prevMonth, prevYear };
}

/**
 * React hook returning current and previous month/year.
 * Memoized on the current date so re-renders within the same
 * day don't recompute.
 */
export function useMonthPair(): MonthPair {
  return useMemo(() => getMonthPair(), []);
}
