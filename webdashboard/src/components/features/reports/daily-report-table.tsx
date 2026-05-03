import { RefreshCw, Bug, CheckCircle, ThumbsUp, ThumbsDown, AlertTriangle } from "lucide-react";
import React from "react";

import { DailyAttendanceReport, AttendanceAdjustmentRequest } from "@/lib/api";
import { cn } from "@/lib/utils";
import { formatUnitsToHours } from "@/lib/utils/attendance";

interface ReportStatusBadgeProps {
  onResolveUnscheduled?: (row: DailyAttendanceReport) => void;
  row: DailyAttendanceReport;
  status: string;
  t: (key: string) => string;
}

function ReportStatusBadge({ status, row, t, onResolveUnscheduled }: ReportStatusBadgeProps) {
  const styles: Record<string, string> = {
    present: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800",
    late: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800",
    absent: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800",
    off: "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800",
    unscheduled: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-200 dark:border-amber-800",
    holiday: "bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300 border-green-200 dark:border-green-800",
    holiday_paid: "bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300 border-green-200 dark:border-green-800",
    holiday_leave: "bg-amber-50 dark:bg-amber-950 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800",
    holiday_unpaid: "bg-rose-50 dark:bg-rose-950 text-rose-700 dark:text-rose-300 border-rose-200 dark:border-rose-800",
    holiday_work: "bg-orange-50 dark:bg-orange-950 text-orange-700 dark:text-orange-300 border-orange-200 dark:border-orange-800",
  };

  const labels: Record<string, string> = {
    present: t("reports.status.present"),
    late: t("reports.status.late"),
    absent: t("reports.status.absent"),
    off: t("reports.status.off"),
    unscheduled: t("reports.status.unscheduled"),
    holiday: "🏖️ Holiday",
    holiday_paid: "🏖️ Holiday (Paid)",
    holiday_leave: "⚡ Holiday (Leave)",
    holiday_unpaid: "🔴 Holiday (Unpaid)",
    holiday_work: "🔶 Holiday Work",
  };

  if (status === "unscheduled") {
    return (
      <button
        className={cn(
          "inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wide border shadow-sm cursor-pointer hover:bg-amber-100 dark:hover:bg-amber-900 hover:border-amber-300 dark:hover:border-amber-700 transition-colors active:scale-95",
          styles.unscheduled
        )}
        onClick={(e) => { e.stopPropagation(); onResolveUnscheduled?.(row); }}
        title={t("reports.status.unscheduled_tooltip")}
      >
        <AlertTriangle size={10} /> {labels.unscheduled}
      </button>
    );
  }

  return (
    <span
      className={cn(
        "px-1.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wide border shadow-sm",
        styles[status] ?? styles.off
      )}
      title={row.holiday_name ?? undefined}
    >
      {labels[status] ?? status}
    </span>
  );
}

interface ReportOtStatusBadgeProps {
  status?: string | null;
  t: (key: string) => string;
}

function ReportOtStatusBadge({ status, t }: ReportOtStatusBadgeProps) {
  if (!status || status === "none") return null;

  const styles: Record<string, string> = {
    pending_employee: "bg-yellow-50 dark:bg-yellow-950 text-yellow-600 dark:text-yellow-300 border-yellow-200 dark:border-yellow-800",
    pending_manager: "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-200 dark:border-orange-800",
    approved: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800",
    rejected: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-200 dark:border-rose-800",
  };

  const labels: Record<string, string> = {
    pending_employee: t("reports.ot_status.pending_employee"),
    pending_manager: t("reports.ot_status.pending_manager"),
    approved: t("reports.ot_status.approved"),
    rejected: t("reports.ot_status.rejected"),
  };

  return (
    <span className={cn("px-1.5 py-0.5 rounded border text-[9px] font-bold uppercase tracking-wide", styles[status] ?? "")}>
      {labels[status] ?? status}
    </span>
  );
}

