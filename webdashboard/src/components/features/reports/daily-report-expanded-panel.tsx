import { RefreshCw, Bug } from "lucide-react";
import React from "react";

import { Typography } from "@/components/ui/typography";
import { DailyAttendanceReport } from "@/lib/api";

import { DailyReportTimeline } from "./daily-report-timeline";

/**
 * Expanded detail panel for a daily report row: attendance timeline plus, in debug
 * mode, the raw generation metadata.
 *
 * Renders panel *content* only — SmartTable supplies the wrapping
 * `<tr><td colSpan>`, so the colSpan always matches the real visible-column
 * count instead of a hand-maintained constant.
 */
export function DailyReportExpandedPanel({
  row,
  formatTime,
  t,
  isDebugMode,
}: {
  formatTime: (time: string | null) => string;
  isDebugMode: boolean;
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

        {isDebugMode ? (
          <div className="mt-8 border-t border-slate-100 dark:border-slate-800 pt-6">
            <div className="flex items-center gap-4 mb-4">
              <div className="w-8 h-8 rounded-lg bg-amber-50 dark:bg-amber-950/10 flex items-center justify-center text-amber-500 shadow-inner border border-amber-500/20">
                <Bug size={14} />
              </div>
              <div>
                <Typography className="text-[10px] font-bold text-amber-500 uppercase tracking-widest" variant="h4">
                  Audit Trail Analysis (JSON debug)
                </Typography>
              </div>
            </div>
            <pre className="text-[10px] font-mono text-emerald-400 bg-slate-950/90 dark:bg-slate-950 rounded-2xl leading-relaxed overflow-x-auto p-4 max-h-[300px]">
              {JSON.stringify(
                row.generation_metadata ?? { message: "No metadata available" },
                null,
                2
              )}
            </pre>
          </div>
        ) : null}
      </div>
    </div>
  );
}
