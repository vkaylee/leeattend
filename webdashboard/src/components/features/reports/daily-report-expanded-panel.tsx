import { RefreshCw } from "lucide-react";
import React from "react";

import { Typography } from "@/components/ui/typography";
import { DailyAttendanceReport } from "@/lib/api";

import { DailyReportTimeline } from "./daily-report-timeline";

/**
 * Expanded detail panel for a daily report row: attendance timeline.
 *
 * Renders panel *content* only — SmartTable supplies the wrapping
 * `<tr><td colSpan>`, so the colSpan always matches the real visible-column
 * count instead of a hand-maintained constant.
 */
export function DailyReportExpandedPanel({
  row,
  formatTime,
  t,
}: {
  formatTime: (time: string | null) => string;
  row: DailyAttendanceReport;
  t: (key: string) => string;
}) {
  return (
    <div className="px-10 py-6 animate-in slide-in-from-top-2 duration-300">
      <div className="bg-white dark:bg-slate-900/90 rounded-[2rem] p-6 shadow-premium border border-slate-100 dark:border-slate-800 overflow-hidden relative">
        <div className="flex items-center gap-3 mb-6 border-b border-slate-100 dark:border-slate-800 pb-4">
          <div className="w-8 h-8 rounded-lg bg-emerald-50 dark:bg-emerald-950/20 flex items-center justify-center text-emerald-500 border border-emerald-500/10">
            <RefreshCw className="animate-pulse" size={14} />
          </div>
          <div>
            <Typography className="text-[10px] font-bold text-emerald-600 dark:text-emerald-400 uppercase tracking-widest" variant="h4">
              {t("reports.timeline.title") || "Attendance Timeline Analysis"}
            </Typography>
            <Typography className="text-[9px] text-slate-400 uppercase tracking-widest mt-0.5 font-bold" variant="p">
              Visualizing Work Sessions &amp; Scheduled Breaks
            </Typography>
          </div>
        </div>

        {/* Visual Timeline Component */}
        <DailyReportTimeline formatTime={formatTime} row={row} t={t} />

      </div>
    </div>
  );
}
