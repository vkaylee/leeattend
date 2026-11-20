import type { TFunction } from "i18next";

export interface IntervalPreviewData {
  breakDuration: number;
  breakEnd: string;
  breakStart: string;
  endTime: string;
  intervalName: string;
  isAutoDeduct: boolean;
  isBreakOvertime?: boolean;
  isFixedBreak: boolean;
  startTime: string;
}

export interface ShiftTimelinePreviewProps {
  intervals: IntervalPreviewData[];
}

export interface TimelineBarProps {
  breakEnd: string;
  breakStart: string;
  endMin: number;
  isCrossMidnight: boolean;
  isFixedBreak: boolean;
  shiftEnd: string;
  shiftStart: string;
  startMin: number;
}

export interface ShiftDetailsListProps {
  intervals: IntervalPreviewData[];
  t: TFunction;
}
