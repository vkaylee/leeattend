import { CalendarDays, RefreshCw, CheckCircle, Bug } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import ResponsiveTable from "@/components/ui/responsive-table";
import { DailyAttendanceReport, AttendanceAdjustmentRequest } from "@/lib/api";
import { cn } from "@/lib/utils";
import { formatHours } from "@/lib/utils/attendance";

import { DailyReportTableRow } from "./daily-report-table-row";
import { ReportOtStatusBadge } from "./report-ot-status-badge";
import { ReportStatusBadge } from "./report-status-badge";

// ── Types ──────────────────────────────────────────────────────────

export interface DailyReportTableProps {
  adjustmentMap: Partial<Record<string, AttendanceAdjustmentRequest>>;
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
}

// ── Skeleton ───────────────────────────────────────────────────────

function SkeletonRow({ hideEmployeeName, isDebugMode }: { hideEmployeeName: boolean; isDebugMode: boolean }) {
  return (
    <tr className="animate-pulse border-b border-slate-50 dark:border-slate-800/50">
      <td className="px-4 py-4"><div className="h-4 w-6 bg-slate-200 dark:bg-slate-800 rounded"></div></td>
      <td className="px-4 py-4"><div className="h-4 w-20 bg-slate-200 dark:bg-slate-800 rounded"></div></td>
      {!hideEmployeeName ? (
        <td className="px-4 py-4">
          <div className="space-y-2">
            <div className="h-4 w-24 bg-slate-200 dark:bg-slate-800 rounded"></div>
            <div className="h-3 w-16 bg-slate-200 dark:bg-slate-800 rounded"></div>
          </div>
        </td>
      ) : null}
      <td className="px-4 py-4"><div className="h-4 w-14 bg-slate-200 dark:bg-slate-800 rounded-full mx-auto"></div></td>
      <td className="px-4 py-4"><div className="h-4 w-8 bg-slate-200 dark:bg-slate-800 rounded mx-auto"></div></td>
      <td className="px-4 py-4"><div className="h-4 w-16 bg-slate-200 dark:bg-slate-800 rounded"></div></td>
      <td className="px-4 py-4"><div className="h-4 w-12 bg-slate-200 dark:bg-slate-800 rounded mx-auto"></div></td>
      <td className="px-4 py-4"><div className="h-4 w-12 bg-slate-200 dark:bg-slate-800 rounded mx-auto"></div></td>
      <td className="px-4 py-4"><div className="h-4 w-10 bg-slate-200 dark:bg-slate-800 rounded ml-auto"></div></td>
      <td className="px-4 py-4"><div className="h-4 w-10 bg-slate-200 dark:bg-slate-800 rounded ml-auto"></div></td>
      <td className="px-4 py-4"><div className="h-6 w-16 bg-slate-200 dark:bg-slate-800 rounded-full mx-auto"></div></td>
      {isDebugMode ? <td className="px-4 py-4"><div className="h-4 w-8 bg-slate-200 dark:bg-slate-800 rounded mx-auto"></div></td> : null}
      <td className="px-4 py-4"><div className="h-6 w-6 bg-slate-200 dark:bg-slate-800 rounded ml-auto"></div></td>
    </tr>
  );
}

// ── Desktop Table Header ──────────────────────────────────────────

function TableHeader({ hideEmployeeName, isDebugMode, t }: { hideEmployeeName: boolean; isDebugMode: boolean; t: (k: string) => string }) {
  return (
    <thead>
      <tr className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky top-0 z-20">
        <th className="w-10 px-2 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest sticky left-0 z-20 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm"></th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest sticky left-[40px] z-20 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm">
          {t("common.date")}
        </th>
        {!hideEmployeeName ? (
          <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest sticky left-[120px] sm:left-[140px] z-20 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm">
            {t("common.employee")}
          </th>
        ) : null}
        <th className="px-3 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">
          {t("reports.table.segment_type")}
        </th>
        <th className="px-1 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center w-10">
          {t("reports.table.segment_number")}
        </th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("reports.table.scheduled")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.in")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.out")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("reports.table.std_hours")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-orange-500 dark:text-orange-400 uppercase tracking-widest text-right">{t("reports.table.ot_hours")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.status")}</th>
        {isDebugMode ? (
          <th className="px-4 py-2.5 text-[10px] font-bold text-amber-600 dark:text-amber-300 uppercase tracking-widest text-center">DEBUG</th>
        ) : null}
        <th className="px-4 py-2.5"></th>
      </tr>
    </thead>
  );
}

// ── Mobile Card ────────────────────────────────────────────────────

/**
 * DailyReportMobileCard — mobile card for a daily attendance report row.
 * Complexity (21) is driven by numerous JSX conditionals: OT approval, multi-segment,
 * debug mode, approval/repro buttons. Decomposed into separate sub-components
 * (SkeletonRow, TableHeader, ReportStatusBadge, ReportOtStatusBadge) already.
 * Further splitting of the card body would reduce readability more than complexity.
 */
