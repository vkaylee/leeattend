import { RefreshCw, Bug, CheckCircle, ThumbsUp, ThumbsDown, ChevronRight, ChevronDown } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { DailyAttendanceReport, AttendanceAdjustmentRequest, ScheduleTrace } from "@/lib/api";
import { cn } from "@/lib/utils";
import { formatHours, getUnitTypeI18nKey, buildWorkUnitsTooltip } from "@/lib/utils/attendance";

import { ScheduleTracePopover } from "../../shifts/schedule-trace-popover";
import { ReportOtStatusBadge } from "./report-ot-status-badge";
import { ReportStatusBadge } from "./report-status-badge";
import { DailyReportTimeline } from "./daily-report-timeline";

function DailyReportTableRowExpanded({
  row,
  formatTime,
  t,
  isDebugMode,
}: {
  row: DailyAttendanceReport;
  formatTime: (time: string | null) => string;
  t: (key: string) => string;
  isDebugMode: boolean;
}) {
  return (
    <tr className="bg-slate-500/5 dark:bg-slate-950/10 animate-in slide-in-from-top-2 duration-300">
      <td className="px-10 py-6" colSpan={11}>
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
                Visualizing Work Sessions & Scheduled Breaks
              </Typography>
            </div>
          </div>
          
          {/* Visual Timeline Component */}
          <DailyReportTimeline row={row} formatTime={formatTime} t={t} />

          {isDebugMode && (
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
          )}
        </div>
      </td>
    </tr>
  );
}

function getDerivedSource(appliedRule?: ScheduleTrace): string {
  if (!appliedRule) return "system";
  switch (appliedRule.priority) {
    case 100:
      return "roster";
    case 80:
      return "pattern";
    case 60:
      return "rules";
    case 40:
      return "department_pattern";
    case 20:
      return "fixed";
    case 10:
      return "department_fixed";
    default:
      return "system";
  }
}

function ShiftCell({ row, t }: { row: DailyAttendanceReport; t: (k: string) => string }) {
  if (!row.scheduled_start) {
    return (
      <span className="text-[9px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest italic opacity-50">
        {t("reports.no_shift")}
      </span>
    );
  }

  const metadata = row.generation_metadata as { schedule_traces?: ScheduleTrace[] } | undefined;
  const traces = metadata?.schedule_traces ?? [];
  const hasTraces = traces.length > 0;

  const appliedRule = traces.find((tr) => tr.status === "applied");
  const derivedSource = getDerivedSource(appliedRule);

  return (
    <div className="flex flex-col">
      <div className="relative w-fit flex items-center">
        <span className={cn(
          "font-bold text-slate-700 dark:text-slate-200 text-xs uppercase tracking-tighter shadow-sm bg-white dark:bg-slate-900 py-0.5 rounded-lg border border-slate-100 dark:border-slate-800",
          hasTraces ? "pl-2 pr-6.5" : "px-2"
        )}>
          {row.scheduled_start.substring(0, 5)} — {row.scheduled_end?.substring(0, 5)}
        </span>
        {hasTraces ? (
          <div className="absolute right-1 top-1/2 -translate-y-1/2 z-10 flex items-center justify-center">
            <ScheduleTracePopover
              endTime={row.scheduled_end}
              shiftName={row.shift_name}
              shiftSymbol={row.shift_symbol}
              source={derivedSource}
              startTime={row.scheduled_start}
              traces={traces}
            />
          </div>
        ) : null}
      </div>
      {row.break_duration_minutes !== null ? (
        <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-1.5 opacity-70 italic">
          {row.break_start
            ? `${t("common.break")}: ${row.break_start.substring(0, 5)}-${row.break_end?.substring(0, 5)}`
            : `${t("common.break")}: ${row.break_duration_minutes.toString()}m`}
        </span>
      ) : null}
    </div>
  );
}

