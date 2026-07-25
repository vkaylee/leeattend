import { RefreshCw, Bug, ChevronRight } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useRowMeta } from "@/hooks/reports/use-row-meta";
import { DailyAttendanceReport, AttendanceAdjustmentRequest } from "@/lib/api";
import { cn } from "@/lib/utils";
import { buildWorkUnitsTooltip, formatHours, getUnitTypeI18nKey } from "@/lib/utils/attendance";

import {
  SegmentTypeCell,
  SegmentCounterCell,
  ScheduledCell,
  OtCell,
  RowActions,
  PunchTimeCell,
} from "./daily-report-cells";
import { DailyReportTimeline, TimeEntry } from "./daily-report-timeline";
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