interface DailyReportTableRowProps {
  adjustmentMap: Record<string, AttendanceAdjustmentRequest>;
  confirmingRow: string | null;
  expandedDebugRows: Set<string>;
  formatDate: (date: string) => string;
  formatTime: (time: string | null) => string;
  hideEmployeeName: boolean;
  isDebugMode: boolean;
  onApproveOt?: (row: DailyAttendanceReport, action: "approved" | "rejected") => void;
  onConfirmClick: (row: DailyAttendanceReport, existing?: AttendanceAdjustmentRequest) => void;
  onReprocessClick: (row: DailyAttendanceReport) => void;
  onResolveUnscheduled?: (row: DailyAttendanceReport) => void;
  onToggleExpandRow: (rowKey: string) => void;
  reprocessingRow: string | null;
  row: DailyAttendanceReport;
  t: (key: string) => string;
  workUnitMinutes: number;
}

function DailyReportTableRow({
  row,
  adjustmentMap,
  confirmingRow,
  expandedDebugRows,
  formatDate,
  formatTime,
  hideEmployeeName,
  isDebugMode,
  onApproveOt,
  onConfirmClick,
  onReprocessClick,
  onResolveUnscheduled,
  onToggleExpandRow,
  reprocessingRow,
  t,
  workUnitMinutes,
}: DailyReportTableRowProps) {
  const rowKey = `${row.employee_id ?? ""}-${row.date ?? ""}`;
  const isReprocessing = reprocessingRow === rowKey;
  const isConfirming = confirmingRow === rowKey;
  const isExpanded = expandedDebugRows.has(rowKey);
  const existingAdjustment = adjustmentMap[rowKey];

  return (
    <React.Fragment>
      <tr
        className={cn(
          "transition-all group",
          isExpanded ? "bg-amber-50 dark:bg-amber-950/20" : "hover:bg-slate-50 dark:hover:bg-slate-800/50"
        )}
      >
        <td className="px-4 py-2.5 font-mono text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase whitespace-nowrap sticky left-0 z-10 bg-white group-hover:bg-slate-50 dark:bg-slate-900 dark:group-hover:bg-slate-800/50 transition-colors">
          {formatDate(row.date)}
        </td>

        {!hideEmployeeName && (
          <td className="px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none whitespace-nowrap sticky left-[80px] sm:left-[100px] z-10 bg-white group-hover:bg-slate-50 dark:bg-slate-900 dark:group-hover:bg-slate-800/50 transition-colors">
            {row.employee_name}
          </td>
        )}

        <td className="px-4 py-2.5">
          {row.scheduled_start ? (
            <div className="flex flex-col">
              <span className="font-bold text-slate-700 dark:text-slate-200 text-xs uppercase tracking-tighter shadow-sm bg-white dark:bg-slate-900 px-2 py-0.5 rounded-lg border border-slate-100 dark:border-slate-800 w-fit">
                {row.scheduled_start.substring(0, 5)} — {row.scheduled_end?.substring(0, 5)}
              </span>
              {row.break_duration_minutes !== null && (
                <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-1.5 opacity-70 italic">
                  {row.break_start
                    ? `${t("common.break")}: ${row.break_start.substring(0, 5)}-${row.break_end?.substring(0, 5)}`
                    : `${t("common.break")}: ${row.break_duration_minutes.toString()}m`}
                </span>
              )}
            </div>
          ) : (
            <span className="text-[9px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest italic opacity-50">
              {t("reports.no_shift")}
            </span>
          )}
        </td>

        <td className="px-4 py-2.5 text-center font-mono font-bold text-xs text-slate-700 dark:text-slate-200 tracking-tighter">
          <div className="shadow-inner bg-slate-50 dark:bg-slate-800/50 rounded-md py-1 px-2 inline-block">
            {formatTime(row.actual_punch_in)}
          </div>
        </td>

        <td className="px-4 py-2.5 text-center font-mono font-bold text-xs text-slate-700 dark:text-slate-200 tracking-tighter">
          <div className="shadow-inner bg-slate-50 dark:bg-slate-800/50 rounded-md py-1 px-2 inline-block">
            {formatTime(row.actual_punch_out)}
          </div>
        </td>

        <td className="px-4 py-2.5 text-right" title={`${row.std_work_units.toString()} units`}>
          <div className="font-mono font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter leading-none">
            {formatUnitsToHours(row.std_work_units, workUnitMinutes)}h
          </div>
        </td>

        <td className="px-4 py-2.5 text-right" title={row.ot_work_units > 0 ? `${row.ot_work_units.toString()} units` : undefined}>
          {row.ot_work_units > 0 ? (
            <div className="flex flex-col items-end gap-1">
              <div>
                <span className="font-mono font-bold text-orange-600 dark:text-orange-400 text-base tracking-tighter leading-none">
                  {formatUnitsToHours(row.ot_work_units, workUnitMinutes)}h
                </span>
              </div>
              <ReportOtStatusBadge status={row.ot_claim_status} t={t} />
              {row.ot_claim_status === "pending_manager" && !!onApproveOt && (
                <div className="flex gap-1 mt-1">
                  <button
                    className="flex items-center gap-0.5 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider bg-emerald-500 text-white hover:bg-emerald-600 transition-colors shadow-sm active:scale-95"
                    onClick={(e) => { e.stopPropagation(); onApproveOt(row, "approved"); }}
                    title={t("reports.ot_approve.approve")}
                  >
                    <ThumbsUp size={10} />
                    {t("reports.ot_approve.approve")}
                  </button>
                  <button
                    className="flex items-center gap-0.5 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider bg-rose-500 text-white hover:bg-rose-600 transition-colors shadow-sm active:scale-95"
                    onClick={(e) => { e.stopPropagation(); onApproveOt(row, "rejected"); }}
                    title={t("reports.ot_approve.reject")}
                  >
                    <ThumbsDown size={10} />
                    {t("reports.ot_approve.reject")}
                  </button>
                </div>
              )}
            </div>
          ) : (
            <span className="text-slate-200 dark:text-slate-700 font-bold">—</span>
          )}
        </td>

        <td className="px-4 py-2.5 text-center">
          <ReportStatusBadge onResolveUnscheduled={onResolveUnscheduled} row={row} status={row.status} t={t} />
        </td>

        {!!isDebugMode && (
          <td className="px-4 py-2.5 text-center">
            <button
              className={cn(
                "p-1.5 rounded-lg transition-all shadow-sm border active:scale-95",
                isExpanded
                  ? "bg-amber-100 dark:bg-amber-900 text-amber-700 dark:text-amber-300 border-amber-200"
                  : "bg-white dark:bg-slate-900 text-amber-600 dark:text-amber-300 border-slate-100 dark:border-slate-800 hover:bg-amber-50 dark:bg-amber-950"
              )}
              onClick={() => { onToggleExpandRow(rowKey); }}
            >
              <Bug size={16} />
            </button>
          </td>
        )}

        <td className="px-4 py-2.5 text-right">
          <div className="flex items-center justify-end gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity focus-within:opacity-100">
            <button
              className={cn(
                "flex items-center justify-center p-1.5 rounded-lg transition-all shadow-sm border active:scale-95",
                existingAdjustment
                  ? "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-200 dark:border-orange-800 hover:bg-orange-100 dark:hover:bg-orange-900"
                  : "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800 hover:bg-emerald-100 dark:hover:bg-emerald-900",
                isConfirming && "opacity-70 cursor-wait"
              )}
              disabled={isConfirming}
              onClick={() => { onConfirmClick(row, existingAdjustment); }}
              title={existingAdjustment ? t("reports.confirm.pending_badge") : t("common.attendance_confirm")}
            >
              {isConfirming
                ? <RefreshCw className="animate-spin" size={14} />
                : <CheckCircle size={14} />
              }
            </button>

            <button
              className="p-1.5 bg-white dark:bg-slate-900 text-blue-600 dark:text-blue-300 hover:bg-blue-50 dark:hover:bg-slate-800 rounded-lg shadow-sm border border-slate-200 dark:border-slate-700 transition-all active:scale-95 disabled:opacity-50"
              disabled={isReprocessing}
              onClick={() => { onReprocessClick(row); }}
              title={t("reports.reprocess.tooltip")}
            >
              <RefreshCw className={cn(isReprocessing && "animate-spin")} size={18} />
            </button>
          </div>
        </td>
      </tr>

      {!!isDebugMode && !!isExpanded && (
        <tr className="bg-slate-900/5 animate-in slide-in-from-top-2 duration-500">
          <td className="px-10 py-10" colSpan={10}>
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
                  row.generation_metadata ?? { message: "No metadata available" },
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
}

interface DailyReportTableProps {
  adjustmentMap: Record<string, AttendanceAdjustmentRequest>;
  confirmingRow: string | null;
  data: DailyAttendanceReport[];
  expandedDebugRows: Set<string>;
  formatDate: (date: string) => string;
  formatTime: (time: string | null) => string;
  hideEmployeeName?: boolean;
  isDebugMode: boolean;
  isLoading: boolean;
  onApproveOt?: (row: DailyAttendanceReport, action: "approved" | "rejected") => void;
  onConfirmClick: (row: DailyAttendanceReport, existing?: AttendanceAdjustmentRequest) => void;
  onReprocessClick: (row: DailyAttendanceReport) => void;
  onResolveUnscheduled?: (row: DailyAttendanceReport) => void;
  onToggleExpandRow: (rowKey: string) => void;
  reprocessingRow: string | null;
  t: (key: string) => string;
  workUnitMinutes: number;
}

export default function DailyReportTable({
  data,
  isLoading,
  workUnitMinutes,
  isDebugMode,
  expandedDebugRows,
  reprocessingRow,
  confirmingRow,
  adjustmentMap,
  t,
  formatDate,
  formatTime,
  onToggleExpandRow,
  onReprocessClick,
  onConfirmClick,
  onApproveOt,
  onResolveUnscheduled,
  hideEmployeeName = false,
}: DailyReportTableProps) {
  return (
    <div className="overflow-auto flex-1 min-h-0">
      <table className="w-full text-left min-w-[1200px] border-collapse relative">
        <thead>
          <tr className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky top-0 z-20">
            <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest sticky left-0 z-20 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm">
              {t("common.date")}
            </th>
            {!hideEmployeeName && (
              <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest sticky left-[80px] sm:left-[100px] z-20 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm">
                {t("common.employee")}
              </th>
            )}
            <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
              {t("common.shift")}
            </th>
            <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">
              {t("common.in")}
            </th>
            <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">
              {t("common.out")}
            </th>
            <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">
              {t("reports.table.std_hours")}
            </th>
            <th className="px-4 py-2.5 text-[10px] font-bold text-orange-500 dark:text-orange-400 uppercase tracking-widest text-right">
              {t("reports.table.ot_hours")}
            </th>
            <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">
              {t("common.status")}
            </th>
            {!!isDebugMode && (
              <th className="px-4 py-2.5 text-[10px] font-bold text-amber-600 dark:text-amber-300 uppercase tracking-widest text-center">
                DEBUG
              </th>
            )}
            <th className="px-4 py-2.5"></th>
          </tr>
        </thead>

        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
          {isLoading ? (
            <tr>
              <td
                className="px-10 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse"
                colSpan={isDebugMode ? (hideEmployeeName ? 9 : 10) : (hideEmployeeName ? 8 : 9)}
              >
                {t("common.loading")}
              </td>
            </tr>
          ) : data.length === 0 ? (
            <tr>
              <td
                className="px-10 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50"
                colSpan={isDebugMode ? (hideEmployeeName ? 9 : 10) : (hideEmployeeName ? 8 : 9)}
              >
                {t("reports.messages.no_data")}
              </td>
            </tr>
          ) : (
            data.map((row: DailyAttendanceReport) => (
              <DailyReportTableRow
                key={`${row.employee_id ?? ""}-${row.date ?? ""}`}
                adjustmentMap={adjustmentMap}
                confirmingRow={confirmingRow}
                expandedDebugRows={expandedDebugRows}
                formatDate={formatDate}
                formatTime={formatTime}
                hideEmployeeName={hideEmployeeName}
                isDebugMode={isDebugMode}
                onApproveOt={onApproveOt}
                onConfirmClick={onConfirmClick}
                onReprocessClick={onReprocessClick}
                onResolveUnscheduled={onResolveUnscheduled}
                onToggleExpandRow={onToggleExpandRow}
                reprocessingRow={reprocessingRow}
                row={row}
                t={t}
                workUnitMinutes={workUnitMinutes}
              />
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
