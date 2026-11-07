import { CalendarDays, RefreshCw, CheckCircle } from "lucide-react";
import { useCallback, useMemo } from "react";

import Button from "@/components/ui/button";
import ResponsiveTable from "@/components/ui/responsive-table";
import { DailyAttendanceReport, AttendanceAdjustmentRequest } from "@/lib/api";
import { cn } from "@/lib/utils";
import { formatHours } from "@/lib/utils/attendance";

import { buildDailyReportColumns, dayGroupKeyOf, segmentKeyOf } from "./daily-report-columns";
import { DailyReportExpandedPanel } from "./daily-report-expanded-panel";
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

function SkeletonRow({ hideEmployeeName }: { hideEmployeeName: boolean }) {
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
      <td className="px-4 py-4"><div className="h-6 w-6 bg-slate-200 dark:bg-slate-800 rounded ml-auto"></div></td>
    </tr>
  );
}

// ── Mobile Card ────────────────────────────────────────────────────

/**
 * DailyReportMobileCard — mobile card for a daily attendance report row.
 * Complexity (21) is driven by numerous JSX conditionals: OT approval, multi-segment,
 * approval/repro buttons. Decomposed into separate sub-components
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
  isLoading,
  onApproveOt,
  onConfirmClick,
  onReprocessClick,
  onResolveUnscheduled,
  onToggleExpandRow,
  reprocessingRow,
  t,
}: DailyReportTableProps) {
  /**
   * Visual grouping is positional: consecutive rows sharing an employee+date form
   * one group. Precomputed once per data change so cells and row classNames can
   * look up their group position by segment key without rescanning the array.
   */
  const groupFlags = useMemo(() => {
    const flags = new Map<string, { isFirst: boolean; isLast: boolean; isMulti: boolean }>();
    data.forEach((row, idx) => {
      const key = dayGroupKeyOf(row);
      flags.set(segmentKeyOf(row), {
        isFirst: idx === 0 || dayGroupKeyOf(data[idx - 1]) !== key,
        isLast: idx === data.length - 1 || dayGroupKeyOf(data[idx + 1]) !== key,
        isMulti: (row.segment_count ?? 1) > 1,
      });
    });
    return flags;
  }, [data]);

  const isFirstInGroup = useCallback(
    (row: DailyAttendanceReport) => groupFlags.get(segmentKeyOf(row))?.isFirst ?? true,
    [groupFlags],
  );

  const columns = useMemo(
    () => buildDailyReportColumns(
      {
        adjustmentMap, confirmingRow, expandedDebugRows, formatDate, formatTime,
        hideEmployeeName, onApproveOt, onConfirmClick, onReprocessClick,
        onResolveUnscheduled, onToggleExpandRow, reprocessingRow, t,
      },
      isFirstInGroup,
    ),
    [
      adjustmentMap, confirmingRow, expandedDebugRows, formatDate, formatTime,
      hideEmployeeName, onApproveOt, onConfirmClick, onReprocessClick,
      onResolveUnscheduled, onToggleExpandRow, reprocessingRow, t, isFirstInGroup,
    ],
  );

  const getRowClassName = useCallback((row: DailyAttendanceReport) => {
    const flags = groupFlags.get(segmentKeyOf(row));
    const isExpanded = expandedDebugRows.has(segmentKeyOf(row));
    return cn(
      isExpanded && "bg-amber-50 dark:bg-amber-950/20",
      flags?.isMulti && "border-l-2 border-l-blue-300 dark:border-l-blue-700",
      flags?.isFirst && flags.isMulti && "border-t-2 border-t-blue-200 dark:border-t-blue-800",
      !flags?.isFirst && flags?.isMulti && "border-t border-t-blue-100/50",
      flags?.isLast && flags.isMulti && "border-b border-b-slate-100",
    );
  }, [groupFlags, expandedDebugRows]);

  const renderExpandedRow = useCallback((row: DailyAttendanceReport) => {
    if (!expandedDebugRows.has(segmentKeyOf(row))) return null;
    return (
      <DailyReportExpandedPanel
        formatTime={formatTime}
        row={row}
        t={t}
      />
    );
  }, [expandedDebugRows, formatTime, t]);

  const renderSkeletonRow = useCallback(
    () => <SkeletonRow hideEmployeeName={hideEmployeeName} />,
    [hideEmployeeName],
  );

  return (
    <div className="overflow-auto flex-1 min-h-0">
      <ResponsiveTable<DailyAttendanceReport>
        columns={columns}
        data={data}
        emptyState={{
          description: t("reports.messages.no_data_desc", "Vui lòng chọn khoảng thời gian khác hoặc kiểm tra lại lịch làm việc."),
          icon: <CalendarDays className="h-8 w-8 text-slate-400" />,
          title: t("reports.messages.no_data", "Không có dữ liệu trong khoảng thời gian này."),
        }}
        getCardKey={(row) => segmentKeyOf(row)}
        getRowClassName={getRowClassName}
        getRowId={(row) => segmentKeyOf(row)}
        isLoading={isLoading}
        renderCard={(row) => (
          <DailyReportMobileCard
            adjustmentMap={adjustmentMap}
            confirmingRow={confirmingRow}
            expandedDebugRows={expandedDebugRows}
            formatDate={formatDate}
            formatTime={formatTime}
            hideEmployeeName={hideEmployeeName}
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
        renderExpandedRow={renderExpandedRow}
        renderSkeletonRow={renderSkeletonRow}
        stickyLeftColumns={hideEmployeeName ? 2 : 3}
      />
    </div>
  );
}
