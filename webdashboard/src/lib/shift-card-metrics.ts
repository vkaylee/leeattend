/**
 * Aggregate metrics for a shift across ALL of its intervals.
 *
 * The shift picker card previously read `intervals[0]` only, which silently
 * mis-rendered split shifts (a 08:00–12:00 + 14:00–18:00 shift appeared as a
 * plain 08:00–12:00 shift). Everything here is pure and React-free so the
 * arithmetic can be unit tested in isolation.
 */

import type { Shift } from "@/lib/api";

import {
  calculateBreakDuration,
  calculateShiftDuration,
  timeToMinutes,
} from "@/lib/shift-timeline-math";

const MINUTES_PER_DAY = 1440;

export type ShiftInterval = NonNullable<Shift["intervals"]>[number];

export type ShiftSegmentKind = "break" | "work";

export interface ShiftSegment {
  endMin: number;
  kind: ShiftSegmentKind;
  startMin: number;
}

export interface ShiftIntervalMetrics {
  breakEnd: string;
  breakMinutes: number;
  breakStart: string;
  crossesMidnight: boolean;
  durationMinutes: number;
  endTime: string;
  isBreakDeducted: boolean;
  isFixedBreak: boolean;
  name: string;
  startTime: string;
  workMinutes: number;
}

export interface ShiftCardMetrics {
  blockCount: number;
  crossesMidnight: boolean;
  expectedPunches: number;
  intervals: ShiftIntervalMetrics[];
  segments: ShiftSegment[];
  spanEnd: string;
  spanStart: string;
  totalBreakMinutes: number;
  totalDurationMinutes: number;
  totalWorkMinutes: number;
}

/**
 * Read `sort_order` defensively: the generated schema types it as required, but
 * older payloads (and hand-built test fixtures) may omit it.
 */
function sortOrderOf(interval: ShiftInterval): number {
  const value: unknown = (interval as { sort_order?: unknown }).sort_order;
  return typeof value === "number" ? value : 0;
}

/** Intervals of a shift in display order. Never returns null. */
export function getShiftIntervals(shift: Shift): ShiftInterval[] {
  const intervals = shift.intervals ?? [];
  return [...intervals].sort((a, b) => sortOrderOf(a) - sortOrderOf(b));
}

/**
 * Split a time range into 24h-timeline segments, wrapping at midnight so a
 * 22:00–06:00 shift renders as two bars instead of one inverted bar.
 */
function splitAtMidnight(startMin: number, endMin: number): [number, number][] {
  if (endMin > startMin) return [[startMin, endMin]];
  if (endMin === startMin) return [];
  return [
    [startMin, MINUTES_PER_DAY],
    [0, endMin],
  ];
}

function toSegments(
  startMin: number,
  endMin: number,
  kind: ShiftSegmentKind,
): ShiftSegment[] {
  return splitAtMidnight(startMin, endMin).map(([from, to]) => ({
    endMin: to,
    kind,
    startMin: from,
  }));
}

function measureInterval(interval: ShiftInterval): ShiftIntervalMetrics {
  const startTime = interval.start_time;
  const endTime = interval.end_time;
  const breakStart = interval.break_start ?? "";
  const breakEnd = interval.break_end ?? "";
  const startMin = timeToMinutes(startTime);
  const endMin = timeToMinutes(endTime);
  const hasValidWorkRange = Boolean(startTime && endTime)
    && Number.isFinite(startMin)
    && Number.isFinite(endMin);
  const isFixedBreak = Boolean(breakStart && breakEnd);
  const breakStartMin = timeToMinutes(breakStart);
  const breakEndMin = timeToMinutes(breakEnd);
  const hasValidFixedBreak = isFixedBreak
    && Number.isFinite(breakStartMin)
    && Number.isFinite(breakEndMin);
  const durationMinutes = hasValidWorkRange
    ? calculateShiftDuration(startTime, endTime, startMin, endMin)
    : 0;
  const breakMinutes = hasValidFixedBreak || !isFixedBreak
    ? calculateBreakDuration(
      hasValidFixedBreak,
      breakStart,
      breakEnd,
      interval.break_duration_minutes,
    )
    : 0;

  const isBreakAutoDeduct = interval.is_break_auto_deduct ?? true;
  const isBreakDeducted = isBreakAutoDeduct && breakMinutes > 0;

  return {
    breakEnd,
    breakMinutes,
    breakStart,
    crossesMidnight: endMin < startMin,
    durationMinutes,
    endTime,
    isBreakDeducted,
    isFixedBreak,
    name: interval.interval_name ?? "",
    startTime,
    workMinutes: Math.max(0, durationMinutes - (isBreakDeducted ? breakMinutes : 0)),
  };
}

const EMPTY_METRICS: ShiftCardMetrics = {
  blockCount: 0,
  crossesMidnight: false,
  expectedPunches: 0,
  intervals: [],
  segments: [],
  spanEnd: "",
  spanStart: "",
  totalBreakMinutes: 0,
  totalDurationMinutes: 0,
  totalWorkMinutes: 0,
};

/** Aggregate every interval of a shift into the numbers a picker card shows. */
export function getShiftCardMetrics(shift: Shift): ShiftCardMetrics {
  const intervals = getShiftIntervals(shift);
  if (intervals.length === 0) return EMPTY_METRICS;

  const measured = intervals.map(measureInterval);
  const segments: ShiftSegment[] = [];

  for (const iv of measured) {
    segments.push(...toSegments(timeToMinutes(iv.startTime), timeToMinutes(iv.endTime), "work"));
    if (iv.isFixedBreak) {
      segments.push(
        ...toSegments(timeToMinutes(iv.breakStart), timeToMinutes(iv.breakEnd), "break"),
      );
    }
  }

  const last = measured[measured.length - 1];

  const totalBreakMinutes = measured.reduce((sum, iv) => sum + iv.breakMinutes, 0);
  const totalDurationMinutes = measured.reduce((sum, iv) => sum + iv.durationMinutes, 0);

  return {
    blockCount: measured.length,
    crossesMidnight: measured.some((iv) => iv.crossesMidnight),
    expectedPunches: measured.length * 2,
    intervals: measured,
    segments,
    spanEnd: last.endTime,
    spanStart: measured[0].startTime,
    totalBreakMinutes,
    totalDurationMinutes,
    totalWorkMinutes: measured.reduce((sum, iv) => sum + iv.workMinutes, 0),
  };
}

/** Compact duration label: 480 → "8h", 510 → "8h30". Returns "—" for zero. */
export function formatDurationShort(minutes: number): string {
  if (minutes <= 0) return "—";
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours === 0) return `${String(mins)}m`;
  if (mins === 0) return `${String(hours)}h`;
  return `${String(hours)}h${String(mins).padStart(2, "0")}`;
}
