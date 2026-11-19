import { Coffee } from "lucide-react";
import { memo } from "react";

import type { ShiftDetailsListProps } from "./types";

export const ShiftDetailsList = memo(function ShiftDetailsList({ intervals, t }: ShiftDetailsListProps) {
  return (
    <div className="bg-white dark:bg-slate-900 p-5 rounded-3xl border border-slate-100 dark:border-slate-800 shadow-sm space-y-3 text-xs">
      {intervals.map((interval, index) => (
        <div className="space-y-2" key={`${interval.startTime}-${interval.endTime}-${index}`}>
          {intervals.length > 1 ? (
            <div className="font-bold text-slate-500 dark:text-slate-400">
              {t("work_segments.intervals.period_number", "Khoảng làm việc {{number}}", { number: index + 1 })}
            </div>
          ) : null}
          {interval.intervalName ? (
            <div className="flex justify-between items-center pb-2.5 border-b border-slate-50 dark:border-slate-800/80">
              <span className="font-bold text-slate-500 dark:text-slate-400">
                {t("shifts.modal.shift_name_label", "Tên ca")}
              </span>
              <span className="font-black text-slate-700 dark:text-slate-200 uppercase">{interval.intervalName}</span>
            </div>
          ) : null}
          <div className="flex justify-between items-center pb-2.5 border-b border-slate-50 dark:border-slate-800/80">
            <span className="font-bold text-slate-500 dark:text-slate-400">
              {t("work_segments.intervals.work_time", "Giờ làm việc")}
            </span>
            <span className="font-bold font-mono text-slate-700 dark:text-slate-200">
              {interval.startTime || "--:--"} – {interval.endTime || "--:--"}
            </span>
          </div>
          <div className="flex justify-between items-center pb-2.5 border-b border-slate-50 dark:border-slate-800/80">
            <span className="font-bold text-slate-500 dark:text-slate-400 flex items-center gap-1.5">
              <Coffee aria-hidden="true" className="text-amber-500" size={12} />
              {t("shifts.modal.break_config", "Nghỉ giữa ca")}
            </span>
            <span className="font-bold text-slate-700 dark:text-slate-200">
              {interval.isFixedBreak
                ? `${interval.breakStart || "--:--"} - ${interval.breakEnd || "--:--"}`
                : `${interval.breakDuration} ${t("common.minutes", "phút")}`}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
});
