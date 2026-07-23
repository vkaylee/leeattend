import {
  Building2,
  UserX,
} from "lucide-react";
import { useCallback } from "react";

import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import { ScheduleTracePopover } from "@/components/shifts/schedule-trace-popover";
import ResponsiveTable from "@/components/ui/responsive-table";
import { ScheduleTrace } from "@/lib/api";
import { cn } from "@/lib/utils";

import type { ZonedNoShowRow } from "../../hooks/use-no-show-rows";

import { formatShiftTime, zoneLabel, ZoneIcon } from "./utils";

interface NoShowTableProps {
  filteredRows: ZonedNoShowRow[];
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  loadMoreRef: React.RefObject<HTMLTableRowElement | null>;
  onlyZoneB: boolean;
  t: (k: string, o?: Record<string, unknown>) => string;
  tracesByKey: Map<string, ScheduleTrace[]>;
}

function TableSkeleton() {
  return (
    <div className="space-y-2 animate-pulse">
      {Array.from({ length: 6 }).map((_, i) => (
        <div
          className="h-14 rounded-xl bg-slate-100 dark:bg-slate-800"
          key={i}
        />
      ))}
    </div>
  );
}

function EmptyStateRow({ empty, hint }: { empty: string; hint: string }) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-20 text-center">
      <div className="w-16 h-16 rounded-2xl bg-emerald-50 dark:bg-emerald-950 flex items-center justify-center shadow-inner">
        <UserX className="text-emerald-500 dark:text-emerald-400" size={28} />
      </div>
      <p className="font-bold text-sm text-slate-700 dark:text-slate-200">{empty}</p>
      <p className="text-xs text-slate-400 dark:text-slate-500 max-w-xs">{hint}</p>
    </div>
  );
}

function zoneRowBg(zone: string): string {
  switch (zone) {
    case "b": return "bg-amber-50/60 dark:bg-amber-950/25 border-l-2 border-l-amber-400 dark:border-l-amber-500";
    case "a": return "bg-slate-50/60 dark:bg-slate-800/40";
    default: return "bg-rose-50/40 dark:bg-rose-950/15";
  }
}

function zoneChipBg(zone: string): string {
  return zone === "b"
    ? "bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300"
    : zone === "a"
      ? "bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400"
      : "bg-rose-100 dark:bg-rose-900/40 text-rose-700 dark:text-rose-300";
}