function OtCell({
  onApproveOt,
  row,
  t,
}: {
  onApproveOt?: (row: DailyAttendanceReport, action: "approved" | "rejected") => void;
  row: DailyAttendanceReport;
  t: (k: string) => string;
}) {
  if (row.ot_work_units <= 0) {
    return <span className="text-slate-200 dark:text-slate-700 font-bold">—</span>;
  }

  const unitLabel = t(getUnitTypeI18nKey(row.unit_type));
  const otTooltip = buildWorkUnitsTooltip(row.ot_work_units, unitLabel, row.unit_coefficient_hours, row.computed_ot_hours);

  return (
    <div className="flex flex-col items-end gap-1">
      <div
        className="cursor-help"
        title={otTooltip}
      >
        <span className="font-mono font-bold text-orange-600 dark:text-orange-400 text-base tracking-tighter leading-none">
          {formatHours(row.computed_ot_hours)}<span className="text-[10px] opacity-70 ml-0.5">h</span>
        </span>
      </div>
      <ReportOtStatusBadge status={row.ot_claim_status} t={t} />
      {row.ot_claim_status === "pending_manager" && onApproveOt ? (
        <div className="flex gap-1 mt-1">
          <Button
            className="flex items-center gap-0.5 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider bg-emerald-500 text-white hover:bg-emerald-600 transition-colors shadow-sm h-auto"
            onClick={(e) => { e.stopPropagation(); onApproveOt(row, "approved"); }}
            title={t("reports.ot_approve.approve")}
            variant="ghost"
          >
            <ThumbsUp size={10} />
            {t("reports.ot_approve.approve")}
          </Button>
          <Button
            className="flex items-center gap-0.5 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider bg-rose-500 text-white hover:bg-rose-600 transition-colors shadow-sm h-auto"
            onClick={(e) => { e.stopPropagation(); onApproveOt(row, "rejected"); }}
            title={t("reports.ot_approve.reject")}
            variant="ghost"
          >
            <ThumbsDown size={10} />
            {t("reports.ot_approve.reject")}
          </Button>
        </div>
      ) : null}
    </div>
  );
}

function RowActions({
  existingAdjustment,
  isConfirming,
  isReprocessing,
  onConfirmClick,
  onReprocessClick,
  row,
  t
}: {
  existingAdjustment: AttendanceAdjustmentRequest | undefined;
  isConfirming: boolean;
  isReprocessing: boolean;
  onConfirmClick: (row: DailyAttendanceReport, existing?: AttendanceAdjustmentRequest) => void;
  onReprocessClick: (row: DailyAttendanceReport) => void;
  row: DailyAttendanceReport;
  t: (key: string) => string;
}) {
  return (
    <div className="flex items-center justify-end gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity focus-within:opacity-100">
      <Button
        className={cn("flex items-center justify-center p-1.5 rounded-lg transition-all shadow-sm border h-auto", existingAdjustment !== undefined ? "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-200 dark:border-orange-800 hover:bg-orange-100 dark:hover:bg-orange-900" : "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800 hover:bg-emerald-100 dark:hover:bg-emerald-900", isConfirming && "opacity-70 cursor-wait")}
        disabled={isConfirming}
        onClick={() => { onConfirmClick(row, existingAdjustment); }}
        title={existingAdjustment !== undefined ? t("reports.confirm.pending_badge") : t("common.attendance_confirm")}
        variant="ghost"
      >
        {isConfirming ? <RefreshCw className="animate-spin" size={14} /> : <CheckCircle size={14} />}
      </Button>
      <Button
        className="p-1.5 bg-white dark:bg-slate-900 text-blue-600 dark:text-blue-300 hover:bg-blue-50 dark:hover:bg-slate-800 rounded-lg shadow-sm border border-slate-200 dark:border-slate-700 transition-all disabled:opacity-50 h-auto"
        disabled={isReprocessing}
        onClick={() => { onReprocessClick(row); }}
        title={t("reports.reprocess.tooltip")}
        variant="ghost"
      >
        <RefreshCw className={cn(isReprocessing && "animate-spin")} size={18} />
      </Button>
    </div>
  );
}

export interface DailyReportTableRowProps {
  adjustmentMap: Partial<Record<string, AttendanceAdjustmentRequest>>;
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
}

