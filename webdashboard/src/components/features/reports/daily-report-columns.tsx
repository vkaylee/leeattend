import type { ColumnDef } from "@tanstack/react-table";

import { Bug, ChevronRight } from "lucide-react";
import React from "react";

import type { SmartTableColumnMeta } from "@/components/ui/smart-table";

import Button from "@/components/ui/button";
import { DailyAttendanceReport, AttendanceAdjustmentRequest } from "@/lib/api";
import { cn } from "@/lib/utils";
import { buildWorkUnitsTooltip, formatHours, getUnitTypeI18nKey } from "@/lib/utils/attendance";

import {
  SegmentTypeCell,
  SegmentCounterCell,
  ScheduledCell,
  OtCell,
  RowActions,
  PunchTimeCellContent,
  PUNCH_TIME_CELL_CLASS,
} from "./daily-report-cells";
import { TimeEntry } from "./daily-report-timeline";
import { ReportStatusBadge } from "./report-status-badge";

/**
 * Column widths for the three pinned identity columns.
 *
 * These feed TanStack's `column.getSize()`, from which SmartTable derives sticky
 * offsets via `column.getStart("left")`. Previously the offsets were hardcoded as
 * Tailwind classes (`left-[40px]`, `left-[120px] sm:left-[140px]`) duplicated
 * across the header and row components — changing one column's width silently
 * broke alignment. Sizes now live here only; offsets are computed.
 */
const EXPANDER_WIDTH = 40;
const DATE_WIDTH = 100;
const EMPLOYEE_WIDTH = 180;

type DailyReportColumn = ColumnDef<DailyAttendanceReport> & { meta?: SmartTableColumnMeta };

export interface DailyReportColumnsOptions {
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
  t: (key: string) => string;
}

/** Stable per-segment identity used for expansion state and React keys. */
export function segmentKeyOf(row: DailyAttendanceReport): string {
  return `${row.employee_id}-${row.date}-${row.segment_id ?? row.sort_order ?? "0"}`;
}

/** Employee+date grouping key — consecutive rows sharing it form one visual group. */
export function dayGroupKeyOf(row: DailyAttendanceReport): string {
  return `${row.employee_id}-${row.date}`;
}

function timeEntriesOf(row: DailyAttendanceReport): TimeEntry[] {
  const metadata = row.generation_metadata as { time_entries?: TimeEntry[] } | undefined;
  return metadata?.time_entries ?? [];
}

/** Mirrors use-row-meta's split-session rule: auto-deduct breaks add a spurious block. */
function hasMultipleSessions(row: DailyAttendanceReport): boolean {
  const workEntries = timeEntriesOf(row).filter((e) => e.entry_type === "work");
  return row.is_break_auto_deduct === true ? workEntries.length > 2 : workEntries.length > 1;
}

const HEADER_CLASS = "px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest";

/**
 * SmartTable's default cell carries `truncate` (overflow-hidden + nowrap), which
 * would clip this table's absolutely-positioned schedule-trace popover and the
 * stacked OT approve/reject buttons. Every cell class below starts from this base
 * to restore normal overflow.
 */
const CELL_BASE = "overflow-visible whitespace-normal";

/**
 * Builds the desktop column set for the daily attendance report.
 *
 * `isFirstInGroup` (used to mute repeated date/employee text) is derived from the
 * previous row in the sorted data, so it is passed in rather than read per-cell.
 */