function NoShowMobileCard({
  row,
  columns,
  tracesByKey,
  t,
}: {
  row: ZonedNoShowRow;
  columns: Record<string, string>;
  tracesByKey: Map<string, ScheduleTrace[]>;
  t: (k: string, o?: Record<string, unknown>) => string;
}) {
  const { key } = zoneLabel(row, t);
  return (
    <div className={cn(
      "rounded-2xl border bg-white p-5 shadow-sm dark:bg-slate-900 border-slate-100 dark:border-slate-800",
      row.zone !== "a" && "border-l-4",
      row.zone === "b" && "border-l-amber-400 dark:border-l-amber-500 bg-amber-50/60 dark:bg-amber-950/25",
      row.zone === "c" && "border-l-rose-400 dark:border-l-rose-500 bg-rose-50/40 dark:bg-rose-950/15",
    )}>
      {/* Header: Zone + Employee code */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3 min-w-0">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-rose-100 dark:bg-rose-900/40">
            <UserX className="text-rose-500 dark:text-rose-400" size={18} />
          </div>
          <div className="min-w-0">
            <div className="text-sm font-bold text-slate-800 dark:text-slate-100 tracking-tight">
              {row.employee_name}
            </div>
            <div className="mt-0.5 font-mono text-[10px] font-bold uppercase tracking-tight text-primary-600 dark:text-primary-400">
              {row.employee_id_number}
            </div>
          </div>
        </div>
        <span className={cn(
          "inline-flex shrink-0 items-center gap-1.5 px-2 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest",
          zoneChipBg(row.zone),
        )}>
          <ZoneIcon zone={row.zone} />
          <span>{key}</span>
        </span>
      </div>

      {/* Detail grid */}
      <div className="mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.department}</div>
          <div className="mt-1 flex items-center gap-1.5">
            <Building2 className="text-blue-500" size={10} />
            <span className="text-[10px] font-bold uppercase tracking-wider text-blue-700 dark:text-blue-300">
              {row.department_name}
            </span>
          </div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.shift_type}</div>
          <div className="mt-1 flex items-center gap-1">
            <span className="text-xs font-bold text-slate-700 dark:text-slate-200 tracking-tight">
              {row.shift_name ?? "—"}
            </span>
            <ScheduleTracePopover
              endTime={row.scheduled_end}
              shiftName={row.shift_name}
              shiftSymbol={row.shift_symbol}
              source="system"
              startTime={row.scheduled_start}
              traces={tracesByKey.get(`${row.employment_id}-${row.date}`) ?? []}
            />
          </div>
        </div>
      </div>

      {/* Shift times */}
      <div className="mt-3 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.shift_in}</div>
          <div className="mt-1 font-mono text-sm font-bold tabular-nums text-slate-700 dark:text-slate-200">
            {formatShiftTime(row.scheduled_start)}
          </div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.shift_out}</div>
          <div className="mt-1 font-mono text-sm font-bold tabular-nums text-slate-700 dark:text-slate-200">
            {formatShiftTime(row.scheduled_end)}
          </div>
        </div>
      </div>
    </div>
  );
}

function NoShowDesktopRow({
  row,
  idx,
  columns,
  tracesByKey,
  t,
}: {
  row: ZonedNoShowRow;
  idx: number;
  columns: Record<string, string>;
  tracesByKey: Map<string, ScheduleTrace[]>;
  t: (k: string, o?: Record<string, unknown>) => string;
}) {
  const { key, hint } = zoneLabel(row, t);
  return (
    <tr
      className={cn(
        "transition-colors group",
        zoneRowBg(row.zone),
        row.zone !== "b" && "hover:bg-rose-50/40 dark:hover:bg-rose-950/20",
        idx % 2 === 0 ? "" : "bg-opacity-50",
      )}
    >
      <td className="px-5 py-3.5">
        <span
          className={cn(
            "inline-flex items-center gap-1.5 px-2 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest",
            zoneChipBg(row.zone),
          )}
        >
          <ZoneIcon zone={row.zone} />
          <span>{key}</span>
          {hint && <span className="opacity-70 normal-case tracking-normal">· {hint}</span>}
          <span className="sr-only">
            {key}{hint ? `, ${hint}` : ""}
          </span>
        </span>
      </td>
      <td className="px-5 py-3.5">
        <span className="font-mono font-bold text-[10px] text-primary-600 dark:text-primary-400 uppercase tracking-tight bg-primary-50 dark:bg-primary-950 px-2 py-1 rounded-lg border border-primary-100 dark:border-primary-800">
          {row.employee_id_number}
        </span>
      </td>
      <td className="px-5 py-3.5">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-xl bg-rose-100 dark:bg-rose-900/40 flex items-center justify-center shrink-0">
            <UserX className="text-rose-500 dark:text-rose-400" size={14} />
          </div>
          <span className="font-bold text-sm text-slate-800 dark:text-slate-100 tracking-tight">
            {row.employee_name}
          </span>
        </div>
      </td>
      <td className="px-5 py-3.5">
        <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 rounded-xl text-[10px] font-bold uppercase tracking-widest border border-blue-100 dark:border-blue-800">
          <Building2 size={10} />
          {row.department_name}
        </span>
      </td>
      <td className="px-5 py-3.5">
        <div className="relative flex items-center gap-1 w-fit">
          <span className="font-bold text-slate-700 dark:text-slate-200 text-xs tracking-tight">
            {row.shift_name ?? "—"}
          </span>
          <ScheduleTracePopover
            endTime={row.scheduled_end}
            shiftName={row.shift_name}
            shiftSymbol={row.shift_symbol}
            source="system"
            startTime={row.scheduled_start}
            traces={tracesByKey.get(`${row.employment_id}-${row.date}`) ?? []}
          />
        </div>
      </td>
      <td className="px-5 py-3.5 text-center">
        <span className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200 tabular-nums bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg">
          {formatShiftTime(row.scheduled_start)}
        </span>
      </td>
      <td className="px-5 py-3.5 text-center">
        <span className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200 tabular-nums bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg">
          {formatShiftTime(row.scheduled_end)}
        </span>
      </td>
    </tr>
  );
}

