import { Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

import { calculateBreakDuration, calculateShiftDuration, timeToMinutes } from "@/lib/shift-timeline-math";

import { MetricsGrid } from "./shift-timeline-preview/metrics-grid";
import { ShiftDetailsList } from "./shift-timeline-preview/shift-details-list";
import { TimelineBar } from "./shift-timeline-preview/timeline-bar";
import type { ShiftTimelinePreviewProps } from "./shift-timeline-preview/types";

// Re-export for consumers that previously imported from this file
export { TimelineBar } from "./shift-timeline-preview/timeline-bar";
export { ShiftDetailsList, BreakPolicy, GracePeriod, EarlyOt } from "./shift-timeline-preview/shift-details-list";
export { MetricsGrid } from "./shift-timeline-preview/metrics-grid";
export type { IntervalPreviewData, ShiftTimelinePreviewProps } from "./shift-timeline-preview/types";

export function ShiftTimelinePreview({
  intervals,
}: ShiftTimelinePreviewProps) {
  const { t } = useTranslation();

  const iv = intervals[0];
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
  if (!iv) {
    return (
      <div className="bg-slate-50 dark:bg-slate-800 p-8 rounded-[32px] border border-slate-100 dark:border-slate-700/80 text-center text-slate-400">
        {t("work_segments.intervals.no_intervals", "Chưa có khung giờ")}
      </div>
    );
  }

  const {
    intervalName: shiftName,
    startTime: shiftStart,
    endTime: shiftEnd,
    breakDuration: shiftBreak,
    gracePeriodMinutes: shiftGrace,
    breakStart,
    breakEnd,
    isFixedBreak,
    isAutoDeduct,
    isBreakOvertime,
    allowEarlyOt,
    maxEarlyOtMinutes: maxEarlyOt,
  } = iv;

  const startMin = timeToMinutes(shiftStart);
  const endMin = timeToMinutes(shiftEnd);
  const shiftDurationMin = calculateShiftDuration(shiftStart, shiftEnd, startMin, endMin);
  const breakDurationMin = calculateBreakDuration(isFixedBreak, breakStart, breakEnd, shiftBreak);
  const workingMinutes = Math.max(0, shiftDurationMin - (isAutoDeduct || isFixedBreak ? breakDurationMin : 0));
  const workingHours = (workingMinutes / 60).toFixed(2);
  const isCrossMidnight = endMin < startMin;

  return (
    <div className="bg-slate-50 dark:bg-slate-800 p-8 rounded-[32px] border border-slate-100 dark:border-slate-700/80 space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Clock className="text-primary-500" size={16} />
          <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest">
            {t("shifts.modal.preview_title", "Xem trước sơ đồ ca")}
          </span>
        </div>
        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider font-mono">
          24H TIMELINE
        </span>
      </div>

      {/* Visual Timeline Bar */}
      <div className="space-y-4">
        <TimelineBar
          allowEarlyOt={allowEarlyOt}
          breakEnd={breakEnd}
          breakStart={breakStart}
          endMin={endMin}
          isCrossMidnight={isCrossMidnight}
          isFixedBreak={isFixedBreak}
          maxEarlyOtMinutes={maxEarlyOt}
          shiftEnd={shiftEnd}
          shiftStart={shiftStart}
          startMin={startMin}
        />

        {/* Timeline Axis Labels */}
        <div className="flex justify-between px-1 text-[9px] font-extrabold text-slate-400 dark:text-slate-500">
          <span>00:00</span>
          <span>04:00</span>
          <span>08:00</span>
          <span>12:00</span>
          <span>16:00</span>
          <span>20:00</span>
          <span>24:00</span>
        </div>
      </div>

      <MetricsGrid shiftDurationMin={shiftDurationMin} t={t} workingHours={workingHours} />

      <ShiftDetailsList
        allowEarlyOt={allowEarlyOt}
        breakEnd={breakEnd}
        breakStart={breakStart}
        isAutoDeduct={isAutoDeduct}
        isBreakOvertime={isBreakOvertime}
        isFixedBreak={isFixedBreak}
        maxEarlyOtMinutes={maxEarlyOt}
        shiftBreak={shiftBreak}
        shiftEnd={shiftEnd}
        shiftGrace={shiftGrace}
        shiftName={shiftName}
        shiftStart={shiftStart}
        t={t}
      />
    </div>
  );
}
