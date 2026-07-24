import { RefreshCw, Bug, CheckCircle, ThumbsUp, ThumbsDown, ChevronRight } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useRowMeta } from "@/hooks/reports/use-row-meta";
import { DailyAttendanceReport, AttendanceAdjustmentRequest, ScheduleTrace } from "@/lib/api";
import { getDerivedSource, SEGMENT_TYPE_COLORS } from "@/lib/report-row-utils";
import { cn } from "@/lib/utils";
import { formatHours, getUnitTypeI18nKey, buildWorkUnitsTooltip } from "@/lib/utils/attendance";

import { ScheduleTracePopover } from "../../shifts/schedule-trace-popover";
import { DailyReportTimeline, TimeEntry } from "./daily-report-timeline";
import { ReportOtStatusBadge } from "./report-ot-status-badge";
import { ReportStatusBadge } from "./report-status-badge";

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
      <td className="px-10 py-6" colSpan={isDebugMode ? 12 : 11}>
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
      </td>
    </tr>
  );
}

function SegmentTypeCell({ row, t }: { row: DailyAttendanceReport; t: (k: string) => string }) {
  if (!row.segment_type) {
    return (
      <span className="text-[9px] font-bold text-slate-300 dark:text-slate-600 italic">—</span>
    );
  }
  const segTypeColor = SEGMENT_TYPE_COLORS[row.segment_type] ?? SEGMENT_TYPE_COLORS.work;
  return (
    <span className={cn(
      "text-[9px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-md border leading-none whitespace-nowrap",
      segTypeColor
    )}>
      {t(`work_segments.segment_type.${row.segment_type}`)}
    </span>
  );
}

function SegmentCounterCell({ row }: { row: DailyAttendanceReport }) {
  const hasSegmentData = row.segment_type !== null && row.segment_type !== undefined;
  const isMultiSegment = (row.segment_count ?? 1) > 1;
  if (!hasSegmentData || !isMultiSegment) {
    return (
      <span className="text-[9px] font-bold text-slate-300 dark:text-slate-600 italic">—</span>
    );
  }
  return (
    <span className="text-[10px] font-mono font-bold text-slate-500 dark:text-slate-400 tabular-nums">
      {(row.sort_order ?? 0) + 1}<span className="text-[8px] opacity-50">/{row.segment_count}</span>
    </span>
  );
}

