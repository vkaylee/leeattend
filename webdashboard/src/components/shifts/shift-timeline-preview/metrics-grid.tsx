import type { TFunction } from "i18next";

import { memo } from "react";

interface MetricsGridProps {
  expectedPunches: number;
  t: TFunction;
  totalBreakMinutes: number;
  totalDurationMinutes: number;
  totalWorkMinutes: number;
}

function formatHours(minutes: number, precision = 2): string {
  return (minutes / 60).toFixed(precision);
}

export const MetricsGrid = memo(function MetricsGrid({
  expectedPunches,
  totalBreakMinutes,
  totalDurationMinutes,
  totalWorkMinutes,
  t,
}: MetricsGridProps) {
  const metrics: { label: string; tone: string; unit: string; value: string }[] = [
    {
      label: t("shifts.modal.presence_time", "Thời gian hiện diện"),
      value: formatHours(totalDurationMinutes, 1),
      tone: "text-slate-700 dark:text-slate-200",
      unit: t("common.hours", "giờ"),
    },
    {
      label: t("shifts.modal.break_time", "Tổng giờ nghỉ"),
      value: formatHours(totalBreakMinutes),
      tone: "text-amber-600 dark:text-amber-400",
      unit: t("common.hours", "giờ"),
    },
    {
      label: t("shifts.modal.working_hours", "Công thực tế"),
      value: formatHours(totalWorkMinutes),
      tone: "text-primary-600 dark:text-primary-400",
      unit: t("common.hours", "giờ"),
    },
    {
      label: t("shifts.modal.expected_punches", "Lần chấm công dự kiến"),
      value: String(expectedPunches),
      tone: "text-slate-700 dark:text-slate-200",
      unit: t("common.count", "lần"),
    },
  ];

  return (
    <div className="grid grid-cols-2 gap-4">
      {metrics.map((metric) => (
        <div
          className="bg-white dark:bg-slate-900 p-4 rounded-2xl border border-slate-100 dark:border-slate-800 shadow-sm space-y-1"
          key={metric.label}
        >
          <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest block">
            {metric.label}
          </span>
          <div className={`flex items-baseline gap-1 ${metric.tone}`}>
            <span className="text-xl font-black font-mono">{metric.value}</span>
            <span className="text-[10px] font-bold">{metric.unit}</span>
          </div>
        </div>
      ))}
    </div>
  );
});
