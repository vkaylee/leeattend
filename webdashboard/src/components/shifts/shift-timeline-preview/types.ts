import type { TFunction } from "i18next";

export interface IntervalPreviewData {
  allowEarlyOt: boolean;
  breakDuration: number;
  breakEnd: string;
  breakStart: string;
  endTime: string;
  gracePeriodMinutes: number;
  intervalName: string;
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isFixedBreak: boolean;
  maxEarlyOtMinutes: number | null;
  startTime: string;
}

export interface ShiftTimelinePreviewProps {
  intervals: IntervalPreviewData[];
}

export interface TimelineBarProps {
  allowEarlyOt: boolean;
  breakEnd: string;
  breakStart: string;
  endMin: number;
  isCrossMidnight: boolean;
  isFixedBreak: boolean;
  maxEarlyOtMinutes: number;
  shiftEnd: string;
  shiftStart: string;
  startMin: number;
}

export interface ShiftDetailsListProps {
  allowEarlyOt: boolean;
  breakEnd: string;
  breakStart: string;
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isFixedBreak: boolean;
  maxEarlyOtMinutes: number;
  shiftBreak: number;
  shiftEnd: string;
  shiftGrace: number;
  shiftName: string;
  shiftStart: string;
  t: TFunction;
}