// eslint-disable-next-line complexity, max-lines-per-function
function DailyReportMobileCard({
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
}: {
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
}) {
  const segmentKey = `${row.employee_id}-${row.date}-${row.segment_id ?? row.sort_order ?? '0'}`;
  const dayKeyEmployee = `${row.employee_id}-${row.date}`;
  const isExpanded = expandedDebugRows.has(segmentKey);
  const isReprocessing = reprocessingRow === dayKeyEmployee;
  const isConfirming = confirmingRow === dayKeyEmployee;
  const existingAdjustment = adjustmentMap[`${row.employment_id}-${row.date}`];

  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900">
      {/* Header: Date + Employee */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border border-slate-100 bg-slate-50 text-slate-400 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-500">
            <CalendarDays size={16} />
          </div>
          <div className="min-w-0">
            <div className="font-mono text-xs font-bold uppercase text-slate-700 dark:text-slate-200">{formatDate(row.date)}</div>
            {!hideEmployeeName ? (
              <div className="mt-0.5 text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">{row.employee_name}</div>
            ) : null}
          </div>
        </div>
        <ReportStatusBadge onResolveUnscheduled={onResolveUnscheduled} row={row} status={row.status} t={t} />
      </div>

      {/* Schedule + Punch */}
      <div className="mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div>
          <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">{t("reports.table.scheduled")}</div>
          <div className="mt-1 text-xs font-bold text-slate-700 dark:text-slate-200">
            {row.scheduled_start ? `${row.scheduled_start.substring(0, 5)} — ${row.scheduled_end?.substring(0, 5)}` : t("reports.no_shift")}
          </div>
        </div>
        <div>
          <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">{t("common.in")} / {t("common.out")}</div>
          <div className="mt-1 font-mono text-xs font-bold text-slate-700 dark:text-slate-200">
            {formatTime(row.actual_punch_in)} / {formatTime(row.actual_punch_out)}
          </div>
        </div>
        <div>
          <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">{t("reports.table.std_hours")}</div>
          <div className="mt-1 font-mono text-sm font-bold text-primary-600 dark:text-primary-400">
            {formatHours(row.computed_std_hours)}h
          </div>
        </div>
        <div>
          <div className="text-[10px] font-bold uppercase tracking-wider text-orange-500">{t("reports.table.ot_hours")}</div>
          <div className="mt-1 font-mono text-sm font-bold text-orange-600 dark:text-orange-400">
            {row.computed_ot_hours > 0 ? `${formatHours(row.computed_ot_hours)}h` : "—"}
          </div>
          {row.ot_claim_status && row.ot_claim_status !== "none" ? (
            <div className="mt-1">
              <ReportOtStatusBadge status={row.ot_claim_status} t={t} />
            </div>
          ) : null}
        </div>
      </div>

      {/* Segment info if multi-segment */}
      {row.segment_type ? (
        <div className="mt-3 flex items-center gap-2">
          <span className="rounded-md border border-blue-200 bg-blue-50 px-2 py-0.5 text-[9px] font-bold uppercase tracking-widest text-blue-700 dark:border-blue-800 dark:bg-blue-950/30 dark:text-blue-300">
            {t(`work_segments.segment_type.${row.segment_type}`)}
          </span>
          {(row.segment_count ?? 1) > 1 ? (
            <span className="font-mono text-[10px] font-bold text-slate-400">
              {(row.sort_order ?? 0) + 1}/{row.segment_count}
            </span>
          ) : null}
        </div>
      ) : null}

      {/* OT Approve (mobile inline) */}
      {row.ot_claim_status === "pending_manager" && onApproveOt ? (
        <div className="mt-4 flex gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
          <Button
            className="flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-emerald-500 px-3 py-2 text-[10px] font-bold uppercase tracking-widest text-white shadow-sm transition-colors hover:bg-emerald-600"
            onClick={() => { onApproveOt(row, "approved"); }}
            variant="ghost"
          >
            <CheckCircle size={14} /> {t("reports.ot_approve.approve")}
          </Button>
          <Button
            className="flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-rose-500 px-3 py-2 text-[10px] font-bold uppercase tracking-widest text-white shadow-sm transition-colors hover:bg-rose-600"
            onClick={() => { onApproveOt(row, "rejected"); }}
            variant="ghost"
          >
            <RefreshCw size={14} /> {t("reports.ot_approve.reject")}
          </Button>
        </div>
      ) : null}

      {/* Actions */}
      <div className="mt-4 flex items-center justify-end gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
        <Button
          className={cn(
            "flex items-center justify-center rounded-lg border p-2 transition-all",
            existingAdjustment
              ? "border-orange-200 bg-orange-50 text-orange-600 dark:border-orange-800 dark:bg-orange-950 dark:text-orange-300"
              : "border-slate-200 bg-white text-slate-500 hover:bg-primary-50 hover:text-primary-600 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-primary-950/30 dark:hover:text-primary-400"
          )}
          disabled={isConfirming}
          onClick={() => { onConfirmClick(row, existingAdjustment); }}
          title={existingAdjustment ? t("reports.confirm.pending_badge") : t("common.attendance_confirm")}
          variant="ghost"
        >
          {isConfirming ? <RefreshCw className="animate-spin" size={14} /> : <CheckCircle size={14} />}
        </Button>
        <Button
          className="rounded-lg border border-slate-200 bg-white p-2 text-blue-600 transition-colors hover:bg-blue-50 disabled:opacity-50 dark:border-slate-700 dark:bg-slate-800 dark:text-blue-300 dark:hover:bg-blue-950/30"
          disabled={isReprocessing}
          onClick={() => { onReprocessClick(row); }}
          title={t("reports.reprocess.tooltip")}
          variant="ghost"
        >
          <RefreshCw className={cn(isReprocessing && "animate-spin")} size={16} />
        </Button>
        {isDebugMode ? (
          <Button
            className={cn("rounded-lg border p-2 transition-all", isExpanded ? "border-amber-200 bg-amber-100 text-amber-700 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300" : "border-slate-200 bg-white text-amber-600 hover:bg-amber-50 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-amber-950/30")}
            onClick={() => { onToggleExpandRow(segmentKey); }}
            variant="ghost"
          >
            <Bug size={16} />
          </Button>
        ) : null}
      </div>
    </div>
  );
}

