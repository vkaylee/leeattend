import { Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { Shift } from "@/lib/api";
import { getShiftCardMetrics, type ShiftInterval } from "@/lib/shift-card-metrics";

import type { ShiftTimelinePreviewProps } from "./shift-timeline-preview/types";

import { MetricsGrid } from "./shift-timeline-preview/metrics-grid";
import { ShiftDetailsList } from "./shift-timeline-preview/shift-details-list";

export { TimelineBar } from "./shift-timeline-preview/timeline-bar";
export { ShiftDetailsList } from "./shift-timeline-preview/shift-details-list";
export { MetricsGrid } from "./shift-timeline-preview/metrics-grid";
export type { IntervalPreviewData, ShiftTimelinePreviewProps } from "./shift-timeline-preview/types";

function toMetricsInput(interval: ShiftTimelinePreviewProps["intervals"][number]): ShiftInterval {
  return {
    break_duration_minutes: interval.breakDuration,
    break_end: interval.isFixedBreak ? interval.breakEnd || null : null,
    break_start: interval.isFixedBreak ? interval.breakStart || null : null,
    end_time: interval.endTime,
    id: interval.intervalName || "draft",
    interval_name: interval.intervalName || null,
    sort_order: 0,
    start_time: interval.startTime,
    version_id: "draft",
  } as ShiftInterval;
}

export function ShiftTimelinePreview({ intervals }: ShiftTimelinePreviewProps) {
  const { t } = useTranslation();
  const first = intervals[0];

  if (!first) {
    return (
      <div className="bg-slate-50 dark:bg-slate-800 p-8 rounded-[32px] border border-slate-100 dark:border-slate-700/80 text-center text-slate-400">
        {t("work_segments.intervals.no_intervals", "Chưa có khoảng làm việc")}
      </div>
    );
  }

  const metrics = getShiftCardMetrics({
    id: "draft",
    intervals: intervals.map(toMetricsInput),
    name: "draft",
  } as unknown as Shift);

  return (
    <div className="bg-slate-50 dark:bg-slate-800 p-8 rounded-[32px] border border-slate-100 dark:border-slate-700/80 space-y-6">
      <div className="flex items-center gap-2">
        <Clock aria-hidden="true" className="text-primary-500" size={16} />
        <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest">
          {t("shifts.modal.preview_title", "Tóm tắt ca làm việc")}
        </span>
      </div>

      <MetricsGrid
        expectedPunches={metrics.expectedPunches}
        t={t}
        totalBreakMinutes={metrics.totalBreakMinutes}
        totalDurationMinutes={metrics.totalDurationMinutes}
        totalWorkMinutes={metrics.totalWorkMinutes}
      />

      <ShiftDetailsList intervals={intervals} t={t} />
    </div>
  );
}
