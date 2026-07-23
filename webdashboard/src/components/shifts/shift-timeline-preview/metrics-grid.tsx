import { memo } from "react";
import type { TFunction } from "i18next";

interface MetricsGridProps {
  shiftDurationMin: number;
  workingHours: string;
  t: TFunction;
}

export const MetricsGrid = memo(function MetricsGrid({
  shiftDurationMin,
  workingHours,
  t,
}: MetricsGridProps) {
  return (
    <div className="grid grid-cols-2 gap-4">
      {/* Metric 1: Total shift hours */}
      <div className="bg-white dark:bg-slate-900 p-4 rounded-2xl border border-slate-100 dark:border-slate-800 shadow-sm space-y-1">
        <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest block">
          {t("shifts.modal.shift_duration", "Tổng thời gian ca")}
        </span>
        <div className="flex items-baseline gap-1 text-slate-700 dark:text-slate-200">
          <span className="text-xl font-black font-mono">
            {(shiftDurationMin / 60).toFixed(1)}
          </span>
          <span className="text-[10px] font-bold text-slate-500">{t("common.hours", "giờ")}</span>
        </div>
      </div>

      {/* Metric 2: Net working hours */}
      <div className="bg-white dark:bg-slate-900 p-4 rounded-2xl border border-slate-100 dark:border-slate-800 shadow-sm space-y-1">
        <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest block">
          {t("shifts.modal.working_hours", "Công thực tế")}
        </span>
        <div className="flex items-baseline gap-1 text-primary-600 dark:text-primary-400">
          <span className="text-xl font-black font-mono">
            {workingHours}
          </span>
          <span className="text-[10px] font-bold text-primary-500">{t("common.hours", "giờ")}</span>
        </div>
      </div>
    </div>
  );
});