export function buildDailyReportColumns(
  opts: DailyReportColumnsOptions,
  isFirstInGroup: (row: DailyAttendanceReport) => boolean,
): DailyReportColumn[] {
  const {
    t, formatDate, formatTime, hideEmployeeName, isDebugMode, expandedDebugRows,
    onToggleExpandRow, onApproveOt, onResolveUnscheduled, onConfirmClick,
    onReprocessClick, adjustmentMap, confirmingRow, reprocessingRow,
  } = opts;

  const columns: DailyReportColumn[] = [
    {
      id: "expander",
      header: "",
      size: EXPANDER_WIDTH,
      meta: { cellClassName: `${CELL_BASE} px-2 py-2.5 text-center`, headerClassName: "px-2 py-2.5" },
      cell: ({ row }) => {
        const key = segmentKeyOf(row.original);
        return (
          <Button
            className="p-1 h-auto text-slate-400 hover:text-slate-600"
            onClick={() => { onToggleExpandRow(key); }}
            variant="ghost"
          >
            <ChevronRight
              className={cn("transition-transform duration-200", expandedDebugRows.has(key) && "rotate-90")}
              size={14}
            />
          </Button>
        );
      },
    },
    {
      id: "date",
      header: t("common.date"),
      size: DATE_WIDTH,
      meta: { cellClassName: `${CELL_BASE} px-4 py-2.5`, headerClassName: HEADER_CLASS },
      cell: ({ row }) => (
        <span className={cn(
          "font-mono text-[11px] font-bold uppercase whitespace-nowrap",
          !isFirstInGroup(row.original) && "text-slate-300",
        )}>
          {formatDate(row.original.date)}
        </span>
      ),
    },
  ];

  if (!hideEmployeeName) {
    columns.push({
      id: "employee",
      header: t("common.employee"),
      size: EMPLOYEE_WIDTH,
      meta: { cellClassName: `${CELL_BASE} px-4 py-2.5`, headerClassName: HEADER_CLASS },
      cell: ({ row }) => (
        <span className={cn(
          "font-bold text-sm tracking-tight uppercase leading-none whitespace-nowrap",
          isFirstInGroup(row.original) ? "text-slate-700" : "text-slate-300",
        )}>
          {row.original.employee_name}
        </span>
      ),
    });
  }

  columns.push(
    {
      id: "segment_type",
      header: t("reports.table.segment_type"),
      size: 120,
      meta: { cellClassName: `${CELL_BASE} px-3 py-2.5 text-center`, headerClassName: `${HEADER_CLASS} text-center` },
      cell: ({ row }) => <SegmentTypeCell row={row.original} t={t} />,
    },
    {
      id: "segment_number",
      header: t("reports.table.segment_number"),
      size: 60,
      meta: { cellClassName: `${CELL_BASE} px-1 py-2.5 text-center`, headerClassName: `${HEADER_CLASS} text-center` },
      cell: ({ row }) => <SegmentCounterCell row={row.original} />,
    },
    {
      id: "scheduled",
      header: t("reports.table.scheduled"),
      size: 200,
      meta: { cellClassName: `${CELL_BASE} px-4 py-2.5`, headerClassName: HEADER_CLASS },
      cell: ({ row }) => <ScheduledCell row={row.original} t={t} />,
    },
    {
      id: "punch_in",
      header: t("common.in"),
      size: 140,
      meta: { cellClassName: `${CELL_BASE} ${PUNCH_TIME_CELL_CLASS}`, headerClassName: `${HEADER_CLASS} text-center` },
      cell: ({ row }) => (
        <PunchTimeCellContent
          formatTime={formatTime}
          hasMultipleSessions={hasMultipleSessions(row.original)}
          side="in"
          time={row.original.actual_punch_in}
          timeEntries={timeEntriesOf(row.original)}
        />
      ),
    },
    {
      id: "punch_out",
      header: t("common.out"),
      size: 140,
      meta: { cellClassName: `${CELL_BASE} ${PUNCH_TIME_CELL_CLASS}`, headerClassName: `${HEADER_CLASS} text-center` },
      cell: ({ row }) => (
        <PunchTimeCellContent
          formatTime={formatTime}
          hasMultipleSessions={false}
          side="out"
          time={row.original.actual_punch_out}
          timeEntries={timeEntriesOf(row.original)}
        />
      ),
    },
    {
      id: "std_hours",
      header: t("reports.table.std_hours"),
      size: 110,
      meta: { cellClassName: `${CELL_BASE} px-4 py-2.5 text-right`, headerClassName: `${HEADER_CLASS} text-right` },
      cell: ({ row }) => (
        <div
          className="font-mono font-bold text-primary-600 text-base tracking-tighter leading-none cursor-help"
          title={buildWorkUnitsTooltip(
            row.original.std_work_units,
            t(getUnitTypeI18nKey(row.original.unit_type)),
            row.original.unit_coefficient_hours,
            row.original.computed_std_hours,
          )}
        >
          {formatHours(row.original.computed_std_hours)}
          <span className="text-[10px] opacity-70 ml-0.5">h</span>
        </div>
      ),
    },
    {
      id: "ot_hours",
      header: t("reports.table.ot_hours"),
      size: 140,
      meta: {
        cellClassName: `${CELL_BASE} px-4 py-2.5 text-right`,
        headerClassName: "px-4 py-2.5 text-[10px] font-bold text-orange-500 dark:text-orange-400 uppercase tracking-widest text-right",
      },
      cell: ({ row }) => <OtCell onApproveOt={onApproveOt} row={row.original} t={t} />,
    },
    {
      id: "status",
      header: t("common.status"),
      size: 140,
      meta: { cellClassName: `${CELL_BASE} px-4 py-2.5 text-center`, headerClassName: `${HEADER_CLASS} text-center` },
      cell: ({ row }) => (
        <ReportStatusBadge
          onResolveUnscheduled={onResolveUnscheduled}
          row={row.original}
          status={row.original.status}
          t={t}
        />
      ),
    },
  );

  if (isDebugMode) {
    columns.push({
      id: "debug",
      header: "DEBUG",
      size: 90,
      meta: {
        cellClassName: `${CELL_BASE} px-4 py-2.5 text-center`,
        headerClassName: "px-4 py-2.5 text-[10px] font-bold text-amber-600 dark:text-amber-300 uppercase tracking-widest text-center",
      },
      cell: ({ row }) => {
        const key = segmentKeyOf(row.original);
        const isExpanded = expandedDebugRows.has(key);
        return (
          <Button
            className={cn(
              "p-1.5 rounded-lg transition-all shadow-sm border h-auto",
              isExpanded ? "bg-amber-100 text-amber-700 border-amber-200" : "bg-white text-amber-600 border-slate-100 hover:bg-amber-50",
            )}
            onClick={() => { onToggleExpandRow(key); }}
            variant="ghost"
          >
            <Bug size={16} />
          </Button>
        );
      },
    });
  }

  columns.push({
    id: "actions",
    header: "",
    size: 120,
    meta: { cellClassName: `${CELL_BASE} px-4 py-2.5 text-right`, headerClassName: "px-4 py-2.5" },
    cell: ({ row }) => {
      const dayKey = `${row.original.employment_id}-${row.original.date}`;
      const dayKeyEmployee = dayGroupKeyOf(row.original);
      return (
        <RowActions
          existingAdjustment={adjustmentMap[dayKey]}
          isConfirming={confirmingRow === dayKeyEmployee}
          isReprocessing={reprocessingRow === dayKeyEmployee}
          onConfirmClick={onConfirmClick}
          onReprocessClick={onReprocessClick}
          row={row.original}
          t={t}
        />
      );
    },
  });

  return columns;
}
