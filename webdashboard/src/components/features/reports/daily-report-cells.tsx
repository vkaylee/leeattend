import { RefreshCw, CheckCircle, ThumbsUp, ThumbsDown } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { DailyAttendanceReport, AttendanceAdjustmentRequest, ScheduleTrace } from "@/lib/api";
import { getDerivedSource, SEGMENT_TYPE_COLORS } from "@/lib/report-row-utils";
import { cn } from "@/lib/utils";
import { formatHours, getUnitTypeI18nKey, buildWorkUnitsTooltip } from "@/lib/utils/attendance";

import { ScheduleTracePopover } from "../../shifts/schedule-trace-popover";
import { TimeEntry } from "./daily-report-timeline";
import { ReportOtStatusBadge } from "./report-ot-status-badge";

// ── SegmentTypeCell ────────────────────────────────────────────────────

export function SegmentTypeCell({ row, t }: { row: DailyAttendanceReport; t: (k: string) => string }) {
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

// ── SegmentCounterCell ─────────────────────────────────────────────────

export function SegmentCounterCell({ row }: { row: DailyAttendanceReport }) {
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

// ── ScheduledTimeDisplay ───────────────────────────────────────────────

function ScheduledTimeDisplay({
  startTime,
  endTime,
  hasTraces,
  derivedSource,
  traces,
  shiftName,
  shiftSymbol,
}: {
  startTime: string;
  endTime: string | null;
  hasTraces: boolean;
  derivedSource: string;
  traces: ScheduleTrace[];
  shiftName?: string | null;
  shiftSymbol?: string | null;
}) {
  return (
    <div className="relative w-fit flex items-center">
      <span className={cn(
        "font-bold text-slate-700 dark:text-slate-200 text-xs uppercase tracking-tighter shadow-sm bg-white dark:bg-slate-900 py-0.5 rounded-lg border border-slate-100 dark:border-slate-800",
        hasTraces ? "pl-2 pr-6.5" : "px-2"
      )}>
        {startTime.substring(0, 5)} — {endTime?.substring(0, 5)}
      </span>
      {hasTraces ? (
        <div className="absolute right-1 top-1/2 -translate-y-1/2 z-10 flex items-center justify-center">
          <ScheduleTracePopover
            endTime={endTime}
            shiftName={shiftName}
            shiftSymbol={shiftSymbol}
            source={derivedSource}
            startTime={startTime}
            traces={traces}
          />
        </div>
      ) : null}
    </div>
  );
}

// ── BreakInfoBadge ─────────────────────────────────────────────────────

function BreakInfoBadge({
  breakStart,
  breakEnd,
  breakDurationMinutes,
  t,
}: {
  breakStart: string | null;
  breakEnd: string | null;
  breakDurationMinutes: number;
  t: (k: string) => string;
}) {
  return (
    <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-0.5 opacity-70 italic">
      {breakStart
        ? `${t("common.break")}: ${breakStart.substring(0, 5)}-${breakEnd?.substring(0, 5)}`
        : `${t("common.break")}: ${breakDurationMinutes.toString()}m`}
    </span>
  );
}

// ── ScheduledCell ──────────────────────────────────────────────────────

export function ScheduledCell({ row, t }: { row: DailyAttendanceReport; t: (k: string) => string }) {
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
      <ScheduledTimeDisplay
        derivedSource={derivedSource}
        endTime={row.scheduled_end}
        hasTraces={hasTraces}
        shiftName={row.shift_name}
        shiftSymbol={row.shift_symbol}
        startTime={row.scheduled_start}
        traces={traces}
      />
      {!hasSegmentData && row.break_duration_minutes !== null ? (
        <BreakInfoBadge
          breakDurationMinutes={row.break_duration_minutes}
          breakEnd={row.break_end}
          breakStart={row.break_start}
          t={t}
        />
      ) : null}
    </div>
  );
}

// ── OtCell ─────────────────────────────────────────────────────────────

export function OtCell({
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

// ── RowActions ─────────────────────────────────────────────────────────

export function RowActions({
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
    <div className="flex items-center justify-end gap-1.5 opacity-0 group-hover/row:opacity-100 transition-opacity focus-within:opacity-100">
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

// ── PunchTimeCell ──────────────────────────────────────────────────────

/** Tailwind classes the punch-time cell needs on its owning `<td>`. */
export const PUNCH_TIME_CELL_CLASS = "px-4 py-2.5 text-center font-mono font-bold text-xs text-slate-700 dark:text-slate-200 tracking-tighter";

/**
 * Punch-time cell body. Renders cell *content* only — the owning `<td>` is
 * supplied by SmartTable, which styles it via `PUNCH_TIME_CELL_CLASS`.
 */
export function PunchTimeCellContent({
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
  );
}