// ── Main Component ─────────────────────────────────────────────────

export default function DailyReportTable({
  adjustmentMap,
  confirmingRow,
  data,
  expandedDebugRows,
  formatDate,
  formatTime,
  hideEmployeeName = false,
  isDebugMode,
  isLoading,
  onApproveOt,
  onConfirmClick,
  onReprocessClick,
  onResolveUnscheduled,
  onToggleExpandRow,
  reprocessingRow,
  t,
}: DailyReportTableProps) {
  const colSpan = isDebugMode ? (hideEmployeeName ? 12 : 13) : (hideEmployeeName ? 11 : 12);

  return (
    <div className="overflow-auto flex-1 min-h-0">
      <ResponsiveTable<DailyAttendanceReport>
        data={data}
        emptyState={{
          icon: <CalendarDays className="h-8 w-8" />,
          title: t("reports.messages.no_data"),
        }}
        getCardKey={(row) => `${row.employee_id}-${row.date}-${row.segment_id ?? row.sort_order ?? '0'}`}
        isLoading={isLoading}
        renderCard={(row) => (
          <DailyReportMobileCard
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
          />
        )}
        renderDesktopTable={() => (
          <table className="w-full text-left min-w-[1200px] border-collapse relative">
            <TableHeader hideEmployeeName={hideEmployeeName} isDebugMode={isDebugMode} t={t} />
            <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
              {isLoading ? (
                <>
                  {Array.from({ length: 5 }).map((_, i) => (
                    <SkeletonRow hideEmployeeName={hideEmployeeName} isDebugMode={isDebugMode} key={`skeleton-${i}`} />
                  ))}
                </>
              ) : data.length === 0 ? (
                <tr>
                  <td className="px-10 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50" colSpan={colSpan}>
                    {t("reports.messages.no_data")}
                  </td>
                </tr>
              ) : (
                data.map((row: DailyAttendanceReport, idx: number, arr: DailyAttendanceReport[]) => {
                  const empDateKey = `${row.employee_id}-${row.date}`;
                  const isFirstInGroup = idx === 0 || `${arr[idx - 1].employee_id}-${arr[idx - 1].date}` !== empDateKey;
                  const isLastInGroup = idx === arr.length - 1 || `${arr[idx + 1].employee_id}-${arr[idx + 1].date}` !== empDateKey;
                  const isMultiSegmentGroup = (row.segment_count ?? 1) > 1;
                  return (
                  <DailyReportTableRow
                    adjustmentMap={adjustmentMap}
                    confirmingRow={confirmingRow}
                    expandedDebugRows={expandedDebugRows}
                    formatDate={formatDate}
                    formatTime={formatTime}
                    hideEmployeeName={hideEmployeeName}
                    isDebugMode={isDebugMode}
                    isFirstInGroup={isFirstInGroup}
                    isLastInGroup={isLastInGroup}
                    isMultiSegmentGroup={isMultiSegmentGroup}
                    key={`${row.employee_id}-${row.date}-${row.segment_id ?? row.sort_order ?? '0'}`}
                    onApproveOt={onApproveOt}
                    onConfirmClick={onConfirmClick}
                    onReprocessClick={onReprocessClick}
                    onResolveUnscheduled={onResolveUnscheduled}
                    onToggleExpandRow={onToggleExpandRow}
                    reprocessingRow={reprocessingRow}
                    row={row}
                    t={t}
                  />
                  );
                })
              )}
            </tbody>
          </table>
        )}
      />
    </div>
  );
}