/**
 * Renders the no-show report table with zone-aware row styling,
 * employee details, shift schedule, and infinite-scroll loading sentinel.
 */
export function NoShowTable({
  filteredRows,
  hasNextPage,
  isFetchingNextPage,
  loadMoreRef,
  tracesByKey,
  onlyZoneB,
  t,
}: NoShowTableProps) {
  const columns = {
    zone: t("unrecorded_shifts.zone_header"),
    employee_code: t("unrecorded_shifts.columns.employee_code"),
    employee_name: t("unrecorded_shifts.columns.employee_name"),
    department: t("unrecorded_shifts.columns.department"),
    shift_type: t("unrecorded_shifts.columns.shift_type"),
    shift_in: t("unrecorded_shifts.columns.shift_in"),
    shift_out: t("unrecorded_shifts.columns.shift_out"),
  };

  if (filteredRows.length === 0) {
    return (
      <EmptyStateRow
        empty={
          onlyZoneB
            ? t("unrecorded_shifts.empty_zone_b")
            : t("unrecorded_shifts.empty")
        }
        hint={
          onlyZoneB
            ? t("unrecorded_shifts.empty_zone_b_hint")
            : t("unrecorded_shifts.empty_hint")
        }
      />
    );
  }

  const renderCard = useCallback((row: ZonedNoShowRow) => (
    <NoShowMobileCard columns={columns} row={row} t={t} tracesByKey={tracesByKey} />
  ), [columns, t, tracesByKey]);

  const renderDesktopTable = useCallback(() => (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800/60 border-b border-slate-200 dark:border-slate-700">
            {Object.entries(columns).map(([k, label]) => (
              <th
                className={cn(
                  "px-5 py-3 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap",
                  (k === "shift_in" || k === "shift_out") ? "text-center" : "text-left",
                )}
                key={k}
              >
                {label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
          {filteredRows.map((row, idx) => (
            <NoShowDesktopRow
              columns={columns}
              idx={idx}
              key={`${row.employment_id}-${row.date}`}
              row={row}
              t={t}
              tracesByKey={tracesByKey}
            />
          ))}
          {hasNextPage && (
            <InfiniteScrollSentinel
              colSpan={7}
              isFetchingNextPage={isFetchingNextPage}
              sentinelRef={loadMoreRef}
            />
          )}
        </tbody>
      </table>
    </div>
  ), [filteredRows, columns, t, tracesByKey, hasNextPage, isFetchingNextPage, loadMoreRef]);

  return (
    <div className="rounded-2xl border border-slate-200 dark:border-slate-700 overflow-hidden shadow-sm">
      <ResponsiveTable<ZonedNoShowRow>
        data={filteredRows}
        getCardKey={(row) => `${row.employment_id}-${row.date}`}
        isLoading={false}
        mobileFooter={
          hasNextPage ? (
            <InfiniteScrollSentinel
              colSpan={7}
              isFetchingNextPage={isFetchingNextPage}
              sentinelRef={loadMoreRef}
            />
          ) : undefined
        }
        renderCard={renderCard}
        renderDesktopTable={renderDesktopTable}
      />
    </div>
  );
}
