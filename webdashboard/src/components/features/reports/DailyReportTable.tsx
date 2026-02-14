import React from "react";
import { RefreshCw, Bug } from "lucide-react";
import { DailyAttendanceReport } from "@/lib/api";
import { formatUnitsToHours } from "@/lib/utils/attendance";
import { cn } from "@/lib/utils";

interface DailyReportTableProps {
  data: DailyAttendanceReport[];
  isLoading: boolean;
  workUnitMinutes: number;
  isDebugMode: boolean;
  expandedDebugRows: Set<string>;
  reprocessingRow: string | null;
  t: (key: string) => string;
  formatDate: (date: string) => string;
  formatTime: (time: string | null) => string;
  onToggleExpandRow: (rowKey: string) => void;
  onReprocessClick: (row: DailyAttendanceReport) => void;
}

export default function DailyReportTable({
  data,
  isLoading,
  workUnitMinutes,
  isDebugMode,
  expandedDebugRows,
  reprocessingRow,
  t,
  formatDate,
  formatTime,
  onToggleExpandRow,
  onReprocessClick,
}: DailyReportTableProps) {
  const getStatusBadge = (status: string) => {
    const styles: Record<string, string> = {
      present: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800",
      late: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800",
      absent: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800",
      off: "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800",
    };

    const labels: Record<string, string> = {
      present: t("reports.status.present"),
      late: t("reports.status.late"),
      absent: t("reports.status.absent"),
      off: t("reports.status.off"),
    };

    return (
      <span
        className={cn(
          "px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
          styles[status] || styles.off
        )}
      >
        {labels[status] || status}
      </span>
    );
  };

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left min-w-[1200px] border-collapse">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
            <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
              {t("common.date")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
              {t("common.employee")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
              {t("common.shift")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
              {t("common.in")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
              {t("common.out")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-right">
              {t("reports.table.hours")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
              {t("common.status")}
            </th>
            {isDebugMode && (
              <th className="px-8 py-6 text-[10px] font-bold text-amber-600 dark:text-amber-300 uppercase tracking-[0.25em] text-center">
                DEBUG
              </th>
            )}
            <th className="px-8 py-6"></th>
          </tr>
        </thead>

        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
          {isLoading ? (
            <tr>
              <td
                colSpan={isDebugMode ? 9 : 8}
                className="px-10 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse"
              >
                {t("common.loading")}
              </td>
            </tr>
          ) : data.length === 0 ? (
            <tr>
              <td
                colSpan={isDebugMode ? 9 : 8}
                className="px-10 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50"
              >
                {t("reports.messages.no_data")}
              </td>
            </tr>
          ) : (
            data.map((row: DailyAttendanceReport, idx: number) => {
              const rowKey = `${row.employee_id}-${row.date}`;
              const isReprocessing = reprocessingRow === rowKey;
              const isExpanded = expandedDebugRows.has(rowKey);

              return (
                <React.Fragment key={`${rowKey}-${idx}`}>
                  <tr
                    className={cn(
                      "transition-all group",
                      isExpanded ? "bg-amber-50 dark:bg-amber-950/20" : "hover:bg-slate-50 dark:hover:bg-slate-800/50"
                    )}
                  >
                    <td className="px-10 py-6 font-mono text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase">
                      {formatDate(row.date)}
                    </td>

                    <td className="px-8 py-6 font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">
                      {row.employee_name}
                    </td>

                    <td className="px-8 py-6">
                      {row.scheduled_start ? (
                        <div className="flex flex-col">
                          <span className="font-bold text-slate-700 dark:text-slate-200 text-xs uppercase tracking-tighter shadow-sm bg-white dark:bg-slate-900 px-2 py-0.5 rounded-lg border border-slate-100 dark:border-slate-800 w-fit">
                            {row.scheduled_start.substring(0, 5)} — {row.scheduled_end?.substring(0, 5)}
                          </span>
                          {row.break_duration_minutes !== null && (
                            <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-1.5 opacity-70 italic">
                              {row.break_start
                                ? `${t("common.break")}: ${row.break_start.substring(0, 5)}-${row.break_end?.substring(0, 5)}`
                                : `${t("common.break")}: ${row.break_duration_minutes}m`}
                            </span>
                          )}
                        </div>
                      ) : (
                        <span className="text-[9px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest italic opacity-50">
                          {t("reports.no_shift")}
                        </span>
                      )}
                    </td>

                    <td className="px-8 py-6 text-center font-mono font-bold text-xs text-slate-700 dark:text-slate-200 tracking-tighter shadow-inner bg-slate-50 dark:bg-slate-800/50 rounded-xl m-2">
                      {formatTime(row.actual_punch_in)}
                    </td>

                    <td className="px-8 py-6 text-center font-mono font-bold text-xs text-slate-700 dark:text-slate-200 tracking-tighter shadow-inner bg-slate-50 dark:bg-slate-800/50 rounded-xl m-2">
                      {formatTime(row.actual_punch_out)}
                    </td>

                    <td className="px-8 py-6 text-right">
                      <div className="font-mono font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter leading-none">
                        {formatUnitsToHours(row.std_work_units + row.ot_work_units, workUnitMinutes)}h
                      </div>
                      <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-60">
                        {row.std_work_units + row.ot_work_units}U
                      </span>
                    </td>

                    <td className="px-8 py-6 text-center">{getStatusBadge(row.status)}</td>

                    {isDebugMode && (
                      <td className="px-8 py-6 text-center">
                        <button
                          onClick={() => onToggleExpandRow(rowKey)}
                          className={cn(
                            "p-2 rounded-xl transition-all shadow-md border active:scale-90",
                            isExpanded
                              ? "bg-amber-100 dark:bg-amber-900 text-amber-700 dark:text-amber-300 border-amber-200"
                              : "bg-white dark:bg-slate-900 text-amber-600 dark:text-amber-300 border-slate-100 dark:border-slate-800 hover:bg-amber-50 dark:bg-amber-950"
                          )}
                        >
                          <Bug size={16} />
                        </button>
                      </td>
                    )}

                    <td className="px-10 py-6 text-right">
                      <button
                        onClick={() => onReprocessClick(row)}
                        disabled={isReprocessing}
                        className="p-3 bg-white dark:bg-slate-900 text-blue-600 dark:text-blue-300 hover:text-blue-700 dark:text-blue-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90 opacity-0 group-hover:opacity-100 disabled:opacity-50"
                        title={t("reports.reprocess.tooltip")}
                      >
                        <RefreshCw size={18} className={cn(isReprocessing && "animate-spin")} />
                      </button>
                    </td>
                  </tr>

                  {isDebugMode && isExpanded && (
                    <tr className="bg-slate-900/5 animate-in slide-in-from-top-2 duration-500">
                      <td colSpan={9} className="px-10 py-10">
                        <div className="bg-slate-900 rounded-[3rem] p-10 shadow-2xl border-2 border-amber-200/20 overflow-hidden relative">
                          <div className="flex items-center gap-4 mb-8 border-b border-white/10 pb-6 relative z-10">
                            <div className="w-10 h-10 rounded-xl bg-amber-50 dark:bg-amber-950/10 flex items-center justify-center text-amber-400 shadow-inner border border-amber-500/20">
                              <Bug size={20} />
                            </div>
                            <div>
                              <h4 className="text-[10px] font-bold text-amber-400 uppercase tracking-[0.3em]">
                                Audit Trail Analysis
                              </h4>
                              <p className="text-[9px] text-white/40 uppercase tracking-widest mt-1 font-bold">
                                BSON Traceability Metadata — Real-time Evaluator Logs
                              </p>
                            </div>
                          </div>
                          <pre className="text-[11px] font-mono text-emerald-400 leading-relaxed overflow-x-auto custom-scrollbar relative z-10 p-2 selection:bg-emerald-50 dark:bg-emerald-950/20 max-h-[400px]">
                            {JSON.stringify(
                              row.generation_metadata || { message: "No metadata available" },
                              null,
                              2
                            )}
                          </pre>
                          <Bug className="absolute -right-16 -bottom-16 w-64 h-64 text-amber-500 opacity-5 -rotate-12 pointer-events-none" />
                        </div>
                      </td>
                    </tr>
                  )}
                </React.Fragment>
              );
            })
          )}
        </tbody>
      </table>
    </div>
  );
}