function ScheduledCell({ row, t }: { row: DailyAttendanceReport; t: (k: string) => string }) {
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

  const hasSegmentData = row.segment_type !== null && row.segment_type !== undefined;

  return (
    <div className="flex flex-col gap-1">
      {row.interval_name ? (
        <span className="text-[9px] font-medium text-slate-500 dark:text-slate-400 truncate max-w-[140px]">
          {row.interval_name}
        </span>
      ) : null}
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
      {/* Break info — only show when NOT in segment mode (break = gap between segments) */}
      {!hasSegmentData && row.break_duration_minutes !== null ? (
        <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-0.5 opacity-70 italic">
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
  isFirstInGroup?: boolean;
  isLastInGroup?: boolean;
  isMultiSegmentGroup?: boolean;
  onApproveOt?: (row: DailyAttendanceReport, action: "approved" | "rejected") => void;
  onConfirmClick: (row: DailyAttendanceReport, existing?: AttendanceAdjustmentRequest) => void;
  onReprocessClick: (row: DailyAttendanceReport) => void;
  onResolveUnscheduled?: (row: DailyAttendanceReport) => void;
  onToggleExpandRow: (rowKey: string) => void;
  reprocessingRow: string | null;
  row: DailyAttendanceReport;
  t: (key: string) => string;
}

function PunchTimeCell({
  time,
  formatTime,
  timeEntries,
  hasMultipleSessions,
  side,
}: {
  time: string | null;
  formatTime: (t: string | null) => string;
  timeEntries: TimeEntry[];
  hasMultipleSessions: boolean;
  side: "in" | "out";
}) {
  return (
    <td className="px-4 py-2.5 text-center font-mono font-bold text-xs text-slate-700 dark:text-slate-200 tracking-tighter">
      <div className="flex items-center justify-center gap-1.5">
        <div className="shadow-inner bg-slate-50 dark:bg-slate-800/50 rounded-md py-1 px-2 inline-block">
          {formatTime(time)}
        </div>
        {side === "in" && hasMultipleSessions ? (
          <span
            className="text-[8px] bg-blue-50 text-blue-600 dark:bg-blue-950/30 dark:text-blue-400 px-1.5 py-0.5 rounded-full font-bold uppercase tracking-wider scale-90 cursor-help"
            title={timeEntries.map((e, i) => `Session ${i + 1}: ${formatTime(e.punch_in_time)} - ${e.punch_out_time ? formatTime(e.punch_out_time) : "---"} (${e.entry_type})`).join("\n")}
          >
            Split
          </span>
        ) : null}
      </div>
    </td>
  );
}

function RowCells({
  row, t, formatTime, formatDate, isFirstInGroup,
  hideEmployeeName, timeEntries, hasMultipleSessions, isExpanded,
  onToggleExpandRow, segmentKey, isDebugMode, onApproveOt, onResolveUnscheduled,
}: {
  row: DailyAttendanceReport; t: (k: string) => string;
  formatTime: (t: string | null) => string; formatDate: (d: string) => string;
  isFirstInGroup?: boolean; hideEmployeeName: boolean;
  timeEntries: TimeEntry[]; hasMultipleSessions: boolean;
  isExpanded: boolean; onToggleExpandRow: (k: string) => void; segmentKey: string;
  isDebugMode: boolean;
  onApproveOt?: (row: DailyAttendanceReport, action: "approved" | "rejected") => void;
  onResolveUnscheduled?: (row: DailyAttendanceReport) => void;
}) {
  return (
    <>
      <td className="w-10 px-2 py-2.5 text-center sticky left-0 z-10 bg-white group-hover:bg-slate-50 dark:bg-slate-900 dark:group-hover:bg-slate-800/50 transition-colors">
        <Button className="p-1 h-auto text-slate-400 hover:text-slate-600" onClick={() => { onToggleExpandRow(segmentKey); }} variant="ghost">
          <ChevronRight className={cn("transition-transform duration-200", isExpanded && "rotate-90")} size={14} />
        </Button>
      </td>
      <td className={cn("px-4 py-2.5 font-mono text-[11px] font-bold uppercase whitespace-nowrap sticky left-[40px] z-10 bg-white group-hover:bg-slate-50 dark:bg-slate-900 dark:group-hover:bg-slate-800/50 transition-colors", !isFirstInGroup && "text-slate-300")}>
        {formatDate(row.date)}
      </td>
      {!hideEmployeeName && (
        <td className={cn("px-4 py-2.5 font-bold text-sm tracking-tight uppercase leading-none whitespace-nowrap sticky left-[120px] sm:left-[140px] z-10 bg-white group-hover:bg-slate-50 dark:bg-slate-900 dark:group-hover:bg-slate-800/50 transition-colors", !isFirstInGroup ? "text-slate-300" : "text-slate-700")}>
          {row.employee_name}
        </td>
      )}
      <td className="px-3 py-2.5 text-center"><SegmentTypeCell row={row} t={t} /></td>
      <td className="px-1 py-2.5 text-center"><SegmentCounterCell row={row} /></td>
      <td className="px-4 py-2.5"><ScheduledCell row={row} t={t} /></td>
      <PunchTimeCell formatTime={formatTime} hasMultipleSessions={hasMultipleSessions} side="in" time={row.actual_punch_in} timeEntries={timeEntries} />
      <PunchTimeCell formatTime={formatTime} hasMultipleSessions={false} side="out" time={row.actual_punch_out} timeEntries={timeEntries} />
      <td className="px-4 py-2.5 text-right">
        <div className="font-mono font-bold text-primary-600 text-base tracking-tighter leading-none cursor-help" title={buildWorkUnitsTooltip(row.std_work_units, t(getUnitTypeI18nKey(row.unit_type)), row.unit_coefficient_hours, row.computed_std_hours)}>
          {formatHours(row.computed_std_hours)}<span className="text-[10px] opacity-70 ml-0.5">h</span>
        </div>
      </td>
      <td className="px-4 py-2.5 text-right"><OtCell onApproveOt={onApproveOt} row={row} t={t} /></td>
      <td className="px-4 py-2.5 text-center"><ReportStatusBadge onResolveUnscheduled={onResolveUnscheduled} row={row} status={row.status} t={t} /></td>
      {isDebugMode ? (
        <td className="px-4 py-2.5 text-center">
          <Button className={cn("p-1.5 rounded-lg transition-all shadow-sm border h-auto", isExpanded ? "bg-amber-100 text-amber-700 border-amber-200" : "bg-white text-amber-600 border-slate-100 hover:bg-amber-50")} onClick={() => { onToggleExpandRow(segmentKey); }} variant="ghost">
            <Bug size={16} />
          </Button>
        </td>
      ) : null}
    </>
  );
}

export function DailyReportTableRow(props: DailyReportTableRowProps) {
  const { row, formatDate, formatTime, t, hideEmployeeName, isFirstInGroup, isLastInGroup, isMultiSegmentGroup, isDebugMode, onApproveOt, onConfirmClick, onReprocessClick, onResolveUnscheduled, onToggleExpandRow } = props;
  const meta = useRowMeta(row, props);

  return (
    <React.Fragment>
      <tr className={cn(
        "transition-all group",
        meta.isExpanded ? "bg-amber-50 dark:bg-amber-950/20" : "hover:bg-slate-50 dark:hover:bg-slate-800/50",
        isFirstInGroup && isMultiSegmentGroup && "border-t-2 border-t-blue-200 dark:border-t-blue-800",
        isMultiSegmentGroup && "border-l-2 border-l-blue-300 dark:border-l-blue-700",
        !isFirstInGroup && isMultiSegmentGroup && "border-t border-t-blue-100/50",
        isLastInGroup && isMultiSegmentGroup && "border-b border-b-slate-100",
      )}>
        <RowCells formatDate={formatDate} formatTime={formatTime} hasMultipleSessions={meta.hasMultipleSessions} hideEmployeeName={hideEmployeeName}
          isDebugMode={isDebugMode} isExpanded={meta.isExpanded}
          isFirstInGroup={isFirstInGroup} onApproveOt={onApproveOt}
          onResolveUnscheduled={onResolveUnscheduled} onToggleExpandRow={onToggleExpandRow} row={row}
          segmentKey={meta.segmentKey} t={t} timeEntries={meta.timeEntries} />
        <td className="px-4 py-2.5 text-right">
          <RowActions
            existingAdjustment={meta.existingAdjustment}
            isConfirming={meta.isConfirming}
            isReprocessing={meta.isReprocessing}
            onConfirmClick={onConfirmClick}
            onReprocessClick={onReprocessClick}
            row={row}
            t={t}
          />
        </td>
      </tr>
      {meta.isExpanded ? (
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