export function DailyReportTableRow({
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
  row,
  t,
}: DailyReportTableRowProps) {
  const rowKey = `${row.employee_id}-${row.date}`;
  const isReprocessing = reprocessingRow === rowKey;
  const isConfirming = confirmingRow === rowKey;
  const isExpanded = expandedDebugRows.has(rowKey);
  const existingAdjustment = adjustmentMap[rowKey];
  const metadata = row.generation_metadata as { time_entries?: any[] } | undefined;
  const timeEntries = metadata?.time_entries ?? [];

  return (
    <React.Fragment>
      <tr className={cn("transition-all group", isExpanded ? "bg-amber-50 dark:bg-amber-950/20" : "hover:bg-slate-50 dark:hover:bg-slate-800/50")}>
        <td className="w-10 px-2 py-2.5 text-center sticky left-0 z-10 bg-white group-hover:bg-slate-50 dark:bg-slate-900 dark:group-hover:bg-slate-800/50 transition-colors">
          <Button
            variant="ghost"
            className="p-1 h-auto text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
            onClick={() => onToggleExpandRow(rowKey)}
          >
            <ChevronRight size={14} className={cn("transition-transform duration-200", isExpanded && "rotate-90")} />
          </Button>
        </td>
        <td className="px-4 py-2.5 font-mono text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase whitespace-nowrap sticky left-[40px] z-10 bg-white group-hover:bg-slate-50 dark:bg-slate-900 dark:group-hover:bg-slate-800/50 transition-colors">
          {formatDate(row.date)}
        </td>
        {!hideEmployeeName ? (
          <td className="px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none whitespace-nowrap sticky left-[120px] sm:left-[140px] z-10 bg-white group-hover:bg-slate-50 dark:bg-slate-900 dark:group-hover:bg-slate-800/50 transition-colors">
            {row.employee_name}
          </td>
        ) : null}
        <td className="px-4 py-2.5"><ShiftCell row={row} t={t} /></td>
        <td className="px-4 py-2.5 text-center font-mono font-bold text-xs text-slate-700 dark:text-slate-200 tracking-tighter">
          <div className="flex items-center justify-center gap-1.5">
            <div className="shadow-inner bg-slate-50 dark:bg-slate-800/50 rounded-md py-1 px-2 inline-block">
              {formatTime(row.actual_punch_in)}
            </div>
            {timeEntries.length > 1 && (
              <span 
                className="text-[8px] bg-blue-50 text-blue-600 dark:bg-blue-950/30 dark:text-blue-400 px-1.5 py-0.5 rounded-full font-bold uppercase tracking-wider scale-90 cursor-help"
                title={timeEntries
                  .map((e: any, i: number) => {
                    const inT = formatTime(e.punch_in_time);
                    const outT = e.punch_out_time ? formatTime(e.punch_out_time) : "---";
                    return `Session ${i + 1}: ${inT} - ${outT} (${e.entry_type})`;
                  })
                  .join("\n")
                }
              >
                Split
              </span>
            )}
          </div>
        </td>
        <td className="px-4 py-2.5 text-center font-mono font-bold text-xs text-slate-700 dark:text-slate-200 tracking-tighter">
          <div className="flex items-center justify-center gap-1.5">
            <div className="shadow-inner bg-slate-50 dark:bg-slate-800/50 rounded-md py-1 px-2 inline-block">
              {formatTime(row.actual_punch_out)}
            </div>
            {timeEntries.length > 1 && (
              <span 
                className="text-[8px] bg-blue-50 text-blue-600 dark:bg-blue-950/30 dark:text-blue-400 px-1.5 py-0.5 rounded-full font-bold uppercase tracking-wider scale-90 cursor-help opacity-0"
                title=""
              >
                Split
              </span>
            )}
          </div>
        </td>
        <td className="px-4 py-2.5 text-right">
          <div 
            className="font-mono font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter leading-none cursor-help"
            title={buildWorkUnitsTooltip(row.std_work_units, t(getUnitTypeI18nKey(row.unit_type)), row.unit_coefficient_hours, row.computed_std_hours)}
          >
            {formatHours(row.computed_std_hours)}<span className="text-[10px] opacity-70 ml-0.5">h</span>
          </div>
        </td>
        <td className="px-4 py-2.5 text-right"><OtCell onApproveOt={onApproveOt} row={row} t={t} /></td>
        <td className="px-4 py-2.5 text-center"><ReportStatusBadge onResolveUnscheduled={onResolveUnscheduled} row={row} status={row.status} t={t} /></td>
        {isDebugMode ? (
          <td className="px-4 py-2.5 text-center">
            <Button
              className={cn("p-1.5 rounded-lg transition-all shadow-sm border h-auto", isExpanded ? "bg-amber-100 dark:bg-amber-900 text-amber-700 dark:text-amber-300 border-amber-200" : "bg-white dark:bg-slate-900 text-amber-600 dark:text-amber-300 border-slate-100 dark:border-slate-800 hover:bg-amber-50 dark:bg-amber-950")}
              onClick={() => { onToggleExpandRow(rowKey); }}
              variant="ghost"
            >
              <Bug size={16} />
            </Button>
          </td>
        ) : null}
        <td className="px-4 py-2.5 text-right">
          <RowActions
            existingAdjustment={existingAdjustment}
            isConfirming={isConfirming}
            isReprocessing={isReprocessing}
            onConfirmClick={onConfirmClick}
            onReprocessClick={onReprocessClick}
            row={row}
            t={t}
          />
        </td>
      </tr>
      {isExpanded ? (
        <DailyReportTableRowExpanded
          formatTime={formatTime}
          isDebugMode={isDebugMode}
          row={row}
          t={t}
        />
      ) : null}
    </React.Fragment>
  );
}
